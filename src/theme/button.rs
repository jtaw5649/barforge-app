use iced::widget::button;
use iced::{Background, Border, Theme};

use super::{lighten, shadow_sm, AppTheme, RADIUS_MD, RADIUS_SM};

type ButtonStyleFn = Box<dyn Fn(&Theme, button::Status) -> button::Style>;

pub fn primary(theme: AppTheme) -> ButtonStyleFn {
    Box::new(move |_, status| {
        let (background, text, shadow) = match status {
            button::Status::Hovered => (
                lighten(theme.accent, 0.08),
                theme.text_normal,
                shadow_sm(),
            ),
            button::Status::Pressed => (theme.accent, theme.text_normal, iced::Shadow::default()),
            button::Status::Disabled => (
                theme.bg_elevated,
                theme.text_faint,
                iced::Shadow::default(),
            ),
            _ => (theme.accent, theme.text_normal, shadow_sm()),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: text,
            border: Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: RADIUS_SM.into(),
            },
            shadow,
            snap: false,
        }
    })
}

pub fn secondary(theme: AppTheme) -> ButtonStyleFn {
    Box::new(move |_, status| {
        let (background, border_color, text) = match status {
            button::Status::Hovered => (theme.bg_elevated, theme.border_strong, theme.text_normal),
            button::Status::Pressed => (theme.bg_surface, theme.border_strong, theme.text_normal),
            button::Status::Disabled => (theme.bg_surface, theme.border_subtle, theme.text_faint),
            _ => (theme.bg_surface, theme.border_default, theme.text_muted),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: text,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    })
}

pub fn ghost(theme: AppTheme) -> ButtonStyleFn {
    Box::new(move |_, status| {
        let (background, text) = match status {
            button::Status::Hovered => (theme.bg_elevated, theme.text_normal),
            button::Status::Pressed => (theme.bg_surface, theme.text_normal),
            button::Status::Disabled => (iced::Color::TRANSPARENT, theme.text_faint),
            _ => (iced::Color::TRANSPARENT, theme.text_muted),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: text,
            border: Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    })
}

pub fn danger(theme: AppTheme) -> ButtonStyleFn {
    Box::new(move |_, status| {
        let (background, text) = match status {
            button::Status::Hovered => (theme.danger_hover, theme.text_normal),
            button::Status::Pressed => (theme.danger, theme.text_normal),
            button::Status::Disabled => (theme.bg_elevated, theme.text_faint),
            _ => (theme.danger, theme.text_normal),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: text,
            border: Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    })
}

pub fn success(theme: AppTheme) -> ButtonStyleFn {
    let success_hover = iced::Color::from_rgb(0.260, 0.720, 0.480);

    Box::new(move |_, status| {
        let (background, text) = match status {
            button::Status::Hovered => (success_hover, theme.text_normal),
            button::Status::Pressed => (theme.success, theme.text_normal),
            button::Status::Disabled => (theme.bg_elevated, theme.text_faint),
            _ => (theme.success, theme.text_normal),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: text,
            border: Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    })
}

pub fn sidebar(theme: AppTheme) -> ButtonStyleFn {
    Box::new(move |_, status| {
        let (background, text) = match status {
            button::Status::Hovered => (theme.sidebar_hover, theme.text_normal),
            button::Status::Pressed => (theme.sidebar_hover, theme.text_normal),
            button::Status::Disabled => (iced::Color::TRANSPARENT, theme.text_faint),
            _ => (iced::Color::TRANSPARENT, theme.text_muted),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: text,
            border: Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    })
}

pub fn sidebar_active(theme: AppTheme) -> ButtonStyleFn {
    Box::new(move |_, status| {
        let background = match status {
            button::Status::Hovered => iced::Color::from_rgba(
                theme.accent.r,
                theme.accent.g,
                theme.accent.b,
                0.18,
            ),
            _ => theme.sidebar_active,
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: theme.text_normal,
            border: Border {
                color: iced::Color::from_rgba(
                    theme.accent.r,
                    theme.accent.g,
                    theme.accent.b,
                    0.5,
                ),
                width: 0.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    })
}

pub fn card(theme: AppTheme) -> ButtonStyleFn {
    Box::new(move |_, status| {
        let (background, border_color, border_width) = match status {
            button::Status::Hovered => (theme.bg_elevated, theme.accent, 1.0),
            button::Status::Pressed => (theme.bg_surface, theme.accent, 1.0),
            _ => (iced::Color::TRANSPARENT, iced::Color::TRANSPARENT, 0.0),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: theme.text_normal,
            border: Border {
                color: border_color,
                width: border_width,
                radius: RADIUS_MD.into(),
            },
            ..Default::default()
        }
    })
}

pub fn theme_active(theme: AppTheme) -> ButtonStyleFn {
    Box::new(move |_, status| {
        let background = match status {
            button::Status::Hovered => lighten(theme.sidebar_active, 0.05),
            _ => theme.sidebar_active,
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: theme.accent,
            border: Border {
                color: theme.accent,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    })
}

pub fn theme_inactive(theme: AppTheme) -> ButtonStyleFn {
    Box::new(move |_, status| {
        let (background, border_color) = match status {
            button::Status::Hovered => (theme.sidebar_hover, theme.border_default),
            button::Status::Pressed => (theme.sidebar_hover, theme.border_default),
            _ => (iced::Color::TRANSPARENT, theme.border_subtle),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: theme.text_muted,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    })
}
