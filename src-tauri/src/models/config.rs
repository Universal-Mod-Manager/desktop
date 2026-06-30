use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use super::GAME_INSTALL_PATH_ROOT_ID;

pub type PluginPathConfig = HashMap<String, HashMap<String, String>>;

#[derive(Debug, Clone, Default, Serialize, specta::Type)]
pub struct AppConfig {
    pub active_plugin: Option<String>,
    pub active_theme: String,
    pub plugin_paths: PluginPathConfig,
}

#[derive(Default, Deserialize)]
struct AppConfigFile {
    #[serde(default)]
    active_plugin: Option<String>,
    #[serde(default)]
    active_theme: String,
    #[serde(default)]
    plugin_paths: PluginPathConfig,
    #[serde(default)]
    game_paths: HashMap<String, String>,
}

impl<'de> Deserialize<'de> for AppConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut file = AppConfigFile::deserialize(deserializer)?;
        for (plugin_id, path) in file.game_paths {
            file.plugin_paths
                .entry(plugin_id)
                .or_default()
                .entry(GAME_INSTALL_PATH_ROOT_ID.to_string())
                .or_insert(path);
        }

        Ok(Self {
            active_plugin: file.active_plugin,
            active_theme: file.active_theme,
            plugin_paths: file.plugin_paths,
        })
    }
}

impl AppConfig {
    pub fn configured_path_roots(
        &self,
        plugin_id: &str,
        metadata: &super::GameMetadata,
    ) -> Result<HashMap<String, String>, String> {
        let configured_paths = self
            .plugin_paths
            .get(plugin_id)
            .ok_or_else(|| format!("No paths configured for '{plugin_id}'"))?;

        let mut roots = HashMap::new();
        for root in &metadata.path_roots {
            let path = configured_paths
                .get(&root.id)
                .filter(|path| !path.trim().is_empty())
                .ok_or_else(|| {
                    format!(
                        "No path configured for '{}' ({}) in plugin '{}'",
                        root.name, root.id, plugin_id
                    )
                })?;
            roots.insert(root.id.clone(), path.clone());
        }

        Ok(roots)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ThemeInfo {
    pub name: String,
    pub is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        GameMetadata, GamePathRoot, LoadOrderWriteTarget, ModDiscovery, ModDiscoveryMode,
    };

    fn metadata_with_roots() -> GameMetadata {
        GameMetadata {
            api_version: 2,
            name: "Example".to_string(),
            executable: "game.exe".to_string(),
            path_roots: vec![
                GamePathRoot {
                    id: "game".to_string(),
                    name: "Game folder".to_string(),
                    description: "Game install folder".to_string(),
                },
                GamePathRoot {
                    id: "local_app_data".to_string(),
                    name: "Local app data".to_string(),
                    description: "Local app data folder".to_string(),
                },
            ],
            mod_discovery: ModDiscovery {
                root_id: "game".to_string(),
                relative_path: "Data".to_string(),
                mode: ModDiscoveryMode::DirectoryMods {
                    required_prefix: None,
                    metadata_file: None,
                },
            },
            load_order_writes: vec![LoadOrderWriteTarget {
                root_id: "local_app_data".to_string(),
                relative_path: "plugins.txt".to_string(),
            }],
        }
    }

    #[test]
    fn old_game_paths_migrate_to_game_root_plugin_paths() {
        let config: AppConfig = serde_json::from_str(
            r#"{"active_plugin":"skyrim-se","active_theme":"","game_paths":{"skyrim-se":"/games/skyrim"}}"#,
        )
        .expect("deserialize old config");

        assert_eq!(
            config.plugin_paths["skyrim-se"][GAME_INSTALL_PATH_ROOT_ID],
            "/games/skyrim"
        );
    }

    #[test]
    fn configured_path_roots_requires_every_declared_root() {
        let metadata = metadata_with_roots();
        let config = AppConfig {
            active_plugin: None,
            active_theme: String::new(),
            plugin_paths: HashMap::from([(
                "skyrim-se".to_string(),
                HashMap::from([("game".to_string(), "/games/skyrim".to_string())]),
            )]),
        };

        let message = config
            .configured_path_roots("skyrim-se", &metadata)
            .expect_err("missing local app data path should be rejected");

        assert_eq!(
            message,
            "No path configured for 'Local app data' (local_app_data) in plugin 'skyrim-se'"
        );
    }
}
