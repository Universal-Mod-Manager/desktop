use crate::state::AppState;
use std::collections::HashMap;

#[tauri::command]
#[specta::specta]
pub fn get_game_paths(
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.game_paths.clone())
}

#[tauri::command]
#[specta::specta]
pub fn set_game_path(
    state: tauri::State<'_, AppState>,
    plugin_id: String,
    path: String,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.game_paths.insert(plugin_id, path);
    let content = serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?;
    std::fs::write(state.data_dir.join("config.json"), content).map_err(|e| e.to_string())?;
    Ok(())
}
