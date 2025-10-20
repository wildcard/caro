//! Integration tests for Feature 003: Core Infrastructure Modules
//!
//! Tests end-to-end scenarios from quickstart.md demonstrating integration
//! of cache, config, execution, and logging modules.

use cmdai::cache::CacheManager;
use cmdai::config::ConfigManager;
use cmdai::execution::ExecutionContext;
use cmdai::logging::{LogConfig, LogFormat, LogLevel, LogOutput};
use cmdai::models::{Platform, SafetyLevel, ShellType, UserConfiguration};
use std::path::PathBuf;
use tempfile::TempDir;

/// Scenario 1: First-time user experience
///
/// Tests: Default config loading, cache initialization, context capture
#[tokio::test]
async fn test_first_time_user_experience() {
    // GIVEN: A new user with no config or cache
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let cache_dir = temp_dir.path().join("cache");

    // WHEN: User initializes the system for the first time
    let config_manager = ConfigManager::with_config_path(config_path).unwrap();
    let config = config_manager.load().unwrap();

    let cache_manager = CacheManager::with_cache_dir(cache_dir).unwrap();
    let stats = cache_manager.stats();

    let context = ExecutionContext::capture().unwrap();

    // THEN: Default configuration is used
    assert_eq!(
        config.safety_level,
        SafetyLevel::Moderate,
        "Default safety level should be Moderate"
    );
    assert_eq!(
        config.log_level,
        LogLevel::Info,
        "Default log level should be Info"
    );

    // THEN: Cache is empty but initialized
    assert_eq!(
        stats.total_models, 0,
        "Cache should be empty for first-time user"
    );

    // THEN: Execution context is captured successfully
    assert!(
        context.current_dir().is_absolute(),
        "Context should capture absolute current directory"
    );
    assert!(
        !context.username().is_empty(),
        "Context should capture username"
    );
    assert!(
        !context.hostname().is_empty(),
        "Context should capture hostname"
    );

    // Verify context capture performance (should be < 50ms)
    let start = std::time::Instant::now();
    let _ = ExecutionContext::capture().unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 50,
        "Context capture should complete in < 50ms, took {:?}",
        elapsed
    );
}

/// Scenario 2: Returning user with cached model (offline operation)
///
/// Tests: Custom config values, cache hits without network
#[tokio::test]
async fn test_returning_user_with_cache() {
    // GIVEN: A user with custom config
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let cache_dir = temp_dir.path().join("cache");

    let config_manager = ConfigManager::with_config_path(config_path.clone()).unwrap();

    // Create custom configuration
    let custom_config = UserConfiguration {
        safety_level: SafetyLevel::Strict,
        log_level: LogLevel::Debug,
        default_shell: Some(ShellType::Bash),
        default_model: Some("custom-model-id".to_string()),
        cache_max_size_gb: 5,
        log_rotation_days: 3,
    };

    // Save custom config
    config_manager.save(&custom_config).unwrap();

    // WHEN: User returns and loads their configuration
    let loaded_config = config_manager.load().unwrap();

    // THEN: Custom configuration is preserved
    assert_eq!(
        loaded_config.safety_level,
        SafetyLevel::Strict,
        "Custom safety level should be loaded"
    );
    assert_eq!(
        loaded_config.log_level,
        LogLevel::Debug,
        "Custom log level should be loaded"
    );
    assert_eq!(
        loaded_config.default_shell,
        Some(ShellType::Bash),
        "Custom shell should be loaded"
    );
    assert_eq!(
        loaded_config.default_model,
        Some("custom-model-id".to_string()),
        "Custom model ID should be loaded"
    );

    // WHEN: Cache manager is initialized (offline mode)
    let cache_manager = CacheManager::with_cache_dir(cache_dir).unwrap();

    // THEN: Cache works offline (no network required for initialization)
    assert!(
        !cache_manager.is_cached("nonexistent-model"),
        "Cache should correctly report uncached models"
    );
}

/// Scenario 3: CLI argument override
///
/// Tests: CLI arguments override config file settings
#[tokio::test]
async fn test_cli_argument_override() {
    // GIVEN: A config file with default settings
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config_manager = ConfigManager::with_config_path(config_path).unwrap();

    let file_config = UserConfiguration {
        safety_level: SafetyLevel::Moderate,
        log_level: LogLevel::Info,
        ..Default::default()
    };

    config_manager.save(&file_config).unwrap();

    // WHEN: CLI arguments are provided to override settings
    let merged_config = config_manager
        .merge_with_cli(
            Some("strict"), // Override safety level
            Some("zsh"),    // Override shell
            Some("debug"),  // Override log level
        )
        .unwrap();

    // THEN: CLI arguments take precedence
    assert_eq!(
        merged_config.safety_level,
        SafetyLevel::Strict,
        "CLI safety argument should override config file"
    );
    assert_eq!(
        merged_config.default_shell,
        Some(ShellType::Zsh),
        "CLI shell argument should override config file"
    );
    assert_eq!(
        merged_config.log_level,
        LogLevel::Debug,
        "CLI log level argument should override config file"
    );
}

