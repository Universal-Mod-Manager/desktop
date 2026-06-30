use serde::{Deserialize, Serialize};

use super::ModEntry;

pub const SUPPORTED_PLUGIN_API_VERSION: u32 = 2;
pub const GAME_INSTALL_PATH_ROOT_ID: &str = "game";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    pub api_version: u32,
    pub name: String,
    pub executable: String,
    pub path_roots: Vec<GamePathRoot>,
    pub mod_discovery: ModDiscovery,
    pub load_order_writes: Vec<LoadOrderWriteTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GamePathRoot {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDiscovery {
    pub root_id: String,
    pub relative_path: String,
    pub mode: ModDiscoveryMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModDiscoveryMode {
    DirectoryMods {
        required_prefix: Option<String>,
        metadata_file: Option<String>,
    },
    PluginFiles {
        extensions: Vec<String>,
        excluded_files: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadOrderWriteTarget {
    pub root_id: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildLoadOrderInput {
    pub mods: Vec<ModEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildLoadOrderOutput {
    pub writes: Vec<GameFileWrite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameFileWrite {
    pub root_id: String,
    pub relative_path: String,
    pub content: String,
}
