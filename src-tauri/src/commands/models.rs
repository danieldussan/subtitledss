use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{State, Emitter};
use crate::whisper::WhisperEngine;
use crate::whisper::model::ModelManager;
use crate::models::ModelDownloader;
use crate::settings::AppConfig;
use tracing::info;

#[tauri::command]
pub async fn download_model(
    model_name: String,
    model_manager: State<'_, Arc<Mutex<ModelManager>>>,
) -> Result<String, String> {
    let models_dir: PathBuf;
    {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        if manager.is_downloaded(&model_name) {
            info!("Model '{}' already downloaded", model_name);
            return Ok(format!("Model '{}' already downloaded", model_name));
        }
        models_dir = manager.models_dir().clone();
    }

    let downloader = ModelDownloader::new(models_dir);
    downloader.download(&model_name)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    info!("Model '{}' downloaded successfully", model_name);
    Ok(format!("Model '{}' downloaded", model_name))
}

#[tauri::command]
pub async fn delete_model(
    model_name: String,
    model_manager: State<'_, Arc<Mutex<ModelManager>>>,
) -> Result<String, String> {
    let manager = model_manager.lock().map_err(|e| e.to_string())?;
    manager.delete_model(&model_name)
        .map_err(|e| format!("Delete failed: {}", e))?;
    info!("Model '{}' deleted", model_name);
    Ok(format!("Model '{}' deleted", model_name))
}

#[tauri::command]
pub async fn list_downloaded_models(
    model_manager: State<'_, Arc<Mutex<ModelManager>>>,
) -> Result<Vec<String>, String> {
    let manager = model_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.list_downloaded())
}

#[tauri::command]
pub async fn load_model(
    model_name: String,
    whisper_engine: State<'_, Arc<Mutex<WhisperEngine>>>,
    model_manager: State<'_, Arc<Mutex<ModelManager>>>,
) -> Result<String, String> {
    let model_path = {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        if !manager.is_downloaded(&model_name) {
            return Err(format!("Model '{}' is not downloaded", model_name));
        }
        manager.get_model_path(&model_name)
    };

    let mut engine = whisper_engine.lock().map_err(|e| e.to_string())?;
    engine.load_model(&model_path)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    info!("Model '{}' loaded successfully", model_name);
    Ok(format!("Model '{}' loaded", model_name))
}

#[tauri::command]
pub async fn switch_model(
    model_name: String,
    whisper_engine: State<'_, Arc<Mutex<WhisperEngine>>>,
    model_manager: State<'_, Arc<Mutex<ModelManager>>>,
    config: State<'_, Arc<Mutex<AppConfig>>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let model_path = {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        if !manager.is_downloaded(&model_name) {
            return Err(format!("Model '{}' is not downloaded", model_name));
        }
        manager.get_model_path(&model_name)
    };

    {
        let mut engine = whisper_engine.lock().map_err(|e| e.to_string())?;
        engine.load_model(&model_path)
            .map_err(|e| format!("Failed to load model: {}", e))?;
    }

    {
        let mut cfg = config.lock().map_err(|e| e.to_string())?;
        cfg.whisper.model = model_name.clone();
        cfg.save().map_err(|e| e.to_string())?;
    }

    let _ = app_handle.emit("model-changed", serde_json::json!({
        "model": model_name,
    }));

    info!("Switched to model '{}' and saved config", model_name);
    Ok(format!("Model '{}' loaded", model_name))
}

#[tauri::command]
pub async fn get_loaded_model(
    whisper_engine: State<'_, Arc<Mutex<WhisperEngine>>>,
) -> Result<Option<String>, String> {
    let engine = whisper_engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.model_path().and_then(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_prefix("ggml-"))
            .map(|s| s.to_string())
    }))
}
