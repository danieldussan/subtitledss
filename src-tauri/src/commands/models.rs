use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{State, Emitter};
use crate::asr::AsrEngine;
use crate::whisper::model::ModelManager;
use crate::models::ModelDownloader;
use crate::sherpa::models as sherpa_models;
use crate::settings::AppConfig;
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
pub struct AvailableModel {
    pub name: String,
    pub label: String,
    pub engine: String,
    pub size_mb: u64,
    pub languages: String,
    pub description: String,
    pub downloaded: bool,
}

#[tauri::command]
pub async fn list_available_models(
    model_manager: State<'_, Arc<Mutex<ModelManager>>>,
) -> Result<Vec<AvailableModel>, String> {
    let (models_dir, downloaded) = {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        (manager.models_dir().clone(), manager.list_downloaded())
    };

    let mut out = Vec::new();
    for m in crate::whisper::model::ModelInfo::available_models() {
        out.push(AvailableModel {
            name: m.name.clone(),
            label: m.name.clone(),
            engine: "whisper".to_string(),
            size_mb: m.size_mb,
            languages: "Multilingual (99 languages)".to_string(),
            description: "OpenAI Whisper (whisper.cpp)".to_string(),
            downloaded: downloaded.contains(&format!("ggml-{}.bin", m.name)),
        });
    }
    for m in sherpa_models::SherpaModelInfo::available() {
        out.push(AvailableModel {
            name: m.name.to_string(),
            label: m.label.to_string(),
            engine: "sherpa".to_string(),
            size_mb: m.size_mb,
            languages: m.languages.to_string(),
            description: m.description.to_string(),
            downloaded: sherpa_models::model_dir(&models_dir, m.name).exists(),
        });
    }
    Ok(out)
}

#[tauri::command]
pub async fn download_model(
    model_name: String,
    model_manager: State<'_, Arc<Mutex<ModelManager>>>,
) -> Result<String, String> {
    let models_dir: PathBuf;
    {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        models_dir = manager.models_dir().clone();
    }

    if sherpa_models::is_sherpa_model(&model_name) {
        let dir = sherpa_models::download(&model_name, &models_dir)
            .await
            .map_err(|e| format!("Download failed: {}", e))?;
        info!("Sherpa model '{}' downloaded to {:?}", model_name, dir);
        return Ok(format!("Model '{}' downloaded", model_name));
    }

    {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        if manager.is_downloaded(&model_name) {
            info!("Model '{}' already downloaded", model_name);
            return Ok(format!("Model '{}' already downloaded", model_name));
        }
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
    let models_dir = {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        manager.models_dir().clone()
    };

    if sherpa_models::is_sherpa_model(&model_name) {
        let dir = sherpa_models::model_dir(&models_dir, &model_name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| format!("Delete failed: {}", e))?;
            info!("Sherpa model '{}' deleted", model_name);
        }
        return Ok(format!("Model '{}' deleted", model_name));
    }

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
    let models_dir = {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        let mut names = manager.list_downloaded();
        names.extend(sherpa_models::list_downloaded(manager.models_dir()));
        names.sort();
        names
    };
    Ok(models_dir)
}

#[tauri::command]
pub async fn load_model(
    model_name: String,
    asr_engine: State<'_, Arc<Mutex<AsrEngine>>>,
    model_manager: State<'_, Arc<Mutex<ModelManager>>>,
) -> Result<String, String> {
    let models_dir = {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        manager.models_dir().clone()
    };

    let model_path: PathBuf = if sherpa_models::is_sherpa_model(&model_name) {
        let dir = sherpa_models::model_dir(&models_dir, &model_name);
        if !dir.exists() {
            return Err(format!("Model '{}' is not downloaded", model_name));
        }
        dir
    } else {
        {
            let manager = model_manager.lock().map_err(|e| e.to_string())?;
            if !manager.is_downloaded(&model_name) {
                return Err(format!("Model '{}' is not downloaded", model_name));
            }
        }
        models_dir.join(format!("ggml-{}.bin", model_name))
    };

    let mut engine = asr_engine.lock().map_err(|e| e.to_string())?;
    engine.load_model(&model_path)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    info!("Model '{}' loaded successfully", model_name);
    Ok(format!("Model '{}' loaded", model_name))
}

#[tauri::command]
pub async fn switch_model(
    model_name: String,
    asr_engine: State<'_, Arc<Mutex<AsrEngine>>>,
    model_manager: State<'_, Arc<Mutex<ModelManager>>>,
    config: State<'_, Arc<Mutex<AppConfig>>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let models_dir = {
        let manager = model_manager.lock().map_err(|e| e.to_string())?;
        manager.models_dir().clone()
    };

    let model_path: PathBuf = if sherpa_models::is_sherpa_model(&model_name) {
        let dir = sherpa_models::model_dir(&models_dir, &model_name);
        if !dir.exists() {
            return Err(format!("Model '{}' is not downloaded", model_name));
        }
        dir
    } else {
        {
            let manager = model_manager.lock().map_err(|e| e.to_string())?;
            if !manager.is_downloaded(&model_name) {
                return Err(format!("Model '{}' is not downloaded", model_name));
            }
        }
        models_dir.join(format!("ggml-{}.bin", model_name))
    };

    {
        let mut engine = asr_engine.lock().map_err(|e| e.to_string())?;
        engine.load_model(&model_path)
            .map_err(|e| format!("Failed to load model: {}", e))?;
    }

    {
        let mut cfg = config.lock().map_err(|e| e.to_string())?;
        cfg.whisper.model = model_name.clone();
        cfg.whisper.engine = if sherpa_models::is_sherpa_model(&model_name) {
            "sherpa".to_string()
        } else {
            "whisper".to_string()
        };
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
    asr_engine: State<'_, Arc<Mutex<AsrEngine>>>,
) -> Result<Option<String>, String> {
    let engine = asr_engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.model_name())
}
