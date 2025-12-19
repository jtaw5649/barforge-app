pub mod handlers;
pub mod message;
pub mod state;

use iced::widget::{
    Space, button, column, container, pick_list, responsive, row, scrollable, stack, text,
    text_input,
};
use iced::{Alignment, Element, Length, Task, Theme};
use iced_aw::Wrap;

use crate::icons::Icon;
use crate::services::is_omarchy_available;
use crate::tasks;
use crate::theme::{
    CARD_WIDTH, PickListColors, RADIUS_MD, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, darken,
    menu_style, pick_list_style,
};
use crate::widget::{
    confirmation_dialog, empty_state, empty_state_dynamic, empty_state_with_action, module_card,
    module_detail_screen, module_row, module_table, notification_toast, preferences_modal,
    settings_screen, sidebar, skeleton_card,
};

pub use message::Message;
pub use state::{
    App, AuthorLoadingState, CategoryFilter, LoadingState, Screen, ScreenshotState, SortField,
    SortOrder, ViewMode,
};

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

            Message::InstallModule(uuid) => handlers::handle_install_module(self, uuid),

            Message::ToggleModule { uuid, enabled } => {
                handlers::handle_toggle_module(self, uuid, enabled)
            }

            Message::SetModulePosition { uuid, section } => {
                handlers::handle_set_module_position(uuid, section)
            }

            Message::PositionChanged(result) => handlers::handle_position_changed(self, result),

            Message::UninstallModule(uuid) => handlers::handle_uninstall_module(self, uuid),

            Message::OpenPreferences(uuid) => handlers::handle_open_preferences(self, uuid),

            Message::InstalledSearchChanged(query) => {
                handlers::handle_installed_search_changed(self, query)
            }

            Message::ClearInstalledSearch => handlers::handle_clear_installed_search(self),

            Message::RefreshRegistry => handlers::handle_refresh_registry(self),

            Message::RegistryLoaded(result) => handlers::handle_registry_loaded(self, result),

            Message::RegistryRefreshed(result) => handlers::handle_registry_refreshed(self, result),

            Message::InstalledLoaded(result) => handlers::handle_installed_loaded(self, result),

            Message::InstallCompleted(result) => handlers::handle_install_completed(self, result),

            Message::ToggleCompleted(result) => handlers::handle_toggle_completed(self, result),

            Message::UninstallCompleted(result) => {
                handlers::handle_uninstall_completed(self, result)
            }

            Message::UpdateModule(uuid) => handlers::handle_update_module(self, uuid),

            Message::UpdateAllModules => handlers::handle_update_all_modules(self),

            Message::UpdateCompleted(result) => handlers::handle_update_completed(self, result),

            Message::UpdateAllCompleted(result) => {
                handlers::handle_update_all_completed(self, result)
            }

            Message::ShowNotification(message, kind) => {
                handlers::handle_show_notification(self, message, kind);
                Task::none()
            }

            Message::DismissNotification => handlers::handle_dismiss_notification(self),

            Message::Tick => handlers::handle_tick(self),

            Message::SystemThemeChanged(is_dark) => {
                handlers::handle_system_theme_changed(self, is_dark)
            }

            Message::SetThemeMode(mode) => handlers::handle_set_theme_mode(self, mode),

            Message::OmarchyThemeChanged => handlers::handle_omarchy_theme_changed(self),

            Message::ScreenshotLoaded(result) => handlers::handle_screenshot_loaded(self, result),

            Message::DetailInstallModule => handlers::handle_detail_install_module(self),

            Message::OpenRepoUrl(url) => {
                handlers::handle_open_repo_url(self, url);
                Task::none()
            }

            Message::RequestConfirmation(action) => {
                handlers::handle_request_confirmation(self, action);
                Task::none()
            }

            Message::ConfirmAction => handlers::handle_confirm_action(self),

            Message::CancelConfirmation => {
                handlers::handle_cancel_confirmation(self);
                Task::none()
            }

            Message::ClearCache => handlers::handle_clear_cache(),

            Message::CacheClearCompleted(result) => {
                handlers::handle_cache_clear_completed(self, result);
                Task::none()
            }

            Message::ResetSettings => handlers::handle_reset_settings(),

            Message::SettingsResetCompleted(result) => {
                handlers::handle_settings_reset_completed(self, result);
                Task::none()
            }

            Message::ToggleTray(enabled) => handlers::handle_toggle_tray(self, enabled),

            Message::FocusSearch => handlers::handle_focus_search(self),

            Message::EscapePressed => handlers::handle_escape_pressed(self),

            Message::PreferenceChanged(uuid, key, value) => {
                handlers::handle_preference_changed(self, uuid, key, value)
            }

            Message::ClosePreferences => handlers::handle_close_preferences(self),

            Message::ResetPreferences(uuid) => handlers::handle_reset_preferences(self, uuid),

            Message::TrayShowWindow => handlers::handle_tray_show_window(),

            Message::TrayCheckUpdates => handlers::handle_tray_check_updates(self),

            Message::TrayQuit => handlers::handle_tray_quit(),

            Message::InstallProgress { uuid, stage } => {
                handlers::handle_install_progress(self, uuid, stage)
            }

            Message::DependencyCheckCompleted(result) => {
                handlers::handle_dependency_check_completed(self, result)
            }

            Message::RevocationCheckCompleted(result) => {
                handlers::handle_revocation_check_completed(self, result)
            }

            Message::SignatureVerified(result) => handlers::handle_signature_verified(self, result),

            Message::SandboxStatusChanged(status) => {
                handlers::handle_sandbox_status_changed(self, status)
            }

            Message::AuthorClicked(username) => handlers::handle_author_clicked(self, username),

            Message::AuthorLoaded(result) => handlers::handle_author_loaded(self, result),

            Message::ModuleReviewsLoaded(result) => {
                handlers::handle_module_reviews_loaded(self, result)
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
            Screen::AuthorProfile(username) => self.view_author_profile(username),
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
                self.notifications
                    .iter()
                    .map(|notif| notification_toast(notif, &self.theme)),
            )
            .spacing(SPACING_SM);

            container(notification_stack)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(SPACING_LG)
                .align_x(Alignment::End)
                .align_y(Alignment::End)
                .into()
        };

        let confirmation_overlay: Element<Message> =
            if let Some(action) = &self.confirmation.pending_action {
                confirmation_dialog(action, &self.theme)
            } else {
                Space::new().into()
            };

        let preferences_overlay: Element<Message> = if let (Some(uuid), Some(schema)) =
            (&self.preferences.open_for, &self.preferences.schema)
        {
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

        stack![
            main_layout,
            notification_overlay,
            confirmation_overlay,
            preferences_overlay
        ]
        .into()
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
                &self.module_detail.reviews,
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

    fn view_author_profile(&self, username: &str) -> Element<'_, Message> {
        use crate::app::state::AuthorLoadingState;

        match &self.author_profile.loading {
            AuthorLoadingState::Loading | AuthorLoadingState::NotLoaded => {
                let text_color = self.theme.text_muted;
                container(
                    text(format!("Loading profile for {}...", username))
                        .size(16)
                        .color(text_color),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            }
            AuthorLoadingState::Loaded(profile) => {
                let author = &profile.author;
                let theme = &self.theme;

                let display_name = author.display();
                let header = column![
                    text(display_name).size(28).color(theme.text_normal),
                    text(format!("@{}", author.username))
                        .size(14)
                        .color(theme.text_muted),
                ]
                .spacing(4);

                let bio = author
                    .bio
                    .as_deref()
                    .map(|bio_text| text(bio_text).size(14).color(theme.text_normal));

                let member_since = text(format!("Member since {}", author.member_since()))
                    .size(12)
                    .color(theme.text_muted);

                let module_count = text(format!("{} modules published", author.module_count))
                    .size(12)
                    .color(theme.text_muted);

                let mut content = column![header].spacing(SPACING_MD);

                if let Some(bio_text) = bio {
                    content = content.push(bio_text);
                }

                content = content.push(member_since).push(module_count);

                if !profile.modules.is_empty() {
                    let modules_header =
                        text("Published Modules").size(18).color(theme.text_normal);

                    let module_list: Element<'_, Message> = column(
                        profile
                            .modules
                            .iter()
                            .map(|m| {
                                let bg = theme.surface;
                                let name_color = theme.text_normal;
                                let desc_color = theme.text_muted;
                                container(
                                    column![
                                        text(&m.name).size(14).color(name_color),
                                        text(&m.description).size(12).color(desc_color),
                                    ]
                                    .spacing(4),
                                )
                                .padding(SPACING_SM)
                                .style(move |_| iced::widget::container::Style {
                                    background: Some(iced::Background::Color(bg)),
                                    border: iced::Border::default()
                                        .rounded(RADIUS_MD)
                                        .color(iced::Color::TRANSPARENT),
                                    ..Default::default()
                                })
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .spacing(SPACING_SM)
                    .into();

                    content = content
                        .push(Space::new().height(SPACING_MD))
                        .push(modules_header)
                        .push(module_list);
                }

                let bg = theme.background;
                scrollable(container(content).padding(SPACING_LG).style(move |_| {
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(bg)),
                        ..Default::default()
                    }
                }))
                .into()
            }
            AuthorLoadingState::Failed(error) => empty_state_dynamic(
                Icon::Error,
                "Failed to load profile",
                error.clone(),
                &self.theme,
            ),
        }
    }

    fn view_browse(&self) -> Element<'_, Message> {
        let search_icon = Icon::Search.svg(16.0);
        let text_color = self.theme.text_normal;
        let placeholder_color = self.theme.text_muted;
        let selection_color = iced::Color::from_rgba(
            self.theme.primary.r,
            self.theme.primary.g,
            self.theme.primary.b,
            0.3,
        );
        let search_field = text_input("Search modules...", self.browse_search_display())
            .on_input(Message::SearchChanged)
            .padding([crate::theme::SPACE_SM, crate::theme::SPACE_SM])
            .width(Length::Fill)
            .style(move |_theme, _status| iced::widget::text_input::Style {
                background: iced::Background::Color(iced::Color::TRANSPARENT),
                border: iced::Border::default(),
                icon: text_color,
                placeholder: placeholder_color,
                value: text_color,
                selection: selection_color,
            });

        let search_input = container(
            row![search_icon, search_field]
                .spacing(crate::theme::SPACE_SM)
                .align_y(Alignment::Center),
        )
        .padding([crate::theme::SPACE_XS, crate::theme::SPACE_MD])
        .width(Length::Fixed(crate::theme::SEARCH_PANEL_WIDTH))
        .style(crate::theme::container::search_bar(self.theme));

        let picker_colors = PickListColors::from_theme(&self.theme);

        let category_picker = container(
            pick_list(
                CategoryFilter::all(),
                Some(self.browse.selected_category),
                Message::CategorySelected,
            )
            .padding(crate::theme::SPACE_SM)
            .style(pick_list_style(picker_colors, RADIUS_MD))
            .menu_style(menu_style(picker_colors, RADIUS_MD, 0.3, 8.0)),
        )
        .width(Length::Fixed(140.0));

        let sort_picker = container(
            pick_list(
                SortField::all(),
                Some(self.browse.sort_field),
                Message::SetSortField,
            )
            .padding(crate::theme::SPACE_SM)
            .style(pick_list_style(picker_colors, RADIUS_MD))
            .menu_style(menu_style(picker_colors, RADIUS_MD, 0.3, 8.0)),
        )
        .width(Length::Fixed(140.0));

        let sort_icon = match self.browse.sort_order {
            SortOrder::Ascending => Icon::ArrowUp,
            SortOrder::Descending => Icon::ArrowDown,
        };

        let sort_toggle = button(sort_icon.colored(16.0, self.theme.text_normal))
            .padding(crate::theme::SPACE_SM)
            .on_press(Message::ToggleSortOrder)
            .style(crate::theme::button::secondary(self.theme));

        let view_mode = self.browse.view_mode;

        let view_cards_btn = button(Icon::Grid.colored(
            16.0,
            if view_mode == state::ViewMode::Cards {
                iced::Color::WHITE
            } else {
                self.theme.text_normal
            },
        ))
        .padding(crate::theme::SPACE_SM)
        .on_press(Message::SetViewMode(state::ViewMode::Cards))
        .style(if view_mode == state::ViewMode::Cards {
            crate::theme::button::primary(self.theme)
        } else {
            crate::theme::button::secondary(self.theme)
        });

        let view_table_btn = button(Icon::List.colored(
            16.0,
            if view_mode == state::ViewMode::Table {
                iced::Color::WHITE
            } else {
                self.theme.text_normal
            },
        ))
        .padding(crate::theme::SPACE_SM)
        .on_press(Message::SetViewMode(state::ViewMode::Table))
        .style(if view_mode == state::ViewMode::Table {
            crate::theme::button::primary(self.theme)
        } else {
            crate::theme::button::secondary(self.theme)
        });

        let view_toggle = row![view_cards_btn, view_table_btn].spacing(crate::theme::SPACE_XS);

        let verified_filter = self.browse.verified_only;

        let verified_toggle = button(
            row![
                Icon::Check.colored(
                    14.0,
                    if verified_filter {
                        iced::Color::WHITE
                    } else {
                        self.theme.text_muted
                    }
                ),
                text("Verified").size(14.0).color(if verified_filter {
                    iced::Color::WHITE
                } else {
                    self.theme.text_muted
                }),
            ]
            .spacing(crate::theme::SPACE_XS)
            .align_y(Alignment::Center),
        )
        .padding([crate::theme::SPACE_XS, crate::theme::SPACE_MD])
        .on_press(Message::ToggleVerifiedOnly)
        .style(if verified_filter {
            crate::theme::button::primary(self.theme)
        } else {
            crate::theme::button::secondary(self.theme)
        });

        let is_refreshing = self.browse.refreshing;
        let refresh_btn: Element<Message> = if is_refreshing {
            container(text("...").size(16.0).color(self.theme.text_muted))
                .padding(crate::theme::SPACE_SM)
                .into()
        } else {
            button(Icon::Updates.colored(16.0, self.theme.text_normal))
                .padding(crate::theme::SPACE_SM)
                .on_press(Message::RefreshRegistry)
                .style(crate::theme::button::ghost(self.theme))
                .into()
        };

        let last_refreshed_text: Element<Message> = match self.browse.last_refreshed {
            Some(instant) => {
                let elapsed = instant.elapsed().as_secs();
                let display = if elapsed < 60 {
                    "Just now".to_string()
                } else if elapsed < 3600 {
                    format!("{}m ago", elapsed / 60)
                } else {
                    format!("{}h ago", elapsed / 3600)
                };
                text(display).size(12.0).color(self.theme.text_muted).into()
            }
            None => Space::new().width(0).into(),
        };

        let controls_row = row![
            category_picker,
            verified_toggle,
            Space::new().width(crate::theme::SPACE_MD),
            sort_picker,
            sort_toggle,
            Space::new().width(crate::theme::SPACE_MD),
            view_toggle,
            Space::new().width(crate::theme::SPACE_MD),
            refresh_btn,
            last_refreshed_text,
        ]
        .spacing(crate::theme::SPACE_SM)
        .align_y(Alignment::Center);

        let header = container(
            row![search_input, Space::new().width(Length::Fill), controls_row]
                .align_y(Alignment::Center),
        )
        .padding([crate::theme::SPACE_MD, crate::theme::SPACE_LG]);

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
                    let skeleton_cards: Vec<Element<Message>> = (0..8)
                        .map(|i| skeleton_card(&theme, card_width, shimmer_frame + i * 3))
                        .collect();

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
            scrollable(column(rows).spacing(SPACING_SM).padding(SPACING_LG))
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
                        format!(
                            "{} update{} available",
                            update_count,
                            if update_count == 1 { "" } else { "s" }
                        )
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
                    let uuid_str = m.uuid.to_string();
                    let uuid_module = m.uuid.clone();
                    let current_ver = m.version.to_string();
                    let new_ver = m
                        .registry_version
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_default();
                    let is_updating = self.installed.updating.contains(&uuid_str);

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
                        button(text("Update").size(12.0))
                            .padding([SPACING_XS, SPACING_SM])
                            .on_press(Message::UpdateModule(uuid_module))
                            .style(move |_theme, status| {
                                let bg = match status {
                                    button::Status::Hovered | button::Status::Pressed => {
                                        primary_hover
                                    }
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

            scrollable(column(rows).spacing(SPACING_SM).padding(SPACING_LG))
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
