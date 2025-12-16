pub mod message;
pub mod state;

use std::time::Duration;

use iced::widget::{button, column, container, pick_list, responsive, row, scrollable, stack, text, text_input, Space};
use iced::{Alignment, Element, Length, Task, Theme};
use iced_aw::Wrap;

use crate::icons::Icon;
use crate::tasks;
use crate::theme::{CARD_WIDTH, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS};

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
            Message::Navigate(screen) => {
                if let Screen::ModuleDetail(ref uuid) = screen {
                    self.module_detail.screenshot = state::ScreenshotState::Loading;
                    self.module_detail.installing = false;
                    self.screen = screen.clone();

                    if let Some(registry) = &self.registry
                        && let Some(module) = registry.modules.iter().find(|m| m.uuid.to_string() == *uuid)
                        && let Some(screenshot_url) = &module.screenshot
                    {
                        return tasks::load_screenshot(screenshot_url.clone());
                    }
                    self.module_detail.screenshot = state::ScreenshotState::NotLoaded;
                    return Task::none();
                }
                self.screen = screen;
                Task::none()
            }

            Message::NavigateBack => {
                self.screen = Screen::Browse;
                self.module_detail.screenshot = state::ScreenshotState::NotLoaded;
                self.module_detail.installing = false;
                Task::none()
            }

            Message::SearchChanged(query) => {
                self.browse.pending_search = Some(query);
                self.browse.search_debounce_start = Some(std::time::Instant::now());
                Task::none()
            }

            Message::CategorySelected(filter) => {
                self.browse.selected_category = filter;
                Task::none()
            }

            Message::SetSortField(field) => {
                self.browse.sort_field = field;
                self.save_settings();
                Task::none()
            }

            Message::ToggleSortOrder => {
                self.browse.sort_order = self.browse.sort_order.toggle();
                self.save_settings();
                Task::none()
            }

            Message::SetViewMode(mode) => {
                self.browse.view_mode = mode;
                self.save_settings();
                Task::none()
            }

            Message::ToggleVerifiedOnly => {
                self.browse.verified_only = !self.browse.verified_only;
                Task::none()
            }

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
                    && let Some(module) = registry.modules.iter().find(|m| m.uuid.to_string() == uuid)
                {
                    return tasks::install_module(
                        uuid,
                        module.name.clone(),
                        module.version.clone(),
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
                self.installed.pending_search = Some(query);
                self.installed.search_debounce_start = Some(std::time::Instant::now());
                Task::none()
            }

            Message::ClearInstalledSearch => {
                self.installed.search_query.clear();
                self.installed.pending_search = None;
                self.installed.search_debounce_start = None;
                Task::none()
            }

            Message::RegistryLoaded(result) => {
                match result {
                    Ok(index) => {
                        self.sync_registry_versions(&index);
                        self.registry = Some(index);
                        self.loading = LoadingState::Idle;
                    }
                    Err(e) => {
                        self.loading = LoadingState::Failed(e);
                    }
                }
                Task::none()
            }

            Message::InstalledLoaded(result) => {
                match result {
                    Ok(modules) => {
                        self.installed_uuids =
                            modules.iter().map(|m| m.uuid.to_string()).collect();
                        self.installed_modules = modules;
                    }
                    Err(e) => {
                        self.push_notification(
                            format!("Failed to load installed modules: {e}"),
                            state::NotificationKind::Error,
                        );
                    }
                }
                Task::none()
            }

            Message::InstallCompleted(result) => {
                self.module_detail.installing = false;
                match result {
                    Ok(module) => {
                        self.installed_uuids.insert(module.uuid.to_string());
                        self.installed_modules.push(module);
                        self.push_notification(
                            "Module installed successfully".to_string(),
                            state::NotificationKind::Success,
                        );
                    }
                    Err(e) => {
                        self.push_notification(
                            format!("Installation failed: {e}"),
                            state::NotificationKind::Error,
                        );
                    }
                }
                Task::none()
            }

            Message::ToggleCompleted(result) => {
                match result {
                    Ok(uuid) => {
                        self.installed.toggling.remove(&uuid);
                        if let Some(m) = self
                            .installed_modules
                            .iter_mut()
                            .find(|m| m.uuid.to_string() == uuid)
                        {
                            m.enabled = !m.enabled;
                        }
                    }
                    Err((uuid, e)) => {
                        self.installed.toggling.remove(&uuid);
                        self.push_notification(format!("Toggle failed: {e}"), state::NotificationKind::Error);
                    }
                }
                Task::none()
            }

            Message::UninstallCompleted(result) => {
                match result {
                    Ok(uuid) => {
                        self.installed.uninstalling.remove(&uuid);
                        self.installed_uuids.remove(&uuid);
                        self.installed_modules.retain(|m| m.uuid.to_string() != uuid);
                        self.push_notification("Module uninstalled".to_string(), state::NotificationKind::Success);
                    }
                    Err(e) => {
                        self.push_notification(format!("Uninstall failed: {e}"), state::NotificationKind::Error);
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

            Message::ScreenshotLoaded(result) => {
                self.module_detail.screenshot = match result {
                    Ok(handle) => state::ScreenshotState::Loaded(handle),
                    Err(_) => state::ScreenshotState::Failed,
                };
                Task::none()
            }

            Message::DetailInstallModule => {
                if let Screen::ModuleDetail(ref uuid) = self.screen {
                    self.module_detail.installing = true;
                    let uuid_clone = uuid.clone();
                    return Task::done(Message::InstallModule(uuid_clone));
                }
                Task::none()
            }

            Message::OpenRepoUrl(url) => {
                let _ = open::that(&url);
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
                    let _ = crate::services::save_preferences(&uuid, &self.preferences.values);
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
                    let _ = crate::services::save_preferences(&uuid, &defaults);
                    self.push_notification(
                        "Preferences reset to defaults".to_string(),
                        state::NotificationKind::Success,
                    );
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
            self.omarchy_palette.is_some(),
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
            && let Some(module) = registry.modules.iter().find(|m| m.uuid.to_string() == uuid)
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
                self.waybar_version.as_deref(),
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
        .width(Length::Fixed(320.0))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(surface)),
            border: iced::Border {
                color: iced::Color::from_rgba(primary.r, primary.g, primary.b, 0.3),
                width: 1.0,
                radius: crate::theme::RADIUS_MD.into(),
            },
            ..Default::default()
        });

        let picker_surface = self.theme.surface;
        let picker_text = self.theme.text;
        let picker_text_secondary = self.theme.text_secondary;
        let picker_border = self.theme.border;
        let picker_primary = self.theme.primary;
        let menu_surface = self.theme.surface;
        let menu_text = self.theme.text;
        let menu_selected_bg = self.theme.primary;
        let menu_border = self.theme.border;
        let category_picker = pick_list(
            CategoryFilter::all(),
            Some(self.browse.selected_category),
            Message::CategorySelected,
        )
        .padding(SPACING_SM)
        .style(move |_theme, status| {
            let border_color = match status {
                iced::widget::pick_list::Status::Active => picker_border,
                iced::widget::pick_list::Status::Hovered
                | iced::widget::pick_list::Status::Opened { .. } => picker_primary,
            };
            iced::widget::pick_list::Style {
                text_color: picker_text,
                placeholder_color: picker_text_secondary,
                handle_color: picker_text_secondary,
                background: iced::Background::Color(picker_surface),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: crate::theme::RADIUS_MD.into(),
                },
            }
        })
        .menu_style(move |_theme| {
            iced::overlay::menu::Style {
                background: iced::Background::Color(menu_surface),
                border: iced::Border {
                    color: menu_border,
                    width: 1.0,
                    radius: crate::theme::RADIUS_MD.into(),
                },
                text_color: menu_text,
                selected_text_color: iced::Color::WHITE,
                selected_background: iced::Background::Color(menu_selected_bg),
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 8.0,
                },
            }
        });

        let sort_picker = pick_list(
            SortField::all(),
            Some(self.browse.sort_field),
            Message::SetSortField,
        )
        .padding(SPACING_SM)
        .style(move |_theme, status| {
            let border_color = match status {
                iced::widget::pick_list::Status::Active => picker_border,
                iced::widget::pick_list::Status::Hovered
                | iced::widget::pick_list::Status::Opened { .. } => picker_primary,
            };
            iced::widget::pick_list::Style {
                text_color: picker_text,
                placeholder_color: picker_text_secondary,
                handle_color: picker_text_secondary,
                background: iced::Background::Color(picker_surface),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: crate::theme::RADIUS_MD.into(),
                },
            }
        })
        .menu_style(move |_theme| {
            iced::overlay::menu::Style {
                background: iced::Background::Color(menu_surface),
                border: iced::Border {
                    color: menu_border,
                    width: 1.0,
                    radius: crate::theme::RADIUS_MD.into(),
                },
                text_color: menu_text,
                selected_text_color: iced::Color::WHITE,
                selected_background: iced::Background::Color(menu_selected_bg),
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 8.0,
                },
            }
        });

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
                        self.waybar_version.as_deref(),
                    ))
                    .padding(SPACING_LG)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    let theme = self.theme;
                    let installed_uuids = self.installed_uuids.clone();
                    let modules: Vec<_> = filtered.into_iter().cloned().collect();
                    let waybar_version = self.waybar_version.clone();

                    responsive(move |size| {
                        let card_width = calculate_card_width(size.width - 2.0 * SPACING_LG);
                        let wb_ver = waybar_version.as_deref();
                        let cards: Vec<Element<Message>> = modules
                            .iter()
                            .map(|m| {
                                let uuid = m.uuid.to_string();
                                let is_installed = installed_uuids.contains(&uuid);
                                module_card(m, is_installed, &theme, card_width, wb_ver)
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

        let header = container(
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
                    let current_ver = m.version.to_string();
                    let new_ver = m.registry_version.as_ref().map(|v| v.to_string()).unwrap_or_default();

                    let theme = self.theme;
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
                            text("Update available")
                                .size(12.0)
                                .color(theme.warning),
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
