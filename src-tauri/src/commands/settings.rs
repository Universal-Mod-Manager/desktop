use crate::models::{GamePathRoot, PluginPathConfig};
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn get_plugin_paths(state: tauri::State<'_, AppState>) -> Result<PluginPathConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.plugin_paths.clone())
}

#[tauri::command]
#[specta::specta]
pub fn set_plugin_path(
    state: tauri::State<'_, AppState>,
    plugin_id: String,
    root_id: String,
    path: String,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config
        .plugin_paths
        .entry(plugin_id)
        .or_default()
        .insert(root_id, path);
    let content = serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?;
    std::fs::write(state.data_dir.join("config.json"), content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_plugin_path_roots(
    state: tauri::State<'_, AppState>,
    plugin_id: String,
) -> Result<Vec<GamePathRoot>, String> {
    let plugin_mgr = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    let metadata = plugin_mgr
        .game_metadata(&plugin_id)
        .map_err(|e| e.to_string())?;
    Ok(metadata.path_roots)
}
