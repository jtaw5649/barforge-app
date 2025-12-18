use std::io::Cursor;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use iced::Task;
use once_cell::sync::Lazy;
use tar::Archive;

use crate::app::Message;
use crate::domain::{BarSection, InstalledModule, ModuleVersion};
use crate::security::{parse_github_url_safe, validate_extraction_path};
use crate::services::paths::{self, HTTP_CLIENT};
use crate::services::{InstallParams, SecureInstaller};

use super::waybar::{handle_css_injection, handle_css_removal};

static DEFAULT_VERSION: Lazy<ModuleVersion> = Lazy::new(|| {
    ModuleVersion::try_from("1.0.0").unwrap_or_else(|_| {
        unreachable!("1.0.0 is always valid semver")
    })
});

pub fn toggle_module(uuid: String, enabled: bool) -> Task<Message> {
    Task::perform(
        toggle_module_async(uuid, enabled),
        Message::ToggleCompleted,
    )
}

pub fn uninstall_module(uuid: String) -> Task<Message> {
    Task::perform(uninstall_module_async(uuid), Message::UninstallCompleted)
}

pub fn change_module_position(uuid: String, section: BarSection) -> Task<Message> {
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
    checksum: Option<String>,
) -> Task<Message> {
    Task::perform(
        install_module_async(uuid, name, version, repo_url, checksum),
        Message::InstallCompleted,
    )
}

