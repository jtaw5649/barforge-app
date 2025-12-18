use iced::Task;

use crate::app::message::Message;
use crate::app::state::{App, LoadingState, NotificationKind};
use crate::domain::{InstalledModule, RegistryIndex};

pub fn handle_registry_loaded(app: &mut App, result: Result<RegistryIndex, String>) -> Task<Message> {
    match result {
        Ok(index) => {
            app.sync_registry_versions(&index);
            app.registry = Some(index);
            app.loading = LoadingState::Idle;
            app.browse.last_refreshed = Some(std::time::Instant::now());
        }
        Err(e) => {
            app.loading = LoadingState::Failed(e);
        }
    }
    Task::none()
}

pub fn handle_registry_refreshed(app: &mut App, result: Result<RegistryIndex, String>) -> Task<Message> {
    app.browse.refreshing = false;
    match result {
        Ok(index) => {
            let count = index.modules.len();
            app.sync_registry_versions(&index);
            app.registry = Some(index);
            app.browse.last_refreshed = Some(std::time::Instant::now());
            app.push_notification(
                format!("Registry refreshed ({count} modules)"),
                NotificationKind::Success,
            );
        }
        Err(e) => {
            app.push_notification(
                format!("Failed to refresh registry: {e}"),
                NotificationKind::Error,
            );
        }
    }
    Task::none()
}

pub fn handle_installed_loaded(
    app: &mut App,
    result: Result<Vec<InstalledModule>, String>,
) -> Task<Message> {
    match result {
        Ok(modules) => {
            app.installed_uuids = modules.iter().map(|m| m.uuid.to_string()).collect();
            app.installed_modules = modules;
        }
        Err(e) => {
            app.push_notification(
                format!("Failed to load installed modules: {e}"),
                NotificationKind::Error,
            );
        }
    }
    Task::none()
}

pub fn handle_install_completed(
    app: &mut App,
    result: Result<InstalledModule, String>,
) -> Task<Message> {
    app.module_detail.installing = false;
    match result {
        Ok(module) => {
            app.installed_uuids.insert(module.uuid.to_string());
            app.installed_modules.push(module);
            app.push_notification(
                "Module installed successfully".to_string(),
                NotificationKind::Success,
            );
        }
        Err(e) => {
            app.push_notification(format!("Installation failed: {e}"), NotificationKind::Error);
        }
    }
    Task::none()
}

pub fn handle_toggle_completed(
    app: &mut App,
    result: Result<String, (String, String)>,
) -> Task<Message> {
    match result {
        Ok(uuid) => {
            app.installed.toggling.remove(&uuid);
            if let Some(m) = app
                .installed_modules
                .iter_mut()
                .find(|m| m.uuid.to_string() == uuid)
            {
                m.enabled = !m.enabled;
            }
        }
        Err((uuid, e)) => {
            app.installed.toggling.remove(&uuid);
            app.push_notification(format!("Toggle failed: {e}"), NotificationKind::Error);
        }
    }
    Task::none()
}

pub fn handle_uninstall_completed(
    app: &mut App,
    result: Result<String, (String, String)>,
) -> Task<Message> {
    match result {
        Ok(uuid) => {
            app.installed.uninstalling.remove(&uuid);
            app.installed_uuids.remove(&uuid);
            app.installed_modules.retain(|m| m.uuid.to_string() != uuid);
            app.push_notification("Module uninstalled".to_string(), NotificationKind::Success);
        }
        Err((uuid, e)) => {
            app.installed.uninstalling.remove(&uuid);
            app.push_notification(format!("Uninstall failed: {e}"), NotificationKind::Error);
        }
    }
    Task::none()
}
