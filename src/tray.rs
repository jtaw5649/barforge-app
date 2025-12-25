use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use ksni::blocking::TrayMethods;
use ksni::menu::StandardItem;
use ksni::{MenuItem, Tray};

type TrayHandle = ksni::blocking::Handle<BarforgeTray>;

static TRAY_HANDLE: OnceLock<Arc<Mutex<Option<TrayHandle>>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowWindow,
    CheckUpdates,
    Quit,
}

pub struct BarforgeTray {
    event_sender: Sender<TrayEvent>,
    update_count: usize,
}

impl Tray for BarforgeTray {
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn icon_name(&self) -> String {
        "applications-other".into()
    }

    fn title(&self) -> String {
        if self.update_count > 0 {
            format!("Barforge ({} updates)", self.update_count)
        } else {
            "Barforge".into()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let show_item = StandardItem {
            label: "Show Barforge".into(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.event_sender.send(TrayEvent::ShowWindow);
            }),
            ..Default::default()
        };

        let updates_label = if self.update_count > 0 {
            format!("Check for Updates ({})", self.update_count)
        } else {
            "Check for Updates".into()
        };

        let updates_item = StandardItem {
            label: updates_label,
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.event_sender.send(TrayEvent::CheckUpdates);
            }),
            ..Default::default()
        };

        let quit_item = StandardItem {
            label: "Quit".into(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.event_sender.send(TrayEvent::Quit);
            }),
            ..Default::default()
        };

        vec![
            show_item.into(),
            updates_item.into(),
            MenuItem::Separator,
            quit_item.into(),
        ]
    }
}

pub fn init() -> Option<Receiver<TrayEvent>> {
    let (event_tx, event_rx) = mpsc::channel();

    let tray = BarforgeTray {
        event_sender: event_tx,
        update_count: 0,
    };

    thread::spawn(move || match tray.spawn() {
        Ok(handle) => {
            let handle_store = TRAY_HANDLE.get_or_init(|| Arc::new(Mutex::new(None)));
            if let Ok(mut guard) = handle_store.lock() {
                *guard = Some(handle);
            }
            tracing::info!("System tray initialized");
        }
        Err(e) => {
            tracing::warn!("Failed to create tray icon: {}", e);
        }
    });

    Some(event_rx)
}

pub fn set_update_count(count: usize) {
    if let Some(handle_store) = TRAY_HANDLE.get()
        && let Ok(guard) = handle_store.lock()
        && let Some(handle) = guard.as_ref()
    {
        handle.update(|tray| {
            tray.update_count = count;
        });
    }
}

pub fn shutdown() {
    if let Some(handle_store) = TRAY_HANDLE.get()
        && let Ok(mut guard) = handle_store.lock()
        && let Some(handle) = guard.take()
    {
        handle.shutdown();
    }
}
