use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text, text_input, Space};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::Message;
use crate::services::{ModulePreferences, PreferenceField, PreferenceValue, PreferencesSchema, SelectOption};
use crate::theme::{
    button as btn_style, AppTheme, FONT_LG, FONT_MD, FONT_SM, FONT_XS, RADIUS_LG, RADIUS_MD,
    RADIUS_SM, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS,
};

pub fn preferences_modal<'a>(
    module_name: &str,
    uuid: &str,
    schema: &'a PreferencesSchema,
    current_values: &'a ModulePreferences,
    theme: &'a AppTheme,
) -> Element<'a, Message> {
    let theme_copy = *theme;

    let backdrop = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
            ..Default::default()
        });

    let title_text = schema
        .title
        .clone()
        .unwrap_or_else(|| format!("{} Preferences", module_name));

    let header = row![
        text(title_text).size(FONT_LG).color(theme.text_normal),
        Space::new().width(Length::Fill),
        button(text("×").size(FONT_LG))
            .on_press(Message::ClosePreferences)
            .style(btn_style::ghost(*theme))
            .padding([SPACE_XS, SPACE_SM]),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let fields: Vec<Element<Message>> = schema
        .fields
        .iter()
        .map(|field| render_field(field, current_values, uuid, theme))
        .collect();

    let fields_column = column(fields).spacing(SPACE_MD).width(Length::Fill);

    let uuid_owned = uuid.to_string();
    let footer = row![
        button(text("Reset to Defaults").size(FONT_SM))
            .on_press(Message::ResetPreferences(uuid_owned.clone()))
            .style(btn_style::secondary(*theme))
            .padding([SPACE_SM, SPACE_MD]),
        Space::new().width(Length::Fill),
        button(text("Close").size(FONT_SM))
            .on_press(Message::ClosePreferences)
            .style(btn_style::primary(*theme))
            .padding([SPACE_SM, SPACE_LG]),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let modal_content = column![
        header,
        Space::new().height(SPACE_MD),
        scrollable(fields_column).height(Length::FillPortion(1)),
        Space::new().height(SPACE_MD),
        footer,
    ]
    .spacing(SPACE_SM)
    .width(Length::Fill);

    let modal = container(modal_content)
        .padding(SPACE_XL)
        .width(Length::Fixed(500.0))
        .max_height(600.0)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme_copy.bg_surface)),
            border: Border {
                color: theme_copy.border,
                width: 1.0,
                radius: RADIUS_LG.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        });

    let centered_modal = container(modal)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill);

    iced::widget::stack![backdrop, centered_modal].into()
}

