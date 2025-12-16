use iced::widget::checkbox::{Status, Style};
use iced::{Background, Border, Color};

use super::{AppTheme, RADIUS_SM};

pub fn themed(theme: AppTheme) -> impl Fn(&iced::Theme, Status) -> Style {
    move |_iced_theme, status| {
        let is_checked = matches!(
            status,
            Status::Active { is_checked: true }
                | Status::Hovered { is_checked: true }
                | Status::Disabled { is_checked: true }
        );

        let (background, border_color, icon_color) = match status {
            Status::Active { .. } => {
                if is_checked {
                    (theme.primary, theme.primary, Color::WHITE)
                } else {
                    (theme.surface, theme.border, theme.text_muted)
                }
            }
            Status::Hovered { .. } => {
                if is_checked {
                    (theme.primary_hover, theme.primary_hover, Color::WHITE)
                } else {
                    (theme.surface_hover, theme.primary, theme.text)
                }
            }
            Status::Disabled { .. } => {
                if is_checked {
                    (theme.text_muted, theme.text_muted, theme.surface)
                } else {
                    (theme.surface, theme.border, theme.text_faint)
                }
            }
        };

        Style {
            background: Background::Color(background),
            icon_color,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            text_color: Some(theme.text_normal),
        }
    }
}
