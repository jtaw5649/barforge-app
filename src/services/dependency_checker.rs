use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DepType {
    Binary,
    PythonModule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepSpec {
    pub name: String,
    pub dep_type: DepType,
    #[serde(default)]
    pub version_req: Option<String>,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct DepResult {
    pub spec: DepSpec,
    pub satisfied: bool,
    pub found_version: Option<String>,
    pub path: Option<PathBuf>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DepReport {
    pub all_satisfied: bool,
    pub missing_required: Vec<String>,
    pub results: HashMap<String, DepResult>,
}

#[derive(Debug, Error)]
pub enum DepCheckError {
    #[error("Invalid binary name: {0}")]
    InvalidBinaryName(String),

    #[error("Invalid Python module name: {0}")]
    InvalidPythonModuleName(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
}

const VALID_BINARY_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-+.";
const VALID_PYTHON_MODULE_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";

#[must_use]
pub fn is_valid_binary_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && !name.contains('/')
        && !name.contains('\0')
        && name.chars().all(|c| VALID_BINARY_CHARS.contains(c))
}

#[must_use]
pub fn is_valid_python_module_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && !name.starts_with('_')
        && name.chars().all(|c| VALID_PYTHON_MODULE_CHARS.contains(c))
}

pub fn check_binary(spec: &DepSpec) -> Result<DepResult, DepCheckError> {
    if !is_valid_binary_name(&spec.name) {
        return Err(DepCheckError::InvalidBinaryName(spec.name.clone()));
    }

    let path = which::which(&spec.name).ok();
    let found = path.is_some();

    let version = if found {
        extract_binary_version(&spec.name)
    } else {
        None
    };

    Ok(DepResult {
        spec: spec.clone(),
        satisfied: found,
        found_version: version,
        path,
        error: None,
    })
}

pub fn check_python_module(spec: &DepSpec) -> Result<DepResult, DepCheckError> {
    if !is_valid_python_module_name(&spec.name) {
        return Err(DepCheckError::InvalidPythonModuleName(spec.name.clone()));
    }

    let output = Command::new("python3")
        .arg("-c")
        .env("WAYBAR_CHECK_MODULE", &spec.name)
        .arg("import os; __import__(os.environ['WAYBAR_CHECK_MODULE'])")
        .output();

    let (satisfied, error) = match output {
        Ok(out) if out.status.success() => (true, None),
        Ok(out) => (
            false,
            Some(String::from_utf8_lossy(&out.stderr).to_string()),
        ),
        Err(e) => (false, Some(e.to_string())),
    };

    let version = if satisfied {
        extract_python_module_version(&spec.name)
    } else {
        None
    };

    Ok(DepResult {
        spec: spec.clone(),
        satisfied,
        found_version: version,
        path: None,
        error,
    })
}

fn extract_binary_version(name: &str) -> Option<String> {
    let output = Command::new(name).arg("--version").output().ok()?;

    let text = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    extract_version(&text)
}

fn extract_python_module_version(name: &str) -> Option<String> {
    let output = Command::new("python3")
        .arg("-c")
        .env("WAYBAR_CHECK_MODULE", name)
        .arg("import os; m = __import__(os.environ['WAYBAR_CHECK_MODULE']); print(getattr(m, '__version__', 'unknown'))")
        .output()
        .ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if version != "unknown" {
            return Some(version);
        }
    }
    None
}

#[must_use]
pub fn extract_version(text: &str) -> Option<String> {
    let version_re = Regex::new(r"\b(\d+\.\d+(?:\.\d+)?(?:-[a-zA-Z0-9.]+)?)\b").ok()?;

    for cap in version_re.captures_iter(text) {
        let version = &cap[1];
        if !looks_like_ip_address(version) {
            return Some(version.to_string());
        }
    }
    None
}

fn looks_like_ip_address(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() == 4 {
        return parts.iter().all(|p| {
            p.parse::<u8>().is_ok() || (p.len() <= 3 && p.chars().all(|c| c.is_ascii_digit()))
        });
    }
    false
}

pub fn check_dependencies(specs: &[DepSpec]) -> DepReport {
    let mut results = HashMap::new();
    let mut missing_required = Vec::new();

    for spec in specs {
        let result = match spec.dep_type {
            DepType::Binary => check_binary(spec),
            DepType::PythonModule => check_python_module(spec),
        };

        let dep_result = match result {
            Ok(r) => r,
            Err(e) => DepResult {
                spec: spec.clone(),
                satisfied: false,
                found_version: None,
                path: None,
                error: Some(e.to_string()),
            },
        };

        if !dep_result.satisfied && !spec.optional {
            missing_required.push(spec.name.clone());
        }

        results.insert(spec.name.clone(), dep_result);
    }

    let all_satisfied = missing_required.is_empty();

    DepReport {
        all_satisfied,
        missing_required,
        results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_name_validation_accepts_valid_names() {
        assert!(is_valid_binary_name("ls"));
        assert!(is_valid_binary_name("python3"));
        assert!(is_valid_binary_name("my-binary"));
        assert!(is_valid_binary_name("my_binary"));
        assert!(is_valid_binary_name("binary.sh"));
        assert!(is_valid_binary_name("g++"));
    }

    #[test]
    fn binary_name_validation_rejects_injection_attempts() {
        assert!(!is_valid_binary_name(""));
        assert!(!is_valid_binary_name("ls; rm -rf /"));
        assert!(!is_valid_binary_name("$(whoami)"));
        assert!(!is_valid_binary_name("`id`"));
        assert!(!is_valid_binary_name("/bin/ls"));
        assert!(!is_valid_binary_name("../../../etc/passwd"));
        assert!(!is_valid_binary_name("bin\0ary"));
    }

    #[test]
    fn python_module_validation_accepts_valid_names() {
        assert!(is_valid_python_module_name("requests"));
        assert!(is_valid_python_module_name("numpy"));
        assert!(is_valid_python_module_name("my_module"));
        assert!(is_valid_python_module_name("module123"));
    }

    #[test]
    fn python_module_validation_rejects_injection() {
        assert!(!is_valid_python_module_name(""));
        assert!(!is_valid_python_module_name("__import__('os')"));
        assert!(!is_valid_python_module_name("os; import sys"));
        assert!(!is_valid_python_module_name("_private"));
        assert!(!is_valid_python_module_name("module-name"));
        assert!(!is_valid_python_module_name("module.submodule"));
    }

    #[test]
    fn version_extraction_finds_semver() {
        assert_eq!(extract_version("version 1.2.3"), Some("1.2.3".to_string()));
        assert_eq!(extract_version("version 2.0.0"), Some("2.0.0".to_string()));
        assert_eq!(
            extract_version("Python 3.11.5"),
            Some("3.11.5".to_string())
        );
        assert_eq!(
            extract_version("git version 2.42.0"),
            Some("2.42.0".to_string())
        );
    }

    #[test]
    fn version_extraction_avoids_ip_addresses() {
        assert_ne!(
            extract_version("Server at 192.168.1.1"),
            Some("192.168.1.1".to_string())
        );
        assert_ne!(
            extract_version("IP: 10.0.0.1"),
            Some("10.0.0.1".to_string())
        );
    }

    #[test]
    fn version_extraction_handles_prereleases() {
        assert_eq!(
            extract_version("1.0.0-alpha"),
            Some("1.0.0-alpha".to_string())
        );
        assert_eq!(
            extract_version("2.0.0-beta.1"),
            Some("2.0.0-beta.1".to_string())
        );
    }

    #[test]
    fn finds_existing_binary_ls() {
        let spec = DepSpec {
            name: "ls".to_string(),
            dep_type: DepType::Binary,
            version_req: None,
            optional: false,
        };
        let result = check_binary(&spec).unwrap();
        assert!(result.satisfied);
        assert!(result.path.is_some());
    }

    #[test]
    fn reports_missing_binary() {
        let spec = DepSpec {
            name: "nonexistent-binary-xyz123".to_string(),
            dep_type: DepType::Binary,
            version_req: None,
            optional: false,
        };
        let result = check_binary(&spec).unwrap();
        assert!(!result.satisfied);
        assert!(result.path.is_none());
    }

    #[test]
    fn check_dependencies_reports_correctly() {
        let specs = vec![
            DepSpec {
                name: "ls".to_string(),
                dep_type: DepType::Binary,
                version_req: None,
                optional: false,
            },
            DepSpec {
                name: "nonexistent-xyz".to_string(),
                dep_type: DepType::Binary,
                version_req: None,
                optional: false,
            },
            DepSpec {
                name: "also-nonexistent".to_string(),
                dep_type: DepType::Binary,
                version_req: None,
                optional: true,
            },
        ];

        let report = check_dependencies(&specs);
        assert!(!report.all_satisfied);
        assert_eq!(report.missing_required.len(), 1);
        assert!(report.missing_required.contains(&"nonexistent-xyz".to_string()));
    }
}
