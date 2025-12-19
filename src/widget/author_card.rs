use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::Message;
use crate::domain::Author;
use crate::icons::Icon;
use crate::theme::{
    AppTheme, FONT_MD, FONT_SM, FONT_XS, ICON_SM, RADIUS_MD, SPACE_MD, SPACE_SM, SPACE_XS,
    button as btn_style,
};

pub fn author_card<'a>(author: &'a Author, theme: &'a AppTheme) -> Element<'a, Message> {
    let theme_copy = *theme;

    let display_name = author.display();

    let mut name_row: Vec<Element<'a, Message>> = vec![
        text(display_name)
            .size(FONT_MD)
            .color(theme.text_normal)
            .into(),
    ];

    if author.verified_author {
        name_row.push(Space::new().width(SPACE_XS).into());
        name_row.push(Icon::Check.colored(ICON_SM, theme.success).into());
    }

    let header = row(name_row).align_y(Alignment::Center);

    let username = text(format!("@{}", author.username))
        .size(FONT_XS)
        .color(theme.text_muted);

    let stats = text(format!("{} modules", author.module_count))
        .size(FONT_XS)
        .color(theme.text_faint);

    let view_btn = button(text("View Profile →").size(FONT_SM))
        .on_press(Message::AuthorClicked(author.username.clone()))
        .style(btn_style::secondary(*theme))
        .padding([SPACE_XS, SPACE_SM]);

    let content = column![
        header,
        username,
        stats,
        Space::new().height(SPACE_SM),
        view_btn
    ]
    .spacing(SPACE_XS);

    container(content)
        .padding(SPACE_MD)
        .width(Length::Fill)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.bg_elevated)),
            border: Border {
                radius: RADIUS_MD.into(),
                width: 1.0,
                color: theme_copy.border_subtle,
            },
            ..Default::default()
        })
        .into()
}

pub fn author_mini<'a>(
    author_name: &'a str,
    verified: bool,
    theme: &'a AppTheme,
) -> Element<'a, Message> {
    let mut items: Vec<Element<'a, Message>> = vec![
        text(format!("by {}", author_name))
            .size(FONT_SM)
            .color(theme.text_muted)
            .into(),
    ];

    if verified {
        items.push(Space::new().width(SPACE_XS).into());
        items.push(Icon::Check.colored(ICON_SM, theme.success).into());
    }

    button(row(items).align_y(Alignment::Center))
        .on_press(Message::AuthorClicked(author_name.to_string()))
        .style(btn_style::ghost(*theme))
        .padding(0)
        .into()
}
