// Contract Test: Interactive Configuration UI
// Tests the full-screen configuration management system

use cmdai::config::{InteractiveConfigUI, ConfigurationState, RetentionPolicy, BackendType};
use anyhow::Result;
use tempfile::tempdir;

#[tokio::test]
async fn test_load_default_configuration() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    
    let ui = InteractiveConfigUI::new(config_path.clone()).await?;
    let config = ui.get_current_config();
    
    // Verify default values
    assert_eq!(config.preferred_backend, BackendType::Auto);
    assert!(config.history_enabled);
    assert_eq!(config.retention_policy.max_age_days, Some(90));
    
    Ok(())
}

#[tokio::test]
async fn test_update_backend_configuration() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    
    let mut ui = InteractiveConfigUI::new(config_path.clone()).await?;
    
    // Update backend preference
    ui.set_preferred_backend(BackendType::MLX).await?;
    ui.save_configuration().await?;
    
    // Reload and verify persistence
    let ui2 = InteractiveConfigUI::new(config_path).await?;
    assert_eq!(ui2.get_current_config().preferred_backend, BackendType::MLX);
    
    Ok(())
}

#[tokio::test]
async fn test_validate_configuration_changes() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    
    let mut ui = InteractiveConfigUI::new(config_path).await?;
    
    // Test invalid retention policy
    let invalid_policy = RetentionPolicy {
        max_entries: Some(0), // Invalid: must be > 0
        max_age_days: Some(0), // Invalid: must be > 0
        preserve_favorites: true,
        preserve_frequently_used: true,
    };
    
    let result = ui.set_retention_policy(invalid_policy).await;
    assert!(result.is_err());
    
    // Test valid retention policy
    let valid_policy = RetentionPolicy {
        max_entries: Some(1000),
        max_age_days: Some(30),
        preserve_favorites: true,
        preserve_frequently_used: false,
    };
    
    let result = ui.set_retention_policy(valid_policy).await;
    assert!(result.is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_export_import() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let export_path = temp_dir.path().join("exported_config.toml");
    
    let mut ui = InteractiveConfigUI::new(config_path.clone()).await?;
    
    // Modify configuration
    ui.set_preferred_backend(BackendType::Ollama).await?;
    ui.set_history_enabled(false).await?;
    ui.save_configuration().await?;
    
    // Export configuration
    ui.export_configuration(&export_path).await?;
    
    // Create new UI and import
    let mut ui2 = InteractiveConfigUI::new(temp_dir.path().join("config2.toml")).await?;
    ui2.import_configuration(&export_path).await?;
    
    // Verify imported settings
    let config = ui2.get_current_config();
    assert_eq!(config.preferred_backend, BackendType::Ollama);
    assert!(!config.history_enabled);
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_change_notifications() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    
    let mut ui = InteractiveConfigUI::new(config_path).await?;
    
    // Subscribe to configuration changes
    let mut change_receiver = ui.subscribe_to_changes();
    
    // Make configuration change
    ui.set_streaming_enabled(false).await?;
    
    // Verify notification received
    let change = change_receiver.recv().await?;
    assert_eq!(change.field_name, "streaming_enabled");
    assert_eq!(change.new_value, "false");
    
    Ok(())
}

#[tokio::test]
async fn test_full_screen_ui_rendering() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    
    let ui = InteractiveConfigUI::new(config_path).await?;
    
    // Test UI components render without errors
    let main_menu = ui.render_main_menu();
    assert!(main_menu.contains("Backend Configuration"));
    assert!(main_menu.contains("History Settings"));
    assert!(main_menu.contains("Safety Configuration"));
    
    let backend_menu = ui.render_backend_configuration();
    assert!(backend_menu.contains("Preferred Backend"));
    assert!(backend_menu.contains("Fallback Chain"));
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_validation_rules() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    
    let mut ui = InteractiveConfigUI::new(config_path).await?;
    
    // Test backend availability validation
    use cmdai::config::BackendConfig;
    
    let invalid_backend_config = BackendConfig {
        endpoint: "invalid://url".to_string(), // Invalid URL
        timeout_ms: 0, // Invalid: must be > 0
        max_retries: 100, // Invalid: too high
    };
    
    let result = ui.set_backend_config("vllm", invalid_backend_config).await;
    assert!(result.is_err());
    
    // Test valid backend configuration
    let valid_backend_config = BackendConfig {
        endpoint: "http://localhost:8000".to_string(),
        timeout_ms: 30000,
        max_retries: 3,
    };
    
    let result = ui.set_backend_config("vllm", valid_backend_config).await;
    assert!(result.is_ok());
    
    Ok(())
}