use crate::settings::AppConfig;
use crate::audio::capture::AudioDeviceInfo;
use std::sync::{Arc, Mutex};
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
) -> Result<(), String> {
    let mut current = state.lock().map_err(|e| e.to_string())?;
    *current = config.clone();
    current.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    crate::audio::AudioCapture::list_devices().map_err(|e| e.to_string())
}
