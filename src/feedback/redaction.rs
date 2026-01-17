//! Sensitive Data Redaction
//!
//! This module provides functionality to redact sensitive data from feedback
//! before it is submitted. This includes:
//!
//! - API keys and tokens (OpenAI, Anthropic, etc.)
//! - Bearer tokens and authorization headers
//! - Home directory paths (replaced with $HOME)
//! - Password and credential patterns
//! - Private keys (SSH, PGP, etc.)
//! - Environment variable values that contain secrets

use once_cell::sync::Lazy;
use regex::Regex;

use crate::feedback::types::FeedbackContext;

// =============================================================================
// Regex Patterns for Sensitive Data
// =============================================================================

/// Pattern for API keys with common prefixes
static API_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Note: hyphen at end of character class to avoid escaping issues
    Regex::new(r"(?i)(api[_-]?key|token|secret|password|credential)[=:]\s*'?([a-zA-Z0-9_.=-]{16,})'?").unwrap()
});

/// Pattern for Bearer tokens in authorization headers
static BEARER_TOKEN_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)bearer\s+([a-zA-Z0-9_.-]{20,})").unwrap()
});

/// Pattern for home directory paths (macOS and Linux)
static HOME_DIR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(/Users/[^/\s]+)|(/home/[^/\s]+)").unwrap()
});

/// Pattern for OpenAI API keys (sk-...)
static OPENAI_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap()
});

/// Pattern for Anthropic API keys (sk-ant-...)
static ANTHROPIC_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"sk-ant-[a-zA-Z0-9-]{20,}").unwrap()
});

/// Pattern for GitHub tokens (ghp_, gho_, ghu_, ghs_, ghr_)
static GITHUB_TOKEN_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(ghp_|gho_|ghu_|ghs_|ghr_)[a-zA-Z0-9]{36,}").unwrap()
});

/// Pattern for generic environment variable assignments with sensitive names
static ENV_VAR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(OPENAI_API_KEY|ANTHROPIC_API_KEY|GITHUB_TOKEN|AWS_SECRET_ACCESS_KEY|DATABASE_URL|DB_PASSWORD|PRIVATE_KEY)=([^\s]+)").unwrap()
});

/// Pattern for Basic auth credentials
static BASIC_AUTH_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)basic\s+([a-zA-Z0-9+/=]{20,})").unwrap()
});

/// Pattern for password fields in JSON/YAML
static PASSWORD_FIELD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)['"]?(password|passwd|pwd|secret)['"]?\s*[=:]\s*['"]?([^'"\s,}]+)['"]?"#)
        .unwrap()
});

/// Pattern for SSH private key markers
static SSH_PRIVATE_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"-----BEGIN (RSA |OPENSSH |DSA |EC |ED25519 )?PRIVATE KEY-----").unwrap()
});

/// Pattern for AWS credentials
static AWS_CREDS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(aws_access_key_id|aws_secret_access_key)\s*=\s*([A-Za-z0-9/+=]+)").unwrap()
});

/// Redaction placeholder
const REDACTED: &str = "[REDACTED]";

// =============================================================================
// Public API
// =============================================================================

