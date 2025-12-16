use std::path::PathBuf;
use std::time::Duration;

use iced::widget::image;
use iced::{Subscription, Task};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc as async_mpsc;
use tokio::time::timeout;

use crate::app::Message;
use crate::domain::{InstalledModule, ModuleVersion, RegistryIndex};
use crate::services::paths;

const REGISTRY_URL: &str = "https://jtaw5649.github.io/waybar-modules-registry/index.json";

pub fn initial_load() -> Task<Message> {
    Task::batch([load_installed(), load_registry()])
}

pub fn load_registry() -> Task<Message> {
    Task::perform(fetch_registry_async(), Message::RegistryLoaded)
}

pub fn load_installed() -> Task<Message> {
    Task::perform(load_installed_async(), Message::InstalledLoaded)
}

pub fn toggle_module(uuid: String, enabled: bool) -> Task<Message> {
    Task::perform(
        toggle_module_async(uuid, enabled),
        Message::ToggleCompleted,
    )
}

pub fn uninstall_module(uuid: String) -> Task<Message> {
    Task::perform(uninstall_module_async(uuid), Message::UninstallCompleted)
}

pub fn install_module(
    uuid: String,
    name: String,
    version: Option<ModuleVersion>,
) -> Task<Message> {
    Task::perform(
        install_module_async(uuid, name, version),
        Message::InstallCompleted,
    )
}

async fn install_module_async(
    uuid: String,
    name: String,
    version: Option<ModuleVersion>,
) -> Result<InstalledModule, String> {
    use std::fs;

    let install_path = paths::module_install_path(&uuid);

    fs::create_dir_all(&install_path)
        .map_err(|e| format!("Failed to create install directory: {e}"))?;

    let version = version.unwrap_or_else(|| {
        ModuleVersion::try_from("1.0.0").expect("valid default version")
    });

    let waybar_module_name = format!("custom/{}", name.replace(' ', "-").to_lowercase());

    let installed = InstalledModule {
        uuid: crate::domain::ModuleUuid::try_from(uuid.as_str())
            .map_err(|e| format!("Invalid UUID: {e}"))?,
        version: version.clone(),
        install_path,
        enabled: false,
        waybar_module_name,
        has_preferences: false,
        installed_at: chrono::Utc::now(),
        registry_version: Some(version),
    };

    let state_path = paths::data_dir().join("installed.json");
    let mut modules: Vec<InstalledModule> = if state_path.exists() {
        let content = tokio::fs::read_to_string(&state_path)
            .await
            .map_err(|e| format!("Failed to read state: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse state: {e}"))?
    } else {
        Vec::new()
    };

    modules.push(installed.clone());

    let content = serde_json::to_string_pretty(&modules)
        .map_err(|e| format!("Failed to serialize state: {e}"))?;

    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create data directory: {e}"))?;
    }

    tokio::fs::write(&state_path, content)
        .await
        .map_err(|e| format!("Failed to write state: {e}"))?;

    tracing::info!("Installed module: {}", uuid);
    Ok(installed)
}

async fn fetch_registry_async() -> Result<RegistryIndex, String> {
    let cache_path = paths::registry_cache_path();

    if let Ok(content) = tokio::fs::read_to_string(&cache_path).await
        && let Ok(index) = serde_json::from_str::<RegistryIndex>(&content)
    {
        tracing::info!("Loaded registry from cache ({} modules)", index.modules.len());
        return Ok(index);
    }

    tracing::info!("Fetching registry from {}", REGISTRY_URL);
    let response = match reqwest::get(REGISTRY_URL).await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::warn!("Network error: {e}, using sample data");
            return Ok(sample_registry());
        }
    };

    let index: RegistryIndex = match response.json().await {
        Ok(idx) => idx,
        Err(e) => {
            tracing::warn!("Parse error: {e}, using sample data");
            return Ok(sample_registry());
        }
    };

    if let Some(parent) = cache_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if let Ok(content) = serde_json::to_string_pretty(&index) {
        let _ = tokio::fs::write(&cache_path, content).await;
    }

    tracing::info!("Fetched {} modules from registry", index.modules.len());
    Ok(index)
}

