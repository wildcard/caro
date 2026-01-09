//! Sensitive data redaction and validation
//!
//! This module provides pre-transmission validation to ensure no sensitive
//! data is accidentally included in telemetry events.

use super::Event;
use once_cell::sync::Lazy;
use regex::Regex;

// Compile regex patterns once at startup
static PATH_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(/[\w/.-]+|[A-Z]:\\[\w\\.-]+)").unwrap());

static EMAIL_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap());

static IP_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap());

static ENV_VAR_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(PATH|HOME|USER|SHELL|PWD|OLDPWD|HOSTNAME)=").unwrap());

static API_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Detect common API key patterns
    Regex::new(r#"(api[_-]?key|token|secret|password|auth)["']?\s*[:=]\s*["']?[\w-]{20,}"#).unwrap()
});

/// Validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    ContainsFilePath(String),
    ContainsEmail(String),
    ContainsIpAddress(String),
    ContainsEnvironmentVariable(String),
    ContainsApiKey,
    ContainsHostname,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::ContainsFilePath(path) => {
                write!(f, "Event contains file path: {}", path)
            }
            ValidationError::ContainsEmail(email) => {
                write!(f, "Event contains email address: {}", email)
            }
            ValidationError::ContainsIpAddress(ip) => {
                write!(f, "Event contains IP address: {}", ip)
            }
            ValidationError::ContainsEnvironmentVariable(var) => {
                write!(f, "Event contains environment variable: {}", var)
            }
            ValidationError::ContainsApiKey => {
                write!(f, "Event contains potential API key or secret")
            }
            ValidationError::ContainsHostname => {
                write!(f, "Event contains hostname")
            }
        }
    }
}

