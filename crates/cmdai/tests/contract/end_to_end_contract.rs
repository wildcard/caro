//! Contract Test: End-to-End Command Generation Workflow
//! 
//! This test validates the complete production system integration with
//! all components working together: backend selection, streaming generation,
//! safety validation, history storage, and configuration management.
//! 
//! MUST FAIL: These tests expect implementation of complete production system
//! with all components integrated and working together.

use cmdai::{CmdAI, Config};
use cmdai::models::{CommandRequest, GeneratedCommand, RiskLevel};
use anyhow::Result;
use tempfile::tempdir;
use std::time::Duration;

#[tokio::test]
async fn test_complete_command_generation_workflow() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let history_path = temp_dir.path().join("history.db");
    
    // Initialize complete system
    let config = Config::builder()
        .config_path(config_path)
        .history_path(history_path)
        .preferred_backend("mock")
        .safety_level(cmdai::config::SafetyLevel::Moderate)
        .build()?;
    
    let cmdai = CmdAI::new(config).await?;
    
    // Test complete workflow
    let request = CommandRequest::builder()
        .user_input("list all files in current directory")
        .working_directory("/home/user/documents")
        .shell_type(cmdai::models::ShellType::Bash)
        .build()?;
    
    let start = std::time::Instant::now();
    let result = cmdai.generate_command(request).await?;
    let total_time = start.elapsed();
    
    // Verify constitutional performance requirements
    assert!(total_time.as_millis() < 2500, 
        "Total workflow time {} ms should include <2s inference + overhead", 
        total_time.as_millis());
    
    // Verify complete result
    assert!(!result.command.is_empty());
    assert!(!result.explanation.is_empty());
    assert!(result.confidence > 0.7);
    assert_eq!(result.risk_level, RiskLevel::Safe);
    assert!(result.generation_time < Duration::from_secs(2));
    assert!(result.validation_time < Duration::from_millis(50));
    
    // Verify history was stored
    let history = cmdai.get_recent_history(1).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].command, result.command);
    
    Ok(())
}

#[tokio::test]
async fn test_dangerous_command_workflow_with_confirmation() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let history_path = temp_dir.path().join("history.db");
    
    let config = Config::builder()
        .config_path(config_path)
        .history_path(history_path)
        .preferred_backend("mock")
        .safety_level(cmdai::config::SafetyLevel::Strict)
        .build()?;
    
    let cmdai = CmdAI::new(config).await?;
    
    // Test dangerous command detection
    let request = CommandRequest::builder()
        .user_input("delete all files in this directory")
        .working_directory("/home/user/important")
        .build()?;
    
    let result = cmdai.generate_command(request).await?;
    
    // Should be flagged as dangerous
    assert!(matches!(result.risk_level, RiskLevel::High | RiskLevel::Critical));
    assert!(!result.safety_warnings.is_empty());
    assert!(!result.suggested_alternatives.is_empty());
    assert!(result.requires_confirmation);
    
    // Test confirmation workflow
    let confirmed_result = cmdai.confirm_and_execute_command(
        &result.command, 
        true // User confirms
    ).await?;
    
    assert!(confirmed_result.was_executed);
    assert!(confirmed_result.user_confirmed);
    
    // Verify history includes safety metadata
    let history = cmdai.get_recent_history(1).await?;
    assert!(history[0].safety_metadata.is_some());
    assert!(history[0].safety_metadata.as_ref().unwrap().user_confirmed);
    
    Ok(())
}

#[tokio::test]
async fn test_streaming_generation_with_cancellation() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = Config::builder()
        .history_path(temp_dir.path().join("history.db"))
        .streaming_enabled(true)
        .build()?;
    
    let cmdai = CmdAI::new(config).await?;
    
    let request = CommandRequest::builder()
        .user_input("analyze this large codebase and generate detailed report")
        .working_directory("/home/user/project")
        .build()?;
    
    // Start streaming generation
    let mut stream = cmdai.generate_command_stream(request).await?;
    
    let mut events = Vec::new();
    let mut progress_count = 0;
    
    // Collect events until we see some progress
    while let Some(event) = stream.next().await {
        events.push(event.clone());
        
        if let cmdai::streaming::StreamEvent::Progress { .. } = event {
            progress_count += 1;
            
            // Cancel after seeing some progress
            if progress_count >= 2 {
                stream.cancel().await?;
                break;
            }
        }
        
        // Safety break
        if events.len() > 100 {
            break;
        }
    }
    
    // Verify we got progress events and cancellation worked
    assert!(progress_count >= 2);
    assert!(events.iter().any(|e| matches!(e, cmdai::streaming::StreamEvent::Progress { .. })));
    
    // Should not have completed normally
    assert!(!events.iter().any(|e| matches!(e, cmdai::streaming::StreamEvent::Completed { .. })));
    
    Ok(())
}