/// Redact sensitive data from a text string
///
/// This function identifies and redacts various forms of sensitive data including:
/// - API keys and tokens
/// - Authorization headers
/// - Home directory paths
/// - Passwords and credentials
/// - Private key markers
///
/// # Arguments
/// * `text` - The text to redact sensitive data from
///
/// # Returns
/// A new string with sensitive data replaced with `[REDACTED]`
///
/// # Examples
///
/// ```
/// use caro::feedback::redaction::redact_sensitive_data;
///
/// let text = "My API key is sk-1234567890abcdefghij";
/// let redacted = redact_sensitive_data(text);
/// assert!(redacted.contains("[REDACTED]"));
/// assert!(!redacted.contains("sk-1234567890abcdefghij"));
/// ```
pub fn redact_sensitive_data(text: &str) -> String {
    let mut result = text.to_string();

    // Redact specific API key patterns first (more specific)
    result = OPENAI_KEY_PATTERN.replace_all(&result, REDACTED).to_string();
    result = ANTHROPIC_KEY_PATTERN.replace_all(&result, REDACTED).to_string();
    result = GITHUB_TOKEN_PATTERN.replace_all(&result, REDACTED).to_string();

    // Redact generic API keys and tokens
    result = API_KEY_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{}={}", &caps[1], REDACTED)
        })
        .to_string();

    // Redact bearer tokens
    result = BEARER_TOKEN_PATTERN
        .replace_all(&result, format!("Bearer {}", REDACTED))
        .to_string();

    // Redact basic auth
    result = BASIC_AUTH_PATTERN
        .replace_all(&result, format!("Basic {}", REDACTED))
        .to_string();

    // Redact environment variables with sensitive names
    result = ENV_VAR_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{}={}", &caps[1], REDACTED)
        })
        .to_string();

    // Redact password fields
    result = PASSWORD_FIELD_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{}={}", &caps[1], REDACTED)
        })
        .to_string();

    // Redact AWS credentials
    result = AWS_CREDS_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{}={}", &caps[1], REDACTED)
        })
        .to_string();

    // Replace home directories with $HOME
    // Note: We use "$$HOME" because $ has special meaning in regex replacements
    result = HOME_DIR_PATTERN.replace_all(&result, "$$HOME").to_string();

    // Mark private key blocks (don't remove the marker, just note it's redacted)
    if SSH_PRIVATE_KEY_PATTERN.is_match(&result) {
        result = SSH_PRIVATE_KEY_PATTERN
            .replace_all(&result, "[PRIVATE KEY REDACTED]")
            .to_string();
    }

    result
}

/// Redact sensitive data from a FeedbackContext
///
/// This function processes all text fields in the context that might contain
/// sensitive information and redacts them in place.
///
/// # Arguments
/// * `context` - Mutable reference to the feedback context to redact
pub fn redact_context(context: &mut FeedbackContext) {
    // Redact error information
    if let Some(ref mut error) = context.error_info {
        error.stderr = redact_sensitive_data(&error.stderr);
        error.stdout = redact_sensitive_data(&error.stdout);
        error.error_message = redact_sensitive_data(&error.error_message);
    }

    // Redact command history
    for entry in &mut context.command_info.command_history {
        entry.command = redact_sensitive_data(&entry.command);
        entry.prompt = redact_sensitive_data(&entry.prompt);
    }

    // Redact generated command and prompt
    context.command_info.generated_command =
        redact_sensitive_data(&context.command_info.generated_command);
    context.command_info.user_prompt = redact_sensitive_data(&context.command_info.user_prompt);

    // Redact paths in system state
    context.system_state.cache_dir = redact_path(&context.system_state.cache_dir);
    if let Some(ref mut config_path) = context.system_state.config_file {
        *config_path = redact_path(config_path);
    }
}

