use std::cell::RefCell;
use std::fs;
use std::time::{Duration, SystemTime};

use thiserror::Error;

use crate::domain::{ModuleCategory, RegistryIndex, RegistryModule};
use crate::services::paths;

const REGISTRY_URL: &str = "https://waybar-modules.github.io/registry/index.json";
const CACHE_TTL: Duration = Duration::from_secs(3600);

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("failed to fetch registry: {0}")]
    FetchError(String),
    #[error("failed to parse registry: {0}")]
    ParseError(String),
    #[error("cache read error: {0}")]
    CacheReadError(String),
    #[error("cache write error: {0}")]
    CacheWriteError(String),
    #[error("no cached registry available")]
    NoCachedRegistry,
}

#[derive(Debug, Clone)]
pub struct RegistryService {
    registry_url: String,
    index: RefCell<Option<RegistryIndex>>,
    last_fetch: RefCell<Option<SystemTime>>,
}

impl RegistryService {
    pub fn new() -> Self {
        Self {
            registry_url: REGISTRY_URL.to_string(),
            index: RefCell::new(None),
            last_fetch: RefCell::new(None),
        }
    }

    #[cfg(test)]
    pub fn with_url(url: &str) -> Self {
        Self {
            registry_url: url.to_string(),
            index: RefCell::new(None),
            last_fetch: RefCell::new(None),
        }
    }

    pub fn is_cache_valid(&self) -> bool {
        if let Some(last_fetch) = *self.last_fetch.borrow()
            && let Ok(elapsed) = last_fetch.elapsed()
        {
            return elapsed < CACHE_TTL;
        }
        false
    }

    pub fn get_cached(&self) -> Option<RegistryIndex> {
        self.index.borrow().clone()
    }

    pub fn load_from_cache(&self) -> Result<(), RegistryError> {
        let cache_path = paths::registry_cache_path();

        if !cache_path.exists() {
            return Err(RegistryError::NoCachedRegistry);
        }

        let content = fs::read_to_string(&cache_path)
            .map_err(|e| RegistryError::CacheReadError(e.to_string()))?;

        let index: RegistryIndex = serde_json::from_str(&content)
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        self.index.replace(Some(index));

        if let Ok(metadata) = fs::metadata(&cache_path)
            && let Ok(modified) = metadata.modified()
        {
            self.last_fetch.replace(Some(modified));
        }

        Ok(())
    }

    pub fn save_to_cache(&self, index: &RegistryIndex) -> Result<(), RegistryError> {
        let cache_path = paths::registry_cache_path();

        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| RegistryError::CacheWriteError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(index)
            .map_err(|e| RegistryError::CacheWriteError(e.to_string()))?;

        fs::write(&cache_path, content)
            .map_err(|e| RegistryError::CacheWriteError(e.to_string()))?;

        Ok(())
    }

    pub fn set_index(&self, index: RegistryIndex) {
        self.index.replace(Some(index));
        self.last_fetch.replace(Some(SystemTime::now()));
    }

    pub fn search(&self, query: &str) -> Vec<RegistryModule> {
        self.index
            .borrow()
            .as_ref()
            .map(|idx| idx.search(query).into_iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn by_category(&self, category: ModuleCategory) -> Vec<RegistryModule> {
        self.index
            .borrow()
            .as_ref()
            .map(|idx| idx.by_category(category).into_iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_module(&self, uuid: &str) -> Option<RegistryModule> {
        self.index.borrow().as_ref().and_then(|idx| {
            idx.modules
                .iter()
                .find(|m| m.uuid.to_string() == uuid)
                .cloned()
        })
    }

    pub fn registry_url(&self) -> &str {
        &self.registry_url
    }

    pub fn module_count(&self) -> usize {
        self.index
            .borrow()
            .as_ref()
            .map(|idx| idx.modules.len())
            .unwrap_or(0)
    }
}

impl Default for RegistryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ModuleUuid;

    fn create_test_registry_module(name: &str, category: ModuleCategory) -> RegistryModule {
        RegistryModule {
            uuid: ModuleUuid::try_from(format!("{}@test", name).as_str()).unwrap(),
            name: name.to_string(),
            description: format!("Test module {}", name),
            author: "test".to_string(),
            category,
            icon: None,
            screenshot: None,
            repo_url: "https://github.com/test/test".to_string(),
            downloads: 0,
            waybar_versions: vec!["0.10".to_string()],
            version: None,
            last_updated: None,
            rating: None,
            verified_author: false,
        }
    }

    fn create_test_index() -> RegistryIndex {
        RegistryIndex {
            version: 1,
            modules: vec![
                create_test_registry_module("weather", ModuleCategory::Weather),
                create_test_registry_module("cpu", ModuleCategory::Hardware),
                create_test_registry_module("network", ModuleCategory::Network),
            ],
            categories: Default::default(),
        }
    }

    #[test]
    fn test_new_creates_empty_service() {
        let service = RegistryService::new();
        assert!(service.get_cached().is_none());
        assert!(!service.is_cache_valid());
    }

    #[test]
    fn test_set_index_makes_cache_valid() {
        let service = RegistryService::new();
        service.set_index(create_test_index());
        assert!(service.is_cache_valid());
        assert!(service.get_cached().is_some());
    }

    #[test]
    fn test_search_empty_query_returns_all() {
        let service = RegistryService::new();
        service.set_index(create_test_index());
        let results = service.search("");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_filters_by_name() {
        let service = RegistryService::new();
        service.set_index(create_test_index());
        let results = service.search("weather");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "weather");
    }

    #[test]
    fn test_by_category_filters_correctly() {
        let service = RegistryService::new();
        service.set_index(create_test_index());
        let results = service.by_category(ModuleCategory::Hardware);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "cpu");
    }

    #[test]
    fn test_get_module_by_uuid() {
        let service = RegistryService::new();
        service.set_index(create_test_index());
        let module = service.get_module("weather@test");
        assert!(module.is_some());
        assert_eq!(module.unwrap().name, "weather");
    }

    #[test]
    fn test_get_module_not_found() {
        let service = RegistryService::new();
        service.set_index(create_test_index());
        let module = service.get_module("nonexistent@test");
        assert!(module.is_none());
    }

    #[test]
    fn test_module_count() {
        let service = RegistryService::new();
        assert_eq!(service.module_count(), 0);
        service.set_index(create_test_index());
        assert_eq!(service.module_count(), 3);
    }

    #[test]
    fn test_registry_url() {
        let service = RegistryService::new();
        assert_eq!(service.registry_url(), REGISTRY_URL);

        let custom = RegistryService::with_url("https://custom.url/index.json");
        assert_eq!(custom.registry_url(), "https://custom.url/index.json");
    }

    #[test]
    fn test_search_empty_index() {
        let service = RegistryService::new();
        let results = service.search("anything");
        assert!(results.is_empty());
    }

    #[test]
    fn test_by_category_empty_index() {
        let service = RegistryService::new();
        let results = service.by_category(ModuleCategory::System);
        assert!(results.is_empty());
    }
}
