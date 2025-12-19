use iced::widget::container;
use iced::{Background, Border, Theme};

use super::{AppTheme, RADIUS_MD};

pub fn sidebar(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn card(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn card_hover(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn list_item(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn page(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_base)),
        border: Border::default(),
        ..Default::default()
    }
}

pub fn panel(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_surface)),
        border: Border::default(),
        ..Default::default()
    }
}

pub fn floating(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn overlay(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(theme.bg_overlay)),
        ..Default::default()
    }
}

pub fn input(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn notification_success(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn notification_error(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn notification_info(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn notification_warning(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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

pub fn modal_backdrop(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
    overlay(theme)
}

pub fn modal(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
    floating(theme)
}

pub fn search_bar(theme: AppTheme) -> impl Fn(&Theme) -> container::Style {
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
        background: Some(Background::Color(iced::Color::from_rgba(
            color.r, color.g, color.b, 0.15,
        ))),
        border: Border {
            color,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}