/// Redact home directory from a path
fn redact_path(path: &std::path::Path) -> std::path::PathBuf {
    let path_str = path.to_string_lossy();
    // Note: We use "$$HOME" because $ has special meaning in regex replacements
    let redacted = HOME_DIR_PATTERN.replace_all(&path_str, "$$HOME");
    std::path::PathBuf::from(redacted.to_string())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::types::*;
    use chrono::Utc;
    use std::path::PathBuf;

    // =========================================================================
    // API Key Redaction Tests
    // =========================================================================

    #[test]
    fn test_redact_openai_api_keys() {
        let text = "My key is sk-1234567890abcdefghijklmnop";
        let redacted = redact_sensitive_data(text);
        assert!(!redacted.contains("sk-1234567890abcdefghijklmnop"));
        assert!(redacted.contains(REDACTED));
    }

    #[test]
    fn test_redact_anthropic_api_keys() {
        let text = "export ANTHROPIC_API_KEY=sk-ant-api03-abcdefghijklmnopqrstuvwxyz";
        let redacted = redact_sensitive_data(text);
        assert!(!redacted.contains("sk-ant-api03-abcdefghijklmnopqrstuvwxyz"));
        assert!(redacted.contains(REDACTED));
    }

    #[test]
    fn test_redact_github_tokens() {
        let texts = vec![
            "GITHUB_TOKEN=ghp_1234567890abcdefghijklmnopqrstuvwxyz12",
            "token: gho_1234567890abcdefghijklmnopqrstuvwxyz12",
        ];

        for text in texts {
            let redacted = redact_sensitive_data(text);
            assert!(
                !redacted.contains("ghp_") && !redacted.contains("gho_"),
                "Should redact GitHub token in: {}",
                text
            );
            assert!(redacted.contains(REDACTED));
        }
    }

    #[test]
    fn test_redact_generic_api_keys() {
        let texts = vec![
            "api_key=abcdef1234567890abcdef",
            "API-KEY: secret_key_value_123456",
            "token=my_super_secret_token_value",
        ];

        for text in texts {
            let redacted = redact_sensitive_data(text);
            assert!(
                redacted.contains(REDACTED),
                "Should redact API key in: {}",
                text
            );
        }
    }

    // =========================================================================
    // Bearer Token Redaction Tests
    // =========================================================================

    #[test]
    fn test_redact_bearer_tokens() {
        let text = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let redacted = redact_sensitive_data(text);
        assert!(!redacted.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert!(redacted.contains(format!("Bearer {}", REDACTED).as_str()));
    }

    #[test]
    fn test_redact_basic_auth() {
        let text = "Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=";
        let redacted = redact_sensitive_data(text);
        assert!(!redacted.contains("dXNlcm5hbWU6cGFzc3dvcmQ="));
        assert!(redacted.contains(REDACTED));
    }

    // =========================================================================
    // Home Directory Redaction Tests
    // =========================================================================

    #[test]
    fn test_redact_macos_home_directory() {
        let text = "/Users/johndoe/secret/file.txt";
        let redacted = redact_sensitive_data(text);
        assert!(redacted.contains("$HOME"));
        assert!(!redacted.contains("/Users/johndoe"));
        assert!(redacted.contains("/secret/file.txt"));
    }

    #[test]
    fn test_redact_linux_home_directory() {
        let text = "/home/username/documents/passwords.txt";
        let redacted = redact_sensitive_data(text);
        assert!(redacted.contains("$HOME"));
        assert!(!redacted.contains("/home/username"));
    }

    #[test]
    fn test_redact_multiple_home_directories() {
        let text = "Source: /Users/alice/src, Dest: /home/bob/backup";
        let redacted = redact_sensitive_data(text);
        assert!(!redacted.contains("/Users/alice"));
        assert!(!redacted.contains("/home/bob"));
        assert_eq!(redacted.matches("$HOME").count(), 2);
    }

    // =========================================================================
    // Environment Variable Redaction Tests
    // =========================================================================

    #[test]
    fn test_redact_openai_env_var() {
        let text = "OPENAI_API_KEY=sk-12345678901234567890";
        let redacted = redact_sensitive_data(text);
        assert!(redacted.contains("OPENAI_API_KEY="));
        assert!(redacted.contains(REDACTED));
        assert!(!redacted.contains("sk-12345678901234567890"));
    }

    #[test]
    fn test_redact_aws_credentials() {
        let text = "aws_secret_access_key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let redacted = redact_sensitive_data(text);
        assert!(!redacted.contains("wJalrXUtnFEMI"));
        assert!(redacted.contains(REDACTED));
    }

    // =========================================================================
    // Password Field Redaction Tests
    // =========================================================================

    #[test]
    fn test_redact_password_in_json() {
        let json = r#"{"username": "test", "password": "secret123", "email": "test@test.com"}"#;
        let redacted = redact_sensitive_data(json);
        assert!(!redacted.contains("secret123"));
        assert!(redacted.contains(REDACTED));
        // Non-sensitive fields should be preserved
        assert!(redacted.contains("test@test.com"));
    }

    #[test]
    fn test_redact_password_variations() {
        let texts = vec![
            "password=mysecret",
            "passwd: secretvalue",
            "'pwd': 'hidden'",
        ];

        for text in texts {
            let redacted = redact_sensitive_data(text);
            assert!(
                redacted.contains(REDACTED),
                "Should redact password in: {}",
                text
            );
        }
    }

    // =========================================================================
    // Private Key Redaction Tests
    // =========================================================================

    #[test]
    fn test_redact_ssh_private_key() {
        let text = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...";
        let redacted = redact_sensitive_data(text);
        assert!(redacted.contains("[PRIVATE KEY REDACTED]"));
        assert!(!redacted.contains("-----BEGIN RSA PRIVATE KEY-----"));
    }

    #[test]
    fn test_redact_openssh_private_key() {
        let text = "-----BEGIN OPENSSH PRIVATE KEY-----";
        let redacted = redact_sensitive_data(text);
        assert!(redacted.contains("[PRIVATE KEY REDACTED]"));
    }

    // =========================================================================
    // Composite/Integration Tests
    // =========================================================================

    #[test]
    fn test_redact_multiple_sensitive_items() {
        let text = r#"
            Config:
            - api_key=sk-1234567890abcdefghijklmnop
            - home: /Users/developer/code
            - Authorization: Bearer token123456789012345678901234
            - password=supersecret
        "#;

        let redacted = redact_sensitive_data(text);

        // All sensitive items should be redacted
        assert!(!redacted.contains("sk-1234567890abcdefghijklmnop"));
        assert!(!redacted.contains("/Users/developer"));
        assert!(!redacted.contains("token123456789012345678901234"));
        assert!(!redacted.contains("supersecret"));

        // Multiple redaction markers should be present
        assert!(redacted.matches(REDACTED).count() >= 3);
        assert!(redacted.contains("$HOME"));
    }

    #[test]
    fn test_preserve_non_sensitive_data() {
        let text = "Running command: ls -la in directory /tmp/build";
        let redacted = redact_sensitive_data(text);

        // Non-sensitive data should be preserved
        assert_eq!(text, redacted);
    }

    // =========================================================================
    // Context Redaction Tests
    // =========================================================================

    #[test]
    fn test_redact_context_error_info() {
        let mut context = create_test_context();
        context.error_info = Some(ErrorInfo {
            exit_code: Some(1),
            stderr: "Error: OPENAI_API_KEY=sk-secret123456789012345".to_string(),
            stdout: "Output from /Users/testuser/script.sh".to_string(),
            error_message: "Failed with password=secret".to_string(),
            error_type: Some("ExecutionError".to_string()),
        });

        redact_context(&mut context);

        let error = context.error_info.as_ref().unwrap();
        assert!(!error.stderr.contains("sk-secret"));
        assert!(!error.stdout.contains("/Users/testuser"));
        assert!(!error.error_message.contains("secret"));
    }

    #[test]
    fn test_redact_context_command_history() {
        let mut context = create_test_context();
        context.command_info.command_history = vec![HistoryEntry {
            prompt: "set api key to sk-12345678901234567890".to_string(),
            command: "export OPENAI_API_KEY=sk-12345678901234567890".to_string(),
            timestamp: Utc::now(),
            success: true,
        }];

        redact_context(&mut context);

        let entry = &context.command_info.command_history[0];
        assert!(!entry.command.contains("sk-"));
        assert!(entry.command.contains(REDACTED));
    }

    #[test]
    fn test_redact_context_paths() {
        let mut context = create_test_context();
        context.system_state.cache_dir = PathBuf::from("/Users/developer/.cache/caro");
        context.system_state.config_file =
            Some(PathBuf::from("/home/devuser/.config/caro/config.toml"));

        redact_context(&mut context);

        let cache_str = context.system_state.cache_dir.to_string_lossy();
        assert!(cache_str.contains("$HOME"));
        assert!(!cache_str.contains("/Users/developer"));

        let config_str = context
            .system_state
            .config_file
            .as_ref()
            .unwrap()
            .to_string_lossy();
        assert!(config_str.contains("$HOME"));
        assert!(!config_str.contains("/home/devuser"));
    }

    // =========================================================================
    // Helper Functions
    // =========================================================================

    fn create_test_context() -> FeedbackContext {
        FeedbackContext {
            timestamp: Utc::now(),
            cmdai_version: "1.0.0".to_string(),
            environment: EnvironmentInfo {
                os: "macos".to_string(),
                os_version: "14.0".to_string(),
                arch: "arm64".to_string(),
                shell: "zsh".to_string(),
                terminal: "Terminal.app".to_string(),
                rust_version: None,
            },
            command_info: CommandInfo {
                user_prompt: "test".to_string(),
                generated_command: "ls".to_string(),
                backend: "static".to_string(),
                model: None,
                command_history: vec![],
            },
            error_info: None,
            system_state: SystemState {
                available_backends: vec![],
                cache_dir: PathBuf::from("/tmp"),
                config_file: None,
                is_ci: false,
                is_interactive: true,
            },
            git_context: None,
        }
    }
}
