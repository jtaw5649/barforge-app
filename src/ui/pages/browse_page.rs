use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;
use std::collections::HashSet;

use crate::domain::{ModuleCategory, RegistryModule};
use crate::ui::widgets::ModuleCard;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use std::sync::OnceLock;

    #[derive(Default)]
    pub struct BrowsePage {
        pub search_entry: gtk::SearchEntry,
        pub category_dropdown: gtk::DropDown,
        pub flow_box: gtk::FlowBox,
        pub modules: RefCell<Vec<RegistryModule>>,
        pub installed_uuids: RefCell<HashSet<String>>,
        pub status_page: adw::StatusPage,
        pub stack: gtk::Stack,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BrowsePage {
        const NAME: &'static str = "WaybarBrowsePage";
        type Type = super::BrowsePage;
        type ParentType = adw::NavigationPage;
    }

    impl ObjectImpl for BrowsePage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build_ui();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("module-selected")
                    .param_types([String::static_type()])
                    .build()]
            })
        }
    }

    impl WidgetImpl for BrowsePage {}
    impl NavigationPageImpl for BrowsePage {}
}

glib::wrapper! {
    pub struct BrowsePage(ObjectSubclass<imp::BrowsePage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl BrowsePage {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("title", "Browse")
            .property("tag", "browse")
            .build()
    }

    fn build_ui(&self) {
        let imp = self.imp();

        let toolbar_view = adw::ToolbarView::new();

        let header = adw::HeaderBar::builder()
            .show_title(false)
            .build();

        imp.search_entry.set_placeholder_text(Some("Search modules..."));
        imp.search_entry.set_hexpand(true);
        imp.search_entry.add_css_class("search");

        let search_clamp = adw::Clamp::builder()
            .maximum_size(400)
            .child(&imp.search_entry)
            .build();

        header.set_title_widget(Some(&search_clamp));

        let categories: Vec<String> = std::iter::once("All Categories".to_string())
            .chain(ModuleCategory::all().iter().map(|c| c.display_name().to_string()))
            .collect();
        let category_model = gtk::StringList::new(&categories.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        imp.category_dropdown.set_model(Some(&category_model));
        imp.category_dropdown.set_selected(0);

        header.pack_end(&imp.category_dropdown);

        toolbar_view.add_top_bar(&header);

        imp.stack.set_transition_type(gtk::StackTransitionType::Crossfade);

        imp.flow_box.set_valign(gtk::Align::Start);
        imp.flow_box.set_homogeneous(true);
        imp.flow_box.set_selection_mode(gtk::SelectionMode::None);
        imp.flow_box.set_max_children_per_line(6);
        imp.flow_box.set_min_children_per_line(2);
        imp.flow_box.set_column_spacing(12);
        imp.flow_box.set_row_spacing(12);
        imp.flow_box.add_css_class("browse-grid");

        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .child(&imp.flow_box)
            .build();

        let content_clamp = adw::Clamp::builder()
            .maximum_size(1200)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .child(&scrolled)
            .build();

        imp.status_page.set_icon_name(Some("system-search-symbolic"));
        imp.status_page.set_title("No Modules Found");
        imp.status_page.set_description(Some("Try adjusting your search or filters"));

        imp.stack.add_named(&content_clamp, Some("content"));
        imp.stack.add_named(&imp.status_page, Some("empty"));
        imp.stack.set_visible_child_name("empty");

        toolbar_view.set_content(Some(&imp.stack));

        self.set_child(Some(&toolbar_view));

        self.setup_signals();
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        imp.search_entry.connect_search_changed(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| {
                page.apply_filters();
            }
        ));

        imp.category_dropdown.connect_selected_notify(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| {
                page.apply_filters();
            }
        ));
    }

    pub fn set_modules(&self, modules: Vec<RegistryModule>) {
        self.imp().modules.replace(modules);
        self.apply_filters();
    }

    pub fn set_installed_uuids(&self, uuids: HashSet<String>) {
        self.imp().installed_uuids.replace(uuids);
        self.refresh_cards();
    }

    pub fn search_query(&self) -> String {
        self.imp().search_entry.text().to_string()
    }

    pub fn selected_category(&self) -> Option<ModuleCategory> {
        let selected = self.imp().category_dropdown.selected();
        if selected == 0 {
            None
        } else {
            ModuleCategory::all().get(selected as usize - 1).copied()
        }
    }

    fn apply_filters(&self) {
        let imp = self.imp();
        let query = self.search_query().to_lowercase();
        let category = self.selected_category();

        let modules = imp.modules.borrow();
        let filtered: Vec<&RegistryModule> = modules
            .iter()
            .filter(|m| {
                let matches_search = query.is_empty() || m.matches_search(&query);
                let matches_category = category.is_none() || category == Some(m.category);
                matches_search && matches_category
            })
            .collect();

        self.populate_flow_box(&filtered);
    }

    fn populate_flow_box(&self, modules: &[&RegistryModule]) {
        let imp = self.imp();

        while let Some(child) = imp.flow_box.first_child() {
            imp.flow_box.remove(&child);
        }

        let installed = imp.installed_uuids.borrow();

        for module in modules {
            let is_installed = installed.contains(&module.uuid.to_string());
            let card = ModuleCard::new(module, is_installed);

            card.connect_activated(glib::clone!(
                #[weak(rename_to = page)]
                self,
                move |card| {
                    page.emit_by_name::<()>("module-selected", &[&card.uuid()]);
                }
            ));

            imp.flow_box.append(&card);
        }

        if modules.is_empty() {
            imp.stack.set_visible_child_name("empty");
        } else {
            imp.stack.set_visible_child_name("content");
        }
    }

    fn refresh_cards(&self) {
        let imp = self.imp();
        let installed = imp.installed_uuids.borrow();

        let mut child = imp.flow_box.first_child();
        while let Some(widget) = child {
            if let Some(card) = widget.downcast_ref::<ModuleCard>() {
                let is_installed = installed.contains(&card.uuid());
                card.set_installed(is_installed);
            }
            child = widget.next_sibling();
        }
    }

    pub fn module_count(&self) -> usize {
        let mut count = 0;
        let mut child = self.imp().flow_box.first_child();
        while let Some(widget) = child {
            count += 1;
            child = widget.next_sibling();
        }
        count
    }

    pub fn connect_module_selected<F: Fn(&Self, &str) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "module-selected",
            false,
            glib::closure_local!(move |page: &Self, uuid: &str| f(page, uuid)),
        )
    }
}

