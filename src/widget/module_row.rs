use iced::widget::{button, column, container, pick_list, row, text, toggler, Space};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::state::ConfirmationAction;
use crate::domain::{BarSection, InstalledModule};
use crate::theme::{
    button as btn_style, menu_style, pick_list_style, shadow_sm, AppTheme, PickListColors,
    FONT_2XS, FONT_SM, FONT_XS, RADIUS_MD, RADIUS_SM, SPACE_LG, SPACE_MD, SPACE_SM,
};

pub fn module_row(
    module: &InstalledModule,
    is_toggling: bool,
    is_uninstalling: bool,
    theme: &AppTheme,
) -> Element<'static, Message> {
    let uuid = module.uuid.to_string();
    let uuid_toggle = module.uuid.clone();
    let uuid_position = module.uuid.clone();
    let uuid_prefs = module.uuid.clone();
    let uuid_uninstall = uuid.clone();
    let has_preferences = module.has_preferences;
    let name = module.waybar_module_name.clone();
    let name_for_confirm = name.clone();
    let enabled = module.enabled;
    let current_section = module
        .position
        .as_ref()
        .map(|p| p.section)
        .unwrap_or(BarSection::Center);

    let status_text = if enabled {
        text("Enabled").size(FONT_2XS).color(theme.success)
    } else {
        text("Disabled").size(FONT_2XS).color(theme.text_faint)
    };

    let toggle_widget: Element<Message> = if is_toggling {
        container(text("...").size(FONT_SM).color(theme.text_muted))
            .padding([SPACE_SM, SPACE_MD])
            .into()
    } else {
        toggler(enabled)
            .on_toggle(move |new_enabled| Message::ToggleModule {
                uuid: uuid_toggle.clone(),
                enabled: new_enabled,
            })
            .size(20.0)
            .into()
    };

    let picker_colors = PickListColors::from_theme(theme);

    let position_picker = pick_list(
        BarSection::all(),
        Some(current_section),
        move |section| Message::SetModulePosition {
            uuid: uuid_position.clone(),
            section,
        },
    )
    .padding([SPACE_SM / 2.0, SPACE_SM])
    .text_size(FONT_XS)
    .style(pick_list_style(picker_colors, RADIUS_SM))
    .menu_style(menu_style(picker_colors, RADIUS_SM, 0.2, 4.0));

    let prefs_widget: Element<Message> = if has_preferences {
        button(text("\u{2699}").size(FONT_SM))
            .on_press(Message::OpenPreferences(uuid_prefs))
            .style(btn_style::ghost(*theme))
            .padding([SPACE_SM, SPACE_SM])
            .into()
    } else {
        Space::new().width(0).into()
    };

    let uninstall_widget: Element<Message> = if is_uninstalling {
        container(text("Removing...").size(FONT_XS).color(theme.text_muted))
            .padding([SPACE_SM, SPACE_MD])
            .into()
    } else {
        button(text("Uninstall").size(FONT_XS))
            .on_press(Message::RequestConfirmation(ConfirmationAction::UninstallModule {
                uuid: uuid_uninstall,
                name: name_for_confirm,
            }))
            .style(btn_style::secondary(*theme))
            .padding([SPACE_SM, SPACE_MD])
            .into()
    };

    let theme_copy = *theme;
    let info_column = column![
        text(name).size(FONT_SM).color(theme.text_normal),
        row![
            text(uuid).size(FONT_XS).color(theme.text_faint),
            Space::new().width(SPACE_MD),
            status_text,
        ]
        .spacing(SPACE_SM),
    ]
    .spacing(SPACE_SM / 2.0);

    container(
        row![
            info_column,
            Space::new().width(Length::Fill),
            position_picker,
            prefs_widget,
            toggle_widget,
            uninstall_widget,
        ]
        .spacing(SPACE_LG)
        .padding(SPACE_MD)
        .align_y(Alignment::Center),
    )
    .style(move |_| iced::widget::container::Style {
        background: Some(Background::Color(theme_copy.bg_surface)),
        border: Border {
            color: theme_copy.border_subtle,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        shadow: shadow_sm(),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}
