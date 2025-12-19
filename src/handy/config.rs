//! Configuration for Handy.Computer integration
//!
//! This module provides configuration options for controlling how
//! caro integrates with Handy.Computer.

use serde::{Deserialize, Serialize};

/// Configuration for Handy.Computer integration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HandyConfig {
    /// Enable Handy integration
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Auto-detect Handy on startup
    #[serde(default = "default_auto_detect")]
    pub auto_detect: bool,

    /// Show Handy status in welcome message
    #[serde(default = "default_show_status")]
    pub show_status: bool,

    /// Timeout for clipboard monitoring (milliseconds)
    #[serde(default = "default_clipboard_timeout_ms")]
    pub clipboard_timeout_ms: u64,

    /// Show transcription in UI
    #[serde(default = "default_show_transcription")]
    pub show_transcription: bool,
}

fn default_enabled() -> bool {
    true
}

fn default_auto_detect() -> bool {
    true
}

fn default_show_status() -> bool {
    true
}

fn default_clipboard_timeout_ms() -> u64 {
    30000 // 30 seconds
}

fn default_show_transcription() -> bool {
    true
}

impl Default for HandyConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            auto_detect: default_auto_detect(),
            show_status: default_show_status(),
            clipboard_timeout_ms: default_clipboard_timeout_ms(),
            show_transcription: default_show_transcription(),
        }
    }
}

impl HandyConfig {
    /// Create a new Handy configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a disabled configuration
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            auto_detect: false,
            show_status: false,
            clipboard_timeout_ms: default_clipboard_timeout_ms(),
            show_transcription: false,
        }
    }

    /// Check if Handy integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if auto-detection is enabled
    pub fn should_auto_detect(&self) -> bool {
        self.enabled && self.auto_detect
    }

    /// Check if status should be shown
    pub fn should_show_status(&self) -> bool {
        self.enabled && self.show_status
    }

    /// Get clipboard timeout duration
    pub fn clipboard_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.clipboard_timeout_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HandyConfig::default();
        assert!(config.enabled);
        assert!(config.auto_detect);
        assert!(config.show_status);
        assert_eq!(config.clipboard_timeout_ms, 30000);
        assert!(config.show_transcription);
    }

    #[test]
    fn test_disabled_config() {
        let config = HandyConfig::disabled();
        assert!(!config.enabled);
        assert!(!config.auto_detect);
        assert!(!config.show_status);
        assert!(!config.show_transcription);
    }

    #[test]
    fn test_config_methods() {
        let config = HandyConfig::default();
        assert!(config.is_enabled());
        assert!(config.should_auto_detect());
        assert!(config.should_show_status());

        let timeout = config.clipboard_timeout();
        assert_eq!(timeout.as_millis(), 30000);
    }

    #[test]
    fn test_config_serialization() {
        let config = HandyConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("enabled = true"));
        assert!(toml_str.contains("auto_detect = true"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            enabled = false
            auto_detect = false
            show_status = false
            clipboard_timeout_ms = 15000
            show_transcription = false
        "#;

        let config: HandyConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.enabled);
        assert!(!config.auto_detect);
        assert!(!config.show_status);
        assert_eq!(config.clipboard_timeout_ms, 15000);
        assert!(!config.show_transcription);
    }
}
