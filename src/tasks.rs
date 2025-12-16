use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Duration;

use flate2::read::GzDecoder;
use iced::widget::image;
use iced::{Subscription, Task};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use tar::Archive;
use tokio::sync::mpsc as async_mpsc;
use tokio::time::timeout;

use crate::app::Message;
use crate::domain::{BarSection, InstalledModule, ModuleVersion, RegistryIndex};

static DEFAULT_VERSION: Lazy<ModuleVersion> = Lazy::new(|| {
    ModuleVersion::try_from("1.0.0").unwrap_or_else(|_| {
        unreachable!("1.0.0 is always valid semver")
    })
});
use crate::security::validate_extraction_path;
use crate::services::paths::{self, HTTP_CLIENT, REGISTRY_URL};

pub fn initial_load() -> Task<Message> {
    Task::batch([load_installed(), load_registry()])
}

pub fn load_registry() -> Task<Message> {
    Task::perform(fetch_registry_async(), Message::RegistryLoaded)
}

pub fn refresh_registry() -> Task<Message> {
    Task::perform(refresh_registry_async(), Message::RegistryRefreshed)
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

pub fn change_module_position(uuid: String, section: crate::domain::BarSection) -> Task<Message> {
    Task::perform(
        change_module_position_async(uuid, section),
        Message::PositionChanged,
    )
}

pub fn update_module(uuid: String, repo_url: String, new_version: ModuleVersion) -> Task<Message> {
    Task::perform(
        update_module_async(uuid, repo_url, new_version),
        Message::UpdateCompleted,
    )
}

pub fn update_all_modules(updates: Vec<(String, String, ModuleVersion)>) -> Task<Message> {
    Task::perform(update_all_modules_async(updates), Message::UpdateAllCompleted)
}

pub fn install_module(
    uuid: String,
    name: String,
    version: Option<ModuleVersion>,
    repo_url: String,
) -> Task<Message> {
    Task::perform(
        install_module_async(uuid, name, version, repo_url),
        Message::InstallCompleted,
    )
}

async fn install_module_async(
    uuid: String,
    name: String,
    version: Option<ModuleVersion>,
    repo_url: String,
) -> Result<InstalledModule, String> {
    let install_path = paths::module_install_path(&uuid);

    tokio::fs::create_dir_all(&install_path)
        .await
        .map_err(|e| format!("Failed to create install directory: {e}"))?;

    download_module_files(&repo_url, &install_path).await?;
    make_scripts_executable(&install_path).await?;

    let has_preferences = install_path.join("preferences.schema.json").exists();

    let version = version.unwrap_or_else(|| DEFAULT_VERSION.clone());

    let waybar_module_name = format!("custom/{}", name.replace(' ', "-").to_lowercase());

    let installed = InstalledModule {
        uuid: crate::domain::ModuleUuid::try_from(uuid.as_str())
            .map_err(|e| format!("Invalid UUID: {e}"))?,
        version: version.clone(),
        install_path,
        enabled: false,
        waybar_module_name,
        has_preferences,
        installed_at: chrono::Utc::now(),
        registry_version: Some(version),
        position: None,
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
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create data directory: {e}"))?;
    }

    tokio::fs::write(&state_path, content)
        .await
        .map_err(|e| format!("Failed to write state: {e}"))?;

    tracing::info!("Installed module: {}", uuid);
    Ok(installed)
}

async fn update_module_async(
    uuid: String,
    repo_url: String,
    new_version: ModuleVersion,
) -> Result<InstalledModule, String> {
    let install_path = paths::module_install_path(&uuid);

    if install_path.exists() {
        tokio::fs::remove_dir_all(&install_path)
            .await
            .map_err(|e| format!("Failed to remove old module files: {e}"))?;
    }

    tokio::fs::create_dir_all(&install_path)
        .await
        .map_err(|e| format!("Failed to create install directory: {e}"))?;

    download_module_files(&repo_url, &install_path).await?;

    let has_preferences = install_path.join("preferences.schema.json").exists();

    let state_path = paths::data_dir().join("installed.json");
    let content = tokio::fs::read_to_string(&state_path)
        .await
        .map_err(|e| format!("Failed to read state: {e}"))?;

    let mut modules: Vec<InstalledModule> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse state: {e}"))?;

    let module = modules
        .iter_mut()
        .find(|m| m.uuid.to_string() == uuid)
        .ok_or_else(|| format!("Module not found: {uuid}"))?;

    module.version = new_version.clone();
    module.registry_version = Some(new_version);
    module.has_preferences = has_preferences;

    let updated = module.clone();

    let new_content =
        serde_json::to_string_pretty(&modules).map_err(|e| format!("Failed to serialize: {e}"))?;

    tokio::fs::write(&state_path, new_content)
        .await
        .map_err(|e| format!("Failed to save state: {e}"))?;

    tracing::info!("Updated module: {}", uuid);
    Ok(updated)
}

async fn update_all_modules_async(
    updates: Vec<(String, String, ModuleVersion)>,
) -> Result<usize, String> {
    let mut success_count = 0;

    for (uuid, repo_url, new_version) in updates {
        match update_module_async(uuid.clone(), repo_url, new_version).await {
            Ok(_) => {
                success_count += 1;
                tracing::info!("Updated module: {}", uuid);
            }
            Err(e) => {
                tracing::warn!("Failed to update module {}: {}", uuid, e);
            }
        }
    }

    Ok(success_count)
}

fn parse_github_url(repo_url: &str) -> Result<(String, String), String> {
    let url = repo_url
        .trim_end_matches('/')
        .trim_end_matches(".git");

    let parts: Vec<&str> = url.split('/').collect();
    let len = parts.len();

    if len < 2 {
        return Err(format!("Invalid GitHub URL: {repo_url}"));
    }

    let owner = parts[len - 2].to_string();
    let repo = parts[len - 1].to_string();

    if owner.is_empty() || repo.is_empty() {
        return Err(format!("Could not extract owner/repo from: {repo_url}"));
    }

    Ok((owner, repo))
}

async fn download_module_files(repo_url: &str, install_path: &Path) -> Result<(), String> {
    let (owner, repo) = parse_github_url(repo_url)?;

    let tarball_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/tarball/main"
    );

    tracing::info!("Downloading module from {}", tarball_url);

    let response = HTTP_CLIENT
        .get(&tarball_url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "waybar-manager")
        .send()
        .await
        .map_err(|e| format!("Failed to download module: {e}"))?;

    if !response.status().is_success() {
        let fallback_url = format!(
            "https://api.github.com/repos/{owner}/{repo}/tarball/master"
        );

        tracing::info!("main branch not found, trying master: {}", fallback_url);

        let response = HTTP_CLIENT
            .get(&fallback_url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "waybar-manager")
            .send()
            .await
            .map_err(|e| format!("Failed to download module: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to download from GitHub: HTTP {}",
                response.status()
            ));
        }

        return extract_tarball(response, install_path).await;
    }

    extract_tarball(response, install_path).await
}

