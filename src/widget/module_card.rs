use iced::widget::{Space, button, column, container, row, text};
use iced::{Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::state::Screen;
use crate::domain::RegistryModule;
use crate::icons::Icon;
use crate::theme::button as btn_style;
use crate::theme::{
    AppTheme, DESCRIPTION_HEIGHT, FONT_LG, FONT_SM, FONT_XS, ICON_XS, RADIUS_LG, RADIUS_SM,
    SPACE_2XS, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS, shadow_hover, shadow_md,
};

pub fn module_card(
    module: &RegistryModule,
    is_installed: bool,
    theme: &AppTheme,
    width: f32,
) -> Element<'static, Message> {
    let uuid = module.uuid.to_string();
    let name = module.name.clone();
    let author = module.author.clone();
    let description = module.truncated_description(100);
    let downloads = module.formatted_downloads();
    let rating = module.rating;
    let verified = module.verified_author;

    let theme_copy = *theme;

    let title = text(name).size(FONT_LG).color(theme.text_normal);

    let author_row = if verified {
        row![
            text(format!("by {author}"))
                .size(FONT_XS)
                .color(theme.text_faint),
            Icon::Check.colored(ICON_XS, theme.success),
        ]
        .spacing(SPACE_XS)
        .align_y(iced::Alignment::Center)
    } else {
        row![
            text(format!("by {author}"))
                .size(FONT_XS)
                .color(theme.text_faint)
        ]
        .align_y(iced::Alignment::Center)
    };

    let header = column![title, author_row].spacing(SPACE_2XS);

    let desc = container(
        text(description)
            .size(FONT_SM)
            .color(theme.text_muted)
            .height(Length::Fixed(DESCRIPTION_HEIGHT)),
    );

    let download_item = row![
        Icon::Download.colored(ICON_XS, theme.text_faint),
        text(downloads).size(FONT_XS).color(theme.text_faint),
    ]
    .spacing(SPACE_XS)
    .align_y(iced::Alignment::Center);

    let stats_row = if let Some(r) = rating {
        row![
            row![
                Icon::Star.colored(ICON_XS, theme.warning),
                text(format!("{:.1}", r))
                    .size(FONT_XS)
                    .color(theme.text_faint),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center),
            download_item
        ]
        .spacing(SPACE_MD)
        .align_y(iced::Alignment::Center)
    } else {
        row![download_item].align_y(iced::Alignment::Center)
    };

    let action_element: Element<Message> = if is_installed {
        container(
            row![
                Icon::Check.colored(ICON_XS, theme.success),
                text("Installed").size(FONT_XS).color(theme.success),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center),
        )
        .padding([SPACE_XS, SPACE_SM])
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.success_muted())),
            border: Border {
                radius: RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        button(text("Install").size(FONT_XS))
            .on_press(Message::DetailInstallModule)
            .style(btn_style::primary_small(theme_copy))
            .padding([SPACE_XS, SPACE_MD])
            .into()
    };

    let footer = row![stats_row, Space::new().width(Length::Fill), action_element]
        .align_y(iced::Alignment::Center);

    let card_content = column![header, desc, footer]
        .spacing(SPACE_MD)
        .width(Length::Fill);

    button(card_content)
        .on_press(Message::Navigate(Screen::ModuleDetail(uuid)))
        .padding(SPACE_LG)
        .style(move |_, status| {
            let (bg, border_color, shadow) = match status {
                iced::widget::button::Status::Hovered => {
                    (theme_copy.bg_elevated, theme_copy.accent, shadow_hover())
                }
                _ => (theme_copy.bg_surface, theme_copy.border_subtle, shadow_md()),
            };
            iced::widget::button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: RADIUS_LG.into(),
                },
                shadow,
                ..Default::default()
            }
        })
        .width(Length::Fixed(width))
        .into()
}
