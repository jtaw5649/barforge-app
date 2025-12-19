use iced::Task;

use crate::app::Message;
use crate::domain::{AuthorProfile, ModuleUuid, RegistryIndex, ReviewsResponse};
use crate::services::paths::{self, API_BASE_URL, HTTP_CLIENT, REGISTRY_URL};

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
        tracing::info!(
            "Loaded registry from cache ({} modules)",
            index.modules.len()
        );
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

pub fn load_author_profile(username: String) -> Task<Message> {
    Task::perform(fetch_author_profile_async(username), Message::AuthorLoaded)
}

async fn fetch_author_profile_async(username: String) -> Result<AuthorProfile, String> {
    let encoded = urlencoding::encode(&username);
    let url = format!("{API_BASE_URL}/api/v1/users/{encoded}");

    tracing::info!("Fetching author profile for {username}");
    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("User not found: {username}"));
    }

    let profile: AuthorProfile = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse author profile: {e}"))?;

    tracing::info!(
        "Loaded author profile: {} ({} modules)",
        profile.author.username,
        profile.modules.len()
    );
    Ok(profile)
}

pub fn load_module_reviews(uuid: ModuleUuid) -> Task<Message> {
    Task::perform(fetch_module_reviews_async(uuid.clone()), move |result| {
        Message::ModuleReviewsLoaded(result.map(|r| (uuid.clone(), r)))
    })
}

async fn fetch_module_reviews_async(uuid: ModuleUuid) -> Result<ReviewsResponse, String> {
    let uuid_str = uuid.to_string();
    let encoded = urlencoding::encode(&uuid_str);
    let url = format!("{API_BASE_URL}/api/v1/modules/{encoded}/reviews");

    tracing::info!("Fetching reviews for module {}", uuid);
    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !response.status().is_success() {
        return Ok(ReviewsResponse::default());
    }

    let reviews: ReviewsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse reviews: {e}"))?;

    tracing::info!("Loaded {} reviews for module {}", reviews.total, uuid);
    Ok(reviews)
}