/// DEVELOPMENT ONLY - Remove before production release
///
/// This function provides sample data when the registry is unavailable.
/// Once the GitHub Pages registry is set up, this fallback should be removed
/// and the app should properly handle registry fetch failures.
#[cfg(debug_assertions)]
fn sample_registry() -> RegistryIndex {
    use crate::domain::{ModuleCategory, ModuleUuid, RegistryModule};
    use std::collections::HashMap;

    tracing::warn!("⚠️  Using SAMPLE registry data - dev mode only");

    RegistryIndex {
        version: 1,
        modules: vec![
            RegistryModule {
                uuid: ModuleUuid::try_from("weather-wttr@community").unwrap(),
                name: "Weather (wttr.in)".to_string(),
                description: "Display current weather conditions using wttr.in API with customizable location and format".to_string(),
                author: "community".to_string(),
                category: ModuleCategory::Weather,
                icon: None,
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/weather-wttr".to_string(),
                downloads: 15420,
                waybar_versions: vec!["0.10".to_string(), "0.11".to_string()],
                version: Some(ModuleVersion::try_from("1.2.0").unwrap()),
                last_updated: Some(chrono::Utc::now() - chrono::Duration::days(5)),
                rating: Some(4.5),
                verified_author: true,
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("cpu-temp@system").unwrap(),
                name: "CPU Temperature".to_string(),
                description: "Monitor CPU temperature with color-coded warnings and tooltips".to_string(),
                author: "system".to_string(),
                category: ModuleCategory::Hardware,
                icon: None,
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/cpu-temp".to_string(),
                downloads: 8932,
                waybar_versions: vec!["0.10".to_string(), "0.11".to_string()],
                version: Some(ModuleVersion::try_from("2.0.1").unwrap()),
                last_updated: Some(chrono::Utc::now() - chrono::Duration::days(12)),
                rating: Some(4.2),
                verified_author: false,
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("network-speed@network").unwrap(),
                name: "Network Speed".to_string(),
                description: "Real-time upload/download speed monitor with graph tooltip".to_string(),
                author: "network".to_string(),
                category: ModuleCategory::Network,
                icon: None,
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/network-speed".to_string(),
                downloads: 12150,
                waybar_versions: vec!["0.10".to_string(), "0.11".to_string()],
                version: Some(ModuleVersion::try_from("1.0.5").unwrap()),
                last_updated: Some(chrono::Utc::now() - chrono::Duration::days(30)),
                rating: Some(4.8),
                verified_author: true,
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("spotify-player@media").unwrap(),
                name: "Spotify Player".to_string(),
                description: "Control Spotify playback with track info, album art, and media controls".to_string(),
                author: "media".to_string(),
                category: ModuleCategory::Media,
                icon: None,
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/spotify-player".to_string(),
                downloads: 22300,
                waybar_versions: vec!["0.10".to_string(), "0.11".to_string()],
                version: Some(ModuleVersion::try_from("3.1.0").unwrap()),
                last_updated: Some(chrono::Utc::now() - chrono::Duration::days(2)),
                rating: Some(4.9),
                verified_author: true,
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("battery-status@power").unwrap(),
                name: "Battery Status".to_string(),
                description: "Advanced battery indicator with time remaining, health status, and charging animation".to_string(),
                author: "power".to_string(),
                category: ModuleCategory::System,
                icon: None,
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/battery-status".to_string(),
                downloads: 18700,
                waybar_versions: vec!["0.10".to_string(), "0.11".to_string()],
                version: Some(ModuleVersion::try_from("1.5.2").unwrap()),
                last_updated: Some(chrono::Utc::now() - chrono::Duration::days(45)),
                rating: Some(4.0),
                verified_author: false,
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("disk-usage@storage").unwrap(),
                name: "Disk Usage".to_string(),
                description: "Monitor disk space usage with customizable mount points and warning thresholds".to_string(),
                author: "storage".to_string(),
                category: ModuleCategory::Hardware,
                icon: None,
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/disk-usage".to_string(),
                downloads: 6420,
                waybar_versions: vec!["0.10".to_string(), "0.11".to_string()],
                version: Some(ModuleVersion::try_from("0.9.0").unwrap()),
                last_updated: Some(chrono::Utc::now() - chrono::Duration::days(90)),
                rating: None,
                verified_author: false,
            },
        ],
        categories: HashMap::new(),
    }
}

