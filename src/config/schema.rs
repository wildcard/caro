//! Configuration schema and state management
//!
//! Provides comprehensive configuration management with TOML serialization,
//! validation rules, and support for interactive configuration UI.

use crate::models::{BackendType, RiskLevel, SafetyLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete configuration state for the cmdai system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationState {
    // Backend Configuration
    pub preferred_backend: BackendType,
    pub backend_configs: HashMap<String, BackendConfig>,
    pub fallback_chain: Vec<BackendType>,
    
    // History Settings
    pub history_enabled: bool,
    pub retention_policy: RetentionPolicy,
    pub privacy_mode: PrivacyLevel,
    pub auto_cleanup_days: u32,
    
    // Safety Configuration
    pub safety_level: SafetyLevel,
    pub confirmation_required: Vec<RiskLevel>,
    pub custom_safety_patterns: Vec<String>,
    
    // UI Preferences
    pub streaming_enabled: bool,
    pub color_output: bool,
    pub verbosity_level: VerbosityLevel,
}

/// Backend-specific configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub model_name: Option<String>,
    pub timeout_seconds: u32,
    pub max_retries: u32,
    pub additional_params: HashMap<String, String>,
}

/// Command history retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub max_entries: Option<usize>,
    pub max_age_days: Option<u32>,
    pub preserve_favorites: bool,
    pub preserve_frequently_used: bool,
}

/// Privacy level for command history storage
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrivacyLevel {
    /// Store all commands including sensitive data
    None,
    /// Filter obvious sensitive patterns (passwords, keys)
    Basic,
    /// Aggressive filtering of potential sensitive data
    Strict,
    /// Store only safe commands, block everything else
    Paranoid,
}

/// UI verbosity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerbosityLevel {
    Quiet,
    Normal,
    Verbose,
    Debug,
}

impl std::fmt::Display for VerbosityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerbosityLevel::Quiet => write!(f, "Quiet"),
            VerbosityLevel::Normal => write!(f, "Normal"),
            VerbosityLevel::Verbose => write!(f, "Verbose"),
            VerbosityLevel::Debug => write!(f, "Debug"),
        }
    }
}

/// Validation rules for configuration values
#[derive(Debug, Clone)]
pub struct ValidationRules {
    pub min_cleanup_days: u32,
    pub max_cleanup_days: u32,
    pub max_retention_entries: usize,
    pub max_custom_patterns: usize,
    pub required_backends: Vec<BackendType>,
}

impl Default for ConfigurationState {
    fn default() -> Self {
        let mut backend_configs = HashMap::new();
        
        // Default MLX configuration for Apple Silicon
        backend_configs.insert("mlx".to_string(), BackendConfig {
            enabled: true,
            endpoint: None,
            model_name: Some("mlx-community/Llama-3.2-3B-Instruct-4bit".to_string()),
            timeout_seconds: 30,
            max_retries: 2,
            additional_params: HashMap::new(),
        });
        
        // Default Mock backend configuration
        backend_configs.insert("mock".to_string(), BackendConfig {
            enabled: true,
            endpoint: None,
            model_name: None,
            timeout_seconds: 5,
            max_retries: 1,
            additional_params: HashMap::new(),
        });
        
        // Default Ollama configuration
        backend_configs.insert("ollama".to_string(), BackendConfig {
            enabled: false,
            endpoint: Some("http://localhost:11434".to_string()),
            model_name: Some("llama3.2:3b".to_string()),
            timeout_seconds: 60,
            max_retries: 3,
            additional_params: HashMap::new(),
        });

        Self {
            preferred_backend: BackendType::Auto,
            backend_configs,
            fallback_chain: vec![BackendType::MLX, BackendType::Mock],
            
            history_enabled: true,
            retention_policy: RetentionPolicy::default(),
            privacy_mode: PrivacyLevel::Basic,
            auto_cleanup_days: 30,
            
            safety_level: SafetyLevel::Moderate,
            confirmation_required: vec![RiskLevel::High, RiskLevel::Critical],
            custom_safety_patterns: Vec::new(),
            
            streaming_enabled: false,
            color_output: true,
            verbosity_level: VerbosityLevel::Normal,
        }
    }
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: None,
            model_name: None,
            timeout_seconds: 30,
            max_retries: 2,
            additional_params: HashMap::new(),
        }
    }
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_entries: Some(10000),
            max_age_days: Some(90),
            preserve_favorites: true,
            preserve_frequently_used: true,
        }
    }
}

