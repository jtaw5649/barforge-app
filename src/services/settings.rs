use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::app::state::{SortField, SortOrder, ViewMode};
use crate::theme::ThemeMode;

fn settings_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("barforge");
    config_dir.join("settings.json")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSettings {
    #[serde(default)]
    pub theme_mode: ThemeMode,
    #[serde(default)]
    pub view_mode: ViewMode,
    #[serde(default)]
    pub sort_field: SortField,
    #[serde(default)]
    pub sort_order: SortOrder,
    #[serde(default)]
    pub tray_enabled: bool,
}

pub fn load_settings() -> UserSettings {
    let path = settings_path();

    if !path.exists() {
        return UserSettings::default();
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &UserSettings) -> io::Result<()> {
    let path = settings_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(settings)?;
    fs::write(&path, json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = UserSettings::default();
        assert!(matches!(settings.theme_mode, ThemeMode::System));
        assert!(matches!(settings.view_mode, ViewMode::Cards));
    }

    #[test]
    fn test_serialize_deserialize() {
        let settings = UserSettings {
            theme_mode: ThemeMode::Dark,
            view_mode: ViewMode::Table,
            sort_field: SortField::Downloads,
            sort_order: SortOrder::Descending,
            tray_enabled: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: UserSettings = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.theme_mode, ThemeMode::Dark));
        assert!(matches!(deserialized.view_mode, ViewMode::Table));
        assert!(matches!(deserialized.sort_field, SortField::Downloads));
    }
}
