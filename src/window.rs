use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;

use crate::application::Application;
use crate::ui::pages::{BrowsePage, InstalledPage};

mod imp {
    use super::*;

    pub struct Window {
        pub split_view: adw::NavigationSplitView,
        pub sidebar_list: gtk::ListBox,
        pub content_stack: gtk::Stack,
        pub content_page: adw::NavigationPage,
        pub toast_overlay: adw::ToastOverlay,
        pub current_page: RefCell<String>,
        pub browse_page: BrowsePage,
        pub installed_page: InstalledPage,
    }

    impl Default for Window {
        fn default() -> Self {
            Self {
                split_view: adw::NavigationSplitView::new(),
                sidebar_list: gtk::ListBox::new(),
                content_stack: gtk::Stack::new(),
                content_page: adw::NavigationPage::new(
                    &gtk::Box::new(gtk::Orientation::Vertical, 0),
                    "Content",
                ),
                toast_overlay: adw::ToastOverlay::new(),
                current_page: RefCell::new("browse".to_string()),
                browse_page: BrowsePage::new(),
                installed_page: InstalledPage::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "WaybarExtensionManagerWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.build_ui();
            obj.connect_signals();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Root, gtk::Native,
            gtk::ShortcutManager, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[derive(Debug, Clone, Copy)]
struct NavItem {
    id: &'static str,
    label: &'static str,
    icon: &'static str,
}

const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        id: "browse",
        label: "Browse",
        icon: "system-search-symbolic",
    },
    NavItem {
        id: "installed",
        label: "Installed",
        icon: "emblem-ok-symbolic",
    },
    NavItem {
        id: "updates",
        label: "Updates",
        icon: "software-update-available-symbolic",
    },
];

impl Window {
    pub fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    fn build_ui(&self) {
        self.set_title(Some("Waybar Extension Manager"));
        self.set_default_size(1000, 700);
        self.set_size_request(800, 500);

        let imp = self.imp();

        let sidebar_content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let sidebar_header = adw::HeaderBar::builder()
            .title_widget(&gtk::Label::new(Some("Extensions")))
            .build();

        imp.sidebar_list
            .set_selection_mode(gtk::SelectionMode::Single);
        imp.sidebar_list.add_css_class("navigation-sidebar");

        for item in NAV_ITEMS {
            let row = self.create_nav_row(item);
            imp.sidebar_list.append(&row);
        }

        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&imp.sidebar_list)
            .build();

        sidebar_content.append(&sidebar_header);
        sidebar_content.append(&scrolled);

        let sidebar_page = adw::NavigationPage::builder()
            .title("Extensions")
            .child(&sidebar_content)
            .build();

        imp.content_stack
            .set_transition_type(gtk::StackTransitionType::Crossfade);

        imp.content_stack.add_named(&imp.browse_page, Some("browse"));
        imp.content_stack.add_named(&imp.installed_page, Some("installed"));

        let updates_placeholder = adw::StatusPage::builder()
            .icon_name("software-update-available-symbolic")
            .title("No Updates Available")
            .description("All your modules are up to date")
            .build();
        imp.content_stack.add_named(&updates_placeholder, Some("updates"));

        imp.toast_overlay.set_child(Some(&imp.content_stack));
        imp.toast_overlay.set_hexpand(true);
        imp.toast_overlay.set_vexpand(true);

        imp.content_page.set_child(Some(&imp.toast_overlay));
        imp.content_page.set_title("Browse");

        imp.split_view.set_sidebar(Some(&sidebar_page));
        imp.split_view.set_content(Some(&imp.content_page));
        imp.split_view.set_max_sidebar_width(250.0);
        imp.split_view.set_min_sidebar_width(180.0);

        self.set_content(Some(&imp.split_view));

        if let Some(first_row) = imp.sidebar_list.row_at_index(0) {
            imp.sidebar_list.select_row(Some(&first_row));
        }

        self.setup_responsive_breakpoints();
    }

    fn create_nav_row(&self, item: &NavItem) -> gtk::ListBoxRow {
        let hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let icon = gtk::Image::from_icon_name(item.icon);
        let label = gtk::Label::builder()
            .label(item.label)
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();

        hbox.append(&icon);
        hbox.append(&label);

        gtk::ListBoxRow::builder()
            .child(&hbox)
            .name(item.id)
            .build()
    }

    fn connect_signals(&self) {
        let imp = self.imp();

        imp.sidebar_list.connect_row_selected(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, row| {
                if let Some(row) = row {
                    let name = row.widget_name();
                    window.navigate_to(name.as_str());
                }
            }
        ));

        imp.browse_page.connect_module_selected(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, uuid| {
                window.show_toast(&format!("Selected module: {}", uuid));
            }
        ));

