use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ModState {
    pub mods: Vec<ModEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ModEntry {
    pub id: String,
    pub enabled: bool,
    pub priority: u32,
}
