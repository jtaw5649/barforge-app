use serde::{Deserialize, Serialize};

use crate::RegistryModule;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Author {
    pub id: u64,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub website_url: Option<String>,
    pub verified_author: bool,
    pub module_count: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AuthorProfile {
    #[serde(flatten)]
    pub author: Author,
    #[serde(default)]
    pub modules: Vec<RegistryModule>,
}

impl Author {
    pub fn display(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.username)
    }

    pub fn member_since(&self) -> String {
        chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.format("%B %Y").to_string())
            .unwrap_or_else(|_| self.created_at.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_author() -> Author {
        Author {
            id: 1,
            username: "testuser".to_string(),
            display_name: Some("Test User".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            bio: Some("A test author".to_string()),
            website_url: Some("https://example.com".to_string()),
            verified_author: true,
            module_count: 5,
            created_at: "2025-01-15T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn display_uses_display_name_when_present() {
        let author = create_test_author();
        assert_eq!(author.display(), "Test User");
    }

    #[test]
    fn display_falls_back_to_username() {
        let mut author = create_test_author();
        author.display_name = None;
        assert_eq!(author.display(), "testuser");
    }

    #[test]
    fn member_since_formats_date() {
        let author = create_test_author();
        assert_eq!(author.member_since(), "January 2025");
    }

    #[test]
    fn deserialize_author() {
        let json = r#"{
            "id": 1,
            "username": "developer",
            "display_name": "Developer Name",
            "avatar_url": "https://example.com/avatar.png",
            "bio": "I make modules",
            "website_url": "https://dev.example.com",
            "verified_author": true,
            "module_count": 10,
            "created_at": "2024-06-01T00:00:00Z"
        }"#;
        let author: Author = serde_json::from_str(json).unwrap();
        assert_eq!(author.username, "developer");
        assert!(author.verified_author);
        assert_eq!(author.module_count, 10);
    }

    #[test]
    fn deserialize_author_minimal() {
        let json = r#"{
            "id": 1,
            "username": "minimal",
            "display_name": null,
            "avatar_url": null,
            "bio": null,
            "website_url": null,
            "verified_author": false,
            "module_count": 0,
            "created_at": "2025-01-01T00:00:00Z"
        }"#;
        let author: Author = serde_json::from_str(json).unwrap();
        assert_eq!(author.username, "minimal");
        assert!(!author.verified_author);
    }
}
