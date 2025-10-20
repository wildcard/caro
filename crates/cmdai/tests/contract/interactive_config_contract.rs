//! Contract Test: Interactive Configuration UI
//! 
//! This test validates the full-screen configuration management system
//! with real-time validation, TOML persistence, and immediate updates.
//! 
//! MUST FAIL: These tests expect implementation of production config system
//! that provides interactive UI with dialoguer and crossterm.

use cmdai::config::{InteractiveConfigUI, ConfigurationState, RetentionPolicy, BackendType, SafetyLevel, PrivacyLevel, VerbosityLevel};
use anyhow::Result;
use tempfile::tempdir;
use std::collections::HashMap;

#[tokio::test]
async fn test_interactive_config_initialization() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    
    // Initialize with default configuration
    let ui = InteractiveConfigUI::new(config_path.clone()).await?;
    let config = ui.get_current_config();
    
    // Verify default values match constitutional requirements
    assert_eq!(config.preferred_backend, BackendType::Auto);
    assert!(config.history_enabled);
    assert_eq!(config.safety_level, SafetyLevel::Moderate);
    assert_eq!(config.privacy_mode, PrivacyLevel::Standard);
    assert!(config.streaming_enabled);
    
    // Verify persistence path is correct
    assert_eq!(ui.get_config_path(), config_path);
    
    Ok(())
}

#[tokio::test]
async fn test_full_screen_ui_rendering() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let ui = InteractiveConfigUI::new(config_path).await?;
    
    // Test main menu rendering
    let main_menu = ui.render_main_menu()?;
    assert!(main_menu.contains("Backend Configuration"));
    assert!(main_menu.contains("History Settings"));
    assert!(main_menu.contains("Safety Configuration"));
    assert!(main_menu.contains("Streaming Options"));
    assert!(main_menu.contains("Privacy Settings"));
    
    // Test backend configuration menu
    let backend_menu = ui.render_backend_configuration()?;
    assert!(backend_menu.contains("Preferred Backend"));
    assert!(backend_menu.contains("Fallback Chain"));
    assert!(backend_menu.contains("Performance Monitoring"));
    
    // Test history settings menu
    let history_menu = ui.render_history_configuration()?;
    assert!(history_menu.contains("Enable History"));
    assert!(history_menu.contains("Retention Policy"));
    assert!(history_menu.contains("Auto Cleanup"));
    
    // Test safety configuration menu
    let safety_menu = ui.render_safety_configuration()?;
    assert!(safety_menu.contains("Safety Level"));
    assert!(safety_menu.contains("Confirmation Required"));
    assert!(safety_menu.contains("Custom Patterns"));
    
    Ok(())
}

#[tokio::test]
async fn test_backend_configuration_management() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let mut ui = InteractiveConfigUI::new(config_path.clone()).await?;
    
    // Test backend preference updates
    ui.set_preferred_backend(BackendType::MLX).await?;
    assert_eq!(ui.get_current_config().preferred_backend, BackendType::MLX);
    
    // Test fallback chain configuration
    let fallback_chain = vec![BackendType::MLX, BackendType::Ollama, BackendType::Mock];
    ui.set_fallback_chain(fallback_chain.clone()).await?;
    assert_eq!(ui.get_current_config().fallback_chain, fallback_chain);
    
    // Test backend-specific configuration
    let backend_config = cmdai::config::BackendConfig {
        endpoint: "http://localhost:11434".to_string(),
        timeout_ms: 30000,
        max_retries: 3,
        headers: HashMap::new(),
    };
    ui.set_backend_config("ollama", backend_config.clone()).await?;
    
    let stored_config = ui.get_backend_config("ollama").await?;
    assert!(stored_config.is_some());
    assert_eq!(stored_config.unwrap().endpoint, backend_config.endpoint);
    
    Ok(())
}

