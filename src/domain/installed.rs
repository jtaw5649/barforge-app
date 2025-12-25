use std::path::PathBuf;

use barforge_registry_types::{ModuleUuid, ModuleVersion};
use serde::{Deserialize, Serialize};

use super::ModulePosition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModule {
    pub uuid: ModuleUuid,
    pub version: ModuleVersion,
    pub install_path: PathBuf,
    pub enabled: bool,
    pub waybar_module_name: String,
    pub has_preferences: bool,
    #[serde(default = "default_installed_at")]
    pub installed_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub registry_version: Option<ModuleVersion>,
    #[serde(default)]
    pub position: Option<ModulePosition>,
}

fn default_installed_at() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

impl InstalledModule {
    pub fn is_custom_module(&self) -> bool {
        self.waybar_module_name.starts_with("custom/")
    }

    pub fn has_update(&self) -> bool {
        self.registry_version
            .as_ref()
            .is_some_and(|registry_ver| registry_ver > &self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_uuid(name: &str) -> ModuleUuid {
        ModuleUuid::try_from(format!("{}@test", name).as_str()).unwrap()
    }

    fn create_test_version() -> ModuleVersion {
        ModuleVersion::try_from("1.0.0").unwrap()
    }

    #[test]
    fn is_custom_module_true_for_custom_prefix() {
        let module = InstalledModule {
            uuid: create_test_uuid("weather"),
            version: create_test_version(),
            install_path: PathBuf::from("/test"),
            enabled: true,
            waybar_module_name: "custom/weather".to_string(),
            has_preferences: false,
            installed_at: chrono::Utc::now(),
            registry_version: None,
            position: None,
        };
        assert!(module.is_custom_module());
    }

    #[test]
    fn is_custom_module_false_for_builtin() {
        let module = InstalledModule {
            uuid: create_test_uuid("clock"),
            version: create_test_version(),
            install_path: PathBuf::from("/test"),
            enabled: true,
            waybar_module_name: "clock".to_string(),
            has_preferences: false,
            installed_at: chrono::Utc::now(),
            registry_version: None,
            position: None,
        };
        assert!(!module.is_custom_module());
    }

    #[test]
    fn has_update_true_when_newer_version() {
        let module = InstalledModule {
            uuid: create_test_uuid("test"),
            version: ModuleVersion::try_from("1.0.0").unwrap(),
            install_path: PathBuf::from("/test"),
            enabled: true,
            waybar_module_name: "custom/test".to_string(),
            has_preferences: false,
            installed_at: chrono::Utc::now(),
            registry_version: Some(ModuleVersion::try_from("2.0.0").unwrap()),
            position: None,
        };
        assert!(module.has_update());
    }

    #[test]
    fn has_update_false_when_same_version() {
        let module = InstalledModule {
            uuid: create_test_uuid("test"),
            version: ModuleVersion::try_from("1.0.0").unwrap(),
            install_path: PathBuf::from("/test"),
            enabled: true,
            waybar_module_name: "custom/test".to_string(),
            has_preferences: false,
            installed_at: chrono::Utc::now(),
            registry_version: Some(ModuleVersion::try_from("1.0.0").unwrap()),
            position: None,
        };
        assert!(!module.has_update());
    }

    #[test]
    fn has_update_false_when_no_registry_version() {
        let module = InstalledModule {
            uuid: create_test_uuid("test"),
            version: ModuleVersion::try_from("1.0.0").unwrap(),
            install_path: PathBuf::from("/test"),
            enabled: true,
            waybar_module_name: "custom/test".to_string(),
            has_preferences: false,
            installed_at: chrono::Utc::now(),
            registry_version: None,
            position: None,
        };
        assert!(!module.has_update());
    }
}
