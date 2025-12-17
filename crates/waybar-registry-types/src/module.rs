use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModuleUuidError {
    #[error("missing '@' separator in UUID")]
    MissingAtSymbol,
    #[error("empty name in UUID")]
    EmptyName,
    #[error("empty namespace in UUID")]
    EmptyNamespace,
    #[error("invalid character in UUID (path separator or null)")]
    InvalidCharacter,
    #[error("path traversal attempt detected in UUID")]
    PathTraversalAttempt,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleUuid {
    name: String,
    namespace: String,
}

impl Serialize for ModuleUuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ModuleUuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ModuleUuid::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl ModuleUuid {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}

impl TryFrom<&str> for ModuleUuid {
    type Error = ModuleUuidError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let Some(at_pos) = value.find('@') else {
            return Err(ModuleUuidError::MissingAtSymbol);
        };

        let name = &value[..at_pos];
        let namespace = &value[at_pos + 1..];

        if name.is_empty() {
            return Err(ModuleUuidError::EmptyName);
        }

        if namespace.is_empty() {
            return Err(ModuleUuidError::EmptyNamespace);
        }

        if name == ".." || namespace == ".." {
            return Err(ModuleUuidError::PathTraversalAttempt);
        }

        if value.contains('/') || value.contains('\\') || value.contains('\0') {
            return Err(ModuleUuidError::InvalidCharacter);
        }

        Ok(Self {
            name: name.to_string(),
            namespace: namespace.to_string(),
        })
    }
}

impl fmt::Display for ModuleUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.namespace)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModuleVersion(semver::Version);

impl ModuleVersion {
    pub fn major(&self) -> u64 {
        self.0.major
    }

    pub fn minor(&self) -> u64 {
        self.0.minor
    }

    pub fn patch(&self) -> u64 {
        self.0.patch
    }
}

impl TryFrom<&str> for ModuleVersion {
    type Error = semver::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(semver::Version::parse(value)?))
    }
}

impl fmt::Display for ModuleVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod module_uuid {
        use super::*;

        #[test]
        fn parses_valid_string() {
            let uuid = ModuleUuid::try_from("weather-wttr@waybar-modules").unwrap();
            assert_eq!(uuid.name(), "weather-wttr");
            assert_eq!(uuid.namespace(), "waybar-modules");
        }

        #[test]
        fn formats_to_string() {
            let uuid = ModuleUuid::try_from("my-module@my-namespace").unwrap();
            assert_eq!(uuid.to_string(), "my-module@my-namespace");
        }

        #[test]
        fn rejects_missing_at_symbol() {
            let result = ModuleUuid::try_from("invalid-uuid");
            assert!(matches!(result, Err(ModuleUuidError::MissingAtSymbol)));
        }

        #[test]
        fn rejects_empty_name() {
            let result = ModuleUuid::try_from("@namespace");
            assert!(matches!(result, Err(ModuleUuidError::EmptyName)));
        }

        #[test]
        fn rejects_empty_namespace() {
            let result = ModuleUuid::try_from("name@");
            assert!(matches!(result, Err(ModuleUuidError::EmptyNamespace)));
        }

        #[test]
        fn uses_first_at_symbol_when_multiple() {
            let uuid = ModuleUuid::try_from("name@namespace@extra").unwrap();
            assert_eq!(uuid.name(), "name");
            assert_eq!(uuid.namespace(), "namespace@extra");
        }

        #[test]
        fn equality_works() {
            let uuid1 = ModuleUuid::try_from("test@ns").unwrap();
            let uuid2 = ModuleUuid::try_from("test@ns").unwrap();
            assert_eq!(uuid1, uuid2);
        }

        #[test]
        fn inequality_works() {
            let uuid1 = ModuleUuid::try_from("test@ns1").unwrap();
            let uuid2 = ModuleUuid::try_from("test@ns2").unwrap();
            assert_ne!(uuid1, uuid2);
        }

        #[test]
        fn hashes_consistently() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(ModuleUuid::try_from("a@b").unwrap());
            set.insert(ModuleUuid::try_from("a@b").unwrap());
            assert_eq!(set.len(), 1);
        }

        #[test]
        fn clones_correctly() {
            let uuid = ModuleUuid::try_from("test@ns").unwrap();
            let cloned = uuid.clone();
            assert_eq!(uuid, cloned);
        }

        #[test]
        fn rejects_forward_slash() {
            let result = ModuleUuid::try_from("test/../escape@ns");
            assert!(matches!(result, Err(ModuleUuidError::InvalidCharacter)));
        }

        #[test]
        fn rejects_backslash() {
            let result = ModuleUuid::try_from("test\\escape@ns");
            assert!(matches!(result, Err(ModuleUuidError::InvalidCharacter)));
        }

        #[test]
        fn rejects_null_byte() {
            let result = ModuleUuid::try_from("test\0escape@ns");
            assert!(matches!(result, Err(ModuleUuidError::InvalidCharacter)));
        }

        #[test]
        fn rejects_parent_dir_reference() {
            let result = ModuleUuid::try_from("..@ns");
            assert!(matches!(result, Err(ModuleUuidError::PathTraversalAttempt)));
        }

        #[test]
        fn rejects_path_chars_in_namespace() {
            let result = ModuleUuid::try_from("test@ns/evil");
            assert!(matches!(result, Err(ModuleUuidError::InvalidCharacter)));
        }
    }

    mod module_version {
        use super::*;

        #[test]
        fn parses_semver_string() {
            let version = ModuleVersion::try_from("1.2.3").unwrap();
            assert_eq!(version.major(), 1);
            assert_eq!(version.minor(), 2);
            assert_eq!(version.patch(), 3);
        }

        #[test]
        fn formats_to_string() {
            let version = ModuleVersion::try_from("2.0.1").unwrap();
            assert_eq!(version.to_string(), "2.0.1");
        }

        #[test]
        fn rejects_invalid_format() {
            let result = ModuleVersion::try_from("not-a-version");
            assert!(result.is_err());
        }

        #[test]
        fn orders_correctly() {
            let v1 = ModuleVersion::try_from("1.0.0").unwrap();
            let v2 = ModuleVersion::try_from("1.0.1").unwrap();
            let v3 = ModuleVersion::try_from("1.1.0").unwrap();
            let v4 = ModuleVersion::try_from("2.0.0").unwrap();

            assert!(v1 < v2);
            assert!(v2 < v3);
            assert!(v3 < v4);
        }

        #[test]
        fn equality_works() {
            let v1 = ModuleVersion::try_from("1.2.3").unwrap();
            let v2 = ModuleVersion::try_from("1.2.3").unwrap();
            assert_eq!(v1, v2);
        }

        #[test]
        fn parses_prerelease() {
            let version = ModuleVersion::try_from("1.0.0-alpha.1").unwrap();
            assert_eq!(version.major(), 1);
        }

        #[test]
        fn orders_prerelease_correctly() {
            let alpha = ModuleVersion::try_from("1.0.0-alpha").unwrap();
            let beta = ModuleVersion::try_from("1.0.0-beta").unwrap();
            let release = ModuleVersion::try_from("1.0.0").unwrap();

            assert!(alpha < beta);
            assert!(beta < release);
        }
    }
}
