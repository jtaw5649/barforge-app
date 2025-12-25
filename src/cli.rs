use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "barforge")]
#[command(about = "Barforge module manager for Waybar")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Launch the graphical interface (default)")]
    Gui,

    #[command(hide = true)]
    InternalSandboxExec {
        #[arg(long)]
        script: PathBuf,
        #[arg(long)]
        module_dir: PathBuf,
    },
}

impl Cli {
    pub fn run_sandbox_exec(script: PathBuf, module_dir: PathBuf) -> ! {
        let config_json = std::env::var("BARFORGE_SANDBOX_CONFIG")
            .expect("Missing BARFORGE_SANDBOX_CONFIG environment variable");

        let config: crate::security::sandbox::SandboxConfig =
            serde_json::from_str(&config_json).expect("Invalid sandbox configuration JSON");

        let result = crate::security::sandbox::apply(&config);

        match result.status {
            crate::security::sandbox::SandboxStatus::FullyEnforced
            | crate::security::sandbox::SandboxStatus::PartiallyEnforced => {}
            crate::security::sandbox::SandboxStatus::NotSupported => {
                eprintln!("WARNING: Landlock sandbox not supported on this kernel");
            }
            crate::security::sandbox::SandboxStatus::Failed => {
                eprintln!("ERROR: Failed to apply sandbox restrictions");
                std::process::exit(127);
            }
        }

        let status = std::process::Command::new("bash")
            .arg(&script)
            .current_dir(&module_dir)
            .env("MODULE_DIR", &module_dir)
            .status()
            .expect("Failed to run script");

        std::process::exit(status.code().unwrap_or(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_no_args_as_none() {
        let cli = Cli::parse_from(["barforge"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn cli_parses_gui_command() {
        let cli = Cli::parse_from(["barforge", "gui"]);
        assert!(matches!(cli.command, Some(Commands::Gui)));
    }

    #[test]
    fn cli_parses_internal_sandbox_exec() {
        let cli = Cli::parse_from([
            "barforge",
            "internal-sandbox-exec",
            "--script",
            "/path/to/script.sh",
            "--module-dir",
            "/path/to/module",
        ]);
        match cli.command {
            Some(Commands::InternalSandboxExec { script, module_dir }) => {
                assert_eq!(script, PathBuf::from("/path/to/script.sh"));
                assert_eq!(module_dir, PathBuf::from("/path/to/module"));
            }
            _ => panic!("Expected InternalSandboxExec command"),
        }
    }
}
