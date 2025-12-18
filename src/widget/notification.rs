use iced::widget::{button, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use crate::app::message::Message;
use crate::app::state::{Notification, NotificationKind};
use crate::icons::Icon;
use crate::theme::{AppTheme, FONT_SM, ICON_MD, NOTIFICATION_WIDTH, RADIUS_MD, SPACE_MD};

pub fn notification_toast(notif: &Notification, theme: &AppTheme) -> Element<'static, Message> {
    let (status_color, icon) = match notif.kind {
        NotificationKind::Success => (theme.success, Icon::Check),
        NotificationKind::Warning => (theme.warning, Icon::Warning),
        NotificationKind::Error => (theme.danger, Icon::Error),
        NotificationKind::Info => (theme.info, Icon::Info),
    };

    let message = notif.message.clone();
    let theme_copy = *theme;

    let dismiss_btn = button(text("×").size(18).color(theme.text_muted))
        .on_press(Message::DismissNotification)
        .padding([2.0, 8.0])
        .style(move |_, status| {
            let text_color = match status {
                iced::widget::button::Status::Hovered => theme_copy.text_normal,
                _ => theme_copy.text_muted,
            };
            iced::widget::button::Style {
                background: None,
                text_color,
                border: Border::default(),
                ..Default::default()
            }
        });

    let content = row![
        icon.colored(ICON_MD, status_color),
        text(message)
            .size(FONT_SM)
            .color(theme.text_normal)
            .width(Length::Fill),
        dismiss_btn,
    ]
    .spacing(SPACE_MD)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    container(content)
        .padding(SPACE_MD)
        .width(Length::Fixed(NOTIFICATION_WIDTH))
        .style(move |_| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.bg_floating)),
            border: Border {
                color: theme_copy.border_subtle,
                width: 1.0,
                radius: RADIUS_MD.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        })
        .into()
}
