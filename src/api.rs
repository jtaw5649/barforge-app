use barforge_registry_client::apis::configuration::Configuration;
use barforge_registry_client::models as api_models;

use crate::domain::{
    Author, AuthorProfile, CategoryInfo, ModuleCategory, ModuleUuid, ModuleVersion, RegistryIndex,
    RegistryModule, Review, ReviewUser, ReviewsResponse,
};
use crate::services::paths::{API_BASE_URL, HTTP_CLIENT};

pub fn registry_configuration() -> Configuration {
    Configuration {
        base_path: API_BASE_URL.to_string(),
        client: (*HTTP_CLIENT).clone(),
        ..Default::default()
    }
}

pub fn map_registry_index(api: api_models::RegistryIndex) -> Result<RegistryIndex, String> {
    let modules = api
        .modules
        .into_iter()
        .map(map_registry_module)
        .collect::<Result<Vec<_>, _>>()?;
    let categories = api
        .categories
        .into_iter()
        .map(|(key, value)| (key, map_category_info(value)))
        .collect();
    let version = parse_u32(api.version, "registry version")?;

    Ok(RegistryIndex {
        version,
        modules,
        categories,
    })
}

pub fn map_registry_module(api: api_models::RegistryModule) -> Result<RegistryModule, String> {
    let api_models::RegistryModule {
        uuid,
        name,
        description,
        author,
        category,
        icon,
        screenshot,
        repo_url,
        downloads,
        version,
        last_updated,
        rating,
        verified_author,
        tags,
        checksum,
        license,
    } = api;
    let uuid = parse_module_uuid(&uuid)?;
    let version = version.flatten();
    let version = version.as_deref().map(parse_module_version).transpose()?;
    let last_updated = parse_optional_timestamp(last_updated.flatten())?;
    let downloads = parse_u64(downloads, "downloads")?;
    let category = map_module_category(category);

    Ok(RegistryModule {
        uuid,
        name,
        description,
        author,
        category,
        icon: icon.flatten(),
        screenshot: screenshot.flatten(),
        repo_url,
        downloads,
        version,
        last_updated,
        rating: rating.flatten(),
        verified_author,
        tags,
        checksum: checksum.flatten(),
        license: license.flatten(),
    })
}

pub fn map_reviews_response(
    api: api_models::ApiV1ModulesUuidReviewsGet200Response,
) -> Result<ReviewsResponse, String> {
    let reviews = api
        .reviews
        .into_iter()
        .map(map_review)
        .collect::<Result<Vec<_>, _>>()?;
    let total = parse_usize(api.total, "reviews total")?;

    Ok(ReviewsResponse { reviews, total })
}

pub fn map_author_profile(
    profile: api_models::ApiV1UsersMeGet200Response,
    modules: Vec<api_models::RegistryModule>,
) -> Result<AuthorProfile, String> {
    let author = Author {
        id: parse_u64(profile.id, "author id")?,
        username: profile.username,
        display_name: profile.display_name,
        avatar_url: profile.avatar_url,
        bio: profile.bio,
        website_url: profile.website_url,
        verified_author: profile.verified_author,
        module_count: parse_u64(profile.module_count, "module count")?,
        created_at: profile.created_at,
    };
    let modules = modules
        .into_iter()
        .map(map_registry_module)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AuthorProfile { author, modules })
}

fn map_category_info(api: api_models::CategoryInfo) -> CategoryInfo {
    CategoryInfo {
        id: api.id.flatten(),
        name: api.name,
        icon: api.icon,
    }
}

fn map_module_category(category: api_models::ModuleCategory) -> ModuleCategory {
    match category {
        api_models::ModuleCategory::System => ModuleCategory::System,
        api_models::ModuleCategory::Hardware => ModuleCategory::Hardware,
        api_models::ModuleCategory::Network => ModuleCategory::Network,
        api_models::ModuleCategory::Audio => ModuleCategory::Audio,
        api_models::ModuleCategory::Power => ModuleCategory::Power,
        api_models::ModuleCategory::Time => ModuleCategory::Time,
        api_models::ModuleCategory::Workspace => ModuleCategory::Workspace,
        api_models::ModuleCategory::Window => ModuleCategory::Window,
        api_models::ModuleCategory::Tray => ModuleCategory::Tray,
        api_models::ModuleCategory::Weather => ModuleCategory::Weather,
        api_models::ModuleCategory::Productivity => ModuleCategory::Productivity,
        api_models::ModuleCategory::Media => ModuleCategory::Media,
        api_models::ModuleCategory::Custom => ModuleCategory::Custom,
    }
}

