pub mod button;
pub mod checkbox;
pub mod container;
pub mod palette;

use iced::gradient::Linear;
use iced::{Color, Radians, Shadow};

use crate::services::OmarchyPalette;
pub use palette::*;

pub const SPACING_2XS: f32 = SPACE_2XS;
pub const SPACING_XS: f32 = SPACE_XS;
pub const SPACING_SM: f32 = SPACE_SM;
pub const SPACING_MD: f32 = SPACE_MD;
pub const SPACING_LG: f32 = SPACE_LG;
pub const SPACING_XL: f32 = SPACE_XL;

pub const FONT_SIZE_2XS: f32 = FONT_2XS;
pub const FONT_SIZE_XS: f32 = FONT_XS;
pub const FONT_SIZE_SM: f32 = FONT_SM;
pub const FONT_SIZE_MD: f32 = FONT_MD;
pub const FONT_SIZE_LG: f32 = FONT_LG;
pub const FONT_SIZE_XL: f32 = FONT_XL;
pub const FONT_SIZE_2XL: f32 = FONT_2XL;
pub const FONT_SIZE_XXL: f32 = FONT_XL;

pub const CARD_WIDTH: f32 = CARD_MIN_WIDTH;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
    Omarchy,
}

#[derive(Debug, Clone, Copy)]
pub struct AppTheme {
    pub bg_base: Color,
    pub bg_surface: Color,
    pub bg_elevated: Color,
    pub bg_floating: Color,
    pub bg_overlay: Color,

    pub accent: Color,
    pub accent_hover: Color,
    pub accent_muted: Color,

    pub text_normal: Color,
    pub text_muted: Color,
    pub text_faint: Color,

    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub danger_hover: Color,
    pub info: Color,

    pub border_subtle: Color,
    pub border_default: Color,
    pub border_strong: Color,

    pub sidebar_bg: Color,
    pub sidebar_hover: Color,
    pub sidebar_active: Color,

