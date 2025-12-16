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

pub fn substitute_preferences(
    content: &str,
    preferences: &std::collections::HashMap<String, crate::services::preferences::PreferenceValue>,
) -> String {
    use crate::services::preferences::PreferenceValue;

    let mut result = content.to_string();
    for (key, value) in preferences {
        let placeholder = format!("$PREF_{}", key);
        let replacement = match value {
            PreferenceValue::String(s) => s.clone(),
            PreferenceValue::Bool(b) => b.to_string(),
            PreferenceValue::Number(n) => {
                if n.fract() == 0.0 {
                    (*n as i64).to_string()
                } else {
                    n.to_string()
                }
            }
        };
        result = result.replace(&placeholder, &replacement);
    }
    result
}

pub fn merge_module_config(
    waybar_content: &str,
    module_content: &str,
    install_path: &str,
) -> Result<String, String> {
    let module_content = module_content.replace("$MODULE_PATH", install_path);

    let waybar_value: serde_json::Value =
        jsonc_parser::parse_to_serde_value(waybar_content, &Default::default())
            .map_err(|e| format!("Failed to parse waybar config: {e}"))?
            .ok_or("Empty waybar config")?;

    let module_value: serde_json::Value =
        jsonc_parser::parse_to_serde_value(&module_content, &Default::default())
            .map_err(|e| format!("Failed to parse module config: {e}"))?
            .ok_or("Empty module config")?;

    let mut waybar_obj = match waybar_value {
        serde_json::Value::Object(obj) => obj,
        _ => return Err("Waybar config is not a JSON object".to_string()),
    };

    let module_obj = match module_value {
        serde_json::Value::Object(obj) => obj,
        _ => return Err("Module config is not a JSON object".to_string()),
    };

    for (key, value) in module_obj {
        waybar_obj.insert(key, value);
    }

    serde_json::to_string_pretty(&serde_json::Value::Object(waybar_obj))
        .map_err(|e| format!("Failed to serialize config: {e}"))
}

pub fn inject_module_css(existing_css: &str, uuid: &str, module_css: &str) -> String {
    format!(
        "{}\n/* BEGIN waybar-manager:{} */\n{}\n/* END waybar-manager:{} */",
        existing_css.trim_end(),
        uuid,
        module_css.trim(),
        uuid
    )
}

pub fn remove_module_css(css_content: &str, uuid: &str) -> String {
    let begin_marker = format!("/* BEGIN waybar-manager:{} */", uuid);
    let end_marker = format!("/* END waybar-manager:{} */", uuid);

    let Some(begin_pos) = css_content.find(&begin_marker) else {
        return css_content.to_string();
    };

    let Some(end_pos) = css_content.find(&end_marker) else {
        return css_content.to_string();
    };

    let before = css_content[..begin_pos].trim_end();
    let after = css_content[end_pos + end_marker.len()..].trim_start();

    if after.is_empty() {
        before.to_string()
    } else {
        format!("{}\n{}", before, after)
    }
}

pub fn remove_module_config(waybar_content: &str, module_name: &str) -> Result<String, String> {
    let waybar_value: serde_json::Value =
        jsonc_parser::parse_to_serde_value(waybar_content, &Default::default())
            .map_err(|e| format!("Failed to parse waybar config: {e}"))?
            .ok_or("Empty waybar config")?;

    let mut waybar_obj = match waybar_value {
        serde_json::Value::Object(obj) => obj,
        _ => return Err("Waybar config is not a JSON object".to_string()),
    };

    waybar_obj.remove(module_name);

    serde_json::to_string_pretty(&serde_json::Value::Object(waybar_obj))
        .map_err(|e| format!("Failed to serialize config: {e}"))
}

pub async fn reload_waybar() -> Result<(), String> {
    let status = tokio::process::Command::new("pkill")
        .args(["-x", "-SIGUSR2", "waybar"])
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

    #[test]
    fn test_merge_module_config_adds_definition() {
        let waybar = r#"{"layer": "top", "modules-center": ["clock"]}"#;
        let module = r#"{"custom/weather": {"exec": "curl wttr.in", "interval": 600}}"#;

        let result = merge_module_config(waybar, module, "/path/to/module").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed["custom/weather"].is_object());
        assert_eq!(parsed["custom/weather"]["interval"], 600);
    }

    #[test]
    fn test_merge_module_config_replaces_module_path() {
        let waybar = r#"{"layer": "top"}"#;
        let module = r#"{"custom/script": {"exec": "$MODULE_PATH/script.sh"}}"#;

        let result = merge_module_config(waybar, module, "/home/user/.local/share/waybar-manager/modules/test@ns").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(
            parsed["custom/script"]["exec"],
            "/home/user/.local/share/waybar-manager/modules/test@ns/script.sh"
        );
    }

    #[test]
    fn test_remove_module_config_strips_definition() {
        let waybar = r#"{"layer": "top", "custom/weather": {"exec": "curl"}, "clock": {}}"#;

        let result = remove_module_config(waybar, "custom/weather").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed.get("custom/weather").is_none());
        assert!(parsed.get("clock").is_some());
        assert!(parsed.get("layer").is_some());
    }

    #[test]
    fn test_inject_module_css_appends_with_markers() {
        let existing = "* { font-family: monospace; }";
        let module_css = ".custom-weather { color: #fff; }";

        let result = inject_module_css(existing, "weather@test", module_css);

        assert!(result.contains("/* BEGIN waybar-manager:weather@test */"));
        assert!(result.contains(".custom-weather { color: #fff; }"));
        assert!(result.contains("/* END waybar-manager:weather@test */"));
        assert!(result.starts_with("* { font-family: monospace; }"));
    }

    #[test]
    fn test_remove_module_css_strips_marked_section() {
        let css = r#"* { font-family: monospace; }
/* BEGIN waybar-manager:weather@test */
.custom-weather { color: #fff; }
/* END waybar-manager:weather@test */
.clock { color: blue; }"#;

        let result = remove_module_css(css, "weather@test");

        assert!(!result.contains("waybar-manager:weather@test"));
        assert!(!result.contains(".custom-weather"));
        assert!(result.contains("* { font-family: monospace; }"));
        assert!(result.contains(".clock { color: blue; }"));
    }

    #[test]
    fn test_remove_module_css_handles_missing_section() {
        let css = "* { font-family: monospace; }";
        let result = remove_module_css(css, "nonexistent@test");
        assert_eq!(result, css);
    }

    #[test]
    fn test_substitute_preferences_replaces_placeholders() {
        use crate::services::preferences::PreferenceValue;
        use std::collections::HashMap;

        let content = r#"{"exec": "echo $PREF_message", "interval": $PREF_interval}"#;
        let mut prefs = HashMap::new();
        prefs.insert("message".to_string(), PreferenceValue::String("Hello".to_string()));
        prefs.insert("interval".to_string(), PreferenceValue::Number(10.0));

        let result = substitute_preferences(content, &prefs);

        assert!(result.contains("echo Hello"));
        assert!(result.contains("\"interval\": 10"));
    }

    #[test]
    fn test_substitute_preferences_handles_bool() {
        use crate::services::preferences::PreferenceValue;
        use std::collections::HashMap;

        let content = r#"{"enabled": $PREF_show}"#;
        let mut prefs = HashMap::new();
        prefs.insert("show".to_string(), PreferenceValue::Bool(true));

        let result = substitute_preferences(content, &prefs);

        assert!(result.contains("true"));
    }
}