async fn extract_tarball(response: reqwest::Response, install_path: &Path) -> Result<(), String> {
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    let install_path = install_path.to_path_buf();

    tokio::task::spawn_blocking(move || {
        extract_tarball_sync(&bytes, &install_path)
    })
    .await
    .map_err(|e| format!("Task failed: {e}"))?
}

fn extract_tarball_sync(bytes: &[u8], install_path: &Path) -> Result<(), String> {
    let cursor = Cursor::new(bytes);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);

    let mut extracted_count = 0;

    for entry in archive
        .entries()
        .map_err(|e| format!("Failed to read archive entries: {e}"))?
    {
        let mut entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;

        let path = entry
            .path()
            .map_err(|e| format!("Failed to get entry path: {e}"))?
            .into_owned();

        let components: Vec<_> = path.components().collect();
        if components.len() <= 1 {
            continue;
        }

        let relative_path: PathBuf = components[1..].iter().collect();
        let dest_path = validate_extraction_path(install_path, &relative_path)
            .map_err(|e| format!("Path traversal blocked: {e}"))?;

        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&dest_path)
                .map_err(|e| format!("Failed to create directory {}: {e}", dest_path.display()))?;
        } else if entry.header().entry_type().is_file() {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {e}"))?;
            }

            let mut file = std::fs::File::create(&dest_path)
                .map_err(|e| format!("Failed to create file {}: {e}", dest_path.display()))?;

            std::io::copy(&mut entry, &mut file)
                .map_err(|e| format!("Failed to write file {}: {e}", dest_path.display()))?;

            extracted_count += 1;
        }
    }

    tracing::info!("Extracted {} files to {}", extracted_count, install_path.display());

    if extracted_count == 0 {
        return Err("No files extracted from archive".to_string());
    }

    Ok(())
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
    let response = match HTTP_CLIENT.get(REGISTRY_URL).send().await {
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

async fn refresh_registry_async() -> Result<RegistryIndex, String> {
    let cache_path = paths::registry_cache_path();
    let _ = tokio::fs::remove_file(&cache_path).await;

    tracing::info!("Force refreshing registry from {}", REGISTRY_URL);
    let response = HTTP_CLIENT
        .get(REGISTRY_URL)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let index: RegistryIndex = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {e}"))?;

    if let Some(parent) = cache_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if let Ok(content) = serde_json::to_string_pretty(&index) {
        let _ = tokio::fs::write(&cache_path, content).await;
    }

    tracing::info!("Refreshed registry: {} modules", index.modules.len());
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
    use crate::services::waybar_config;

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

    let waybar_module_name = module.waybar_module_name.clone();
    let install_path = module.install_path.clone();
    let section = module
        .position
        .as_ref()
        .map(|p| p.section)
        .unwrap_or(BarSection::Center);

    module.enabled = enabled;

    let new_content =
        serde_json::to_string_pretty(&modules).map_err(|e| (uuid.clone(), format!("Failed to serialize: {e}")))?;

    tokio::fs::write(&state_path, new_content)
        .await
        .map_err(|e| (uuid.clone(), format!("Failed to save state: {e}")))?;

    if let Ok(waybar_content) = waybar_config::load_config().await {
        let modified = if enabled {
            let config_path = install_path.join("config.jsonc");
            let with_module_config = if config_path.exists() {
                if let Ok(module_config) = tokio::fs::read_to_string(&config_path).await {
                    let prefs = crate::services::preferences::load_preferences(&uuid);
                    tracing::debug!("Loaded {} preferences for {}", prefs.len(), uuid);
                    let module_config = waybar_config::substitute_preferences(&module_config, &prefs);
                    tracing::debug!("Substituted config: {}", module_config);
                    let install_path_str = install_path.to_string_lossy();
                    waybar_config::merge_module_config(&waybar_content, &module_config, &install_path_str)
                        .unwrap_or_else(|e| {
                            tracing::warn!("Failed to merge module config: {e}");
                            waybar_content.clone()
                        })
                } else {
                    tracing::warn!("Failed to read module config from {:?}", config_path);
                    waybar_content.clone()
                }
            } else {
                tracing::debug!("No config.jsonc found at {:?}", config_path);
                waybar_content.clone()
            };
            waybar_config::add_module(&with_module_config, &waybar_module_name, section)
        } else {
            let without_config = waybar_config::remove_module_config(&waybar_content, &waybar_module_name)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to remove module config: {e}");
                    waybar_content.clone()
                });
            waybar_config::remove_module(&without_config, &waybar_module_name)
        };

        if let Ok(new_waybar_content) = modified {
            let _ = waybar_config::backup_config().await;

            if waybar_config::save_config(&new_waybar_content).await.is_ok() {
                let _ = waybar_config::reload_waybar().await;
            }
        }
    }

    if enabled {
        handle_css_injection(&uuid, &install_path).await;
    } else {
        handle_css_removal(&uuid).await;
    }

    tracing::info!("Module {} {}", uuid, if enabled { "enabled" } else { "disabled" });
    Ok(uuid)
}