impl Default for BrowsePage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ModuleUuid;
    use crate::skip_if_no_gtk;
    use serial_test::serial;
    use std::rc::Rc;

    fn create_test_module(name: &str, category: ModuleCategory) -> RegistryModule {
        RegistryModule {
            uuid: ModuleUuid::try_from(format!("{}@test", name).as_str()).unwrap(),
            name: name.to_string(),
            description: format!("Test module {}", name),
            author: "test-author".to_string(),
            category,
            icon: None,
            screenshot: None,
            repo_url: "https://github.com/test/test".to_string(),
            downloads: 0,
            waybar_versions: vec!["0.10".to_string()],
        }
    }

    #[test]
    #[serial(gtk)]
    fn test_browse_page_default() {
        skip_if_no_gtk!();
        let page = BrowsePage::default();
        assert_eq!(page.search_query(), "");
        assert!(page.selected_category().is_none());
    }

    #[test]
    #[serial(gtk)]
    fn test_browse_page_has_title() {
        skip_if_no_gtk!();
        let page = BrowsePage::new();
        assert_eq!(page.title(), "Browse");
    }

    #[test]
    #[serial(gtk)]
    fn test_browse_page_has_tag() {
        skip_if_no_gtk!();
        let page = BrowsePage::new();
        assert_eq!(page.tag().as_deref(), Some("browse"));
    }

    #[test]
    #[serial(gtk)]
    fn test_set_modules_populates_flow_box() {
        skip_if_no_gtk!();
        let page = BrowsePage::new();
        let modules = vec![
            create_test_module("weather", ModuleCategory::Weather),
            create_test_module("cpu", ModuleCategory::Hardware),
        ];

        page.set_modules(modules);
        assert_eq!(page.module_count(), 2);
    }

    #[test]
    #[serial(gtk)]
    fn test_empty_modules_shows_status_page() {
        skip_if_no_gtk!();
        let page = BrowsePage::new();
        page.set_modules(vec![]);
        assert_eq!(page.imp().stack.visible_child_name().as_deref(), Some("empty"));
    }

    #[test]
    #[serial(gtk)]
    fn test_modules_show_content() {
        skip_if_no_gtk!();
        let page = BrowsePage::new();
        page.set_modules(vec![create_test_module("test", ModuleCategory::System)]);
        assert_eq!(page.imp().stack.visible_child_name().as_deref(), Some("content"));
    }

    #[test]
    #[serial(gtk)]
    fn test_module_selected_signal() {
        skip_if_no_gtk!();
        let page = BrowsePage::new();
        let modules = vec![create_test_module("test", ModuleCategory::System)];
        page.set_modules(modules);

        let received_uuid = Rc::new(RefCell::new(String::new()));
        let received_clone = received_uuid.clone();

        page.connect_module_selected(move |_, uuid| {
            received_clone.replace(uuid.to_string());
        });

        page.emit_by_name::<()>("module-selected", &[&"test@test"]);
        assert_eq!(*received_uuid.borrow(), "test@test");
    }

    #[test]
    #[serial(gtk)]
    fn test_set_installed_uuids_updates_cards() {
        skip_if_no_gtk!();
        let page = BrowsePage::new();
        let modules = vec![
            create_test_module("module1", ModuleCategory::System),
            create_test_module("module2", ModuleCategory::System),
        ];
        page.set_modules(modules);

        let mut installed = HashSet::new();
        installed.insert("module1@test".to_string());
        page.set_installed_uuids(installed);

        let mut found_installed = false;
        let mut child = page.imp().flow_box.first_child();
        while let Some(widget) = child {
            if let Some(card) = widget.downcast_ref::<ModuleCard>() {
                if card.uuid() == "module1@test" {
                    found_installed = card.is_installed();
                }
            }
            child = widget.next_sibling();
        }
        assert!(found_installed);
    }
}
