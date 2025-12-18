use iced::{Color, Shadow, Vector};

pub const BG_BASE: Color = Color::from_rgb(0.098, 0.106, 0.122);
pub const BG_SURFACE: Color = Color::from_rgb(0.133, 0.141, 0.157);
pub const BG_ELEVATED: Color = Color::from_rgb(0.165, 0.176, 0.196);
pub const BG_FLOATING: Color = Color::from_rgb(0.196, 0.208, 0.231);
pub const BG_OVERLAY: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.75);

pub const SIDEBAR_BG: Color = BG_SURFACE;
pub const SIDEBAR_HOVER: Color = Color::from_rgb(0.165, 0.176, 0.196);
pub const SIDEBAR_ACTIVE: Color = Color::from_rgba(0.380, 0.490, 0.980, 0.15);

pub const ACCENT: Color = Color::from_rgb(0.380, 0.490, 0.980);
pub const ACCENT_HOVER: Color = Color::from_rgb(0.478, 0.573, 0.984);
pub const ACCENT_PRESSED: Color = Color::from_rgb(0.290, 0.388, 0.831);
pub const ACCENT_MUTED: Color = Color::from_rgba(0.380, 0.490, 0.980, 0.15);

pub const TEXT_NORMAL: Color = Color::from_rgb(0.965, 0.969, 0.976);
pub const TEXT_MUTED: Color = Color::from_rgb(0.690, 0.710, 0.745);
pub const TEXT_FAINT: Color = Color::from_rgb(0.322, 0.345, 0.400);

pub const SUCCESS: Color = Color::from_rgb(0.176, 0.800, 0.440);
pub const SUCCESS_MUTED: Color = Color::from_rgba(0.176, 0.800, 0.440, 0.15);
pub const WARNING: Color = Color::from_rgb(0.945, 0.769, 0.059);
pub const DANGER: Color = Color::from_rgb(0.906, 0.298, 0.235);
pub const DANGER_HOVER: Color = Color::from_rgb(0.753, 0.224, 0.169);
pub const INFO: Color = Color::from_rgb(0.204, 0.596, 0.859);

pub const BORDER_SUBTLE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.05);
pub const BORDER_DEFAULT: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
pub const BORDER_STRONG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.12);

pub const LIGHT_BG_BASE: Color = Color::from_rgb(0.976, 0.976, 0.980);
pub const LIGHT_BG_SURFACE: Color = Color::from_rgb(1.0, 1.0, 1.0);
pub const LIGHT_BG_ELEVATED: Color = Color::from_rgb(0.965, 0.965, 0.973);
pub const LIGHT_BG_FLOATING: Color = Color::from_rgb(1.0, 1.0, 1.0);
pub const LIGHT_BG_OVERLAY: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.35);
pub const LIGHT_SIDEBAR_BG: Color = Color::from_rgb(0.965, 0.965, 0.973);
pub const LIGHT_SIDEBAR_HOVER: Color = Color::from_rgb(0.941, 0.941, 0.953);
pub const LIGHT_TEXT_NORMAL: Color = Color::from_rgb(0.094, 0.102, 0.114);
pub const LIGHT_TEXT_MUTED: Color = Color::from_rgb(0.380, 0.400, 0.440);
pub const LIGHT_TEXT_FAINT: Color = Color::from_rgb(0.520, 0.540, 0.580);
pub const LIGHT_BORDER_SUBTLE: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.04);
pub const LIGHT_BORDER_DEFAULT: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);
pub const LIGHT_BORDER_STRONG: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.12);

pub const BADGE_SYSTEM: Color = Color::from_rgba(0.380, 0.490, 0.820, 0.25);
pub const BADGE_HARDWARE: Color = Color::from_rgba(0.259, 0.631, 0.557, 0.25);
pub const BADGE_NETWORK: Color = Color::from_rgba(0.298, 0.600, 0.780, 0.25);
pub const BADGE_AUDIO: Color = Color::from_rgba(0.800, 0.420, 0.420, 0.25);
pub const BADGE_POWER: Color = Color::from_rgba(0.820, 0.580, 0.220, 0.25);
pub const BADGE_TIME: Color = Color::from_rgba(0.580, 0.440, 0.760, 0.25);
pub const BADGE_WORKSPACE: Color = Color::from_rgba(0.310, 0.620, 0.480, 0.25);
pub const BADGE_WINDOW: Color = Color::from_rgba(0.800, 0.500, 0.280, 0.25);
pub const BADGE_TRAY: Color = Color::from_rgba(0.680, 0.440, 0.640, 0.25);
pub const BADGE_WEATHER: Color = Color::from_rgba(0.380, 0.600, 0.780, 0.25);
pub const BADGE_PRODUCTIVITY: Color = Color::from_rgba(0.760, 0.440, 0.520, 0.25);
pub const BADGE_MEDIA: Color = Color::from_rgba(0.720, 0.360, 0.440, 0.25);
pub const BADGE_CUSTOM: Color = Color::from_rgba(0.480, 0.500, 0.560, 0.25);

