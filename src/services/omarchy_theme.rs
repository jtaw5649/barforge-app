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

fn omarchy_theme_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("omarchy/current/theme")
}

fn alacritty_toml_path() -> PathBuf {
    omarchy_theme_dir().join("alacritty.toml")
}

pub fn is_omarchy_available() -> bool {
    alacritty_toml_path().exists()
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

fn parse_alacritty_toml(content: &str) -> HashMap<String, Color> {
    let mut colors = HashMap::new();

    let Ok(value) = content.parse::<toml::Table>() else {
        return colors;
    };

    let Some(colors_table) = value.get("colors").and_then(|v| v.as_table()) else {
        return colors;
    };

    if let Some(primary) = colors_table.get("primary").and_then(|v| v.as_table()) {
        for (name, value) in primary {
            if let Some(hex) = value.as_str()
                && let Some(color) = hex_to_color(hex)
            {
                colors.insert(name.clone(), color);
            }
        }
    }

    if let Some(normal) = colors_table.get("normal").and_then(|v| v.as_table()) {
        for (name, value) in normal {
            if let Some(hex) = value.as_str()
                && let Some(color) = hex_to_color(hex)
            {
                colors.insert(name.clone(), color);
            }
        }
    }

    if let Some(bright) = colors_table.get("bright").and_then(|v| v.as_table()) {
        for (name, value) in bright {
            if let Some(hex) = value.as_str()
                && let Some(color) = hex_to_color(hex)
            {
                colors.insert(format!("bright_{name}"), color);
            }
        }
    }

    colors
}

pub fn load_omarchy_palette() -> Option<OmarchyPalette> {
    let content = fs::read_to_string(alacritty_toml_path()).ok()?;
    let colors = parse_alacritty_toml(&content);
    if colors.is_empty() {
        return None;
    }

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

    mod hex_to_color_tests {
        use super::*;

        #[test]
        fn test_with_hash_prefix() {
            let color = hex_to_color("#1e1e2e").unwrap();
            assert_eq!(color.r, 30.0 / 255.0);
            assert_eq!(color.g, 30.0 / 255.0);
            assert_eq!(color.b, 46.0 / 255.0);
        }

        #[test]
        fn test_without_hash_prefix() {
            let color = hex_to_color("89b4fa").unwrap();
            assert_eq!(color.r, 137.0 / 255.0);
            assert_eq!(color.g, 180.0 / 255.0);
            assert_eq!(color.b, 250.0 / 255.0);
        }

        #[test]
        fn test_invalid_length() {
            assert!(hex_to_color("#fff").is_none());
            assert!(hex_to_color("1234").is_none());
        }

        #[test]
        fn test_invalid_characters() {
            assert!(hex_to_color("#gggggg").is_none());
        }
    }

    mod parse_alacritty_toml_tests {
        use super::*;

        #[test]
        fn test_parses_valid_toml() {
            let toml_content = r##"
[colors.primary]
background = "#1e1e2e"
foreground = "#cdd6f4"

[colors.normal]
black = "#45475a"
red = "#f38ba8"
green = "#a6e3a1"
yellow = "#f9e2af"
blue = "#89b4fa"
magenta = "#f5c2e7"
cyan = "#94e2d5"
white = "#bac2de"

[colors.bright]
black = "#585b70"
red = "#f38ba8"
green = "#a6e3a1"
yellow = "#f9e2af"
blue = "#89b4fa"
magenta = "#f5c2e7"
cyan = "#94e2d5"
white = "#a6adc8"
"##;
            let colors = parse_alacritty_toml(toml_content);
            assert!(colors.contains_key("background"));
            assert!(colors.contains_key("foreground"));
            assert!(colors.contains_key("black"));
            assert!(colors.contains_key("blue"));
            assert!(colors.contains_key("bright_black"));
            assert!(colors.contains_key("bright_white"));
        }

        #[test]
        fn test_parses_primary_colors() {
            let toml_content = r##"
[colors.primary]
background = "#1e1e2e"
foreground = "#cdd6f4"
"##;
            let colors = parse_alacritty_toml(toml_content);
            let bg = colors.get("background").unwrap();
            assert_eq!(bg.r, 30.0 / 255.0);
            assert_eq!(bg.g, 30.0 / 255.0);
            assert_eq!(bg.b, 46.0 / 255.0);
        }

        #[test]
        fn test_handles_empty_content() {
            let colors = parse_alacritty_toml("");
            assert!(colors.is_empty());
        }

        #[test]
        fn test_handles_invalid_toml() {
            let colors = parse_alacritty_toml("not valid toml { }}");
            assert!(colors.is_empty());
        }

        #[test]
        fn test_handles_missing_colors_section() {
            let toml_content = r##"
[font]
size = 12.0
"##;
            let colors = parse_alacritty_toml(toml_content);
            assert!(colors.is_empty());
        }
    }
}
