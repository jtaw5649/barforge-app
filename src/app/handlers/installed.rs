use iced::Task;

use crate::app::message::Message;
use crate::app::state::{App, NotificationKind};
use crate::domain::InstalledModule;
use crate::tasks;

pub fn handle_install_module(app: &mut App, uuid: crate::domain::ModuleUuid) -> Task<Message> {
    let uuid_str = uuid.to_string();
    if app.installed_uuids.contains(&uuid_str) {
        app.push_notification(
            "Module already installed".to_string(),
            NotificationKind::Info,
        );
        app.module_detail.installing = false;
        return Task::none();
    }

    if let Some(registry) = &app.registry
        && let Some(module) = registry.find_by_uuid(&uuid_str)
    {
        return tasks::install_module(
            uuid_str,
            module.name.clone(),
            module.version.clone(),
            module.repo_url.clone(),
            module.checksum.clone(),
        );
    }

    app.push_notification(
        "Module not found in registry".to_string(),
        NotificationKind::Error,
    );
    app.module_detail.installing = false;
    Task::none()
}

pub fn handle_toggle_module(
    app: &mut App,
    uuid: crate::domain::ModuleUuid,
    enabled: bool,
) -> Task<Message> {
    let uuid_str = uuid.to_string();
    app.installed.toggling.insert(uuid_str.clone());
    tasks::toggle_module(uuid_str, enabled)
}

pub fn handle_set_module_position(
    uuid: crate::domain::ModuleUuid,
    section: crate::domain::BarSection,
) -> Task<Message> {
    tasks::change_module_position(uuid.to_string(), section)
}

pub fn handle_position_changed(app: &mut App, result: Result<String, String>) -> Task<Message> {
    match result {
        Ok(uuid) => {
            if let Some(module) = app
                .installed_modules
                .iter()
                .find(|m| m.uuid.to_string() == uuid)
            {
                let section_name = module
                    .position
                    .as_ref()
                    .map(|p| format!("{}", p.section))
                    .unwrap_or_else(|| "center".to_string());
                app.push_notification(
                    format!("Moved to {section_name}"),
                    NotificationKind::Success,
                );
            }
            tasks::load_installed()
        }
        Err(e) => {
            app.push_notification(
                format!("Failed to change position: {e}"),
                NotificationKind::Error,
            );
            Task::none()
        }
    }
}

pub fn handle_uninstall_module(app: &mut App, uuid: crate::domain::ModuleUuid) -> Task<Message> {
    let uuid_str = uuid.to_string();
    app.installed.uninstalling.insert(uuid_str.clone());
    tasks::uninstall_module(uuid_str)
}

pub fn handle_update_module(app: &mut App, uuid: crate::domain::ModuleUuid) -> Task<Message> {
    let uuid_str = uuid.to_string();
    if app.installed_modules.iter().any(|m| m.uuid == uuid)
        && let Some(registry) = &app.registry
        && let Some(registry_module) = registry.find_by_uuid(&uuid_str)
        && let Some(new_version) = &registry_module.version
    {
        app.installed.updating.insert(uuid_str.clone());
        return tasks::update_module(
            uuid_str,
            registry_module.repo_url.clone(),
            new_version.clone(),
        );
    }
    app.push_notification(
        "Cannot update: module not found".to_string(),
        NotificationKind::Error,
    );
    Task::none()
}

pub fn handle_update_all_modules(app: &mut App) -> Task<Message> {
    if app.installed.updating_all {
        return Task::none();
    }

    let updates: Vec<_> = app
        .installed_modules
        .iter()
        .filter_map(|installed| {
            let uuid = installed.uuid.to_string();
            app.registry.as_ref().and_then(|registry| {
                registry
                    .modules
                    .iter()
                    .find(|m| m.uuid.to_string() == uuid)
                    .and_then(|reg_mod| {
                        reg_mod.version.as_ref().and_then(|new_ver| {
                            if new_ver > &installed.version {
                                Some((uuid, reg_mod.repo_url.clone(), new_ver.clone()))
                            } else {
                                None
                            }
                        })
                    })
            })
        })
        .collect();

    if updates.is_empty() {
        app.push_notification(
            "All modules are up to date".to_string(),
            NotificationKind::Info,
        );
        return Task::none();
    }

    app.installed.updating_all = true;
    tasks::update_all_modules(updates)
}

pub fn handle_update_completed(
    app: &mut App,
    result: Result<InstalledModule, String>,
) -> Task<Message> {
    match result {
        Ok(updated_module) => {
            let uuid = updated_module.uuid.to_string();
            app.installed.updating.remove(&uuid);

            if let Some(existing) = app
                .installed_modules
                .iter_mut()
                .find(|m| m.uuid.to_string() == uuid)
            {
                existing.version = updated_module.version;
                existing.registry_version = updated_module.registry_version;
            }

            app.push_notification(
                format!("Updated {}", updated_module.waybar_module_name),
                NotificationKind::Success,
            );
        }
        Err(e) => {
            app.push_notification(format!("Update failed: {e}"), NotificationKind::Error);
        }
    }
    Task::none()
}

pub fn handle_update_all_completed(app: &mut App, result: Result<usize, String>) -> Task<Message> {
    app.installed.updating_all = false;
    match result {
        Ok(count) => {
            app.push_notification(
                format!(
                    "Updated {} module{}",
                    count,
                    if count == 1 { "" } else { "s" }
                ),
                NotificationKind::Success,
            );
            tasks::load_installed()
        }
        Err(e) => {
            app.push_notification(format!("Batch update failed: {e}"), NotificationKind::Error);
            Task::none()
        }
    }
}
