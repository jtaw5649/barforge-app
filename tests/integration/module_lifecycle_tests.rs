use waybar_manager::app::state::{LoadingState, Screen};
use waybar_manager::testing::{
    test_app, test_app_with_installed, test_app_with_registry, InstalledModuleBuilder,
    RegistryModuleBuilder,
};

#[test]
fn test_app_tracks_installed_module_uuids() {
    let installed = vec![
        InstalledModuleBuilder::new("mod1").build(),
        InstalledModuleBuilder::new("mod2").build(),
    ];
    let app = test_app_with_installed(installed);

    assert!(app.installed_uuids.contains("mod1@test"));
    assert!(app.installed_uuids.contains("mod2@test"));
    assert!(!app.installed_uuids.contains("mod3@test"));
}

#[test]
fn test_app_filters_modules_by_search_query() {
    let modules = vec![
        RegistryModuleBuilder::new("cpu-monitor")
            .tags(vec!["system", "cpu"])
            .build(),
        RegistryModuleBuilder::new("battery-widget")
            .tags(vec!["power", "battery"])
            .build(),
        RegistryModuleBuilder::new("memory-usage")
            .tags(vec!["system", "memory"])
            .build(),
    ];
    let mut app = test_app_with_registry(modules);

    app.browse.search_query = "cpu".to_string();
    let filtered = app.filtered_modules();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "cpu-monitor");
}

#[test]
fn test_app_filters_modules_by_tag() {
    let modules = vec![
        RegistryModuleBuilder::new("cpu-monitor")
            .tags(vec!["system", "cpu"])
            .build(),
        RegistryModuleBuilder::new("battery-widget")
            .tags(vec!["power", "battery"])
            .build(),
        RegistryModuleBuilder::new("memory-usage")
            .tags(vec!["system", "memory"])
            .build(),
    ];
    let mut app = test_app_with_registry(modules);

    app.browse.search_query = "system".to_string();
    let filtered = app.filtered_modules();

    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_installed_module_detects_available_update() {
    let installed = InstalledModuleBuilder::new("outdated-mod")
        .version("1.0.0")
        .registry_version("2.0.0")
        .build();

    assert!(installed.has_update());
}

#[test]
fn test_installed_module_no_update_when_current() {
    let installed = InstalledModuleBuilder::new("current-mod")
        .version("2.0.0")
        .registry_version("2.0.0")
        .build();

    assert!(!installed.has_update());
}

#[test]
fn test_app_counts_available_updates() {
    let installed = vec![
        InstalledModuleBuilder::new("outdated1")
            .version("1.0.0")
            .registry_version("2.0.0")
            .build(),
        InstalledModuleBuilder::new("current")
            .version("2.0.0")
            .registry_version("2.0.0")
            .build(),
        InstalledModuleBuilder::new("outdated2")
            .version("1.5.0")
            .registry_version("3.0.0")
            .build(),
    ];
    let app = test_app_with_installed(installed);

    assert_eq!(app.update_count(), 2);
}

#[test]
fn test_app_modules_with_updates_returns_only_outdated() {
    let installed = vec![
        InstalledModuleBuilder::new("outdated")
            .version("1.0.0")
            .registry_version("2.0.0")
            .build(),
        InstalledModuleBuilder::new("current")
            .version("2.0.0")
            .registry_version("2.0.0")
            .build(),
    ];
    let app = test_app_with_installed(installed);

    let updates = app.modules_with_updates();

    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].uuid.to_string(), "outdated@test");
}

#[test]
fn test_loading_state_is_loading() {
    assert!(LoadingState::Loading.is_loading());
    assert!(!LoadingState::Idle.is_loading());
    assert!(!LoadingState::Failed("error".to_string()).is_loading());
}

#[test]
fn test_screen_label() {
    assert_eq!(Screen::Browse.label(), "Browse");
    assert_eq!(Screen::Installed.label(), "Installed");
    assert_eq!(Screen::Settings.label(), "Settings");
}

#[test]
fn test_registry_load_error_sets_failed_state() {
    use waybar_manager::app::handlers::handle_registry_loaded;

    let mut app = test_app();
    let error_message = "Network error: connection refused".to_string();

    let _task = handle_registry_loaded(&mut app, Err(error_message.clone()));

    match &app.loading {
        LoadingState::Failed(msg) => assert_eq!(msg, &error_message),
        other => panic!("Expected LoadingState::Failed, got {:?}", other),
    }
}
