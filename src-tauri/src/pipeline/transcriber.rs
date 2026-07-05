use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tokio::task::JoinHandle;
use tauri::Emitter;
use tracing::{info, warn};

use crate::audio::buffer::RingBuffer;
use crate::audio::capture::SAMPLES_PUSHED;
use crate::settings::config::AppConfig;
use crate::whisper::engine::WhisperEngine;
use crate::whisper::params::TranscriptionParams;

/// 3 seconds at 16kHz — balance between speed and whisper accuracy
const CHUNK_SAMPLES: usize = 48000;
/// Max buffer before dropping old audio (8 seconds)
/// If buffer exceeds this, we drop oldest audio to stay near real-time
const MAX_BUFFER_SAMPLES: usize = 128000;

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
        app_handle: tauri::AppHandle,
        config: AppConfig,
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

        info!("Pipeline starting: language={}, threads={}, gpu={}", language, threads, gpu);

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
                        tokio::time::sleep(Duration::from_millis(100)).await;
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
                    // Emit lag warning
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
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    continue;
                }

                // Take exactly CHUNK_SAMPLES — lock, take, drop immediately
                let audio_chunk = match buffer.lock().ok() {
                    Some(mut buf) => buf.take(CHUNK_SAMPLES),
                    None => continue,
                };

                if audio_chunk.is_empty() {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    continue;
                }

                // Transcribe — lock engine, transcribe, drop immediately
                let start_time = std::time::Instant::now();
                let segments = match engine.lock().ok() {
                    Some(eng) => {
                        let params = TranscriptionParams {
                            language: Some(language.clone()),
                            threads,
                            gpu,
                            translate: false,
                        };
                        match eng.transcribe(&audio_chunk, &params) {
                            Ok(s) => s,
                            Err(e) => {
                                warn!("Transcription error: {}", e);
                                continue;
                            }
                        }
                    }
                    None => continue,
                };

                let elapsed_ms = start_time.elapsed().as_millis();
                let chunk_seconds = audio_chunk.len() as f64 / 16000.0;
                let speed_ratio = chunk_seconds * 1000.0 / elapsed_ms as f64;

                if !segments.is_empty() {
                    let text: String = segments
                        .iter()
                        .map(|s| s.text.trim())
                        .collect::<Vec<_>>()
                        .join(" ");

                    info!(
                        "Transcription ({}ms, {:.1}s chunk, {:.1}x speed): {}",
                        elapsed_ms, chunk_seconds, speed_ratio, text
                    );

                    // Emit to frontend
                    let _ = app_handle.emit("transcription", serde_json::json!({
                        "text": text,
                        "start": segments.first().map(|s| s.start).unwrap_or(0.0),
                        "end": segments.last().map(|s| s.end).unwrap_or(0.0),
                        "speed_ratio": speed_ratio,
                    }));
                } else {
                    info!(
                        "Silent chunk ({:.1}s, {}ms)",
                        chunk_seconds, elapsed_ms
                    );
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
