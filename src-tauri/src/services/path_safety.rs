use std::path::{Component, Path, PathBuf};

pub fn validate_plugin_relative_path(relative_path: &str) -> Result<(), String> {
    if relative_path.trim().is_empty() {
        return Err("Plugin returned an empty relative_path".to_string());
    }

    if relative_path.contains('\\') || relative_path.contains(':') {
        return Err(format!(
            "Plugin returned unsafe relative_path: {relative_path}"
        ));
    }

    let path = Path::new(relative_path);
    if path.is_absolute() {
        return Err(format!(
            "Plugin returned absolute relative_path: {relative_path}"
        ));
    }

    let mut has_file_component = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => has_file_component = true,
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(format!(
                    "Plugin returned unsafe relative_path: {relative_path}"
                ));
            }
        }
    }

    if !has_file_component {
        return Err("Plugin returned an empty relative_path".to_string());
    }

    Ok(())
}

pub fn join_validated_plugin_path(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    validate_plugin_relative_path(relative_path)?;
    Ok(root.join(relative_path))
}

#[cfg(test)]
mod tests {
    use super::{join_validated_plugin_path, validate_plugin_relative_path};
    use std::path::Path;

    #[test]
    fn plugin_relative_path_allows_normal_relative_paths() {
        assert!(validate_plugin_relative_path("loadorder.txt").is_ok());
        assert!(validate_plugin_relative_path("profiles/loadorder.txt").is_ok());
        assert!(
            validate_plugin_relative_path("local-app-data/Skyrim Special Edition/plugins.txt")
                .is_ok()
        );
        assert!(validate_plugin_relative_path("documents/The Witcher 3/mods.settings").is_ok());
    }

    #[test]
    fn plugin_relative_path_rejects_traversal_and_absolute_paths() {
        for path in [
            "",
            ".",
            "./loadorder.txt",
            "../outside.txt",
            "profiles/../../outside.txt",
            "/tmp/outside.txt",
        ] {
            assert!(
                validate_plugin_relative_path(path).is_err(),
                "path should be rejected: {path}"
            );
        }
    }

    #[test]
    fn plugin_relative_path_rejects_windows_prefixes_and_separators() {
        for path in [
            "C:/Users/player/outside.txt",
            "C:\\Users\\player\\outside.txt",
            "\\\\server\\share\\outside.txt",
            "profiles\\loadorder.txt",
        ] {
            assert!(
                validate_plugin_relative_path(path).is_err(),
                "path should be rejected: {path}"
            );
        }
    }

    #[test]
    fn validated_plugin_path_joins_without_canonicalizing() {
        let root = Path::new("/missing/game-root");
        let path = join_validated_plugin_path(root, "documents/The Witcher 3/mods.settings")
            .expect("valid path should join");

        assert_eq!(path, root.join("documents/The Witcher 3/mods.settings"));
    }
}
