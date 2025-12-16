use iced::Color;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct OmarchyPalette {
    pub background: Color,
    pub foreground: Color,
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
    pub bright_black: Color,
    pub bright_red: Color,
    pub bright_green: Color,
    pub bright_yellow: Color,
    pub bright_blue: Color,
    pub bright_magenta: Color,
    pub bright_cyan: Color,
    pub bright_white: Color,
}

fn omarchy_theme_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("omarchy/current/theme/waybar.css")
}

pub fn is_omarchy_available() -> bool {
    omarchy_theme_path().exists()
}

fn hex_to_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::from_rgb8(r, g, b))
}

fn parse_waybar_css(content: &str) -> HashMap<String, Color> {
    let mut colors = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with("@define-color") {
            continue;
        }

        let parts: Vec<&str> = line
            .trim_start_matches("@define-color")
            .trim()
            .trim_end_matches(';')
            .splitn(2, char::is_whitespace)
            .map(str::trim)
            .collect();

        if parts.len() == 2 {
            let name = parts[0];
            let value = parts[1];

            if value.starts_with('#')
                && let Some(color) = hex_to_color(value)
            {
                colors.insert(name.to_string(), color);
            }
        }
    }

    colors
}

pub fn load_omarchy_palette() -> Option<OmarchyPalette> {
    let path = omarchy_theme_path();
    let content = fs::read_to_string(&path).ok()?;
    let colors = parse_waybar_css(&content);

    let fallback = Color::from_rgb8(128, 128, 128);

    Some(OmarchyPalette {
        background: colors.get("background").copied().unwrap_or(fallback),
        foreground: colors.get("foreground").copied().unwrap_or(fallback),
        black: colors.get("black").copied().unwrap_or(fallback),
        red: colors.get("red").copied().unwrap_or(fallback),
        green: colors.get("green").copied().unwrap_or(fallback),
        yellow: colors.get("yellow").copied().unwrap_or(fallback),
        blue: colors.get("blue").copied().unwrap_or(fallback),
        magenta: colors.get("magenta").copied().unwrap_or(fallback),
        cyan: colors.get("cyan").copied().unwrap_or(fallback),
        white: colors.get("white").copied().unwrap_or(fallback),
        bright_black: colors.get("bright_black").copied().unwrap_or(fallback),
        bright_red: colors.get("bright_red").copied().unwrap_or(fallback),
        bright_green: colors.get("bright_green").copied().unwrap_or(fallback),
        bright_yellow: colors.get("bright_yellow").copied().unwrap_or(fallback),
        bright_blue: colors.get("bright_blue").copied().unwrap_or(fallback),
        bright_magenta: colors.get("bright_magenta").copied().unwrap_or(fallback),
        bright_cyan: colors.get("bright_cyan").copied().unwrap_or(fallback),
        bright_white: colors.get("bright_white").copied().unwrap_or(fallback),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_color() {
        let color = hex_to_color("#1e1e2e").unwrap();
        assert_eq!(color.r, 30.0 / 255.0);
        assert_eq!(color.g, 30.0 / 255.0);
        assert_eq!(color.b, 46.0 / 255.0);
    }

    #[test]
    fn test_parse_waybar_css() {
        let css = r#"
@define-color background #1e1e2e;
@define-color foreground #cdd6f4;
@define-color blue #89b4fa;
/* Comment line */
@define-color teal @cyan;
"#;
        let colors = parse_waybar_css(css);
        assert!(colors.contains_key("background"));
        assert!(colors.contains_key("foreground"));
        assert!(colors.contains_key("blue"));
        assert!(!colors.contains_key("teal"));
    }
}
