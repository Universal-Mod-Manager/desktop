use crate::models::{PluginInfo, PluginMetadata};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

struct DiscoveredPlugin {
    info: PluginInfo,
    metadata: PluginMetadata,
    dir: PathBuf,
}

pub struct PluginManager {
    plugins: Vec<DiscoveredPlugin>,
    plugins_dir: PathBuf,
    active_plugin_id: Option<String>,
}

impl PluginManager {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            plugins: Vec::new(),
            plugins_dir: data_dir.join("plugins"),
            active_plugin_id: None,
        }
    }

    pub fn discover_plugins(&mut self) -> Result<()> {
        self.plugins.clear();
        fs::create_dir_all(&self.plugins_dir)?;

        for entry in fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let metadata_path = entry.path().join("metadata.json");
            if !metadata_path.exists() {
                continue;
            }

            let content = fs::read_to_string(&metadata_path)?;
            let metadata: PluginMetadata = serde_json::from_str(&content)?;
            let id = entry.file_name().to_string_lossy().to_string();

            self.plugins.push(DiscoveredPlugin {
                info: PluginInfo {
                    id: id.clone(),
                    name: metadata.name.clone(),
                    icon_path: entry
                        .path()
                        .join(&metadata.icon_path)
                        .to_string_lossy()
                        .to_string(),
                },
                metadata,
                dir: entry.path(),
            });
        }

        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|p| p.info.clone()).collect()
    }

    pub fn select_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if !self.plugins.iter().any(|p| p.info.id == plugin_id) {
            anyhow::bail!("Plugin '{}' not found", plugin_id);
        }
        self.active_plugin_id = Some(plugin_id.to_string());
        Ok(())
    }

    pub fn get_active_plugin_id(&self) -> Option<&str> {
        self.active_plugin_id.as_deref()
    }

    pub fn call_plugin_fn(&self, plugin_id: &str, fn_name: &str, input: &str) -> Result<String> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.info.id == plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", plugin_id))?;

        let wasm_path = plugin.dir.join(&plugin.metadata.wasm_path);
        if !wasm_path.exists() {
            anyhow::bail!("WASM file not found: {}", wasm_path.to_string_lossy());
        }

        let manifest = extism::Manifest::new([extism::Wasm::file(&wasm_path)]);
        let mut ext_plugin = extism::PluginBuilder::new(manifest)
            .with_wasi(true)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to load WASM plugin: {}", e))?;

        let result: String = ext_plugin
            .call(fn_name, input)
            .map_err(|e| anyhow::anyhow!("Plugin call '{}' failed: {}", fn_name, e))?;

        Ok(result)
    }
}
