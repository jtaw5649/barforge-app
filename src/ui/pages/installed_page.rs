use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

use crate::domain::InstalledModule;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use std::sync::OnceLock;

    #[derive(Default)]
    pub struct InstalledPage {
        pub list_box: gtk::ListBox,
        pub modules: RefCell<Vec<InstalledModule>>,
        pub status_page: adw::StatusPage,
        pub stack: gtk::Stack,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InstalledPage {
        const NAME: &'static str = "WaybarInstalledPage";
        type Type = super::InstalledPage;
        type ParentType = adw::NavigationPage;
    }

    impl ObjectImpl for InstalledPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build_ui();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("module-toggled")
                        .param_types([String::static_type(), bool::static_type()])
                        .build(),
                    Signal::builder("module-preferences")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("module-uninstall")
                        .param_types([String::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl WidgetImpl for InstalledPage {}
    impl NavigationPageImpl for InstalledPage {}
}

glib::wrapper! {
    pub struct InstalledPage(ObjectSubclass<imp::InstalledPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl InstalledPage {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("title", "Installed")
            .property("tag", "installed")
            .build()
    }

    fn build_ui(&self) {
        let imp = self.imp();

        let toolbar_view = adw::ToolbarView::new();

        let header = adw::HeaderBar::builder()
            .title_widget(&adw::WindowTitle::new("Installed Modules", ""))
            .build();

        toolbar_view.add_top_bar(&header);

        imp.stack.set_transition_type(gtk::StackTransitionType::Crossfade);

        imp.list_box.set_selection_mode(gtk::SelectionMode::None);
        imp.list_box.add_css_class("boxed-list");

        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .child(&imp.list_box)
            .build();

        let content_clamp = adw::Clamp::builder()
            .maximum_size(800)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .child(&scrolled)
            .build();

        imp.status_page.set_icon_name(Some("emblem-ok-symbolic"));
        imp.status_page.set_title("No Modules Installed");
        imp.status_page.set_description(Some("Browse the registry to find and install modules"));

        imp.stack.add_named(&content_clamp, Some("content"));
        imp.stack.add_named(&imp.status_page, Some("empty"));
        imp.stack.set_visible_child_name("empty");

        toolbar_view.set_content(Some(&imp.stack));

        self.set_child(Some(&toolbar_view));
    }

    pub fn set_modules(&self, modules: Vec<InstalledModule>) {
        self.imp().modules.replace(modules);
        self.refresh_list();
    }

    fn refresh_list(&self) {
        let imp = self.imp();

        while let Some(child) = imp.list_box.first_child() {
            imp.list_box.remove(&child);
        }

        let modules = imp.modules.borrow();

        for module in modules.iter() {
            let row = self.create_module_row(module);
            imp.list_box.append(&row);
        }

        if modules.is_empty() {
            imp.stack.set_visible_child_name("empty");
        } else {
            imp.stack.set_visible_child_name("content");
        }
    }

    fn create_module_row(&self, module: &InstalledModule) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(module.waybar_module_name.as_str())
            .subtitle(module.uuid.to_string())
            .build();

        let uuid = module.uuid.to_string();

        let toggle = gtk::Switch::builder()
            .valign(gtk::Align::Center)
            .active(module.enabled)
            .build();

        let toggle_uuid = uuid.clone();
        toggle.connect_state_set(glib::clone!(
            #[weak(rename_to = page)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, state| {
                page.emit_by_name::<()>("module-toggled", &[&toggle_uuid, &state]);
                glib::Propagation::Proceed
            }
        ));

        row.add_suffix(&toggle);

        if module.has_preferences {
            let prefs_button = gtk::Button::builder()
                .icon_name("emblem-system-symbolic")
                .valign(gtk::Align::Center)
                .css_classes(["flat"])
                .tooltip_text("Preferences")
                .build();

            let prefs_uuid = uuid.clone();
            prefs_button.connect_clicked(glib::clone!(
                #[weak(rename_to = page)]
                self,
                move |_| {
                    page.emit_by_name::<()>("module-preferences", &[&prefs_uuid]);
                }
            ));

            row.add_suffix(&prefs_button);
        }

        let uninstall_button = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .valign(gtk::Align::Center)
            .css_classes(["flat", "error"])
            .tooltip_text("Uninstall")
            .build();

        uninstall_button.connect_clicked(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| {
                page.emit_by_name::<()>("module-uninstall", &[&uuid]);
            }
        ));

        row.add_suffix(&uninstall_button);
        row.set_activatable_widget(Some(&toggle));

        row
    }

    pub fn module_count(&self) -> usize {
        self.imp().modules.borrow().len()
    }

    pub fn update_module_state(&self, uuid: &str, enabled: bool) {
        let mut modules = self.imp().modules.borrow_mut();
        if let Some(module) = modules.iter_mut().find(|m| m.uuid.to_string() == uuid) {
            module.enabled = enabled;
        }
        drop(modules);
        self.refresh_list();
    }

    pub fn remove_module(&self, uuid: &str) {
        let mut modules = self.imp().modules.borrow_mut();
        modules.retain(|m| m.uuid.to_string() != uuid);
        drop(modules);
        self.refresh_list();
    }

    pub fn connect_module_toggled<F: Fn(&Self, &str, bool) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "module-toggled",
            false,
            glib::closure_local!(move |page: &Self, uuid: &str, state: bool| f(page, uuid, state)),
        )
    }

    pub fn connect_module_preferences<F: Fn(&Self, &str) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "module-preferences",
            false,
            glib::closure_local!(move |page: &Self, uuid: &str| f(page, uuid)),
        )
    }

    pub fn connect_module_uninstall<F: Fn(&Self, &str) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "module-uninstall",
            false,
            glib::closure_local!(move |page: &Self, uuid: &str| f(page, uuid)),
        )
    }
}

