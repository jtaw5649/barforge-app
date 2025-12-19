use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{ModuleCategory, ModuleUuid, ModuleVersion};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RegistryIndex {
    pub version: u32,
    pub modules: Vec<RegistryModule>,
    pub categories: HashMap<String, CategoryInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CategoryInfo {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub icon: String,
}

impl RegistryIndex {
    pub fn search(&self, query: &str) -> Vec<&RegistryModule> {
        if query.is_empty() {
            return self.modules.iter().collect();
        }
        self.modules
            .iter()
            .filter(|m| m.matches_search(query))
            .collect()
    }

    pub fn by_category(&self, category: ModuleCategory) -> Vec<&RegistryModule> {
        self.modules
            .iter()
            .filter(|m| m.category == category)
            .collect()
    }

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
        fn deserialize_with_tags() {
            let json = r#"{
                "uuid": "test@dev",
                "name": "Test",
                "description": "A test module",
                "author": "dev",
                "category": "system",
                "icon": null,
                "screenshot": null,
                "repo_url": "https://github.com/test/test",
                "downloads": 100,
                "tags": ["keyword1", "keyword2"]
            }"#;
            let module: RegistryModule = serde_json::from_str(json).unwrap();
            assert_eq!(module.tags, vec!["keyword1", "keyword2"]);
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
            let mut module = create_test_registry_module("test");
            module.downloads = 500;
            assert_eq!(module.formatted_downloads(), "500");
        }

        #[test]
        fn formatted_downloads_thousands() {
            let mut module = create_test_registry_module("test");
            module.downloads = 1_500;
            assert_eq!(module.formatted_downloads(), "1.5k");
            module.downloads = 12_300;
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
            module.description =
                "This is a very long description that should be truncated".to_string();
            assert_eq!(module.truncated_description(20), "This is a very lo...");
        }
    }

    mod registry_index {
        use super::*;

        fn create_test_index() -> RegistryIndex {
            RegistryIndex {
                version: 1,
                modules: vec![
                    create_test_registry_module("weather-wttr"),
                    {
                        let mut m = create_test_registry_module("cpu-monitor");
                        m.category = ModuleCategory::Hardware;
                        m
                    },
                    {
                        let mut m = create_test_registry_module("network-speed");
                        m.category = ModuleCategory::Network;
                        m
                    },
                ],
                categories: HashMap::new(),
            }
        }

        #[test]
        fn search_empty_query_returns_all() {
            let index = create_test_index();
            let results = index.search("");
            assert_eq!(results.len(), 3);
        }

        #[test]
        fn search_filters_by_name() {
            let index = create_test_index();
            let results = index.search("weather");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "weather-wttr");
        }

        #[test]
        fn by_category_filters_correctly() {
            let index = create_test_index();
            let results = index.by_category(ModuleCategory::Hardware);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "cpu-monitor");
        }

        #[test]
        fn find_by_uuid_existing() {
            let index = create_test_index();
            let result = index.find_by_uuid("weather-wttr@test");
            assert!(result.is_some());
            assert_eq!(result.unwrap().name, "weather-wttr");
        }

        #[test]
        fn find_by_uuid_not_found() {
            let index = create_test_index();
            let result = index.find_by_uuid("nonexistent@test");
            assert!(result.is_none());
        }

        #[test]
        fn by_category_empty_when_no_match() {
            let index = create_test_index();
            let results = index.by_category(ModuleCategory::Weather);
            assert!(results.is_empty());
        }

        #[test]
        fn deserialize_full_api_response() {
            let json = r#"{
                "version": 1,
                "modules": [{
                    "uuid": "weather-wttr@waybar-modules",
                    "name": "Weather WTTR",
                    "description": "Weather display using wttr.in",
                    "author": "johndoe",
                    "category": "weather",
                    "icon": "weather-symbolic",
                    "screenshot": null,
                    "repo_url": "https://github.com/example/weather",
                    "downloads": 1500,
                    "version": "1.0.0",
                    "last_updated": "2024-12-01T12:00:00Z",
                    "rating": 4.5,
                    "verified_author": true,
                    "tags": ["weather", "forecast"],
                    "checksum": "abc123"
                }],
                "categories": {
                    "weather": {
                        "id": "weather",
                        "name": "Weather",
                        "icon": "weather-symbolic"
                    }
                }
            }"#;
            let index: RegistryIndex = serde_json::from_str(json).unwrap();
            assert_eq!(index.version, 1);
            assert_eq!(index.modules.len(), 1);
            assert_eq!(index.modules[0].name, "Weather WTTR");
        }
    }
}