async fn install_module_async(
    uuid: String,
    name: String,
    version: Option<ModuleVersion>,
    repo_url: String,
    checksum: Option<String>,
) -> Result<InstalledModule, String> {
    let install_path = paths::module_install_path(&uuid);
    let version = version.unwrap_or_else(|| DEFAULT_VERSION.clone());

    if let Some(expected_hash) = checksum {
        install_secure(&uuid, &version.to_string(), &expected_hash, &install_path).await?;
    } else {
        tokio::fs::create_dir_all(&install_path)
            .await
            .map_err(|e| format!("Failed to create install directory: {e}"))?;

        download_module_files(&repo_url, &install_path).await?;
        make_scripts_executable(&install_path).await?;
    }

    let has_preferences = install_path.join("preferences.schema.json").exists();
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

async fn install_secure(
    uuid: &str,
    version: &str,
    expected_hash: &str,
    dest_dir: &Path,
) -> Result<(), String> {
    let package_url = paths::package_url(uuid, version);
    let signature_url = paths::signature_url(uuid, version);

    tracing::info!("Fetching signature from {}", signature_url);
    let signature = HTTP_CLIENT
        .get(&signature_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch signature: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Failed to read signature: {e}"))?;

    tracing::info!("Downloading package from {}", package_url);
    let package_data = HTTP_CLIENT
        .get(&package_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download package: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read package: {e}"))?;

    let installer = SecureInstaller::new();
    let params = InstallParams {
        uuid,
        version,
        package_data: &package_data,
        signature: &signature,
        expected_hash,
        dest_dir,
    };
    installer
        .install(params, |stage| {
            tracing::debug!("Install stage: {}", stage.description());
        })
        .await
        .map_err(|e| format!("Secure installation failed: {e}"))?;

    Ok(())
}

async fn update_module_async(
    uuid: String,
    repo_url: String,
    new_version: ModuleVersion,
) -> Result<InstalledModule, String> {
    use crate::services::waybar_config;

    let install_path = paths::module_install_path(&uuid);
    let state_path = paths::data_dir().join("installed.json");

    let content = tokio::fs::read_to_string(&state_path)
        .await
        .map_err(|e| format!("Failed to read state: {e}"))?;

    let mut modules: Vec<InstalledModule> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse state: {e}"))?;

    let module = modules
        .iter()
        .find(|m| m.uuid.to_string() == uuid)
        .ok_or_else(|| format!("Module not found: {uuid}"))?;

    let was_enabled = module.enabled;
    let waybar_module_name = module.waybar_module_name.clone();
    let section = module
        .position
        .as_ref()
        .map(|p| p.section)
        .unwrap_or(BarSection::Center);

    if was_enabled {
        handle_css_removal(&uuid).await;

        if let Ok(waybar_content) = waybar_config::load_config().await {
            let without_config =
                waybar_config::remove_module_config(&waybar_content, &waybar_module_name)
                    .unwrap_or_else(|e| {
                        tracing::warn!("Failed to remove module config during update: {e}");
                        waybar_content.clone()
                    });

            if let Ok(new_waybar_content) =
                waybar_config::remove_module(&without_config, &waybar_module_name)
            {
                let _ = waybar_config::save_config(&new_waybar_content).await;
            }
        }
    }

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

    if was_enabled {
        if let Ok(waybar_content) = waybar_config::load_config().await {
            let config_path = install_path.join("config.jsonc");
            let with_module_config = if config_path.exists() {
                if let Ok(module_config) = tokio::fs::read_to_string(&config_path).await {
                    let prefs = crate::services::preferences::load_preferences(&uuid);
                    let module_config = waybar_config::substitute_preferences(&module_config, &prefs);
                    let install_path_str = install_path.to_string_lossy();
                    waybar_config::merge_module_config(&waybar_content, &module_config, &install_path_str)
                        .unwrap_or_else(|e| {
                            tracing::warn!("Failed to merge module config during update: {e}");
                            waybar_content.clone()
                        })
                } else {
                    waybar_content.clone()
                }
            } else {
                waybar_content.clone()
            };

            if let Ok(with_module) =
                waybar_config::add_module(&with_module_config, &waybar_module_name, section)
            {
                if let Err(e) = waybar_config::backup_config().await {
                    tracing::warn!("Failed to backup waybar config: {e}");
                }

                if waybar_config::save_config(&with_module).await.is_ok()
                    && let Err(e) = waybar_config::reload_waybar().await
                {
                    tracing::warn!("Failed to reload waybar: {e}");
                }
            }
        }

        handle_css_injection(&uuid, &install_path).await;
    }

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
    parse_github_url_safe(url).map_err(|e| e.to_string())
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
            if let Err(e) = waybar_config::backup_config().await {
                tracing::warn!("Failed to backup waybar config: {e}");
            }

            if waybar_config::save_config(&new_waybar_content).await.is_ok()
                && let Err(e) = waybar_config::reload_waybar().await
            {
                tracing::warn!("Failed to reload waybar: {e}");
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
            if let Err(e) = waybar_config::backup_config().await {
                tracing::warn!("Failed to backup waybar config: {e}");
            }

            if waybar_config::save_config(&new_waybar_content).await.is_ok()
                && let Err(e) = waybar_config::reload_waybar().await
            {
                tracing::warn!("Failed to reload waybar: {e}");
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

async fn uninstall_module_async(uuid: String) -> Result<String, (String, String)> {
    use crate::services::waybar_config;

    let state_path = paths::data_dir().join("installed.json");
    let install_path = paths::module_install_path(&uuid);

    let content = tokio::fs::read_to_string(&state_path)
        .await
        .map_err(|e| (uuid.clone(), format!("Failed to read state: {e}")))?;

    let mut modules: Vec<InstalledModule> =
        serde_json::from_str(&content).map_err(|e| (uuid.clone(), format!("Failed to parse state: {e}")))?;

    let module = modules
        .iter()
        .find(|m| m.uuid.to_string() == uuid)
        .ok_or_else(|| (uuid.clone(), format!("Module not found: {uuid}")))?;

    let was_enabled = module.enabled;
    let waybar_module_name = module.waybar_module_name.clone();

    if was_enabled {
        if let Ok(waybar_content) = waybar_config::load_config().await {
            let without_config =
                waybar_config::remove_module_config(&waybar_content, &waybar_module_name)
                    .unwrap_or_else(|e| {
                        tracing::warn!("Failed to remove module config: {e}");
                        waybar_content.clone()
                    });

            let without_module =
                waybar_config::remove_module(&without_config, &waybar_module_name);

            if let Ok(new_waybar_content) = without_module {
                if let Err(e) = waybar_config::backup_config().await {
                    tracing::warn!("Failed to backup waybar config: {e}");
                }

                if waybar_config::save_config(&new_waybar_content).await.is_ok()
                    && let Err(e) = waybar_config::reload_waybar().await
                {
                    tracing::warn!("Failed to reload waybar: {e}");
                }
            }
        }

        handle_css_removal(&uuid).await;
    }

    match tokio::fs::remove_dir_all(&install_path).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err((uuid, format!("Failed to remove module files: {e}"))),
    }

    modules.retain(|m| m.uuid.to_string() != uuid);

    let new_content =
        serde_json::to_string_pretty(&modules).map_err(|e| (uuid.clone(), format!("Failed to serialize: {e}")))?;

    tokio::fs::write(&state_path, new_content)
        .await
        .map_err(|e| (uuid.clone(), format!("Failed to save state: {e}")))?;

    tracing::info!("Uninstalled module {}", uuid);
    Ok(uuid)
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
