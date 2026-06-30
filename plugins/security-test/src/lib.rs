use extism_pdk::*;
use serde::{Deserialize, Serialize};

const GAME_ROOT_ID: &str = "game";

#[derive(Serialize)]
struct GameMetadata {
    api_version: u32,
    name: String,
    executable: String,
    path_roots: Vec<GamePathRoot>,
    mod_discovery: ModDiscovery,
    load_order_writes: Vec<LoadOrderWriteTarget>,
}

#[derive(Serialize)]
struct GamePathRoot {
    id: String,
    name: String,
    description: String,
}

#[derive(Serialize)]
struct ModDiscovery {
    root_id: String,
    relative_path: String,
    mode: ModDiscoveryMode,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ModDiscoveryMode {
    DirectoryMods {
        required_prefix: Option<String>,
        metadata_file: Option<String>,
    },
}

#[derive(Serialize)]
struct LoadOrderWriteTarget {
    root_id: String,
    relative_path: String,
}

#[derive(Deserialize)]
struct ModEntry {
    id: String,
    enabled: bool,
    priority: u32,
}

#[derive(Deserialize)]
struct BuildLoadOrderInput {
    mods: Vec<ModEntry>,
}

#[derive(Serialize)]
struct BuildLoadOrderOutput {
    writes: Vec<GameFileWrite>,
}

#[derive(Serialize)]
struct GameFileWrite {
    root_id: String,
    relative_path: String,
    content: String,
}

#[derive(Serialize)]
struct ProbeResult {
    operation: String,
    success: bool,
    detail: String,
}

#[derive(Serialize)]
struct ProbeReport {
    results: Vec<ProbeResult>,
}

fn probe_result(operation: &str, result: Result<String, String>) -> ProbeResult {
    match result {
        Ok(detail) => ProbeResult {
            operation: operation.to_string(),
            success: true,
            detail,
        },
        Err(detail) => ProbeResult {
            operation: operation.to_string(),
            success: false,
            detail,
        },
    }
}

#[plugin_fn]
pub fn get_game_metadata() -> FnResult<String> {
    let metadata = GameMetadata {
        api_version: 2,
        name: "Security Test Plugin".to_string(),
        executable: "security-test.exe".to_string(),
        path_roots: vec![GamePathRoot {
            id: GAME_ROOT_ID.to_string(),
            name: "Security test game folder".to_string(),
            description: "Folder used for security probe mod discovery.".to_string(),
        }],
        mod_discovery: ModDiscovery {
            root_id: GAME_ROOT_ID.to_string(),
            relative_path: "mods".to_string(),
            mode: ModDiscoveryMode::DirectoryMods {
                required_prefix: None,
                metadata_file: None,
            },
        },
        load_order_writes: vec![LoadOrderWriteTarget {
            root_id: GAME_ROOT_ID.to_string(),
            relative_path: "security-loadorder.txt".to_string(),
        }],
    };
    Ok(serde_json::to_string(&metadata)?)
}

#[plugin_fn]
pub fn build_load_order(input: String) -> FnResult<String> {
    let input: BuildLoadOrderInput = serde_json::from_str(&input)?;
    let mut enabled: Vec<&ModEntry> = input
        .mods
        .iter()
        .filter(|game_mod| game_mod.enabled)
        .collect();
    enabled.sort_by_key(|game_mod| game_mod.priority);

    let mut content = enabled
        .iter()
        .map(|game_mod| game_mod.id.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if !content.is_empty() {
        content.push('\n');
    }

    let output = BuildLoadOrderOutput {
        writes: vec![GameFileWrite {
            root_id: GAME_ROOT_ID.to_string(),
            relative_path: "security-loadorder.txt".to_string(),
            content,
        }],
    };
    Ok(serde_json::to_string(&output)?)
}

#[plugin_fn]
pub fn probe_http() -> FnResult<String> {
    let req = HttpRequest::new("https://example.com").with_method("GET");
    let result = match http::request::<()>(&req, None) {
        Ok(_) => Ok("HTTP request unexpectedly succeeded".to_string()),
        Err(err) => Err(err.to_string()),
    };

    Ok(serde_json::to_string(&ProbeReport {
        results: vec![probe_result("http_get_example", result)],
    })?)
}

#[plugin_fn]
pub fn probe_filesystem() -> FnResult<String> {
    let test_file = "plugin-security-created.txt";

    let list_result = match std::fs::read_dir(".") {
        Ok(entries) => Ok(format!(
            "listed {} visible entries",
            entries.filter_map(Result::ok).count()
        )),
        Err(err) => Err(err.to_string()),
    };

    let create_result = match std::fs::write(test_file, "plugin security probe") {
        Ok(_) => Ok(format!("created {test_file}")),
        Err(err) => Err(err.to_string()),
    };

    let delete_result = match std::fs::remove_file(test_file) {
        Ok(_) => Ok(format!("deleted {test_file}")),
        Err(err) => Err(err.to_string()),
    };

    Ok(serde_json::to_string(&ProbeReport {
        results: vec![
            probe_result("list_current_directory", list_result),
            probe_result("create_file", create_result),
            probe_result("delete_file", delete_result),
        ],
    })?)
}
