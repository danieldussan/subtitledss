use std::sync::{Arc, Mutex};
use tauri::State;
use crate::whisper::WhisperEngine;
use crate::whisper::TranscriptionParams;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TranscriptionResult {
    pub segments: Vec<TranscriptionSegment>,
    pub full_text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TranscriptionSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[tauri::command]
pub async fn transcribe_audio(
    audio_data: Vec<f32>,
    language: Option<String>,
    threads: Option<u32>,
    state: State<'_, Arc<Mutex<WhisperEngine>>>,
) -> Result<TranscriptionResult, String> {
    let engine = state.lock().map_err(|e| e.to_string())?;

    if !engine.is_loaded() {
        return Err("Whisper model not loaded".to_string());
    }

    let params = TranscriptionParams {
        language,
        threads: threads.unwrap_or(4),
        gpu: false,
        translate: false,
        ..Default::default()
    };

    let segments = engine.transcribe(&audio_data, &params).map_err(|e| e.to_string())?;

    let full_text = segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");

    let result_segments = segments
        .iter()
        .map(|s| TranscriptionSegment {
            start: s.start,
            end: s.end,
            text: s.text.clone(),
        })
        .collect();

    Ok(TranscriptionResult {
        segments: result_segments,
        full_text,
    })
}
