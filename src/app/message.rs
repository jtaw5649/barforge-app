use iced::widget::image;

use crate::app::state::{CategoryFilter, ConfirmationAction, NotificationKind, Screen, SortField, ViewMode};
use crate::domain::{BarSection, InstalledModule, ModuleUuid, RegistryIndex};
use crate::services::{DepReport, InstallStage, PreferenceValue};
use crate::security::SandboxStatus;
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
    ModuleClicked(ModuleUuid),
    InstallModule(ModuleUuid),

    ToggleModule { uuid: ModuleUuid, enabled: bool },
    SetModulePosition { uuid: ModuleUuid, section: BarSection },
    PositionChanged(Result<String, String>),
    UninstallModule(ModuleUuid),
    UpdateModule(ModuleUuid),
    UpdateAllModules,
    OpenPreferences(ModuleUuid),

    InstalledSearchChanged(String),
    ClearInstalledSearch,

    RefreshRegistry,
    RegistryLoaded(Result<RegistryIndex, String>),
    RegistryRefreshed(Result<RegistryIndex, String>),
    InstalledLoaded(Result<Vec<InstalledModule>, String>),
    InstallCompleted(Result<InstalledModule, String>),
    ToggleCompleted(Result<String, (String, String)>),
    UninstallCompleted(Result<String, (String, String)>),
    UpdateCompleted(Result<InstalledModule, String>),
    UpdateAllCompleted(Result<usize, String>),

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

    PreferenceChanged(ModuleUuid, String, PreferenceValue),
    ClosePreferences,
    ResetPreferences(ModuleUuid),

    TrayShowWindow,
    TrayCheckUpdates,
    TrayQuit,

    InstallProgress { uuid: ModuleUuid, stage: InstallStage },
    DependencyCheckCompleted(Result<DepReport, String>),
    RevocationCheckCompleted(Result<(), String>),
    SignatureVerified(Result<(), String>),
    SandboxStatusChanged(SandboxStatus),
}
