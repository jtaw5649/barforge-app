use thiserror::Error;
use url::Url;

#[derive(Debug, Error, PartialEq)]
pub enum UrlValidationError {
    #[error("invalid URL format: {0}")]
    InvalidFormat(String),
    #[error("URL scheme not allowed: {0}")]
    DisallowedScheme(String),
    #[error("domain not allowed: {0}")]
    DisallowedDomain(String),
    #[error("missing URL scheme")]
    MissingScheme,
    #[error("invalid GitHub URL path: {0}")]
    InvalidGitHubPath(String),
}

pub fn validate_web_url(url_str: &str) -> Result<(), UrlValidationError> {
    let parsed =
        Url::parse(url_str).map_err(|e| UrlValidationError::InvalidFormat(e.to_string()))?;

    let scheme = parsed.scheme().to_lowercase();
    match scheme.as_str() {
        "https" | "http" => Ok(()),
        other => Err(UrlValidationError::DisallowedScheme(other.to_string())),
    }
}

pub fn validate_github_url(url_str: &str) -> Result<Url, UrlValidationError> {
    let parsed =
        Url::parse(url_str).map_err(|e| UrlValidationError::InvalidFormat(e.to_string()))?;

    let scheme = parsed.scheme().to_lowercase();
    if scheme != "https" {
        return Err(UrlValidationError::DisallowedScheme(scheme));
    }

    let host = parsed.host_str().unwrap_or("");
    if host != "github.com" {
        return Err(UrlValidationError::DisallowedDomain(host.to_string()));
    }

    Ok(parsed)
}

pub fn parse_github_url_safe(url_str: &str) -> Result<(String, String), UrlValidationError> {
    let parsed = validate_github_url(url_str)?;

    let path_segments: Vec<&str> = parsed
        .path_segments()
        .map(|segments| segments.filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    if path_segments.len() < 2 {
        return Err(UrlValidationError::InvalidGitHubPath(
            "URL must contain owner and repository".to_string(),
        ));
    }

    let owner = path_segments[0].to_string();
    let repo = path_segments[1].to_string();

    Ok((owner, repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https_url() {
        let result = validate_web_url("https://github.com/user/repo");
        assert!(result.is_ok());
    }

    #[test]
    fn accepts_http_url() {
        let result = validate_web_url("http://example.com/page");
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_javascript_scheme() {
        let result = validate_web_url("javascript:alert('xss')");
        assert!(matches!(result, Err(UrlValidationError::DisallowedScheme(_))));
    }

    #[test]
    fn rejects_file_scheme() {
        let result = validate_web_url("file:///etc/passwd");
        assert!(matches!(result, Err(UrlValidationError::DisallowedScheme(_))));
    }

    #[test]
    fn rejects_data_scheme() {
        let result = validate_web_url("data:text/html,<script>alert('xss')</script>");
        assert!(matches!(result, Err(UrlValidationError::DisallowedScheme(_))));
    }

    #[test]
    fn rejects_malformed_url() {
        let result = validate_web_url("not a valid url");
        assert!(matches!(result, Err(UrlValidationError::InvalidFormat(_))));
    }

    #[test]
    fn accepts_url_with_port() {
        let result = validate_web_url("https://localhost:8080/api");
        assert!(result.is_ok());
    }

    #[test]
    fn accepts_url_with_query_params() {
        let result = validate_web_url("https://github.com/search?q=rust");
        assert!(result.is_ok());
    }

    #[test]
    fn github_rejects_non_github_domain() {
        let result = validate_github_url("https://evil.com/user/repo");
        assert!(matches!(result, Err(UrlValidationError::DisallowedDomain(_))));
    }

    #[test]
    fn github_accepts_valid_https() {
        let result = validate_github_url("https://github.com/waybar-modules/weather");
        assert!(result.is_ok());
    }

    #[test]
    fn github_rejects_http_scheme() {
        let result = validate_github_url("http://github.com/user/repo");
        assert!(matches!(result, Err(UrlValidationError::DisallowedScheme(_))));
    }

    #[test]
    fn github_rejects_api_subdomain() {
        let result = validate_github_url("https://api.github.com/user/repo");
        assert!(matches!(result, Err(UrlValidationError::DisallowedDomain(_))));
    }

    #[test]
    fn github_rejects_raw_subdomain() {
        let result = validate_github_url("https://raw.githubusercontent.com/user/repo/file");
        assert!(matches!(result, Err(UrlValidationError::DisallowedDomain(_))));
    }

    #[test]
    fn parse_github_extracts_owner_repo() {
        let result = parse_github_url_safe("https://github.com/waybar-modules/weather");
        assert_eq!(result, Ok(("waybar-modules".to_string(), "weather".to_string())));
    }

    #[test]
    fn parse_github_handles_trailing_slash() {
        let result = parse_github_url_safe("https://github.com/owner/repo/");
        assert_eq!(result, Ok(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn parse_github_handles_extra_path_segments() {
        let result = parse_github_url_safe("https://github.com/owner/repo/tree/main");
        assert_eq!(result, Ok(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn parse_github_rejects_missing_repo() {
        let result = parse_github_url_safe("https://github.com/owner");
        assert!(matches!(result, Err(UrlValidationError::InvalidGitHubPath(_))));
    }

    #[test]
    fn parse_github_rejects_root_path() {
        let result = parse_github_url_safe("https://github.com/");
        assert!(matches!(result, Err(UrlValidationError::InvalidGitHubPath(_))));
    }
}
