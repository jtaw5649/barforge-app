use crate::helpers::{TestContext, mock_registry_failure, mock_registry_success};
use barforge_registry_types::{ModuleCategory, RegistryIndex};

#[tokio::test]
async fn test_mock_server_serves_registry() {
    let ctx = TestContext::new().await;
    mock_registry_success(&ctx).await;

    let response = reqwest::get(&ctx.registry_url())
        .await
        .expect("request failed");

    assert_eq!(response.status(), 200);

    let registry: RegistryIndex = response.json().await.expect("failed to parse json");

    assert_eq!(registry.modules.len(), 2);
    assert_eq!(registry.modules[0].name, "CPU Monitor");
    assert_eq!(registry.modules[1].name, "Memory Monitor");
}

#[tokio::test]
async fn test_temp_directories_created() {
    let ctx = TestContext::new().await;

    assert!(ctx.data_dir.exists());
    assert!(ctx.cache_dir.exists());
    assert!(ctx.config_dir.exists());
}

#[tokio::test]
async fn test_registry_module_has_correct_category() {
    let ctx = TestContext::new().await;
    mock_registry_success(&ctx).await;

    let response = reqwest::get(&ctx.registry_url())
        .await
        .expect("request failed");
    let registry: RegistryIndex = response.json().await.expect("failed to parse json");

    assert_eq!(registry.modules[0].category, ModuleCategory::System);
    assert_eq!(registry.modules[1].category, ModuleCategory::System);
}

#[tokio::test]
async fn test_registry_module_has_tags() {
    let ctx = TestContext::new().await;
    mock_registry_success(&ctx).await;

    let response = reqwest::get(&ctx.registry_url())
        .await
        .expect("request failed");
    let registry: RegistryIndex = response.json().await.expect("failed to parse json");

    assert!(registry.modules[0].tags.contains(&"cpu".to_string()));
    assert!(registry.modules[1].tags.contains(&"memory".to_string()));
}

#[tokio::test]
async fn test_registry_search_finds_by_name() {
    let ctx = TestContext::new().await;
    mock_registry_success(&ctx).await;

    let response = reqwest::get(&ctx.registry_url())
        .await
        .expect("request failed");
    let registry: RegistryIndex = response.json().await.expect("failed to parse json");

    let results = registry.search("CPU");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "CPU Monitor");
}

#[tokio::test]
async fn test_registry_search_finds_by_tag() {
    let ctx = TestContext::new().await;
    mock_registry_success(&ctx).await;

    let response = reqwest::get(&ctx.registry_url())
        .await
        .expect("request failed");
    let registry: RegistryIndex = response.json().await.expect("failed to parse json");

    let results = registry.search("ram");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Memory Monitor");
}

#[tokio::test]
async fn test_registry_by_category_filters_correctly() {
    let ctx = TestContext::new().await;
    mock_registry_success(&ctx).await;

    let response = reqwest::get(&ctx.registry_url())
        .await
        .expect("request failed");
    let registry: RegistryIndex = response.json().await.expect("failed to parse json");

    let system_modules = registry.by_category(ModuleCategory::System);

    assert_eq!(system_modules.len(), 2);
}

#[tokio::test]
async fn test_mock_server_can_return_error() {
    let ctx = TestContext::new().await;
    mock_registry_failure(&ctx, 500).await;

    let response = reqwest::get(&ctx.registry_url())
        .await
        .expect("request failed");

    assert_eq!(response.status(), 500);
}

#[tokio::test]
async fn test_registry_find_by_uuid() {
    let ctx = TestContext::new().await;
    mock_registry_success(&ctx).await;

    let response = reqwest::get(&ctx.registry_url())
        .await
        .expect("request failed");
    let registry: RegistryIndex = response.json().await.expect("failed to parse json");

    let found = registry.find_by_uuid("cpu-monitor@testauthor");

    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "CPU Monitor");
}

#[tokio::test]
async fn test_registry_find_by_uuid_not_found() {
    let ctx = TestContext::new().await;
    mock_registry_success(&ctx).await;

    let response = reqwest::get(&ctx.registry_url())
        .await
        .expect("request failed");
    let registry: RegistryIndex = response.json().await.expect("failed to parse json");

    let found = registry.find_by_uuid("nonexistent@author");

    assert!(found.is_none());
}
