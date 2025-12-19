use std::path::PathBuf;
use std::time::Duration;

use iced::Subscription;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc as async_mpsc;
use tokio::time::timeout;

use crate::app::Message;

fn omarchy_theme_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("omarchy/current/theme")
}

pub fn watch_omarchy_theme() -> Subscription<Message> {
    Subscription::run(|| {
        iced::futures::stream::unfold(WatcherState::Ready, |state| async move {
            match state {
                WatcherState::Ready => {
                    let theme_path = omarchy_theme_path();
                    if !theme_path.exists() {
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        return Some((Message::Tick, WatcherState::Ready));
                    }

                    let (tx, rx) = async_mpsc::unbounded_channel();
                    let watcher_result = RecommendedWatcher::new(
                        move |res: Result<notify::Event, notify::Error>| {
                            if let Ok(event) = res
                                && (event.kind.is_modify() || event.kind.is_create())
                            {
                                let _ = tx.send(());
                            }
                        },
                        notify::Config::default(),
                    );

                    match watcher_result {
                        Ok(mut watcher) => {
                            if watcher
                                .watch(&theme_path, RecursiveMode::NonRecursive)
                                .is_ok()
                            {
                                Some((
                                    Message::Tick,
                                    WatcherState::Watching {
                                        _watcher: watcher,
                                        rx,
                                    },
                                ))
                            } else {
                                Some((Message::Tick, WatcherState::Unavailable))
                            }
                        }
                        Err(_) => Some((Message::Tick, WatcherState::Unavailable)),
                    }
                }
                WatcherState::Watching { _watcher, mut rx } => {
                    match timeout(Duration::from_millis(500), rx.recv()).await {
                        Ok(Some(())) => Some((
                            Message::OmarchyThemeChanged,
                            WatcherState::Watching { _watcher, rx },
                        )),
                        Ok(None) => Some((Message::Tick, WatcherState::Ready)),
                        Err(_) => Some((Message::Tick, WatcherState::Watching { _watcher, rx })),
                    }
                }
                WatcherState::Unavailable => {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    Some((Message::Tick, WatcherState::Ready))
                }
            }
        })
    })
}

enum WatcherState {
    Ready,
    Watching {
        _watcher: RecommendedWatcher,
        rx: async_mpsc::UnboundedReceiver<()>,
    },
    Unavailable,
}
