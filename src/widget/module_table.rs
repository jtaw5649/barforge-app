use std::collections::HashSet;

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::state::{Screen, SortField, SortOrder};
use crate::domain::RegistryModule;
use crate::icons::Icon;
use crate::theme::{
    AppTheme, FONT_SM, FONT_XS, ICON_XS, RADIUS_SM, SPACE_MD, SPACE_SM, SPACE_XS,
};

use super::format_relative_time;

pub fn module_table<'a>(
    modules: &[&'a RegistryModule],
    installed_uuids: &HashSet<String>,
    theme: &AppTheme,
    sort_field: SortField,
    sort_order: SortOrder,
) -> Element<'a, Message> {
    let theme_copy = *theme;

    let sort_indicator = |field: SortField| -> &'static str {
        if sort_field == field {
            match sort_order {
                SortOrder::Ascending => " ▲",
                SortOrder::Descending => " ▼",
            }
        } else {
            ""
        }
    };

    let header_style = move |_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(theme_copy.bg_elevated)),
        border: Border {
            color: theme_copy.border_subtle,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    };

    let header_btn = |label: &str, field: SortField| {
        let label_with_indicator = format!("{}{}", label, sort_indicator(field));
        button(text(label_with_indicator).size(FONT_XS).color(theme_copy.text_muted))
            .on_press(if sort_field == field {
                Message::ToggleSortOrder
            } else {
                Message::SetSortField(field)
            })
            .padding([SPACE_XS, SPACE_SM])
            .style(move |_, _| iced::widget::button::Style {
                background: None,
                text_color: theme_copy.text_muted,
                ..Default::default()
            })
    };

    let header = container(
        row![
            container(header_btn("Name", SortField::Name)).width(Length::FillPortion(3)),
            container(text("Author").size(FONT_XS).color(theme_copy.text_muted))
                .width(Length::FillPortion(2))
                .padding([SPACE_XS, SPACE_SM]),
            container(text("Category").size(FONT_XS).color(theme_copy.text_muted))
                .width(Length::FillPortion(2))
                .padding([SPACE_XS, SPACE_SM]),
            container(header_btn("Downloads", SortField::Downloads)).width(Length::FillPortion(1)),
            container(header_btn("Rating", SortField::Rating)).width(Length::FillPortion(1)),
            container(header_btn("Updated", SortField::RecentlyUpdated)).width(Length::FillPortion(1)),
            container(text("Status").size(FONT_XS).color(theme_copy.text_muted))
                .width(Length::FillPortion(1))
                .padding([SPACE_XS, SPACE_SM]),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(header_style)
    .padding([SPACE_XS, SPACE_MD]);

    let rows: Vec<Element<Message>> = modules
        .iter()
        .map(|module| {
            let uuid = module.uuid.to_string();
            let is_installed = installed_uuids.contains(&uuid);

            let status_elem: Element<Message> = if is_installed {
                row![
                    Icon::Check.colored(ICON_XS, theme_copy.success),
                    text("Installed").size(FONT_XS).color(theme_copy.success),
                ]
                .spacing(SPACE_XS)
                .align_y(iced::Alignment::Center)
                .into()
            } else {
                Space::new().into()
            };

            let rating_text = module
                .rating
                .map(|r| format!("{:.1}", r))
                .unwrap_or_else(|| "-".to_string());

            let updated_text = module
                .last_updated
                .as_ref()
                .map(format_relative_time)
                .unwrap_or_else(|| "-".to_string());

            let verified_badge: Element<Message> = if module.verified_author {
                Icon::Check.colored(ICON_XS, theme_copy.success).into()
            } else {
                Space::new().into()
            };

            let row_content = row![
                container(
                    row![
                        text(&module.name).size(FONT_SM).color(theme_copy.text_normal),
                        verified_badge,
                    ]
                    .spacing(SPACE_XS)
                    .align_y(iced::Alignment::Center)
                )
                .width(Length::FillPortion(3))
                .padding([SPACE_XS, SPACE_SM]),
                container(text(&module.author).size(FONT_XS).color(theme_copy.text_muted))
                    .width(Length::FillPortion(2))
                    .padding([SPACE_XS, SPACE_SM]),
                container(text(module.category.display_name()).size(FONT_XS).color(theme_copy.text_muted))
                    .width(Length::FillPortion(2))
                    .padding([SPACE_XS, SPACE_SM]),
                container(text(module.formatted_downloads()).size(FONT_XS).color(theme_copy.text_muted))
                    .width(Length::FillPortion(1))
                    .padding([SPACE_XS, SPACE_SM]),
                container(text(rating_text).size(FONT_XS).color(theme_copy.text_muted))
                    .width(Length::FillPortion(1))
                    .padding([SPACE_XS, SPACE_SM]),
                container(text(updated_text).size(FONT_XS).color(theme_copy.text_muted))
                    .width(Length::FillPortion(1))
                    .padding([SPACE_XS, SPACE_SM]),
                container(status_elem)
                    .width(Length::FillPortion(1))
                    .padding([SPACE_XS, SPACE_SM]),
            ]
            .align_y(iced::Alignment::Center);

            let uuid_clone = uuid.clone();
            button(row_content)
                .on_press(Message::Navigate(Screen::ModuleDetail(uuid_clone)))
                .padding(0)
                .width(Length::Fill)
                .style(move |_, status| {
                    let bg = match status {
                        iced::widget::button::Status::Hovered => theme_copy.bg_elevated,
                        iced::widget::button::Status::Pressed => theme_copy.bg_surface,
                        _ => iced::Color::TRANSPARENT,
                    };
                    iced::widget::button::Style {
                        background: Some(Background::Color(bg)),
                        border: Border {
                            color: theme_copy.border_subtle,
                            width: 0.0,
                            radius: 0.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .into()
        })
        .collect();

    let table_body = column(rows).spacing(1);

    let table = column![header, scrollable(table_body).height(Length::Fill)]
        .spacing(0);

    container(table)
        .style(move |_| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.bg_surface)),
            border: Border {
                color: theme_copy.border_subtle,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        })
        .padding(0)
        .into()
}
