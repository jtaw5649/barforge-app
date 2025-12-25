use std::path::PathBuf;
use std::time::Duration;

use once_cell::sync::Lazy;
use reqwest::Client;
use url::Url;

pub const API_BASE_URL: &str = "https://api.barforge.dev";
pub const REGISTRY_URL: &str = "https://api.barforge.dev/api/v1/index";
pub const SECURITY_CHECK_URL: &str = "https://api.barforge.dev/security/check";
pub const PACKAGES_BASE_URL: &str = "https://api.barforge.dev/packages";

#[must_use]
pub fn package_url(uuid: &str, version: &str) -> String {
    packages_url(uuid, version, "package.tar.gz")
}

#[must_use]
pub fn signature_url(uuid: &str, version: &str) -> String {
    packages_url(uuid, version, "package.tar.gz.minisig")
}

fn packages_url(uuid: &str, version: &str, filename: &str) -> String {
    let mut url = Url::parse(PACKAGES_BASE_URL).expect("valid packages base url");
    {
        let mut segments = url
            .path_segments_mut()
            .expect("packages base url must be a base");
        segments.push(uuid);
        segments.push(version);
        segments.push(filename);
    }
    url.to_string()
}

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(4)
        .build()
        .unwrap_or_else(|e| {
            tracing::error!("Failed to create configured HTTP client: {e}, using default");
            Client::new()
        })
});

static HOME_DIR: Lazy<PathBuf> =
    Lazy::new(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));

static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::data_dir()
        .unwrap_or_else(|| HOME_DIR.join(".local/share"))
        .join("barforge")
});

static CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::config_dir()
        .unwrap_or_else(|| HOME_DIR.join(".config"))
        .join("barforge")
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
        .join("barforge")
}

pub fn registry_cache_path() -> PathBuf {
    cache_dir().join("registry.json")
}

pub fn screenshots_cache_dir() -> PathBuf {
    cache_dir().join("screenshots")
}

pub fn screenshot_cache_path(url: &str) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();
    screenshots_cache_dir().join(format!("{:016x}.png", hash))
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
        assert!(path.to_string_lossy().contains("barforge"));
    }

    #[test]
    fn test_config_dir_exists() {
        let path = config_dir();
        assert!(path.to_string_lossy().contains("barforge"));
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
        assert!(path.to_string_lossy().contains("barforge"));
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

    #[test]
    fn test_screenshots_cache_dir_under_cache() {
        let path = screenshots_cache_dir();
        assert!(path.starts_with(cache_dir()));
        assert!(path.to_string_lossy().ends_with("screenshots"));
    }

    #[test]
    fn test_screenshot_cache_path_is_png() {
        let path = screenshot_cache_path("https://example.com/image.png");
        assert!(path.to_string_lossy().ends_with(".png"));
    }

    #[test]
    fn test_screenshot_cache_path_deterministic() {
        let url = "https://example.com/screenshot.png";
        let path1 = screenshot_cache_path(url);
        let path2 = screenshot_cache_path(url);
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_screenshot_cache_path_different_urls() {
        let path1 = screenshot_cache_path("https://example.com/a.png");
        let path2 = screenshot_cache_path("https://example.com/b.png");
        assert_ne!(path1, path2);
    }

    #[test]
    fn test_package_url_encoding() {
        let url = package_url("weather@test", "1.0.0+build.1");
        assert!(url.contains("weather@test"));
        assert!(url.contains("1.0.0+build.1"));
        assert!(url.ends_with("package.tar.gz"));
        assert!(!url.contains("%40"));
        assert!(!url.contains("%2"));
    }

    #[test]
    fn test_signature_url_encoding() {
        let url = signature_url("weather@test", "1.0.0+build.1");
        assert!(url.contains("weather@test"));
        assert!(url.contains("1.0.0+build.1"));
        assert!(url.ends_with("package.tar.gz.minisig"));
        assert!(!url.contains("%40"));
        assert!(!url.contains("%2"));
    }

    #[test]
    fn test_security_check_url_constant() {
        assert!(SECURITY_CHECK_URL.contains("security/check"));
    }

    #[test]
    fn api_urls_use_custom_domain() {
        assert!(API_BASE_URL.contains("api.barforge.dev"));
        assert!(REGISTRY_URL.contains("api.barforge.dev"));
        assert!(SECURITY_CHECK_URL.contains("api.barforge.dev"));
        assert!(PACKAGES_BASE_URL.contains("api.barforge.dev"));
    }
}
