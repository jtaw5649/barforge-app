pub mod handlers;
pub mod message;
pub mod state;

use std::time::Duration;

use iced::widget::{button, column, container, pick_list, responsive, row, scrollable, stack, text, text_input, Space};
use iced::{Alignment, Element, Length, Task, Theme};
use iced_aw::Wrap;

use crate::icons::Icon;
use crate::security::validate_web_url;
use crate::services::is_omarchy_available;
use crate::tasks;
use crate::theme::{darken, menu_style, pick_list_style, PickListColors, CARD_WIDTH, RADIUS_MD, SEARCH_PANEL_WIDTH, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS};

use crate::tray::TrayEvent;
use crate::widget::{confirmation_dialog, empty_state, empty_state_dynamic, empty_state_with_action, module_card, module_detail_screen, module_row, module_table, notification_toast, preferences_modal, settings_screen, sidebar, skeleton_card};

pub use message::Message;
pub use state::{App, CategoryFilter, LoadingState, Screen, ScreenshotState, SortField, SortOrder, ViewMode};

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            loading: LoadingState::Loading,
            ..Default::default()
        };
        (app, tasks::initial_load())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(screen) => handlers::handle_navigate(self, screen),

            Message::NavigateBack => handlers::handle_navigate_back(self),

            Message::SearchChanged(query) => handlers::handle_search_changed(self, query),

            Message::CategorySelected(filter) => handlers::handle_category_selected(self, filter),

            Message::SetSortField(field) => handlers::handle_set_sort_field(self, field),

            Message::ToggleSortOrder => handlers::handle_toggle_sort_order(self),

            Message::SetViewMode(mode) => handlers::handle_set_view_mode(self, mode),

            Message::ToggleVerifiedOnly => handlers::handle_toggle_verified_only(self),

            Message::ModuleClicked(_uuid) => Task::none(),

            Message::InstallModule(uuid) => {
                if self.installed_uuids.contains(&uuid) {
                    self.push_notification(
                        "Module already installed".to_string(),
                        state::NotificationKind::Info,
                    );
                    self.module_detail.installing = false;
                    return Task::none();
                }

                if let Some(registry) = &self.registry
                    && let Some(module) = registry.find_by_uuid(&uuid)
                {
                    return tasks::install_module(
                        uuid,
                        module.name.clone(),
                        module.version.clone(),
                        module.repo_url.clone(),
                    );
                }

                self.push_notification(
                    "Module not found in registry".to_string(),
                    state::NotificationKind::Error,
                );
                self.module_detail.installing = false;
                Task::none()
            }

            Message::ToggleModule { uuid, enabled } => {
                self.installed.toggling.insert(uuid.clone());
                tasks::toggle_module(uuid, enabled)
            }

            Message::SetModulePosition { uuid, section } => {
                tasks::change_module_position(uuid, section)
            }

            Message::PositionChanged(result) => {
                match result {
                    Ok(uuid) => {
                        if let Some(module) = self.installed_modules.iter().find(|m| m.uuid.to_string() == uuid) {
                            let section_name = module
                                .position
                                .as_ref()
                                .map(|p| format!("{}", p.section))
                                .unwrap_or_else(|| "center".to_string());
                            self.push_notification(
                                format!("Moved to {}", section_name),
                                state::NotificationKind::Success,
                            );
                        }
                        return tasks::load_installed();
                    }
                    Err(e) => {
                        self.push_notification(
                            format!("Failed to change position: {e}"),
                            state::NotificationKind::Error,
                        );
                    }
                }
                Task::none()
            }

            Message::UninstallModule(uuid) => {
                self.installed.uninstalling.insert(uuid.clone());
                tasks::uninstall_module(uuid)
            }

            Message::OpenPreferences(uuid) => {
                if let Some(installed) = self.installed_modules.iter().find(|m| m.uuid.to_string() == uuid) {
                    let schema = crate::services::load_schema(&installed.install_path);
                    if let Some(schema) = schema {
                        let values = crate::services::load_preferences(&uuid);
                        let merged = crate::services::preferences::merge_with_defaults(values, &schema);
                        self.preferences.open_for = Some(uuid);
                        self.preferences.schema = Some(schema);
                        self.preferences.values = merged;
                        self.preferences.module_name = installed.waybar_module_name.clone();
                    } else {
                        self.push_notification(
                            "This module has no configurable preferences".to_string(),
                            state::NotificationKind::Info,
                        );
                    }
                }
                Task::none()
            }

            Message::InstalledSearchChanged(query) => {
                handlers::handle_installed_search_changed(self, query)
            }

            Message::ClearInstalledSearch => handlers::handle_clear_installed_search(self),

            Message::RefreshRegistry => {
                self.browse.refreshing = true;
                tasks::refresh_registry()
            }

            Message::RegistryLoaded(result) => handlers::handle_registry_loaded(self, result),

            Message::RegistryRefreshed(result) => handlers::handle_registry_refreshed(self, result),

            Message::InstalledLoaded(result) => handlers::handle_installed_loaded(self, result),

            Message::InstallCompleted(result) => handlers::handle_install_completed(self, result),

            Message::ToggleCompleted(result) => handlers::handle_toggle_completed(self, result),

            Message::UninstallCompleted(result) => handlers::handle_uninstall_completed(self, result),

            Message::UpdateModule(uuid) => {
                if let Some(_installed) = self.installed_modules.iter().find(|m| m.uuid.to_string() == uuid)
                    && let Some(registry) = &self.registry
                    && let Some(registry_module) = registry.find_by_uuid(&uuid)
                    && let Some(new_version) = &registry_module.version
                {
                    self.installed.updating.insert(uuid.clone());
                    return tasks::update_module(
                        uuid,
                        registry_module.repo_url.clone(),
                        new_version.clone(),
                    );
                }
                self.push_notification(
                    "Cannot update: module not found".to_string(),
                    state::NotificationKind::Error,
                );
                Task::none()
            }

            Message::UpdateAllModules => {
                if self.installed.updating_all {
                    return Task::none();
                }

                let updates: Vec<_> = self
                    .installed_modules
                    .iter()
                    .filter_map(|installed| {
                        let uuid = installed.uuid.to_string();
                        self.registry.as_ref().and_then(|registry| {
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
                    self.push_notification(
                        "All modules are up to date".to_string(),
                        state::NotificationKind::Info,
                    );
                    return Task::none();
                }

                self.installed.updating_all = true;
                tasks::update_all_modules(updates)
            }

            Message::UpdateCompleted(result) => {
                match result {
                    Ok(updated_module) => {
                        let uuid = updated_module.uuid.to_string();
                        self.installed.updating.remove(&uuid);

                        if let Some(existing) = self.installed_modules.iter_mut().find(|m| m.uuid.to_string() == uuid) {
                            existing.version = updated_module.version;
                            existing.registry_version = updated_module.registry_version;
                        }

                        self.push_notification(
                            format!("Updated {}", updated_module.waybar_module_name),
                            state::NotificationKind::Success,
                        );
                    }
                    Err(e) => {
                        self.push_notification(
                            format!("Update failed: {e}"),
                            state::NotificationKind::Error,
                        );
                    }
                }
                Task::none()
            }

            Message::UpdateAllCompleted(result) => {
                self.installed.updating_all = false;
                match result {
                    Ok(count) => {
                        self.push_notification(
                            format!("Updated {} module{}", count, if count == 1 { "" } else { "s" }),
                            state::NotificationKind::Success,
                        );
                        return tasks::load_installed();
                    }
                    Err(e) => {
                        self.push_notification(
                            format!("Batch update failed: {e}"),
                            state::NotificationKind::Error,
                        );
                    }
                }
                Task::none()
            }

            Message::ShowNotification(message, kind) => {
                self.push_notification(message, kind);
                Task::none()
            }

            Message::DismissNotification => {
                if self.confirmation.pending_action.is_some() {
                    self.confirmation.pending_action = None;
                } else if matches!(self.screen, Screen::ModuleDetail(_)) {
                    self.screen = Screen::Browse;
                    self.module_detail.screenshot = state::ScreenshotState::NotLoaded;
                    self.module_detail.installing = false;
                } else {
                    self.notifications.pop_front();
                }
                Task::none()
            }

            Message::Tick => {
                if self.loading.is_loading() || self.module_detail.screenshot == ScreenshotState::Loading {
                    self.advance_spinner();
                }

                self.notifications.retain(|notif| {
                    notif.kind == state::NotificationKind::Error
                        || notif.created_at.elapsed() <= Duration::from_secs(5)
                });

                self.apply_debounced_searches();

                if let Some(tray_event) = self.poll_tray_events() {
                    return match tray_event {
                        TrayEvent::ShowWindow => Task::done(Message::TrayShowWindow),
                        TrayEvent::CheckUpdates => Task::done(Message::TrayCheckUpdates),
                        TrayEvent::Quit => Task::done(Message::TrayQuit),
                    };
                }

                Task::none()
            }

            Message::SystemThemeChanged(is_dark) => {
                self.set_system_dark(is_dark);
                Task::none()
            }

            Message::SetThemeMode(mode) => {
                self.set_theme_mode(mode);
                self.save_settings();
                Task::none()
            }

            Message::OmarchyThemeChanged => {
                if self.theme_mode == crate::theme::ThemeMode::Omarchy {
                    let palette = crate::services::load_omarchy_palette();
                    self.set_omarchy_palette(palette);
                }
                Task::none()
            }

            Message::ScreenshotLoaded(result) => handlers::handle_screenshot_loaded(self, result),

            Message::DetailInstallModule => {
                if let Screen::ModuleDetail(ref uuid) = self.screen {
                    self.module_detail.installing = true;
                    let uuid_clone = uuid.clone();
                    return Task::done(Message::InstallModule(uuid_clone));
                }
                Task::none()
            }

            Message::OpenRepoUrl(url) => {
                match validate_web_url(&url) {
                    Ok(()) => {
                        if let Err(e) = open::that(&url) {
                            tracing::warn!("Failed to open URL in browser: {e}");
                        }
                    }
                    Err(e) => {
                        self.push_notification(
                            format!("Cannot open URL: {e}"),
                            state::NotificationKind::Error,
                        );
                    }
                }
                Task::none()
            }

            Message::RequestConfirmation(action) => {
                self.confirmation.pending_action = Some(action);
                Task::none()
            }

            Message::ConfirmAction => {
                if let Some(action) = self.confirmation.pending_action.take() {
                    match action {
                        state::ConfirmationAction::UninstallModule { uuid, .. } => {
                            self.installed.uninstalling.insert(uuid.clone());
                            return tasks::uninstall_module(uuid);
                        }
                    }
                }
                Task::none()
            }

            Message::CancelConfirmation => {
                self.confirmation.pending_action = None;
                Task::none()
            }

            Message::ClearCache => tasks::clear_cache(),

            Message::CacheClearCompleted(result) => {
                match result {
                    Ok(()) => {
                        self.push_notification("Cache cleared successfully".to_string(), state::NotificationKind::Success);
                    }
                    Err(e) => {
                        self.push_notification(format!("Failed to clear cache: {e}"), state::NotificationKind::Error);
                    }
                }
                Task::none()
            }

            Message::ResetSettings => tasks::reset_settings(),

            Message::SettingsResetCompleted(result) => {
                match result {
                    Ok(()) => {
                        self.push_notification("Settings reset successfully".to_string(), state::NotificationKind::Success);
                    }
                    Err(e) => {
                        self.push_notification(format!("Failed to reset settings: {e}"), state::NotificationKind::Error);
                    }
                }
                Task::none()
            }

            Message::ToggleTray(enabled) => {
                self.tray_enabled = enabled;
                self.save_settings();

                if enabled && self.tray_receiver.is_none() {
                    self.tray_receiver = crate::tray::init();
                    self.push_notification(
                        "Tray icon enabled".to_string(),
                        state::NotificationKind::Success,
                    );
                } else if !enabled {
                    crate::tray::shutdown();
                    self.tray_receiver = None;
                    self.push_notification(
                        "Tray icon disabled".to_string(),
                        state::NotificationKind::Info,
                    );
                }

                Task::none()
            }

            Message::FocusSearch => {
                if !matches!(self.screen, Screen::Browse | Screen::Installed) {
                    self.screen = Screen::Browse;
                }
                Task::none()
            }

            Message::EscapePressed => {
                if self.preferences.open_for.is_some() {
                    self.preferences.open_for = None;
                    self.preferences.schema = None;
                    self.preferences.values.clear();
                } else if self.confirmation.pending_action.is_some() {
                    self.confirmation.pending_action = None;
                } else if matches!(self.screen, Screen::ModuleDetail(_)) {
                    self.screen = Screen::Browse;
                    self.module_detail.screenshot = state::ScreenshotState::NotLoaded;
                    self.module_detail.installing = false;
                } else if !self.browse.search_query.is_empty() && self.screen == Screen::Browse {
                    self.browse.search_query.clear();
                    self.browse.pending_search = None;
                } else if !self.installed.search_query.is_empty() && self.screen == Screen::Installed {
                    self.installed.search_query.clear();
                    self.installed.pending_search = None;
                } else {
                    self.notifications.pop_front();
                }
                Task::none()
            }

            Message::PreferenceChanged(uuid, key, value) => {
                if self.preferences.open_for.as_ref() == Some(&uuid) {
                    self.preferences.values.insert(key, value);
                    if let Err(e) = crate::services::save_preferences(&uuid, &self.preferences.values) {
                        tracing::warn!("Failed to save preferences: {e}");
                        self.push_notification(
                            "Failed to save preferences".to_string(),
                            state::NotificationKind::Error,
                        );
                    }
                }
                Task::none()
            }

            Message::ClosePreferences => {
                self.preferences.open_for = None;
                self.preferences.schema = None;
                self.preferences.values.clear();
                Task::none()
            }

            Message::ResetPreferences(uuid) => {
                if let Some(schema) = &self.preferences.schema {
                    let defaults = crate::services::preferences::get_default_preferences(schema);
                    self.preferences.values = defaults.clone();
                    match crate::services::save_preferences(&uuid, &defaults) {
                        Ok(()) => {
                            self.push_notification(
                                "Preferences reset to defaults".to_string(),
                                state::NotificationKind::Success,
                            );
                        }
                        Err(e) => {
                            tracing::warn!("Failed to save reset preferences: {e}");
                            self.push_notification(
                                "Failed to save preferences".to_string(),
                                state::NotificationKind::Error,
                            );
                        }
                    }
                }
                Task::none()
            }

            Message::TrayShowWindow => {
                Task::none()
            }

            Message::TrayCheckUpdates => {
                self.screen = Screen::Updates;
                self.push_notification(
                    "Checking for updates...".to_string(),
                    state::NotificationKind::Info,
                );
                tasks::load_registry()
            }

            Message::TrayQuit => {
                crate::tray::shutdown();
                std::process::exit(0);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = sidebar(
            &self.screen,
            self.installed_modules.len(),
            self.update_count(),
            &self.theme,
            self.theme_mode,
            is_omarchy_available(),
        );
        let content = match &self.screen {
            Screen::Browse => self.view_browse(),
            Screen::Installed => self.view_installed(),
            Screen::Updates => self.view_updates(),
            Screen::Settings => self.view_settings(),
            Screen::ModuleDetail(uuid) => self.view_module_detail(uuid),
        };

        let bg = self.theme.background;
        let main_content = container(content)
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(bg)),
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill);

        let main_layout = row![sidebar, main_content];

        let notification_overlay: Element<Message> = if self.notifications.is_empty() {
            Space::new().into()
        } else {
            let notification_stack = column(
                self.notifications.iter().map(|notif| notification_toast(notif, &self.theme))
            ).spacing(SPACING_SM);

            container(notification_stack)
                .width(Length::Fill)
                .padding([SPACING_LG, 0.0])
                .align_x(Alignment::Center)
                .into()
        };

        let confirmation_overlay: Element<Message> =
            if let Some(action) = &self.confirmation.pending_action {
                confirmation_dialog(action, &self.theme)
            } else {
                Space::new().into()
            };

        let preferences_overlay: Element<Message> =
            if let (Some(uuid), Some(schema)) = (&self.preferences.open_for, &self.preferences.schema) {
                preferences_modal(
                    &self.preferences.module_name,
                    uuid,
                    schema,
                    &self.preferences.values,
                    &self.theme,
                )
            } else {
                Space::new().into()
            };

        stack![main_layout, notification_overlay, confirmation_overlay, preferences_overlay].into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        settings_screen(&self.theme, self.tray_enabled)
    }

    fn view_module_detail(&self, uuid: &str) -> Element<'_, Message> {
        if let Some(registry) = &self.registry
            && let Some(module) = registry.find_by_uuid(uuid)
        {
            let is_installed = self.installed_uuids.contains(uuid);
            let installed_at = self
                .installed_modules
                .iter()
                .find(|m| m.uuid.to_string() == uuid)
                .map(|m| m.installed_at);
            return module_detail_screen(
                module,
                &self.module_detail.screenshot,
                is_installed,
                installed_at,
                self.module_detail.installing,
                &self.theme,
            );
        }
        empty_state(
            Icon::Error,
            "Module not found",
            "The requested module could not be found",
            &self.theme,
        )
    }

    fn view_browse(&self) -> Element<'_, Message> {
        let search_icon = Icon::Search.svg(16.0);
        let text_color = self.theme.text;
        let placeholder_color = self.theme.text_muted;
        let selection_color = iced::Color::from_rgba(
            self.theme.primary.r,
            self.theme.primary.g,
            self.theme.primary.b,
            0.3,
        );
        let search_field = text_input("Search modules...", self.browse_search_display())
            .on_input(Message::SearchChanged)
            .padding([SPACING_SM, SPACING_SM])
            .width(Length::Fill)
            .style(move |_theme, _status| iced::widget::text_input::Style {
                background: iced::Background::Color(iced::Color::TRANSPARENT),
                border: iced::Border::default(),
                icon: text_color,
                placeholder: placeholder_color,
                value: text_color,
                selection: selection_color,
            });
        let surface = self.theme.surface;
        let primary = self.theme.primary;
        let search_input = container(
            row![search_icon, search_field]
                .spacing(SPACING_SM)
                .align_y(Alignment::Center),
        )
        .padding([SPACING_XS, SPACING_MD])
        .width(Length::Fixed(SEARCH_PANEL_WIDTH))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(surface)),
            border: iced::Border {
                color: iced::Color::from_rgba(primary.r, primary.g, primary.b, 0.3),
                width: 1.0,
                radius: crate::theme::RADIUS_MD.into(),
            },
            ..Default::default()
        });

        let picker_colors = PickListColors {
            surface: self.theme.surface,
            text: self.theme.text,
            text_muted: self.theme.text_secondary,
            border: self.theme.border,
            primary: self.theme.primary,
            menu_surface: self.theme.surface,
            menu_border: self.theme.border,
            menu_text: self.theme.text,
            menu_selected_bg: self.theme.primary,
        };
        let category_picker = pick_list(
            CategoryFilter::all(),
            Some(self.browse.selected_category),
            Message::CategorySelected,
        )
        .padding(SPACING_SM)
        .style(pick_list_style(picker_colors, RADIUS_MD))
        .menu_style(menu_style(picker_colors, RADIUS_MD, 0.3, 8.0));

        let sort_picker = pick_list(
            SortField::all(),
            Some(self.browse.sort_field),
            Message::SetSortField,
        )
        .padding(SPACING_SM)
        .style(pick_list_style(picker_colors, RADIUS_MD))
        .menu_style(menu_style(picker_colors, RADIUS_MD, 0.3, 8.0));

        let sort_icon = match self.browse.sort_order {
            SortOrder::Ascending => "↑",
            SortOrder::Descending => "↓",
        };
        let sort_btn_bg = self.theme.surface;
        let sort_btn_text = self.theme.text;
        let sort_btn_border = self.theme.border;
        let sort_btn_hover = self.theme.bg_elevated;
        let sort_toggle = button(text(sort_icon).size(16.0))
            .padding(SPACING_SM)
            .on_press(Message::ToggleSortOrder)
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => sort_btn_hover,
                    _ => sort_btn_bg,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: sort_btn_text,
                    border: iced::Border {
                        color: sort_btn_border,
                        width: 1.0,
                        radius: crate::theme::RADIUS_MD.into(),
                    },
                    ..Default::default()
                }
            });

        let view_mode = self.browse.view_mode;
        let view_btn_bg = self.theme.surface;
        let view_btn_active = self.theme.primary;
        let view_btn_text = self.theme.text;
        let view_btn_border = self.theme.border;

        let view_cards_btn = button(Icon::Grid.svg(16.0))
            .padding(SPACING_SM)
            .on_press(Message::SetViewMode(state::ViewMode::Cards))
            .style(move |_theme, _| {
                let bg = if view_mode == state::ViewMode::Cards {
                    view_btn_active
                } else {
                    view_btn_bg
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: view_btn_text,
                    border: iced::Border {
                        color: view_btn_border,
                        width: 1.0,
                        radius: crate::theme::RADIUS_MD.into(),
                    },
                    ..Default::default()
                }
            });

        let view_table_btn = button(Icon::List.svg(16.0))
            .padding(SPACING_SM)
            .on_press(Message::SetViewMode(state::ViewMode::Table))
            .style(move |_theme, _| {
                let bg = if view_mode == state::ViewMode::Table {
                    view_btn_active
                } else {
                    view_btn_bg
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: view_btn_text,
                    border: iced::Border {
                        color: view_btn_border,
                        width: 1.0,
                        radius: crate::theme::RADIUS_MD.into(),
                    },
                    ..Default::default()
                }
            });

        let view_toggle = row![view_cards_btn, view_table_btn].spacing(0);

        let verified_filter = self.browse.verified_only;
        let verified_btn_bg = if verified_filter { self.theme.success } else { self.theme.surface };
        let verified_btn_text = if verified_filter { self.theme.bg_base } else { self.theme.text };
        let verified_btn_border = if verified_filter { self.theme.success } else { self.theme.border };

        let verified_toggle = button(
            row![
                Icon::Check.colored(14.0, verified_btn_text),
                text("Verified").size(14.0).color(verified_btn_text),
            ]
            .spacing(SPACING_XS)
            .align_y(Alignment::Center),
        )
        .padding([SPACING_XS, SPACING_SM])
        .on_press(Message::ToggleVerifiedOnly)
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(verified_btn_bg)),
            text_color: verified_btn_text,
            border: iced::Border {
                color: verified_btn_border,
                width: 1.0,
                radius: crate::theme::RADIUS_MD.into(),
            },
            ..Default::default()
        });

        let refresh_btn_bg = self.theme.surface;
        let refresh_btn_text = self.theme.text;
        let refresh_btn_border = self.theme.border;
        let refresh_btn_hover = self.theme.bg_elevated;
        let is_refreshing = self.browse.refreshing;
        let refresh_btn: Element<Message> = if is_refreshing {
            container(text("↻").size(16.0).color(self.theme.text_muted))
                .padding(SPACING_SM)
                .into()
        } else {
            button(text("↻").size(16.0))
                .padding(SPACING_SM)
                .on_press(Message::RefreshRegistry)
                .style(move |_theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => refresh_btn_hover,
                        _ => refresh_btn_bg,
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: refresh_btn_text,
                        border: iced::Border {
                            color: refresh_btn_border,
                            width: 1.0,
                            radius: crate::theme::RADIUS_MD.into(),
                        },
                        ..Default::default()
                    }
                })
                .into()
        };

        let last_refreshed_text: Element<Message> = match self.browse.last_refreshed {
            Some(instant) => {
                let elapsed = instant.elapsed().as_secs();
                let display = if elapsed < 60 {
                    "just now".to_string()
                } else if elapsed < 3600 {
                    format!("{}m ago", elapsed / 60)
                } else {
                    format!("{}h ago", elapsed / 3600)
                };
                text(display).size(12.0).color(self.theme.text_muted).into()
            }
            None => Space::new().width(0).into(),
        };

        let header = container(
            row![
                search_input,
                Space::new().width(Length::Fill),
                verified_toggle,
                view_toggle,
                text("Sort:").size(14.0).color(self.theme.text_secondary),
                sort_picker,
                sort_toggle,
                category_picker,
                refresh_btn,
                last_refreshed_text,
            ]
            .spacing(SPACING_SM)
            .align_y(Alignment::Center),
        )
        .padding([SPACING_MD, SPACING_LG]);

        let content: Element<Message> = match &self.loading {
            LoadingState::Loading => {
                let theme = self.theme;
                let shimmer_frame = self.spinner_frame;
                let spinner_char = self.spinner_char();
                let text_muted = self.theme.text_muted;

                let loading_indicator = container(
                    row![
                        text(spinner_char).size(14.0).color(text_muted),
                        text("Loading modules...").size(14.0).color(text_muted),
                    ]
                    .spacing(SPACING_SM)
                    .align_y(Alignment::Center),
                )
                .padding([SPACING_SM, SPACING_LG]);

                let skeleton_content = responsive(move |size| {
                    let card_width = calculate_card_width(size.width - 2.0 * SPACING_LG);
                    let skeleton_cards: Vec<Element<Message>> =
                        (0..8).map(|i| skeleton_card(&theme, card_width, shimmer_frame + i * 3)).collect();

                    let grid = Wrap::with_elements(skeleton_cards)
                        .spacing(SPACING_LG)
                        .line_spacing(SPACING_LG)
                        .align_items(Alignment::Start);

                    scrollable(container(grid).padding(SPACING_LG))
                        .height(Length::Fill)
                        .into()
                });

                column![loading_indicator, skeleton_content]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }

            LoadingState::Failed(error) => empty_state_dynamic(
                Icon::Error,
                "Failed to load modules",
                error.clone(),
                &self.theme,
            ),

            LoadingState::Idle => {
                let filtered = self.filtered_modules();

                if filtered.is_empty() {
                    empty_state(
                        Icon::Search,
                        "No modules found",
                        "Try adjusting your search or category filter",
                        &self.theme,
                    )
                } else if self.browse.view_mode == state::ViewMode::Table {
                    let modules: Vec<_> = filtered.into_iter().collect();
                    container(module_table(
                        &modules,
                        &self.installed_uuids,
                        &self.theme,
                        self.browse.sort_field,
                        self.browse.sort_order,
                    ))
                    .padding(SPACING_LG)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    let theme = self.theme;
                    let installed_uuids = self.installed_uuids.clone();
                    let modules: Vec<_> = filtered.into_iter().cloned().collect();

                    responsive(move |size| {
                        let card_width = calculate_card_width(size.width - 2.0 * SPACING_LG);
                        let cards: Vec<Element<Message>> = modules
                            .iter()
                            .map(|m| {
                                let uuid = m.uuid.to_string();
                                let is_installed = installed_uuids.contains(&uuid);
                                module_card(m, is_installed, &theme, card_width)
                            })
                            .collect();

                        let grid = Wrap::with_elements(cards)
                            .spacing(SPACING_LG)
                            .line_spacing(SPACING_LG)
                            .align_items(Alignment::Start);

                        scrollable(container(grid).padding(SPACING_LG))
                            .height(Length::Fill)
                            .into()
                    })
                    .into()
                }
            }
        };

        column![header, content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_installed(&self) -> Element<'_, Message> {
        let module_count = self.installed_modules.len();
        let enabled_count = self.installed_modules.iter().filter(|m| m.enabled).count();

        let search_query = self.installed_search_display();
        let has_search = !search_query.is_empty();

        let text_color = self.theme.text;
        let placeholder_color = self.theme.text_muted;
        let selection_color = iced::Color::from_rgba(
            self.theme.primary.r,
            self.theme.primary.g,
            self.theme.primary.b,
            0.3,
        );
        let surface = self.theme.surface;
        let border_color = self.theme.border;

        let search_input = text_input("Search installed...", search_query)
            .on_input(Message::InstalledSearchChanged)
            .padding([SPACING_SM, SPACING_MD])
            .width(Length::Fixed(250.0))
            .style(move |_, status| iced::widget::text_input::Style {
                background: iced::Background::Color(surface),
                border: iced::Border {
                    color: match status {
                        iced::widget::text_input::Status::Active => border_color,
                        iced::widget::text_input::Status::Hovered => border_color,
                        iced::widget::text_input::Status::Focused { .. } => selection_color,
                        iced::widget::text_input::Status::Disabled => border_color,
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                icon: text_color,
                placeholder: placeholder_color,
                value: text_color,
                selection: selection_color,
            });

        let header = container(
            row![
                column![
                    text("Installed Modules").size(20).color(self.theme.text),
                    text(format!(
                        "{} modules ({} enabled)",
                        module_count, enabled_count
                    ))
                    .size(13)
                    .color(self.theme.text_secondary),
                ]
                .spacing(SPACING_SM / 2.0),
                Space::new().width(Length::Fill),
                search_input,
            ]
            .align_y(Alignment::Center),
        )
        .padding([SPACING_MD, SPACING_LG]);

        let filtered_modules: Vec<_> = self
            .installed_modules
            .iter()
            .filter(|m| {
                if search_query.is_empty() {
                    true
                } else {
                    let query_lower = search_query.to_lowercase();
                    m.waybar_module_name.to_lowercase().contains(&query_lower)
                        || m.uuid.to_string().to_lowercase().contains(&query_lower)
                }
            })
            .collect();

        let rows: Vec<Element<Message>> = filtered_modules
            .iter()
            .map(|m| {
                let uuid = m.uuid.to_string();
                let is_toggling = self.installed.toggling.contains(&uuid);
                let is_uninstalling = self.installed.uninstalling.contains(&uuid);
                module_row(m, is_toggling, is_uninstalling, &self.theme)
            })
            .collect();

        let content: Element<Message> = if self.installed_modules.is_empty() {
            empty_state_with_action(
                Icon::Installed,
                "No modules installed",
                "Browse modules to find and install new ones",
                "Browse Modules",
                Message::Navigate(Screen::Browse),
                &self.theme,
            )
        } else if rows.is_empty() && has_search {
            empty_state_dynamic(
                Icon::Search,
                "No matches found",
                format!("No modules match \"{}\"", search_query),
                &self.theme,
            )
        } else {
            scrollable(
                column(rows)
                    .spacing(SPACING_SM)
                    .padding(SPACING_LG),
            )
            .height(Length::Fill)
            .into()
        };

        column![header, content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_updates(&self) -> Element<'_, Message> {
        let updates = self.modules_with_updates();
        let update_count = updates.len();
        let is_updating_all = self.installed.updating_all;

        let update_all_btn = if update_count > 0 && !is_updating_all {
            let primary = self.theme.primary;
            let primary_hover = darken(primary, 0.1);
            button(
                row![
                    Icon::Updates.svg(14.0),
                    text(format!("Update All ({})", update_count)).size(14.0),
                ]
                .spacing(SPACING_XS)
                .align_y(Alignment::Center),
            )
            .padding([SPACING_SM, SPACING_MD])
            .on_press(Message::UpdateAllModules)
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => primary_hover,
                    _ => primary,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: crate::theme::RADIUS_MD.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
        } else if is_updating_all {
            let surface = self.theme.surface;
            let text_muted = self.theme.text_muted;
            button(
                row![
                    text(self.spinner_char()).size(14.0),
                    text("Updating...").size(14.0),
                ]
                .spacing(SPACING_XS)
                .align_y(Alignment::Center),
            )
            .padding([SPACING_SM, SPACING_MD])
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(surface)),
                text_color: text_muted,
                border: iced::Border {
                    radius: crate::theme::RADIUS_MD.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
        } else {
            let surface = self.theme.surface;
            let text_muted = self.theme.text_muted;
            button(text("No Updates").size(14.0))
                .padding([SPACING_SM, SPACING_MD])
                .style(move |_theme, _status| button::Style {
                    background: Some(iced::Background::Color(surface)),
                    text_color: text_muted,
                    border: iced::Border {
                        radius: crate::theme::RADIUS_MD.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
        };

        let header = container(
            row![
                column![
                    text("Updates").size(20).color(self.theme.text),
                    text(if update_count > 0 {
                        format!("{} update{} available", update_count, if update_count == 1 { "" } else { "s" })
                    } else {
                        "All modules are up to date".to_string()
                    })
                    .size(13)
                    .color(self.theme.text_secondary),
                ]
                .spacing(SPACING_SM / 2.0),
                Space::new().width(Length::Fill),
                update_all_btn,
            ]
            .align_y(Alignment::Center),
        )
        .padding([SPACING_MD, SPACING_LG]);

        let content: Element<Message> = if updates.is_empty() {
            empty_state(
                Icon::Check,
                "No updates available",
                "All your modules are up to date",
                &self.theme,
            )
        } else {
            let rows: Vec<Element<Message>> = updates
                .iter()
                .map(|m| {
                    let uuid = m.uuid.to_string();
                    let current_ver = m.version.to_string();
                    let new_ver = m.registry_version.as_ref().map(|v| v.to_string()).unwrap_or_default();
                    let is_updating = self.installed.updating.contains(&uuid);

                    let theme = self.theme;
                    let primary = theme.primary;
                    let primary_hover = darken(primary, 0.1);

                    let update_btn: Element<Message> = if is_updating {
                        let surface = theme.surface;
                        let text_muted = theme.text_muted;
                        button(
                            row![
                                text(self.spinner_char()).size(12.0),
                                text("Updating").size(12.0),
                            ]
                            .spacing(SPACING_XS)
                            .align_y(Alignment::Center),
                        )
                        .padding([SPACING_XS, SPACING_SM])
                        .style(move |_theme, _status| button::Style {
                            background: Some(iced::Background::Color(surface)),
                            text_color: text_muted,
                            border: iced::Border {
                                radius: crate::theme::RADIUS_SM.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .into()
                    } else {
                        let uuid_clone = uuid.clone();
                        button(text("Update").size(12.0))
                            .padding([SPACING_XS, SPACING_SM])
                            .on_press(Message::UpdateModule(uuid_clone))
                            .style(move |_theme, status| {
                                let bg = match status {
                                    button::Status::Hovered | button::Status::Pressed => primary_hover,
                                    _ => primary,
                                };
                                button::Style {
                                    background: Some(iced::Background::Color(bg)),
                                    text_color: iced::Color::WHITE,
                                    border: iced::Border {
                                        radius: crate::theme::RADIUS_SM.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            })
                            .into()
                    };

                    container(
                        row![
                            column![
                                text(&m.waybar_module_name).size(14.0).color(theme.text),
                                text(format!("{} → {}", current_ver, new_ver))
                                    .size(12.0)
                                    .color(theme.text_secondary),
                            ]
                            .spacing(SPACING_SM / 2.0),
                            Space::new().width(Length::Fill),
                            update_btn,
                        ]
                        .align_y(Alignment::Center)
                        .padding(SPACING_MD),
                    )
                    .style(move |_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme.surface)),
                        border: iced::Border {
                            color: theme.border,
                            width: 1.0,
                            radius: crate::theme::RADIUS_MD.into(),
                        },
                        ..Default::default()
                    })
                    .into()
                })
                .collect();

            scrollable(
                column(rows)
                    .spacing(SPACING_SM)
                    .padding(SPACING_LG),
            )
            .height(Length::Fill)
            .into()
        };

        column![header, content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn theme(&self) -> Theme {
        if self.system_is_dark && matches!(self.theme_mode, crate::theme::ThemeMode::System)
            || matches!(self.theme_mode, crate::theme::ThemeMode::Dark)
        {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        use iced::event::{self, Event};
        use iced::keyboard;
        use iced::keyboard::key::Named;
        use iced::time;
        use std::time::Duration;

        let keyboard_sub = event::listen_with(|event, _status, _id| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Escape),
                ..
            }) => Some(Message::EscapePressed),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) => {
                if modifiers.control() && c.as_str() == "f" {
                    Some(Message::FocusSearch)
                } else if !modifiers.control() && !modifiers.alt() {
                    match c.as_str() {
                        "1" => Some(Message::Navigate(Screen::Browse)),
                        "2" => Some(Message::Navigate(Screen::Installed)),
                        "3" => Some(Message::Navigate(Screen::Updates)),
                        "4" => Some(Message::Navigate(Screen::Settings)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        });

        let omarchy_watcher = if self.theme_mode == crate::theme::ThemeMode::Omarchy {
            tasks::watch_omarchy_theme()
        } else {
            iced::Subscription::none()
        };

        iced::Subscription::batch([
            keyboard_sub,
            omarchy_watcher,
            time::every(Duration::from_millis(50)).map(|_| Message::Tick),
        ])
    }
}

fn calculate_card_width(available_width: f32) -> f32 {
    let min_card_width = 280.0;
    let max_card_width = CARD_WIDTH;
    let columns = ((available_width + SPACING_LG) / (min_card_width + SPACING_LG))
        .floor()
        .max(1.0);
    let card_width = (available_width - SPACING_LG * (columns - 1.0)) / columns;
    card_width.clamp(min_card_width, max_card_width)
}
