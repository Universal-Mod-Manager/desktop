use crate::models::{BuildLoadOrderInput, BuildLoadOrderOutput, GameMetadata, ModEntry, ModInfo};
use crate::services::join_validated_plugin_path;
use crate::state::AppState;
use std::collections::{HashMap, HashSet};
use std::path::Path;

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
    let plugin_id = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        match config.active_plugin.as_deref() {
            Some(id) => id.to_string(),
            None => return Ok(()),
        }
    };

    let mods = {
        let mgr = state.mod_manager.lock().map_err(|e| e.to_string())?;
        mgr.list_mods()
            .into_iter()
            .map(|game_mod| ModEntry {
                id: game_mod.id,
                enabled: game_mod.enabled,
                priority: game_mod.priority,
            })
            .collect()
    };
    let input = BuildLoadOrderInput { mods };

    let (metadata, output) = {
        let plugin_mgr = state.plugin_manager.lock().map_err(|e| e.to_string())?;
        let metadata = plugin_mgr
            .game_metadata(&plugin_id)
            .map_err(|e| e.to_string())?;
        let output = plugin_mgr
            .build_load_order(&plugin_id, &input)
            .map_err(|e| e.to_string())?;
        (metadata, output)
    };

    let path_roots = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.configured_path_roots(&plugin_id, &metadata)?
    };
    write_declared_load_order_files(&plugin_id, &path_roots, &metadata, output)?;

    Ok(())
}

fn write_declared_load_order_files(
    plugin_id: &str,
    path_roots: &HashMap<String, String>,
    metadata: &GameMetadata,
    output: BuildLoadOrderOutput,
) -> Result<(), String> {
    let declared_paths = declared_load_order_paths(metadata);
    for write in output.writes {
        ensure_declared_load_order_path(
            plugin_id,
            &declared_paths,
            &write.root_id,
            &write.relative_path,
        )?;
        let root_path = path_roots.get(&write.root_id).ok_or_else(|| {
            format!(
                "No path configured for root '{}' in plugin '{}'",
                write.root_id, plugin_id
            )
        })?;

        let file_path = join_validated_plugin_path(Path::new(root_path), &write.relative_path)?;
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&file_path, write.content).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn declared_load_order_paths(metadata: &GameMetadata) -> HashSet<(&str, &str)> {
    metadata
        .load_order_writes
        .iter()
        .map(|target| (target.root_id.as_str(), target.relative_path.as_str()))
        .collect()
}

fn ensure_declared_load_order_path(
    plugin_id: &str,
    declared_paths: &HashSet<(&str, &str)>,
    root_id: &str,
    relative_path: &str,
) -> Result<(), String> {
    if declared_paths.contains(&(root_id, relative_path)) {
        return Ok(());
    }

    Err(format!(
        "Plugin '{plugin_id}' returned undeclared load-order path '{root_id}:{relative_path}'"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        GameFileWrite, GamePathRoot, LoadOrderWriteTarget, ModDiscovery, ModDiscoveryMode,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before UNIX_EPOCH")
                .as_nanos();
            let id = NEXT_TEST_DIR_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("umm-{name}-{}-{nanos}-{id}", std::process::id()));
            fs::create_dir_all(&path).expect("create temp test directory");
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn metadata_with_write_targets(targets: &[(&str, &str)]) -> GameMetadata {
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
                relative_path: "mods".to_string(),
                mode: ModDiscoveryMode::DirectoryMods {
                    required_prefix: None,
                    metadata_file: None,
                },
            },
            load_order_writes: targets
                .iter()
                .map(|(root_id, relative_path)| LoadOrderWriteTarget {
                    root_id: (*root_id).to_string(),
                    relative_path: (*relative_path).to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn load_order_write_path_must_be_declared_by_plugin_metadata() {
        let metadata = metadata_with_write_targets(&[("local_app_data", "declared.txt")]);
        let declared_paths = declared_load_order_paths(&metadata);

        assert!(ensure_declared_load_order_path(
            "example",
            &declared_paths,
            "local_app_data",
            "declared.txt"
        )
        .is_ok());
        assert_eq!(
            ensure_declared_load_order_path("example", &declared_paths, "game", "other.txt")
                .expect_err("undeclared write target should be rejected"),
            "Plugin 'example' returned undeclared load-order path 'game:other.txt'"
        );
    }

    #[test]
    fn declared_load_order_writes_create_nested_files() {
        let temp_dir = TestDir::new("declared-load-order-write");
        let relative_path = "plugins.txt";
        let metadata = metadata_with_write_targets(&[("local_app_data", relative_path)]);
        let output = BuildLoadOrderOutput {
            writes: vec![GameFileWrite {
                root_id: "local_app_data".to_string(),
                relative_path: relative_path.to_string(),
                content: "*Skyrim.esm\n".to_string(),
            }],
        };

        let path_roots = HashMap::from([(
            "local_app_data".to_string(),
            temp_dir.path.to_string_lossy().to_string(),
        )]);
        write_declared_load_order_files("skyrim-se", &path_roots, &metadata, output)
            .expect("write declared load-order file");

        let content =
            fs::read_to_string(temp_dir.path.join(relative_path)).expect("read load-order file");
        assert_eq!(content, "*Skyrim.esm\n");
    }
}
