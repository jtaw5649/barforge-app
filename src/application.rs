use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::domain::{ModuleCategory, ModuleUuid, ModuleVersion, RegistryModule, InstalledModule};
use crate::window::Window;
use std::collections::HashSet;
use std::path::PathBuf;

const APP_ID: &str = "org.waybar.ExtensionManager";

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Application {}

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "WaybarExtensionManagerApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_actions();
            obj.setup_accels();
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self) {
            let app = self.obj();

            let window = app.active_window().unwrap_or_else(|| {
                let window = Window::new(&app);
                app.add_window(&window);
                app.load_sample_data(&window);
                window.upcast()
            });

            window.present();
        }

        fn startup(&self) {
            self.parent_startup();
        }

        fn shutdown(&self) {
            self.parent_shutdown();
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gio::ApplicationFlags::NON_UNIQUE)
            .build()
    }

    fn setup_actions(&self) {
        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(glib::clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| app.quit()
        ));
        self.add_action(&action_quit);

        let action_about = gio::SimpleAction::new("about", None);
        action_about.connect_activate(glib::clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| app.show_about()
        ));
        self.add_action(&action_about);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
    }

    fn show_about(&self) {
        let about = adw::AboutDialog::builder()
            .application_name("Waybar Extension Manager")
            .application_icon("application-x-addon")
            .developer_name("jtaw")
            .version(env!("CARGO_PKG_VERSION"))
            .copyright("© 2025 jtaw")
            .license_type(gtk::License::MitX11)
            .website("https://github.com/jtaw5649/waybar-manager")
            .build();

        if let Some(window) = self.active_window() {
            about.present(Some(&window));
        }
    }

    fn load_sample_data(&self, window: &Window) {
        let sample_modules = vec![
            RegistryModule {
                uuid: ModuleUuid::try_from("weather-wttr@waybar-modules").unwrap(),
                name: "Weather (wttr.in)".to_string(),
                description: "Display weather using wttr.in API".to_string(),
                author: "waybar-community".to_string(),
                category: ModuleCategory::Weather,
                icon: Some("weather-clear-symbolic".to_string()),
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/weather-wttr".to_string(),
                downloads: 1523,
                waybar_versions: vec!["0.10".to_string(), "0.11".to_string()],
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("cpu-monitor@waybar-modules").unwrap(),
                name: "CPU Monitor".to_string(),
                description: "Advanced CPU usage monitor with per-core stats".to_string(),
                author: "waybar-community".to_string(),
                category: ModuleCategory::Hardware,
                icon: Some("utilities-system-monitor-symbolic".to_string()),
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/cpu-monitor".to_string(),
                downloads: 2341,
                waybar_versions: vec!["0.10".to_string()],
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("network-speed@waybar-modules").unwrap(),
                name: "Network Speed".to_string(),
                description: "Real-time network upload/download speed".to_string(),
                author: "waybar-community".to_string(),
                category: ModuleCategory::Network,
                icon: Some("network-transmit-receive-symbolic".to_string()),
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/network-speed".to_string(),
                downloads: 1876,
                waybar_versions: vec!["0.10".to_string(), "0.11".to_string()],
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("spotify@waybar-modules").unwrap(),
                name: "Spotify".to_string(),
                description: "Show currently playing track from Spotify".to_string(),
                author: "waybar-community".to_string(),
                category: ModuleCategory::Media,
                icon: Some("audio-x-generic-symbolic".to_string()),
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/spotify".to_string(),
                downloads: 3421,
                waybar_versions: vec!["0.10".to_string()],
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("battery-notify@waybar-modules").unwrap(),
                name: "Battery Notifier".to_string(),
                description: "Enhanced battery module with notifications".to_string(),
                author: "waybar-community".to_string(),
                category: ModuleCategory::Power,
                icon: Some("battery-good-symbolic".to_string()),
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/battery-notify".to_string(),
                downloads: 987,
                waybar_versions: vec!["0.10".to_string()],
            },
            RegistryModule {
                uuid: ModuleUuid::try_from("pomodoro@waybar-modules").unwrap(),
                name: "Pomodoro Timer".to_string(),
                description: "Productivity timer with waybar integration".to_string(),
                author: "waybar-community".to_string(),
                category: ModuleCategory::Productivity,
                icon: Some("alarm-symbolic".to_string()),
                screenshot: None,
                repo_url: "https://github.com/waybar-modules/pomodoro".to_string(),
                downloads: 654,
                waybar_versions: vec!["0.10".to_string()],
            },
        ];

        let mut installed_uuids = HashSet::new();
        installed_uuids.insert("weather-wttr@waybar-modules".to_string());
        installed_uuids.insert("cpu-monitor@waybar-modules".to_string());

        window.browse_page().set_modules(sample_modules);
        window.browse_page().set_installed_uuids(installed_uuids);

        let installed_modules = vec![
            InstalledModule {
                uuid: ModuleUuid::try_from("weather-wttr@waybar-modules").unwrap(),
                version: ModuleVersion::try_from("1.2.0").unwrap(),
                install_path: PathBuf::from("/home/user/.local/share/waybar-manager/modules/weather-wttr@waybar-modules"),
                enabled: true,
                waybar_module_name: "custom/weather".to_string(),
                has_preferences: true,
            },
            InstalledModule {
                uuid: ModuleUuid::try_from("cpu-monitor@waybar-modules").unwrap(),
                version: ModuleVersion::try_from("2.0.1").unwrap(),
                install_path: PathBuf::from("/home/user/.local/share/waybar-manager/modules/cpu-monitor@waybar-modules"),
                enabled: false,
                waybar_module_name: "custom/cpu".to_string(),
                has_preferences: false,
            },
        ];

        window.installed_page().set_modules(installed_modules);
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skip_if_no_gtk;

    #[test]
    fn test_application_has_quit_action() {
        skip_if_no_gtk!();
        let app = Application::new();
        app.register(None::<&gio::Cancellable>).unwrap();
        assert!(app.lookup_action("quit").is_some());
    }

    #[test]
    fn test_application_has_about_action() {
        skip_if_no_gtk!();
        let app = Application::new();
        app.register(None::<&gio::Cancellable>).unwrap();
        assert!(app.lookup_action("about").is_some());
    }

    #[test]
    fn test_quit_accelerator_bound() {
        skip_if_no_gtk!();
        let app = Application::new();
        app.register(None::<&gio::Cancellable>).unwrap();
        let accels = app.accels_for_action("app.quit");
        assert!(accels.iter().any(|a| a == "<Control>q"));
    }
}
