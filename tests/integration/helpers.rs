use std::path::PathBuf;
use tempfile::TempDir;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method, matchers::path};

pub struct TestContext {
    pub mock_server: MockServer,
    pub _temp_dir: TempDir,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl TestContext {
    pub async fn new() -> Self {
        let mock_server = MockServer::start().await;
        let _temp_dir = TempDir::new().expect("failed to create temp dir");

        let data_dir = _temp_dir.path().join("data");
        let cache_dir = _temp_dir.path().join("cache");
        let config_dir = _temp_dir.path().join("config");

        std::fs::create_dir_all(&data_dir).expect("failed to create data dir");
        std::fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
        std::fs::create_dir_all(&config_dir).expect("failed to create config dir");

        Self {
            mock_server,
            _temp_dir,
            data_dir,
            cache_dir,
            config_dir,
        }
    }

    pub fn registry_url(&self) -> String {
        format!("{}/api/v1/index", self.mock_server.uri())
    }
}

pub fn sample_registry_json() -> &'static str {
    include_str!("fixtures/sample_registry.json")
}

pub async fn mock_registry_success(ctx: &TestContext) {
    Mock::given(method("GET"))
        .and(path("/api/v1/index"))
        .respond_with(ResponseTemplate::new(200).set_body_string(sample_registry_json()))
        .mount(&ctx.mock_server)
        .await;
}

pub async fn mock_registry_failure(ctx: &TestContext, status: u16) {
    Mock::given(method("GET"))
        .and(path("/api/v1/index"))
        .respond_with(ResponseTemplate::new(status))
        .mount(&ctx.mock_server)
        .await;
}
