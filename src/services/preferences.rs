use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use super::paths::{module_preferences_path, preferences_dir};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PreferenceField {
    Text {
        key: String,
        label: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        default: Option<String>,
        #[serde(default)]
        placeholder: Option<String>,
    },
    Boolean {
        key: String,
        label: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        default: Option<bool>,
    },
    Select {
        key: String,
        label: String,
        #[serde(default)]
        description: Option<String>,
        options: Vec<SelectOption>,
        #[serde(default)]
        default: Option<String>,
    },
    Number {
        key: String,
        label: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        default: Option<f64>,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
    },
}

impl PreferenceField {
    pub fn key(&self) -> &str {
        match self {
            PreferenceField::Text { key, .. } => key,
            PreferenceField::Boolean { key, .. } => key,
            PreferenceField::Select { key, .. } => key,
            PreferenceField::Number { key, .. } => key,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            PreferenceField::Text { label, .. } => label,
            PreferenceField::Boolean { label, .. } => label,
            PreferenceField::Select { label, .. } => label,
            PreferenceField::Number { label, .. } => label,
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            PreferenceField::Text { description, .. } => description.as_deref(),
            PreferenceField::Boolean { description, .. } => description.as_deref(),
            PreferenceField::Select { description, .. } => description.as_deref(),
            PreferenceField::Number { description, .. } => description.as_deref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreferencesSchema {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub fields: Vec<PreferenceField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PreferenceValue {
    String(String),
    Bool(bool),
    Number(f64),
}

impl PreferenceValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PreferenceValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PreferenceValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            PreferenceValue::Number(n) => Some(*n),
            _ => None,
        }
    }
}

impl From<String> for PreferenceValue {
    fn from(s: String) -> Self {
        PreferenceValue::String(s)
    }
}

impl From<bool> for PreferenceValue {
    fn from(b: bool) -> Self {
        PreferenceValue::Bool(b)
    }
}

impl From<f64> for PreferenceValue {
    fn from(n: f64) -> Self {
        PreferenceValue::Number(n)
    }
}

pub type ModulePreferences = HashMap<String, PreferenceValue>;

pub fn load_schema(module_path: &Path) -> Option<PreferencesSchema> {
    let schema_path = module_path.join("preferences.schema.json");
    if !schema_path.exists() {
        return None;
    }

    fs::read_to_string(&schema_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

pub fn load_preferences(uuid: &str) -> ModulePreferences {
    let path = module_preferences_path(uuid);
    if !path.exists() {
        return HashMap::new();
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn save_preferences(uuid: &str, prefs: &ModulePreferences) -> io::Result<()> {
    let path = module_preferences_path(uuid);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(prefs)?;
    fs::write(&path, json)?;

    Ok(())
}

pub fn get_default_preferences(schema: &PreferencesSchema) -> ModulePreferences {
    let mut prefs = HashMap::new();

    for field in &schema.fields {
        match field {
            PreferenceField::Text { key, default, .. } => {
                if let Some(val) = default {
                    prefs.insert(key.clone(), PreferenceValue::String(val.clone()));
                }
            }
            PreferenceField::Boolean { key, default, .. } => {
                if let Some(val) = default {
                    prefs.insert(key.clone(), PreferenceValue::Bool(*val));
                }
            }
            PreferenceField::Select { key, default, .. } => {
                if let Some(val) = default {
                    prefs.insert(key.clone(), PreferenceValue::String(val.clone()));
                }
            }
            PreferenceField::Number { key, default, .. } => {
                if let Some(val) = default {
                    prefs.insert(key.clone(), PreferenceValue::Number(*val));
                }
            }
        }
    }

    prefs
}

pub fn merge_with_defaults(prefs: ModulePreferences, schema: &PreferencesSchema) -> ModulePreferences {
    let mut result = get_default_preferences(schema);
    result.extend(prefs);
    result
}

pub fn delete_preferences(uuid: &str) -> io::Result<()> {
    let path = module_preferences_path(uuid);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn clear_all_preferences() -> io::Result<()> {
    let dir = preferences_dir();
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_value_string() {
        let val = PreferenceValue::String("test".to_string());
        assert_eq!(val.as_string(), Some("test"));
        assert_eq!(val.as_bool(), None);
    }

    #[test]
    fn test_preference_value_bool() {
        let val = PreferenceValue::Bool(true);
        assert_eq!(val.as_bool(), Some(true));
        assert_eq!(val.as_string(), None);
    }

    #[test]
    fn test_preference_value_number() {
        let val = PreferenceValue::Number(42.5);
        assert_eq!(val.as_number(), Some(42.5));
        assert_eq!(val.as_bool(), None);
    }

    #[test]
    fn test_deserialize_schema() {
        let json = r#"{
            "title": "Weather Settings",
            "fields": [
                {
                    "type": "text",
                    "key": "location",
                    "label": "Location",
                    "default": "auto"
                },
                {
                    "type": "boolean",
                    "key": "show_humidity",
                    "label": "Show Humidity",
                    "default": true
                },
                {
                    "type": "select",
                    "key": "units",
                    "label": "Temperature Units",
                    "options": [
                        {"value": "celsius", "label": "Celsius"},
                        {"value": "fahrenheit", "label": "Fahrenheit"}
                    ],
                    "default": "celsius"
                }
            ]
        }"#;

        let schema: PreferencesSchema = serde_json::from_str(json).unwrap();
        assert_eq!(schema.title, Some("Weather Settings".to_string()));
        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.fields[0].key(), "location");
        assert_eq!(schema.fields[1].key(), "show_humidity");
        assert_eq!(schema.fields[2].key(), "units");
    }

    #[test]
    fn test_get_default_preferences() {
        let schema = PreferencesSchema {
            title: None,
            fields: vec![
                PreferenceField::Text {
                    key: "name".to_string(),
                    label: "Name".to_string(),
                    description: None,
                    default: Some("default_name".to_string()),
                    placeholder: None,
                },
                PreferenceField::Boolean {
                    key: "enabled".to_string(),
                    label: "Enabled".to_string(),
                    description: None,
                    default: Some(true),
                },
            ],
        };

        let defaults = get_default_preferences(&schema);
        assert_eq!(defaults.get("name").and_then(|v| v.as_string()), Some("default_name"));
        assert_eq!(defaults.get("enabled").and_then(|v| v.as_bool()), Some(true));
    }
}
