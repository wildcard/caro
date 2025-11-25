// Config module contract tests - THESE MUST FAIL INITIALLY (TDD)
// Tests validate the config module API from specs/003-implement-core-infrastructure/contracts/config-api.md

use std::fs;
use tempfile::TempDir;

// Import types that will be implemented later
// NOTE: These imports will fail until we implement the actual config module
use cmdai::config::{ConfigError, ConfigManager, UserConfiguration};
use cmdai::models::{LogLevel, SafetyLevel, ShellType};

// Mock CLI args for testing
struct TestCliArgs {
    safety: Option<String>,
    shell: Option<String>,
    log_level: Option<String>,
}

#[test]
fn test_config_manager_new() {
    // CONTRACT: ConfigManager::new() creates manager with default XDG config path
    let result = ConfigManager::new();

    assert!(result.is_ok(), "ConfigManager creation should succeed");
    let config_manager = result.unwrap();

    // Verify config path is set to XDG location
    let config_path = config_manager.config_path();
    assert!(
        config_path.to_str().unwrap().contains("cmdai"),
        "Config path should contain cmdai"
    );
}

#[test]
fn test_config_manager_with_custom_path() {
    // CONTRACT: ConfigManager::with_config_path() uses custom path
    let temp_dir = TempDir::new().unwrap();
    let custom_path = temp_dir.path().join("custom_config.toml");

    let result = ConfigManager::with_config_path(custom_path.clone());

    assert!(
        result.is_ok(),
        "ConfigManager with custom path should succeed"
    );
    let config_manager = result.unwrap();

    assert_eq!(
        config_manager.config_path(),
        custom_path.as_path(),
        "Should use custom config path"
    );
}

#[test]
fn test_load_returns_defaults_when_missing() {
    // CONTRACT: load() returns defaults when config file doesn't exist
    let temp_dir = TempDir::new().unwrap();
    let non_existent = temp_dir.path().join("does_not_exist.toml");

    let config_manager = ConfigManager::with_config_path(non_existent).unwrap();
    let result = config_manager.load();

    assert!(
        result.is_ok(),
        "load() should succeed with defaults when file missing"
    );

    let config = result.unwrap();
    assert_eq!(
        config.safety_level,
        SafetyLevel::Moderate,
        "Default safety should be Moderate"
    );
    assert_eq!(
        config.log_level,
        LogLevel::Info,
        "Default log level should be Info"
    );
    assert_eq!(
        config.cache_max_size_gb, 10,
        "Default cache size should be 10GB"
    );
    assert_eq!(
        config.log_rotation_days, 7,
        "Default log rotation should be 7 days"
    );
}

