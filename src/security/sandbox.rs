use landlock::{
    ABI, Access, AccessFs, AccessNet, LandlockStatus, NetPort, PathBeneath, PathFd, Ruleset,
    RulesetAttr, RulesetCreatedAttr, RulesetStatus,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const LANDLOCK_ABI: ABI = ABI::V5;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SandboxConfig {
    pub allow_network: bool,
    pub allowed_ports: Vec<u16>,
    pub extra_ro_paths: Vec<PathBuf>,
    pub extra_rw_paths: Vec<PathBuf>,
}

#[derive(Debug)]
#[must_use]
pub struct SandboxResult {
    pub status: SandboxStatus,
    pub abi_version: Option<u32>,
    pub network_isolated: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SandboxStatus {
    FullyEnforced,
    PartiallyEnforced,
    NotSupported,
    Failed,
}

impl SandboxStatus {
    #[must_use]
    pub fn is_secure(&self) -> bool {
        matches!(
            self,
            SandboxStatus::FullyEnforced | SandboxStatus::PartiallyEnforced
        )
    }

    #[must_use]
    pub fn allows_execution(&self) -> bool {
        !matches!(self, SandboxStatus::Failed)
    }

    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            SandboxStatus::FullyEnforced => "All security restrictions active",
            SandboxStatus::PartiallyEnforced => "Some restrictions unavailable (kernel too old)",
            SandboxStatus::NotSupported => "Landlock not supported on this kernel",
            SandboxStatus::Failed => "Failed to apply sandbox restrictions",
        }
    }

    #[must_use]
    pub fn severity(&self) -> SandboxSeverity {
        match self {
            SandboxStatus::FullyEnforced => SandboxSeverity::Success,
            SandboxStatus::PartiallyEnforced => SandboxSeverity::Warning,
            SandboxStatus::NotSupported | SandboxStatus::Failed => SandboxSeverity::Error,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SandboxSeverity {
    Success,
    Warning,
    Error,
}

const ALLOWED_READ_PATH_PARENTS: &[&str] = &[
    "/usr/share",
    "/usr/lib",
    "/usr/local/share",
    "/etc/fonts",
    "/var/lib/fonts",
    "/opt",
];

const ALLOWED_WRITE_PATH_PARENTS: &[&str] = &["/tmp", "/var/tmp"];

#[must_use]
pub fn is_allowed_read_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    ALLOWED_READ_PATH_PARENTS
        .iter()
        .any(|parent| path_str.starts_with(parent))
}

#[must_use]
pub fn is_allowed_write_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    ALLOWED_WRITE_PATH_PARENTS
        .iter()
        .any(|parent| path_str.starts_with(parent))
}

pub fn apply(config: &SandboxConfig) -> SandboxResult {
    let waybar_config = dirs::config_dir()
        .map(|p| p.join("waybar"))
        .unwrap_or_else(|| PathBuf::from(shellexpand::tilde("~/.config/waybar").to_string()));

    let cache_dir = dirs::cache_dir()
        .map(|p| p.join("waybar-manager"))
        .unwrap_or_else(|| {
            PathBuf::from(shellexpand::tilde("~/.cache/waybar-manager").to_string())
        });

    let _ = std::fs::create_dir_all(&waybar_config);
    let _ = std::fs::create_dir_all(&cache_dir);

    match apply_rules(&waybar_config, &cache_dir, config) {
        Ok(status) => {
            let abi_version = match status.landlock {
                LandlockStatus::Available { effective_abi, .. } => Some(effective_abi as u32),
                _ => None,
            };

            let sandbox_status = match (&status.landlock, &status.ruleset) {
                (LandlockStatus::Available { .. }, RulesetStatus::FullyEnforced) => {
                    SandboxStatus::FullyEnforced
                }
                (LandlockStatus::Available { .. }, RulesetStatus::PartiallyEnforced) => {
                    SandboxStatus::PartiallyEnforced
                }
                (LandlockStatus::NotEnabled | LandlockStatus::NotImplemented, _)
                | (_, RulesetStatus::NotEnforced) => SandboxStatus::NotSupported,
            };

            SandboxResult {
                status: sandbox_status,
                abi_version,
                network_isolated: config.allow_network,
            }
        }
        Err(_) => SandboxResult {
            status: SandboxStatus::Failed,
            abi_version: None,
            network_isolated: false,
        },
    }
}

fn apply_rules(
    waybar_config: &Path,
    cache_dir: &Path,
    config: &SandboxConfig,
) -> Result<landlock::RestrictionStatus, landlock::RulesetError> {
    let read_only = AccessFs::from_read(LANDLOCK_ABI);
    let read_write = AccessFs::from_all(LANDLOCK_ABI);

    let mut ruleset_attr = Ruleset::default().handle_access(read_write)?;

    if config.allow_network {
        ruleset_attr = ruleset_attr.handle_access(AccessNet::ConnectTcp)?;
    }

    let mut ruleset = ruleset_attr.create()?;

    let system_ro = [
        "/usr/share",
        "/usr/lib",
        "/usr/lib64",
        "/lib",
        "/lib64",
        "/etc/fonts",
        "/etc/ssl",
        "/etc/ca-certificates",
        "/etc/pki",
        "/etc/resolv.conf",
        "/etc/hosts",
        "/etc/nsswitch.conf",
        "/run/systemd/resolve",
    ];

    for path in system_ro {
        if let Ok(fd) = PathFd::new(path) {
            ruleset = ruleset.add_rule(PathBeneath::new(fd, read_only))?;
        }
    }

    if let Ok(fd) = PathFd::new(waybar_config) {
        ruleset = ruleset.add_rule(PathBeneath::new(fd, read_write))?;
    }
    if let Ok(fd) = PathFd::new(cache_dir) {
        ruleset = ruleset.add_rule(PathBeneath::new(fd, read_write))?;
    }
    if let Ok(fd) = PathFd::new("/tmp") {
        ruleset = ruleset.add_rule(PathBeneath::new(fd, read_write))?;
    }

    for path in ["/usr/bin", "/bin", "/usr/local/bin"] {
        if let Ok(fd) = PathFd::new(path) {
            ruleset = ruleset.add_rule(PathBeneath::new(fd, read_only))?;
        }
    }

    for path in &config.extra_ro_paths {
        if is_allowed_read_path(path) {
            if let Ok(fd) = PathFd::new(path) {
                ruleset = ruleset.add_rule(PathBeneath::new(fd, read_only))?;
            }
        } else {
            tracing::warn!(
                "Rejected read path request outside whitelist: {}",
                path.display()
            );
        }
    }

    for path in &config.extra_rw_paths {
        if is_allowed_write_path(path) {
            if let Ok(fd) = PathFd::new(path) {
                ruleset = ruleset.add_rule(PathBeneath::new(fd, read_write))?;
            }
        } else {
            tracing::warn!(
                "Rejected write path request outside whitelist: {}",
                path.display()
            );
        }
    }

    if config.allow_network {
        for port in &config.allowed_ports {
            ruleset = ruleset.add_rule(NetPort::new(*port, AccessNet::ConnectTcp))?;
        }
    }

    ruleset.restrict_self()
}

#[must_use]
pub fn is_available() -> bool {
    Ruleset::default()
        .handle_access(AccessFs::from_all(ABI::V1))
        .and_then(|r| r.create())
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_path_whitelist_allows_system_directories() {
        assert!(is_allowed_read_path(Path::new("/usr/share/fonts")));
        assert!(is_allowed_read_path(Path::new("/usr/share/icons/hicolor")));
        assert!(is_allowed_read_path(Path::new("/etc/fonts/conf.d")));
        assert!(is_allowed_read_path(Path::new("/var/lib/fonts/custom")));
        assert!(is_allowed_read_path(Path::new("/usr/lib/python3.12")));
        assert!(is_allowed_read_path(Path::new("/opt/application")));
    }

    #[test]
    fn read_path_whitelist_rejects_sensitive_paths() {
        assert!(!is_allowed_read_path(Path::new("/etc/shadow")));
        assert!(!is_allowed_read_path(Path::new("/etc/passwd")));
        assert!(!is_allowed_read_path(Path::new("/home/user/.ssh")));
        assert!(!is_allowed_read_path(Path::new("/root")));
        assert!(!is_allowed_read_path(Path::new("/var/log/auth.log")));
        assert!(!is_allowed_read_path(Path::new("/etc/sudoers")));
    }

    #[test]
    fn write_path_whitelist_allows_temp_directories() {
        assert!(is_allowed_write_path(Path::new("/tmp/waybar-module")));
        assert!(is_allowed_write_path(Path::new("/var/tmp/cache")));
        assert!(is_allowed_write_path(Path::new("/tmp")));
    }

    #[test]
    fn write_path_whitelist_rejects_system_paths() {
        assert!(!is_allowed_write_path(Path::new("/etc/passwd")));
        assert!(!is_allowed_write_path(Path::new("/etc/shadow")));
        assert!(!is_allowed_write_path(Path::new("/home/user/.bashrc")));
        assert!(!is_allowed_write_path(Path::new("/usr/bin/malicious")));
        assert!(!is_allowed_write_path(Path::new(
            "/root/.ssh/authorized_keys"
        )));
        assert!(!is_allowed_write_path(Path::new("/var/log/auth.log")));
    }

    #[test]
    fn sandbox_status_severity_mapping() {
        assert_eq!(
            SandboxStatus::FullyEnforced.severity(),
            SandboxSeverity::Success
        );
        assert_eq!(
            SandboxStatus::PartiallyEnforced.severity(),
            SandboxSeverity::Warning
        );
        assert_eq!(
            SandboxStatus::NotSupported.severity(),
            SandboxSeverity::Error
        );
        assert_eq!(SandboxStatus::Failed.severity(), SandboxSeverity::Error);
    }

    #[test]
    fn sandbox_status_is_secure() {
        assert!(SandboxStatus::FullyEnforced.is_secure());
        assert!(SandboxStatus::PartiallyEnforced.is_secure());
        assert!(!SandboxStatus::NotSupported.is_secure());
        assert!(!SandboxStatus::Failed.is_secure());
    }

    #[test]
    fn sandbox_status_allows_execution() {
        assert!(SandboxStatus::FullyEnforced.allows_execution());
        assert!(SandboxStatus::PartiallyEnforced.allows_execution());
        assert!(SandboxStatus::NotSupported.allows_execution());
        assert!(!SandboxStatus::Failed.allows_execution());
    }

    #[test]
    fn sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(!config.allow_network);
        assert!(config.allowed_ports.is_empty());
        assert!(config.extra_ro_paths.is_empty());
        assert!(config.extra_rw_paths.is_empty());
    }

    #[test]
    fn sandbox_config_serialization() {
        let config = SandboxConfig {
            allow_network: true,
            allowed_ports: vec![80, 443],
            extra_ro_paths: vec![PathBuf::from("/usr/share/fonts")],
            extra_rw_paths: vec![PathBuf::from("/tmp/module")],
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SandboxConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.allow_network, deserialized.allow_network);
        assert_eq!(config.allowed_ports, deserialized.allowed_ports);
        assert_eq!(config.extra_ro_paths, deserialized.extra_ro_paths);
        assert_eq!(config.extra_rw_paths, deserialized.extra_rw_paths);
    }
}
