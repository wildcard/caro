//! Configuration module for managing user preferences and settings
//!
//! Provides TOML-based configuration with defaults, CLI override, and env var support.

use crate::models::ConfigSchema;
use std::path::{Path, PathBuf};

mod schema;
pub use schema::SchemaValidator;

// Re-export models types for convenience
pub use crate::models::{UserConfiguration, UserConfigurationBuilder};

/// Configuration-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error: {message}\nPath: {path}\n\nSuggestion: {suggestion}")]
    IoError {
        message: String,
        path: PathBuf,
        suggestion: String,
    },

    #[error("Permission denied: {path}\n\nThe config file/directory requires {operation} permissions.\nSuggestion: Fix permissions with:\n  chmod 600 {path}")]
    PermissionDenied {
        path: PathBuf,
        operation: String,
    },

    #[error("TOML parse error in {path}:\n{message}\n\nSuggestion: Check the TOML syntax at the reported location.\nValid TOML example:\n  [general]\n  safety_level = \"moderate\"\n  default_shell = \"bash\"")]
    ParseError {
        path: PathBuf,
        message: String,
    },

    #[error("TOML serialization error: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Invalid {field}: '{value}'\nValid values: {valid_values}\n\nSuggestion: Update your config file with one of the valid values.{did_you_mean}")]
    InvalidValue {
        field: String,
        value: String,
        valid_values: String,
        did_you_mean: String,
    },

    #[error("Invalid {field}: {value}\nReason: {reason}\n\nSuggestion: {suggestion}")]
    InvalidRange {
        field: String,
        value: String,
        reason: String,
        suggestion: String,
    },

    #[error("Missing required field: {field}\n\nSuggestion: Add this to your config file:\n  [{section}]\n  {field} = {example_value}")]
    MissingField {
        field: String,
        section: String,
        example_value: String,
    },

    #[error("Config directory error: {message}\n\nSuggestion: {suggestion}")]
    DirectoryError {
        message: String,
        suggestion: String,
    },

    #[error("Deprecated key: {old_key}\n\nThis configuration key has been renamed.\nReplace '{old_key}' with '{new_key}' in your config file: {config_path}")]
    DeprecatedKey {
        old_key: String,
        new_key: String,
        config_path: String,
    },

    #[error("Unknown configuration key: {key} in section [{section}]\n\nSuggestion: Remove this key or check for typos.\nValid keys for [{section}]:\n{valid_keys}")]
    UnknownKey {
        key: String,
        section: String,
        valid_keys: String,
    },
}

impl ConfigError {
    /// Wrap an IO error with context and suggestions
    pub fn from_io_error(err: std::io::Error, path: PathBuf) -> Self {
        let suggestion = match err.kind() {
            std::io::ErrorKind::PermissionDenied => {
                return Self::PermissionDenied {
                    path,
                    operation: "read/write".to_string(),
                }
            }
            std::io::ErrorKind::NotFound => {
                format!(
                    "The config file doesn't exist. Create it with default values:\n  cmdai config init"
                )
            }
            std::io::ErrorKind::AlreadyExists => {
                format!("The path already exists: {}", path.display())
            }
            _ => format!("Check that the path is accessible: {}", path.display()),
        };

        Self::IoError {
            message: err.to_string(),
            path,
            suggestion,
        }
    }

    /// Create a parse error with file context
    pub fn from_parse_error(err: toml::de::Error, path: PathBuf) -> Self {
        Self::ParseError {
            path,
            message: err.to_string(),
        }
    }

    /// Create an invalid value error with "did you mean?" suggestion
    pub fn invalid_value(field: &str, value: &str, valid_values: &[&str]) -> Self {
        let valid_str = valid_values.join(", ");
        let did_you_mean = Self::suggest_similar(value, valid_values);

        Self::InvalidValue {
            field: field.to_string(),
            value: value.to_string(),
            valid_values: valid_str,
            did_you_mean,
        }
    }

    /// Create an invalid range error with suggestions
    pub fn invalid_range(field: &str, value: impl ToString, min: u64, max: u64) -> Self {
        Self::InvalidRange {
            field: field.to_string(),
            value: value.to_string(),
            reason: format!("must be between {} and {}", min, max),
            suggestion: format!("Set {} to a value between {} and {}", field, min, max),
        }
    }