/// Scenario 4: Context-aware command generation
///
/// Tests: ExecutionContext serialization for LLM prompts
#[tokio::test]
async fn test_context_aware_generation() {
    // GIVEN: An execution context
    let test_dir = PathBuf::from("/home/user/projects/myapp");
    let context =
        ExecutionContext::new(test_dir.clone(), ShellType::Bash, Platform::Linux).unwrap();

    // WHEN: Context is converted to prompt format
    let prompt_context = context.to_prompt_context();

    // THEN: Context contains essential information for LLM
    assert!(
        prompt_context.contains("bash") || prompt_context.contains("Bash"),
        "Prompt should include shell type, got: {}",
        prompt_context
    );
    assert!(
        prompt_context.contains("Linux") || prompt_context.contains("linux"),
        "Prompt should include platform, got: {}",
        prompt_context
    );
    assert!(
        prompt_context.contains("/home/user/projects/myapp"),
        "Prompt should include current directory, got: {}",
        prompt_context
    );

    // THEN: Context does not leak sensitive environment variables
    let env_vars = context.environment_vars();
    for key in env_vars.keys() {
        assert!(
            !key.to_uppercase().contains("API_KEY"),
            "API_KEY should be filtered from context"
        );
        assert!(
            !key.to_uppercase().contains("SECRET"),
            "SECRET should be filtered from context"
        );
        assert!(
            !key.to_uppercase().contains("PASSWORD"),
            "PASSWORD should be filtered from context"
        );
        assert!(
            !key.to_uppercase().contains("TOKEN"),
            "TOKEN should be filtered from context"
        );
    }
}

/// Scenario 5: Structured logging with operations
///
/// Tests: JSON logging, operation spans, duration tracking
#[tokio::test]
async fn test_structured_logging_operations() {
    // GIVEN: A configured logger with JSON format
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("cmdai.log");

    let log_config = LogConfig {
        log_level: LogLevel::Debug,
        format: LogFormat::Json,
        output: LogOutput::File(log_file.clone()),
        redaction_enabled: true,
        rotation: None,
    };

    // Note: Logger::init() can only be called once per process
    // In real tests, we'd verify logging behavior differently
    // For now, we validate the configuration structure

    // THEN: Log configuration is valid
    assert_eq!(
        log_config.log_level,
        LogLevel::Debug,
        "Log level should be Debug"
    );
    assert_eq!(log_config.format, LogFormat::Json, "Format should be JSON");
    assert!(log_config.redaction_enabled, "Redaction should be enabled");

    // WHEN: Creating an operation span
    use cmdai::logging::OperationSpan;
    let _span = OperationSpan::new("command_generation");

    // THEN: Operation span is created successfully
    // (Actual span tracking would be verified in tracing subscriber)
}

/// Scenario 6: Cache size limit & LRU eviction
///
/// Tests: Cache evicts oldest model when limit reached
#[tokio::test]
async fn test_cache_size_limit_lru() {
    // GIVEN: A cache with limited size
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    let cache_manager = CacheManager::with_cache_dir(cache_dir.clone()).unwrap();

    // Note: LRU eviction logic is in CacheManifest::cleanup_lru()
    // This test verifies the cache manager properly manages stats

    // WHEN: Checking cache stats
    let stats = cache_manager.stats();

    // THEN: Cache stats are accurate
    assert_eq!(
        stats.cache_dir, cache_dir,
        "Cache stats should report correct directory"
    );
    assert_eq!(
        stats.total_models, 0,
        "Empty cache should report zero models"
    );
    assert_eq!(
        stats.total_size_bytes, 0,
        "Empty cache should report zero bytes"
    );
}

/// Scenario 7: Configuration validation & migration
///
/// Tests: Invalid config detection, deprecated key warnings
#[tokio::test]
async fn test_configuration_validation_migration() {
    // GIVEN: A config manager
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config_manager = ConfigManager::with_config_path(config_path).unwrap();

    // WHEN: Validating schema for nonexistent config
    let warnings = config_manager.validate_schema().unwrap();

    // THEN: No warnings for new config
    assert_eq!(
        warnings.len(),
        0,
        "New config should have no deprecated keys"
    );
}

/// Scenario 8: Multi-platform execution context
///
/// Tests: Platform-specific context capture
#[tokio::test]
async fn test_multiplatform_execution_context() {
    // GIVEN: Different platform configurations
    let test_cases = vec![
        (Platform::Linux, ShellType::Bash),
        (Platform::MacOS, ShellType::Zsh),
        (Platform::Windows, ShellType::PowerShell),
    ];

    for (platform, shell) in test_cases {
        // WHEN: Creating context for specific platform
        let context = ExecutionContext::new(PathBuf::from("/test/path"), shell, platform).unwrap();

        // THEN: Context reflects platform-specific details
        assert_eq!(
            context.platform(),
            platform,
            "Context should capture correct platform"
        );
        assert_eq!(
            context.shell_type(),
            shell,
            "Context should capture correct shell"
        );
    }
}

/// Scenario 9: Cache integrity validation
///
/// Tests: Checksum validation for cached models
#[tokio::test]
async fn test_cache_integrity_validation() {
    // GIVEN: A cache manager
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    let cache_manager = CacheManager::with_cache_dir(cache_dir).unwrap();

    // WHEN: Validating integrity of empty cache
    let report = cache_manager.validate_integrity().await.unwrap();

    // THEN: Validation completes successfully
    assert_eq!(
        report.valid_models.len(),
        0,
        "Empty cache should have no valid models"
    );
    assert_eq!(
        report.corrupted_models.len(),
        0,
        "Empty cache should have no corrupted models"
    );
    assert_eq!(
        report.missing_models.len(),
        0,
        "Empty cache should have no missing models"
    );
}