        imp.installed_page.connect_module_toggled(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, uuid, enabled| {
                let action = if enabled { "enabled" } else { "disabled" };
                window.show_toast(&format!("Module {} {}", uuid, action));
            }
        ));

        imp.installed_page.connect_module_uninstall(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |page, uuid| {
                page.remove_module(uuid);
                window.show_toast(&format!("Uninstalled module: {}", uuid));
            }
        ));

        imp.installed_page.connect_module_preferences(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, uuid| {
                window.show_toast(&format!("Opening preferences for: {}", uuid));
            }
        ));
    }

    fn navigate_to(&self, page_id: &str) {
        let imp = self.imp();

        imp.content_stack.set_visible_child_name(page_id);
        imp.current_page.replace(page_id.to_string());

        if let Some(item) = NAV_ITEMS.iter().find(|i| i.id == page_id) {
            imp.content_page.set_title(item.label);
        }

        if imp.split_view.is_collapsed() {
            imp.split_view.set_show_content(true);
        }
    }

    fn setup_responsive_breakpoints(&self) {
        let imp = self.imp();

        let collapsed_breakpoint = adw::Breakpoint::new(adw::BreakpointCondition::new_length(
            adw::BreakpointConditionLengthType::MaxWidth,
            860.0,
            adw::LengthUnit::Sp,
        ));
        collapsed_breakpoint.add_setter(&imp.split_view, "collapsed", Some(&true.to_value()));
        collapsed_breakpoint.add_setter(&imp.split_view, "show-content", Some(&true.to_value()));

        self.add_breakpoint(collapsed_breakpoint);
    }

    pub fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        self.imp().toast_overlay.add_toast(toast);
    }

    pub fn browse_page(&self) -> &BrowsePage {
        &self.imp().browse_page
    }

    pub fn installed_page(&self) -> &InstalledPage {
        &self.imp().installed_page
    }

    pub fn app(&self) -> Option<Application> {
        self.application().and_downcast::<Application>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skip_if_no_gtk;
    use serial_test::serial;

    #[test]
    fn test_nav_items_has_expected_pages() {
        let ids: Vec<_> = NAV_ITEMS.iter().map(|item| item.id).collect();
        assert!(ids.contains(&"browse"));
        assert!(ids.contains(&"installed"));
        assert!(ids.contains(&"updates"));
    }

    #[test]
    fn test_nav_items_all_have_labels() {
        for item in NAV_ITEMS {
            assert!(!item.label.is_empty(), "Nav item {} has empty label", item.id);
        }
    }

    #[test]
    fn test_nav_items_all_have_icons() {
        for item in NAV_ITEMS {
            assert!(
                item.icon.ends_with("-symbolic"),
                "Nav item {} icon should be symbolic: {}",
                item.id,
                item.icon
            );
        }
    }

    #[test]
    fn test_nav_items_count() {
        assert_eq!(NAV_ITEMS.len(), 3);
    }

    #[test]
    #[serial(gtk)]
    fn test_window_default_page_is_browse() {
        skip_if_no_gtk!();
        let window = glib::Object::builder::<Window>().build();
        assert_eq!(*window.imp().current_page.borrow(), "browse");
    }

    #[test]
    #[serial(gtk)]
    fn test_window_has_split_view() {
        skip_if_no_gtk!();
        let window = glib::Object::builder::<Window>().build();
        let _ = &window.imp().split_view;
    }

    #[test]
    #[serial(gtk)]
    fn test_window_has_toast_overlay() {
        skip_if_no_gtk!();
        let window = glib::Object::builder::<Window>().build();
        let _ = &window.imp().toast_overlay;
    }

    #[test]
    #[serial(gtk)]
    fn test_window_navigate_to_installed() {
        skip_if_no_gtk!();
        let window = glib::Object::builder::<Window>().build();
        window.navigate_to("installed");
        assert_eq!(*window.imp().current_page.borrow(), "installed");
    }

    #[test]
    #[serial(gtk)]
    fn test_window_navigate_to_updates() {
        skip_if_no_gtk!();
        let window = glib::Object::builder::<Window>().build();
        window.navigate_to("updates");
        assert_eq!(*window.imp().current_page.borrow(), "updates");
    }

    #[test]
    #[serial(gtk)]
    fn test_window_show_toast() {
        skip_if_no_gtk!();
        let window = glib::Object::builder::<Window>().build();
        window.show_toast("Test message");
    }

    #[test]
    #[serial(gtk)]
    fn test_window_app_returns_none_without_application() {
        skip_if_no_gtk!();
        let window = glib::Object::builder::<Window>().build();
        assert!(window.app().is_none());
    }

    #[test]
    #[serial(gtk)]
    fn test_window_has_browse_page() {
        skip_if_no_gtk!();
        let window = glib::Object::builder::<Window>().build();
        let _ = window.browse_page();
    }

    #[test]
    #[serial(gtk)]
    fn test_window_has_installed_page() {
        skip_if_no_gtk!();
        let window = glib::Object::builder::<Window>().build();
        let _ = window.installed_page();
    }
}
