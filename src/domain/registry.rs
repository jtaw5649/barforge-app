use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{ModuleCategory, ModuleUuid, ModuleVersion};

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
    pub waybar_versions: Vec<String>,
}

impl RegistryModule {
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower)
            || self.description.to_lowercase().contains(&query_lower)
            || self.author.to_lowercase().contains(&query_lower)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModule {
    pub uuid: ModuleUuid,
    pub version: ModuleVersion,
    pub install_path: PathBuf,
    pub enabled: bool,
    pub waybar_module_name: String,
    pub has_preferences: bool,
}

impl InstalledModule {
    pub fn is_custom_module(&self) -> bool {
        self.waybar_module_name.starts_with("custom/")
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
    pub id: String,
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
            waybar_versions: vec!["0.10".to_string(), "0.11".to_string()],
        }
    }

    mod registry_module {
        use super::*;

        #[test]
        fn test_matches_search_by_name() {
            let module = create_test_registry_module("weather-wttr");
            assert!(module.matches_search("weather"));
            assert!(module.matches_search("WEATHER"));
        }

        #[test]
        fn test_matches_search_by_description() {
            let module = create_test_registry_module("test");
            assert!(module.matches_search("test module"));
        }

        #[test]
        fn test_matches_search_by_author() {
            let module = create_test_registry_module("test");
            assert!(module.matches_search("test-author"));
        }

        #[test]
        fn test_matches_search_no_match() {
            let module = create_test_registry_module("test");
            assert!(!module.matches_search("nonexistent"));
        }
    }

    mod installed_module {
        use super::*;

        #[test]
        fn test_is_custom_module_true() {
            let module = InstalledModule {
                uuid: create_test_uuid("weather"),
                version: create_test_version(),
                install_path: PathBuf::from("/test"),
                enabled: true,
                waybar_module_name: "custom/weather".to_string(),
                has_preferences: false,
            };
            assert!(module.is_custom_module());
        }

        #[test]
        fn test_is_custom_module_false() {
            let module = InstalledModule {
                uuid: create_test_uuid("clock"),
                version: create_test_version(),
                install_path: PathBuf::from("/test"),
                enabled: true,
                waybar_module_name: "clock".to_string(),
                has_preferences: false,
            };
            assert!(!module.is_custom_module());
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
        fn test_search_empty_query_returns_all() {
            let index = create_test_index();
            let results = index.search("");
            assert_eq!(results.len(), 3);
        }

        #[test]
        fn test_search_filters_by_name() {
            let index = create_test_index();
            let results = index.search("weather");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "weather-wttr");
        }

        #[test]
        fn test_by_category() {
            let index = create_test_index();
            let results = index.by_category(ModuleCategory::Hardware);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "cpu-monitor");
        }

        #[test]
        fn test_by_category_empty() {
            let index = create_test_index();
            let results = index.by_category(ModuleCategory::Weather);
            assert!(results.is_empty());
        }
    }
}
