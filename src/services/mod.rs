pub mod dependency_checker;
pub mod module_installer;
pub mod omarchy_theme;
pub mod package_config;
pub mod paths;
pub mod preferences;
pub mod settings;
pub mod waybar_config;

pub use dependency_checker::{
    DepCheckError, DepReport, DepResult, DepSpec, DepType, check_binary, check_dependencies,
    check_python_module, extract_version, is_valid_binary_name, is_valid_python_module_name,
};
pub use module_installer::{
    InstallError, InstallParams, InstallResult, InstallStage, SecureInstaller,
};
pub use omarchy_theme::{OmarchyPalette, is_omarchy_available, load_omarchy_palette};
pub use package_config::{PackageConfigError, PackageInfo, PackageToml, Permissions};
pub use preferences::{
    ModulePreferences, PreferenceField, PreferenceValue, PreferencesSchema, SelectOption,
    load_preferences, load_schema, save_preferences,
};
pub use settings::{UserSettings, load_settings, save_settings};