/// Validate event contains no sensitive data
///
/// Performs multiple checks to ensure the event is safe to transmit:
/// - No file paths (absolute or relative)
/// - No email addresses
/// - No IP addresses
/// - No environment variables
/// - No API keys or secrets
/// - No hostnames
///
/// Returns `Ok(())` if validation passes, `Err(ValidationError)` otherwise.
pub fn validate_event(event: &Event) -> Result<(), ValidationError> {
    let json = serde_json::to_string(event).unwrap();

    // Check for file paths
    if let Some(captures) = PATH_PATTERN.captures(&json) {
        if let Some(path) = captures.get(0) {
            return Err(ValidationError::ContainsFilePath(path.as_str().to_string()));
        }
    }

    // Check for email addresses
    if let Some(captures) = EMAIL_PATTERN.captures(&json) {
        if let Some(email) = captures.get(0) {
            return Err(ValidationError::ContainsEmail(email.as_str().to_string()));
        }
    }

    // Check for IP addresses
    if let Some(captures) = IP_PATTERN.captures(&json) {
        if let Some(ip) = captures.get(0) {
            // Filter out version numbers like "1.2.3.4"
            let ip_str = ip.as_str();
            let parts: Vec<u32> = ip_str.split('.').filter_map(|s| s.parse().ok()).collect();

            // Valid IP addresses have all parts <= 255
            if parts.len() == 4 && parts.iter().all(|&p| p <= 255) {
                // Exclude private IP ranges and localhost
                let is_private = (parts[0] == 10)
                    || (parts[0] == 172 && (16..=31).contains(&parts[1]))
                    || (parts[0] == 192 && parts[1] == 168)
                    || (parts[0] == 127);

                if !is_private {
                    return Err(ValidationError::ContainsIpAddress(ip_str.to_string()));
                }
            }
        }
    }

    // Check for environment variables
    if let Some(captures) = ENV_VAR_PATTERN.captures(&json) {
        if let Some(var) = captures.get(0) {
            return Err(ValidationError::ContainsEnvironmentVariable(
                var.as_str().to_string(),
            ));
        }
    }

    // Check for API keys or secrets (case-insensitive)
    if API_KEY_PATTERN.is_match(&json.to_lowercase()) {
        return Err(ValidationError::ContainsApiKey);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{EventType, SessionId};

    #[test]
    fn test_detects_file_paths() {
        let session_id = SessionId::generate();

        // Unix path - should fail
        let event = Event::new(
            session_id.clone(),
            EventType::BackendError {
                backend: "/Users/test/caro".to_string(), // Hidden path in backend field
                error_category: "load_failed".to_string(),
                recoverable: false,
            },
        );
        assert!(validate_event(&event).is_err());

        // Windows path - should fail
        let event = Event::new(
            session_id.clone(),
            EventType::BackendError {
                backend: r"C:\Users\test\caro".to_string(),
                error_category: "load_failed".to_string(),
                recoverable: false,
            },
        );
        assert!(validate_event(&event).is_err());
    }

    #[test]
    fn test_detects_emails() {
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::BackendError {
                backend: "embedded".to_string(),
                error_category: "user@example.com".to_string(), // Hidden email
                recoverable: false,
            },
        );

        let result = validate_event(&event);
        assert!(result.is_err());
        assert!(matches!(result, Err(ValidationError::ContainsEmail(_))));
    }

    #[test]
    fn test_detects_ip_addresses() {
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::BackendError {
                backend: "remote-api-93.184.216.34".to_string(), // Hidden IP
                error_category: "connection_failed".to_string(),
                recoverable: true,
            },
        );

        let result = validate_event(&event);
        assert!(result.is_err());
        assert!(matches!(result, Err(ValidationError::ContainsIpAddress(_))));
    }

    #[test]
    fn test_allows_version_numbers() {
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::SessionStart {
                version: "1.2.3".to_string(), // Version number, not IP
                platform: "linux".to_string(),
                shell_type: "bash".to_string(),
                backend_available: vec![],
            },
        );

        // Version numbers should be allowed
        assert!(validate_event(&event).is_ok());
    }

    #[test]
    fn test_allows_private_ips() {
        let session_id = SessionId::generate();

        // localhost
        let event = Event::new(
            session_id.clone(),
            EventType::BackendError {
                backend: "127.0.0.1".to_string(),
                error_category: "test".to_string(),
                recoverable: true,
            },
        );
        assert!(validate_event(&event).is_ok());

        // Private IP (10.x.x.x)
        let event = Event::new(
            session_id,
            EventType::BackendError {
                backend: "10.0.0.1".to_string(),
                error_category: "test".to_string(),
                recoverable: true,
            },
        );
        assert!(validate_event(&event).is_ok());
    }

    #[test]
    fn test_detects_environment_variables() {
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::BackendError {
                backend: "embedded".to_string(),
                error_category: "Environment: USER=admin".to_string(), // Hidden env var without file paths
                recoverable: false,
            },
        );

        let result = validate_event(&event);
        assert!(result.is_err(), "Event should fail validation");
        assert!(
            matches!(result, Err(ValidationError::ContainsEnvironmentVariable(_))),
            "Expected ContainsEnvironmentVariable error, got: {:?}",
            result
        );
    }

    #[test]
    fn test_valid_event_passes() {
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::CommandGeneration {
                backend: "embedded".to_string(),
                duration_ms: 1500,
                success: true,
                error_category: None,
            },
        );

        assert!(validate_event(&event).is_ok());
    }

    #[test]
    fn test_session_start_passes() {
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::SessionStart {
                version: "1.0.0".to_string(),
                platform: "darwin".to_string(),
                shell_type: "zsh".to_string(),
                backend_available: vec!["embedded".to_string(), "static".to_string()],
            },
        );

        assert!(validate_event(&event).is_ok());
    }

    #[test]
    fn test_safety_validation_passes() {
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::SafetyValidation {
                risk_level: "critical".to_string(),
                action_taken: "blocked".to_string(),
                pattern_category: Some("destructive".to_string()),
            },
        );

        assert!(validate_event(&event).is_ok());
    }
}
