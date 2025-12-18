use std::path::Path;

use crate::services::paths;

pub async fn handle_css_injection(uuid: &str, install_path: &Path) {
    use crate::services::waybar_config;

    let css_path = install_path.join("style.css");
    if !css_path.exists() {
        return;
    }

    let Ok(module_css) = tokio::fs::read_to_string(&css_path).await else {
        return;
    };

    let waybar_style_path = paths::waybar_style_path();
    let existing_css = tokio::fs::read_to_string(&waybar_style_path)
        .await
        .unwrap_or_default();

    let new_css = waybar_config::inject_module_css(&existing_css, uuid, &module_css);

    if let Err(e) = tokio::fs::write(&waybar_style_path, new_css).await {
        tracing::warn!("Failed to inject CSS: {e}");
    }
}

pub async fn handle_css_removal(uuid: &str) {
    use crate::services::waybar_config;

    let waybar_style_path = paths::waybar_style_path();
    let Ok(existing_css) = tokio::fs::read_to_string(&waybar_style_path).await else {
        return;
    };

    let new_css = waybar_config::remove_module_css(&existing_css, uuid);

    if let Err(e) = tokio::fs::write(&waybar_style_path, new_css).await {
        tracing::warn!("Failed to remove CSS: {e}");
    }
}
