use iced::Task;

use crate::app::Message;
use crate::domain::RegistryIndex;
use crate::services::paths::{self, HTTP_CLIENT, REGISTRY_URL};

pub fn load_registry() -> Task<Message> {
    Task::perform(fetch_registry_async(), Message::RegistryLoaded)
}

pub fn refresh_registry() -> Task<Message> {
    Task::perform(refresh_registry_async(), Message::RegistryRefreshed)
}

async fn fetch_registry_async() -> Result<RegistryIndex, String> {
    let cache_path = paths::registry_cache_path();

    if let Ok(content) = tokio::fs::read_to_string(&cache_path).await
        && let Ok(index) = serde_json::from_str::<RegistryIndex>(&content)
    {
        tracing::info!("Loaded registry from cache ({} modules)", index.modules.len());
        return Ok(index);
    }

    tracing::info!("Fetching registry");
    let response = HTTP_CLIENT
        .get(REGISTRY_URL)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let index: RegistryIndex = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse registry: {e}"))?;

    if let Some(parent) = cache_path.parent()
        && let Err(e) = tokio::fs::create_dir_all(parent).await
    {
        tracing::warn!("Failed to create cache directory: {e}");
    }
    if let Ok(content) = serde_json::to_string_pretty(&index)
        && let Err(e) = tokio::fs::write(&cache_path, content).await
    {
        tracing::warn!("Failed to write registry cache: {e}");
    }

    tracing::info!("Fetched {} modules from registry", index.modules.len());
    Ok(index)
}

async fn refresh_registry_async() -> Result<RegistryIndex, String> {
    let cache_path = paths::registry_cache_path();
    if let Err(e) = tokio::fs::remove_file(&cache_path).await {
        tracing::debug!("Cache file removal skipped: {e}");
    }

    tracing::info!("Force refreshing registry");
    let response = HTTP_CLIENT
        .get(REGISTRY_URL)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let index: RegistryIndex = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {e}"))?;

    if let Some(parent) = cache_path.parent()
        && let Err(e) = tokio::fs::create_dir_all(parent).await
    {
        tracing::warn!("Failed to create cache directory: {e}");
    }
    if let Ok(content) = serde_json::to_string_pretty(&index)
        && let Err(e) = tokio::fs::write(&cache_path, content).await
    {
        tracing::warn!("Failed to write registry cache: {e}");
    }

    tracing::info!("Refreshed registry: {} modules", index.modules.len());
    Ok(index)
}

