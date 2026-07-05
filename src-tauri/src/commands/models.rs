use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;
use crate::whisper::WhisperEngine;
use crate::whisper::model::ModelManager;
use crate::models::ModelDownloader;
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