impl Default for InstalledPage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ModuleUuid, ModuleVersion};
    use crate::skip_if_no_gtk;
    use serial_test::serial;
    use std::path::PathBuf;
    use std::rc::Rc;

    fn create_test_module(name: &str, enabled: bool) -> InstalledModule {
        InstalledModule {
            uuid: ModuleUuid::try_from(format!("{}@test", name).as_str()).unwrap(),
            version: ModuleVersion::try_from("1.0.0").unwrap(),
            install_path: PathBuf::from(format!("/test/{}", name)),
            enabled,
            waybar_module_name: format!("custom/{}", name),
            has_preferences: false,
        }
    }

    #[test]
    #[serial(gtk)]
    fn test_installed_page_default() {
        skip_if_no_gtk!();
        let page = InstalledPage::default();
        assert_eq!(page.module_count(), 0);
    }

    #[test]
    #[serial(gtk)]
    fn test_installed_page_has_title() {
        skip_if_no_gtk!();
        let page = InstalledPage::new();
        assert_eq!(page.title(), "Installed");
    }

    #[test]
    #[serial(gtk)]
    fn test_installed_page_has_tag() {
        skip_if_no_gtk!();
        let page = InstalledPage::new();
        assert_eq!(page.tag().as_deref(), Some("installed"));
    }

    #[test]
    #[serial(gtk)]
    fn test_set_modules_updates_count() {
        skip_if_no_gtk!();
        let page = InstalledPage::new();
        let modules = vec![
            create_test_module("weather", true),
            create_test_module("cpu", false),
        ];

        page.set_modules(modules);
        assert_eq!(page.module_count(), 2);
    }

    #[test]
    #[serial(gtk)]
    fn test_empty_modules_shows_status_page() {
        skip_if_no_gtk!();
        let page = InstalledPage::new();
        page.set_modules(vec![]);
        assert_eq!(page.imp().stack.visible_child_name().as_deref(), Some("empty"));
    }

    #[test]
    #[serial(gtk)]
    fn test_modules_show_content() {
        skip_if_no_gtk!();
        let page = InstalledPage::new();
        page.set_modules(vec![create_test_module("test", true)]);
        assert_eq!(page.imp().stack.visible_child_name().as_deref(), Some("content"));
    }

    #[test]
    #[serial(gtk)]
    fn test_remove_module() {
        skip_if_no_gtk!();
        let page = InstalledPage::new();
        page.set_modules(vec![
            create_test_module("module1", true),
            create_test_module("module2", false),
        ]);

        assert_eq!(page.module_count(), 2);
        page.remove_module("module1@test");
        assert_eq!(page.module_count(), 1);
    }

    #[test]
    #[serial(gtk)]
    fn test_module_toggled_signal() {
        skip_if_no_gtk!();
        let page = InstalledPage::new();
        page.set_modules(vec![create_test_module("test", true)]);

        let received = Rc::new(RefCell::new((String::new(), false)));
        let received_clone = received.clone();

        page.connect_module_toggled(move |_, uuid, state| {
            received_clone.replace((uuid.to_string(), state));
        });

        page.emit_by_name::<()>("module-toggled", &[&"test@test", &false]);

        let (uuid, state) = received.borrow().clone();
        assert_eq!(uuid, "test@test");
        assert!(!state);
    }

    #[test]
    #[serial(gtk)]
    fn test_module_uninstall_signal() {
        skip_if_no_gtk!();
        let page = InstalledPage::new();
        page.set_modules(vec![create_test_module("test", true)]);

        let received_uuid = Rc::new(RefCell::new(String::new()));
        let received_clone = received_uuid.clone();

        page.connect_module_uninstall(move |_, uuid| {
            received_clone.replace(uuid.to_string());
        });

        page.emit_by_name::<()>("module-uninstall", &[&"test@test"]);
        assert_eq!(*received_uuid.borrow(), "test@test");
    }

    #[test]
    #[serial(gtk)]
    fn test_module_preferences_signal() {
        skip_if_no_gtk!();
        let page = InstalledPage::new();

        let mut module = create_test_module("test", true);
        module.has_preferences = true;
        page.set_modules(vec![module]);

        let received_uuid = Rc::new(RefCell::new(String::new()));
        let received_clone = received_uuid.clone();

        page.connect_module_preferences(move |_, uuid| {
            received_clone.replace(uuid.to_string());
        });

        page.emit_by_name::<()>("module-preferences", &[&"test@test"]);
        assert_eq!(*received_uuid.borrow(), "test@test");
    }
}
