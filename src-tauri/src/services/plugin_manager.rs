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

        if plugin.metadata.wasm_path.trim().is_empty() {
            anyhow::bail!(
                "Plugin '{}' metadata.json has an empty wasm_path",
                plugin_id
            );
        }

        let wasm_path = plugin.dir.join(&plugin.metadata.wasm_path);
        if !wasm_path.exists() {
            anyhow::bail!(
                "Plugin '{}' declares wasm_path '{}' in metadata.json, but the WASM file was not found at '{}'. Build the plugin or fix metadata.json.",
                plugin_id,
                plugin.metadata.wasm_path,
                wasm_path.to_string_lossy()
            );
        }

        let manifest = extism::Manifest::new([extism::Wasm::file(&wasm_path)]).disallow_all_hosts();
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
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

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("src-tauri should have a parent directory")
            .to_path_buf()
    }

    fn security_test_manager() -> (TestDir, PluginManager) {
        let temp_dir = TestDir::new("plugin-security");
        let project_root = project_root();
        let plugin_src = project_root.join("plugins/security-test");
        let wasm_src = plugin_src.join("target/wasm32-wasip1/release/security_test_plugin.wasm");

        assert!(
            wasm_src.exists(),
            "security-test WASM missing; run `cargo build --release --target wasm32-wasip1` in plugins/security-test first"
        );

        let plugin_dest = temp_dir.path.join("plugins/security-test");
        fs::create_dir_all(&plugin_dest).expect("create plugin test directory");
        fs::copy(
            plugin_src.join("metadata.json"),
            plugin_dest.join("metadata.json"),
        )
        .expect("copy plugin metadata");
        fs::copy(wasm_src, plugin_dest.join("plugin.wasm")).expect("copy plugin wasm");
        fs::write(plugin_dest.join("icon.png"), []).expect("create plugin icon");

        let mut manager = PluginManager::new(&temp_dir.path);
        manager
            .discover_plugins()
            .expect("discover security-test plugin");
        (temp_dir, manager)
    }

    fn assert_probe_blocked(json: &str, expected_operations: &[&str]) {
        let value: Value = serde_json::from_str(json).expect("probe output should be JSON");
        let results = value["results"]
            .as_array()
            .expect("probe output should include results array");

        for expected_operation in expected_operations {
            let result = results
                .iter()
                .find(|item| item["operation"].as_str() == Some(expected_operation))
                .unwrap_or_else(|| panic!("missing probe operation: {expected_operation}"));

            assert_eq!(
                result["success"].as_bool(),
                Some(false),
                "probe operation should be blocked: {expected_operation}, output: {json}"
            );
        }
    }

    #[test]
    fn plugin_call_reports_declared_missing_wasm_path() {
        let temp_dir = TestDir::new("missing-plugin-wasm");
        let plugin_dest = temp_dir.path.join("plugins/example");
        fs::create_dir_all(&plugin_dest).expect("create plugin test directory");
        fs::write(
            plugin_dest.join("metadata.json"),
            r#"{"name":"Example","wasm_path":"plugin.wasm","icon_path":"icon.png"}"#,
        )
        .expect("write plugin metadata");

        let mut manager = PluginManager::new(&temp_dir.path);
        manager.discover_plugins().expect("discover example plugin");

        let message = manager
            .call_plugin_fn("example", "get_game_metadata", "")
            .expect_err("missing declared wasm should return an error")
            .to_string();

        assert!(
            message.contains("Plugin 'example' declares wasm_path 'plugin.wasm' in metadata.json"),
            "unexpected missing WASM error: {message}"
        );
    }

    #[test]
    fn security_test_plugin_cannot_make_http_requests() {
        let (_temp_dir, manager) = security_test_manager();

        match manager.call_plugin_fn("security-test", "probe_http", "") {
            Ok(output) => assert_probe_blocked(&output, &["http_get_example"]),
            Err(err) => {
                let message = err.to_string();
                assert!(
                    message.contains("Plugin call 'probe_http' failed"),
                    "unexpected HTTP probe failure: {message}"
                );
            }
        }
    }

    #[test]
    fn security_test_plugin_cannot_access_raw_filesystem() {
        let (_temp_dir, manager) = security_test_manager();

        let output = manager
            .call_plugin_fn("security-test", "probe_filesystem", "")
            .expect("call filesystem security probe");

        assert_probe_blocked(
            &output,
            &["list_current_directory", "create_file", "delete_file"],
        );
    }
}