    /// Create a directory error with suggestions
    pub fn directory_error(message: impl Into<String>) -> Self {
        let message = message.into();
        let suggestion = if message.contains("not determine") {
            "Set the XDG_CONFIG_HOME environment variable:\n  export XDG_CONFIG_HOME=$HOME/.config".to_string()
        } else {
            "Check directory permissions and available disk space.".to_string()
        };

        Self::DirectoryError {
            message,
            suggestion,
        }
    }

    /// Suggest similar valid values using Levenshtein distance
    fn suggest_similar(input: &str, valid_values: &[&str]) -> String {
        let lowercase_input = input.to_lowercase();

        // Find the closest match
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for &valid in valid_values {
            let distance = levenshtein_distance(&lowercase_input, &valid.to_lowercase());
            if distance < best_distance && distance <= 2 {
                best_distance = distance;
                best_match = Some(valid);
            }
        }

        match best_match {
            Some(suggestion) => format!("\n\nDid you mean '{}'?", suggestion),
            None => String::new(),
        }
    }
}

/// Simple Levenshtein distance implementation for typo suggestions
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];

    for (i, a_char) in a.chars().enumerate() {
        curr_row[0] = i + 1;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            curr_row[j + 1] = std::cmp::min(
                std::cmp::min(
                    curr_row[j] + 1,         // insertion
                    prev_row[j + 1] + 1      // deletion
                ),
                prev_row[j] + cost           // substitution
            );
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

/// Manages user configuration
pub struct ConfigManager {
    config_path: PathBuf,
    schema: ConfigSchema,
}

impl ConfigManager {
    /// Create a new ConfigManager with default XDG config directory
    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| {
                ConfigError::directory_error("Could not determine config directory")
            })?
            .join("cmdai");

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| ConfigError::from_io_error(e, config_dir.clone()))?;
        }

        let config_path = config_dir.join("config.toml");

        Ok(Self {
            config_path,
            schema: ConfigSchema::default(),
        })
    }

    /// Create a ConfigManager with a custom config path
    pub fn with_config_path(config_path: PathBuf) -> Result<Self, ConfigError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::from_io_error(e, parent.to_path_buf()))?;
            }
        }

        Ok(Self {
            config_path,
            schema: ConfigSchema::default(),
        })
    }

    /// Get the config file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Load configuration from file, or return defaults if not found
    pub fn load(&self) -> Result<UserConfiguration, ConfigError> {
        if !self.config_path.exists() {
            return Ok(UserConfiguration::default());
        }

        let contents = std::fs::read_to_string(&self.config_path)
            .map_err(|e| ConfigError::from_io_error(e, self.config_path.clone()))?;

        let config: UserConfiguration = toml::from_str(&contents)
            .map_err(|e| ConfigError::from_parse_error(e, self.config_path.clone()))?;

        // Validate configuration
        config.validate().map_err(|e| {
            // Convert validation error string to specific error type
            if e.contains("cache_max_size_gb") {
                if let Some(value) = e.split("got ").nth(1) {
                    return ConfigError::invalid_range("cache_max_size_gb", value.trim(), 1, 1000);
                }
            } else if e.contains("log_rotation_days") {
                if let Some(value) = e.split("got ").nth(1) {
                    return ConfigError::invalid_range("log_rotation_days", value.trim(), 1, 365);
                }
            }
            // Fallback to generic validation error for backward compatibility
            ConfigError::InvalidRange {
                field: "configuration".to_string(),
                value: String::new(),
                reason: e.clone(),
                suggestion: "Check the configuration values against the valid ranges.".to_string(),
            }
        })?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, config: &UserConfiguration) -> Result<(), ConfigError> {
        // Validate before saving
        config.validate().map_err(|e| {
            if e.contains("cache_max_size_gb") {
                if let Some(value) = e.split("got ").nth(1) {
                    return ConfigError::invalid_range("cache_max_size_gb", value.trim(), 1, 1000);
                }
            } else if e.contains("log_rotation_days") {
                if let Some(value) = e.split("got ").nth(1) {
                    return ConfigError::invalid_range("log_rotation_days", value.trim(), 1, 365);
                }
            }
            ConfigError::InvalidRange {
                field: "configuration".to_string(),
                value: String::new(),
                reason: e.clone(),
                suggestion: "Check the configuration values against the valid ranges.".to_string(),
            }
        })?;

        let toml_string = toml::to_string_pretty(config)?;
        std::fs::write(&self.config_path, toml_string)
            .map_err(|e| ConfigError::from_io_error(e, self.config_path.clone()))?;

        Ok(())
    }

    /// Merge CLI arguments with file config (CLI takes precedence)
    pub fn merge_with_cli(
        &self,
        cli_safety: Option<&str>,
        cli_shell: Option<&str>,
        cli_log_level: Option<&str>,
    ) -> Result<UserConfiguration, ConfigError> {
        let mut config = self.load()?;

        // Override with CLI args if provided
        if let Some(safety_str) = cli_safety {
            config.safety_level = safety_str.parse().map_err(|_| {
                ConfigError::invalid_value(
                    "safety_level",
                    safety_str,
                    &["strict", "moderate", "permissive"],
                )
            })?;
        }

        if let Some(shell_str) = cli_shell {
            config.default_shell = Some(shell_str.parse().map_err(|_| {
                ConfigError::invalid_value(
                    "default_shell",
                    shell_str,
                    &["bash", "zsh", "fish", "sh", "powershell", "cmd"],
                )
            })?);
        }

        if let Some(log_str) = cli_log_level {
            config.log_level = log_str.parse().map_err(|_| {
                ConfigError::invalid_value("log_level", log_str, &["debug", "info", "warn", "error"])
            })?;
        }

        Ok(config)
    }

    /// Merge environment variables with config (env vars take precedence over file)
    pub fn merge_with_env(&self) -> Result<UserConfiguration, ConfigError> {
        let mut config = self.load()?;

        // Check for environment variable overrides
        if let Ok(safety_str) = std::env::var("CMDAI_SAFETY_LEVEL") {
            config.safety_level = safety_str.parse().map_err(|_| {
                ConfigError::invalid_value(
                    "CMDAI_SAFETY_LEVEL",
                    &safety_str,
                    &["strict", "moderate", "permissive"],
                )
            })?;
        }

        if let Ok(shell_str) = std::env::var("CMDAI_DEFAULT_SHELL") {
            config.default_shell = Some(shell_str.parse().map_err(|_| {
                ConfigError::invalid_value(
                    "CMDAI_DEFAULT_SHELL",
                    &shell_str,
                    &["bash", "zsh", "fish", "sh", "powershell", "cmd"],
                )
            })?);
        }

        if let Ok(log_str) = std::env::var("CMDAI_LOG_LEVEL") {
            config.log_level = log_str.parse().map_err(|_| {
                ConfigError::invalid_value(
                    "CMDAI_LOG_LEVEL",
                    &log_str,
                    &["debug", "info", "warn", "error"],
                )
            })?;
        }

        if let Ok(model_str) = std::env::var("CMDAI_DEFAULT_MODEL") {
            config.default_model = Some(model_str);
        }

        if let Ok(cache_str) = std::env::var("CMDAI_CACHE_MAX_SIZE_GB") {
            let cache_size: u64 = cache_str.parse().map_err(|_| {
                ConfigError::InvalidRange {
                    field: "CMDAI_CACHE_MAX_SIZE_GB".to_string(),
                    value: cache_str.clone(),
                    reason: "must be a positive integer".to_string(),
                    suggestion: "Set CMDAI_CACHE_MAX_SIZE_GB to a number between 1 and 1000".to_string(),
                }
            })?;

            if cache_size < 1 || cache_size > 1000 {
                return Err(ConfigError::invalid_range("CMDAI_CACHE_MAX_SIZE_GB", cache_size, 1, 1000));
            }

            config.cache_max_size_gb = cache_size;
        }

        Ok(config)
    }

    /// Validate config file against schema (check for deprecated/unknown keys)
    pub fn validate_schema(&self) -> Result<Vec<String>, ConfigError> {
        if !self.config_path.exists() {
            return Ok(Vec::new());
        }

        let contents = std::fs::read_to_string(&self.config_path)
            .map_err(|e| ConfigError::from_io_error(e, self.config_path.clone()))?;

        let value: toml::Value = toml::from_str(&contents)
            .map_err(|e| ConfigError::from_parse_error(e, self.config_path.clone()))?;

        let mut warnings = Vec::new();

        // Check for deprecated keys
        if let toml::Value::Table(table) = value {
            for (section, section_value) in &table {
                if let toml::Value::Table(section_table) = section_value {
                    for key in section_table.keys() {
                        let full_key = format!("{}.{}", section, key);
                        if let Some(new_key) = self.schema.deprecated_keys.get(&full_key) {
                            warnings.push(format!(
                                "Deprecated key '{}' (use '{}' instead)",
                                full_key, new_key
                            ));
                        }
                    }
                }
            }
        }

        Ok(warnings)
    }

    /// Get config path as string
    pub fn config_path_string(&self) -> String {
        self.config_path.to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LogLevel, SafetyLevel};
    use tempfile::TempDir;

    #[test]
    fn test_config_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_manager = ConfigManager::with_config_path(config_path);
        assert!(config_manager.is_ok());
    }

    #[test]
    fn test_load_defaults_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        let config_manager = ConfigManager::with_config_path(config_path).unwrap();
        let config = config_manager.load();
        assert!(config.is_ok());
        assert_eq!(config.unwrap().safety_level, SafetyLevel::Moderate);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_manager = ConfigManager::with_config_path(config_path).unwrap();

        let config = UserConfiguration {
            safety_level: SafetyLevel::Strict,
            log_level: LogLevel::Debug,
            ..Default::default()
        };

        assert!(config_manager.save(&config).is_ok());
        let loaded = config_manager.load().unwrap();
        assert_eq!(loaded.safety_level, SafetyLevel::Strict);
        assert_eq!(loaded.log_level, LogLevel::Debug);
    }

    #[test]
    fn test_error_messages_have_suggestions() {
        // Test invalid value error with typo suggestion
        let error = ConfigError::invalid_value("safety_level", "stric", &["strict", "moderate", "permissive"]);
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("Did you mean 'strict'?"));

        // Test invalid value error without typo match
        let error = ConfigError::invalid_value("safety_level", "xyz", &["strict", "moderate", "permissive"]);
        let error_msg = error.to_string();
        assert!(error_msg.contains("Valid values: strict, moderate, permissive"));
        assert!(!error_msg.contains("Did you mean"));

        // Test invalid range error
        let error = ConfigError::invalid_range("cache_max_size_gb", 2000, 1, 1000);
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("between 1 and 1000"));

        // Test directory error
        let error = ConfigError::directory_error("Could not determine config directory");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("XDG_CONFIG_HOME"));

        // Test permission denied error
        let error = ConfigError::PermissionDenied {
            path: PathBuf::from("/test/config.toml"),
            operation: "read/write".to_string(),
        };
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("chmod 600"));
    }

    #[test]
    fn test_levenshtein_distance() {
        // Direct access to test the helper function
        use super::levenshtein_distance;

        assert_eq!(levenshtein_distance("strict", "stric"), 1);
        assert_eq!(levenshtein_distance("moderate", "modrate"), 1);
        assert_eq!(levenshtein_distance("bash", "bsh"), 1);
        assert_eq!(levenshtein_distance("debug", "info"), 5); // debug -> info: d->i, e->n, b->f, u->o, g removed = 5
        assert_eq!(levenshtein_distance("", "test"), 4);
        assert_eq!(levenshtein_distance("test", ""), 4);
    }

    #[test]
    fn test_did_you_mean_suggestions() {
        // Test close match
        let error = ConfigError::invalid_value("shell", "bsh", &["bash", "zsh", "fish"]);
        let error_msg = error.to_string();
        assert!(error_msg.contains("Did you mean 'bash'?"));

        // Test exact match (should not happen in practice)
        let error = ConfigError::invalid_value("shell", "bash", &["bash", "zsh", "fish"]);
        let error_msg = error.to_string();
        assert!(error_msg.contains("Did you mean 'bash'?"));

        // Test no close match
        let error = ConfigError::invalid_value("shell", "xyz", &["bash", "zsh", "fish"]);
        let error_msg = error.to_string();
        assert!(!error_msg.contains("Did you mean"));
    }

    #[test]
    fn test_io_error_context() {
        use std::io;

        // Test permission denied
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let config_err = ConfigError::from_io_error(io_err, PathBuf::from("/test/config.toml"));

        match config_err {
            ConfigError::PermissionDenied { path, operation } => {
                assert_eq!(path, PathBuf::from("/test/config.toml"));
                assert_eq!(operation, "read/write");
            }
            _ => panic!("Expected PermissionDenied error"),
        }

        // Test not found
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let config_err = ConfigError::from_io_error(io_err, PathBuf::from("/test/config.toml"));
        let error_msg = config_err.to_string();
        assert!(error_msg.contains("cmdai config init"));
    }

    #[test]
    fn test_parse_error_context() {
        // Create a parse error by parsing invalid TOML
        let invalid_toml = "[section\n  missing_bracket = true";
        let parse_err = toml::from_str::<toml::Value>(invalid_toml).unwrap_err();
        let config_err = ConfigError::from_parse_error(parse_err, PathBuf::from("/test/config.toml"));
        let error_msg = config_err.to_string();
        assert!(error_msg.contains("/test/config.toml"));
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("TOML"));
    }
}
