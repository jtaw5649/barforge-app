use std::path::PathBuf;

use once_cell::sync::Lazy;

static HOME_DIR: Lazy<PathBuf> = Lazy::new(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));

static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::data_dir()
        .unwrap_or_else(|| HOME_DIR.join(".local/share"))
        .join("waybar-manager")
});

static CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::config_dir()
        .unwrap_or_else(|| HOME_DIR.join(".config"))
        .join("waybar-manager")
});

static WAYBAR_CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::config_dir()
        .unwrap_or_else(|| HOME_DIR.join(".config"))
        .join("waybar")
});

pub fn data_dir() -> &'static PathBuf {
    &DATA_DIR
}

pub fn config_dir() -> &'static PathBuf {
    &CONFIG_DIR
}

pub fn modules_dir() -> PathBuf {
    data_dir().join("modules")
}

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| HOME_DIR.join(".cache"))
        .join("waybar-manager")
}

pub fn registry_cache_path() -> PathBuf {
    cache_dir().join("registry.json")
}

pub fn preferences_dir() -> PathBuf {
    config_dir().join("prefs")
}

pub fn waybar_config_path() -> PathBuf {
    WAYBAR_CONFIG_DIR.join("config.jsonc")
}

pub fn waybar_style_path() -> PathBuf {
    WAYBAR_CONFIG_DIR.join("style.css")
}

pub fn module_install_path(uuid: &str) -> PathBuf {
    modules_dir().join(uuid)
}

pub fn module_preferences_path(uuid: &str) -> PathBuf {
    preferences_dir().join(format!("{}.json", uuid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_dir_exists() {
        let path = data_dir();
        assert!(path.to_string_lossy().contains("waybar-manager"));
    }

    #[test]
    fn test_config_dir_exists() {
        let path = config_dir();
        assert!(path.to_string_lossy().contains("waybar-manager"));
    }

    #[test]
    fn test_modules_dir_under_data() {
        let path = modules_dir();
        assert!(path.starts_with(data_dir()));
        assert!(path.to_string_lossy().ends_with("modules"));
    }

    #[test]
    fn test_cache_dir_contains_app_name() {
        let path = cache_dir();
        assert!(path.to_string_lossy().contains("waybar-manager"));
    }

    #[test]
    fn test_registry_cache_path_is_json() {
        let path = registry_cache_path();
        assert!(path.to_string_lossy().ends_with("registry.json"));
    }

    #[test]
    fn test_preferences_dir_under_config() {
        let path = preferences_dir();
        assert!(path.starts_with(config_dir()));
    }

    #[test]
    fn test_waybar_config_path_is_jsonc() {
        let path = waybar_config_path();
        assert!(path.to_string_lossy().ends_with("config.jsonc"));
    }

    #[test]
    fn test_waybar_style_path_is_css() {
        let path = waybar_style_path();
        assert!(path.to_string_lossy().ends_with("style.css"));
    }

    #[test]
    fn test_module_install_path_contains_uuid() {
        let path = module_install_path("weather@test");
        assert!(path.to_string_lossy().contains("weather@test"));
    }

    #[test]
    fn test_module_preferences_path_is_json() {
        let path = module_preferences_path("weather@test");
        assert!(path.to_string_lossy().ends_with("weather@test.json"));
    }
}
