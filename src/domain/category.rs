use std::fmt;

use iced::Color;
use serde::{Deserialize, Serialize};

use crate::theme::palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleCategory {
    System,
    Hardware,
    Network,
    Audio,
    Power,
    Time,
    Workspace,
    Window,
    Tray,
    Weather,
    Productivity,
    Media,
    Custom,
}

impl ModuleCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Hardware => "Hardware",
            Self::Network => "Network",
            Self::Audio => "Audio",
            Self::Power => "Power",
            Self::Time => "Time & Date",
            Self::Workspace => "Workspace",
            Self::Window => "Window",
            Self::Tray => "System Tray",
            Self::Weather => "Weather",
            Self::Productivity => "Productivity",
            Self::Media => "Media",
            Self::Custom => "Custom",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::System => "utilities-system-monitor-symbolic",
            Self::Hardware => "computer-symbolic",
            Self::Network => "network-workgroup-symbolic",
            Self::Audio => "audio-speakers-symbolic",
            Self::Power => "battery-symbolic",
            Self::Time => "appointment-symbolic",
            Self::Workspace => "view-grid-symbolic",
            Self::Window => "window-symbolic",
            Self::Tray => "application-x-addon-symbolic",
            Self::Weather => "weather-clear-symbolic",
            Self::Productivity => "x-office-calendar-symbolic",
            Self::Media => "applications-multimedia-symbolic",
            Self::Custom => "applications-other-symbolic",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::System,
            Self::Hardware,
            Self::Network,
            Self::Audio,
            Self::Power,
            Self::Time,
            Self::Workspace,
            Self::Window,
            Self::Tray,
            Self::Weather,
            Self::Productivity,
            Self::Media,
            Self::Custom,
        ]
    }

    pub fn badge_color(&self) -> Color {
        match self {
            Self::System => palette::BADGE_SYSTEM,
            Self::Hardware => palette::BADGE_HARDWARE,
            Self::Network => palette::BADGE_NETWORK,
            Self::Audio => palette::BADGE_AUDIO,
            Self::Power => palette::BADGE_POWER,
            Self::Time => palette::BADGE_TIME,
            Self::Workspace => palette::BADGE_WORKSPACE,
            Self::Window => palette::BADGE_WINDOW,
            Self::Tray => palette::BADGE_TRAY,
            Self::Weather => palette::BADGE_WEATHER,
            Self::Productivity => palette::BADGE_PRODUCTIVITY,
            Self::Media => palette::BADGE_MEDIA,
            Self::Custom => palette::BADGE_CUSTOM,
        }
    }

    pub fn badge_text_color(&self) -> Color {
        match self {
            Self::System => palette::BADGE_TEXT_SYSTEM,
            Self::Hardware => palette::BADGE_TEXT_HARDWARE,
            Self::Network => palette::BADGE_TEXT_NETWORK,
            Self::Audio => palette::BADGE_TEXT_AUDIO,
            Self::Power => palette::BADGE_TEXT_POWER,
            Self::Time => palette::BADGE_TEXT_TIME,
            Self::Workspace => palette::BADGE_TEXT_WORKSPACE,
            Self::Window => palette::BADGE_TEXT_WINDOW,
            Self::Tray => palette::BADGE_TEXT_TRAY,
            Self::Weather => palette::BADGE_TEXT_WEATHER,
            Self::Productivity => palette::BADGE_TEXT_PRODUCTIVITY,
            Self::Media => palette::BADGE_TEXT_MEDIA,
            Self::Custom => palette::BADGE_TEXT_CUSTOM,
        }
    }
}

impl fmt::Display for ModuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl TryFrom<&str> for ModuleCategory {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "system" => Ok(Self::System),
            "hardware" => Ok(Self::Hardware),
            "network" => Ok(Self::Network),
            "audio" => Ok(Self::Audio),
            "power" => Ok(Self::Power),
            "time" => Ok(Self::Time),
            "workspace" => Ok(Self::Workspace),
            "window" => Ok(Self::Window),
            "tray" => Ok(Self::Tray),
            "weather" => Ok(Self::Weather),
            "productivity" => Ok(Self::Productivity),
            "media" => Ok(Self::Media),
            "custom" => Ok(Self::Custom),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_returns_human_readable() {
        assert_eq!(ModuleCategory::System.display_name(), "System");
        assert_eq!(ModuleCategory::Time.display_name(), "Time & Date");
        assert_eq!(ModuleCategory::Tray.display_name(), "System Tray");
    }

    #[test]
    fn test_icon_returns_symbolic_icon() {
        for category in ModuleCategory::all() {
            let icon = category.icon();
            assert!(
                icon.ends_with("-symbolic"),
                "Icon for {:?} should be symbolic: {}",
                category,
                icon
            );
        }
    }

    #[test]
    fn test_all_returns_all_categories() {
        let all = ModuleCategory::all();
        assert_eq!(all.len(), 13);
        assert!(all.contains(&ModuleCategory::System));
        assert!(all.contains(&ModuleCategory::Custom));
    }

    #[test]
    fn test_display_trait() {
        assert_eq!(format!("{}", ModuleCategory::Weather), "Weather");
    }

    #[test]
    fn test_try_from_valid_strings() {
        assert_eq!(
            ModuleCategory::try_from("system").unwrap(),
            ModuleCategory::System
        );
        assert_eq!(
            ModuleCategory::try_from("WEATHER").unwrap(),
            ModuleCategory::Weather
        );
        assert_eq!(
            ModuleCategory::try_from("Audio").unwrap(),
            ModuleCategory::Audio
        );
    }

    #[test]
    fn test_try_from_invalid_string() {
        assert!(ModuleCategory::try_from("invalid").is_err());
    }

    #[test]
    fn test_clone() {
        let category = ModuleCategory::Network;
        let cloned = category;
        assert_eq!(category, cloned);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ModuleCategory::Audio);
        set.insert(ModuleCategory::Audio);
        assert_eq!(set.len(), 1);
    }
}
