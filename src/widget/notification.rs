use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::app::message::Message;
use crate::app::state::{Notification, NotificationKind};
use crate::icons::Icon;
use crate::theme::{
    shadow_md, AppTheme, FONT_SM, ICON_MD, NOTIFICATION_WIDTH, RADIUS_MD, SPACE_MD, SPACE_SM,
};

pub fn notification_toast(notif: &Notification, theme: &AppTheme) -> Element<'static, Message> {
    let (icon, icon_color) = match notif.kind {
        NotificationKind::Success => (Icon::Check, theme.text_normal),
        NotificationKind::Error => (Icon::Error, theme.text_normal),
        NotificationKind::Info => (Icon::Info, theme.info),
    };

    let message = notif.message.clone();
    let theme_copy = *theme;

    let dismiss_btn = button(text("×").size(16).color(theme.text_muted))
        .on_press(Message::DismissNotification)
        .padding(SPACE_SM)
        .style(move |_, status| {
            let (background, text_color) = match status {
                iced::widget::button::Status::Active => (Color::TRANSPARENT, theme_copy.text_muted),
                iced::widget::button::Status::Hovered => {
                    (theme_copy.bg_elevated, theme_copy.text_normal)
                }
                iced::widget::button::Status::Pressed => {
                    (theme_copy.bg_surface, theme_copy.text_normal)
                }
                iced::widget::button::Status::Disabled => {
                    (Color::TRANSPARENT, theme_copy.text_faint)
                }
            };
            iced::widget::button::Style {
                background: Some(Background::Color(background)),
                text_color,
                border: Border::default(),
                ..Default::default()
            }
        });

    let content = row![
        icon.colored(ICON_MD, icon_color),
        text(message).size(FONT_SM).color(theme.text_normal),
        Space::new().width(Length::Fill),
        dismiss_btn,
    ]
    .spacing(SPACE_MD)
    .align_y(Alignment::Center)
    .padding(SPACE_MD);

    let (bg_color, border_color, border_width) = match notif.kind {
        NotificationKind::Success => (theme.success, theme.success, 0.0),
        NotificationKind::Error => (theme.danger, theme.danger, 0.0),
        NotificationKind::Info => (theme.bg_floating, theme.info, 1.0),
    };

    container(content)
        .style(move |_| iced::widget::container::Style {
            background: Some(Background::Color(bg_color)),
            border: Border {
                color: border_color,
                width: border_width,
                radius: RADIUS_MD.into(),
            },
            shadow: shadow_md(),
            ..Default::default()
        })
        .width(Length::Fixed(NOTIFICATION_WIDTH))
        .into()
}
