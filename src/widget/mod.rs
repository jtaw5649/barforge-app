mod category_style;
mod confirmation_dialog;
mod empty_state;
mod module_card;
mod module_detail_screen;
mod module_row;
mod module_table;
mod notification;
mod preferences_modal;
mod settings_screen;
mod sidebar;
mod skeleton_card;

pub use confirmation_dialog::confirmation_dialog;
pub use empty_state::{empty_state, empty_state_dynamic, empty_state_with_action};
pub use module_card::module_card;
pub use module_detail_screen::module_detail_screen;
pub use module_row::module_row;
pub use module_table::module_table;
pub use notification::notification_toast;
pub use preferences_modal::preferences_modal;
pub use settings_screen::settings_screen;
pub use sidebar::sidebar;
pub use skeleton_card::skeleton_card;

use chrono::{DateTime, Utc};

pub fn format_relative_time(date: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*date);

    if duration.num_days() > 365 {
        let years = duration.num_days() / 365;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    } else if duration.num_days() > 30 {
        let months = duration.num_days() / 30;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else if duration.num_days() > 0 {
        let days = duration.num_days();
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if duration.num_hours() > 0 {
        let hours = duration.num_hours();
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else {
        "Just now".to_string()
    }
}

pub fn rating_stars_text(rating: f32) -> String {
    let full_stars = rating.floor() as usize;
    let half_star = (rating - rating.floor()) >= 0.5;
    let empty_stars = 5 - full_stars - if half_star { 1 } else { 0 };

    let mut stars = "★".repeat(full_stars);
    if half_star {
        stars.push('⯪');
    }
    stars.push_str(&"☆".repeat(empty_stars));
    stars
}

pub use category_style::{badge_color, badge_text_color};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    mod rating_stars_text_tests {
        use super::*;

        #[test]
        fn test_zero_rating() {
            assert_eq!(rating_stars_text(0.0), "☆☆☆☆☆");
        }

        #[test]
        fn test_full_rating() {
            assert_eq!(rating_stars_text(5.0), "★★★★★");
        }

        #[test]
        fn test_half_star() {
            assert_eq!(rating_stars_text(2.5), "★★⯪☆☆");
        }

        #[test]
        fn test_three_stars() {
            assert_eq!(rating_stars_text(3.0), "★★★☆☆");
        }

        #[test]
        fn test_high_partial() {
            assert_eq!(rating_stars_text(4.7), "★★★★⯪");
        }
    }

    mod format_relative_time_tests {
        use super::*;

        #[test]
        fn test_just_now() {
            let now = Utc::now();
            assert_eq!(format_relative_time(&now), "Just now");
        }

        #[test]
        fn test_hours_ago() {
            let date = Utc::now() - Duration::hours(3);
            assert_eq!(format_relative_time(&date), "3 hours ago");
        }

        #[test]
        fn test_one_hour_ago() {
            let date = Utc::now() - Duration::hours(1);
            assert_eq!(format_relative_time(&date), "1 hour ago");
        }

        #[test]
        fn test_days_ago() {
            let date = Utc::now() - Duration::days(5);
            assert_eq!(format_relative_time(&date), "5 days ago");
        }

        #[test]
        fn test_one_day_ago() {
            let date = Utc::now() - Duration::days(1);
            assert_eq!(format_relative_time(&date), "1 day ago");
        }

        #[test]
        fn test_months_ago() {
            let date = Utc::now() - Duration::days(60);
            assert_eq!(format_relative_time(&date), "2 months ago");
        }

        #[test]
        fn test_one_month_ago() {
            let date = Utc::now() - Duration::days(31);
            assert_eq!(format_relative_time(&date), "1 month ago");
        }

        #[test]
        fn test_years_ago() {
            let date = Utc::now() - Duration::days(730);
            assert_eq!(format_relative_time(&date), "2 years ago");
        }

        #[test]
        fn test_one_year_ago() {
            let date = Utc::now() - Duration::days(400);
            assert_eq!(format_relative_time(&date), "1 year ago");
        }
    }
}
