use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct AppConfig {
    pub active_plugin: Option<String>,
    pub active_theme: String,
    pub game_paths: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ThemeInfo {
    pub name: String,
    pub is_active: bool,
}
