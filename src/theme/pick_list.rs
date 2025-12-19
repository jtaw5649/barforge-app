use iced::overlay::menu;
use iced::widget::pick_list::{Status, Style};
use iced::{Background, Border, Color, Shadow, Vector};

use super::AppTheme;

#[derive(Clone, Copy)]
pub struct PickListColors {
    pub surface: Color,
    pub text: Color,
    pub text_muted: Color,
    pub border: Color,
    pub primary: Color,
    pub menu_surface: Color,
    pub menu_border: Color,
    pub menu_text: Color,
    pub menu_selected_bg: Color,
}

impl PickListColors {
    #[must_use]
    pub fn from_theme(theme: &AppTheme) -> Self {
        Self {
            surface: theme.bg_surface,
            text: theme.text_normal,
            text_muted: theme.text_muted,
            border: theme.border_subtle,
            primary: theme.primary,
            menu_surface: theme.bg_surface,
            menu_border: theme.border_subtle,
            menu_text: theme.text_normal,
            menu_selected_bg: theme.primary,
        }
    }
}

pub fn pick_list_style(
    colors: PickListColors,
    radius: f32,
) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    move |_theme, status| {
        let border_color = match status {
            Status::Active => colors.border,
            Status::Hovered | Status::Opened { .. } => colors.primary,
        };
        Style {
            text_color: colors.text,
            placeholder_color: colors.text_muted,
            handle_color: colors.text_muted,
            background: Background::Color(colors.surface),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius.into(),
            },
        }
    }
}

pub fn menu_style(
    colors: PickListColors,
    radius: f32,
    shadow_alpha: f32,
    shadow_blur: f32,
) -> impl Fn(&iced::Theme) -> menu::Style + Clone {
    move |_theme| menu::Style {
        background: Background::Color(colors.menu_surface),
        border: Border {
            color: colors.menu_border,
            width: 1.0,
            radius: radius.into(),
        },
        text_color: colors.menu_text,
        selected_text_color: Color::WHITE,
        selected_background: Background::Color(colors.menu_selected_bg),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, shadow_alpha),
            offset: Vector::new(0.0, shadow_blur / 2.0),
            blur_radius: shadow_blur,
        },
    }
}