async fn handle_css_injection(uuid: &str, install_path: &Path) {
    use crate::services::waybar_config;

    let css_path = install_path.join("style.css");
    if !css_path.exists() {
        return;
    }

    let Ok(module_css) = tokio::fs::read_to_string(&css_path).await else {
        return;
    };

    let waybar_style_path = paths::waybar_style_path();
    let existing_css = tokio::fs::read_to_string(&waybar_style_path)
        .await
        .unwrap_or_default();

    let new_css = waybar_config::inject_module_css(&existing_css, uuid, &module_css);

    if let Err(e) = tokio::fs::write(&waybar_style_path, new_css).await {
        tracing::warn!("Failed to inject CSS: {e}");
    }
}

async fn handle_css_removal(uuid: &str) {
    use crate::services::waybar_config;

    let waybar_style_path = paths::waybar_style_path();
    let Ok(existing_css) = tokio::fs::read_to_string(&waybar_style_path).await else {
        return;
    };

    let new_css = waybar_config::remove_module_css(&existing_css, uuid);

    if let Err(e) = tokio::fs::write(&waybar_style_path, new_css).await {
        tracing::warn!("Failed to remove CSS: {e}");
    }
}

async fn change_module_position_async(
    uuid: String,
    new_section: BarSection,
) -> Result<String, String> {
    use crate::services::waybar_config;

    let state_path = paths::data_dir().join("installed.json");

    let content = tokio::fs::read_to_string(&state_path)
        .await
        .map_err(|e| format!("Failed to read state: {e}"))?;

    let mut modules: Vec<InstalledModule> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse state: {e}"))?;

    let module = modules
        .iter_mut()
        .find(|m| m.uuid.to_string() == uuid)
        .ok_or_else(|| format!("Module not found: {uuid}"))?;

    let waybar_module_name = module.waybar_module_name.clone();
    let was_enabled = module.enabled;
    let old_section = module.position.as_ref().map(|p| p.section);

    module.position = Some(crate::domain::ModulePosition {
        section: new_section,
        order: None,
    });

    let new_content =
        serde_json::to_string_pretty(&modules).map_err(|e| format!("Failed to serialize: {e}"))?;

    tokio::fs::write(&state_path, new_content)
        .await
        .map_err(|e| format!("Failed to save state: {e}"))?;

    if was_enabled
        && let Ok(waybar_content) = waybar_config::load_config().await
    {
        let modified = waybar_config::remove_module(&waybar_content, &waybar_module_name)
            .and_then(|content| waybar_config::add_module(&content, &waybar_module_name, new_section));

        if let Ok(new_waybar_content) = modified {
            let _ = waybar_config::backup_config().await;

            if waybar_config::save_config(&new_waybar_content).await.is_ok() {
                let _ = waybar_config::reload_waybar().await;
            }
        }
    }

    tracing::info!(
        "Changed position of {} from {:?} to {:?}",
        uuid,
        old_section,
        new_section
    );
    Ok(uuid)
}

