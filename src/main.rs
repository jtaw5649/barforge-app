use clap::Parser;
use std::io::IsTerminal;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use waybar_manager::cli::{Cli, Commands};

fn setup_tracing() {
    let is_terminal = std::io::stderr().is_terminal();

    let default_filter = if is_terminal {
        "waybar_manager=debug"
    } else {
        "waybar_manager=info"
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
        let _ = writeln!(std::io::stderr(), "{msg}");
    }));
}

fn main() -> iced::Result {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::InternalSandboxExec { script, module_dir }) => {
            Cli::run_sandbox_exec(script, module_dir);
        }
        Some(Commands::Gui) | None => run_gui(),
    }
}

fn run_gui() -> iced::Result {
    setup_tracing();
    setup_panic_handler();

    tracing::info!(
        "Waybar Manager v{} starting (PID {})",
        env!("CARGO_PKG_VERSION"),
        std::process::id()
    );

    iced::application(
        waybar_manager::app::App::new,
        waybar_manager::app::App::update,
        waybar_manager::app::App::view,
    )
    .title("Waybar Manager")
    .theme(waybar_manager::app::App::theme)
    .subscription(waybar_manager::app::App::subscription)
    .window_size((1200.0, 800.0))
    .run()
}
