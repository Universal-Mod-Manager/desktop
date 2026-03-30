use crate::models::{ModInfo, PluginInfo};
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn list_plugins(state: tauri::State<'_, AppState>) -> Result<Vec<PluginInfo>, String> {
    let manager = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.list_plugins())
}

#[tauri::command]
#[specta::specta]
pub fn get_active_plugin(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let manager = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_active_plugin_id().map(|s| s.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn select_plugin(
    state: tauri::State<'_, AppState>,
    plugin_id: String,
) -> Result<Vec<ModInfo>, String> {
    {
        let mut plugin_mgr = state.plugin_manager.lock().map_err(|e| e.to_string())?;
        plugin_mgr
            .select_plugin(&plugin_id)
            .map_err(|e| e.to_string())?;
    }

    let mod_directory = {
        let plugin_mgr = state.plugin_manager.lock().map_err(|e| e.to_string())?;
        let metadata_json = plugin_mgr
            .call_plugin_fn(&plugin_id, "get_game_metadata", "")
            .map_err(|e| e.to_string())?;
        let metadata: serde_json::Value =
            serde_json::from_str(&metadata_json).map_err(|e| e.to_string())?;
        metadata["mod_directory"]
            .as_str()
            .unwrap_or("mods")
            .to_string()
    };

    let game_path = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config
            .game_paths
            .get(&plugin_id)
            .ok_or_else(|| format!("No game path configured for '{}'", plugin_id))?
            .clone()
    };

    let mods = {
        let mut mod_mgr = state.mod_manager.lock().map_err(|e| e.to_string())?;
        mod_mgr
            .load_mods_for_plugin(&plugin_id, &game_path, &mod_directory)
            .map_err(|e| e.to_string())?;
        mod_mgr.list_mods()
    };

    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        config.active_plugin = Some(plugin_id);
        let content = serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?;
        std::fs::write(state.data_dir.join("config.json"), content).map_err(|e| e.to_string())?;
    }

    Ok(mods)
}
