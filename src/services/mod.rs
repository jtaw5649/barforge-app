pub mod omarchy_theme;
pub mod paths;
pub mod preferences;
pub mod settings;
pub mod waybar_config;
mod module_service;
mod registry_service;

pub use module_service::{ModuleError, ModuleService};
pub use omarchy_theme::{is_omarchy_available, load_omarchy_palette, OmarchyPalette};
pub use preferences::{
    load_preferences, load_schema, save_preferences, ModulePreferences, PreferenceField,
    PreferenceValue, PreferencesSchema, SelectOption,
};
pub use registry_service::{RegistryError, RegistryService};
pub use settings::{load_settings, save_settings, UserSettings};
