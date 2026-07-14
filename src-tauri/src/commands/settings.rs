use crate::settings::AppConfig;
use crate::audio::capture::AudioDeviceInfo;
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
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut current = state.lock().map_err(|e| e.to_string())?;
    *current = config.clone();
    current.save().map_err(|e| e.to_string())?;

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
