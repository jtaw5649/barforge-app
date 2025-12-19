mod module;
mod registry;
mod watchers;
mod waybar;

pub use module::{
    change_module_position, install_module, make_scripts_executable, toggle_module,
    uninstall_module, update_all_modules, update_module,
};
pub use registry::{load_author_profile, load_module_reviews, load_registry, refresh_registry};
pub use watchers::watch_omarchy_theme;

use std::time::Duration;

use iced::Task;
use iced::widget::image;

use crate::app::Message;
use crate::domain::InstalledModule;
use crate::services::paths::{self, HTTP_CLIENT};

pub fn initial_load() -> Task<Message> {
    Task::batch([load_installed(), load_registry()])
}

pub fn load_installed() -> Task<Message> {
    Task::perform(load_installed_async(), Message::InstalledLoaded)
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

    let modules: Vec<InstalledModule> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse installed modules: {e}"))?;

    tracing::info!("Loaded {} installed modules", modules.len());
    Ok(modules)
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

    if let Some(parent) = cache_path.parent()
        && let Err(e) = tokio::fs::create_dir_all(parent).await
    {
        tracing::debug!("Failed to create screenshot cache directory: {e}");
    }
    if let Err(e) = tokio::fs::write(&cache_path, &bytes).await {
        tracing::debug!("Failed to cache screenshot: {e}");
    }

    Ok(image::Handle::from_bytes(bytes.to_vec()))
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
