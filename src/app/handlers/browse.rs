use iced::Task;

use crate::app::message::Message;
use crate::app::state::{App, CategoryFilter, SortField, ViewMode};

pub fn handle_search_changed(app: &mut App, query: String) -> Task<Message> {
    app.browse.pending_search = Some(query);
    app.browse.search_debounce_start = Some(std::time::Instant::now());
    Task::none()
}

pub fn handle_category_selected(app: &mut App, filter: CategoryFilter) -> Task<Message> {
    app.browse.selected_category = filter;
    Task::none()
}

pub fn handle_set_sort_field(app: &mut App, field: SortField) -> Task<Message> {
    app.browse.sort_field = field;
    app.save_settings();
    Task::none()
}

pub fn handle_toggle_sort_order(app: &mut App) -> Task<Message> {
    app.browse.sort_order = app.browse.sort_order.toggle();
    app.save_settings();
    Task::none()
}

pub fn handle_set_view_mode(app: &mut App, mode: ViewMode) -> Task<Message> {
    app.browse.view_mode = mode;
    app.save_settings();
    Task::none()
}

pub fn handle_toggle_verified_only(app: &mut App) -> Task<Message> {
    app.browse.verified_only = !app.browse.verified_only;
    Task::none()
}

pub fn handle_installed_search_changed(app: &mut App, query: String) -> Task<Message> {
    app.installed.pending_search = Some(query);
    app.installed.search_debounce_start = Some(std::time::Instant::now());
    Task::none()
}

pub fn handle_clear_installed_search(app: &mut App) -> Task<Message> {
    app.installed.search_query.clear();
    app.installed.pending_search = None;
    app.installed.search_debounce_start = None;
    Task::none()
}
