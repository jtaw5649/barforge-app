use iced::Color;

use crate::domain::ModuleCategory;
use crate::theme::palette;

pub fn badge_color(category: ModuleCategory) -> Color {
    match category {
        ModuleCategory::System => palette::BADGE_SYSTEM,
        ModuleCategory::Hardware => palette::BADGE_HARDWARE,
        ModuleCategory::Network => palette::BADGE_NETWORK,
        ModuleCategory::Audio => palette::BADGE_AUDIO,
        ModuleCategory::Power => palette::BADGE_POWER,
        ModuleCategory::Time => palette::BADGE_TIME,
        ModuleCategory::Workspace => palette::BADGE_WORKSPACE,
        ModuleCategory::Window => palette::BADGE_WINDOW,
        ModuleCategory::Tray => palette::BADGE_TRAY,
        ModuleCategory::Weather => palette::BADGE_WEATHER,
        ModuleCategory::Productivity => palette::BADGE_PRODUCTIVITY,
        ModuleCategory::Media => palette::BADGE_MEDIA,
        ModuleCategory::Custom => palette::BADGE_CUSTOM,
    }
}

pub fn badge_text_color(category: ModuleCategory) -> Color {
    match category {
        ModuleCategory::System => palette::BADGE_TEXT_SYSTEM,
        ModuleCategory::Hardware => palette::BADGE_TEXT_HARDWARE,
        ModuleCategory::Network => palette::BADGE_TEXT_NETWORK,
        ModuleCategory::Audio => palette::BADGE_TEXT_AUDIO,
        ModuleCategory::Power => palette::BADGE_TEXT_POWER,
        ModuleCategory::Time => palette::BADGE_TEXT_TIME,
        ModuleCategory::Workspace => palette::BADGE_TEXT_WORKSPACE,
        ModuleCategory::Window => palette::BADGE_TEXT_WINDOW,
        ModuleCategory::Tray => palette::BADGE_TEXT_TRAY,
        ModuleCategory::Weather => palette::BADGE_TEXT_WEATHER,
        ModuleCategory::Productivity => palette::BADGE_TEXT_PRODUCTIVITY,
        ModuleCategory::Media => palette::BADGE_TEXT_MEDIA,
        ModuleCategory::Custom => palette::BADGE_TEXT_CUSTOM,
    }
}
