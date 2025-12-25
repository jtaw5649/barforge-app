use iced::widget::{Space, button, column, container, row, text};
use iced::{Background, Border, Element, Length, Vector};

use crate::app::message::Message;
use crate::app::state::Screen;
use crate::icons::{Icon, app_logo, omarchy_icon};
use crate::theme::{
    AppTheme, FONT_LG, FONT_SM, FONT_XS, ICON_LG, ICON_MD, ICON_SM, SIDEBAR_WIDTH, SPACE_MD,
    SPACE_SM, SPACE_XL, SPACE_XS, ThemeMode, button as btn_style,
};

pub fn sidebar(
    current: &Screen,
    installed_count: usize,
    update_count: usize,
    theme: &AppTheme,
    theme_mode: ThemeMode,
    omarchy_available: bool,
) -> Element<'static, Message> {
    let theme_copy = *theme;

    let marketplace_label = text("MARKETPLACE")
        .size(FONT_XS)
        .color(theme.text_faint)
        .font(iced::Font::MONOSPACE);

    let browse_btn = {
        let is_active = matches!(current, Screen::Browse);
        let color = if is_active {
            theme.text_normal
        } else {
            theme.text_muted
        };
        button(
            row![
                Icon::Browse.colored(ICON_MD, color),
                text("Browse").size(FONT_SM).color(color)
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::Navigate(Screen::Browse))
        .style(if is_active {
            btn_style::sidebar_active(theme_copy)
        } else {
            btn_style::sidebar(theme_copy)
        })
        .width(Length::Fill)
        .padding([SPACE_MD, SPACE_SM])
    };

    let library_label = text("LIBRARY")
        .size(FONT_XS)
        .color(theme.text_faint)
        .font(iced::Font::MONOSPACE);

    let installed_btn = {
        let is_active = matches!(current, Screen::Installed);
        let color = if is_active {
            theme.text_normal
        } else {
            theme.text_muted
        };
        let content = if installed_count > 0 {
            row![
                Icon::Installed.colored(ICON_MD, color),
                text("Installed").size(FONT_SM).color(color),
                Space::new().width(Length::Fill),
                container(
                    text(installed_count.to_string())
                        .size(FONT_XS)
                        .color(theme.text_normal)
                )
                .padding([2.0, 8.0])
                .style(move |_| container::Style {
                    background: Some(Background::Color(theme_copy.bg_elevated)),
                    border: Border {
                        radius: 10.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center)
        } else {
            row![
                Icon::Installed.colored(ICON_MD, color),
                text("Installed").size(FONT_SM).color(color)
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center)
        };

        button(content)
            .on_press(Message::Navigate(Screen::Installed))
            .style(if is_active {
                btn_style::sidebar_active(theme_copy)
            } else {
                btn_style::sidebar(theme_copy)
            })
            .width(Length::Fill)
            .padding([SPACE_MD, SPACE_SM])
    };

    let updates_btn = {
        let is_active = matches!(current, Screen::Updates);
        let color = if is_active {
            theme.text_normal
        } else {
            theme.text_muted
        };
        let content = if update_count > 0 {
            row![
                Icon::Updates.colored(ICON_MD, color),
                text("Updates").size(FONT_SM).color(color),
                Space::new().width(Length::Fill),
                container(
                    text(update_count.to_string())
                        .size(FONT_XS)
                        .color(theme.bg_base)
                )
                .padding([2.0, 8.0])
                .style(move |_| container::Style {
                    background: Some(Background::Color(theme_copy.warning)),
                    border: Border {
                        radius: 10.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center)
        } else {
            row![
                Icon::Updates.colored(ICON_MD, color),
                text("Updates").size(FONT_SM).color(color)
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center)
        };

        button(content)
            .on_press(Message::Navigate(Screen::Updates))
            .style(if is_active {
                btn_style::sidebar_active(theme_copy)
            } else {
                btn_style::sidebar(theme_copy)
            })
            .width(Length::Fill)
            .padding([SPACE_MD, SPACE_SM])
    };

    let nav = column![
        marketplace_label,
        Space::new().height(SPACE_XS),
        browse_btn,
        Space::new().height(SPACE_XL),
        library_label,
        Space::new().height(SPACE_XS),
        installed_btn,
        updates_btn
    ]
    .padding([0.0, SPACE_SM]);

    let header = container(
        row![
            app_logo(ICON_LG),
            text("Barforge").size(FONT_LG).color(theme.text_normal),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
    )
    .padding([SPACE_XL, SPACE_MD]);

    let theme_icon_btn = |mode: ThemeMode, icon: Icon| {
        let is_active = theme_mode == mode;
        let icon_color = if is_active {
            theme_copy.accent
        } else {
            theme_copy.text_muted
        };
        button(container(icon.colored(ICON_SM, icon_color)).center_x(ICON_SM + SPACE_SM * 2.0))
            .on_press(Message::SetThemeMode(mode))
            .padding([SPACE_XS, SPACE_SM])
            .style(if is_active {
                btn_style::theme_active(theme_copy)
            } else {
                btn_style::theme_inactive(theme_copy)
            })
    };

    let omarchy_btn = |is_active: bool| {
        button(container(omarchy_icon(ICON_SM)).center_x(ICON_SM + SPACE_SM * 2.0))
            .on_press(Message::SetThemeMode(ThemeMode::Omarchy))
            .padding([SPACE_XS, SPACE_SM])
            .style(if is_active {
                btn_style::theme_active(theme_copy)
            } else {
                btn_style::theme_inactive(theme_copy)
            })
    };

    let theme_toggles: Element<Message> = {
        let mut modes: Vec<Element<Message>> = vec![
            theme_icon_btn(ThemeMode::Light, Icon::Sun).into(),
            theme_icon_btn(ThemeMode::Dark, Icon::Moon).into(),
        ];
        if omarchy_available {
            modes.push(omarchy_btn(theme_mode == ThemeMode::Omarchy).into());
        }
        row(modes)
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center)
            .into()
    };

    let is_settings_active = matches!(current, Screen::Settings);
    let settings_icon_color = if is_settings_active {
        theme.accent
    } else {
        theme.text_normal
    };
    let settings_btn = button(Icon::Settings.colored(ICON_SM, settings_icon_color))
        .on_press(Message::Navigate(Screen::Settings))
        .padding([SPACE_XS, SPACE_SM])
        .style(if is_settings_active {
            btn_style::sidebar_active(theme_copy)
        } else {
            btn_style::sidebar(theme_copy)
        });

    let settings_row = container(
        row![
            text("Settings").size(FONT_XS).color(theme.text_muted),
            Space::new().width(Length::Fill),
            settings_btn
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([SPACE_XS, SPACE_MD]);

    let theme_row = container(theme_toggles)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .padding([SPACE_XS, SPACE_SM]);

    let version = container(
        text(format!("v{}", env!("CARGO_PKG_VERSION")))
            .size(FONT_XS)
            .color(theme.text_faint),
    )
    .padding([SPACE_SM, SPACE_MD]);

    container(column![
        header,
        nav,
        Space::new().height(Length::Fill),
        settings_row,
        theme_row,
        version
    ])
    .style(move |_| iced::widget::container::Style {
        background: Some(Background::Color(theme_copy.sidebar_bg)),
        border: Border {
            color: theme_copy.border_subtle,
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            offset: Vector::new(2.0, 0.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    })
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .height(Length::Fill)
    .into()
}
