use crate::security::SandboxConfig;
use crate::services::dependency_checker::{DepSpec, DepType};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageConfigError {
    #[error("Failed to read Package.toml: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse Package.toml: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

#[derive(Debug, Deserialize)]
pub struct PackageToml {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, DepEntry>,
    #[serde(default)]
    pub permissions: Permissions,
}

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub install_script: Option<String>,
    #[serde(default)]
    pub uninstall_script: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DepEntry {
    Simple(String),
    Detailed {
        version: Option<String>,
        #[serde(default)]
        optional: bool,
        #[serde(rename = "type", default = "default_dep_type")]
        dep_type: String,
    },
}

fn default_dep_type() -> String {
    "binary".to_string()
}

#[derive(Debug, Default, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub ports: Vec<u16>,
    #[serde(default)]
    pub read_paths: Vec<String>,
    #[serde(default)]
    pub write_paths: Vec<String>,
}

impl std::str::FromStr for PackageToml {
    type Err = PackageConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(toml::from_str(s)?)
    }
}

impl PackageToml {
    pub fn from_file(path: &Path) -> Result<Self, PackageConfigError> {
        let content = std::fs::read_to_string(path)?;
        content.parse()
    }

    pub fn to_dep_specs(&self) -> Vec<DepSpec> {
        self.dependencies
            .iter()
            .map(|(name, entry)| match entry {
                DepEntry::Simple(version) => DepSpec {
                    name: name.clone(),
                    dep_type: DepType::Binary,
                    version_req: Some(version.clone()),
                    optional: false,
                },
                DepEntry::Detailed {
                    version,
                    optional,
                    dep_type,
                } => DepSpec {
                    name: name.clone(),
                    dep_type: match dep_type.as_str() {
                        "python" | "python_module" => DepType::PythonModule,
                        _ => DepType::Binary,
                    },
                    version_req: version.clone(),
                    optional: *optional,
                },
            })
            .collect()
    }

    pub fn to_sandbox_config(&self) -> SandboxConfig {
        SandboxConfig {
            allow_network: self.permissions.network,
            allowed_ports: self.permissions.ports.clone(),
            extra_ro_paths: self
                .permissions
                .read_paths
                .iter()
                .map(|s| shellexpand::tilde(s).to_string().into())
                .collect(),
            extra_rw_paths: self
                .permissions
                .write_paths
                .iter()
                .map(|s| shellexpand::tilde(s).to_string().into())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parses_minimal_package() {
        let toml = r#"
[package]
name = "test-module"
version = "1.0.0"
"#;
        let pkg = PackageToml::from_str(toml).unwrap();
        assert_eq!(pkg.package.name, "test-module");
        assert_eq!(pkg.package.version, "1.0.0");
    }

    #[test]
    fn parses_full_package() {
        let toml = r#"
[package]
name = "full-module"
version = "2.0.0"
description = "A full test module"
install_script = "install.sh"
uninstall_script = "uninstall.sh"

[dependencies]
curl = ">=7.0"
python3 = { version = ">=3.8", optional = true }
requests = { type = "python", optional = false }

[permissions]
network = true
ports = [80, 443]
read_paths = ["/usr/share/fonts"]
write_paths = ["/tmp/module-cache"]
"#;
        let pkg = PackageToml::from_str(toml).unwrap();
        assert_eq!(pkg.package.name, "full-module");
        assert_eq!(
            pkg.package.description,
            Some("A full test module".to_string())
        );
        assert!(pkg.permissions.network);
        assert_eq!(pkg.permissions.ports, vec![80, 443]);
    }

    #[test]
    fn converts_to_dep_specs() {
        let toml = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
curl = ">=7.0"
requests = { type = "python", optional = true }
"#;
        let pkg = PackageToml::from_str(toml).unwrap();
        let specs = pkg.to_dep_specs();

        assert_eq!(specs.len(), 2);

        let curl_spec = specs.iter().find(|s| s.name == "curl").unwrap();
        assert_eq!(curl_spec.dep_type, DepType::Binary);
        assert!(!curl_spec.optional);

        let requests_spec = specs.iter().find(|s| s.name == "requests").unwrap();
        assert_eq!(requests_spec.dep_type, DepType::PythonModule);
        assert!(requests_spec.optional);
    }

    #[test]
    fn converts_to_sandbox_config() {
        let toml = r#"
[package]
name = "test"
version = "1.0.0"

[permissions]
network = true
ports = [443]
read_paths = ["/usr/share"]
write_paths = ["/tmp"]
"#;
        let pkg = PackageToml::from_str(toml).unwrap();
        let config = pkg.to_sandbox_config();

        assert!(config.allow_network);
        assert_eq!(config.allowed_ports, vec![443]);
        assert_eq!(config.extra_ro_paths.len(), 1);
        assert_eq!(config.extra_rw_paths.len(), 1);
    }

    #[test]
    fn handles_missing_optional_sections() {
        let toml = r#"
[package]
name = "minimal"
version = "0.1.0"
"#;
        let pkg = PackageToml::from_str(toml).unwrap();
        assert!(pkg.dependencies.is_empty());
        assert!(!pkg.permissions.network);
        assert!(pkg.permissions.ports.is_empty());
    }
}