impl Default for PrivacyLevel {
    fn default() -> Self {
        PrivacyLevel::Basic
    }
}

impl Default for VerbosityLevel {
    fn default() -> Self {
        VerbosityLevel::Normal
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            min_cleanup_days: 1,
            max_cleanup_days: 365,
            max_retention_entries: 100000,
            max_custom_patterns: 50,
            required_backends: vec![BackendType::Mock], // Mock always required as fallback
        }
    }
}

impl ConfigurationState {
    /// Create a new configuration state with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate the configuration state against validation rules
    pub fn validate(&self, rules: &ValidationRules) -> Result<(), ConfigValidationError> {
        // Validate cleanup days
        if self.auto_cleanup_days < rules.min_cleanup_days 
            || self.auto_cleanup_days > rules.max_cleanup_days {
            return Err(ConfigValidationError::InvalidCleanupDays {
                value: self.auto_cleanup_days,
                min: rules.min_cleanup_days,
                max: rules.max_cleanup_days,
            });
        }
        
        // Validate retention policy
        if let Some(max_entries) = self.retention_policy.max_entries {
            if max_entries > rules.max_retention_entries {
                return Err(ConfigValidationError::InvalidRetentionEntries {
                    value: max_entries,
                    max: rules.max_retention_entries,
                });
            }
        }
        
        // Validate custom safety patterns
        if self.custom_safety_patterns.len() > rules.max_custom_patterns {
            return Err(ConfigValidationError::TooManyCustomPatterns {
                count: self.custom_safety_patterns.len(),
                max: rules.max_custom_patterns,
            });
        }
        
        // Validate required backends are configured
        for required_backend in &rules.required_backends {
            let backend_key = required_backend.to_string().to_lowercase();
            if !self.backend_configs.contains_key(&backend_key) {
                return Err(ConfigValidationError::MissingRequiredBackend {
                    backend: *required_backend,
                });
            }
        }
        
        // Validate fallback chain has at least one enabled backend
        let has_enabled_fallback = self.fallback_chain.iter().any(|backend| {
            let backend_key = backend.to_string().to_lowercase();
            self.backend_configs.get(&backend_key)
                .map(|config| config.enabled)
                .unwrap_or(false)
        });
        
        if !has_enabled_fallback {
            return Err(ConfigValidationError::NoEnabledFallbackBackends);
        }
        
        Ok(())
    }
    
    /// Serialize configuration to TOML string
    pub fn to_toml(&self) -> Result<String, ConfigSerializationError> {
        toml::to_string_pretty(self)
            .map_err(|e| ConfigSerializationError::SerializationFailed(e.to_string()))
    }
    
    /// Deserialize configuration from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, ConfigSerializationError> {
        toml::from_str(toml_str)
            .map_err(|e| ConfigSerializationError::DeserializationFailed(e.to_string()))
    }
    
    /// Update backend configuration
    pub fn update_backend_config(&mut self, backend_name: &str, config: BackendConfig) {
        self.backend_configs.insert(backend_name.to_string(), config);
    }
    
    /// Enable or disable a specific backend
    pub fn set_backend_enabled(&mut self, backend_name: &str, enabled: bool) -> Result<(), String> {
        if let Some(config) = self.backend_configs.get_mut(backend_name) {
            config.enabled = enabled;
            Ok(())
        } else {
            Err(format!("Backend '{}' not found in configuration", backend_name))
        }
    }
    
    /// Add a custom safety pattern
    pub fn add_custom_safety_pattern(&mut self, pattern: String) -> Result<(), String> {
        if self.custom_safety_patterns.contains(&pattern) {
            return Err("Pattern already exists".to_string());
        }
        
        // Basic pattern validation
        if pattern.trim().is_empty() {
            return Err("Pattern cannot be empty".to_string());
        }
        
        self.custom_safety_patterns.push(pattern);
        Ok(())
    }
    
    /// Remove a custom safety pattern
    pub fn remove_custom_safety_pattern(&mut self, pattern: &str) -> bool {
        if let Some(pos) = self.custom_safety_patterns.iter().position(|p| p == pattern) {
            self.custom_safety_patterns.remove(pos);
            true
        } else {
            false
        }
    }
}

/// Configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("Invalid cleanup days: {value} (must be between {min} and {max})")]
    InvalidCleanupDays { value: u32, min: u32, max: u32 },
    
    #[error("Invalid retention entries: {value} (must be <= {max})")]
    InvalidRetentionEntries { value: usize, max: usize },
    
    #[error("Too many custom patterns: {count} (must be <= {max})")]
    TooManyCustomPatterns { count: usize, max: usize },
    
    #[error("Missing required backend: {backend:?}")]
    MissingRequiredBackend { backend: BackendType },
    
    #[error("No enabled backends in fallback chain")]
    NoEnabledFallbackBackends,
}

/// Configuration serialization errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigSerializationError {
    #[error("Failed to serialize configuration: {0}")]
    SerializationFailed(String),
    
    #[error("Failed to deserialize configuration: {0}")]
    DeserializationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configuration() {
        let config = ConfigurationState::default();
        
        assert_eq!(config.preferred_backend, BackendType::Auto);
        assert!(config.history_enabled);
        assert_eq!(config.privacy_mode, PrivacyLevel::Basic);
        assert_eq!(config.verbosity_level, VerbosityLevel::Normal);
        assert!(config.color_output);
    }

    #[test]
    fn test_toml_serialization() {
        let config = ConfigurationState::default();
        let toml_str = config.to_toml().expect("Should serialize to TOML");
        
        // Verify we can deserialize it back
        let deserialized = ConfigurationState::from_toml(&toml_str)
            .expect("Should deserialize from TOML");
        
        assert_eq!(config.preferred_backend, deserialized.preferred_backend);
        assert_eq!(config.history_enabled, deserialized.history_enabled);
    }

    #[test]
    fn test_configuration_validation() {
        let mut config = ConfigurationState::default();
        let rules = ValidationRules::default();
        
        // Valid configuration should pass
        assert!(config.validate(&rules).is_ok());
        
        // Invalid cleanup days should fail
        config.auto_cleanup_days = 500; // Exceeds max
        assert!(config.validate(&rules).is_err());
    }

    #[test]
    fn test_backend_management() {
        let mut config = ConfigurationState::default();
        
        // Enable/disable backends
        assert!(config.set_backend_enabled("mlx", false).is_ok());
        assert!(config.set_backend_enabled("nonexistent", true).is_err());
        
        // Add custom backend
        let new_backend = BackendConfig {
            enabled: true,
            endpoint: Some("http://example.com".to_string()),
            model_name: Some("custom-model".to_string()),
            timeout_seconds: 45,
            max_retries: 5,
            additional_params: HashMap::new(),
        };
        
        config.update_backend_config("custom", new_backend);
        assert!(config.backend_configs.contains_key("custom"));
    }

    #[test]
    fn test_custom_safety_patterns() {
        let mut config = ConfigurationState::default();
        
        // Add valid pattern
        assert!(config.add_custom_safety_pattern("rm -rf".to_string()).is_ok());
        assert_eq!(config.custom_safety_patterns.len(), 1);
        
        // Duplicate pattern should fail
        assert!(config.add_custom_safety_pattern("rm -rf".to_string()).is_err());
        
        // Empty pattern should fail
        assert!(config.add_custom_safety_pattern("".to_string()).is_err());
        
        // Remove pattern
        assert!(config.remove_custom_safety_pattern("rm -rf"));
        assert_eq!(config.custom_safety_patterns.len(), 0);
        assert!(!config.remove_custom_safety_pattern("nonexistent"));
    }
}