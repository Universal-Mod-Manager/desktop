use crate::models::ModInfo;
use crate::state::AppState;
use std::path::{Component, Path};

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

    validate_plugin_relative_path(relative_path)?;

    let file_path = std::path::Path::new(&game_path).join(relative_path);
    std::fs::write(&file_path, content).map_err(|e| e.to_string())?;

    Ok(())
}

fn validate_plugin_relative_path(relative_path: &str) -> Result<(), String> {
    if relative_path.trim().is_empty() {
        return Err("Plugin returned an empty relative_path".to_string());
    }

    if relative_path.contains('\\') || relative_path.contains(':') {
        return Err(format!(
            "Plugin returned unsafe relative_path: {relative_path}"
        ));
    }

    let path = Path::new(relative_path);
    if path.is_absolute() {
        return Err(format!(
            "Plugin returned absolute relative_path: {relative_path}"
        ));
    }

    let mut has_file_component = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => has_file_component = true,
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(format!(
                    "Plugin returned unsafe relative_path: {relative_path}"
                ));
            }
        }
    }

    if !has_file_component {
        return Err("Plugin returned an empty relative_path".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_plugin_relative_path;

    #[test]
    fn plugin_relative_path_allows_normal_relative_paths() {
        assert!(validate_plugin_relative_path("loadorder.txt").is_ok());
        assert!(validate_plugin_relative_path("profiles/loadorder.txt").is_ok());
    }

    #[test]
    fn plugin_relative_path_rejects_traversal_and_absolute_paths() {
        for path in [
            "",
            ".",
            "./loadorder.txt",
            "../outside.txt",
            "profiles/../../outside.txt",
            "/tmp/outside.txt",
        ] {
            assert!(
                validate_plugin_relative_path(path).is_err(),
                "path should be rejected: {path}"
            );
        }
    }

    #[test]
    fn plugin_relative_path_rejects_windows_prefixes_and_separators() {
        for path in [
            "C:/Users/player/outside.txt",
            "C:\\Users\\player\\outside.txt",
            "\\\\server\\share\\outside.txt",
            "profiles\\loadorder.txt",
        ] {
            assert!(
                validate_plugin_relative_path(path).is_err(),
                "path should be rejected: {path}"
            );
        }
    }
}