#[cfg(not(debug_assertions))]
fn sample_registry() -> RegistryIndex {
    // In release builds, return empty registry - proper error handling should be implemented
    tracing::error!("Registry unavailable and no sample data in release mode");
    RegistryIndex::default()
}

async fn load_installed_async() -> Result<Vec<InstalledModule>, String> {
    let state_path = paths::data_dir().join("installed.json");

    if !state_path.exists() {
        tracing::debug!("No installed modules file found");
        return Ok(Vec::new());
    }

    let content = tokio::fs::read_to_string(&state_path)
        .await
        .map_err(|e| format!("Failed to read installed modules: {e}"))?;

    let modules: Vec<InstalledModule> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse installed modules: {e}"))?;

    tracing::info!("Loaded {} installed modules", modules.len());
    Ok(modules)
}

async fn toggle_module_async(uuid: String, enabled: bool) -> Result<String, (String, String)> {
    let state_path = paths::data_dir().join("installed.json");

    let content = tokio::fs::read_to_string(&state_path)
        .await
        .map_err(|e| (uuid.clone(), format!("Failed to read state: {e}")))?;

    let mut modules: Vec<InstalledModule> =
        serde_json::from_str(&content).map_err(|e| (uuid.clone(), format!("Failed to parse state: {e}")))?;

    let module = modules
        .iter_mut()
        .find(|m| m.uuid.to_string() == uuid)
        .ok_or_else(|| (uuid.clone(), format!("Module not found: {uuid}")))?;

    module.enabled = enabled;

    let new_content =
        serde_json::to_string_pretty(&modules).map_err(|e| (uuid.clone(), format!("Failed to serialize: {e}")))?;

    tokio::fs::write(&state_path, new_content)
        .await
        .map_err(|e| (uuid.clone(), format!("Failed to save state: {e}")))?;

    tracing::info!("Module {} {}", uuid, if enabled { "enabled" } else { "disabled" });
    Ok(uuid)
}

async fn uninstall_module_async(uuid: String) -> Result<String, String> {
    let state_path = paths::data_dir().join("installed.json");
    let install_path = paths::module_install_path(&uuid);

    if install_path.exists() {
        tokio::fs::remove_dir_all(&install_path)
            .await
            .map_err(|e| format!("Failed to remove module files: {e}"))?;
    }

    let content = tokio::fs::read_to_string(&state_path)
        .await
        .map_err(|e| format!("Failed to read state: {e}"))?;

    let mut modules: Vec<InstalledModule> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse state: {e}"))?;

    modules.retain(|m| m.uuid.to_string() != uuid);

    let new_content =
        serde_json::to_string_pretty(&modules).map_err(|e| format!("Failed to serialize: {e}"))?;

    tokio::fs::write(&state_path, new_content)
        .await
        .map_err(|e| format!("Failed to save state: {e}"))?;

    tracing::info!("Uninstalled module {}", uuid);
    Ok(uuid)
}

pub fn load_screenshot(url: String) -> Task<Message> {
    Task::perform(load_screenshot_async(url), Message::ScreenshotLoaded)
}

