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

        if name == ".." || namespace == ".." || value.contains("../") || value.contains("..\\") {
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
            let uuid = ModuleUuid::try_from("weather-wttr@barforge").unwrap();
            assert_eq!(uuid.to_string(), "weather-wttr@barforge");
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
        fn rejects_path_traversal() {
            let result = ModuleUuid::try_from("../@test");
            assert!(matches!(result, Err(ModuleUuidError::PathTraversalAttempt)));
        }
    }

    mod module_version {
        use super::*;

        #[test]
        fn formats_to_string() {
            let version = ModuleVersion::try_from("2.0.1").unwrap();
            assert_eq!(version.to_string(), "2.0.1");
        }
    }
}
