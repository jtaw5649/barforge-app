use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
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
    fn display_name_returns_human_readable() {
        assert_eq!(ModuleCategory::System.display_name(), "System");
        assert_eq!(ModuleCategory::Time.display_name(), "Time & Date");
        assert_eq!(ModuleCategory::Tray.display_name(), "System Tray");
    }

    #[test]
    fn all_returns_all_categories() {
        let all = ModuleCategory::all();
        assert_eq!(all.len(), 13);
        assert!(all.contains(&ModuleCategory::System));
        assert!(all.contains(&ModuleCategory::Custom));
    }

    #[test]
    fn display_trait_works() {
        assert_eq!(format!("{}", ModuleCategory::Weather), "Weather");
    }

    #[test]
    fn try_from_valid_strings() {
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
    fn try_from_invalid_string_fails() {
        assert!(ModuleCategory::try_from("invalid").is_err());
    }

    #[test]
    fn copy_works() {
        let category = ModuleCategory::Network;
        let copied = category;
        assert_eq!(category, copied);
    }

    #[test]
    fn hashes_consistently() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ModuleCategory::Audio);
        set.insert(ModuleCategory::Audio);
        assert_eq!(set.len(), 1);
    }
}
