use crate::settings::AppConfig;
use crate::audio::capture::AudioDeviceInfo;
use crate::ai::config::{AiConfig, AiProviderType};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tauri::State;

#[tauri::command]
pub async fn get_config(
    state: State<'_, Arc<Mutex<AppConfig>>>,
) -> Result<AppConfig, String> {
    let config = state.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    state: State<'_, Arc<Mutex<AppConfig>>>,
    ai_state: State<'_, Arc<Mutex<AiConfig>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut current = state.lock().map_err(|e| e.to_string())?;
    *current = config.clone();
    current.save().map_err(|e| e.to_string())?;

    // Sync the separate AI config state
    let provider = match config.ai.provider.to_lowercase().as_str() {
        "lmstudio" | "lm_studio" => AiProviderType::LmStudio,
        "deepseek" => AiProviderType::DeepSeek,
        _ => AiProviderType::Ollama,
    };
    {
        let mut ai = ai_state.lock().map_err(|e| e.to_string())?;
        *ai = AiConfig {
            provider,
            base_url: config.ai.base_url.clone(),
            api_key: config.ai.api_key.clone(),
            model: config.ai.model.clone(),
        };
    }

    let _ = app_handle.emit("overlay-config-updated", serde_json::json!({
        "fontSize": config.overlay.font_size,
        "showOriginal": config.translation.show_original,
        "maxVisibleLines": config.overlay.max_visible_lines,
        "lineGap": config.overlay.line_gap,
        "maxLineWidth": config.overlay.max_line_width,
        "displayDurationMs": config.overlay.display_duration_ms,
        "fadeDurationMs": config.overlay.fade_duration_ms,
    }));

    Ok(())
}

#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    crate::audio::AudioCapture::list_devices().map_err(|e| e.to_string())
}
