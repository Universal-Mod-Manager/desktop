use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub icon_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PluginMetadata {
    pub name: String,
    pub wasm_path: String,
    pub icon_path: String,
}
