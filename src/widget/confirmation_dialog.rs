use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::state::ConfirmationAction;
use crate::theme::{
    button as button_style, shadow_lg, AppTheme, FONT_MD, FONT_SM, RADIUS_LG, SPACE_LG, SPACE_MD,
    SPACE_SM,
};

pub fn confirmation_dialog(action: &ConfirmationAction, theme: &AppTheme) -> Element<'static, Message> {
    let (title, message) = match action {
        ConfirmationAction::UninstallModule { name, .. } => (
            "Uninstall Module",
            format!("Are you sure you want to uninstall \"{}\"? This action cannot be undone.", name),
        ),
    };

    let theme_copy = *theme;

    let cancel_btn = button(text("Cancel").size(FONT_SM))
        .on_press(Message::CancelConfirmation)
        .padding([SPACE_SM, SPACE_MD])
        .style(button_style::secondary(*theme));

    let confirm_btn = button(text("Uninstall").size(FONT_SM))
        .on_press(Message::ConfirmAction)
        .padding([SPACE_SM, SPACE_MD])
        .style(button_style::danger(*theme));

    let buttons = row![cancel_btn, confirm_btn]
        .spacing(SPACE_SM)
        .align_y(Alignment::Center);

    let dialog_content = column![
        text(title).size(FONT_MD).color(theme.text_normal),
        text(message).size(FONT_SM).color(theme.text_muted),
        Space::new().height(SPACE_SM),
        buttons,
    ]
    .spacing(SPACE_SM)
    .align_x(Alignment::Center)
    .width(Length::Fixed(350.0));

    let dialog = container(dialog_content)
        .padding(SPACE_LG)
        .style(move |_| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.bg_floating)),
            border: Border {
                color: theme_copy.border_default,
                width: 1.0,
                radius: RADIUS_LG.into(),
            },
            shadow: shadow_lg(),
            ..Default::default()
        });

    container(dialog)
        .center(Length::Fill)
        .style(move |_| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.bg_overlay)),
            ..Default::default()
        })
        .into()
}
