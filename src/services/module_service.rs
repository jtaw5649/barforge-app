use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use thiserror::Error;

use crate::domain::{InstalledModule, ModuleVersion, RegistryModule};
use crate::services::paths;

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("module not found: {0}")]
    NotFound(String),
    #[error("module already installed: {0}")]
    AlreadyInstalled(String),
    #[error("failed to install module: {0}")]
    InstallError(String),
    #[error("failed to uninstall module: {0}")]
    UninstallError(String),
    #[error("failed to load installed modules: {0}")]
    LoadError(String),
    #[error("failed to save module state: {0}")]
    SaveError(String),
}

#[derive(Debug, Clone)]
pub struct ModuleService {
    installed: RefCell<HashMap<String, InstalledModule>>,
}

impl ModuleService {
    pub fn new() -> Self {
        Self {
            installed: RefCell::new(HashMap::new()),
        }
    }

    pub fn load_installed(&self) -> Result<(), ModuleError> {
        let state_path = Self::state_file_path();

        if !state_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&state_path)
            .map_err(|e| ModuleError::LoadError(e.to_string()))?;

        let modules: Vec<InstalledModule> = serde_json::from_str(&content)
            .map_err(|e| ModuleError::LoadError(e.to_string()))?;

        let mut installed = self.installed.borrow_mut();
        for module in modules {
            installed.insert(module.uuid.to_string(), module);
        }

        Ok(())
    }

    pub fn save_state(&self) -> Result<(), ModuleError> {
        let state_path = Self::state_file_path();

        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ModuleError::SaveError(e.to_string()))?;
        }

        let binding = self.installed.borrow();
        let modules: Vec<&InstalledModule> = binding.values().collect();
        let content = serde_json::to_string_pretty(&modules)
            .map_err(|e| ModuleError::SaveError(e.to_string()))?;

        fs::write(&state_path, content)
            .map_err(|e| ModuleError::SaveError(e.to_string()))?;

        Ok(())
    }

    fn state_file_path() -> PathBuf {
        paths::data_dir().join("installed.json")
    }

    pub fn is_installed(&self, uuid: &str) -> bool {
        self.installed.borrow().contains_key(uuid)
    }

    pub fn get_installed(&self, uuid: &str) -> Option<InstalledModule> {
        self.installed.borrow().get(uuid).cloned()
    }

    pub fn list_installed(&self) -> Vec<InstalledModule> {
        self.installed.borrow().values().cloned().collect()
    }

    pub fn list_enabled(&self) -> Vec<InstalledModule> {
        self.installed
            .borrow()
            .values()
            .filter(|m| m.enabled)
            .cloned()
            .collect()
    }

    pub fn install(&self, registry_module: &RegistryModule, version: &str) -> Result<InstalledModule, ModuleError> {
        let uuid_str = registry_module.uuid.to_string();

        if self.is_installed(&uuid_str) {
            return Err(ModuleError::AlreadyInstalled(uuid_str));
        }

        let install_path = paths::module_install_path(&uuid_str);
        fs::create_dir_all(&install_path)
            .map_err(|e| ModuleError::InstallError(e.to_string()))?;

        let version = ModuleVersion::try_from(version)
            .map_err(|e| ModuleError::InstallError(e.to_string()))?;

        let waybar_module_name = format!("custom/{}", registry_module.name.replace(' ', "-").to_lowercase());

        let installed = InstalledModule {
            uuid: registry_module.uuid.clone(),
            version,
            install_path,
            enabled: false,
            waybar_module_name,
            has_preferences: false,
            installed_at: chrono::Utc::now(),
            registry_version: registry_module.version.clone(),
            position: None,
        };

        self.installed.borrow_mut().insert(uuid_str, installed.clone());
        self.save_state()?;

        Ok(installed)
    }

    pub fn uninstall(&self, uuid: &str) -> Result<(), ModuleError> {
        if !self.is_installed(uuid) {
            return Err(ModuleError::NotFound(uuid.to_string()));
        }

        let install_path = paths::module_install_path(uuid);
        if install_path.exists() {
            fs::remove_dir_all(&install_path)
                .map_err(|e| ModuleError::UninstallError(e.to_string()))?;
        }

        let prefs_path = paths::module_preferences_path(uuid);
        if prefs_path.exists() {
            let _ = fs::remove_file(&prefs_path);
        }

        self.installed.borrow_mut().remove(uuid);
        self.save_state()?;

        Ok(())
    }

    pub fn enable(&self, uuid: &str) -> Result<(), ModuleError> {
        let mut installed = self.installed.borrow_mut();
        let module = installed
            .get_mut(uuid)
            .ok_or_else(|| ModuleError::NotFound(uuid.to_string()))?;

        module.enabled = true;
        drop(installed);
        self.save_state()?;

        Ok(())
    }

    pub fn disable(&self, uuid: &str) -> Result<(), ModuleError> {
        let mut installed = self.installed.borrow_mut();
        let module = installed
            .get_mut(uuid)
            .ok_or_else(|| ModuleError::NotFound(uuid.to_string()))?;

        module.enabled = false;
        drop(installed);
        self.save_state()?;

        Ok(())
    }

    pub fn installed_count(&self) -> usize {
        self.installed.borrow().len()
    }

    pub fn enabled_count(&self) -> usize {
        self.installed.borrow().values().filter(|m| m.enabled).count()
    }
}

