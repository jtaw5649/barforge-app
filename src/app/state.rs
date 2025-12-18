use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::time::{Duration, Instant};

use iced::widget::image;

use crate::domain::{InstalledModule, ModuleCategory, RegistryIndex};
use crate::security::SandboxStatus;
use crate::services::{
    is_omarchy_available, load_omarchy_palette, load_settings, InstallStage, ModulePreferences,
    OmarchyPalette, PreferencesSchema,
};
use crate::theme::{AppTheme, ThemeMode};

use std::sync::mpsc::Receiver;
use crate::tray::TrayEvent;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Browse,
    Installed,
    Updates,
    Settings,
    ModuleDetail(String),
}

impl Screen {
    pub fn label(&self) -> &'static str {
        match self {
            Screen::Browse => "Browse",
            Screen::Installed => "Installed",
            Screen::Updates => "Updates",
            Screen::Settings => "Settings",
            Screen::ModuleDetail(_) => "Module Detail",
        }
    }

    pub fn is_main_nav(&self) -> bool {
        matches!(self, Screen::Browse | Screen::Installed | Screen::Updates)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CategoryFilter(pub Option<ModuleCategory>);

impl CategoryFilter {
    pub fn all() -> Vec<Self> {
        std::iter::once(Self(None))
            .chain(ModuleCategory::all().iter().map(|c| Self(Some(*c))))
            .collect()
    }

    pub fn inner(&self) -> Option<ModuleCategory> {
        self.0
    }
}

impl fmt::Display for CategoryFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(category) => write!(f, "{}", category.display_name()),
            None => write!(f, "All Categories"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortField {
    #[default]
    Name,
    Downloads,
    #[serde(rename = "recentlyupdated")]
    RecentlyUpdated,
    Rating,
}

impl SortField {
    pub fn all() -> &'static [Self] {
        &[Self::Name, Self::Downloads, Self::RecentlyUpdated, Self::Rating]
    }
}

impl fmt::Display for SortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortField::Name => write!(f, "Name"),
            SortField::Downloads => write!(f, "Downloads"),
            SortField::RecentlyUpdated => write!(f, "Recently Updated"),
            SortField::Rating => write!(f, "Rating"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Ascending,
    #[default]
    Descending,
}

impl SortOrder {
    pub fn toggle(self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    #[default]
    Cards,
    Table,
}

#[derive(Debug, Clone, Default)]
pub struct BrowseState {
    pub search_query: String,
    pub selected_category: CategoryFilter,
    pub pending_search: Option<String>,
    pub search_debounce_start: Option<std::time::Instant>,
    pub sort_field: SortField,
    pub sort_order: SortOrder,
    pub view_mode: ViewMode,
    pub verified_only: bool,
    pub refreshing: bool,
    pub last_refreshed: Option<std::time::Instant>,
}

#[derive(Debug, Clone, Default)]
pub struct InstalledState {
    pub toggling: HashSet<String>,
    pub uninstalling: HashSet<String>,
    pub updating: HashSet<String>,
    pub updating_all: bool,
    pub search_query: String,
    pub pending_search: Option<String>,
    pub search_debounce_start: Option<std::time::Instant>,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub kind: NotificationKind,
    pub created_at: Instant,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NotificationKind {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Default)]
pub enum LoadingState {
    #[default]
    Idle,
    Loading,
    Failed(String),
}

impl LoadingState {
    pub fn is_loading(&self) -> bool {
        matches!(self, LoadingState::Loading)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ScreenshotState {
    #[default]
    NotLoaded,
    Loading,
    Loaded(image::Handle),
    Failed,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleDetailState {
    pub screenshot: ScreenshotState,
    pub installing: bool,
    pub install_stage: Option<InstallStage>,
}

#[derive(Debug, Clone)]
pub enum ConfirmationAction {
    UninstallModule { uuid: String, name: String },
}

#[derive(Debug, Clone, Default)]
pub struct ConfirmationState {
    pub pending_action: Option<ConfirmationAction>,
}

#[derive(Debug, Clone, Default)]
pub struct PreferencesState {
    pub open_for: Option<String>,
    pub schema: Option<PreferencesSchema>,
    pub values: ModulePreferences,
    pub module_name: String,
}

pub struct App {
    pub screen: Screen,

    pub registry: Option<RegistryIndex>,
    pub installed_modules: Vec<InstalledModule>,
    pub installed_uuids: HashSet<String>,

    pub browse: BrowseState,
    pub installed: InstalledState,

    pub notifications: VecDeque<Notification>,
    pub loading: LoadingState,

    pub theme_mode: ThemeMode,
    pub system_is_dark: bool,
    pub theme: AppTheme,
    pub omarchy_palette: Option<OmarchyPalette>,

    pub module_detail: ModuleDetailState,
    pub confirmation: ConfirmationState,
    pub preferences: PreferencesState,

    pub spinner_frame: usize,
    pub last_spinner_update: Instant,

    pub tray_enabled: bool,
    pub tray_receiver: Option<Receiver<TrayEvent>>,

    pub sandbox_status: Option<SandboxStatus>,
}

impl Default for App {
    fn default() -> Self {
        let system_is_dark = true;
        let settings = load_settings();

        let saved_theme_mode = settings.theme_mode;
        let use_omarchy = is_omarchy_available()
            && (saved_theme_mode == ThemeMode::Omarchy || saved_theme_mode == ThemeMode::System);

        let (theme_mode, omarchy_palette, theme) = if use_omarchy {
            let palette = load_omarchy_palette();
            let theme = palette
                .as_ref()
                .map(AppTheme::from_omarchy)
                .unwrap_or_else(AppTheme::dark);
            (ThemeMode::Omarchy, palette, theme)
        } else {
            let mode = saved_theme_mode;
            (mode, None, AppTheme::from_mode(mode, system_is_dark))
        };

        let browse = BrowseState {
            view_mode: settings.view_mode,
            sort_field: settings.sort_field,
            sort_order: settings.sort_order,
            ..Default::default()
        };

        let tray_enabled = settings.tray_enabled;

        let tray_receiver = if tray_enabled {
            crate::tray::init()
        } else {
            None
        };

        Self {
            screen: Screen::default(),
            registry: None,
            installed_modules: Vec::new(),
            installed_uuids: HashSet::new(),
            browse,
            installed: InstalledState::default(),
            notifications: VecDeque::new(),
            loading: LoadingState::default(),
            theme_mode,
            system_is_dark,
            theme,
            omarchy_palette,
            module_detail: ModuleDetailState::default(),
            confirmation: ConfirmationState::default(),
            preferences: PreferencesState::default(),
            spinner_frame: 0,
            last_spinner_update: Instant::now(),
            tray_enabled,
            tray_receiver,
            sandbox_status: None,
        }
    }
}

const SPINNER_FRAMES: &[&str] = &["◐", "◓", "◑", "◒"];

impl App {
    pub fn spinner_char(&self) -> &'static str {
        SPINNER_FRAMES[self.spinner_frame % SPINNER_FRAMES.len()]
    }

    pub fn advance_spinner(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_spinner_update) >= Duration::from_millis(80) {
            self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
            self.last_spinner_update = now;
        }
    }

    pub fn update_theme(&mut self) {
        self.theme = match self.theme_mode {
            ThemeMode::Omarchy => self
                .omarchy_palette
                .as_ref()
                .map(AppTheme::from_omarchy)
                .unwrap_or_else(AppTheme::dark),
            other => AppTheme::from_mode(other, self.system_is_dark),
        };
    }

    pub fn set_omarchy_palette(&mut self, palette: Option<OmarchyPalette>) {
        self.omarchy_palette = palette;
        if self.theme_mode == ThemeMode::Omarchy {
            self.update_theme();
        }
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
        if mode == ThemeMode::Omarchy && self.omarchy_palette.is_none() {
            self.omarchy_palette = load_omarchy_palette();
        }
        self.update_theme();
    }

    pub fn set_system_dark(&mut self, is_dark: bool) {
        self.system_is_dark = is_dark;
        self.update_theme();
    }

    pub fn apply_debounced_searches(&mut self) {
        const DEBOUNCE_MS: u64 = 150;

        if let (Some(query), Some(start)) = (&self.browse.pending_search, self.browse.search_debounce_start)
            && start.elapsed() >= Duration::from_millis(DEBOUNCE_MS)
        {
            self.browse.search_query = query.clone();
            self.browse.pending_search = None;
            self.browse.search_debounce_start = None;
        }

        if let (Some(query), Some(start)) = (&self.installed.pending_search, self.installed.search_debounce_start)
            && start.elapsed() >= Duration::from_millis(DEBOUNCE_MS)
        {
            self.installed.search_query = query.clone();
            self.installed.pending_search = None;
            self.installed.search_debounce_start = None;
        }
    }

    pub fn browse_search_display(&self) -> &str {
        self.browse.pending_search.as_deref().unwrap_or(&self.browse.search_query)
    }

    pub fn installed_search_display(&self) -> &str {
        self.installed.pending_search.as_deref().unwrap_or(&self.installed.search_query)
    }

    pub fn push_notification(&mut self, message: String, kind: NotificationKind) {
        const MAX_NOTIFICATIONS: usize = 3;
        self.notifications.push_back(Notification {
            message,
            kind,
            created_at: Instant::now(),
        });
        while self.notifications.len() > MAX_NOTIFICATIONS {
            self.notifications.pop_front();
        }
    }

    pub fn filtered_modules(&self) -> Vec<&crate::domain::RegistryModule> {
        let Some(registry) = &self.registry else {
            return Vec::new();
        };

        let query = &self.browse.search_query;
        let category = self.browse.selected_category.inner();
        let verified_only = self.browse.verified_only;

        let mut modules: Vec<_> = registry
            .modules
            .iter()
            .filter(|m| {
                let matches_search = query.is_empty() || m.matches_search(query);
                let matches_category =
                    category.is_none() || category.as_ref() == Some(&m.category);
                let matches_verified = !verified_only || m.verified_author;
                matches_search && matches_category && matches_verified
            })
            .collect();

        modules.sort_by(|a, b| {
            let cmp = match self.browse.sort_field {
                SortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortField::Downloads => a.downloads.cmp(&b.downloads),
                SortField::RecentlyUpdated => a.last_updated.cmp(&b.last_updated),
                SortField::Rating => {
                    let a_rating = a.rating.unwrap_or(0.0);
                    let b_rating = b.rating.unwrap_or(0.0);
                    a_rating.partial_cmp(&b_rating).unwrap_or(std::cmp::Ordering::Equal)
                }
            };
            match self.browse.sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });

        modules
    }

    pub fn sync_registry_versions(&mut self, registry: &RegistryIndex) {
        for installed in &mut self.installed_modules {
            let uuid_str = installed.uuid.to_string();
            if let Some(registry_module) = registry.find_by_uuid(&uuid_str) {
                installed.registry_version = registry_module.version.clone();
            }
        }
    }

    pub fn modules_with_updates(&self) -> Vec<&InstalledModule> {
        self.installed_modules
            .iter()
            .filter(|m| m.has_update())
            .collect()
    }

    pub fn update_count(&self) -> usize {
        self.installed_modules.iter().filter(|m| m.has_update()).count()
    }

    pub fn save_settings(&self) {
        let settings = crate::services::UserSettings {
            theme_mode: self.theme_mode,
            view_mode: self.browse.view_mode,
            sort_field: self.browse.sort_field,
            sort_order: self.browse.sort_order,
            tray_enabled: self.tray_enabled,
        };
        if let Err(e) = crate::services::save_settings(&settings) {
            tracing::warn!("Failed to save user settings: {e}");
        }
    }

    pub fn poll_tray_events(&self) -> Option<TrayEvent> {
        self.tray_receiver
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
    }
}
