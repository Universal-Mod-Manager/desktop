use crate::models::{
    BuildLoadOrderInput, BuildLoadOrderOutput, GameMetadata, PluginInfo, PluginMetadata,
    SUPPORTED_PLUGIN_API_VERSION,
};
use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::validate_plugin_relative_path;

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

    pub fn game_metadata(&self, plugin_id: &str) -> Result<GameMetadata> {
        let metadata_json = self.call_plugin_fn(plugin_id, "get_game_metadata", "")?;
        let metadata: GameMetadata = serde_json::from_str(&metadata_json)?;
        validate_game_metadata(plugin_id, &metadata)?;
        Ok(metadata)
    }

    pub fn build_load_order(
        &self,
        plugin_id: &str,
        input: &BuildLoadOrderInput,
    ) -> Result<BuildLoadOrderOutput> {
        let input_json = serde_json::to_string(input)?;
        let output_json = self.call_plugin_fn(plugin_id, "build_load_order", &input_json)?;
        let output: BuildLoadOrderOutput = serde_json::from_str(&output_json)?;

        if output.writes.is_empty() {
            anyhow::bail!("Plugin '{plugin_id}' returned no load-order writes");
        }

        Ok(output)
    }

    fn call_plugin_fn(&self, plugin_id: &str, fn_name: &str, input: &str) -> Result<String> {
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

fn validate_game_metadata(plugin_id: &str, metadata: &GameMetadata) -> Result<()> {
    if metadata.api_version != SUPPORTED_PLUGIN_API_VERSION {
        anyhow::bail!(
            "Plugin '{plugin_id}' uses API version {}, but this app supports version {}",
            metadata.api_version,
            SUPPORTED_PLUGIN_API_VERSION
        );
    }

    if metadata.path_roots.is_empty() {
        anyhow::bail!("Plugin '{plugin_id}' declares no path roots");
    }

    let mut root_ids = HashSet::new();
    for root in &metadata.path_roots {
        if root.id.trim().is_empty() {
            anyhow::bail!("Plugin '{plugin_id}' declares an empty path root id");
        }
        if root.name.trim().is_empty() {
            anyhow::bail!("Plugin '{plugin_id}' declares an empty path root name");
        }
        if !root_ids.insert(root.id.as_str()) {
            anyhow::bail!(
                "Plugin '{plugin_id}' declares duplicate path root '{}'",
                root.id
            );
        }
    }

    if !root_ids.contains(metadata.mod_discovery.root_id.as_str()) {
        anyhow::bail!(
            "Plugin '{plugin_id}' uses undeclared discovery path root '{}'",
            metadata.mod_discovery.root_id
        );
    }
    validate_plugin_relative_path(&metadata.mod_discovery.relative_path)
        .map_err(anyhow::Error::msg)?;

    if metadata.load_order_writes.is_empty() {
        anyhow::bail!("Plugin '{plugin_id}' declares no load-order writes");
    }

    let mut declared_paths = HashSet::new();
    for target in &metadata.load_order_writes {
        if !root_ids.contains(target.root_id.as_str()) {
            anyhow::bail!(
                "Plugin '{plugin_id}' declares load-order path '{}' under undeclared root '{}'",
                target.relative_path,
                target.root_id
            );
        }
        validate_plugin_relative_path(&target.relative_path).map_err(anyhow::Error::msg)?;
        if !declared_paths.insert((target.root_id.as_str(), target.relative_path.as_str())) {
            anyhow::bail!(
                "Plugin '{plugin_id}' declares duplicate load-order path '{}:{}'",
                target.root_id,
                target.relative_path
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        BuildLoadOrderInput, GamePathRoot, LoadOrderWriteTarget, ModDiscovery, ModDiscoveryMode,
        ModEntry,
    };
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

    fn bundled_plugin_manager(
        plugin_id: &str,
        wasm_name: &str,
        target: &str,
    ) -> (TestDir, PluginManager) {
        let temp_dir = TestDir::new(plugin_id);
        let project_root = project_root();
        let plugin_src = project_root.join(format!("plugins/{plugin_id}"));
        let wasm_src = plugin_src.join(format!("target/{target}/release/{wasm_name}.wasm"));

        assert!(
            wasm_src.exists(),
            "bundled plugin WASM missing at '{}'; build the plugin before running this test",
            wasm_src.display()
        );

        let plugin_dest = temp_dir.path.join("plugins").join(plugin_id);
        fs::create_dir_all(&plugin_dest).expect("create plugin test directory");
        fs::copy(
            plugin_src.join("metadata.json"),
            plugin_dest.join("metadata.json"),
        )
        .expect("copy plugin metadata");
        fs::copy(wasm_src, plugin_dest.join("plugin.wasm")).expect("copy plugin wasm");
        fs::write(plugin_dest.join("icon.png"), []).expect("create plugin icon");

        let mut manager = PluginManager::new(&temp_dir.path);
        manager.discover_plugins().expect("discover bundled plugin");
        (temp_dir, manager)
    }

    fn security_test_manager() -> (TestDir, PluginManager) {
        bundled_plugin_manager("security-test", "security_test_plugin", "wasm32-wasip1")
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

    fn valid_metadata() -> GameMetadata {
        GameMetadata {
            api_version: SUPPORTED_PLUGIN_API_VERSION,
            name: "Example".to_string(),
            executable: "game.exe".to_string(),
            path_roots: vec![GamePathRoot {
                id: "game".to_string(),
                name: "Game folder".to_string(),
                description: "Game install folder".to_string(),
            }],
            mod_discovery: ModDiscovery {
                root_id: "game".to_string(),
                relative_path: "mods".to_string(),
                mode: ModDiscoveryMode::DirectoryMods {
                    required_prefix: None,
                    metadata_file: None,
                },
            },
            load_order_writes: vec![LoadOrderWriteTarget {
                root_id: "game".to_string(),
                relative_path: "loadorder.txt".to_string(),
            }],
        }
    }

    fn mod_entry(id: &str, enabled: bool, priority: u32) -> ModEntry {
        ModEntry {
            id: id.to_string(),
            enabled,
            priority,
        }
    }

    #[test]
    fn game_metadata_validation_rejects_unsupported_api_version() {
        let mut metadata = valid_metadata();
        metadata.api_version = 1;

        let message = validate_game_metadata("example", &metadata)
            .expect_err("unsupported API version should be rejected")
            .to_string();

        assert_eq!(
            message,
            "Plugin 'example' uses API version 1, but this app supports version 2"
        );
    }

    #[test]
    fn game_metadata_validation_rejects_empty_load_order_writes() {
        let mut metadata = valid_metadata();
        metadata.load_order_writes.clear();

        let message = validate_game_metadata("example", &metadata)
            .expect_err("empty write targets should be rejected")
            .to_string();

        assert_eq!(message, "Plugin 'example' declares no load-order writes");
    }

    #[test]
    fn game_metadata_validation_rejects_duplicate_path_roots() {
        let mut metadata = valid_metadata();
        metadata.path_roots.push(GamePathRoot {
            id: "game".to_string(),
            name: "Duplicate game folder".to_string(),
            description: "Duplicate root".to_string(),
        });

        let message = validate_game_metadata("example", &metadata)
            .expect_err("duplicate path root should be rejected")
            .to_string();

        assert_eq!(
            message,
            "Plugin 'example' declares duplicate path root 'game'"
        );
    }

    #[test]
    fn game_metadata_validation_rejects_undeclared_write_roots() {
        let mut metadata = valid_metadata();
        metadata.load_order_writes[0].root_id = "documents".to_string();

        let message = validate_game_metadata("example", &metadata)
            .expect_err("undeclared write root should be rejected")
            .to_string();

        assert_eq!(
            message,
            "Plugin 'example' declares load-order path 'loadorder.txt' under undeclared root 'documents'"
        );
    }

    #[test]
    fn game_metadata_validation_rejects_duplicate_write_targets() {
        let mut metadata = valid_metadata();
        metadata.load_order_writes.push(LoadOrderWriteTarget {
            root_id: "game".to_string(),
            relative_path: "loadorder.txt".to_string(),
        });

        let message = validate_game_metadata("example", &metadata)
            .expect_err("duplicate write target should be rejected")
            .to_string();

        assert_eq!(
            message,
            "Plugin 'example' declares duplicate load-order path 'game:loadorder.txt'"
        );
    }

    #[test]
    fn game_metadata_validation_rejects_unsafe_write_targets() {
        let mut metadata = valid_metadata();
        metadata.load_order_writes = vec![LoadOrderWriteTarget {
            root_id: "game".to_string(),
            relative_path: "../outside.txt".to_string(),
        }];

        let message = validate_game_metadata("example", &metadata)
            .expect_err("unsafe write target should be rejected")
            .to_string();

        assert!(
            message.contains("Plugin returned unsafe relative_path: ../outside.txt"),
            "unexpected unsafe path error: {message}"
        );
    }

    #[test]
    fn skyrim_plugin_wasm_round_trip_builds_declared_load_order_writes() {
        let (_temp_dir, manager) =
            bundled_plugin_manager("skyrim-se", "skyrim_se_plugin", "wasm32-unknown-unknown");
        let metadata = manager
            .game_metadata("skyrim-se")
            .expect("load Skyrim metadata");
        let input = BuildLoadOrderInput {
            mods: vec![
                mod_entry("ELFX.esp", false, 30),
                mod_entry("WeatherOverhaul.esp", true, 10),
            ],
        };

        let output = manager
            .build_load_order("skyrim-se", &input)
            .expect("build Skyrim load order");
        let declared_paths = metadata
            .load_order_writes
            .iter()
            .map(|target| (target.root_id.as_str(), target.relative_path.as_str()))
            .collect::<Vec<_>>();

        assert!(declared_paths.contains(&("local_app_data", "plugins.txt")));
        assert!(declared_paths.contains(&("local_app_data", "loadorder.txt")));
        assert_eq!(output.writes.len(), 2);

        let plugins_txt = output
            .writes
            .iter()
            .find(|write| write.root_id == "local_app_data" && write.relative_path == "plugins.txt")
            .expect("plugins.txt write");
        assert_eq!(
            plugins_txt.content,
            "*Skyrim.esm\n*Update.esm\n*Dawnguard.esm\n*HearthFires.esm\n*Dragonborn.esm\n*WeatherOverhaul.esp\nELFX.esp\n"
        );
    }

    #[test]
    fn witcher_plugin_wasm_round_trip_builds_declared_mods_settings() {
        let (_temp_dir, manager) =
            bundled_plugin_manager("witcher3", "witcher3_plugin", "wasm32-unknown-unknown");
        let metadata = manager
            .game_metadata("witcher3")
            .expect("load Witcher metadata");
        let input = BuildLoadOrderInput {
            mods: vec![
                mod_entry("modBetterWeather", false, 20),
                mod_entry("modArmorEnhanced", true, 10),
            ],
        };

        let output = manager
            .build_load_order("witcher3", &input)
            .expect("build Witcher load order");

        assert_eq!(metadata.load_order_writes[0].root_id, "documents");
        assert_eq!(metadata.load_order_writes[0].relative_path, "mods.settings");
        assert_eq!(output.writes.len(), 1);
        assert_eq!(
            output.writes[0].content,
            "[modArmorEnhanced]\nEnabled=1\nPriority=1\n\n[modBetterWeather]\nEnabled=0\nPriority=2\n"
        );
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