#[tokio::test]
async fn test_history_configuration_with_validation() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let mut ui = InteractiveConfigUI::new(config_path).await?;
    
    // Test history enable/disable
    ui.set_history_enabled(false).await?;
    assert!(!ui.get_current_config().history_enabled);
    
    ui.set_history_enabled(true).await?;
    assert!(ui.get_current_config().history_enabled);
    
    // Test retention policy configuration with validation
    let retention_policy = RetentionPolicy {
        max_entries: Some(1000),
        max_age_days: Some(30),
        preserve_favorites: true,
        preserve_frequently_used: true,
    };
    ui.set_retention_policy(retention_policy.clone()).await?;
    assert_eq!(ui.get_current_config().retention_policy.max_entries, retention_policy.max_entries);
    
    // Test invalid retention policy (should fail validation)
    let invalid_policy = RetentionPolicy {
        max_entries: Some(0), // Invalid: must be > 0
        max_age_days: Some(0), // Invalid: must be > 0
        preserve_favorites: true,
        preserve_frequently_used: true,
    };
    let result = ui.set_retention_policy(invalid_policy).await;
    assert!(result.is_err());
    
    // Test auto cleanup configuration
    ui.set_auto_cleanup_days(90).await?;
    assert_eq!(ui.get_current_config().auto_cleanup_days, 90);
    
    Ok(())
}

