use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};
use tracing::{info, warn};

use crate::asr::engine::{EngineKind, TranscriptionSegment};
use crate::asr::AsrEngine;
use crate::commands::export::{export_entries, ExportEntry, ExportFormat};
use crate::diarization::engine::{DiarizationEngine, SpeakerTurn};
use crate::history::HistoryDb;
use crate::settings::AppConfig;
use crate::transcription::merge::fusionar_segmentos;
use crate::vad::detector::VadDetector;
use crate::video::processor::VideoProcessor;
use crate::whisper::params::TranscriptionParams;

/// WAV extraction is always 16 kHz mono
/// (see VideoProcessor::extract_audio).
pub const SAMPLE_RATE: usize = 16000;
/// 0.3 s of padding on each side of a VAD speech window.
pub const VAD_PAD_SAMPLES: usize = 4800;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizedSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub speaker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTranscriptionResult {
    pub id: i64,
    pub video_name: String,
    pub segments: Vec<DiarizedSegment>,
    pub full_text: String,
    pub translated_text: Option<String>,
    pub target_language: Option<String>,
    pub duration_seconds: f64,
    pub diarization_status: String,
    pub speaker_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTranscriptionEntry {
    pub id: i64,
    pub video_path: String,
    pub video_name: String,
    pub duration_seconds: Option<f64>,
    pub language: String,
    pub full_text: String,
    pub translated_text: Option<String>,
    pub target_language: Option<String>,
    pub summary: Option<String>,
    pub segments: Vec<DiarizedSegment>,
    pub created_at: String,
    pub diarization_status: String,
    pub speaker_count: u32,
}

pub struct VideoTranscriptionState {
    pub db: Arc<Mutex<HistoryDb>>,
    pub asr: Arc<Mutex<AsrEngine>>,
    pub diarization: Arc<Mutex<DiarizationEngine>>,
}

#[tauri::command]
pub async fn transcribe_video(
    video_path: String,
    language: Option<String>,
    target_language: Option<String>,
    enable_diarization: bool,
    state: State<'_, VideoTranscriptionState>,
    ai_config_state: State<'_, Arc<Mutex<crate::ai::config::AiConfig>>>,
    config_state: State<'_, Arc<Mutex<AppConfig>>>,
    app_handle: tauri::AppHandle,
) -> Result<VideoTranscriptionResult, String> {
    let path = PathBuf::from(&video_path);
    if !path.exists() {
        return Err(format!("Video file not found: {}", video_path));
    }

    let video_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Step 1: Extract audio
    let _ = app_handle.emit(
        "video-transcription-progress",
        serde_json::json!({
            "step": "extracting",
            "progress": 0.0,
            "message": "Extracting audio from video...",
        }),
    );

    let audio_path = VideoProcessor::extract_audio(&path)
        .await
        .map_err(|e| format!("Failed to extract audio: {}", e))?;

    let _ = app_handle.emit(
        "video-transcription-progress",
        serde_json::json!({
            "step": "extracting",
            "progress": 1.0,
            "message": "Audio extracted successfully",
        }),
    );

    // Step 2: Get duration
    let duration = VideoProcessor::get_duration(&path).await.unwrap_or(0.0);

    // Step 3: Diarization (if enabled)
    let (speaker_turns, diarization_status, speaker_count) = if enable_diarization {
        let _ = app_handle.emit(
            "video-transcription-progress",
            serde_json::json!({
                "step": "diarizing",
                "progress": 0.0,
                "message": "Detecting speakers...",
            }),
        );

        let diarization = state.diarization.lock().unwrap();
        match diarization.diarize(&audio_path) {
            Ok(turns) => {
                let distinct = turns
                    .iter()
                    .map(|t| &t.speaker)
                    .collect::<std::collections::HashSet<_>>()
                    .len() as u32;
                let _ = app_handle.emit(
                    "video-transcription-progress",
                    serde_json::json!({
                        "step": "diarizing",
                        "progress": 1.0,
                        "message": format!("Detected {} speaker(s)", distinct),
                    }),
                );
                (Some(turns), "succeeded".to_string(), distinct)
            }
            Err(e) => {
                warn!("Diarization failed: {}", e);
                let _ = app_handle.emit(
                    "video-transcription-progress",
                    serde_json::json!({
                        "step": "diarizing",
                        "progress": 1.0,
                        "message": "Diarization failed, continuing without speaker labels",
                    }),
                );
                (None, "failed".to_string(), 0)
            }
        }
    } else {
        (None, "disabled".to_string(), 0)
    };

    // Step 4: Transcribe with ASR engine
    let _ = app_handle.emit(
        "video-transcription-progress",
        serde_json::json!({
            "step": "transcribing",
            "progress": 0.0,
            "message": "Transcribing audio...",
        }),
    );

    // Read audio file to f32 samples
    let audio_data =
        read_wav_to_f32(&audio_path).map_err(|e| format!("Failed to read audio: {}", e))?;

    // Build transcription params from the app config (clone inside the lock,
    // drop the guard before any heavy work).
    let (threads, gpu, beam_size, compute_type, vad_threshold) = {
        let cfg = config_state.lock().map_err(|e| e.to_string())?;
        (
            cfg.whisper.threads,
            cfg.whisper.gpu,
            cfg.whisper.beam_size,
            cfg.whisper.compute_type.clone(),
            cfg.audio.vad_threshold as f32,
        )
    };

    let params = TranscriptionParams {
        language: language.clone(),
        threads,
        gpu,
        beam_size,
        compute_type,
        translate: false,
    };

    let segments = {
        let mut engine = state.asr.lock().unwrap();
        if !engine.is_loaded() {
            return Err("ASR model not loaded".to_string());
        }

        if engine.kind() == EngineKind::Ctranslate2 {
            // CTranslate2 gets VAD-based chunking (windows + progress events).
            transcribe_with_vad(
                &mut engine,
                &audio_data,
                SAMPLE_RATE,
                &params,
                vad_threshold,
                &app_handle,
            )
            .map_err(|e| format!("Transcription failed: {}", e))?
        } else {
            // whisper.cpp / sherpa keep the single-pass transcription.
            engine
                .transcribe(&audio_data, &params)
                .map_err(|e| format!("Transcription failed: {}", e))?
        }
    };

    let _ = app_handle.emit(
        "video-transcription-progress",
        serde_json::json!({
            "step": "transcribing",
            "progress": 1.0,
            "message": "Transcription complete",
        }),
    );

    // Step 5: Merge segments with speaker turns
    let _ = app_handle.emit(
        "video-transcription-progress",
        serde_json::json!({
            "step": "merging",
            "progress": 0.0,
            "message": "Processing segments...",
        }),
    );

    // Fusion de segmentos en bloques naturales (port de transcribir.py)
    let fused = fusionar_segmentos(&segments, 15.0);

    let blocks: Vec<(f64, f64)> = fused.iter().map(|s| (s.start, s.end)).collect();
    let speakers = assign_speakers(&blocks, speaker_turns.as_deref().unwrap_or(&[]));

    let diarized_segments: Vec<DiarizedSegment> = fused
        .iter()
        .zip(speakers)
        .map(|(seg, speaker)| DiarizedSegment {
            start: seg.start,
            end: seg.end,
            text: seg.text.clone(),
            speaker,
        })
        .collect();

    let full_text: String = diarized_segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // Step 5.5: Translate if target_language is specified
    let translated_text = if let Some(ref target_lang) = target_language {
        if target_lang != "none" && target_lang != language.as_deref().unwrap_or("auto") {
            let _ = app_handle.emit(
                "video-transcription-progress",
                serde_json::json!({
                    "step": "translating",
                    "progress": 0.0,
                    "message": format!("Translating to {}...", target_lang),
                }),
            );

            let ai_config = {
                let cfg = ai_config_state.lock().unwrap();
                cfg.clone()
            };

            let lang_name = match target_lang.as_str() {
                "es" | "Spanish" => "Spanish",
                "en" | "English" => "English",
                "fr" | "French" => "French",
                "de" | "German" => "German",
                "pt" | "Portuguese" => "Portuguese",
                "it" | "Italian" => "Italian",
                "ja" | "Japanese" => "Japanese",
                "zh" | "Chinese" => "Chinese",
                "ko" | "Korean" => "Korean",
                "ru" | "Russian" => "Russian",
                "ar" | "Arabic" => "Arabic",
                _ => target_lang,
            };

            let system_prompt = format!(
                "You are a professional translator. Translate the following text to {}. \
                 Preserve the original meaning, tone, and formatting. \
                 Only output the translation, nothing else.",
                lang_name
            );

            let messages = vec![crate::ai::config::ChatMessage::user(&full_text)];

            let provider = crate::ai::provider::create_provider(ai_config);
            match provider.chat(&system_prompt, &messages).await {
                Ok(translated) => {
                    let _ = app_handle.emit(
                        "video-transcription-progress",
                        serde_json::json!({
                            "step": "translating",
                            "progress": 1.0,
                            "message": "Translation complete",
                        }),
                    );
                    Some(translated)
                }
                Err(e) => {
                    warn!("Translation failed: {}", e);
                    let _ = app_handle.emit(
                        "video-transcription-progress",
                        serde_json::json!({
                            "step": "translating",
                            "progress": 1.0,
                            "message": "Translation failed, using original text",
                        }),
                    );
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // Step 6: Save to database
    let _ = app_handle.emit(
        "video-transcription-progress",
        serde_json::json!({
            "step": "saving",
            "progress": 0.0,
            "message": "Saving transcription...",
        }),
    );

    let segments_json = serde_json::to_string(&diarized_segments)
        .map_err(|e| format!("Failed to serialize segments: {}", e))?;

    let lang = language.unwrap_or_else(|| "auto".to_string());
    let tgt_lang = target_language.clone();
    let trans_text = translated_text.clone();
    let id = {
        let db = state.db.lock().unwrap();
        db.insert_video_transcription(
            &video_path,
            &video_name,
            duration,
            &lang,
            &full_text,
            trans_text.as_deref(),
            tgt_lang.as_deref(),
            &segments_json,
            &diarization_status,
            speaker_count,
        )
        .map_err(|e| format!("Failed to save transcription: {}", e))?
    };

    // Clean up temp WAV file
    let _ = tokio::fs::remove_file(&audio_path).await;

    let done_message = match diarization_status.as_str() {
        "succeeded" => format!(
            "Transcription complete — {} speaker(s) detected",
            speaker_count
        ),
        "failed" => "Transcription complete — Speaker detection failed".to_string(),
        _ => "Transcription complete".to_string(),
    };

    let _ = app_handle.emit(
        "video-transcription-progress",
        serde_json::json!({
            "step": "done",
            "progress": 1.0,
            "message": done_message,
        }),
    );

    info!(
        "Video transcription complete: {} segments, {:.1}s duration",
        diarized_segments.len(),
        duration
    );

    Ok(VideoTranscriptionResult {
        id,
        video_name,
        segments: diarized_segments,
        full_text,
        translated_text,
        target_language,
        duration_seconds: duration,
        diarization_status,
        speaker_count,
    })
}

#[tauri::command]
pub async fn list_video_transcriptions(
    limit: Option<i64>,
    state: State<'_, VideoTranscriptionState>,
) -> Result<Vec<VideoTranscriptionEntry>, String> {
    let db = state.db.lock().unwrap();
    let entries = db
        .get_video_transcriptions(limit.unwrap_or(50))
        .map_err(|e| format!("Failed to list transcriptions: {}", e))?;
    Ok(entries)
}

#[tauri::command]
pub async fn delete_video_transcription(
    id: i64,
    state: State<'_, VideoTranscriptionState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.delete_video_transcription(id)
        .map_err(|e| format!("Failed to delete transcription: {}", e))
}

#[tauri::command]
pub async fn get_video_transcription(
    id: i64,
    state: State<'_, VideoTranscriptionState>,
) -> Result<VideoTranscriptionEntry, String> {
    let db = state.db.lock().unwrap();
    db.get_video_transcription(id)
        .map_err(|e| format!("Failed to get transcription: {}", e))
}

#[tauri::command]
pub async fn update_video_transcription_summary(
    id: i64,
    summary: String,
    state: State<'_, VideoTranscriptionState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.update_video_transcription_summary(id, &summary)
        .map_err(|e| format!("Failed to update summary: {}", e))
}

/// A contiguous region of speech within the full-length sample buffer
/// (sample indices, half-open `[start, end)`).
#[derive(Debug, Clone, PartialEq)]
pub struct SpeechWindow {
    pub start: usize,
    pub end: usize,
}

/// Energy-based VAD over ~1 s frames (VadDetector with a 2-frame dead band)
/// splits `samples` into speech windows. Whole-file silence → empty `Vec`.
pub fn detect_speech_windows(
    samples: &[f32],
    sample_rate: usize,
    threshold: f32,
) -> Vec<SpeechWindow> {
    if samples.is_empty() {
        return Vec::new();
    }

    let frame_size = sample_rate.max(1);
    let mut vad = VadDetector::new(threshold, frame_size);
    vad.set_max_silence_frames(2);

    let mut windows = Vec::new();
    let mut in_window = false;
    let mut window_start = 0usize;

    for (i, chunk) in samples.chunks(frame_size).enumerate() {
        let chunk_start = i * frame_size;
        let chunk_end = (chunk_start + chunk.len()).min(samples.len());
        let is_speaking = vad.detect(chunk).is_speaking;

        if is_speaking && !in_window {
            in_window = true;
            window_start = chunk_start;
        } else if !is_speaking && in_window {
            windows.push(SpeechWindow {
                start: window_start,
                end: chunk_end,
            });
            in_window = false;
        }
    }
    if in_window {
        windows.push(SpeechWindow {
            start: window_start,
            end: samples.len(),
        });
    }
    windows
}

/// Window bounds padded by 0.3 s on each side, clamped to the buffer.
pub fn window_chunk_bounds(
    window: &SpeechWindow,
    sample_rate: usize,
    total_len: usize,
) -> (usize, usize) {
    let pad = ((0.3 * sample_rate as f64) as usize).min(VAD_PAD_SAMPLES);
    let start = window.start.saturating_sub(pad);
    let end = (window.end + pad).min(total_len);
    (start, end)
}

/// Shift every segment by `delta` seconds into the absolute timeline.
pub fn offset_segments(segs: &[TranscriptionSegment], delta: f64) -> Vec<TranscriptionSegment> {
    segs.iter()
        .map(|s| TranscriptionSegment {
            start: s.start + delta,
            end: s.end + delta,
            text: s.text.clone(),
        })
        .collect()
}

/// Transcribe a long (full-video) buffer with VAD-based chunking. Every chunk
/// is padded by 0.3 s per side and its segments are offset back into the
/// absolute timeline. Emits one progress event per chunk. Used for
/// CTranslate2 only (whisper.cpp / sherpa keep single-pass).
fn transcribe_with_vad(
    engine: &mut AsrEngine,
    samples: &[f32],
    sample_rate: usize,
    params: &TranscriptionParams,
    vad_threshold: f32,
    app_handle: &tauri::AppHandle,
) -> anyhow::Result<Vec<TranscriptionSegment>> {
    let windows = detect_speech_windows(samples, sample_rate, vad_threshold);
    let total = windows.len().max(1);

    let mut all = Vec::new();
    for (i, window) in windows.iter().enumerate() {
        let (start, end) = window_chunk_bounds(window, sample_rate, samples.len());
        if start >= end {
            continue;
        }

        let chunk = &samples[start..end];
        let chunk_segs = engine.transcribe(chunk, params)?;

        let absolute_start = start as f64 / sample_rate as f64;
        all.extend(offset_segments(&chunk_segs, absolute_start));

        app_handle
            .emit(
                "video-transcription-progress",
                serde_json::json!({
                    "step": "transcribing",
                    "progress": (i + 1) as f64 / total as f64,
                    "message": format!("Transcribing chunk {}/{}", i + 1, total),
                }),
            )
            .map_err(|e| anyhow::anyhow!("Failed to emit progress: {}", e))?;
    }

    Ok(all)
}

/// Assign speakers to transcribed blocks from a set of diarization turns.
///
/// Rules:
/// 1. Greatest-overlap turn wins; ties → temporally nearest turn start.
/// 2. No overlap → nearest turn by `|block.start - turn.start|`.
/// 3. Block inside a gap ≤ ~2.0 s keeps the most-relevant speaker: same
///    speaker on both sides → propagated; otherwise → most-recent (left) turn.
/// 4. Never invents speakers: empty turns → all `None`.
pub fn assign_speakers(blocks: &[(f64, f64)], turns: &[SpeakerTurn]) -> Vec<Option<String>> {
    blocks
        .iter()
        .map(|&(start, end)| assign_block_speaker(start, end, turns))
        .collect()
}

fn overlap(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> f64 {
    (a_end.min(b_end) - a_start.max(b_start)).max(0.0)
}

fn assign_block_speaker(start: f64, end: f64, turns: &[SpeakerTurn]) -> Option<String> {
    if turns.is_empty() {
        return None;
    }

    // Rule 1: greatest overlap, ties → nearest turn start.
    let mut best: Option<(f64, f64, f64, usize)> = None; // (overlap, -dist, -start, idx)
    for (idx, t) in turns.iter().enumerate() {
        let ov = overlap(start, end, t.start, t.end);
        if ov <= 0.0 {
            continue;
        }
        let dist = (t.start - start).abs();
        let cand = (ov, -dist, -t.start as f64, idx);
        let better = match &best {
            None => true,
            Some(b) => (cand.0, cand.1, cand.2, cand.3) > (b.0, b.1, b.2, b.3),
        };
        if better {
            best = Some(cand);
        }
    }
    if let Some((_, _, _, idx)) = best {
        return Some(turns[idx].speaker.clone());
    }

    // Rule 3: block inside a gap between two consecutive turns.
    let prev = turns.iter().rev().find(|t| t.end <= start);
    let next = turns.iter().find(|t| t.start >= end);
    if let (Some(prev_t), Some(next_t)) = (prev, next) {
        let gap = next_t.start - prev_t.end;
        if (0.0..=2.0).contains(&gap) {
            // Same speaker both sides → propagate; different → most-recent.
            return Some(prev_t.speaker.clone());
        }
    }

    // Rule 2: nearest turn by |block.start - turn.start|.
    turns
        .iter()
        .min_by(|a, b| {
            let da = (a.start - start).abs();
            let db = (b.start - start).abs();
            da.partial_cmp(&db)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.start
                        .partial_cmp(&b.start)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
        .map(|t| t.speaker.clone())
}

/// Read a 16kHz mono WAV file to f32 samples.
fn read_wav_to_f32(path: &std::path::Path) -> anyhow::Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)?;
    let samples: Result<Vec<f32>, _> = reader
        .samples::<i16>()
        .map(|s| s.map(|s| s as f32 / 32768.0))
        .collect();
    Ok(samples?)
}

#[tauri::command]
pub async fn export_video_transcription(
    id: i64,
    format: String,
    path: String,
    state: State<'_, VideoTranscriptionState>,
) -> Result<(), String> {
    let entry = {
        let db = state.db.lock().unwrap();
        db.get_video_transcription(id)
            .map_err(|e| format!("Failed to get transcription: {}", e))?
    };

    let export_format = match format.as_str() {
        "srt" => ExportFormat::Srt,
        "vtt" => ExportFormat::Vtt,
        "txt" => ExportFormat::Txt,
        "json" => ExportFormat::Json,
        "ass" => ExportFormat::Ass,
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    let entries: Vec<ExportEntry> = entry
        .segments
        .iter()
        .enumerate()
        .map(|(i, seg)| {
            let timestamp = seconds_to_rfc3339(seg.start);
            ExportEntry {
                id: (i + 1) as i64,
                timestamp,
                language: entry.language.clone(),
                original_text: seg.text.clone(),
                translation: None,
                speaker: seg.speaker.clone(),
                duration: Some((seg.end - seg.start).max(0.0)),
            }
        })
        .collect();

    let path = PathBuf::from(path);
    export_entries(&entries, &export_format, &path).map_err(|e| e.to_string())
}

fn seconds_to_rfc3339(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    let millis = ((seconds - total_secs as f64) * 1000.0) as u32;

    format!(
        "1970-01-01T{:02}:{:02}:{:02}.{:03}Z",
        hours, minutes, secs, millis
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seconds_to_rfc3339() {
        assert_eq!(seconds_to_rfc3339(0.0), "1970-01-01T00:00:00.000Z");
        assert_eq!(seconds_to_rfc3339(65.5), "1970-01-01T00:01:05.500Z");
        assert_eq!(seconds_to_rfc3339(3661.123), "1970-01-01T01:01:01.123Z");
    }

    #[test]
    fn test_diarized_segment_serialization() {
        let seg = DiarizedSegment {
            start: 1.0,
            end: 3.5,
            text: "Hello world".to_string(),
            speaker: Some("SPEAKER_00".to_string()),
        };
        let json = serde_json::to_string(&seg).unwrap();
        let parsed: DiarizedSegment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.text, "Hello world");
        assert_eq!(parsed.speaker, Some("SPEAKER_00".to_string()));
    }

    #[test]
    fn test_diarized_segment_no_speaker() {
        let seg = DiarizedSegment {
            start: 0.0,
            end: 2.0,
            text: "Test".to_string(),
            speaker: None,
        };
        let json = serde_json::to_string(&seg).unwrap();
        assert!(json.contains("null"));
    }

    fn voice(frame: usize) -> Vec<f32> {
        (0..frame).map(|i| (i as f32 * 0.1).sin() * 0.5).collect()
    }

    fn silence(frame: usize) -> Vec<f32> {
        vec![0.0; frame]
    }

    // ── offset_segments ───────────────────────────────────────────

    #[test]
    fn test_offset_segments_shifts_timestamps() {
        let segs = vec![TranscriptionSegment {
            start: 0.5,
            end: 1.5,
            text: "hi".to_string(),
        }];
        let out = offset_segments(&segs, 10.0);
        assert_eq!(out[0].start, 10.5);
        assert_eq!(out[0].end, 11.5);
        assert_eq!(out[0].text, "hi");
    }

    #[test]
    fn test_offset_segments_empty() {
        assert!(offset_segments(&[], 5.0).is_empty());
    }

    // ── detect_speech_windows ─────────────────────────────────────

    #[test]
    fn test_detect_windows_silence_is_empty() {
        let audio = silence(16000 * 6);
        assert!(detect_speech_windows(&audio, 16000, 0.01).is_empty());
    }

    #[test]
    fn test_detect_windows_single_voice_chunk() {
        let mut audio = Vec::new();
        for _ in 0..4 {
            audio.extend(voice(16000));
        }
        let windows = detect_speech_windows(&audio, 16000, 0.01);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].start, 0);
        assert_eq!(windows[0].end, audio.len());
    }

    #[test]
    fn test_detect_windows_splits_after_silence() {
        let mut audio = Vec::new();
        audio.extend(voice(16000));
        audio.extend(voice(16000));
        audio.extend(silence(16000));
        audio.extend(silence(16000));
        audio.extend(voice(16000));
        audio.extend(voice(16000));

        let windows = detect_speech_windows(&audio, 16000, 0.01);
        assert_eq!(windows.len(), 2);
        assert_eq!(
            windows[0],
            SpeechWindow {
                start: 0,
                end: 64000
            }
        );
        assert_eq!(
            windows[1],
            SpeechWindow {
                start: 64000,
                end: 96000
            }
        );
    }

    #[test]
    fn test_detect_windows_empty_audio() {
        assert!(detect_speech_windows(&[], 16000, 0.01).is_empty());
    }

    // ── window_chunk_bounds ───────────────────────────────────────

    #[test]
    fn test_window_chunk_bounds_pads_and_clamps() {
        // Middle window: padded both sides.
        let w = SpeechWindow {
            start: 64000,
            end: 96000,
        };
        let (s, e) = window_chunk_bounds(&w, 16000, 160000);
        assert_eq!(s, 64000 - 4800);
        assert_eq!(e, 96000 + 4800);

        // First window: start clamped to 0.
        let w0 = SpeechWindow {
            start: 0,
            end: 64000,
        };
        let (s0, e0) = window_chunk_bounds(&w0, 16000, 64000);
        assert_eq!(s0, 0);
        assert_eq!(e0, 64000);
    }

    // ── assign_speakers ───────────────────────────────────────────

    fn turn(start: f64, end: f64, speaker: &str) -> SpeakerTurn {
        SpeakerTurn {
            start,
            end,
            speaker: speaker.to_string(),
        }
    }

    #[test]
    fn test_assign_speakers_empty_turns_all_none() {
        let blocks = [(0.0, 1.0), (2.0, 3.0)];
        assert_eq!(assign_speakers(&blocks, &[]), vec![None, None]);
    }

    #[test]
    fn test_assign_speakers_overlap_wins() {
        let turns = vec![turn(0.0, 2.0, "A"), turn(2.0, 4.0, "B")];
        // Block inside A only.
        assert_eq!(
            assign_speakers(&[(0.5, 1.5)], &turns),
            vec![Some("A".to_string())]
        );
    }

    #[test]
    fn test_assign_speakers_tie_broken_by_nearest_start() {
        let turns = vec![turn(0.0, 2.0, "A"), turn(2.0, 4.0, "B")];
        // Equal overlap with both → nearest turn start → "A".
        assert_eq!(
            assign_speakers(&[(1.0, 3.0)], &turns),
            vec![Some("A".to_string())]
        );
    }

    #[test]
    fn test_assign_speakers_gap_same_speaker_propagates() {
        let turns = vec![turn(0.0, 1.0, "A"), turn(1.5, 2.5, "A")];
        assert_eq!(
            assign_speakers(&[(1.0, 1.5)], &turns),
            vec![Some("A".to_string())]
        );
    }

    #[test]
    fn test_assign_speakers_gap_different_speakers_most_recent() {
        let turns = vec![turn(0.0, 1.0, "A"), turn(1.5, 2.5, "B")];
        assert_eq!(
            assign_speakers(&[(1.0, 1.5)], &turns),
            vec![Some("A".to_string())]
        );
    }

    #[test]
    fn test_assign_speakers_nearest_when_large_gap() {
        let turns = vec![turn(0.0, 1.0, "A"), turn(10.0, 11.0, "B")];
        // Closer to A.
        assert_eq!(
            assign_speakers(&[(5.0, 6.0)], &turns),
            vec![Some("A".to_string())]
        );
        // Closer to B.
        assert_eq!(
            assign_speakers(&[(8.0, 9.0)], &turns),
            vec![Some("B".to_string())]
        );
    }

    #[test]
    fn test_assign_speakers_single_speaker_only() {
        let turns = vec![turn(0.0, 10.0, "SPEAKER_00")];
        let blocks = [(0.5, 1.0), (9.0, 9.5)];
        let speakers = assign_speakers(&blocks, &turns);
        assert_eq!(speakers, vec![Some("SPEAKER_00".to_string()); 2]);
    }
}
