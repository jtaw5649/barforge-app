use std::time::Duration;

use iced::Task;

use crate::app::message::Message;
use crate::app::state::{App, ConfirmationAction, NotificationKind, Screen, ScreenshotState};
use crate::security::validate_web_url;
use crate::tasks;
use crate::tray::TrayEvent;

pub fn handle_tick(app: &mut App) -> Task<Message> {
    if app.loading.is_loading() || app.module_detail.screenshot == ScreenshotState::Loading {
        app.advance_spinner();
    }

    app.notifications.retain(|notif| {
        notif.kind == NotificationKind::Error
            || notif.created_at.elapsed() <= Duration::from_secs(5)
    });

    app.apply_debounced_searches();

    if let Some(tray_event) = app.poll_tray_events() {
        return match tray_event {
            TrayEvent::ShowWindow => Task::done(Message::TrayShowWindow),
            TrayEvent::CheckUpdates => Task::done(Message::TrayCheckUpdates),
            TrayEvent::Quit => Task::done(Message::TrayQuit),
        };
    }

    Task::none()
}

pub fn handle_show_notification(app: &mut App, message: String, kind: NotificationKind) {
    app.push_notification(message, kind);
}

pub fn handle_dismiss_notification(app: &mut App) -> Task<Message> {
    if app.confirmation.pending_action.is_some() {
        app.confirmation.pending_action = None;
    } else if matches!(app.screen, Screen::ModuleDetail(_)) {
        app.screen = Screen::Browse;
        app.module_detail.screenshot = ScreenshotState::NotLoaded;
        app.module_detail.installing = false;
    } else {
        app.notifications.pop_front();
    }
    Task::none()
}

pub fn handle_request_confirmation(app: &mut App, action: ConfirmationAction) {
    app.confirmation.pending_action = Some(action);
}

pub fn handle_confirm_action(app: &mut App) -> Task<Message> {
    if let Some(action) = app.confirmation.pending_action.take() {
        match action {
            ConfirmationAction::UninstallModule { uuid, .. } => {
                app.installed.uninstalling.insert(uuid.clone());
                return tasks::uninstall_module(uuid);
            }
        }
    }
    Task::none()
}

pub fn handle_cancel_confirmation(app: &mut App) {
    app.confirmation.pending_action = None;
}

pub fn handle_clear_cache() -> Task<Message> {
    tasks::clear_cache()
}

pub fn handle_cache_clear_completed(app: &mut App, result: Result<(), String>) {
    match result {
        Ok(()) => {
            app.push_notification(
                "Cache cleared successfully".to_string(),
                NotificationKind::Success,
            );
        }
        Err(e) => {
            app.push_notification(
                format!("Failed to clear cache: {e}"),
                NotificationKind::Error,
            );
        }
    }
}

pub fn handle_reset_settings() -> Task<Message> {
    tasks::reset_settings()
}

pub fn handle_settings_reset_completed(app: &mut App, result: Result<(), String>) {
    match result {
        Ok(()) => {
            app.push_notification(
                "Settings reset successfully".to_string(),
                NotificationKind::Success,
            );
        }
        Err(e) => {
            app.push_notification(
                format!("Failed to reset settings: {e}"),
                NotificationKind::Error,
            );
        }
    }
}

pub fn handle_open_repo_url(app: &mut App, url: String) {
    match validate_web_url(&url) {
        Ok(()) => {
            if let Err(e) = open::that(&url) {
                tracing::warn!("Failed to open URL in browser: {e}");
            }
        }
        Err(e) => {
            app.push_notification(format!("Cannot open URL: {e}"), NotificationKind::Error);
        }
    }
}

pub fn handle_detail_install_module(app: &mut App) -> Task<Message> {
    if let Screen::ModuleDetail(ref uuid_str) = app.screen
        && let Ok(uuid) = crate::domain::ModuleUuid::try_from(uuid_str.as_str())
    {
        app.module_detail.installing = true;
        return Task::done(Message::InstallModule(uuid));
    }
    Task::none()
}

pub fn handle_refresh_registry(app: &mut App) -> Task<Message> {
    app.browse.refreshing = true;
    tasks::refresh_registry()
}

pub fn handle_tray_show_window() -> Task<Message> {
    Task::none()
}

pub fn handle_tray_check_updates(app: &mut App) -> Task<Message> {
    app.screen = Screen::Updates;
    app.push_notification(
        "Checking for updates...".to_string(),
        NotificationKind::Info,
    );
    tasks::load_registry()
}

pub fn handle_tray_quit() -> ! {
    crate::tray::shutdown();
    std::process::exit(0);
}
