use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GameMetadata {
    name: String,
    mod_directory: String,
    load_order_file: String,
    executable: String,
}

#[derive(Deserialize)]
struct ModEntry {
    id: String,
    enabled: bool,
    priority: u32,
}

#[derive(Deserialize)]
struct WriteLoadOrderInput {
    mods: Vec<ModEntry>,
}

#[derive(Serialize)]
struct WriteLoadOrderOutput {
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
pub fn get_game_name() -> FnResult<String> {
    Ok("Security Test Plugin".to_string())
}

#[plugin_fn]
pub fn get_game_metadata() -> FnResult<String> {
    let metadata = GameMetadata {
        name: "Security Test Plugin".to_string(),
        mod_directory: "mods".to_string(),
        load_order_file: "security-loadorder.txt".to_string(),
        executable: "security-test.exe".to_string(),
    };
    Ok(serde_json::to_string(&metadata)?)
}

#[plugin_fn]
pub fn write_load_order(input: String) -> FnResult<String> {
    let data: WriteLoadOrderInput = serde_json::from_str(&input)?;

    let mut enabled: Vec<&ModEntry> = data.mods.iter().filter(|m| m.enabled).collect();
    enabled.sort_by_key(|m| m.priority);

    let content = enabled
        .iter()
        .map(|m| m.id.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let output = WriteLoadOrderOutput {
        relative_path: "security-loadorder.txt".to_string(),
        content,
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
