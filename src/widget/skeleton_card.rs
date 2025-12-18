use iced::widget::{column, container, row, Space};
use iced::{Background, Border, Color, Element, Length, Radians};

use crate::app::message::Message;
use crate::theme::{
    container as cont_style, AppTheme, FONT_MD, FONT_SM, FONT_XS, RADIUS_SM, SKELETON_BAR_MD,
    SKELETON_BAR_SM, SPACE_MD, SPACE_SM, SPACE_XS,
};

fn shimmer_gradient(base_color: Color, shimmer_phase: f32) -> Background {
    let highlight = Color {
        r: (base_color.r + 0.08).min(1.0),
        g: (base_color.g + 0.08).min(1.0),
        b: (base_color.b + 0.08).min(1.0),
        a: base_color.a,
    };

    let stops = vec![
        iced::gradient::ColorStop { offset: 0.0, color: base_color },
        iced::gradient::ColorStop { offset: shimmer_phase.max(0.0), color: base_color },
        iced::gradient::ColorStop { offset: (shimmer_phase + 0.15).clamp(0.0, 1.0), color: highlight },
        iced::gradient::ColorStop { offset: (shimmer_phase + 0.3).min(1.0), color: base_color },
        iced::gradient::ColorStop { offset: 1.0, color: base_color },
    ];

    Background::Gradient(iced::Gradient::Linear(
        iced::gradient::Linear::new(Radians(0.0)).add_stops(stops),
    ))
}

fn skeleton_bar(
    width: Length,
    height: f32,
    theme: &AppTheme,
    shimmer_phase: f32,
) -> Element<'static, Message> {
    let bg_color = theme.bg_elevated;
    let gradient = shimmer_gradient(bg_color, shimmer_phase);

    container(Space::new())
        .width(width)
        .height(Length::Fixed(height))
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(gradient),
            border: Border {
                radius: RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

pub fn skeleton_card(theme: &AppTheme, width: f32, shimmer_frame: usize) -> Element<'static, Message> {
    let shimmer_phase = ((shimmer_frame % 30) as f32 / 30.0) * 1.3 - 0.3;

    let header = column![
        skeleton_bar(Length::FillPortion(7), FONT_MD, theme, shimmer_phase),
        skeleton_bar(Length::FillPortion(4), FONT_XS, theme, shimmer_phase),
    ]
    .spacing(SPACE_XS)
    .width(Length::Fill);

    let description = column![
        skeleton_bar(Length::Fill, FONT_SM, theme, shimmer_phase),
        skeleton_bar(Length::FillPortion(8), FONT_SM, theme, shimmer_phase),
    ]
    .spacing(SPACE_XS)
    .width(Length::Fill);

    let footer = row![
        skeleton_bar(Length::Fixed(SKELETON_BAR_MD), FONT_XS + 4.0, theme, shimmer_phase),
        Space::new().width(Length::Fill),
        skeleton_bar(Length::Fixed(SKELETON_BAR_SM), FONT_XS, theme, shimmer_phase),
    ]
    .align_y(iced::Alignment::Center);

    let card_content = column![header, description, footer, Space::new().height(SPACE_SM)]
        .spacing(SPACE_SM)
        .width(Length::Fill);

    container(card_content)
        .padding(SPACE_MD)
        .width(Length::Fixed(width))
        .style(cont_style::card(*theme))
        .into()
}
