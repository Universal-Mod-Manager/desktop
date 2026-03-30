use crate::models::ThemeInfo;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ThemeManager {
    themes_dir: PathBuf,
    active_theme: Option<String>,
}

impl ThemeManager {
    pub fn new(data_dir: &Path, active_theme: &str) -> Self {
        Self {
            themes_dir: data_dir.join("themes"),
            active_theme: if active_theme.is_empty() {
                None
            } else {
                Some(active_theme.to_string())
            },
        }
    }

    pub fn list_themes(&self) -> Result<Vec<ThemeInfo>> {
        let mut themes = Vec::new();
        fs::create_dir_all(&self.themes_dir)?;

        for entry in fs::read_dir(&self.themes_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "css") {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                themes.push(ThemeInfo {
                    is_active: self.active_theme.as_deref() == Some(name.as_str()),
                    name,
                });
            }
        }

        Ok(themes)
    }

    pub fn get_theme_css(&self, name: &str) -> Result<String> {
        if name.is_empty() {
            return Ok(String::new());
        }
        let path = self.themes_dir.join(format!("{}.css", name));
        if !path.exists() {
            anyhow::bail!("Theme '{}' not found", name);
        }
        Ok(fs::read_to_string(path)?)
    }

    pub fn set_active_theme(&mut self, name: &str) -> Result<()> {
        if name.is_empty() {
            self.active_theme = None;
            return Ok(());
        }
        let path = self.themes_dir.join(format!("{}.css", name));
        if !path.exists() {
            anyhow::bail!("Theme '{}' not found", name);
        }
        self.active_theme = Some(name.to_string());
        Ok(())
    }

    pub fn get_active_theme(&self) -> &str {
        self.active_theme.as_deref().unwrap_or("")
    }
}