async fn load_screenshot_async(url: String) -> Result<image::Handle, String> {
    use std::fs;
    use std::time::SystemTime;

    const CACHE_TTL_DAYS: u64 = 7;

    let cache_path = paths::screenshot_cache_path(&url);

    if let Ok(metadata) = fs::metadata(&cache_path)
        && let Ok(modified) = metadata.modified()
    {
        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::from_secs(u64::MAX));

        if age < Duration::from_secs(CACHE_TTL_DAYS * 24 * 60 * 60) {
            tracing::debug!("Loading screenshot from cache: {:?}", cache_path);
            if let Ok(bytes) = fs::read(&cache_path) {
                return Ok(image::Handle::from_bytes(bytes));
            }
        }
    }

    tracing::debug!("Fetching screenshot from {}", url);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch screenshot: {e}"))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read screenshot bytes: {e}"))?;

    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&cache_path, &bytes);

    Ok(image::Handle::from_bytes(bytes.to_vec()))
}

fn omarchy_theme_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("omarchy/current/theme")
}

pub fn watch_omarchy_theme() -> Subscription<Message> {
    Subscription::run(|| {
        iced::futures::stream::unfold(WatcherState::Ready, |state| async move {
            match state {
                WatcherState::Ready => {
                    let theme_path = omarchy_theme_path();
                    if !theme_path.exists() {
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        return Some((Message::Tick, WatcherState::Ready));
                    }

                    let (tx, rx) = async_mpsc::unbounded_channel();
                    let watcher_result = RecommendedWatcher::new(
                        move |res: Result<notify::Event, notify::Error>| {
                            if let Ok(event) = res
                                && (event.kind.is_modify() || event.kind.is_create())
                            {
                                let _ = tx.send(());
                            }
                        },
                        notify::Config::default(),
                    );

                    match watcher_result {
                        Ok(mut watcher) => {
                            if watcher.watch(&theme_path, RecursiveMode::NonRecursive).is_ok() {
                                Some((Message::Tick, WatcherState::Watching { _watcher: watcher, rx }))
                            } else {
                                Some((Message::Tick, WatcherState::Unavailable))
                            }
                        }
                        Err(_) => Some((Message::Tick, WatcherState::Unavailable)),
                    }
                }
                WatcherState::Watching { _watcher, mut rx } => {
                    match timeout(Duration::from_millis(500), rx.recv()).await {
                        Ok(Some(())) => {
                            Some((Message::OmarchyThemeChanged, WatcherState::Watching { _watcher, rx }))
                        }
                        Ok(None) => {
                            Some((Message::Tick, WatcherState::Ready))
                        }
                        Err(_) => {
                            Some((Message::Tick, WatcherState::Watching { _watcher, rx }))
                        }
                    }
                }
                WatcherState::Unavailable => {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    Some((Message::Tick, WatcherState::Ready))
                }
            }
        })
    })
}

enum WatcherState {
    Ready,
    Watching {
        _watcher: RecommendedWatcher,
        rx: async_mpsc::UnboundedReceiver<()>,
    },
    Unavailable,
}

pub fn clear_cache() -> Task<Message> {
    Task::perform(clear_cache_async(), Message::CacheClearCompleted)
}

async fn clear_cache_async() -> Result<(), String> {
    let cache_path = paths::cache_dir();

    if !cache_path.exists() {
        return Ok(());
    }

    tokio::fs::remove_dir_all(&cache_path)
        .await
        .map_err(|e| format!("Failed to clear cache: {e}"))?;

    tokio::fs::create_dir_all(&cache_path)
        .await
        .map_err(|e| format!("Failed to recreate cache directory: {e}"))?;

    tracing::info!("Cache cleared");
    Ok(())
}

pub fn reset_settings() -> Task<Message> {
    Task::perform(reset_settings_async(), Message::SettingsResetCompleted)
}

async fn reset_settings_async() -> Result<(), String> {
    let prefs_path = paths::preferences_dir();

    if !prefs_path.exists() {
        return Ok(());
    }

    tokio::fs::remove_dir_all(&prefs_path)
        .await
        .map_err(|e| format!("Failed to reset settings: {e}"))?;

    tokio::fs::create_dir_all(&prefs_path)
        .await
        .map_err(|e| format!("Failed to recreate preferences directory: {e}"))?;

    tracing::info!("Settings reset");
    Ok(())
}
