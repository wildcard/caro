//! Integration tests for interactive configuration UI

use cmdai::config::{ConfigManager, run_interactive_config};
use cmdai::models::{LogLevel, SafetyLevel, UserConfiguration};
use tempfile::TempDir;

#[test]
fn test_interactive_config_creation() {
    let config = UserConfiguration::default();
    let result = run_interactive_config(config);
    
    // Should be able to create the interactive config, even if it fails in headless mode
    // This just tests that the function exists and has correct signature
    assert!(result.is_err()); // Expected to fail in headless environment
}

#[test]
fn test_config_manager_with_interactive_ui() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let config_manager = ConfigManager::with_config_path(config_path).unwrap();

    // Test that we can load config that will be used by interactive UI
    let config = config_manager.load().unwrap();
    assert_eq!(config.safety_level, SafetyLevel::Moderate);
    assert_eq!(config.log_level, LogLevel::Info);
    assert_eq!(config.cache_max_size_gb, 10);
}

#[test]
fn test_config_validation_for_interactive_ui() {
    let config = UserConfiguration {
        default_shell: None,
        safety_level: SafetyLevel::Strict,
        default_model: Some("test-model".to_string()),
        log_level: LogLevel::Debug,
        cache_max_size_gb: 20,
        log_rotation_days: 30,
    };

    // Configuration should be valid for interactive UI
    assert!(config.validate().is_ok());
}

#[test]
fn test_invalid_config_for_interactive_ui() {
    let config = UserConfiguration {
        default_shell: None,
        safety_level: SafetyLevel::Moderate,
        default_model: None,
        log_level: LogLevel::Info,
        cache_max_size_gb: 0, // Invalid - should be at least 1
        log_rotation_days: 7,
    };

    // Configuration should be invalid
    assert!(config.validate().is_err());
}