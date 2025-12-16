use std::process::Command;
use std::sync::OnceLock;

static WAYBAR_VERSION: OnceLock<Option<String>> = OnceLock::new();

pub fn detect_waybar_version() -> Option<String> {
    WAYBAR_VERSION
        .get_or_init(|| {
            Command::new("waybar")
                .arg("--version")
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout)
                            .ok()
                            .and_then(|s| parse_version(&s))
                    } else {
                        None
                    }
                })
        })
        .clone()
}

fn parse_version(output: &str) -> Option<String> {
    output
        .lines()
        .next()
        .and_then(|line| {
            line.split_whitespace()
                .find(|word| word.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                .map(|v| v.to_string())
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompatibilityStatus {
    Compatible,
    MaybeCompatible,
    #[default]
    Unknown,
}

impl CompatibilityStatus {
    pub fn from_versions(module_versions: &[String], local_version: Option<&str>) -> Self {
        let Some(local) = local_version else {
            return Self::Unknown;
        };

        if module_versions.is_empty() {
            return Self::Unknown;
        }

        let local_major = extract_major_version(local);

        for v in module_versions {
            if let Some(module_major) = extract_major_version(v)
                && let Some(local_major) = local_major
                && module_major == local_major
            {
                return Self::Compatible;
            }
        }

        Self::MaybeCompatible
    }
}

fn extract_major_version(version: &str) -> Option<u32> {
    version
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_standard() {
        let output = "waybar 0.10.4";
        assert_eq!(parse_version(output), Some("0.10.4".to_string()));
    }

    #[test]
    fn test_parse_version_with_prefix() {
        let output = "Waybar version 0.9.22";
        assert_eq!(parse_version(output), Some("0.9.22".to_string()));
    }

    #[test]
    fn test_extract_major() {
        assert_eq!(extract_major_version("0.10.4"), Some(0));
        assert_eq!(extract_major_version("1.0.0"), Some(1));
    }

    #[test]
    fn test_compatibility_compatible() {
        let versions = vec!["0.10".to_string(), "0.9".to_string()];
        assert_eq!(
            CompatibilityStatus::from_versions(&versions, Some("0.10.4")),
            CompatibilityStatus::Compatible
        );
    }

    #[test]
    fn test_compatibility_maybe() {
        let versions = vec!["0.9".to_string()];
        assert_eq!(
            CompatibilityStatus::from_versions(&versions, Some("1.0.0")),
            CompatibilityStatus::MaybeCompatible
        );
    }

    #[test]
    fn test_compatibility_unknown_no_local() {
        let versions = vec!["0.10".to_string()];
        assert_eq!(
            CompatibilityStatus::from_versions(&versions, None),
            CompatibilityStatus::Unknown
        );
    }

    #[test]
    fn test_compatibility_unknown_empty_versions() {
        assert_eq!(
            CompatibilityStatus::from_versions(&[], Some("0.10.4")),
            CompatibilityStatus::Unknown
        );
    }
}
