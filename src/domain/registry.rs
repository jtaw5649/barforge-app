use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{ModuleCategory, ModuleUuid, ModuleVersion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryModule {
    pub uuid: ModuleUuid,
    pub name: String,
    pub description: String,
    pub author: String,
    pub category: ModuleCategory,
    pub icon: Option<String>,
    pub screenshot: Option<String>,
    pub repo_url: String,
    pub downloads: u64,
    #[serde(default)]
    pub version: Option<ModuleVersion>,
    #[serde(default)]
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub rating: Option<f32>,
    #[serde(default)]
    pub verified_author: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
}

impl RegistryModule {
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower)
            || self.description.to_lowercase().contains(&query_lower)
            || self.author.to_lowercase().contains(&query_lower)
            || self
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower))
    }

    pub fn formatted_downloads(&self) -> String {
        match self.downloads {
            0..=999 => self.downloads.to_string(),
            1_000..=999_999 => format!("{:.1}k", self.downloads as f64 / 1_000.0),
            _ => format!("{:.1}M", self.downloads as f64 / 1_000_000.0),
        }
    }

    pub fn truncated_description(&self, max_len: usize) -> String {
        if self.description.len() <= max_len {
            self.description.clone()
        } else {
            format!("{}...", &self.description[..max_len.saturating_sub(3)])
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistryIndex {
    pub version: u32,
    pub modules: Vec<RegistryModule>,
    pub categories: HashMap<String, CategoryInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub icon: String,
}

impl RegistryIndex {
    pub fn find_by_uuid(&self, uuid: &str) -> Option<&RegistryModule> {
        self.modules.iter().find(|m| m.uuid.to_string() == uuid)
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

    fn create_test_registry_module(name: &str) -> RegistryModule {
        RegistryModule {
            uuid: create_test_uuid(name),
            name: name.to_string(),
            description: format!("A test module called {}", name),
            author: "test-author".to_string(),
            category: ModuleCategory::System,
            icon: Some("test-icon-symbolic".to_string()),
            screenshot: None,
            repo_url: "https://github.com/test/test".to_string(),
            downloads: 100,
            version: Some(create_test_version()),
            last_updated: None,
            rating: None,
            verified_author: false,
            tags: Vec::new(),
            checksum: None,
            license: None,
        }
    }

    mod registry_module {
        use super::*;

        #[test]
        fn matches_search_by_name() {
            let module = create_test_registry_module("weather-wttr");
            assert!(module.matches_search("weather"));
            assert!(module.matches_search("WEATHER"));
        }

        #[test]
        fn matches_search_by_description() {
            let module = create_test_registry_module("test");
            assert!(module.matches_search("test module"));
        }

        #[test]
        fn matches_search_by_author() {
            let module = create_test_registry_module("test");
            assert!(module.matches_search("test-author"));
        }

        #[test]
        fn matches_search_no_match() {
            let module = create_test_registry_module("test");
            assert!(!module.matches_search("nonexistent"));
        }

        #[test]
        fn matches_search_by_tag() {
            let mut module = create_test_registry_module("test");
            module.tags = vec!["weather".to_string(), "forecast".to_string()];
            assert!(module.matches_search("forecast"));
            assert!(module.matches_search("WEATHER"));
        }

        #[test]
        fn deserialize_without_tags_defaults_empty() {
            let json = r#"{
                "uuid": "test@dev",
                "name": "Test",
                "description": "A test module",
                "author": "dev",
                "category": "system",
                "icon": null,
                "screenshot": null,
                "repo_url": "https://github.com/test/test",
                "downloads": 100
            }"#;
            let module: RegistryModule = serde_json::from_str(json).unwrap();
            assert!(module.tags.is_empty());
        }

        #[test]
        fn formatted_downloads_under_thousand() {
            let module = create_test_registry_module("test");
            assert_eq!(module.formatted_downloads(), "100");
        }

        #[test]
        fn formatted_downloads_thousands() {
            let mut module = create_test_registry_module("test");
            module.downloads = 12_345;
            assert_eq!(module.formatted_downloads(), "12.3k");
        }

        #[test]
        fn formatted_downloads_millions() {
            let mut module = create_test_registry_module("test");
            module.downloads = 1_500_000;
            assert_eq!(module.formatted_downloads(), "1.5M");
        }

        #[test]
        fn truncated_description_short() {
            let mut module = create_test_registry_module("test");
            module.description = "Short desc".to_string();
            assert_eq!(module.truncated_description(100), "Short desc");
        }

        #[test]
        fn truncated_description_long() {
            let mut module = create_test_registry_module("test");
            module.description = "This is a very long description".to_string();
            assert_eq!(module.truncated_description(20), "This is a very lo...");
        }
    }

    mod registry_index {
        use super::*;

        #[test]
        fn find_by_uuid_returns_match() {
            let module = create_test_registry_module("module-one");
            let uuid = module.uuid.to_string();
            let index = RegistryIndex {
                version: 1,
                modules: vec![module],
                categories: HashMap::new(),
            };
            assert!(index.find_by_uuid(&uuid).is_some());
        }

        #[test]
        fn find_by_uuid_returns_none() {
            let index = RegistryIndex {
                version: 1,
                modules: Vec::new(),
                categories: HashMap::new(),
            };
            assert!(index.find_by_uuid("missing@uuid").is_none());
        }
    }
}
