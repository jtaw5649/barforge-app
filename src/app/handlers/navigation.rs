use iced::widget::image;
use iced::Task;

use crate::app::message::Message;
use crate::app::state::{App, Screen, ScreenshotState};
use crate::tasks;

pub fn handle_navigate(app: &mut App, screen: Screen) -> Task<Message> {
    if let Screen::ModuleDetail(ref uuid) = screen {
        app.module_detail.screenshot = ScreenshotState::Loading;
        app.module_detail.installing = false;
        app.screen = screen.clone();

        if let Some(registry) = &app.registry
            && let Some(module) = registry.find_by_uuid(uuid)
            && let Some(screenshot_url) = &module.screenshot
        {
            return tasks::load_screenshot(screenshot_url.clone());
        }
        app.module_detail.screenshot = ScreenshotState::NotLoaded;
        return Task::none();
    }
    app.screen = screen;
    Task::none()
}

pub fn handle_navigate_back(app: &mut App) -> Task<Message> {
    app.screen = Screen::Browse;
    app.module_detail.screenshot = ScreenshotState::NotLoaded;
    app.module_detail.installing = false;
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
