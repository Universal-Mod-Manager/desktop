use crate::models::ModInfo;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn list_mods(state: tauri::State<'_, AppState>) -> Result<Vec<ModInfo>, String> {
    let manager = state.mod_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.list_mods())
}

#[tauri::command]
#[specta::specta]
pub fn toggle_mod(
    state: tauri::State<'_, AppState>,
    mod_id: String,
    enabled: bool,
) -> Result<(), String> {
    {
        let mut manager = state.mod_manager.lock().map_err(|e| e.to_string())?;
        manager
            .toggle_mod(&mod_id, enabled)
            .map_err(|e| e.to_string())?;
    }
    sync_load_order_to_game(&state)
}

#[tauri::command]
#[specta::specta]
pub fn reorder_mods(state: tauri::State<'_, AppState>, mod_ids: Vec<String>) -> Result<(), String> {
    {
        let mut manager = state.mod_manager.lock().map_err(|e| e.to_string())?;
        manager.reorder_mods(mod_ids).map_err(|e| e.to_string())?;
    }
    sync_load_order_to_game(&state)
}

fn sync_load_order_to_game(state: &AppState) -> Result<(), String> {
    let (plugin_id, game_path) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        let pid = match config.active_plugin.as_deref() {
            Some(id) => id.to_string(),
            None => return Ok(()),
        };
        let gp = config
            .game_paths
            .get(&pid)
            .ok_or_else(|| format!("No game path for '{}'", pid))?
            .clone();
        (pid, gp)
    };

    let mods = {
        let mgr = state.mod_manager.lock().map_err(|e| e.to_string())?;
        mgr.list_mods()
    };

    let input = serde_json::json!({ "mods": mods }).to_string();

    let result = {
        let plugin_mgr = state.plugin_manager.lock().map_err(|e| e.to_string())?;
        plugin_mgr
            .call_plugin_fn(&plugin_id, "write_load_order", &input)
            .map_err(|e| e.to_string())?
    };

    let output: serde_json::Value = serde_json::from_str(&result).map_err(|e| e.to_string())?;
    let relative_path = output["relative_path"]
        .as_str()
        .ok_or("Plugin returned no relative_path")?;
    let content = output["content"]
        .as_str()
        .ok_or("Plugin returned no content")?;

    let file_path = std::path::Path::new(&game_path).join(relative_path);
    std::fs::write(&file_path, content).map_err(|e| e.to_string())?;

    Ok(())
}
