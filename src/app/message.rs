use iced::widget::image;

use crate::app::state::{CategoryFilter, ConfirmationAction, NotificationKind, Screen, SortField, ViewMode};
use crate::domain::{InstalledModule, RegistryIndex};
use crate::services::PreferenceValue;
use crate::theme::ThemeMode;

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Screen),

    SearchChanged(String),
    CategorySelected(CategoryFilter),
    SetSortField(SortField),
    ToggleSortOrder,
    SetViewMode(ViewMode),
    ToggleVerifiedOnly,
    ModuleClicked(String),
    InstallModule(String),

    ToggleModule { uuid: String, enabled: bool },
    UninstallModule(String),
    OpenPreferences(String),

    InstalledSearchChanged(String),
    ClearInstalledSearch,

    RegistryLoaded(Result<RegistryIndex, String>),
    InstalledLoaded(Result<Vec<InstalledModule>, String>),
    InstallCompleted(Result<InstalledModule, String>),
    ToggleCompleted(Result<String, (String, String)>),
    UninstallCompleted(Result<String, String>),

    ShowNotification(String, NotificationKind),
    DismissNotification,
    Tick,

    SystemThemeChanged(bool),
    SetThemeMode(ThemeMode),
    OmarchyThemeChanged,

    NavigateBack,
    ScreenshotLoaded(Result<image::Handle, String>),
    DetailInstallModule,
    OpenRepoUrl(String),

    RequestConfirmation(ConfirmationAction),
    ConfirmAction,
    CancelConfirmation,

    ClearCache,
    CacheClearCompleted(Result<(), String>),
    ResetSettings,
    SettingsResetCompleted(Result<(), String>),
    ToggleTray(bool),

    FocusSearch,
    EscapePressed,

    PreferenceChanged(String, String, PreferenceValue),
    ClosePreferences,
    ResetPreferences(String),

    TrayShowWindow,
    TrayCheckUpdates,
    TrayQuit,
}
