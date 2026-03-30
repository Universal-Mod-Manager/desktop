use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AppConfig {
    pub active_plugin: Option<String>,
    pub active_theme: String,
    pub game_paths: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_plugin: None,
            active_theme: String::new(),
            game_paths: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ThemeInfo {
    pub name: String,
    pub is_active: bool,
}
