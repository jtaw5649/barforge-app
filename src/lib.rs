pub mod config;
pub mod domain;
pub mod services;
pub mod ui;

mod application;
mod window;

pub use application::Application;
pub use window::Window;

#[cfg(test)]
pub mod test_utils {
    use std::sync::OnceLock;
    use std::thread::ThreadId;

    static GTK_INIT_THREAD: OnceLock<ThreadId> = OnceLock::new();

    pub fn init_gtk() -> bool {
        let current = std::thread::current().id();

        match GTK_INIT_THREAD.get() {
            Some(&init_thread) if init_thread == current => true,
            Some(_) => false,
            None => {
                if gtk::init().is_ok() {
                    let _ = GTK_INIT_THREAD.set(current);
                    true
                } else {
                    false
                }
            }
        }
    }

    #[macro_export]
    macro_rules! skip_if_no_gtk {
        () => {
            if !$crate::test_utils::init_gtk() {
                eprintln!("SKIPPED: GTK not available on this thread");
                return;
            }
        };
    }
}
