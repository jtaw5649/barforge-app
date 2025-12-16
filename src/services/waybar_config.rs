use std::path::PathBuf;

use crate::domain::BarSection;
use crate::services::paths;

pub async fn load_config() -> Result<String, String> {
    let path = paths::waybar_config_path();

    if !path.exists() {
        return Err(format!("Waybar config not found at {}", path.display()));
    }

    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read waybar config: {e}"))
}

pub async fn save_config(content: &str) -> Result<(), String> {
    let path = paths::waybar_config_path();

    tokio::fs::write(&path, content)
        .await
        .map_err(|e| format!("Failed to write waybar config: {e}"))
}

pub async fn backup_config() -> Result<PathBuf, String> {
    let path = paths::waybar_config_path();

    if !path.exists() {
        return Err("No config to backup".to_string());
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("config.jsonc.{}.backup", timestamp);
    let backup_path = path.with_file_name(backup_name);

    tokio::fs::copy(&path, &backup_path)
        .await
        .map_err(|e| format!("Failed to create backup: {e}"))?;

    tracing::info!("Created waybar config backup at {}", backup_path.display());

    Ok(backup_path)
}

pub fn add_module(content: &str, module_name: &str, section: BarSection) -> Result<String, String> {
    let array_key = section.array_key();

    let value: serde_json::Value = jsonc_parser::parse_to_serde_value(content, &Default::default())
        .map_err(|e| format!("Failed to parse waybar config: {e}"))?
        .ok_or("Empty waybar config")?;

    let mut obj = match value {
        serde_json::Value::Object(obj) => obj,
        _ => return Err("Waybar config is not a JSON object".to_string()),
    };

    let modules_array = obj
        .entry(array_key)
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));

    let arr = modules_array
        .as_array_mut()
        .ok_or_else(|| format!("{} is not an array", array_key))?;

    let module_value = serde_json::Value::String(module_name.to_string());
    if !arr.contains(&module_value) {
        arr.push(module_value);
        tracing::info!("Added {} to {}", module_name, array_key);
    } else {
        tracing::info!("{} already in {}", module_name, array_key);
    }

    serde_json::to_string_pretty(&serde_json::Value::Object(obj))
        .map_err(|e| format!("Failed to serialize config: {e}"))
}

pub fn remove_module(content: &str, module_name: &str) -> Result<String, String> {
    let value: serde_json::Value = jsonc_parser::parse_to_serde_value(content, &Default::default())
        .map_err(|e| format!("Failed to parse waybar config: {e}"))?
        .ok_or("Empty waybar config")?;

    let mut obj = match value {
        serde_json::Value::Object(obj) => obj,
        _ => return Err("Waybar config is not a JSON object".to_string()),
    };

    for array_key in ["modules-left", "modules-center", "modules-right"] {
        if let Some(modules) = obj.get_mut(array_key)
            && let Some(arr) = modules.as_array_mut()
        {
            let original_len = arr.len();
            arr.retain(|v| v.as_str() != Some(module_name));

            if arr.len() < original_len {
                tracing::info!("Removed {} from {}", module_name, array_key);
            }
        }
    }

    serde_json::to_string_pretty(&serde_json::Value::Object(obj))
        .map_err(|e| format!("Failed to serialize config: {e}"))
}

pub async fn reload_waybar() -> Result<(), String> {
    let status = tokio::process::Command::new("pkill")
        .args(["-SIGUSR2", "waybar"])
        .status()
        .await
        .map_err(|e| format!("Failed to send reload signal: {e}"))?;

    if status.success() || status.code() == Some(1) {
        tracing::info!("Sent reload signal to waybar");
        Ok(())
    } else {
        Err(format!("pkill failed with status: {}", status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CONFIG: &str = r#"{
    "layer": "top",
    "position": "top",
    "modules-left": ["sway/workspaces"],
    "modules-center": ["clock"],
    "modules-right": ["battery", "network"]
}"#;

    #[test]
    fn test_add_module_to_left() {
        let result = add_module(SAMPLE_CONFIG, "custom/weather", BarSection::Left).unwrap();
        assert!(result.contains("custom/weather"));

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let left = parsed["modules-left"].as_array().unwrap();
        assert!(left.iter().any(|v| v == "custom/weather"));
    }

    #[test]
    fn test_add_module_to_center() {
        let result = add_module(SAMPLE_CONFIG, "custom/music", BarSection::Center).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let center = parsed["modules-center"].as_array().unwrap();
        assert!(center.iter().any(|v| v == "custom/music"));
    }

    #[test]
    fn test_add_module_to_right() {
        let result = add_module(SAMPLE_CONFIG, "custom/cpu", BarSection::Right).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let right = parsed["modules-right"].as_array().unwrap();
        assert!(right.iter().any(|v| v == "custom/cpu"));
    }

    #[test]
    fn test_add_module_idempotent() {
        let result1 = add_module(SAMPLE_CONFIG, "clock", BarSection::Center).unwrap();
        let result2 = add_module(&result1, "clock", BarSection::Center).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result2).unwrap();
        let center = parsed["modules-center"].as_array().unwrap();
        let clock_count = center.iter().filter(|v| v.as_str() == Some("clock")).count();
        assert_eq!(clock_count, 1);
    }

    #[test]
    fn test_remove_module() {
        let result = remove_module(SAMPLE_CONFIG, "clock").unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let center = parsed["modules-center"].as_array().unwrap();
        assert!(!center.iter().any(|v| v == "clock"));
    }

    #[test]
    fn test_remove_module_from_any_section() {
        let result = remove_module(SAMPLE_CONFIG, "battery").unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let right = parsed["modules-right"].as_array().unwrap();
        assert!(!right.iter().any(|v| v == "battery"));
    }

    #[test]
    fn test_remove_nonexistent_module() {
        let result = remove_module(SAMPLE_CONFIG, "nonexistent").unwrap();
        let original: serde_json::Value = serde_json::from_str(SAMPLE_CONFIG).unwrap();
        let new: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(
            original["modules-left"].as_array().unwrap().len(),
            new["modules-left"].as_array().unwrap().len()
        );
    }

    #[test]
    fn test_add_module_creates_missing_array() {
        let config = r#"{"layer": "top"}"#;
        let result = add_module(config, "custom/test", BarSection::Left).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["modules-left"].is_array());
    }
}