#[tokio::test]
async fn test_backend_fallback_mechanism() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = Config::builder()
        .history_path(temp_dir.path().join("history.db"))
        .preferred_backend("unavailable_backend")
        .fallback_chain(vec!["another_unavailable".to_string(), "mock".to_string()])
        .build()?;
    
    let cmdai = CmdAI::new(config).await?;
    
    let request = CommandRequest::builder()
        .user_input("test fallback mechanism")
        .working_directory("/home/user")
        .build()?;
    
    // Should fall back to mock backend
    let result = cmdai.generate_command(request).await?;
    
    assert!(!result.command.is_empty());
    assert_eq!(result.backend_used, "mock");
    
    // Verify fallback was logged
    let metadata = result.generation_metadata.unwrap();
    assert!(metadata.fallback_occurred);
    assert!(!metadata.attempted_backends.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_semantic_search_integration() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = Config::builder()
        .history_path(temp_dir.path().join("history.db"))
        .semantic_search_enabled(true)
        .build()?;
    
    let cmdai = CmdAI::new(config).await?;
    
    // Generate and store some commands
    let test_commands = vec![
        "find all Python files in project",
        "count lines of code in repository", 
        "search for TODO comments",
        "list running Docker containers",
    ];
    
    for input in test_commands {
        let request = CommandRequest::builder()
            .user_input(input)
            .working_directory("/home/user/project")
            .build()?;
        
        let _result = cmdai.generate_command(request).await?;
    }
    
    // Test semantic search
    let search_results = cmdai.search_history_semantic("code analysis").await?;
    
    assert!(!search_results.is_empty());
    
    // Should find relevant commands based on semantic similarity
    let relevant_found = search_results.iter().any(|result| {
        result.entry.user_input.as_ref().map_or(false, |input| {
            input.contains("Python") || input.contains("lines") || input.contains("TODO")
        })
    });
    
    assert!(relevant_found, "Semantic search should find code-related commands");
    
    // Verify search performance
    let start = std::time::Instant::now();
    let _results = cmdai.search_history_semantic("container management").await?;
    let search_time = start.elapsed();
    
    assert!(search_time.as_millis() < 50, 
        "Semantic search {} ms exceeds constitutional requirement", 
        search_time.as_millis());
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_persistence_and_updates() -> Result<()> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    
    // Create initial configuration
    {
        let config = Config::builder()
            .config_path(config_path.clone())
            .preferred_backend("mock")
            .safety_level(cmdai::config::SafetyLevel::Moderate)
            .build()?;
        
        let cmdai = CmdAI::new(config).await?;
        
        // Update configuration
        cmdai.update_config()
            .set_preferred_backend("mlx")
            .set_safety_level(cmdai::config::SafetyLevel::Strict)
            .set_streaming_enabled(false)
            .apply().await?;
    }
    
    // Reload and verify persistence
    {
        let config = Config::from_file(&config_path)?;
        let cmdai = CmdAI::new(config).await?;
        
        let current_config = cmdai.get_current_config().await?;
        assert_eq!(current_config.preferred_backend, "mlx");
        assert_eq!(current_config.safety_level, cmdai::config::SafetyLevel::Strict);
        assert!(!current_config.streaming_enabled);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_command_generation() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = Config::builder()
        .history_path(temp_dir.path().join("history.db"))
        .preferred_backend("mock")
        .build()?;
    
    let cmdai = std::sync::Arc::new(CmdAI::new(config).await?);
    
    // Generate commands concurrently
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let cmdai_clone = cmdai.clone();
        let handle = tokio::spawn(async move {
            let request = CommandRequest::builder()
                .user_input(&format!("concurrent test {}", i))
                .working_directory("/home/user")
                .build()
                .unwrap();
            
            let result = cmdai_clone.generate_command(request).await.unwrap();
            
            // Verify result quality
            assert!(!result.command.is_empty());
            assert!(!result.explanation.is_empty());
            assert!(result.confidence > 0.5);
            
            result.command
        });
        handles.push(handle);
    }
    
    // Wait for all generations to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await?;
        results.push(result);
    }
    
    // Verify all succeeded
    assert_eq!(results.len(), 10);
    
    // Verify history stored all commands
    let history = cmdai.get_recent_history(10).await?;
    assert_eq!(history.len(), 10);
    
    Ok(())
}

