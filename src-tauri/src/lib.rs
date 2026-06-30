mod commands;
mod models;
mod services;
mod state;

use models::{AppConfig, GAME_INSTALL_PATH_ROOT_ID};
use services::{ModManager, PluginManager, ThemeManager};
use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            commands::mods::list_mods,
            commands::mods::toggle_mod,
            commands::mods::reorder_mods,
            commands::plugins::list_plugins,
            commands::plugins::get_active_plugin,
            commands::plugins::select_plugin,
            commands::settings::get_plugin_path_roots,
            commands::settings::get_plugin_paths,
            commands::settings::set_plugin_path,
            commands::themes::list_themes,
            commands::themes::get_theme_css,
            commands::themes::get_active_theme,
            commands::themes::set_active_theme,
        ])
        .events(tauri_specta::collect_events![]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            std::fs::create_dir_all(data_dir.join("plugins"))?;
            std::fs::create_dir_all(data_dir.join("profiles"))?;

            let config_path = data_dir.join("config.json");
            let mut config: AppConfig = if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                AppConfig::default()
            };

            #[cfg(debug_assertions)]
            setup_dev_environment(&data_dir, &mut config);

            let content = serde_json::to_string_pretty(&config)?;
            std::fs::write(&config_path, content)?;

            let theme_manager = ThemeManager::new(&data_dir, &config.active_theme);

            let mut plugin_manager = PluginManager::new(&data_dir);
            plugin_manager.discover_plugins()?;

            let mut mod_manager = ModManager::new(&data_dir);
            if let Some(plugin_id) = &config.active_plugin {
                if let Ok(metadata) = plugin_manager.game_metadata(plugin_id) {
                    if let Ok(path_roots) = config.configured_path_roots(plugin_id, &metadata) {
                        let _ = mod_manager.load_mods_for_plugin(
                            plugin_id,
                            &path_roots,
                            &metadata.mod_discovery,
                        );
                    }
                }
            }

            app.manage(AppState {
                mod_manager: Mutex::new(mod_manager),
                plugin_manager: Mutex::new(plugin_manager),
                theme_manager: Mutex::new(theme_manager),
                config: Mutex::new(config),
                data_dir,
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(debug_assertions)]
fn setup_dev_environment(data_dir: &std::path::Path, config: &mut AppConfig) {
    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    for (plugin_id, roots) in [
        (
            "skyrim-se",
            &[
                (GAME_INSTALL_PATH_ROOT_ID, "skyrim-se"),
                ("local_app_data", "skyrim-se-local-app-data"),
            ][..],
        ),
        (
            "witcher3",
            &[
                (GAME_INSTALL_PATH_ROOT_ID, "witcher3"),
                ("documents", "witcher3-documents"),
            ][..],
        ),
        (
            "security-test",
            &[(GAME_INSTALL_PATH_ROOT_ID, "security-test")][..],
        ),
    ] {
        for (root_id, dir_name) in roots {
            let has_path = config
                .plugin_paths
                .get(plugin_id)
                .and_then(|paths| paths.get(*root_id))
                .is_some();
            if has_path {
                continue;
            }

            let fake_path = project_root.join(format!(".ignored/fake-games/{dir_name}"));
            if fake_path.exists() {
                config
                    .plugin_paths
                    .entry(plugin_id.to_string())
                    .or_default()
                    .insert(root_id.to_string(), fake_path.to_string_lossy().to_string());
            }
        }
    }

    let themes_src = project_root.join("themes");
    let themes_dest = data_dir.join("themes");
    if themes_src.exists() {
        let _ = std::fs::create_dir_all(&themes_dest);
        if let Ok(entries) = std::fs::read_dir(&themes_src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "css") {
                    let _ = std::fs::copy(&path, themes_dest.join(entry.file_name()));
                }
            }
        }
    }

    for (id, wasm_name, target) in [
        ("skyrim-se", "skyrim_se_plugin", "wasm32-unknown-unknown"),
        ("witcher3", "witcher3_plugin", "wasm32-unknown-unknown"),
        ("security-test", "security_test_plugin", "wasm32-wasip1"),
    ] {
        let plugin_dest = data_dir.join("plugins").join(id);
        let plugin_src = project_root.join(format!("plugins/{}", id));
        if plugin_src.join("metadata.json").exists() {
            let _ = std::fs::create_dir_all(&plugin_dest);
            let _ = std::fs::copy(
                plugin_src.join("metadata.json"),
                plugin_dest.join("metadata.json"),
            );
            let wasm_src = plugin_src.join(format!("target/{}/release/{}.wasm", target, wasm_name));
            if wasm_src.exists() {
                let _ = std::fs::copy(&wasm_src, plugin_dest.join("plugin.wasm"));
            }
            if !plugin_dest.join("icon.png").exists() {
                let _ = std::fs::write(plugin_dest.join("icon.png"), []);
            }
        }
    }
}
