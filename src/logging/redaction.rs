//! Sensitive data redaction for logs

use once_cell::sync::Lazy;
use regex::Regex;

static API_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(api[_-]?key|token|secret|password|passwd|bearer[_-]?token|auth[_-]?token|client[_-]?secret|access[_-]?key|secret[_-]?key|AWS_SECRET_ACCESS_KEY|AWS_ACCESS_KEY_ID)[\s:=]+["']?([a-zA-Z0-9_\-\.]+)["']?"#)
        .unwrap()
});

/// Redaction utilities
///
/// Automatically redacts sensitive data like API keys, tokens, passwords, and secrets
/// from log messages to prevent credential leakage.
pub struct Redaction;

impl Redaction {
    /// Redact sensitive data from a string
    ///
    /// Identifies and redacts common patterns for sensitive data including:
    /// - API keys
    /// - Tokens (bearer, auth, client)
    /// - Passwords and secrets
    /// - AWS credentials
    ///
    /// # Example
    ///
    /// ```
    /// use caro::logging::Redaction;
    ///
    /// let log_message = "Authenticating with api_key=sk_live_abc123 for user request";
    /// let safe_message = Redaction::redact(log_message);
    ///
    /// assert!(safe_message.contains("[REDACTED"));
    /// assert!(!safe_message.contains("sk_live_abc123"));
    ///
    /// println!("Safe to log: {}", safe_message);
    /// // Output: "Authenticating with api_key=[REDACTED-api_key] for user request"
    /// ```
    ///
    /// # Example with Multiple Secrets
    ///
    /// ```
    /// use caro::logging::Redaction;
    ///
    /// let text = "Config: api_key=secret123 and password=hunter2";
    /// let redacted = Redaction::redact(text);
    ///
    /// // Both secrets are redacted
    /// assert!(!redacted.contains("secret123"));
    /// assert!(!redacted.contains("hunter2"));
    /// assert!(redacted.contains("[REDACTED-api_key]"));
    /// assert!(redacted.contains("[REDACTED-password]"));
    /// ```
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
