use thiserror::Error;
use url::Url;

#[derive(Debug, Error, PartialEq)]
pub enum UrlValidationError {
    #[error("invalid URL format: {0}")]
    InvalidFormat(String),
    #[error("URL scheme not allowed: {0}")]
    DisallowedScheme(String),
    #[error("missing URL scheme")]
    MissingScheme,
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
}
