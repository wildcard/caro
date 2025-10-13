//! Sensitive data redaction for logs

use once_cell::sync::Lazy;
use regex::Regex;

static API_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(api[_-]?key|token|secret|password|passwd|bearer[_-]?token|auth[_-]?token|client[_-]?secret|access[_-]?key|secret[_-]?key|AWS_SECRET_ACCESS_KEY|AWS_ACCESS_KEY_ID)[\s:=]+["']?([a-zA-Z0-9_\-\.]+)["']?"#)
        .unwrap()
});

/// Redaction utilities
pub struct Redaction;

impl Redaction {
    /// Redact sensitive data from a string
    pub fn redact(text: &str) -> String {
        API_KEY_PATTERN
            .replace_all(text, "$1=[REDACTED-$1]")
            .to_string()
    }

    /// Check if text contains sensitive data
    pub fn contains_sensitive(text: &str) -> bool {
        API_KEY_PATTERN.is_match(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_api_key() {
        let text = "Using api_key=sk_test_12345 for request";
        let redacted = Redaction::redact(text);
        assert!(!redacted.contains("sk_test_12345"));
        assert!(redacted.contains("[REDACTED"));
    }

    #[test]
    fn test_redact_token() {
        let text = "Using token=abc123token for auth";
        let redacted = Redaction::redact(text);
        assert!(!redacted.contains("abc123token"));
        assert!(redacted.contains("[REDACTED"));
    }

    #[test]
    fn test_contains_sensitive() {
        assert!(Redaction::contains_sensitive("api_key=secret"));
        assert!(!Redaction::contains_sensitive("normal log message"));
    }
}
