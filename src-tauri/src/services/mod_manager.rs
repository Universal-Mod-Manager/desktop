use crate::models::{ModDiscovery, ModDiscoveryMode, ModEntry, ModInfo, ModState};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::join_validated_plugin_path;

pub struct ModManager {
    mods: Vec<ModInfo>,
    profile_dir: PathBuf,
    current_plugin: Option<String>,
}

impl ModManager {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            mods: Vec::new(),
            profile_dir: data_dir.join("profiles"),
            current_plugin: None,
        }
    }

    pub fn load_mods_for_plugin(
        &mut self,
        plugin_id: &str,
        path_roots: &HashMap<String, String>,
        discovery: &ModDiscovery,
    ) -> Result<()> {
        self.current_plugin = Some(plugin_id.to_string());

        let root_path = path_roots.get(&discovery.root_id).ok_or_else(|| {
            anyhow!(
                "No path configured for root '{}' in plugin '{}'",
                discovery.root_id,
                plugin_id
            )
        })?;
        let mods_path = join_validated_plugin_path(Path::new(root_path), &discovery.relative_path)
            .map_err(anyhow::Error::msg)?;
        let mut discovered = if mods_path.exists() {
            match &discovery.mode {
                ModDiscoveryMode::DirectoryMods {
                    required_prefix,
                    metadata_file,
                } => self.discover_directory_mods(
                    &mods_path,
                    required_prefix.as_deref(),
                    metadata_file.as_deref(),
                )?,
                ModDiscoveryMode::PluginFiles {
                    extensions,
                    excluded_files,
                } => self.discover_plugin_files(&mods_path, extensions, excluded_files)?,
            }
        } else {
            Vec::new()
        };

        let state_path = self.profile_dir.join(plugin_id).join("modstate.json");
        if state_path.exists() {
            let content = fs::read_to_string(&state_path)?;
            let saved: ModState = serde_json::from_str(&content)?;
            for entry in &saved.mods {
                if let Some(m) = discovered.iter_mut().find(|m| m.id == entry.id) {
                    m.enabled = entry.enabled;
                    m.priority = entry.priority;
                }
            }
        }

        discovered.sort_by_key(|m| m.priority);
        self.mods = discovered;
        Ok(())
    }

    fn discover_directory_mods(
        &self,
        mods_path: &Path,
        required_prefix: Option<&str>,
        metadata_file: Option<&str>,
    ) -> Result<Vec<ModInfo>> {
        let mut discovered = Vec::new();
        for entry in fs::read_dir(mods_path)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let mod_id = entry.file_name().to_string_lossy().to_string();
            if required_prefix.is_some_and(|prefix| !mod_id.starts_with(prefix)) {
                continue;
            }

            let (name, version, description) = match metadata_file {
                Some(file_name) => {
                    let metadata_path = entry.path().join(file_name);
                    if metadata_path.exists() {
                        Self::read_directory_mod_metadata(&mod_id, &metadata_path)?
                    } else {
                        Self::directory_mod_defaults(&mod_id)
                    }
                }
                None => Self::directory_mod_defaults(&mod_id),
            };

            discovered.push(ModInfo {
                id: mod_id,
                name,
                version,
                description,
                enabled: true,
                priority: discovered.len() as u32,
            });
        }
        Ok(discovered)
    }

    fn directory_mod_defaults(mod_id: &str) -> (String, String, String) {
        (
            mod_id.to_string(),
            "1.0".to_string(),
            "Directory mod".to_string(),
        )
    }

    fn read_directory_mod_metadata(
        mod_id: &str,
        metadata_path: &Path,
    ) -> Result<(String, String, String)> {
        let content = fs::read_to_string(metadata_path).map_err(|err| {
            anyhow!(
                "Failed to read mod metadata '{}': {err}",
                metadata_path.display()
            )
        })?;
        let metadata: serde_json::Value = serde_json::from_str(&content).map_err(|err| {
            anyhow!(
                "Failed to parse mod metadata '{}': {err}",
                metadata_path.display()
            )
        })?;

        Ok((
            metadata["name"].as_str().unwrap_or(mod_id).to_string(),
            metadata["version"].as_str().unwrap_or("1.0").to_string(),
            metadata["description"].as_str().unwrap_or("").to_string(),
        ))
    }

    fn discover_plugin_files(
        &self,
        mods_path: &Path,
        extensions: &[String],
        excluded_files: &[String],
    ) -> Result<Vec<ModInfo>> {
        let allowed_extensions: Vec<String> = extensions
            .iter()
            .map(|extension| extension.trim_start_matches('.').to_ascii_lowercase())
            .filter(|extension| !extension.is_empty())
            .collect();

        let mut discovered = Vec::new();
        for entry in fs::read_dir(mods_path)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }

            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            if excluded_files.iter().any(|excluded| excluded == &file_name) {
                continue;
            }

            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if !allowed_extensions
                .iter()
                .any(|allowed| allowed == &extension.to_ascii_lowercase())
            {
                continue;
            }

            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            discovered.push(ModInfo {
                id: file_name,
                name,
                version: "1.0".to_string(),
                description: format!(".{extension} plugin"),
                enabled: true,
                priority: discovered.len() as u32,
            });
        }
        Ok(discovered)
    }

    pub fn list_mods(&self) -> Vec<ModInfo> {
        self.mods.clone()
    }

    pub fn toggle_mod(&mut self, mod_id: &str, enabled: bool) -> Result<()> {
        if let Some(m) = self.mods.iter_mut().find(|m| m.id == mod_id) {
            m.enabled = enabled;
            self.save()?;
        }
        Ok(())
    }

    pub fn reorder_mods(&mut self, mod_ids: Vec<String>) -> Result<()> {
        let mut reordered = Vec::new();
        for (i, id) in mod_ids.iter().enumerate() {
            if let Some(m) = self.mods.iter().find(|m| m.id == *id) {
                let mut m = m.clone();
                m.priority = i as u32;
                reordered.push(m);
            }
        }
        self.mods = reordered;
        self.save()?;
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let plugin_id = match &self.current_plugin {
            Some(id) => id,
            None => return Ok(()),
        };
        let profile_dir = self.profile_dir.join(plugin_id);
        fs::create_dir_all(&profile_dir)?;
        let state = ModState {
            mods: self
                .mods
                .iter()
                .map(|m| ModEntry {
                    id: m.id.clone(),
                    enabled: m.enabled,
                    priority: m.priority,
                })
                .collect(),
        };
        let content = serde_json::to_string_pretty(&state)?;
        fs::write(profile_dir.join("modstate.json"), content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
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

    fn touch(path: &Path) {
        fs::create_dir_all(path.parent().expect("test file should have a parent"))
            .expect("create test file parent");
        fs::write(path, []).expect("write test file");
    }

    fn game_path_roots(game_root: &Path) -> HashMap<String, String> {
        HashMap::from([("game".to_string(), game_root.to_string_lossy().to_string())])
    }

    #[test]
    fn plugin_file_discovery_excludes_official_masters_and_keeps_extensions() {
        let temp_dir = TestDir::new("plugin-file-discovery");
        let game_root = temp_dir.path.join("game");
        touch(&game_root.join("Data/Skyrim.esm"));
        touch(&game_root.join("Data/Update.esm"));
        touch(&game_root.join("Data/SkyUI_SE.esp"));
        touch(&game_root.join("Data/ELFX.ESP"));
        touch(&game_root.join("Data/readme.txt"));

        let discovery = ModDiscovery {
            root_id: "game".to_string(),
            relative_path: "Data".to_string(),
            mode: ModDiscoveryMode::PluginFiles {
                extensions: vec!["esm".to_string(), ".esp".to_string(), "esl".to_string()],
                excluded_files: vec![
                    "Skyrim.esm".to_string(),
                    "Update.esm".to_string(),
                    "Dawnguard.esm".to_string(),
                    "HearthFires.esm".to_string(),
                    "Dragonborn.esm".to_string(),
                ],
            },
        };
        let mut manager = ModManager::new(&temp_dir.path);
        let path_roots = game_path_roots(&game_root);
        manager
            .load_mods_for_plugin("skyrim-se", &path_roots, &discovery)
            .expect("discover plugin files");

        let mut ids = manager
            .list_mods()
            .into_iter()
            .map(|game_mod| game_mod.id)
            .collect::<Vec<_>>();
        ids.sort();

        assert_eq!(ids, vec!["ELFX.ESP", "SkyUI_SE.esp"]);
    }

    #[test]
    fn directory_mod_discovery_requires_prefix_and_uses_directory_id() {
        let temp_dir = TestDir::new("directory-mod-discovery");
        let game_root = temp_dir.path.join("game");
        fs::create_dir_all(game_root.join("mods/modHDCharacters/content"))
            .expect("create Witcher mod directory");
        fs::create_dir_all(game_root.join("mods/modArmorEnhanced/content"))
            .expect("create Witcher mod directory");
        fs::create_dir_all(game_root.join("mods/BetterWeather/content"))
            .expect("create non-matching directory");
        touch(&game_root.join("mods/modHDCharacters/content/texture.cache"));
        touch(&game_root.join("mods/modArmorEnhanced/content/blob0.bundle"));

        let discovery = ModDiscovery {
            root_id: "game".to_string(),
            relative_path: "mods".to_string(),
            mode: ModDiscoveryMode::DirectoryMods {
                required_prefix: Some("mod".to_string()),
                metadata_file: None,
            },
        };
        let mut manager = ModManager::new(&temp_dir.path);
        let path_roots = game_path_roots(&game_root);
        manager
            .load_mods_for_plugin("witcher3", &path_roots, &discovery)
            .expect("discover directory mods");

        let mut mods = manager.list_mods();
        mods.sort_by(|left, right| left.id.cmp(&right.id));

        assert_eq!(
            mods.iter()
                .map(|game_mod| game_mod.id.as_str())
                .collect::<Vec<_>>(),
            vec!["modArmorEnhanced", "modHDCharacters"]
        );
        assert!(mods.iter().all(|game_mod| game_mod.name == game_mod.id));
        assert!(mods
            .iter()
            .all(|game_mod| game_mod.description == "Directory mod"));
    }
}
