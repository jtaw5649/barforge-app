use gtk::prelude::*;
use std::io::IsTerminal;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

fn setup_tracing() {
    let is_terminal = std::io::stderr().is_terminal();

    let default_filter = if is_terminal {
        "waybar_manager=debug,gtk=warn,gdk=warn,glib=warn"
    } else {
        "waybar_manager=info,gtk=warn,gdk=warn,glib=warn"
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let fmt_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(is_terminal)
        .with_target(false);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();
}

fn setup_panic_handler() {
    use std::io::Write;

    std::panic::set_hook(Box::new(|panic_info| {
        let mut msg = String::from("PANIC: ");
        if let Some(location) = panic_info.location() {
            msg.push_str(&format!(
                "{}:{}:{} - ",
                location.file(),
                location.line(),
                location.column()
            ));
        }
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            msg.push_str(s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            msg.push_str(s);
        } else {
            msg.push_str("unknown panic");
        }
        let _ = writeln!(std::io::stderr(), "{}", msg);
    }));
}

fn setup_signal_handlers() {
    use glib::ControlFlow;

    for sig in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP] {
        glib::unix_signal_add_local(sig, move || {
            info!("received signal {}", sig);
            ControlFlow::Continue
        });
    }
}

fn ignore_rt_signals() {
    unsafe {
        for sig in [41, 42, 43, 44, 45, 46, 47, 48, 49, 50] {
            libc::signal(sig, libc::SIG_IGN);
        }
    }
}

fn main() -> glib::ExitCode {
    setup_tracing();
    setup_panic_handler();
    ignore_rt_signals();

    info!(
        "Waybar Extension Manager v{} starting (PID {})",
        env!("CARGO_PKG_VERSION"),
        std::process::id()
    );

    if std::env::var("GSK_RENDERER").is_err() {
        unsafe { std::env::set_var("GSK_RENDERER", "gl") };
    }

    let app = waybar_manager::Application::new();

    setup_signal_handlers();

    let exit_code = app.run();

    info!("exiting with code {:?}", exit_code);
    exit_code
}
