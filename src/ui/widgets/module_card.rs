use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, RefCell};

use crate::domain::RegistryModule;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use std::sync::OnceLock;

    #[derive(Default)]
    pub struct ModuleCard {
        pub uuid: RefCell<String>,
        pub module_name: RefCell<String>,
        pub is_installed: Cell<bool>,

        pub overlay: gtk::Overlay,
        pub icon: gtk::Image,
        pub name_label: gtk::Label,
        pub author_label: gtk::Label,
        pub category_badge: gtk::Label,
        pub installed_badge: gtk::Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ModuleCard {
        const NAME: &'static str = "WaybarModuleCard";
        type Type = super::ModuleCard;
        type ParentType = gtk::FlowBoxChild;
    }

    impl ObjectImpl for ModuleCard {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build_ui();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("activated").build()])
        }
    }

    impl WidgetImpl for ModuleCard {}
    impl FlowBoxChildImpl for ModuleCard {}
}

glib::wrapper! {
    pub struct ModuleCard(ObjectSubclass<imp::ModuleCard>)
        @extends gtk::FlowBoxChild, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ModuleCard {
    pub fn new(module: &RegistryModule, is_installed: bool) -> Self {
        let card: Self = glib::Object::builder().build();
        card.set_from_module(module, is_installed);
        card
    }

    fn build_ui(&self) {
        let imp = self.imp();

        let frame = gtk::Frame::builder()
            .css_classes(["card", "module-card"])
            .focusable(true)
            .build();

        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(glib::clone!(
            #[weak(rename_to = card)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, _| {
                if key == gtk::gdk::Key::Return || key == gtk::gdk::Key::space {
                    card.emit_by_name::<()>("activated", &[]);
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        frame.add_controller(key_controller);

        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        imp.icon.set_pixel_size(48);
        imp.icon.set_halign(gtk::Align::Center);
        imp.icon.add_css_class("dim-label");

        imp.name_label.set_halign(gtk::Align::Center);
        imp.name_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        imp.name_label.add_css_class("heading");

        imp.author_label.set_halign(gtk::Align::Center);
        imp.author_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        imp.author_label.add_css_class("dim-label");
        imp.author_label.add_css_class("caption");

        let badge_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk::Align::Center)
            .build();

        imp.category_badge.add_css_class("caption");
        imp.category_badge.add_css_class("dim-label");

        imp.installed_badge.set_label("Installed");
        imp.installed_badge.add_css_class("caption");
        imp.installed_badge.add_css_class("success");
        imp.installed_badge.set_visible(false);

        badge_box.append(&imp.category_badge);
        badge_box.append(&imp.installed_badge);

        main_box.append(&imp.icon);
        main_box.append(&imp.name_label);
        main_box.append(&imp.author_label);
        main_box.append(&badge_box);

        frame.set_child(Some(&main_box));
        self.set_child(Some(&frame));
        self.set_size_request(180, 160);
    }

    pub fn set_from_module(&self, module: &RegistryModule, is_installed: bool) {
        let imp = self.imp();

        imp.uuid.replace(module.uuid.to_string());
        imp.module_name.replace(module.name.clone());
        imp.is_installed.set(is_installed);

        let icon_name = module.icon.as_deref().unwrap_or(module.category.icon());
        imp.icon.set_icon_name(Some(icon_name));

        imp.name_label.set_label(&module.name);
        imp.author_label.set_label(&format!("by {}", module.author));
        imp.category_badge.set_label(module.category.display_name());

        imp.installed_badge.set_visible(is_installed);

        let accessible_label = if is_installed {
            format!("{} by {} (installed)", module.name, module.author)
        } else {
            format!("{} by {}", module.name, module.author)
        };
        self.update_property(&[gtk::accessible::Property::Label(&accessible_label)]);
    }

    pub fn uuid(&self) -> String {
        self.imp().uuid.borrow().clone()
    }

    pub fn module_name(&self) -> String {
        self.imp().module_name.borrow().clone()
    }

    pub fn is_installed(&self) -> bool {
        self.imp().is_installed.get()
    }

    pub fn set_installed(&self, installed: bool) {
        self.imp().is_installed.set(installed);
        self.imp().installed_badge.set_visible(installed);
    }

    pub fn connect_activated<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "activated",
            false,
            glib::closure_local!(move |card: &Self| f(card)),
        )
    }
}

impl Default for ModuleCard {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ModuleCategory, ModuleUuid};
    use crate::skip_if_no_gtk;
    use serial_test::serial;
    use std::cell::Cell;
    use std::rc::Rc;

    fn create_test_module(name: &str) -> RegistryModule {
        RegistryModule {
            uuid: ModuleUuid::try_from(format!("{}@test", name).as_str()).unwrap(),
            name: name.to_string(),
            description: format!("Test module {}", name),
            author: "test-author".to_string(),
            category: ModuleCategory::Weather,
            icon: None,
            screenshot: None,
            repo_url: "https://github.com/test/test".to_string(),
            downloads: 0,
            waybar_versions: vec!["0.10".to_string()],
        }
    }

    #[test]
    #[serial(gtk)]
    fn test_module_card_default() {
        skip_if_no_gtk!();
        let card = ModuleCard::default();
        assert_eq!(card.uuid(), "");
        assert!(!card.is_installed());
    }

    #[test]
    #[serial(gtk)]
    fn test_module_card_new_not_installed() {
        skip_if_no_gtk!();
        let module = create_test_module("weather");
        let card = ModuleCard::new(&module, false);

        assert_eq!(card.uuid(), "weather@test");
        assert_eq!(card.module_name(), "weather");
        assert!(!card.is_installed());
    }

    #[test]
    #[serial(gtk)]
    fn test_module_card_new_installed() {
        skip_if_no_gtk!();
        let module = create_test_module("cpu-monitor");
        let card = ModuleCard::new(&module, true);

        assert_eq!(card.uuid(), "cpu-monitor@test");
        assert!(card.is_installed());
    }

    #[test]
    #[serial(gtk)]
    fn test_set_installed() {
        skip_if_no_gtk!();
        let module = create_test_module("network");
        let card = ModuleCard::new(&module, false);

        assert!(!card.is_installed());
        card.set_installed(true);
        assert!(card.is_installed());
        card.set_installed(false);
        assert!(!card.is_installed());
    }

    #[test]
    #[serial(gtk)]
    fn test_activated_signal() {
        skip_if_no_gtk!();
        let module = create_test_module("test");
        let card = ModuleCard::new(&module, false);

        let signal_received = Rc::new(Cell::new(false));
        let received_clone = signal_received.clone();

        card.connect_activated(move |_| {
            received_clone.set(true);
        });

        card.emit_by_name::<()>("activated", &[]);
        assert!(signal_received.get());
    }

    #[test]
    #[serial(gtk)]
    fn test_uses_category_icon_as_fallback() {
        skip_if_no_gtk!();
        let module = create_test_module("weather");
        let card = ModuleCard::new(&module, false);
        let icon_name = card.imp().icon.icon_name();
        assert!(icon_name.is_some());
        assert!(icon_name.unwrap().ends_with("-symbolic"));
    }
}