#[tokio::test]
async fn test_performance_monitoring_integration() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = Config::builder()
        .history_path(temp_dir.path().join("history.db"))
        .performance_monitoring_enabled(true)
        .build()?;
    
    let cmdai = CmdAI::new(config).await?;
    
    // Generate several commands to build performance data
    for i in 0..5 {
        let request = CommandRequest::builder()
            .user_input(&format!("performance test {}", i))
            .working_directory("/home/user")
            .build()?;
        
        let _result = cmdai.generate_command(request).await?;
    }
    
    // Check performance metrics
    let metrics = cmdai.get_performance_metrics().await?;
    
    assert!(metrics.total_commands_generated >= 5);
    assert!(metrics.average_generation_time > Duration::from_millis(0));
    assert!(metrics.average_validation_time > Duration::from_millis(0));
    assert_eq!(metrics.constitutional_violations, 0); // Should have no violations
    
    // Verify constitutional compliance tracking
    assert!(metrics.startup_time_compliant);
    assert!(metrics.inference_time_compliant);
    assert!(metrics.validation_time_compliant);
    assert!(metrics.history_write_compliant);
    
    Ok(())
}

#[tokio::test]
async fn test_error_recovery_and_graceful_degradation() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = Config::builder()
        .history_path(temp_dir.path().join("history.db"))
        .preferred_backend("error_backend") // Backend that will fail
        .fallback_chain(vec!["mock".to_string()])
        .build()?;
    
    let cmdai = CmdAI::new(config).await?;
    
    let request = CommandRequest::builder()
        .user_input("test error recovery")
        .working_directory("/home/user")
        .build()?;
    
    // Should recover gracefully using fallback
    let result = cmdai.generate_command(request).await?;
    
    assert!(!result.command.is_empty());
    assert_eq!(result.backend_used, "mock");
    
    // Verify error was logged but didn't break the system
    let error_logs = cmdai.get_recent_error_logs().await?;
    assert!(!error_logs.is_empty());
    assert!(error_logs[0].contains("error_backend") || error_logs[0].contains("fallback"));
    
    Ok(())
}

#[tokio::test]
async fn test_constitutional_compliance_validation() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = Config::builder()
        .history_path(temp_dir.path().join("history.db"))
        .constitutional_compliance_checking(true)
        .build()?;
    
    let cmdai = CmdAI::new(config).await?;
    
    // Test all constitutional requirements in one workflow
    let request = CommandRequest::builder()
        .user_input("comprehensive constitutional test")
        .working_directory("/home/user")
        .build()?;
    
    let overall_start = std::time::Instant::now();
    let result = cmdai.generate_command(request).await?;
    let total_time = overall_start.elapsed();
    
    // Constitutional requirement validation
    assert!(result.generation_time < Duration::from_secs(2), 
        "Inference time {} ms violates constitutional requirement", 
        result.generation_time.as_millis());
    
    assert!(result.validation_time < Duration::from_millis(50), 
        "Validation time {} ms violates constitutional requirement", 
        result.validation_time.as_millis());
    
    // Test history write performance
    let history_start = std::time::Instant::now();
    let _history = cmdai.get_recent_history(1).await?;
    let history_time = history_start.elapsed();
    
    assert!(history_time.as_millis() < 10, 
        "History read time {} ms violates constitutional requirement", 
        history_time.as_millis());
    
    // Verify constitutional compliance report
    let compliance_report = cmdai.get_constitutional_compliance_report().await?;
    assert!(compliance_report.simplicity_compliant);
    assert!(compliance_report.library_first_compliant);
    assert!(compliance_report.test_first_compliant);
    assert!(compliance_report.safety_first_compliant);
    assert!(compliance_report.observability_compliant);
    
    Ok(())
}