use iced::Task;

use crate::app::message::Message;
use crate::app::state::{App, NotificationKind};
use crate::domain::ModuleUuid;
use crate::security::SandboxStatus;
use crate::services::{DepReport, InstallStage};

pub fn handle_install_progress(app: &mut App, uuid: ModuleUuid, stage: InstallStage) -> Task<Message> {
    if let crate::app::state::Screen::ModuleDetail(detail_uuid) = &app.screen
        && *detail_uuid == uuid.to_string()
    {
        app.module_detail.install_stage = Some(stage);
    }
    Task::none()
}

pub fn handle_dependency_check_completed(
    app: &mut App,
    result: Result<DepReport, String>,
) -> Task<Message> {
    match result {
        Ok(report) => {
            if !report.all_satisfied {
                let missing = report.missing_required.join(", ");
                app.push_notification(
                    format!("Missing dependencies: {missing}"),
                    NotificationKind::Warning,
                );
            }
        }
        Err(e) => {
            app.push_notification(
                format!("Dependency check failed: {e}"),
                NotificationKind::Error,
            );
        }
    }
    Task::none()
}

pub fn handle_revocation_check_completed(
    app: &mut App,
    result: Result<(), String>,
) -> Task<Message> {
    if let Err(e) = result {
        app.module_detail.installing = false;
        app.module_detail.install_stage = None;
        app.push_notification(
            format!("Module revoked or check failed: {e}"),
            NotificationKind::Error,
        );
    }
    Task::none()
}

pub fn handle_signature_verified(app: &mut App, result: Result<(), String>) -> Task<Message> {
    if let Err(e) = result {
        app.module_detail.installing = false;
        app.module_detail.install_stage = None;
        app.push_notification(
            format!("Signature verification failed: {e}"),
            NotificationKind::Error,
        );
    }
    Task::none()
}

pub fn handle_sandbox_status_changed(app: &mut App, status: SandboxStatus) -> Task<Message> {
    app.sandbox_status = Some(status);

    match status {
        SandboxStatus::FullyEnforced => {}
        SandboxStatus::PartiallyEnforced => {
            app.push_notification(
                "Sandbox partially enforced (older kernel)".to_string(),
                NotificationKind::Warning,
            );
        }
        SandboxStatus::NotSupported => {
            app.push_notification(
                "Sandbox not supported on this system".to_string(),
                NotificationKind::Warning,
            );
        }
        SandboxStatus::Failed => {
            app.push_notification(
                "Failed to apply sandbox restrictions".to_string(),
                NotificationKind::Error,
            );
        }
    }
    Task::none()
}