    pub primary: Color,
    pub primary_hover: Color,
    pub secondary: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub background: Color,
    pub border: Color,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl AppTheme {
    pub const fn dark() -> Self {
        Self {
            bg_base: BG_BASE,
            bg_surface: BG_SURFACE,
            bg_elevated: BG_ELEVATED,
            bg_floating: BG_FLOATING,
            bg_overlay: BG_OVERLAY,

            accent: ACCENT,
            accent_hover: ACCENT_HOVER,
            accent_muted: ACCENT_MUTED,

            text_normal: TEXT_NORMAL,
            text_muted: TEXT_MUTED,
            text_faint: TEXT_FAINT,

            success: SUCCESS,
            warning: WARNING,
            danger: DANGER,
            danger_hover: DANGER_HOVER,
            info: INFO,

            border_subtle: BORDER_SUBTLE,
            border_default: BORDER_DEFAULT,
            border_strong: BORDER_STRONG,

            sidebar_bg: SIDEBAR_BG,
            sidebar_hover: SIDEBAR_HOVER,
            sidebar_active: SIDEBAR_ACTIVE,

            primary: ACCENT,
            primary_hover: ACCENT_HOVER,
            secondary: ACCENT,
            text: TEXT_NORMAL,
            text_secondary: TEXT_MUTED,
            surface: BG_SURFACE,
            surface_hover: BG_ELEVATED,
            background: BG_BASE,
            border: BORDER_DEFAULT,
        }
    }

    pub const fn light() -> Self {
        Self {
            bg_base: LIGHT_BG_BASE,
            bg_surface: LIGHT_BG_SURFACE,
            bg_elevated: LIGHT_BG_ELEVATED,
            bg_floating: LIGHT_BG_FLOATING,
            bg_overlay: LIGHT_BG_OVERLAY,

            accent: ACCENT,
            accent_hover: ACCENT_HOVER,
            accent_muted: ACCENT_MUTED,

            text_normal: LIGHT_TEXT_NORMAL,
            text_muted: LIGHT_TEXT_MUTED,
            text_faint: LIGHT_TEXT_FAINT,

            success: SUCCESS,
            warning: WARNING,
            danger: DANGER,
            danger_hover: DANGER_HOVER,
            info: INFO,

            border_subtle: LIGHT_BORDER_SUBTLE,
            border_default: LIGHT_BORDER_DEFAULT,
            border_strong: LIGHT_BORDER_STRONG,

            sidebar_bg: LIGHT_SIDEBAR_BG,
            sidebar_hover: LIGHT_SIDEBAR_HOVER,
            sidebar_active: ACCENT_MUTED,

            primary: ACCENT,
            primary_hover: ACCENT_HOVER,
            secondary: ACCENT,
            text: LIGHT_TEXT_NORMAL,
            text_secondary: LIGHT_TEXT_MUTED,
            surface: LIGHT_BG_SURFACE,
            surface_hover: LIGHT_BG_ELEVATED,
            background: LIGHT_BG_BASE,
            border: LIGHT_BORDER_DEFAULT,
        }
    }

    pub fn from_mode(mode: ThemeMode, system_is_dark: bool) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
            ThemeMode::Omarchy => Self::dark(),
            ThemeMode::System => {
                if system_is_dark {
                    Self::dark()
                } else {
                    Self::light()
                }
            }
        }
    }

    pub fn from_omarchy(palette: &OmarchyPalette) -> Self {
        let surface = lighten(palette.background, 0.06);
        let elevated = lighten(palette.background, 0.12);
        let floating = lighten(palette.background, 0.18);

        Self {
            bg_base: palette.background,
            bg_surface: surface,
            bg_elevated: elevated,
            bg_floating: floating,
            bg_overlay: with_alpha(palette.black, 0.65),

            accent: palette.blue,
            accent_hover: palette.bright_blue,
            accent_muted: with_alpha(palette.blue, 0.15),

            text_normal: palette.foreground,
            text_muted: darken(palette.foreground, 0.25),
            text_faint: darken(palette.foreground, 0.50),

            success: palette.green,
            warning: palette.yellow,
            danger: palette.red,
            danger_hover: palette.bright_red,
            info: palette.cyan,

            border_subtle: with_alpha(palette.foreground, 0.06),
            border_default: with_alpha(palette.foreground, 0.10),
            border_strong: with_alpha(palette.foreground, 0.16),

            sidebar_bg: darken(palette.background, 0.25),
            sidebar_hover: lighten(palette.background, 0.08),
            sidebar_active: with_alpha(palette.blue, 0.15),

            primary: palette.blue,
            primary_hover: palette.bright_blue,
            secondary: palette.magenta,
            text: palette.foreground,
            text_secondary: darken(palette.foreground, 0.25),
            surface,
            surface_hover: elevated,
            background: palette.background,
            border: with_alpha(palette.foreground, 0.10),
        }
    }
}

pub fn shadow_subtle() -> Shadow {
    shadow_sm()
}

pub fn shadow_card() -> Shadow {
    shadow_md()
}

pub fn shadow_elevated() -> Shadow {
    shadow_lg()
}

pub fn shadow_glow_primary(theme: &AppTheme) -> Shadow {
    shadow_glow(theme.accent)
}

pub fn shadow_glow_success(theme: &AppTheme) -> Shadow {
    shadow_glow(theme.success)
}

pub fn gradient_primary(theme: &AppTheme) -> Linear {
    Linear::new(Radians(std::f32::consts::PI * 0.75))
        .add_stop(0.0, theme.accent)
        .add_stop(1.0, theme.accent_hover)
}

pub fn gradient_sidebar(theme: &AppTheme) -> Linear {
    Linear::new(Radians(std::f32::consts::PI * 0.5))
        .add_stop(0.0, lighten(theme.sidebar_bg, 0.08))
        .add_stop(0.3, theme.sidebar_bg)
        .add_stop(1.0, darken(theme.sidebar_bg, 0.15))
}

pub fn gradient_card_hover(theme: &AppTheme) -> Linear {
    Linear::new(Radians(std::f32::consts::PI * 0.75))
        .add_stop(0.0, lighten(theme.bg_elevated, 0.06))
        .add_stop(0.5, theme.bg_elevated)
        .add_stop(1.0, darken(theme.bg_surface, 0.03))
}
