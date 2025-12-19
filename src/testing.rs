use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::time::Instant;

use waybar_registry_types::{
    ModuleCategory, ModuleUuid, ModuleVersion, RegistryIndex, RegistryModule,
};

use crate::app::state::{
    App, AuthorProfileState, BrowseState, CategoryFilter, ConfirmationState, InstalledState,
    LoadingState, ModuleDetailState, PreferencesState, Screen, SortField, SortOrder, ViewMode,
};
use crate::domain::InstalledModule;
use crate::theme::{AppTheme, ThemeMode};

pub fn test_uuid(name: &str) -> ModuleUuid {
    ModuleUuid::try_from(format!("{name}@test").as_str()).expect("valid test uuid format")
}

pub fn test_version(version: &str) -> ModuleVersion {
    ModuleVersion::try_from(version).expect("valid semver version")
}

pub struct InstalledModuleBuilder {
    uuid: ModuleUuid,
    version: ModuleVersion,
    install_path: PathBuf,
    enabled: bool,
    waybar_module_name: String,
    has_preferences: bool,
    registry_version: Option<ModuleVersion>,
}

impl InstalledModuleBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            uuid: test_uuid(name),
            version: test_version("1.0.0"),
            install_path: PathBuf::from(format!("/tmp/test-modules/{name}")),
            enabled: true,
            waybar_module_name: format!("custom/{name}"),
            has_preferences: false,
            registry_version: None,
        }
    }

    pub fn version(mut self, v: &str) -> Self {
        self.version = test_version(v);
        self
    }

    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }

    pub fn has_preferences(mut self, h: bool) -> Self {
        self.has_preferences = h;
        self
    }

    pub fn registry_version(mut self, v: &str) -> Self {
        self.registry_version = Some(test_version(v));
        self
    }

    pub fn build(self) -> InstalledModule {
        InstalledModule {
            uuid: self.uuid,
            version: self.version,
            install_path: self.install_path,
            enabled: self.enabled,
            waybar_module_name: self.waybar_module_name,
            has_preferences: self.has_preferences,
            installed_at: chrono::Utc::now(),
            registry_version: self.registry_version,
            position: None,
        }
    }
}

pub struct RegistryModuleBuilder {
    uuid: ModuleUuid,
    name: String,
    description: String,
    author: String,
    category: ModuleCategory,
    repo_url: String,
    downloads: u64,
    version: Option<ModuleVersion>,
    tags: Vec<String>,
}

impl RegistryModuleBuilder {
    pub fn new(name: &str) -> Self {
        let author = "testauthor";
        Self {
            uuid: ModuleUuid::try_from(format!("{name}@{author}").as_str())
                .expect("valid uuid format"),
            name: name.to_string(),
            description: format!("Test module: {name}"),
            author: author.to_string(),
            category: ModuleCategory::System,
            repo_url: format!("https://github.com/{author}/{name}"),
            downloads: 100,
            version: Some(test_version("1.0.0")),
            tags: vec![],
        }
    }

    pub fn author(mut self, a: &str) -> Self {
        self.author = a.to_string();
        self.uuid =
            ModuleUuid::try_from(format!("{}@{a}", self.name).as_str()).expect("valid uuid format");
        self.repo_url = format!("https://github.com/{a}/{}", self.name);
        self
    }

    pub fn category(mut self, c: ModuleCategory) -> Self {
        self.category = c;
        self
    }

    pub fn downloads(mut self, d: u64) -> Self {
        self.downloads = d;
        self
    }

    pub fn version(mut self, v: &str) -> Self {
        self.version = Some(test_version(v));
        self
    }

    pub fn tags(mut self, t: Vec<&str>) -> Self {
        self.tags = t.into_iter().map(String::from).collect();
        self
    }

    pub fn build(self) -> RegistryModule {
        RegistryModule {
            uuid: self.uuid,
            name: self.name,
            description: self.description,
            author: self.author,
            category: self.category,
            icon: None,
            screenshot: None,
            repo_url: self.repo_url,
            downloads: self.downloads,
            version: self.version,
            last_updated: None,
            rating: None,
            verified_author: false,
            tags: self.tags,
            checksum: None,
        }
    }
}

