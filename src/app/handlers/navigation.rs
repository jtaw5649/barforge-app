use iced::Task;
use iced::widget::image;

use crate::app::message::Message;
use crate::app::state::{App, AuthorLoadingState, ReviewsLoadingState, Screen, ScreenshotState};
use crate::domain::ModuleUuid;
use crate::tasks;

pub fn handle_navigate(app: &mut App, screen: Screen) -> Task<Message> {
    if let Screen::ModuleDetail(ref uuid) = screen {
        app.module_detail.screenshot = ScreenshotState::Loading;
        app.module_detail.installing = false;
        app.module_detail.reviews = ReviewsLoadingState::Loading;
        app.screen = screen.clone();

        let mut tasks_to_run: Vec<Task<Message>> = Vec::new();

        if let Some(registry) = &app.registry
            && let Some(module) = registry.find_by_uuid(uuid)
        {
            if let Some(screenshot_url) = &module.screenshot {
                tasks_to_run.push(tasks::load_screenshot(screenshot_url.clone()));
            } else {
                app.module_detail.screenshot = ScreenshotState::NotLoaded;
            }

            if let Ok(module_uuid) = ModuleUuid::try_from(uuid.as_str()) {
                tasks_to_run.push(tasks::load_module_reviews(module_uuid));
            }
        } else {
            app.module_detail.screenshot = ScreenshotState::NotLoaded;
            app.module_detail.reviews = ReviewsLoadingState::NotLoaded;
        }

        return if tasks_to_run.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks_to_run)
        };
    }
    app.screen = screen;
    Task::none()
}

pub fn handle_navigate_back(app: &mut App) -> Task<Message> {
    app.screen = Screen::Browse;
    app.module_detail.screenshot = ScreenshotState::NotLoaded;
    app.module_detail.installing = false;
    app.module_detail.reviews = ReviewsLoadingState::NotLoaded;
    Task::none()
}

pub fn handle_screenshot_loaded(
    app: &mut App,
    result: Result<image::Handle, String>,
) -> Task<Message> {
    match result {
        Ok(handle) => {
            app.module_detail.screenshot = ScreenshotState::Loaded(handle);
        }
        Err(_) => {
            app.module_detail.screenshot = ScreenshotState::Failed;
        }
    }
    Task::none()
}

pub fn handle_author_clicked(app: &mut App, username: String) -> Task<Message> {
    app.author_profile.loading = AuthorLoadingState::Loading;
    app.screen = Screen::AuthorProfile(username.clone());
    tasks::load_author_profile(username)
}
