use crate::history::HistoryDb;
use crate::history::HistoryEntry;
use std::sync::{Arc, Mutex};
use tauri::State;

#[tauri::command]
pub async fn get_history(
    limit: Option<i64>,
    state: State<'_, Arc<Mutex<HistoryDb>>>,
) -> Result<Vec<HistoryEntry>, String> {
    let db = state.lock().map_err(|e| e.to_string())?;
    let entries = db.get_all(limit.unwrap_or(100)).map_err(|e| e.to_string())?;
    Ok(entries)
}

#[tauri::command]
pub async fn search_history(
    query: String,
    limit: Option<i64>,
    state: State<'_, Arc<Mutex<HistoryDb>>>,
) -> Result<Vec<HistoryEntry>, String> {
    let db = state.lock().map_err(|e| e.to_string())?;
    let result = db.search(&query, limit.unwrap_or(100)).map_err(|e| e.to_string())?;
    Ok(result.entries)
}

#[tauri::command]
pub async fn clear_history(
    state: State<'_, Arc<Mutex<HistoryDb>>>,
) -> Result<(), String> {
    let db = state.lock().map_err(|e| e.to_string())?;
    db.clear().map_err(|e| e.to_string())?;
    Ok(())
}