#[tokio::test]
async fn test_safety_configuration_levels() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let mut ui = InteractiveConfigUI::new(config_path).await?;
    
    // Test safety level configuration
    ui.set_safety_level(SafetyLevel::Strict).await?;
    assert_eq!(ui.get_current_config().safety_level, SafetyLevel::Strict);
    
    // Test confirmation requirements
    let confirmation_levels = vec![cmdai::models::RiskLevel::High, cmdai::models::RiskLevel::Critical];
    ui.set_confirmation_required(confirmation_levels.clone()).await?;
    assert_eq!(ui.get_current_config().confirmation_required, confirmation_levels);
    
    // Test custom safety patterns
    let custom_patterns = vec![
        r"rm\s+-rf\s+/".to_string(),
        r"sudo\s+rm".to_string(),
    ];
    ui.set_custom_safety_patterns(custom_patterns.clone()).await?;
    assert_eq!(ui.get_current_config().custom_safety_patterns, custom_patterns);
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_persistence() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    
    // Create and configure UI
    {
        let mut ui = InteractiveConfigUI::new(config_path.clone()).await?;
        ui.set_preferred_backend(BackendType::Ollama).await?;
        ui.set_history_enabled(false).await?;
        ui.set_safety_level(SafetyLevel::Strict).await?;
        ui.save_configuration().await?;
    }
    
    // Reload and verify persistence
    {
        let ui = InteractiveConfigUI::new(config_path.clone()).await?;
        let config = ui.get_current_config();
        assert_eq!(config.preferred_backend, BackendType::Ollama);
        assert!(!config.history_enabled);
        assert_eq!(config.safety_level, SafetyLevel::Strict);
    }
    
    // Verify TOML file exists and is readable
    assert!(config_path.exists());
    let toml_content = std::fs::read_to_string(&config_path)?;
    assert!(toml_content.contains("preferred_backend"));
    assert!(toml_content.contains("history_enabled"));
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_export_import() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let export_path = temp_dir.path().join("exported_config.toml");
    
    // Configure and export
    {
        let mut ui = InteractiveConfigUI::new(config_path.clone()).await?;
        ui.set_preferred_backend(BackendType::vLLM).await?;
        ui.set_streaming_enabled(false).await?;
        ui.set_privacy_mode(PrivacyLevel::Enhanced).await?;
        ui.save_configuration().await?;
        
        ui.export_configuration(&export_path).await?;
    }
    
    // Import into new configuration
    {
        let config_path2 = temp_dir.path().join("config2.toml");
        let mut ui2 = InteractiveConfigUI::new(config_path2).await?;
        ui2.import_configuration(&export_path).await?;
        
        let config = ui2.get_current_config();
        assert_eq!(config.preferred_backend, BackendType::vLLM);
        assert!(!config.streaming_enabled);
        assert_eq!(config.privacy_mode, PrivacyLevel::Enhanced);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_real_time_configuration_updates() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let mut ui = InteractiveConfigUI::new(config_path).await?;
    
    // Subscribe to configuration changes
    let mut change_receiver = ui.subscribe_to_changes().await?;
    
    // Make configuration change in background task
    let ui_clone = ui.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        ui_clone.set_streaming_enabled(false).await.unwrap();
    });
    
    // Verify notification received
    let change = tokio::time::timeout(
        tokio::time::Duration::from_secs(1), 
        change_receiver.recv()
    ).await??;
    
    assert_eq!(change.field_name, "streaming_enabled");
    assert_eq!(change.old_value, "true");
    assert_eq!(change.new_value, "false");
    assert!(change.timestamp <= chrono::Utc::now());
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_validation_rules() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let mut ui = InteractiveConfigUI::new(config_path).await?;
    
    // Test backend configuration validation
    let invalid_backend_config = cmdai::config::BackendConfig {
        endpoint: "invalid://url".to_string(), // Invalid URL
        timeout_ms: 0, // Invalid: must be > 0
        max_retries: 100, // Invalid: too high
        headers: HashMap::new(),
    };
    
    let result = ui.set_backend_config("test", invalid_backend_config).await;
    assert!(result.is_err());
    
    // Test valid backend configuration
    let valid_backend_config = cmdai::config::BackendConfig {
        endpoint: "http://localhost:8000".to_string(),
        timeout_ms: 30000,
        max_retries: 3,
        headers: HashMap::new(),
    };
    
    let result = ui.set_backend_config("test", valid_backend_config).await;
    assert!(result.is_ok());
    
    // Test retention policy validation
    let invalid_retention = RetentionPolicy {
        max_entries: Some(0), // Invalid
        max_age_days: Some(0), // Invalid
        preserve_favorites: true,
        preserve_frequently_used: true,
    };
    
    let result = ui.set_retention_policy(invalid_retention).await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_ui_navigation() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let ui = InteractiveConfigUI::new(config_path).await?;
    
    // Test menu navigation structure
    let navigation = ui.get_navigation_structure()?;
    assert_eq!(navigation.main_menu.len(), 5); // Backend, History, Safety, Streaming, Privacy
    
    // Test each submenu has appropriate options
    let backend_options = ui.get_backend_configuration_options()?;
    assert!(backend_options.contains(&"Preferred Backend".to_string()));
    assert!(backend_options.contains(&"Fallback Chain".to_string()));
    
    let history_options = ui.get_history_configuration_options()?;
    assert!(history_options.contains(&"Enable History".to_string()));
    assert!(history_options.contains(&"Retention Policy".to_string()));
    
    let safety_options = ui.get_safety_configuration_options()?;
    assert!(safety_options.contains(&"Safety Level".to_string()));
    assert!(safety_options.contains(&"Custom Patterns".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_performance() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let mut ui = InteractiveConfigUI::new(config_path).await?;
    
    // Test configuration load time
    let start = std::time::Instant::now();
    let _config = ui.get_current_config();
    let load_duration = start.elapsed();
    assert!(load_duration.as_millis() < 10, "Configuration load too slow");
    
    // Test configuration save time
    let start = std::time::Instant::now();
    ui.set_preferred_backend(BackendType::Mock).await?;
    let save_duration = start.elapsed();
    assert!(save_duration.as_millis() < 50, "Configuration save too slow");
    
    // Test UI rendering performance
    let start = std::time::Instant::now();
    let _menu = ui.render_main_menu()?;
    let render_duration = start.elapsed();
    assert!(render_duration.as_millis() < 100, "UI rendering too slow");
    
    Ok(())
}