use crate::models::{ModEntry, ModInfo, ModState};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

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
        game_path: &str,
        mod_directory: &str,
    ) -> Result<()> {
        self.current_plugin = Some(plugin_id.to_string());

        let mods_path = Path::new(game_path).join(mod_directory);
        let mut discovered = if mods_path.exists() {
            let has_dir_mods = fs::read_dir(&mods_path)?.filter_map(|e| e.ok()).any(|e| {
                e.file_type().is_ok_and(|ft| ft.is_dir()) && e.path().join("mod.json").exists()
            });

            if has_dir_mods {
                self.discover_directory_mods(&mods_path)?
            } else {
                self.discover_file_mods(&mods_path)?
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

    fn discover_directory_mods(&self, mods_path: &Path) -> Result<Vec<ModInfo>> {
        let mut discovered = Vec::new();
        for entry in fs::read_dir(mods_path)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let mod_json = entry.path().join("mod.json");
            if !mod_json.exists() {
                continue;
            }
            let content = fs::read_to_string(&mod_json)?;
            let meta: serde_json::Value = serde_json::from_str(&content)?;
            let mod_id = entry.file_name().to_string_lossy().to_string();
            discovered.push(ModInfo {
                id: mod_id.clone(),
                name: meta["name"].as_str().unwrap_or(&mod_id).to_string(),
                version: meta["version"].as_str().unwrap_or("1.0").to_string(),
                description: meta["description"].as_str().unwrap_or("").to_string(),
                enabled: true,
                priority: discovered.len() as u32,
            });
        }
        Ok(discovered)
    }

    fn discover_file_mods(&self, mods_path: &Path) -> Result<Vec<ModInfo>> {
        let mut discovered = Vec::new();
        for entry in fs::read_dir(mods_path)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "txt" || ext == "json" || ext == "ini" {
                continue;
            }
            let mod_id = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let display_name = mod_id.chars().fold(String::new(), |mut acc, c| {
                if c.is_uppercase() && !acc.is_empty() {
                    acc.push(' ');
                }
                acc.push(c);
                acc
            });
            discovered.push(ModInfo {
                id: mod_id,
                name: display_name,
                version: "1.0".to_string(),
                description: format!(".{} mod", ext),
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