pub fn test_registry(modules: Vec<RegistryModule>) -> RegistryIndex {
    RegistryIndex {
        version: 1,
        modules,
        categories: Default::default(),
    }
}

pub fn test_app() -> App {
    App {
        screen: Screen::default(),
        registry: None,
        installed_modules: Vec::new(),
        installed_uuids: HashSet::new(),
        browse: BrowseState {
            search_query: String::new(),
            selected_category: CategoryFilter::default(),
            pending_search: None,
            search_debounce_start: None,
            sort_field: SortField::Downloads,
            sort_order: SortOrder::Descending,
            view_mode: ViewMode::default(),
            verified_only: false,
            refreshing: false,
            last_refreshed: None,
        },
        installed: InstalledState::default(),
        notifications: VecDeque::new(),
        loading: LoadingState::Idle,
        theme_mode: ThemeMode::Dark,
        system_is_dark: true,
        theme: AppTheme::dark(),
        omarchy_palette: None,
        module_detail: ModuleDetailState::default(),
        author_profile: AuthorProfileState::default(),
        confirmation: ConfirmationState::default(),
        preferences: PreferencesState::default(),
        spinner_frame: 0,
        last_spinner_update: Instant::now(),
        tray_enabled: false,
        tray_receiver: None,
        sandbox_status: None,
    }
}

pub fn test_app_with_registry(modules: Vec<RegistryModule>) -> App {
    let mut app = test_app();
    app.registry = Some(test_registry(modules));
    app
}

pub fn test_app_with_installed(installed: Vec<InstalledModule>) -> App {
    let mut app = test_app();
    app.installed_uuids = installed.iter().map(|m| m.uuid.to_string()).collect();
    app.installed_modules = installed;
    app
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_creates_valid_uuid() {
        let uuid = test_uuid("my-module");
        assert_eq!(uuid.to_string(), "my-module@test");
    }

    #[test]
    fn test_version_creates_valid_semver() {
        let version = test_version("2.3.4");
        assert_eq!(version.to_string(), "2.3.4");
    }

    #[test]
    fn installed_module_builder_defaults() {
        let module = InstalledModuleBuilder::new("test-mod").build();

        assert_eq!(module.uuid.to_string(), "test-mod@test");
        assert_eq!(module.version.to_string(), "1.0.0");
        assert!(module.enabled);
        assert!(!module.has_preferences);
    }

    #[test]
    fn installed_module_builder_with_options() {
        let module = InstalledModuleBuilder::new("custom")
            .version("2.0.0")
            .enabled(false)
            .has_preferences(true)
            .registry_version("2.1.0")
            .build();

        assert_eq!(module.version.to_string(), "2.0.0");
        assert!(!module.enabled);
        assert!(module.has_preferences);
        assert_eq!(
            module.registry_version.map(|v| v.to_string()),
            Some("2.1.0".to_string())
        );
    }

    #[test]
    fn registry_module_builder_defaults() {
        let module = RegistryModuleBuilder::new("cpu-monitor").build();

        assert_eq!(module.name, "cpu-monitor");
        assert_eq!(module.author, "testauthor");
        assert_eq!(module.uuid.to_string(), "cpu-monitor@testauthor");
    }

    #[test]
    fn test_app_has_empty_state() {
        let app = test_app();

        assert!(app.registry.is_none());
        assert!(app.installed_modules.is_empty());
        assert!(app.installed_uuids.is_empty());
    }

    #[test]
    fn test_app_with_registry_sets_modules() {
        let modules = vec![RegistryModuleBuilder::new("mod1").build()];
        let app = test_app_with_registry(modules);

        assert!(app.registry.is_some());
        assert_eq!(app.registry.as_ref().unwrap().modules.len(), 1);
    }

    #[test]
    fn test_app_with_installed_syncs_uuids() {
        let modules = vec![InstalledModuleBuilder::new("installed1").build()];
        let app = test_app_with_installed(modules);

        assert_eq!(app.installed_modules.len(), 1);
        assert!(app.installed_uuids.contains("installed1@test"));
    }
}
