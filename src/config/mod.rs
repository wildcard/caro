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
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Invalid configuration: {0}")]
    ValidationError(String),

    #[error("Config directory error: {0}")]
    DirectoryError(String),

    #[error("Deprecated key: {old_key} (use {new_key} instead)")]
    DeprecatedKey { old_key: String, new_key: String },
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
                ConfigError::DirectoryError("Could not determine config directory".to_string())
            })?
            .join("caro");

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
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
                std::fs::create_dir_all(parent)?;
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

        let contents = std::fs::read_to_string(&self.config_path)?;
        let config: UserConfiguration = toml::from_str(&contents)?;

        // Validate configuration
        config.validate().map_err(ConfigError::ValidationError)?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, config: &UserConfiguration) -> Result<(), ConfigError> {
        // Validate before saving
        config.validate().map_err(ConfigError::ValidationError)?;

        let toml_string = toml::to_string_pretty(config)?;
        std::fs::write(&self.config_path, toml_string)?;

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
            config.safety_level = safety_str.parse().map_err(ConfigError::ValidationError)?;
        }

        if let Some(shell_str) = cli_shell {
            config.default_shell = Some(shell_str.parse().map_err(ConfigError::ValidationError)?);
        }

        if let Some(log_str) = cli_log_level {
            config.log_level = log_str.parse().map_err(ConfigError::ValidationError)?;
        }

        Ok(config)
    }

    /// Merge environment variables with config (env vars take precedence over file)
    pub fn merge_with_env(&self) -> Result<UserConfiguration, ConfigError> {
        let mut config = self.load()?;

        // Check for environment variable overrides
        if let Ok(safety_str) = std::env::var("CARO_SAFETY_LEVEL") {
            config.safety_level = safety_str.parse().map_err(ConfigError::ValidationError)?;
        }

        if let Ok(shell_str) = std::env::var("CARO_DEFAULT_SHELL") {
            config.default_shell = Some(shell_str.parse().map_err(ConfigError::ValidationError)?);
        }

        if let Ok(log_str) = std::env::var("CARO_LOG_LEVEL") {
            config.log_level = log_str.parse().map_err(ConfigError::ValidationError)?;
        }

        if let Ok(model_str) = std::env::var("CARO_DEFAULT_MODEL") {
            config.default_model = Some(model_str);
        }

        if let Ok(cache_str) = std::env::var("CARO_CACHE_MAX_SIZE_GB") {
            config.cache_max_size_gb = cache_str.parse().map_err(|_| {
                ConfigError::ValidationError(format!("Invalid cache size: {}", cache_str))
            })?;
        }

        Ok(config)
    }

    /// Validate config file against schema (check for deprecated/unknown keys)
    pub fn validate_schema(&self) -> Result<Vec<String>, ConfigError> {
        if !self.config_path.exists() {
            return Ok(Vec::new());
        }

        let contents = std::fs::read_to_string(&self.config_path)?;
        let value: toml::Value = toml::from_str(&contents)?;

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
}
