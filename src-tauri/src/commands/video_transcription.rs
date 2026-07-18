use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};
use tracing::{info, warn};

use crate::commands::export::{ExportEntry, ExportFormat, export_entries};
use crate::diarization::engine::DiarizationEngine;
use crate::history::HistoryDb;
use crate::video::processor::VideoProcessor;
use crate::whisper::engine::WhisperEngine;
use crate::whisper::params::TranscriptionParams;

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
}

pub struct VideoTranscriptionState {
    pub db: Arc<Mutex<HistoryDb>>,
    pub whisper: Arc<Mutex<WhisperEngine>>,
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
    let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
        "step": "extracting",
        "progress": 0.0,
        "message": "Extracting audio from video...",
    }));

    let audio_path = VideoProcessor::extract_audio(&path)
        .await
        .map_err(|e| format!("Failed to extract audio: {}", e))?;

    let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
        "step": "extracting",
        "progress": 1.0,
        "message": "Audio extracted successfully",
    }));

    // Step 2: Get duration
    let duration = VideoProcessor::get_duration(&path)
        .await
        .unwrap_or(0.0);

    // Step 3: Diarization (if enabled)
    let speaker_turns = if enable_diarization {
        let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
            "step": "diarizing",
            "progress": 0.0,
            "message": "Detecting speakers...",
        }));

        let diarization = state.diarization.lock().unwrap();
        match diarization.diarize(&audio_path) {
            Ok(turns) => {
                let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
                    "step": "diarizing",
                    "progress": 1.0,
                    "message": format!("Detected {} speaker(s)", turns.iter().map(|t| &t.speaker).collect::<std::collections::HashSet<_>>().len()),
                }));
                Some(turns)
            }
            Err(e) => {
                warn!("Diarization failed: {}", e);
                let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
                    "step": "diarizing",
                    "progress": 1.0,
                    "message": "Diarization failed, continuing without speaker labels",
                }));
                None
            }
        }
    } else {
        None
    };

    // Step 4: Transcribe with Whisper
    let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
        "step": "transcribing",
        "progress": 0.0,
        "message": "Transcribing audio...",
    }));

    // Read audio file to f32 samples
    let audio_data = read_wav_to_f32(&audio_path).map_err(|e| format!("Failed to read audio: {}", e))?;

    let segments = {
        let engine = state.whisper.lock().unwrap();
        if !engine.is_loaded() {
            return Err("Whisper model not loaded".to_string());
        }

        let params = TranscriptionParams {
            language: language.clone(),
            threads: 4,
            gpu: false,
            translate: false,
            ..Default::default()
        };

        engine
            .transcribe(&audio_data, &params)
            .map_err(|e| format!("Transcription failed: {}", e))?
    };

    let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
        "step": "transcribing",
        "progress": 1.0,
        "message": "Transcription complete",
    }));

    // Step 5: Merge segments with speaker turns
    let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
        "step": "merging",
        "progress": 0.0,
        "message": "Processing segments...",
    }));

    let diarized_segments: Vec<DiarizedSegment> = segments
        .iter()
        .map(|seg| {
            let speaker = speaker_turns.as_ref().and_then(|turns| {
                let mid_time = (seg.start + seg.end) / 2.0;
                turns
                    .iter()
                    .find(|t| mid_time >= t.start && mid_time <= t.end)
                    .map(|t| t.speaker.clone())
            });

            DiarizedSegment {
                start: seg.start,
                end: seg.end,
                text: seg.text.clone(),
                speaker,
            }
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
            let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
                "step": "translating",
                "progress": 0.0,
                "message": format!("Translating to {}...", target_lang),
            }));

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
                    let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
                        "step": "translating",
                        "progress": 1.0,
                        "message": "Translation complete",
                    }));
                    Some(translated)
                }
                Err(e) => {
                    warn!("Translation failed: {}", e);
                    let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
                        "step": "translating",
                        "progress": 1.0,
                        "message": "Translation failed, using original text",
                    }));
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
    let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
        "step": "saving",
        "progress": 0.0,
        "message": "Saving transcription...",
    }));

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
        )
        .map_err(|e| format!("Failed to save transcription: {}", e))?
    };

    // Clean up temp WAV file
    let _ = tokio::fs::remove_file(&audio_path).await;

    let _ = app_handle.emit("video-transcription-progress", serde_json::json!({
        "step": "done",
        "progress": 1.0,
        "message": "Transcription complete",
    }));

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
}
