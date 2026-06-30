use crate::models::AppConfig;
use crate::services::{ModManager, PluginManager, ThemeManager};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub mod_manager: Mutex<ModManager>,
    pub plugin_manager: Mutex<PluginManager>,
    pub theme_manager: Mutex<ThemeManager>,
    pub config: Mutex<AppConfig>,
    pub data_dir: PathBuf,
}
