use std::sync::{Arc, Mutex};
use tauri::State;
use crate::translation::marian::MarianEngine;
use crate::translation::model::{MarianModelInfo, MarianModelManager};
use tracing::info;

#[tauri::command]
pub async fn download_marian_model(
    source_lang: String,
    target_lang: String,
    marian_engine: State<'_, Arc<Mutex<MarianEngine>>>,
) -> Result<String, String> {
    if !MarianEngine::is_supported(&source_lang, &target_lang) {
        return Err(format!("Unsupported language pair: {} → {}", source_lang, target_lang));
    }

    {
        let engine = marian_engine.lock().map_err(|e| e.to_string())?;
        if engine.is_downloaded(&source_lang, &target_lang) {
            info!("Marian model {}→{} already downloaded", source_lang, target_lang);
            return Ok(format!("Model {}→{} already downloaded", source_lang, target_lang));
        }
    }

    let info = match (source_lang.as_str(), target_lang.as_str()) {
        ("en", "es") => MarianModelInfo::en_es(),
        ("es", "en") => MarianModelInfo::es_en(),
        _ => return Err(format!("Unsupported language pair: {} → {}", source_lang, target_lang)),
    };

    let models_dir = {
        let engine = marian_engine.lock().map_err(|e| e.to_string())?;
        engine.models_dir()
    };

    let manager = MarianModelManager::new(models_dir);
    manager.download_async(&info).await
        .map_err(|e| format!("Download failed: {}", e))?;

    info!("Marian model {}→{} downloaded successfully", source_lang, target_lang);
    Ok(format!("Model {}→{} downloaded", source_lang, target_lang))
}

#[tauri::command]
pub async fn check_marian_model(
    source_lang: String,
    target_lang: String,
    marian_engine: State<'_, Arc<Mutex<MarianEngine>>>,
) -> Result<bool, String> {
    let engine = marian_engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.is_downloaded(&source_lang, &target_lang))
}

#[tauri::command]
pub async fn delete_marian_model(
    source_lang: String,
    target_lang: String,
    marian_engine: State<'_, Arc<Mutex<MarianEngine>>>,
) -> Result<String, String> {
    let engine = marian_engine.lock().map_err(|e| e.to_string())?;
    engine.delete_model(&source_lang, &target_lang)
        .map_err(|e| format!("Delete failed: {}", e))?;
    info!("Marian model {}→{} deleted", source_lang, target_lang);
    Ok(format!("Model {}→{} deleted", source_lang, target_lang))
}

#[tauri::command]
pub async fn list_marian_models(
    marian_engine: State<'_, Arc<Mutex<MarianEngine>>>,
) -> Result<Vec<serde_json::Value>, String> {
    let engine = marian_engine.lock().map_err(|e| e.to_string())?;
    let pairs = MarianModelInfo::supported_pairs();
    let mut result = Vec::new();

    for pair in &pairs {
        let parts: Vec<&str> = pair.split('-').collect();
        if parts.len() == 2 {
            let downloaded = engine.is_downloaded(parts[0], parts[1]);
            result.push(serde_json::json!({
                "source": parts[0],
                "target": parts[1],
                "downloaded": downloaded,
            }));
        }
    }

    Ok(result)
}