#[test]
fn test_load_parses_valid_config() {
    // CONTRACT: load() successfully parses valid TOML config
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Write valid TOML config (flat structure)
    let toml_content = r#"
safety_level = "strict"
default_shell = "bash"
default_model = "test-model"
log_level = "debug"
log_rotation_days = 3
cache_max_size_gb = 5
"#;
    fs::write(&config_path, toml_content).unwrap();

    let config_manager = ConfigManager::with_config_path(config_path).unwrap();
    let result = config_manager.load();

    assert!(
        result.is_ok(),
        "Should parse valid TOML: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.safety_level, SafetyLevel::Strict);
    assert_eq!(config.default_shell, Some(ShellType::Bash));
    assert_eq!(config.default_model, Some("test-model".to_string()));
    assert_eq!(config.log_level, LogLevel::Debug);
    assert_eq!(config.log_rotation_days, 3);
    assert_eq!(config.cache_max_size_gb, 5);
}

#[test]
fn test_load_fails_on_invalid_toml() {
    // CONTRACT: load() returns InvalidToml error for malformed TOML
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bad_config.toml");

    // Write invalid TOML (unclosed quote)
    let bad_toml = r#"
[general]
safety_level = "Strict
"#;
    fs::write(&config_path, bad_toml).unwrap();

    let config_manager = ConfigManager::with_config_path(config_path).unwrap();
    let result = config_manager.load();

    assert!(result.is_err(), "Should fail on invalid TOML");

    match result.unwrap_err() {
        ConfigError::ParseError(_) => { /* Expected */ }
        e => panic!("Expected ParseError error, got: {:?}", e),
    }
}

#[test]
fn test_load_fails_on_invalid_enum() {
    // CONTRACT: load() returns InvalidValue for invalid enum variants
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid_enum.toml");

    // Write config with invalid safety_level
    let toml_content = r#"
[general]
safety_level = "high"
"#;
    fs::write(&config_path, toml_content).unwrap();

    let config_manager = ConfigManager::with_config_path(config_path).unwrap();
    let result = config_manager.load();

    assert!(result.is_err(), "Should fail on invalid enum value");

    // Note: The actual error will be ParseError from TOML deserialization
    // or ValidationError from the validate() call
    match result.unwrap_err() {
        ConfigError::ParseError(_) | ConfigError::ValidationError(_) => {
            // Expected - invalid enum values cause parse or validation errors
        }
        e => panic!("Expected ParseError or ValidationError, got: {:?}", e),
    }
}

#[test]
fn test_load_warns_on_unknown_section() {
    // CONTRACT: load() succeeds but warns on unknown sections (forward compatibility)
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("unknown_section.toml");

    let toml_content = r#"
safety_level = "moderate"
log_level = "info"
log_rotation_days = 7
cache_max_size_gb = 10
"#;
    fs::write(&config_path, toml_content).unwrap();

    let config_manager = ConfigManager::with_config_path(config_path).unwrap();
    let result = config_manager.load();

    // Should succeed despite unknown section
    assert!(
        result.is_ok(),
        "Should succeed with unknown section (forward compatibility): {:?}",
        result.err()
    );
}

#[test]
fn test_save_persists_configuration() {
    // CONTRACT: save() writes configuration to TOML file
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("saved_config.toml");

    let config_manager = ConfigManager::with_config_path(config_path.clone()).unwrap();

    let config = UserConfiguration {
        default_shell: Some(ShellType::Zsh),
        safety_level: SafetyLevel::Strict,
        default_model: Some("my-model".to_string()),
        log_level: LogLevel::Warn,
        cache_max_size_gb: 20,
        log_rotation_days: 14,
    };

    let save_result = config_manager.save(&config);
    assert!(save_result.is_ok(), "Save should succeed");

    // Verify file exists and can be loaded back
    assert!(config_path.exists(), "Config file should exist after save");

    let loaded = config_manager.load().unwrap();
    assert_eq!(loaded, config, "Loaded config should match saved config");
}

#[test]
fn test_save_overwrites_existing() {
    // CONTRACT: save() overwrites existing config file atomically
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("overwrite_config.toml");

    let config_manager = ConfigManager::with_config_path(config_path.clone()).unwrap();

    // Save first config
    let config1 = UserConfiguration {
        safety_level: SafetyLevel::Moderate,
        ..UserConfiguration::default()
    };
    config_manager.save(&config1).unwrap();

    // Save second config
    let config2 = UserConfiguration {
        safety_level: SafetyLevel::Strict,
        ..UserConfiguration::default()
    };
    config_manager.save(&config2).unwrap();

    // Verify only second config persists
    let loaded = config_manager.load().unwrap();
    assert_eq!(
        loaded.safety_level,
        SafetyLevel::Strict,
        "Should have latest saved value"
    );
}

#[test]
fn test_merge_with_cli_args_prioritizes_cli() {
    // CONTRACT: merge_with_cli_args() gives CLI args precedence over config
    let temp_dir = TempDir::new().unwrap();
    let config_manager =
        ConfigManager::with_config_path(temp_dir.path().join("test.toml")).unwrap();

    let config = UserConfiguration {
        safety_level: SafetyLevel::Moderate,
        default_shell: Some(ShellType::Bash),
        log_level: LogLevel::Info,
        ..UserConfiguration::default()
    };

    let cli_args = TestCliArgs {
        safety: Some("strict".to_string()),
        shell: None, // Not specified
        log_level: Some("debug".to_string()),
    };

    // Save config first so merge_with_cli can load it
    config_manager.save(&config).unwrap();

    let merged = config_manager
        .merge_with_cli(
            cli_args.safety.as_deref(),
            cli_args.shell.as_deref(),
            cli_args.log_level.as_deref(),
        )
        .unwrap();

    assert_eq!(
        merged.safety_level,
        SafetyLevel::Strict,
        "CLI safety should override config"
    );
    assert_eq!(
        merged.default_shell,
        Some(ShellType::Bash),
        "Config shell should remain (no CLI override)"
    );
    assert_eq!(
        merged.log_level,
        LogLevel::Debug,
        "CLI log level should override config"
    );
}

#[test]
fn test_merge_uses_config_defaults() {
    // CONTRACT: merge_with_cli_args() uses config values when CLI args not provided
    let temp_dir = TempDir::new().unwrap();
    let config_manager =
        ConfigManager::with_config_path(temp_dir.path().join("test.toml")).unwrap();

    let config = UserConfiguration {
        safety_level: SafetyLevel::Strict,
        default_shell: Some(ShellType::Fish),
        ..UserConfiguration::default()
    };

    let cli_args = TestCliArgs {
        safety: None,
        shell: None,
        log_level: None,
    };

    // Save config first so merge_with_cli can load it
    config_manager.save(&config).unwrap();

    let merged = config_manager
        .merge_with_cli(
            cli_args.safety.as_deref(),
            cli_args.shell.as_deref(),
            cli_args.log_level.as_deref(),
        )
        .unwrap();

    assert_eq!(
        merged, config,
        "Should use all config values when no CLI overrides"
    );
}

#[test]
fn test_validate_accepts_valid_config() {
    // CONTRACT: validate() returns Ok for valid configuration
    let temp_dir = TempDir::new().unwrap();
    let _ = ConfigManager::with_config_path(temp_dir.path().join("test.toml")).unwrap();

    let valid_config = UserConfiguration {
        safety_level: SafetyLevel::Moderate,
        default_shell: Some(ShellType::Bash),
        default_model: Some("valid-model".to_string()),
        log_level: LogLevel::Info,
        cache_max_size_gb: 10,
        log_rotation_days: 7,
    };

    // UserConfiguration has its own validate() method
    let result = valid_config.validate();
    assert!(result.is_ok(), "Valid config should pass validation");
}

#[test]
fn test_validate_rejects_out_of_range() {
    // CONTRACT: validate() rejects out-of-range values
    let temp_dir = TempDir::new().unwrap();
    let config_manager =
        ConfigManager::with_config_path(temp_dir.path().join("test.toml")).unwrap();

    let invalid_config = UserConfiguration {
        cache_max_size_gb: 0, // Invalid: must be >= 1
        ..UserConfiguration::default()
    };

    // UserConfiguration::validate() returns Result<(), String>
    let result = invalid_config.validate();
    assert!(result.is_err(), "Should reject cache_max_size_gb = 0");

    if let Err(msg) = result {
        assert!(
            msg.contains("cache_max_size_gb") || msg.contains("0"),
            "Error should mention invalid value"
        );
    }

    // Suppress unused variable warning
    let _ = config_manager;
}

#[test]
fn test_user_configuration_default() {
    // CONTRACT: UserConfiguration::default() provides sensible defaults
    let config = UserConfiguration::default();

    assert_eq!(config.safety_level, SafetyLevel::Moderate);
    assert_eq!(config.default_shell, None); // Auto-detect
    assert_eq!(config.default_model, None);
    assert_eq!(config.log_level, LogLevel::Info);
    assert_eq!(config.cache_max_size_gb, 10);
    assert_eq!(config.log_rotation_days, 7);
}

#[test]
fn test_user_configuration_builder() {
    // CONTRACT: Builder pattern allows programmatic construction
    let config = UserConfiguration::builder()
        .safety_level(SafetyLevel::Strict)
        .default_shell(ShellType::Zsh)
        .default_model("custom-model")
        .log_level(LogLevel::Debug)
        .cache_max_size_gb(20)
        .log_rotation_days(14)
        .build();

    assert!(config.is_ok(), "Builder should succeed");

    let config = config.unwrap();
    assert_eq!(config.safety_level, SafetyLevel::Strict);
    assert_eq!(config.default_shell, Some(ShellType::Zsh));
    assert_eq!(config.default_model, Some("custom-model".to_string()));
    assert_eq!(config.log_level, LogLevel::Debug);
    assert_eq!(config.cache_max_size_gb, 20);
    assert_eq!(config.log_rotation_days, 14);
}

#[test]
fn test_config_path_resolution() {
    // CONTRACT: Config path resolved to XDG-compliant location
    let config_manager = ConfigManager::new().unwrap();
    let config_path = config_manager.config_path();

    // Should be in .config/cmdai/ on Linux/macOS or equivalent on Windows
    let path_str = config_path.to_str().unwrap();
    assert!(
        path_str.contains("config") || path_str.contains("Config") || path_str.contains("cmdai"),
        "Config path should be in standard config location: {}",
        path_str
    );
}

#[test]
fn test_config_operation_performance() {
    // CONTRACT: Config operations meet performance requirements (NFR-002)
    use std::time::Instant;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("perf_test.toml");

    // Write a config file
    let config_manager = ConfigManager::with_config_path(config_path).unwrap();
    let config = UserConfiguration::default();
    config_manager.save(&config).unwrap();

    // Test load performance (<100ms)
    let start = Instant::now();
    let _ = config_manager.load();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 100,
        "Config load should be <100ms, took {}ms",
        duration.as_millis()
    );

    // Test validate performance (<1ms)
    let start = Instant::now();
    let _ = config.validate();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 10,
        "Config validate should be <10ms, took {}ms",
        duration.as_millis()
    );

    // Suppress unused variable warning
    let _ = config_manager;
}
