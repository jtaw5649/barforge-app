use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::app::state::{SortField, SortOrder, ViewMode};
use crate::theme::ThemeMode;

fn settings_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("waybar-manager");
    config_dir.join("settings.json")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSettings {
    #[serde(default)]
    pub theme_mode: ThemeModeSettings,
    #[serde(default)]
    pub view_mode: ViewModeSettings,
    #[serde(default)]
    pub sort_field: SortFieldSettings,
    #[serde(default)]
    pub sort_order: SortOrderSettings,
    #[serde(default)]
    pub tray_enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeModeSettings {
    Light,
    Dark,
    #[default]
    System,
    Omarchy,
}

impl From<ThemeMode> for ThemeModeSettings {
    fn from(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => ThemeModeSettings::Light,
            ThemeMode::Dark => ThemeModeSettings::Dark,
            ThemeMode::System => ThemeModeSettings::System,
            ThemeMode::Omarchy => ThemeModeSettings::Omarchy,
        }
    }
}

impl From<ThemeModeSettings> for ThemeMode {
    fn from(settings: ThemeModeSettings) -> Self {
        match settings {
            ThemeModeSettings::Light => ThemeMode::Light,
            ThemeModeSettings::Dark => ThemeMode::Dark,
            ThemeModeSettings::System => ThemeMode::System,
            ThemeModeSettings::Omarchy => ThemeMode::Omarchy,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ViewModeSettings {
    #[default]
    Cards,
    Table,
}

impl From<ViewMode> for ViewModeSettings {
    fn from(mode: ViewMode) -> Self {
        match mode {
            ViewMode::Cards => ViewModeSettings::Cards,
            ViewMode::Table => ViewModeSettings::Table,
        }
    }
}

impl From<ViewModeSettings> for ViewMode {
    fn from(settings: ViewModeSettings) -> Self {
        match settings {
            ViewModeSettings::Cards => ViewMode::Cards,
            ViewModeSettings::Table => ViewMode::Table,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortFieldSettings {
    #[default]
    Name,
    Downloads,
    RecentlyUpdated,
    Rating,
}

impl From<SortField> for SortFieldSettings {
    fn from(field: SortField) -> Self {
        match field {
            SortField::Name => SortFieldSettings::Name,
            SortField::Downloads => SortFieldSettings::Downloads,
            SortField::RecentlyUpdated => SortFieldSettings::RecentlyUpdated,
            SortField::Rating => SortFieldSettings::Rating,
        }
    }
}

impl From<SortFieldSettings> for SortField {
    fn from(settings: SortFieldSettings) -> Self {
        match settings {
            SortFieldSettings::Name => SortField::Name,
            SortFieldSettings::Downloads => SortField::Downloads,
            SortFieldSettings::RecentlyUpdated => SortField::RecentlyUpdated,
            SortFieldSettings::Rating => SortField::Rating,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrderSettings {
    Ascending,
    #[default]
    Descending,
}

impl From<SortOrder> for SortOrderSettings {
    fn from(order: SortOrder) -> Self {
        match order {
            SortOrder::Ascending => SortOrderSettings::Ascending,
            SortOrder::Descending => SortOrderSettings::Descending,
        }
    }
}

impl From<SortOrderSettings> for SortOrder {
    fn from(settings: SortOrderSettings) -> Self {
        match settings {
            SortOrderSettings::Ascending => SortOrder::Ascending,
            SortOrderSettings::Descending => SortOrder::Descending,
        }
    }
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
        assert!(matches!(settings.theme_mode, ThemeModeSettings::System));
        assert!(matches!(settings.view_mode, ViewModeSettings::Cards));
    }

    #[test]
    fn test_theme_mode_conversion() {
        let mode = ThemeMode::Dark;
        let settings: ThemeModeSettings = mode.into();
        let back: ThemeMode = settings.into();
        assert_eq!(back, ThemeMode::Dark);
    }

    #[test]
    fn test_view_mode_conversion() {
        let mode = ViewMode::Table;
        let settings: ViewModeSettings = mode.into();
        let back: ViewMode = settings.into();
        assert_eq!(back, ViewMode::Table);
    }

    #[test]
    fn test_serialize_deserialize() {
        let settings = UserSettings {
            theme_mode: ThemeModeSettings::Dark,
            view_mode: ViewModeSettings::Table,
            sort_field: SortFieldSettings::Downloads,
            sort_order: SortOrderSettings::Descending,
            tray_enabled: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: UserSettings = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.theme_mode, ThemeModeSettings::Dark));
        assert!(matches!(deserialized.view_mode, ViewModeSettings::Table));
    }
}
