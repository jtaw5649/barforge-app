use iced::widget::image::{self, Image};
use iced::widget::svg::{Handle, Svg};
use iced::{Color, Length};
use once_cell::sync::Lazy;

const BACK_SVG: &[u8] = include_bytes!("../assets/icons/ui/back.svg");
const BROWSE_SVG: &[u8] = include_bytes!("../assets/icons/ui/browse.svg");
const INSTALLED_SVG: &[u8] = include_bytes!("../assets/icons/ui/installed.svg");
const UPDATES_SVG: &[u8] = include_bytes!("../assets/icons/ui/updates.svg");
const SEARCH_SVG: &[u8] = include_bytes!("../assets/icons/ui/search.svg");
const DOWNLOAD_SVG: &[u8] = include_bytes!("../assets/icons/ui/download.svg");
const CHECK_SVG: &[u8] = include_bytes!("../assets/icons/ui/check.svg");
const ERROR_SVG: &[u8] = include_bytes!("../assets/icons/ui/error.svg");
const INFO_SVG: &[u8] = include_bytes!("../assets/icons/ui/info.svg");
const APP_LOGO_SVG: &[u8] = include_bytes!("../assets/icons/ui/app-logo.svg");
const SUN_SVG: &[u8] = include_bytes!("../assets/icons/ui/sun.svg");
const MOON_SVG: &[u8] = include_bytes!("../assets/icons/ui/moon.svg");
const SETTINGS_SVG: &[u8] = include_bytes!("../assets/icons/ui/settings.svg");
const GRID_SVG: &[u8] = include_bytes!("../assets/icons/ui/grid.svg");
const LIST_SVG: &[u8] = include_bytes!("../assets/icons/ui/list.svg");
const STAR_SVG: &[u8] = include_bytes!("../assets/icons/ui/star.svg");
const STAR_HALF_SVG: &[u8] = include_bytes!("../assets/icons/ui/star-half.svg");
const STAR_EMPTY_SVG: &[u8] = include_bytes!("../assets/icons/ui/star-empty.svg");
const OMARCHY_PNG: &[u8] = include_bytes!("../assets/icons/ui/omarchy.png");

static OMARCHY_HANDLE: Lazy<image::Handle> =
    Lazy::new(|| image::Handle::from_bytes(OMARCHY_PNG.to_vec()));

#[derive(Debug, Clone, Copy)]
pub enum Icon {
    Back,
    Browse,
    Installed,
    Updates,
    Search,
    Download,
    Check,
    Error,
    Info,
    AppLogo,
    Sun,
    Moon,
    Omarchy,
    Settings,
    Grid,
    List,
    Star,
    StarHalf,
    StarEmpty,
}

impl Icon {
    fn svg_bytes(self) -> Option<&'static [u8]> {
        match self {
            Icon::Back => Some(BACK_SVG),
            Icon::Browse => Some(BROWSE_SVG),
            Icon::Installed => Some(INSTALLED_SVG),
            Icon::Updates => Some(UPDATES_SVG),
            Icon::Search => Some(SEARCH_SVG),
            Icon::Download => Some(DOWNLOAD_SVG),
            Icon::Check => Some(CHECK_SVG),
            Icon::Error => Some(ERROR_SVG),
            Icon::Info => Some(INFO_SVG),
            Icon::AppLogo => Some(APP_LOGO_SVG),
            Icon::Sun => Some(SUN_SVG),
            Icon::Moon => Some(MOON_SVG),
            Icon::Settings => Some(SETTINGS_SVG),
            Icon::Grid => Some(GRID_SVG),
            Icon::List => Some(LIST_SVG),
            Icon::Star => Some(STAR_SVG),
            Icon::StarHalf => Some(STAR_HALF_SVG),
            Icon::StarEmpty => Some(STAR_EMPTY_SVG),
            Icon::Omarchy => None,
        }
    }

    pub fn svg(self, size: f32) -> Svg<'static> {
        let bytes = self.svg_bytes().unwrap_or(SETTINGS_SVG);
        Svg::new(Handle::from_memory(bytes))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
    }

    pub fn colored(self, size: f32, color: Color) -> Svg<'static> {
        let bytes = self.svg_bytes().unwrap_or(SETTINGS_SVG);
        Svg::new(Handle::from_memory(bytes))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .style(move |_, _| iced::widget::svg::Style { color: Some(color) })
    }

    pub fn image(self, size: f32) -> Image<image::Handle> {
        let handle = match self {
            Icon::Omarchy => OMARCHY_HANDLE.clone(),
            Icon::AppLogo => image::Handle::from_bytes(APP_LOGO_SVG.to_vec()),
            _ => image::Handle::from_bytes(self.svg_bytes().unwrap_or(SETTINGS_SVG).to_vec()),
        };
        Image::new(handle)
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
    }
}

pub fn omarchy_icon(size: f32) -> Image<image::Handle> {
    Image::new(OMARCHY_HANDLE.clone())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
}

pub fn app_logo(size: f32) -> Svg<'static> {
    Svg::new(Handle::from_memory(APP_LOGO_SVG))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
}
