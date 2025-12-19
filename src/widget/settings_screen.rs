use iced::widget::{Space, button, checkbox, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::icons::Icon;
use crate::theme::{
    AppTheme, FONT_2XL, FONT_LG, FONT_MD, FONT_SM, FONT_XS, ICON_SM, SETTINGS_CONTENT_MAX_WIDTH,
    SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS, button as btn_style, checkbox as chk_style,
    container as cont_style,
};

const GITHUB_URL: &str = "https://github.com/jtaw5649/waybar-manager";

pub fn settings_screen(theme: &AppTheme, tray_enabled: bool) -> Element<'_, Message> {
    let header = text("Settings").size(FONT_2XL).color(theme.text_normal);

    let data_label = text("Data").size(FONT_LG).color(theme.text_normal);

    let clear_cache_btn = button(
        row![
            Icon::Error.colored(ICON_SM, theme.text_muted),
            text("Clear Cache").size(FONT_SM).color(theme.text_normal),
        ]
        .spacing(SPACE_SM)
        .align_y(Alignment::Center),
    )
    .on_press(Message::ClearCache)
    .style(btn_style::secondary(*theme))
    .padding([SPACE_SM, SPACE_MD]);

    let clear_cache_desc = text("Remove cached registry data to force a fresh download")
        .size(FONT_XS)
        .color(theme.text_faint);

    let reset_settings_btn = button(
        row![
            Icon::Error.colored(ICON_SM, theme.danger),
            text("Reset Settings")
                .size(FONT_SM)
                .color(theme.text_normal),
        ]
        .spacing(SPACE_SM)
        .align_y(Alignment::Center),
    )
    .on_press(Message::ResetSettings)
    .style(btn_style::secondary(*theme))
    .padding([SPACE_SM, SPACE_MD]);

    let reset_settings_desc = text("Reset all module preferences to defaults")
        .size(FONT_XS)
        .color(theme.text_faint);

    let data_section = container(
        column![
            data_label,
            Space::new().height(SPACE_MD),
            column![clear_cache_btn, clear_cache_desc].spacing(SPACE_XS),
            Space::new().height(SPACE_SM),
            column![reset_settings_btn, reset_settings_desc].spacing(SPACE_XS),
        ]
        .spacing(SPACE_XS),
    )
    .style(cont_style::card(*theme))
    .padding(SPACE_LG)
    .width(Length::Fill);

    let about_label = text("About").size(FONT_LG).color(theme.text_normal);

    let version_row = row![
        text("Version").size(FONT_MD).color(theme.text_muted),
        Space::new().width(Length::Fill),
        text(format!("v{}", env!("CARGO_PKG_VERSION")))
            .size(FONT_MD)
            .color(theme.text_normal),
    ]
    .align_y(Alignment::Center);

    let github_btn = button(
        row![
            text("View on GitHub").size(FONT_SM),
            text("→").size(FONT_SM),
        ]
        .spacing(SPACE_SM),
    )
    .on_press(Message::OpenRepoUrl(GITHUB_URL.to_string()))
    .style(btn_style::secondary(*theme))
    .padding([SPACE_SM, SPACE_LG]);

    let about_section = container(
        column![
            about_label,
            Space::new().height(SPACE_MD),
            version_row,
            Space::new().height(SPACE_SM),
            github_btn,
        ]
        .spacing(SPACE_XS),
    )
    .style(cont_style::card(*theme))
    .padding(SPACE_LG)
    .width(Length::Fill);

    let appearance_label = text("Appearance").size(FONT_LG).color(theme.text_normal);

    let tray_checkbox = checkbox(tray_enabled)
        .label("Show system tray icon")
        .on_toggle(Message::ToggleTray)
        .style(chk_style::themed(*theme));

    let tray_desc = text("Display an icon in the system tray for quick access")
        .size(FONT_XS)
        .color(theme.text_faint);

    let appearance_section = container(
        column![
            appearance_label,
            Space::new().height(SPACE_MD),
            column![tray_checkbox, tray_desc].spacing(SPACE_XS),
        ]
        .spacing(SPACE_XS),
    )
    .style(cont_style::card(*theme))
    .padding(SPACE_LG)
    .width(Length::Fill);

    let content = column![header, appearance_section, data_section, about_section]
        .spacing(SPACE_XL)
        .padding(SPACE_XL)
        .max_width(SETTINGS_CONTENT_MAX_WIDTH);

    scrollable(
        container(content)
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
