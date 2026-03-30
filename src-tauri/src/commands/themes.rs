use crate::models::ThemeInfo;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn list_themes(state: tauri::State<'_, AppState>) -> Result<Vec<ThemeInfo>, String> {
    let manager = state.theme_manager.lock().map_err(|e| e.to_string())?;
    manager.list_themes().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_theme_css(
    state: tauri::State<'_, AppState>,
    theme_name: String,
) -> Result<String, String> {
    let manager = state.theme_manager.lock().map_err(|e| e.to_string())?;
    manager
        .get_theme_css(&theme_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_active_theme(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let manager = state.theme_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_active_theme().to_string())
}

#[tauri::command]
#[specta::specta]
pub fn set_active_theme(
    state: tauri::State<'_, AppState>,
    theme_name: String,
) -> Result<String, String> {
    let mut manager = state.theme_manager.lock().map_err(|e| e.to_string())?;
    manager
        .set_active_theme(&theme_name)
        .map_err(|e| e.to_string())?;

    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        config.active_theme = theme_name.clone();
        let content = serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?;
        std::fs::write(state.data_dir.join("config.json"), content).map_err(|e| e.to_string())?;
    }

    manager
        .get_theme_css(&theme_name)
        .map_err(|e| e.to_string())
}