pub const BADGE_TEXT_SYSTEM: Color = Color::from_rgb(0.580, 0.660, 0.950);
pub const BADGE_TEXT_HARDWARE: Color = Color::from_rgb(0.400, 0.780, 0.700);
pub const BADGE_TEXT_NETWORK: Color = Color::from_rgb(0.480, 0.760, 0.920);
pub const BADGE_TEXT_AUDIO: Color = Color::from_rgb(0.950, 0.600, 0.600);
pub const BADGE_TEXT_POWER: Color = Color::from_rgb(0.960, 0.750, 0.400);
pub const BADGE_TEXT_TIME: Color = Color::from_rgb(0.750, 0.620, 0.900);
pub const BADGE_TEXT_WORKSPACE: Color = Color::from_rgb(0.480, 0.780, 0.640);
pub const BADGE_TEXT_WINDOW: Color = Color::from_rgb(0.950, 0.680, 0.480);
pub const BADGE_TEXT_TRAY: Color = Color::from_rgb(0.850, 0.620, 0.800);
pub const BADGE_TEXT_WEATHER: Color = Color::from_rgb(0.560, 0.780, 0.920);
pub const BADGE_TEXT_PRODUCTIVITY: Color = Color::from_rgb(0.920, 0.620, 0.700);
pub const BADGE_TEXT_MEDIA: Color = Color::from_rgb(0.900, 0.560, 0.640);
pub const BADGE_TEXT_CUSTOM: Color = Color::from_rgb(0.680, 0.700, 0.760);

pub const SPACE_2XS: f32 = 2.0;
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;
pub const SPACE_2XL: f32 = 32.0;
pub const SPACE_3XL: f32 = 48.0;

pub const FONT_2XS: f32 = 10.0;
pub const FONT_XS: f32 = 11.0;
pub const FONT_SM: f32 = 12.0;
pub const FONT_MD: f32 = 14.0;
pub const FONT_LG: f32 = 16.0;
pub const FONT_XL: f32 = 18.0;
pub const FONT_2XL: f32 = 22.0;
pub const FONT_3XL: f32 = 28.0;

pub const RADIUS_XS: f32 = 4.0;
pub const RADIUS_SM: f32 = 6.0;
pub const RADIUS_MD: f32 = 8.0;
pub const RADIUS_LG: f32 = 12.0;
pub const RADIUS_XL: f32 = 16.0;

pub const SIDEBAR_WIDTH: f32 = 240.0;
pub const CARD_MIN_WIDTH: f32 = 280.0;
pub const CARD_MAX_WIDTH: f32 = 340.0;
pub const MODAL_WIDTH: f32 = 520.0;
pub const NOTIFICATION_WIDTH: f32 = 380.0;
pub const HEADER_HEIGHT: f32 = 64.0;
pub const NAV_ITEM_HEIGHT: f32 = 44.0;

pub const SEARCH_PANEL_WIDTH: f32 = 320.0;
pub const FILTER_DROPDOWN_WIDTH: f32 = 250.0;
pub const PREFERENCES_MODAL_WIDTH: f32 = 500.0;
pub const PREFERENCES_MODAL_MAX_HEIGHT: f32 = 600.0;
pub const CONFIRMATION_DIALOG_WIDTH: f32 = 350.0;
pub const NUMBER_INPUT_WIDTH: f32 = 120.0;
pub const SKELETON_BAR_SM: f32 = 40.0;
pub const SKELETON_BAR_MD: f32 = 60.0;
pub const DESCRIPTION_HEIGHT: f32 = 48.0;
pub const SCREENSHOT_LOADING_HEIGHT: f32 = 200.0;
pub const SCREENSHOT_MAX_HEIGHT: f32 = 400.0;
pub const SCREENSHOT_FAILED_HEIGHT: f32 = 120.0;
pub const SETTINGS_CONTENT_MAX_WIDTH: f32 = 600.0;
pub const DETAIL_CONTENT_MAX_WIDTH: f32 = 900.0;

pub const ICON_XS: f32 = 12.0;
pub const ICON_SM: f32 = 16.0;
pub const ICON_MD: f32 = 20.0;
pub const ICON_LG: f32 = 24.0;
pub const ICON_XL: f32 = 32.0;
pub const ICON_2XL: f32 = 48.0;

pub fn shadow_sm() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
        offset: Vector::new(0.0, 1.0),
        blur_radius: 3.0,
    }
}

pub fn shadow_md() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.20),
        offset: Vector::new(0.0, 4.0),
        blur_radius: 12.0,
    }
}

pub fn shadow_lg() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.28),
        offset: Vector::new(0.0, 8.0),
        blur_radius: 24.0,
    }
}

pub fn shadow_xl() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.40),
        offset: Vector::new(0.0, 16.0),
        blur_radius: 48.0,
    }
}

pub fn shadow_hover() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
        offset: Vector::new(0.0, 6.0),
        blur_radius: 16.0,
    }
}

pub fn shadow_glow(color: Color) -> Shadow {
    Shadow {
        color: Color::from_rgba(color.r, color.g, color.b, 0.4),
        offset: Vector::new(0.0, 0.0),
        blur_radius: 20.0,
    }
}

pub fn shadow_accent_glow() -> Shadow {
    shadow_glow(ACCENT)
}

pub fn shadow_success_glow() -> Shadow {
    shadow_glow(SUCCESS)
}

pub fn darken(color: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0);
    Color::from_rgba(
        color.r * (1.0 - factor),
        color.g * (1.0 - factor),
        color.b * (1.0 - factor),
        color.a,
    )
}

pub fn lighten(color: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0);
    Color::from_rgba(
        color.r + (1.0 - color.r) * factor,
        color.g + (1.0 - color.g) * factor,
        color.b + (1.0 - color.b) * factor,
        color.a,
    )
}

pub fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::from_rgba(color.r, color.g, color.b, alpha.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    #[test]
    fn test_darken_clamping() {
        let color = Color::from_rgb(1.0, 1.0, 1.0);
        let result = darken(color, 1.5);
        assert!(approx_eq(result.r, 0.0));
    }
}
