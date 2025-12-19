use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Review {
    pub id: u64,
    pub rating: u8,
    pub title: Option<String>,
    pub body: Option<String>,
    pub helpful_count: u64,
    pub user: ReviewUser,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ReviewUser {
    pub username: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ReviewsResponse {
    pub reviews: Vec<Review>,
    pub total: usize,
}

impl Review {
    pub fn star_display(&self) -> String {
        let filled = self.rating as usize;
        let empty = 5 - filled;
        format!("{}{}", "★".repeat(filled), "☆".repeat(empty))
    }

    pub fn relative_time(&self) -> String {
        chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(dt);

                if duration.num_days() > 365 {
                    format!("{} years ago", duration.num_days() / 365)
                } else if duration.num_days() > 30 {
                    format!("{} months ago", duration.num_days() / 30)
                } else if duration.num_days() > 0 {
                    format!("{} days ago", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{} hours ago", duration.num_hours())
                } else {
                    "just now".to_string()
                }
            })
            .unwrap_or_else(|_| self.created_at.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_review() -> Review {
        Review {
            id: 1,
            rating: 4,
            title: Some("Great module!".to_string()),
            body: Some("Works well".to_string()),
            helpful_count: 5,
            user: ReviewUser {
                username: "testuser".to_string(),
                avatar_url: Some("https://example.com/avatar.png".to_string()),
            },
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: None,
        }
    }

    #[test]
    fn star_display_full_stars() {
        let mut review = create_test_review();
        review.rating = 5;
        assert_eq!(review.star_display(), "★★★★★");
    }

    #[test]
    fn star_display_partial_stars() {
        let review = create_test_review();
        assert_eq!(review.star_display(), "★★★★☆");
    }

    #[test]
    fn star_display_no_stars() {
        let mut review = create_test_review();
        review.rating = 0;
        assert_eq!(review.star_display(), "☆☆☆☆☆");
    }

    #[test]
    fn deserialize_review() {
        let json = r#"{
            "id": 1,
            "rating": 5,
            "title": "Excellent",
            "body": "Works perfectly",
            "helpful_count": 10,
            "user": {
                "username": "reviewer",
                "avatar_url": null
            },
            "created_at": "2025-01-01T00:00:00Z"
        }"#;
        let review: Review = serde_json::from_str(json).unwrap();
        assert_eq!(review.rating, 5);
        assert_eq!(review.user.username, "reviewer");
    }

    #[test]
    fn deserialize_reviews_response() {
        let json = r#"{
            "reviews": [],
            "total": 0
        }"#;
        let response: ReviewsResponse = serde_json::from_str(json).unwrap();
        assert!(response.reviews.is_empty());
        assert_eq!(response.total, 0);
    }
}