impl Default for ModuleService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ModuleCategory, ModuleUuid};

    fn create_test_registry_module(name: &str) -> RegistryModule {
        RegistryModule {
            uuid: ModuleUuid::try_from(format!("{}@test", name).as_str()).unwrap(),
            name: name.to_string(),
            description: format!("Test module {}", name),
            author: "test".to_string(),
            category: ModuleCategory::System,
            icon: None,
            screenshot: None,
            repo_url: "https://github.com/test/test".to_string(),
            downloads: 0,
            version: None,
            last_updated: None,
            rating: None,
            verified_author: false,
        }
    }

    #[test]
    fn test_new_creates_empty_service() {
        let service = ModuleService::new();
        assert_eq!(service.installed_count(), 0);
        assert!(!service.is_installed("anything@test"));
    }

    #[test]
    fn test_is_installed_false_when_not_installed() {
        let service = ModuleService::new();
        assert!(!service.is_installed("weather@test"));
    }

    #[test]
    fn test_list_installed_empty() {
        let service = ModuleService::new();
        assert!(service.list_installed().is_empty());
    }

    #[test]
    fn test_list_enabled_empty() {
        let service = ModuleService::new();
        assert!(service.list_enabled().is_empty());
    }

    #[test]
    fn test_enabled_count_zero() {
        let service = ModuleService::new();
        assert_eq!(service.enabled_count(), 0);
    }

    #[test]
    fn test_get_installed_returns_none() {
        let service = ModuleService::new();
        assert!(service.get_installed("nonexistent@test").is_none());
    }

    #[test]
    fn test_uninstall_not_found() {
        let service = ModuleService::new();
        let result = service.uninstall("nonexistent@test");
        assert!(matches!(result, Err(ModuleError::NotFound(_))));
    }

    #[test]
    fn test_enable_not_found() {
        let service = ModuleService::new();
        let result = service.enable("nonexistent@test");
        assert!(matches!(result, Err(ModuleError::NotFound(_))));
    }

    #[test]
    fn test_disable_not_found() {
        let service = ModuleService::new();
        let result = service.disable("nonexistent@test");
        assert!(matches!(result, Err(ModuleError::NotFound(_))));
    }

    #[test]
    fn test_waybar_module_name_format() {
        let module = create_test_registry_module("Weather Module");
        let name = format!("custom/{}", module.name.replace(' ', "-").to_lowercase());
        assert_eq!(name, "custom/weather-module");
    }
}
