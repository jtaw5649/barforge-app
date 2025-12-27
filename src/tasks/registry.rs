use iced::Task;

use barforge_registry_client::apis::Error as ApiError;
use barforge_registry_client::apis::configuration::Configuration;
use barforge_registry_client::apis::default_api;
use reqwest::StatusCode;

use crate::api::{
    map_author_profile, map_registry_index, map_reviews_response, registry_configuration,
};
use crate::app::Message;
use crate::domain::{AuthorProfile, ModuleUuid, RegistryIndex, ReviewsResponse};
use crate::services::paths;

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
    let config = registry_configuration();
    let api_index = default_api::api_v1_index_get(&config)
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let index = map_registry_index(api_index).map_err(|e| format!("Invalid registry data: {e}"))?;

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
    let config = registry_configuration();
    let api_index = default_api::api_v1_index_get(&config)
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let index = map_registry_index(api_index).map_err(|e| format!("Invalid registry data: {e}"))?;

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
    let config = registry_configuration();
    fetch_author_profile_with_config(&config, &username).await
}

async fn fetch_author_profile_with_config(
    config: &Configuration,
    username: &str,
) -> Result<AuthorProfile, String> {
    tracing::info!("Fetching author profile for {username}");
    let profile = match default_api::api_v1_users_username_get(config, username).await {
        Ok(profile) => profile,
        Err(ApiError::ResponseError(response)) if response.status == StatusCode::NOT_FOUND => {
            return Err(format!("User not found: {username}"));
        }
        Err(error) => return Err(format!("Network error: {error}")),
    };
    let modules = match default_api::api_v1_users_username_modules_get(config, username).await {
        Ok(response) => response.modules,
        Err(error) => {
            tracing::warn!("Failed to fetch author modules for {username}: {error}");
            Vec::new()
        }
    };
    let profile = map_author_profile(profile, modules)
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
    tracing::info!("Fetching reviews for module {}", uuid);
    let config = registry_configuration();
    let api_reviews = match default_api::api_v1_modules_uuid_reviews_get(&config, &uuid_str).await {
        Ok(reviews) => reviews,
        Err(ApiError::ResponseError(_)) => return Ok(ReviewsResponse::default()),
        Err(error) => return Err(format!("Network error: {error}")),
    };
    let reviews =
        map_reviews_response(api_reviews).map_err(|e| format!("Failed to parse reviews: {e}"))?;

    tracing::info!("Loaded {} reviews for module {}", reviews.total, uuid);
    Ok(reviews)
}

#[cfg(test)]
mod tests {
    use super::*;
    use barforge_registry_client::apis::configuration::Configuration;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::services::paths::HTTP_CLIENT;

    #[tokio::test]
    async fn fetch_author_profile_falls_back_to_empty_modules_on_failure() {
        let mock_server = MockServer::start().await;

        let profile_body = serde_json::json!({
            "version": 1,
            "id": 42,
            "username": "jane",
            "display_name": null,
            "avatar_url": null,
            "bio": null,
            "website_url": null,
            "github_url": null,
            "twitter_url": null,
            "bluesky_url": null,
            "discord_url": null,
            "sponsor_url": null,
            "verified_author": true,
            "role": "user",
            "module_count": 0,
            "created_at": "2025-01-01T00:00:00Z"
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/users/jane"))
            .respond_with(ResponseTemplate::new(200).set_body_json(profile_body))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/users/jane/modules"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = Configuration {
            base_path: mock_server.uri(),
            client: (*HTTP_CLIENT).clone(),
            ..Default::default()
        };

        let profile = fetch_author_profile_with_config(&config, "jane")
            .await
            .expect("profile should load without modules");

        assert_eq!(profile.author.username, "jane");
        assert!(profile.modules.is_empty());
    }
}