async fn uninstall_module_async(uuid: String) -> Result<String, String> {
    let state_path = paths::data_dir().join("installed.json");
    let install_path = paths::module_install_path(&uuid);

    let remove_dir = async {
        match tokio::fs::remove_dir_all(&install_path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("Failed to remove module files: {e}")),
        }
    };

    let read_state = tokio::fs::read_to_string(&state_path);

    let (remove_result, read_result) = tokio::join!(remove_dir, read_state);

    remove_result?;
    let content = read_result.map_err(|e| format!("Failed to read state: {e}"))?;

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
    use std::time::SystemTime;

    const CACHE_TTL_DAYS: u64 = 7;

    let cache_path = paths::screenshot_cache_path(&url);

    if let Ok(metadata) = tokio::fs::metadata(&cache_path).await
        && let Ok(modified) = metadata.modified()
    {
        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::from_secs(u64::MAX));

        if age < Duration::from_secs(CACHE_TTL_DAYS * 24 * 60 * 60) {
            tracing::debug!("Loading screenshot from cache: {:?}", cache_path);
            if let Ok(bytes) = tokio::fs::read(&cache_path).await {
                return Ok(image::Handle::from_bytes(bytes));
            }
        }
    }

    tracing::debug!("Fetching screenshot from {}", url);

    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch screenshot: {e}"))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read screenshot bytes: {e}"))?;

    if let Some(parent) = cache_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let _ = tokio::fs::write(&cache_path, &bytes).await;

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

pub async fn make_scripts_executable(install_path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut entries = tokio::fs::read_dir(install_path)
        .await
        .map_err(|e| format!("Failed to read directory: {e}"))?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "sh") {
            let metadata = tokio::fs::metadata(&path)
                .await
                .map_err(|e| format!("Failed to get metadata: {e}"))?;
            let mut perms = metadata.permissions();
            perms.set_mode(perms.mode() | 0o111);
            tokio::fs::set_permissions(&path, perms)
                .await
                .map_err(|e| format!("Failed to set permissions: {e}"))?;
            tracing::debug!("Made executable: {}", path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_make_scripts_executable() {
        let dir = tempdir().unwrap();
        let script_path = dir.path().join("test.sh");
        std::fs::write(&script_path, "#!/bin/bash\necho hello").unwrap();

        let meta_before = std::fs::metadata(&script_path).unwrap();
        assert_eq!(meta_before.permissions().mode() & 0o111, 0);

        make_scripts_executable(dir.path()).await.unwrap();

        let meta_after = std::fs::metadata(&script_path).unwrap();
        assert_ne!(meta_after.permissions().mode() & 0o111, 0);
    }

    #[tokio::test]
    async fn test_make_scripts_executable_ignores_non_sh() {
        let dir = tempdir().unwrap();
        let json_path = dir.path().join("config.json");
        std::fs::write(&json_path, "{}").unwrap();

        let meta_before = std::fs::metadata(&json_path).unwrap();
        let mode_before = meta_before.permissions().mode();

        make_scripts_executable(dir.path()).await.unwrap();

        let meta_after = std::fs::metadata(&json_path).unwrap();
        assert_eq!(meta_after.permissions().mode(), mode_before);
    }
}
