use iced::Task;

use crate::app::message::Message;
use crate::app::state::{App, NotificationKind, Screen};
use crate::theme::ThemeMode;

pub fn handle_system_theme_changed(app: &mut App, is_dark: bool) -> Task<Message> {
    app.set_system_dark(is_dark);
    Task::none()
}

pub fn handle_set_theme_mode(app: &mut App, mode: ThemeMode) -> Task<Message> {
    app.set_theme_mode(mode);
    app.save_settings();
    Task::none()
}

pub fn handle_omarchy_theme_changed(app: &mut App) -> Task<Message> {
    if app.theme_mode == ThemeMode::Omarchy {
        let palette = crate::services::load_omarchy_palette();
        app.set_omarchy_palette(palette);
    }
    Task::none()
}

pub fn handle_toggle_tray(app: &mut App, enabled: bool) -> Task<Message> {
    app.tray_enabled = enabled;
    app.save_settings();

    if enabled && app.tray_receiver.is_none() {
        app.tray_receiver = crate::tray::init();
        app.push_notification("Tray icon enabled".to_string(), NotificationKind::Success);
    } else if !enabled {
        crate::tray::shutdown();
        app.tray_receiver = None;
        app.push_notification("Tray icon disabled".to_string(), NotificationKind::Info);
    }

    Task::none()
}

pub fn handle_open_preferences(app: &mut App, uuid: crate::domain::ModuleUuid) -> Task<Message> {
    let uuid_str = uuid.to_string();
    if let Some(installed) = app.installed_modules.iter().find(|m| m.uuid == uuid) {
        let schema = crate::services::load_schema(&installed.install_path);
        if let Some(schema) = schema {
            let values = crate::services::load_preferences(&uuid_str);
            let merged = crate::services::preferences::merge_with_defaults(values, &schema);
            app.preferences.open_for = Some(uuid_str);
            app.preferences.schema = Some(schema);
            app.preferences.values = merged;
            app.preferences.module_name = installed.waybar_module_name.clone();
        } else {
            app.push_notification(
                "This module has no configurable preferences".to_string(),
                NotificationKind::Info,
            );
        }
    }
    Task::none()
}

pub fn handle_preference_changed(
    app: &mut App,
    uuid: crate::domain::ModuleUuid,
    key: String,
    value: crate::services::PreferenceValue,
) -> Task<Message> {
    let uuid_str = uuid.to_string();
    if app.preferences.open_for.as_ref() == Some(&uuid_str) {
        app.preferences.values.insert(key, value);
        if let Err(e) = crate::services::save_preferences(&uuid_str, &app.preferences.values) {
            tracing::warn!("Failed to save preferences: {e}");
            app.push_notification(
                "Failed to save preferences".to_string(),
                NotificationKind::Error,
            );
        }
    }
    Task::none()
}

pub fn handle_close_preferences(app: &mut App) -> Task<Message> {
    app.preferences.open_for = None;
    app.preferences.schema = None;
    app.preferences.values.clear();
    Task::none()
}

pub fn handle_reset_preferences(app: &mut App, uuid: crate::domain::ModuleUuid) -> Task<Message> {
    let uuid_str = uuid.to_string();
    if let Some(schema) = &app.preferences.schema {
        let defaults = crate::services::preferences::get_default_preferences(schema);
        app.preferences.values = defaults.clone();
        match crate::services::save_preferences(&uuid_str, &defaults) {
            Ok(()) => {
                app.push_notification(
                    "Preferences reset to defaults".to_string(),
                    NotificationKind::Success,
                );
            }
            Err(e) => {
                tracing::warn!("Failed to save reset preferences: {e}");
                app.push_notification(
                    "Failed to save preferences".to_string(),
                    NotificationKind::Error,
                );
            }
        }
    }
    Task::none()
}

pub fn handle_focus_search(app: &mut App) -> Task<Message> {
    if !matches!(app.screen, Screen::Browse | Screen::Installed) {
        app.screen = Screen::Browse;
    }
    Task::none()
}

pub fn handle_escape_pressed(app: &mut App) -> Task<Message> {
    if app.preferences.open_for.is_some() {
        app.preferences.open_for = None;
        app.preferences.schema = None;
        app.preferences.values.clear();
    } else if app.confirmation.pending_action.is_some() {
        app.confirmation.pending_action = None;
    } else if matches!(app.screen, Screen::ModuleDetail(_)) {
        app.screen = Screen::Browse;
        app.module_detail.screenshot = crate::app::state::ScreenshotState::NotLoaded;
        app.module_detail.installing = false;
    } else if !app.browse.search_query.is_empty() && app.screen == Screen::Browse {
        app.browse.search_query.clear();
        app.browse.pending_search = None;
    } else if !app.installed.search_query.is_empty() && app.screen == Screen::Installed {
        app.installed.search_query.clear();
        app.installed.pending_search = None;
    } else {
        app.notifications.pop_front();
    }
    Task::none()
}
