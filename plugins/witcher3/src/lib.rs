use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GameMetadata {
    name: String,
    mod_directory: String,
    mod_extension: String,
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

#[plugin_fn]
pub fn get_game_name() -> FnResult<String> {
    Ok("The Witcher 3: Wild Hunt".to_string())
}

#[plugin_fn]
pub fn get_game_metadata() -> FnResult<String> {
    let metadata = GameMetadata {
        name: "The Witcher 3: Wild Hunt".to_string(),
        mod_directory: "Data".to_string(),
        mod_extension: ".data".to_string(),
        load_order_file: "order.txt".to_string(),
        executable: "witcher3.exe".to_string(),
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
        relative_path: "order.txt".to_string(),
        content,
    };
    Ok(serde_json::to_string(&output)?)
}

#[plugin_fn]
pub fn read_load_order(content: String) -> FnResult<String> {
    let order: Vec<String> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();
    Ok(serde_json::to_string(&order)?)
}
