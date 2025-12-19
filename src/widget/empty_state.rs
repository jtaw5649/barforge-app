use iced::widget::{button, column, container, text};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::icons::Icon;
use crate::theme::{
    AppTheme, FONT_2XL, FONT_SM, ICON_2XL, SPACE_MD, SPACE_SM, button as button_style,
};

pub fn empty_state(
    icon: Icon,
    title: &'static str,
    subtitle: &'static str,
    theme: &AppTheme,
) -> Element<'static, Message> {
    container(
        column![
            icon.svg(ICON_2XL),
            text(title).size(FONT_2XL).color(theme.text_normal),
            text(subtitle).size(FONT_SM).color(theme.text_muted),
        ]
        .spacing(SPACE_MD)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}

pub fn empty_state_dynamic(
    icon: Icon,
    title: &'static str,
    subtitle: String,
    theme: &AppTheme,
) -> Element<'static, Message> {
    container(
        column![
            icon.svg(ICON_2XL),
            text(title).size(FONT_2XL).color(theme.text_normal),
            text(subtitle).size(FONT_SM).color(theme.text_muted),
        ]
        .spacing(SPACE_MD)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}

pub fn empty_state_with_action(
    icon: Icon,
    title: &'static str,
    subtitle: &'static str,
    action_label: &'static str,
    action_message: Message,
    theme: &AppTheme,
) -> Element<'static, Message> {
    let action_button = button(text(action_label).size(FONT_SM))
        .on_press(action_message)
        .padding([SPACE_SM, SPACE_MD])
        .style(button_style::secondary(*theme));

    container(
        column![
            icon.svg(ICON_2XL),
            text(title).size(FONT_2XL).color(theme.text_normal),
            text(subtitle).size(FONT_SM).color(theme.text_muted),
            action_button,
        ]
        .spacing(SPACE_MD)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}
