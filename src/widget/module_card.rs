use iced::widget::{button, column, container, row, text, Space};
use iced::{Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::state::Screen;
use crate::domain::RegistryModule;
use crate::icons::Icon;
use crate::theme::{
    shadow_md, shadow_hover, AppTheme, FONT_LG, FONT_SM, FONT_XS, ICON_SM, ICON_XS, RADIUS_LG,
    RADIUS_SM, SPACE_2XS, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS,
};

use super::category_style;
use super::{format_relative_time, rating_stars_text};

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
    let category = module.category;
    let downloads = module.formatted_downloads();
    let rating = module.rating;
    let verified = module.verified_author;
    let last_updated = module.last_updated;

    let theme_copy = *theme;
    let badge_bg = category_style::badge_color(category);
    let badge_text = category_style::badge_text_color(category);

    let title_row = row![text(name).size(FONT_LG).color(theme.text_normal),]
        .align_y(iced::Alignment::Center);

    let author_row: Element<Message> = if verified {
        row![
            text(format!("by {author}")).size(FONT_XS).color(theme.text_faint),
            Icon::Check.colored(ICON_XS, theme.success),
        ]
        .spacing(SPACE_XS)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        text(format!("by {author}"))
            .size(FONT_XS)
            .color(theme.text_faint)
            .into()
    };

    let header = column![title_row, author_row].spacing(SPACE_2XS);

    let desc = container(
        text(description)
            .size(FONT_SM)
            .color(theme.text_muted),
    )
    .height(Length::Fixed(48.0));

    let category_badge = container(
        text(category.display_name())
            .size(FONT_XS)
            .color(badge_text),
    )
    .padding([SPACE_XS, SPACE_SM])
    .style(move |_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(badge_bg)),
        border: Border {
            radius: RADIUS_SM.into(),
            width: 1.0,
            color: iced::Color::from_rgba(badge_text.r, badge_text.g, badge_text.b, 0.2),
        },
        ..Default::default()
    });

    let download_row = row![
        Icon::Download.colored(ICON_XS, theme.text_faint),
        text(downloads).size(FONT_XS).color(theme.text_faint),
    ]
    .spacing(SPACE_XS)
    .align_y(iced::Alignment::Center);

    let rating_row: Element<Message> = if let Some(r) = rating {
        row![
            text(rating_stars_text(r)).size(FONT_XS).color(theme.warning),
            text(format!("{:.1}", r)).size(FONT_XS).color(theme.text_faint),
        ]
        .spacing(SPACE_XS)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        Space::new().into()
    };

    let updated_row: Element<Message> = if let Some(dt) = last_updated {
        text(format_relative_time(&dt))
            .size(FONT_XS)
            .color(theme.text_faint)
            .into()
    } else {
        Space::new().into()
    };

    let stats_row = row![rating_row, download_row, updated_row]
        .spacing(SPACE_MD)
        .align_y(iced::Alignment::Center);

    let footer = row![category_badge, Space::new().width(Length::Fill), stats_row]
        .align_y(iced::Alignment::Center);

    let install_indicator: Element<Message> = if is_installed {
        container(
            row![
                Icon::Check.colored(ICON_SM, theme.success),
                text("Installed").size(FONT_SM).color(theme.success),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center),
        )
        .padding([SPACE_XS, SPACE_SM])
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(iced::Color::from_rgba(
                theme_copy.success.r,
                theme_copy.success.g,
                theme_copy.success.b,
                0.12,
            ))),
            border: Border {
                radius: RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::new().into()
    };

    let status_row = row![Space::new().width(Length::Fill), install_indicator];

    let card_content = column![header, desc, footer, status_row]
        .spacing(SPACE_MD)
        .width(Length::Fill);

    button(card_content)
        .on_press(Message::Navigate(Screen::ModuleDetail(uuid)))
        .padding([SPACE_LG, SPACE_MD])
        .style(move |_, status| {
            let (bg, border_color, border_width, shadow) = match status {
                iced::widget::button::Status::Hovered => (
                    theme_copy.bg_elevated,
                    theme_copy.accent,
                    1.5,
                    shadow_hover(),
                ),
                iced::widget::button::Status::Pressed => (
                    theme_copy.bg_surface,
                    theme_copy.accent,
                    1.5,
                    shadow_md(),
                ),
                _ => (
                    theme_copy.bg_surface,
                    theme_copy.border_subtle,
                    1.0,
                    shadow_md(),
                ),
            };
            iced::widget::button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    color: border_color,
                    width: border_width,
                    radius: RADIUS_LG.into(),
                },
                shadow,
                ..Default::default()
            }
        })
        .width(Length::Fixed(width))
        .into()
}
