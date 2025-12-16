use iced::widget::container;
use iced::{Background, Border, Theme};

use super::{AppTheme, RADIUS_MD};

pub fn sidebar(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.sidebar_bg)),
        border: Border {
            color: theme.border_subtle,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn card(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_surface)),
        border: Border {
            color: theme.border_subtle,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn card_hover(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_elevated)),
        border: Border {
            color: theme.accent,
            width: 2.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn list_item(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_surface)),
        border: Border {
            color: theme.border_subtle,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn page(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_base)),
        border: Border::default(),
        ..Default::default()
    }
}

pub fn panel(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_surface)),
        border: Border::default(),
        ..Default::default()
    }
}

pub fn floating(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_floating)),
        border: Border {
            color: theme.border_default,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn overlay(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_overlay)),
        ..Default::default()
    }
}

pub fn input(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_elevated)),
        border: Border {
            color: theme.border_default,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn notification_success(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.success)),
        border: Border {
            color: theme.success,
            width: 0.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn notification_error(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.danger)),
        border: Border {
            color: theme.danger,
            width: 0.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn notification_info(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_floating)),
        border: Border {
            color: theme.info,
            width: 2.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn notification_warning(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.warning)),
        border: Border {
            color: theme.warning,
            width: 0.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn modal_backdrop(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    overlay(app_theme)
}

pub fn modal(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    floating(app_theme)
}

pub fn search_bar(app_theme: &AppTheme) -> impl Fn(&Theme) -> container::Style {
    let theme = *app_theme;
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_elevated)),
        border: Border {
            color: theme.border_default,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

pub fn badge(color: iced::Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn badge_outline(color: iced::Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(iced::Color::from_rgba(color.r, color.g, color.b, 0.15))),
        border: Border {
            color,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}
