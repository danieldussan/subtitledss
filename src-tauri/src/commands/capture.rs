use std::sync::{Arc, Mutex, atomic::{AtomicU32, AtomicUsize}};
use tauri::State;
use crate::audio::{AudioCapture, RingBuffer};
use crate::audio::capture::AUDIO_LEVEL;
use crate::whisper::WhisperEngine;
use crate::pipeline::TranscriptionPipeline;
use crate::settings::AppConfig;
use tracing::{info, error};

#[tauri::command]
pub async fn start_capture(
    audio_capture: State<'_, Arc<Mutex<AudioCapture>>>,
    audio_buffer: State<'_, Arc<Mutex<RingBuffer>>>,
    whisper_engine: State<'_, Arc<Mutex<WhisperEngine>>>,
    pipeline: State<'_, Arc<Mutex<TranscriptionPipeline>>>,
    config: State<'_, Arc<Mutex<AppConfig>>>,
    actual_sample_rate: State<'_, Arc<AtomicU32>>,
    actual_channels: State<'_, Arc<AtomicUsize>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    info!("Starting capture...");

    // Check if whisper model is loaded
    {
        let whisper = whisper_engine.lock().map_err(|e| e.to_string())?;
        if !whisper.is_loaded() {
            error!("Whisper model not loaded!");
            return Err("Whisper model not loaded. Download and load a model first.".to_string());
        }
        info!("Whisper model is loaded");
    }

    // Start audio capture
    let device_name: Option<String> = {
        let cfg = config.lock().map_err(|e| e.to_string())?;
        info!("Audio config: source={}, device={}, sample_rate={}", 
            cfg.audio.source, cfg.audio.device, cfg.audio.sample_rate);
        if cfg.audio.device == "default" {
            None
        } else {
            Some(cfg.audio.device.clone())
        }
    };

    {
        let mut buf = audio_buffer.lock().map_err(|e| e.to_string())?;
        buf.clear();
    }

    let buffer_arc = audio_buffer.inner().clone();
    let device_str = device_name.as_deref();
    let rate_arc = actual_sample_rate.inner().clone();
    let ch_arc = actual_channels.inner().clone();

    {
        let mut capture = audio_capture.lock().map_err(|e| e.to_string())?;
        if capture.is_running() {
            info!("Already capturing");
            return Ok("Already capturing".to_string());
        }
        capture.start(buffer_arc, device_str, rate_arc, ch_arc)
            .map_err(|e| {
                let msg = format!("Failed to start capture: {}", e);
                error!("{}", msg);
                msg
            })?;
    }

    // Start transcription pipeline
    let pipeline_config = {
        let cfg = config.lock().map_err(|e| e.to_string())?;
        cfg.clone()
    };

    {
        let mut pipe = pipeline.lock().map_err(|e| e.to_string())?;
        pipe.start(
            audio_buffer.inner().clone(),
            whisper_engine.inner().clone(),
            app_handle,
            pipeline_config,
        );
    }

    info!("Capture + pipeline started successfully");
    Ok("Capture started".to_string())
}

#[tauri::command]
pub async fn stop_capture(
    audio_capture: State<'_, Arc<Mutex<AudioCapture>>>,
    pipeline: State<'_, Arc<Mutex<TranscriptionPipeline>>>,
) -> Result<String, String> {
    // Stop pipeline first
    {
        let mut pipe = pipeline.lock().map_err(|e| e.to_string())?;
        pipe.stop();
    }

    // Then stop audio capture
    {
        let mut capture = audio_capture.lock().map_err(|e| e.to_string())?;
        capture.stop();
    }

    // Reset audio level
    AUDIO_LEVEL.store(0, std::sync::atomic::Ordering::Relaxed);

    info!("Capture + pipeline stopped");
    Ok("Capture stopped".to_string())
}

#[tauri::command]
pub async fn get_audio_level() -> Result<f32, String> {
    Ok(AUDIO_LEVEL.load(std::sync::atomic::Ordering::Relaxed) as f32 / 1_000_000.0)
}
