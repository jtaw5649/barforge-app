pub mod omarchy_theme;
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
