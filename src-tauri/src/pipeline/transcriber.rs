use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tokio::task::JoinHandle;
use tauri::Emitter;
use tracing::{info, warn};

use crate::audio::buffer::RingBuffer;
use crate::audio::capture::SAMPLES_PUSHED;
use crate::history::HistoryDb;
use crate::settings::config::AppConfig;
use crate::whisper::engine::WhisperEngine;
use crate::whisper::params::TranscriptionParams;
use crate::translation::marian::MarianEngine;

/// 1.5 seconds at 16kHz — smaller chunks for faster iteration
const CHUNK_SAMPLES: usize = 24000;
/// Max buffer before dropping old audio (6 seconds)
const MAX_BUFFER_SAMPLES: usize = 96000;

pub struct TranscriptionPipeline {
    running: Arc<AtomicBool>,
    task_handle: Option<JoinHandle<()>>,
}

impl TranscriptionPipeline {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            task_handle: None,
        }
    }

    pub fn start(
        &mut self,
        buffer: Arc<Mutex<RingBuffer>>,
        engine: Arc<Mutex<WhisperEngine>>,
        history_db: Arc<Mutex<HistoryDb>>,
        app_handle: tauri::AppHandle,
        config: AppConfig,
        marian_engine: Arc<Mutex<MarianEngine>>,
    ) {
        if self.running.load(Ordering::Relaxed) {
            info!("Pipeline already running");
            return;
        }

        self.running.store(true, Ordering::Relaxed);
        let running = self.running.clone();

        let language = config.whisper.language.clone();
        let threads = config.whisper.threads;
        let gpu = config.whisper.gpu;
        let translation_enabled = config.translation.enabled;
        let source_lang = config.translation.source_lang.clone();
        let target_lang = config.translation.target_lang.clone();

        info!("Pipeline starting: language={}, threads={}, gpu={}, translation_enabled={}",
            language, threads, gpu, translation_enabled);

        let handle = tokio::spawn(async move {
            let mut last_samples_pushed: u64 = 0;

            loop {
                if !running.load(Ordering::Relaxed) {
                    info!("Pipeline stopping");
                    break;
                }

                // Log capture stats every 10 seconds
                let total_samples = SAMPLES_PUSHED.load(Ordering::Relaxed);
                if total_samples != last_samples_pushed {
                    let delta = total_samples - last_samples_pushed;
                    if delta > 0 {
                        info!(
                            "Capture: {} total ({:.1}s), Δ{} ({:.1}s) this cycle",
                            total_samples,
                            total_samples as f64 / 16000.0,
                            delta,
                            delta as f64 / 16000.0,
                        );
                    }
                    last_samples_pushed = total_samples;
                }

                // Check buffer — drop old audio if falling behind
                let buffer_snapshot = buffer.lock().ok().map(|buf| {
                    let len = buf.len();
                    let excess = if len > MAX_BUFFER_SAMPLES { len - CHUNK_SAMPLES } else { 0 };
                    (len, excess)
                });
                let (samples_available, buffer_excess) = match buffer_snapshot {
                    Some(v) => v,
                    None => {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        continue;
                    }
                };

                // Drop excess audio to stay near real-time
                if buffer_excess > 0 {
                    info!(
                        "Dropping {} samples ({:.1}s) to stay near real-time (buffer was {} samples)",
                        buffer_excess,
                        buffer_excess as f64 / 16000.0,
                        samples_available
                    );
                    if let Ok(mut buf) = buffer.lock() {
                        buf.drain_to(buffer_excess);
                    }
                    let _ = app_handle.emit("pipeline-status", serde_json::json!({
                        "status": "dropping_audio",
                        "dropped_seconds": buffer_excess as f64 / 16000.0,
                    }));
                }

                // Check if we have a full chunk
                let available = match buffer.lock().ok() {
                    Some(buf) => buf.len(),
                    None => continue,
                };

                if available < CHUNK_SAMPLES {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // Take exactly CHUNK_SAMPLES
                let audio_chunk = match buffer.lock().ok() {
                    Some(mut buf) => buf.take(CHUNK_SAMPLES),
                    None => continue,
                };

                if audio_chunk.is_empty() {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // Transcribe — do NOT hold any other lock while transcribing
                let start_time = std::time::Instant::now();
                let segments = {
                    let eng = match engine.lock() {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    let params = TranscriptionParams {
                        language: Some(language.clone()),
                        threads,
                        gpu,
                        translate: false,
                        ..Default::default()
                    };
                    match eng.transcribe(&audio_chunk, &params) {
                        Ok(s) => s,
                        Err(e) => {
                            warn!("Transcription error: {}", e);
                            continue;
                        }
                    }
                };

                let elapsed_ms = start_time.elapsed().as_millis();
                let chunk_seconds = audio_chunk.len() as f64 / 16000.0;
                let speed_ratio = chunk_seconds * 1000.0 / elapsed_ms as f64;

                if segments.is_empty() {
                    info!("Silent chunk ({:.1}s, {}ms)", chunk_seconds, elapsed_ms);
                    continue;
                }

                let text: String = segments
                    .iter()
                    .map(|s| s.text.trim())
                    .filter(|s| !s.is_empty() && *s != "[BLANK_AUDIO]")
                    .collect::<Vec<_>>()
                    .join(" ");

                if text.is_empty() {
                    info!("Empty after filter ({:.1}s, {}ms)", chunk_seconds, elapsed_ms);
                    continue;
                }

                info!(
                    "Transcription ({}ms, {:.1}s chunk, {:.1}x speed): {}",
                    elapsed_ms, chunk_seconds, speed_ratio, text
                );

                // Emit transcription immediately (without translation)
                let _ = app_handle.emit("transcription", serde_json::json!({
                    "id": chrono::Utc::now().timestamp_millis(),
                    "text": text,
                    "translation": serde_json::Value::Null,
                    "start": segments.first().map(|s| s.start).unwrap_or(0.0),
                    "end": segments.last().map(|s| s.end).unwrap_or(0.0),
                    "speed_ratio": speed_ratio,
                }));

                // Store in history immediately (with translation=null)
                {
                    if let Ok(db) = history_db.lock() {
                        if let Err(e) = db.insert(&language, &text, None, None) {
                            warn!("Failed to insert history: {}", e);
                        }
                    }
                }

                // Fire-and-forget translation — does NOT block the loop
                if translation_enabled
                    && MarianEngine::is_supported(&source_lang, &target_lang)
                {
                    let src = source_lang.clone();
                    let tgt = target_lang.clone();
                    let text_clone = text.clone();
                    let engine_clone = marian_engine.clone();
                    let app_clone = app_handle.clone();
                    let text_for_emit = text.clone();

                    info!("Translating {}→{}: \"{}\"", src, tgt, text_clone);

                    tokio::spawn(async move {
                        let translation = {
                            let mut eng = match engine_clone.lock() {
                                Ok(e) => e,
                                Err(e) => {
                                    warn!("Engine lock poisoned: {}", e);
                                    return;
                                }
                            };
                            if let Err(e) = eng.load(&src, &tgt) {
                                warn!("Model load failed ({}→{}): {}", src, tgt, e);
                                return;
                            }
                            match eng.translate(&text_clone) {
                                Ok(t) => {
                                    info!("Translation ({}→{}): {}", src, tgt, t);
                                    t
                                }
                                Err(e) => {
                                    warn!("Translation failed ({}→{}): {}", src, tgt, e);
                                    return;
                                }
                            }
                        };

                        // Emit translation result to frontend
                        let _ = app_clone.emit("translation-result", serde_json::json!({
                            "text": text_for_emit,
                            "translation": translation,
                        }));
                    });
                }
            }

            info!("Pipeline task ended");
        });

        self.task_handle = Some(handle);
    }

    pub fn stop(&mut self) {
        info!("Stopping transcription pipeline");
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}
