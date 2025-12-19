use crate::services::paths::SECURITY_CHECK_URL;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RevocationError {
    #[error("Module {uuid} version {version} has been revoked: {reason}")]
    Revoked {
        uuid: String,
        version: String,
        reason: String,
    },

    #[error("Network error during revocation check: {0}")]
    NetworkError(String),

    #[error("Invalid response from security server: {0}")]
    InvalidResponse(String),

    #[error("Security check timed out")]
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OfflinePolicy {
    AllowOffline,
    #[default]
    FailClosed,
}

#[derive(Debug, serde::Deserialize)]
struct SecurityCheckResponse {
    revoked: bool,
    reason: Option<String>,
}

pub async fn check_revocation(
    uuid: &str,
    version: &str,
    policy: OfflinePolicy,
) -> Result<(), RevocationError> {
    let encoded_uuid = urlencoding::encode(uuid);
    let encoded_version = urlencoding::encode(version);
    let url = format!("{SECURITY_CHECK_URL}?uuid={encoded_uuid}&version={encoded_version}");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| RevocationError::NetworkError(e.to_string()))?;

    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) if e.is_timeout() => return Err(RevocationError::Timeout),
        Err(e) if e.is_connect() => {
            return match policy {
                OfflinePolicy::AllowOffline => {
                    tracing::warn!(
                        "Revocation check skipped (offline mode): {} v{}",
                        uuid,
                        version
                    );
                    Ok(())
                }
                OfflinePolicy::FailClosed => Err(RevocationError::NetworkError(format!(
                    "Cannot verify module safety (network unavailable): {e}"
                ))),
            };
        }
        Err(e) => return Err(RevocationError::NetworkError(e.to_string())),
    };

    if !response.status().is_success() {
        return Err(RevocationError::InvalidResponse(format!(
            "Server returned status {}",
            response.status()
        )));
    }

    let check: SecurityCheckResponse = response
        .json()
        .await
        .map_err(|e| RevocationError::InvalidResponse(e.to_string()))?;

    if check.revoked {
        return Err(RevocationError::Revoked {
            uuid: uuid.to_string(),
            version: version.to_string(),
            reason: check
                .reason
                .unwrap_or_else(|| "No reason provided".to_string()),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_policy_default_is_fail_closed() {
        assert_eq!(OfflinePolicy::default(), OfflinePolicy::FailClosed);
    }

    #[test]
    fn revocation_error_display() {
        let err = RevocationError::Revoked {
            uuid: "test-uuid".to_string(),
            version: "1.0.0".to_string(),
            reason: "Security vulnerability".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("test-uuid"));
        assert!(msg.contains("1.0.0"));
        assert!(msg.contains("Security vulnerability"));
    }

    #[test]
    fn url_encoding_handles_special_chars() {
        let uuid = "test/uuid+special";
        let encoded = urlencoding::encode(uuid);
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('+'));
    }
}