fn map_review(api: api_models::Review) -> Result<Review, String> {
    let rating = u8::try_from(api.rating)
        .map_err(|_| format!("review rating must be between 0 and {}", u8::MAX))?;

    Ok(Review {
        id: parse_u64(api.id, "review id")?,
        rating,
        title: api.title.flatten(),
        body: api.body.flatten(),
        helpful_count: parse_u64(api.helpful_count, "helpful count")?,
        user: map_review_user(*api.user),
        created_at: api.created_at,
        updated_at: api.updated_at.flatten(),
    })
}

fn map_review_user(api: api_models::ReviewUser) -> ReviewUser {
    ReviewUser {
        username: api.username,
        avatar_url: api.avatar_url.flatten(),
    }
}

fn parse_module_uuid(value: &str) -> Result<ModuleUuid, String> {
    ModuleUuid::try_from(value).map_err(|err| format!("invalid module uuid: {err}"))
}

fn parse_module_version(value: &str) -> Result<ModuleVersion, String> {
    ModuleVersion::try_from(value).map_err(|err| format!("invalid module version: {err}"))
}

fn parse_optional_timestamp(
    value: Option<String>,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
    let Some(value) = value else {
        return Ok(None);
    };

    let parsed = chrono::DateTime::parse_from_rfc3339(&value)
        .map_err(|err| format!("invalid last_updated timestamp: {err}"))?;

    Ok(Some(parsed.with_timezone(&chrono::Utc)))
}

fn parse_u64(value: i64, field: &str) -> Result<u64, String> {
    if value < 0 {
        return Err(format!("{field} must be non-negative"));
    }
    Ok(value as u64)
}

fn parse_u32(value: i32, field: &str) -> Result<u32, String> {
    if value < 0 {
        return Err(format!("{field} must be non-negative"));
    }
    Ok(value as u32)
}

