pub mod dependency_checker;
pub mod module_installer;
pub mod omarchy_theme;
pub mod package_config;
pub mod paths;
pub mod preferences;
pub mod settings;
pub mod waybar_config;

pub use omarchy_theme::{is_omarchy_available, load_omarchy_palette, OmarchyPalette};
pub use preferences::{
    load_preferences, load_schema, save_preferences, ModulePreferences, PreferenceField,
    PreferenceValue, PreferencesSchema, SelectOption,
};
pub use settings::{load_settings, save_settings, UserSettings};
pub use dependency_checker::{
    check_dependencies, check_binary, check_python_module, extract_version,
    is_valid_binary_name, is_valid_python_module_name,
    DepCheckError, DepReport, DepResult, DepSpec, DepType,
};
pub use package_config::{PackageConfigError, PackageToml, PackageInfo, Permissions};
pub use module_installer::{InstallError, InstallParams, InstallResult, InstallStage, SecureInstaller};