fn render_field<'a>(
    field: &'a PreferenceField,
    values: &'a ModulePreferences,
    uuid: &str,
    theme: &'a AppTheme,
) -> Element<'a, Message> {
    let theme_copy = *theme;
    let uuid_owned = uuid.to_string();

    match field {
        PreferenceField::Text {
            key,
            label,
            description,
            placeholder,
            ..
        } => {
            let current_value = values
                .get(key)
                .and_then(|v| v.as_string())
                .unwrap_or("");

            let key_owned = key.clone();
            let input = text_input(placeholder.as_deref().unwrap_or(""), current_value)
                .on_input(move |val| {
                    Message::PreferenceChanged(
                        uuid_owned.clone(),
                        key_owned.clone(),
                        PreferenceValue::String(val),
                    )
                })
                .padding(SPACE_SM)
                .width(Length::Fill)
                .style(move |_, status| {
                    let border_color = match status {
                        iced::widget::text_input::Status::Focused { .. } => theme_copy.primary,
                        _ => theme_copy.border,
                    };
                    iced::widget::text_input::Style {
                        background: Background::Color(theme_copy.bg_elevated),
                        border: Border {
                            color: border_color,
                            width: 1.0,
                            radius: RADIUS_SM.into(),
                        },
                        icon: theme_copy.text_normal,
                        placeholder: theme_copy.text_faint,
                        value: theme_copy.text_normal,
                        selection: iced::Color::from_rgba(
                            theme_copy.primary.r,
                            theme_copy.primary.g,
                            theme_copy.primary.b,
                            0.3,
                        ),
                    }
                });

            let mut col = column![
                text(label).size(FONT_MD).color(theme.text_normal),
                input,
            ]
            .spacing(SPACE_XS);

            if let Some(desc) = description {
                col = col.push(text(desc).size(FONT_XS).color(theme.text_muted));
            }

            container(col)
                .width(Length::Fill)
                .padding(SPACE_SM)
                .style(move |_: &iced::Theme| iced::widget::container::Style {
                    background: Some(Background::Color(theme_copy.bg_base)),
                    border: Border {
                        radius: RADIUS_MD.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        }

        PreferenceField::Boolean {
            key,
            label,
            description,
            ..
        } => {
            let current_value = values.get(key).and_then(|v| v.as_bool()).unwrap_or(false);

            let key_owned = key.clone();
            let label_owned = label.clone();
            let cb = checkbox(current_value)
                .label(label_owned)
                .on_toggle(move |val| {
                    Message::PreferenceChanged(
                        uuid_owned.clone(),
                        key_owned.clone(),
                        PreferenceValue::Bool(val),
                    )
                });

            let mut col = column![cb].spacing(SPACE_XS);

            if let Some(desc) = description {
                col = col.push(text(desc).size(FONT_XS).color(theme.text_muted));
            }

            container(col)
                .width(Length::Fill)
                .padding(SPACE_SM)
                .style(move |_: &iced::Theme| iced::widget::container::Style {
                    background: Some(Background::Color(theme_copy.bg_base)),
                    border: Border {
                        radius: RADIUS_MD.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        }

        PreferenceField::Select {
            key,
            label,
            description,
            options,
            ..
        } => {
            let current_value = values.get(key).and_then(|v| v.as_string());

            let selected = options
                .iter()
                .find(|o| Some(o.value.as_str()) == current_value)
                .cloned();

            let key_owned = key.clone();
            let picker = pick_list(
                options.clone(),
                selected,
                move |opt: SelectOption| {
                    Message::PreferenceChanged(
                        uuid_owned.clone(),
                        key_owned.clone(),
                        PreferenceValue::String(opt.value),
                    )
                },
            )
            .padding(SPACE_SM)
            .width(Length::Fill)
            .style(move |_, _| iced::widget::pick_list::Style {
                text_color: theme_copy.text_normal,
                placeholder_color: theme_copy.text_faint,
                handle_color: theme_copy.text_muted,
                background: Background::Color(theme_copy.bg_elevated),
                border: Border {
                    color: theme_copy.border,
                    width: 1.0,
                    radius: RADIUS_SM.into(),
                },
            })
            .menu_style(move |_| iced::overlay::menu::Style {
                background: Background::Color(theme_copy.bg_surface),
                border: Border {
                    color: theme_copy.border,
                    width: 1.0,
                    radius: RADIUS_SM.into(),
                },
                text_color: theme_copy.text_normal,
                selected_text_color: iced::Color::WHITE,
                selected_background: Background::Color(theme_copy.primary),
                shadow: iced::Shadow::default(),
            });

            let mut col = column![
                text(label).size(FONT_MD).color(theme.text_normal),
                picker,
            ]
            .spacing(SPACE_XS);

            if let Some(desc) = description {
                col = col.push(text(desc).size(FONT_XS).color(theme.text_muted));
            }

            container(col)
                .width(Length::Fill)
                .padding(SPACE_SM)
                .style(move |_: &iced::Theme| iced::widget::container::Style {
                    background: Some(Background::Color(theme_copy.bg_base)),
                    border: Border {
                        radius: RADIUS_MD.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        }

        PreferenceField::Number {
            key,
            label,
            description,
            min,
            max,
            ..
        } => {
            let current_value = values
                .get(key)
                .and_then(|v| v.as_number())
                .map(|n| n.to_string())
                .unwrap_or_default();

            let key_owned = key.clone();
            let min_val = *min;
            let max_val = *max;

            let input = text_input("0", &current_value)
                .on_input(move |val| {
                    let num = val.parse::<f64>().unwrap_or(0.0);
                    let clamped = match (min_val, max_val) {
                        (Some(min), Some(max)) => num.clamp(min, max),
                        (Some(min), None) => num.max(min),
                        (None, Some(max)) => num.min(max),
                        (None, None) => num,
                    };
                    Message::PreferenceChanged(
                        uuid_owned.clone(),
                        key_owned.clone(),
                        PreferenceValue::Number(clamped),
                    )
                })
                .padding(SPACE_SM)
                .width(Length::Fixed(120.0))
                .style(move |_, status| {
                    let border_color = match status {
                        iced::widget::text_input::Status::Focused { .. } => theme_copy.primary,
                        _ => theme_copy.border,
                    };
                    iced::widget::text_input::Style {
                        background: Background::Color(theme_copy.bg_elevated),
                        border: Border {
                            color: border_color,
                            width: 1.0,
                            radius: RADIUS_SM.into(),
                        },
                        icon: theme_copy.text_normal,
                        placeholder: theme_copy.text_faint,
                        value: theme_copy.text_normal,
                        selection: iced::Color::from_rgba(
                            theme_copy.primary.r,
                            theme_copy.primary.g,
                            theme_copy.primary.b,
                            0.3,
                        ),
                    }
                });

            let range_text = match (min, max) {
                (Some(min), Some(max)) => format!("Range: {} - {}", min, max),
                (Some(min), None) => format!("Min: {}", min),
                (None, Some(max)) => format!("Max: {}", max),
                (None, None) => String::new(),
            };

            let mut col = column![
                text(label).size(FONT_MD).color(theme.text_normal),
                row![input, text(range_text).size(FONT_XS).color(theme.text_muted)]
                    .spacing(SPACE_SM)
                    .align_y(Alignment::Center),
            ]
            .spacing(SPACE_XS);

            if let Some(desc) = description {
                col = col.push(text(desc).size(FONT_XS).color(theme.text_muted));
            }

            container(col)
                .width(Length::Fill)
                .padding(SPACE_SM)
                .style(move |_: &iced::Theme| iced::widget::container::Style {
                    background: Some(Background::Color(theme_copy.bg_base)),
                    border: Border {
                        radius: RADIUS_MD.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        }
    }
}

impl std::fmt::Display for SelectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}