fn parse_usize(value: i32, field: &str) -> Result<usize, String> {
    if value < 0 {
        return Err(format!("{field} must be non-negative"));
    }
    Ok(value as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api_module_category() -> api_models::ModuleCategory {
        api_models::ModuleCategory::Weather
    }

    fn api_registry_module() -> api_models::RegistryModule {
        api_models::RegistryModule {
            uuid: "weather@barforge".to_string(),
            name: "Weather".to_string(),
            description: "Weather module".to_string(),
            author: "barforge".to_string(),
            category: api_module_category(),
            icon: Some(Some("weather-icon".to_string())),
            screenshot: None,
            repo_url: "https://github.com/barforge/weather".to_string(),
            downloads: 42,
            version: Some(Some("1.2.3".to_string())),
            last_updated: Some(Some("2025-01-01T00:00:00Z".to_string())),
            rating: Some(Some(4.5)),
            verified_author: true,
            tags: vec!["weather".to_string(), "forecast".to_string()],
            checksum: Some(Some("abc123".to_string())),
            license: Some(Some("MIT".to_string())),
        }
    }

    #[test]
    fn map_registry_module_maps_fields() {
        let module = map_registry_module(api_registry_module()).expect("valid module mapping");

        assert_eq!(module.uuid.to_string(), "weather@barforge");
        assert_eq!(module.name, "Weather");
        assert_eq!(module.description, "Weather module");
        assert_eq!(module.author, "barforge");
        assert_eq!(module.category, ModuleCategory::Weather);
        assert_eq!(module.downloads, 42);
        assert_eq!(module.version.expect("version").to_string(), "1.2.3");
        assert!(module.last_updated.is_some());
        assert_eq!(module.tags, vec!["weather", "forecast"]);
        assert_eq!(module.checksum.as_deref(), Some("abc123"));
        assert_eq!(module.license.as_deref(), Some("MIT"));
        assert!(module.verified_author);
    }

    #[test]
    fn map_registry_module_rejects_invalid_uuid() {
        let mut module = api_registry_module();
        module.uuid = "invalid-uuid".to_string();

        assert!(map_registry_module(module).is_err());
    }

    #[test]
    fn map_registry_module_rejects_invalid_version() {
        let mut module = api_registry_module();
        module.version = Some(Some("not-a-version".to_string()));

        assert!(map_registry_module(module).is_err());
    }

    #[test]
    fn map_registry_module_rejects_negative_downloads() {
        let mut module = api_registry_module();
        module.downloads = -5;

        assert!(map_registry_module(module).is_err());
    }

    #[test]
    fn map_registry_index_maps_categories() {
        let mut categories = std::collections::HashMap::new();
        categories.insert(
            "weather".to_string(),
            api_models::CategoryInfo {
                id: Some(Some("weather".to_string())),
                name: "Weather".to_string(),
                icon: "weather-icon".to_string(),
            },
        );

        let api_index = api_models::RegistryIndex {
            version: 1,
            modules: vec![api_registry_module()],
            categories,
        };

        let index = map_registry_index(api_index).expect("valid registry index");
        let category = index.categories.get("weather").expect("category entry");

        assert_eq!(index.version, 1);
        assert_eq!(index.modules.len(), 1);
        assert_eq!(category.id.as_deref(), Some("weather"));
        assert_eq!(category.name, "Weather");
    }

    #[test]
    fn map_reviews_response_maps_review_fields() {
        let api_review = api_models::Review {
            id: 10,
            rating: 5,
            title: Some(Some("Great".to_string())),
            body: Some(Some("Works well".to_string())),
            helpful_count: 2,
            user: Box::new(api_models::ReviewUser {
                username: "reviewer".to_string(),
                avatar_url: None,
            }),
            created_at: "2025-02-01T00:00:00Z".to_string(),
            updated_at: None,
        };

        let api_response = api_models::ApiV1ModulesUuidReviewsGet200Response {
            version: 1,
            reviews: vec![api_review],
            total: 1,
        };

        let response = map_reviews_response(api_response).expect("valid reviews response");

        assert_eq!(response.total, 1);
        assert_eq!(response.reviews.len(), 1);
        assert_eq!(response.reviews[0].rating, 5);
        assert_eq!(response.reviews[0].user.username, "reviewer");
    }

    #[test]
    fn map_author_profile_merges_modules() {
        let profile = api_models::ApiV1UsersMeGet200Response {
            version: 1,
            id: 7,
            username: "author".to_string(),
            display_name: Some("Author".to_string()),
            avatar_url: None,
            bio: Some("Bio".to_string()),
            website_url: Some("https://example.com".to_string()),
            github_url: None,
            twitter_url: None,
            bluesky_url: None,
            discord_url: None,
            sponsor_url: None,
            verified_author: true,
            role: api_models::UserRole::User,
            module_count: 2,
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let modules = vec![api_registry_module()];
        let author_profile = map_author_profile(profile, modules).expect("valid author profile");

        assert_eq!(author_profile.author.username, "author");
        assert_eq!(
            author_profile.author.display_name.as_deref(),
            Some("Author")
        );
        assert_eq!(author_profile.modules.len(), 1);
        assert_eq!(author_profile.modules[0].name, "Weather");
    }

    #[test]
    fn map_reviews_response_rejects_invalid_rating() {
        let api_review = api_models::Review {
            id: 10,
            rating: -1,
            title: None,
            body: None,
            helpful_count: 0,
            user: Box::new(api_models::ReviewUser {
                username: "reviewer".to_string(),
                avatar_url: None,
            }),
            created_at: "2025-02-01T00:00:00Z".to_string(),
            updated_at: None,
        };

        let api_response = api_models::ApiV1ModulesUuidReviewsGet200Response {
            version: 1,
            reviews: vec![api_review],
            total: 1,
        };

        assert!(map_reviews_response(api_response).is_err());
    }
}
