use chrono::{DateTime, Utc};
use iced::widget::{button, column, container, image, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::state::ScreenshotState;
use crate::domain::RegistryModule;
use crate::icons::Icon;
use crate::theme::{
    button as btn_style, container as cont_style, AppTheme, DETAIL_CONTENT_MAX_WIDTH, FONT_2XL,
    FONT_LG, FONT_MD, FONT_SM, FONT_XS, ICON_MD, ICON_SM, RADIUS_MD, RADIUS_SM,
    SCREENSHOT_FAILED_HEIGHT, SCREENSHOT_LOADING_HEIGHT, SCREENSHOT_MAX_HEIGHT, SPACE_LG, SPACE_MD,
    SPACE_SM, SPACE_XL, SPACE_XS,
};

use super::category_style;
use super::format_relative_time;

fn rating_stars_element<'a>(rating: f32, theme: &AppTheme) -> Element<'a, Message> {
    let full_stars = rating.floor() as usize;
    let has_half = rating - rating.floor() >= 0.5;
    let empty_stars = 5 - full_stars - if has_half { 1 } else { 0 };

    let mut stars: Vec<Element<Message>> = Vec::new();

    for _ in 0..full_stars {
        stars.push(Icon::Star.colored(14.0, theme.warning).into());
    }
    if has_half {
        stars.push(Icon::StarHalf.colored(14.0, theme.warning).into());
    }
    for _ in 0..empty_stars {
        stars.push(Icon::StarEmpty.colored(14.0, theme.text_faint).into());
    }

    row(stars)
        .spacing(2.0)
        .align_y(Alignment::Center)
        .into()
}

pub fn module_detail_screen<'a>(
    module: &'a RegistryModule,
    screenshot_state: &ScreenshotState,
    is_installed: bool,
    installed_at: Option<DateTime<Utc>>,
    installing: bool,
    theme: &'a AppTheme,
) -> Element<'a, Message> {
    let theme_copy = *theme;
    let badge_bg = category_style::badge_color(module.category);
    let badge_text = category_style::badge_text_color(module.category);

    let back_button = button(
        row![
            Icon::Back.colored(ICON_SM, theme.text_normal),
            text("Back to Browse")
                .size(FONT_SM)
                .color(theme.text_normal),
        ]
        .spacing(SPACE_XS)
        .align_y(Alignment::Center),
    )
    .on_press(Message::NavigateBack)
    .style(btn_style::secondary(*theme))
    .padding([SPACE_XS, SPACE_SM]);

    let category_badge = container(
        text(module.category.display_name())
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

    let header_row = row![back_button, Space::new().width(Length::Fill), category_badge]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let hero_section = {
        let mut title_row_items: Vec<Element<Message>> = vec![
            text(&module.name)
                .size(FONT_2XL)
                .color(theme.text_normal)
                .into(),
        ];

        if module.verified_author {
            title_row_items.push(Space::new().width(SPACE_XS).into());
            title_row_items.push(Icon::Check.colored(ICON_MD, theme.success).into());
        }

        let title_row = row(title_row_items).align_y(Alignment::Center);

        let author = text(format!("by {}", module.author))
            .size(FONT_MD)
            .color(theme.text_muted);

        let mut stats_items: Vec<Element<Message>> = vec![
            row![
                Icon::Download.colored(ICON_SM, theme.text_muted),
                text(module.formatted_downloads())
                    .size(FONT_SM)
                    .color(theme.text_muted),
            ]
            .spacing(SPACE_XS)
            .align_y(Alignment::Center)
            .into(),
        ];

        if let Some(rating) = module.rating {
            stats_items.push(Space::new().width(SPACE_MD).into());
            stats_items.push(rating_stars_element(rating, theme));
            stats_items.push(Space::new().width(SPACE_XS).into());
            stats_items.push(
                text(format!("{:.1}", rating))
                    .size(FONT_SM)
                    .color(theme.text_muted)
                    .into(),
            );
        }

        let stats_row = row(stats_items).align_y(Alignment::Center);

        column![title_row, author, stats_row].spacing(SPACE_XS)
    };

    let install_section: Element<Message> = if is_installed {
        let installed_text = if let Some(date) = installed_at {
            format!("Installed {}", format_relative_time(&date))
        } else {
            "Installed".to_string()
        };

        container(
            row![
                Icon::Check.colored(ICON_SM, theme.success),
                text(installed_text).size(FONT_SM).color(theme.success),
            ]
            .spacing(SPACE_XS)
            .align_y(Alignment::Center),
        )
        .padding([SPACE_SM, SPACE_LG])
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
    } else if installing {
        container(
            text("Installing...")
                .size(FONT_SM)
                .color(theme.text_muted),
        )
        .padding([SPACE_SM, SPACE_LG])
        .into()
    } else {
        button(
            row![
                Icon::Download.colored(ICON_SM, theme.text_normal),
                text("Install").size(FONT_SM),
            ]
            .spacing(SPACE_SM)
            .align_y(Alignment::Center),
        )
        .on_press(Message::DetailInstallModule)
        .style(btn_style::primary(*theme))
        .padding([SPACE_SM, SPACE_XL])
        .into()
    };

    let screenshot_section: Element<Message> = match screenshot_state {
        ScreenshotState::Loading => container(
            column![
                text("Loading screenshot...")
                    .size(FONT_SM)
                    .color(theme.text_faint),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(SCREENSHOT_LOADING_HEIGHT))
        .center(Length::Fill)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.bg_elevated)),
            border: Border {
                radius: RADIUS_MD.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into(),

        ScreenshotState::Loaded(handle) => container(
            image(handle.clone())
                .width(Length::Fill)
                .content_fit(iced::ContentFit::Contain),
        )
        .width(Length::Fill)
        .max_height(SCREENSHOT_MAX_HEIGHT)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.bg_elevated)),
            border: Border {
                radius: RADIUS_MD.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into(),

        ScreenshotState::Failed => container(
            column![
                Icon::Info.colored(ICON_MD, theme.text_faint),
                text("Screenshot unavailable")
                    .size(FONT_SM)
                    .color(theme.text_faint),
            ]
            .spacing(SPACE_SM)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(SCREENSHOT_FAILED_HEIGHT))
        .center(Length::Fill)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.bg_elevated)),
            border: Border {
                radius: RADIUS_MD.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into(),

        ScreenshotState::NotLoaded => Space::new().into(),
    };

    let about_title = text("About").size(FONT_LG).color(theme.text_normal);
    
    let description_content: Element<'a, Message> = if module.description.len() > 0 {
         text(&module.description)
            .size(FONT_MD)
            .color(theme.text_muted)
            .into()
    } else {
        text("No description provided.").size(FONT_SM).color(theme.text_faint).into()
    };

    let description_section = container(
        column![
            about_title,
            Space::new().height(SPACE_SM),
            description_content,
        ],
    )
    .style(cont_style::card(*theme))
    .padding(SPACE_LG)
    .width(Length::Fill);

    let reviews_section = container(
        column![
            text("Reviews").size(FONT_LG).color(theme.text_normal),
            Space::new().height(SPACE_SM),
            container(
                text("Reviews coming soon in v1.0.0")
                    .size(FONT_SM)
                    .color(theme.text_faint)
            )
            .padding(SPACE_LG)
            .width(Length::Fill)
            .style(move |_: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(theme_copy.bg_base)),
                border: Border {
                    radius: RADIUS_SM.into(),
                    width: 1.0,
                    color: theme_copy.border_subtle,
                },
                ..Default::default()
            })
        ]
    )
    .style(cont_style::card(*theme))
    .padding(SPACE_LG)
    .width(Length::Fill);

    let mut info_items: Vec<Element<Message>> = Vec::new();

    if let Some(version) = &module.version {
        info_items.push(
            row![
                text("Version").size(FONT_SM).color(theme.text_muted),
                Space::new().width(Length::Fill),
                text(version.to_string()).size(FONT_SM).color(theme.text_normal),
            ]
            .width(Length::Fill)
            .into(),
        );
    }

    if let Some(last_updated) = &module.last_updated {
        info_items.push(
            row![
                text("Last Updated").size(FONT_SM).color(theme.text_muted),
                Space::new().width(Length::Fill),
                text(format_relative_time(last_updated)).size(FONT_SM).color(theme.text_normal),
            ]
            .width(Length::Fill)
            .into(),
        );
    }

    let info_section: Element<Message> = if info_items.is_empty() {
        Space::new().into()
    } else {
        container(
            column![
                text("Module Info")
                    .size(FONT_LG)
                    .color(theme.text_normal),
                Space::new().height(SPACE_SM),
                column(info_items).spacing(SPACE_SM),
            ],
        )
        .style(cont_style::card(*theme))
        .padding(SPACE_LG)
        .width(Length::Fill)
        .into()
    };

    let repo_button = button(
        row![
            text("View Repository").size(FONT_SM),
            text("→").size(FONT_SM),
        ]
        .spacing(SPACE_SM),
    )
    .on_press(Message::OpenRepoUrl(module.repo_url.clone()))
    .style(btn_style::secondary(*theme))
    .padding([SPACE_SM, SPACE_LG]);

    let links_section = container(repo_button)
        .style(cont_style::card(*theme))
        .padding(SPACE_LG)
        .width(Length::Fill);

    let content = column![
        header_row,
        Space::new().height(SPACE_MD),
        hero_section,
        Space::new().height(SPACE_MD),
        install_section,
        Space::new().height(SPACE_LG),
        screenshot_section,
        Space::new().height(SPACE_LG),
        description_section,
        Space::new().height(SPACE_LG),
        reviews_section,
        Space::new().height(SPACE_MD),
        info_section,
        Space::new().height(SPACE_MD),
        links_section,
    ]
    .padding(SPACE_XL)
    .max_width(DETAIL_CONTENT_MAX_WIDTH);

    scrollable(
        container(content)
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}