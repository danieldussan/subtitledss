use tauri::Manager;
use tracing::info;

#[tauri::command]
pub async fn toggle_overlay(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let overlay = app_handle.get_webview_window("overlay")
        .ok_or_else(|| "Overlay window not found".to_string())?;

    if overlay.is_visible().unwrap_or(false) {
        overlay.hide().map_err(|e| e.to_string())?;
        info!("Overlay hidden");
        Ok(false)
    } else {
        overlay.show().map_err(|e| e.to_string())?;
        info!("Overlay shown");
        Ok(true)
    }
}

#[tauri::command]
pub async fn show_overlay(app_handle: tauri::AppHandle) -> Result<(), String> {
    let overlay = app_handle.get_webview_window("overlay")
        .ok_or_else(|| "Overlay window not found".to_string())?;
    overlay.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn hide_overlay(app_handle: tauri::AppHandle) -> Result<(), String> {
    let overlay = app_handle.get_webview_window("overlay")
        .ok_or_else(|| "Overlay window not found".to_string())?;
    overlay.hide().map_err(|e| e.to_string())?;
    Ok(())
}
