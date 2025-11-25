//! Contract Test: Advanced Safety Validator
//! 
//! This test validates the multi-modal safety validation system with
//! behavioral analysis, pattern matching, and context-aware risk assessment.
//! 
//! MUST FAIL: These tests expect implementation of production safety system
//! that provides comprehensive validation beyond basic patterns.

use cmdai::safety::{AdvancedSafetyValidator, ValidationResult, PatternEngine, BehavioralAnalyzer, ContextAnalyzer, RiskAssessor};
use cmdai::models::RiskLevel;
use anyhow::Result;
use std::time::Duration;

#[tokio::test]
async fn test_advanced_safety_validator_initialization() -> Result<()> {
    // Initialize with production safety patterns
    let validator = AdvancedSafetyValidator::new().await?;
    
    // Verify all components initialized
    assert!(validator.has_pattern_engine());
    assert!(validator.has_behavioral_analyzer());
    assert!(validator.has_context_analyzer());
    assert!(validator.has_risk_assessor());
    
    // Verify pattern loading
    let pattern_count = validator.get_loaded_pattern_count();
    assert!(pattern_count >= 50, "Should load comprehensive safety patterns");
    
    Ok(())
}

#[tokio::test]
async fn test_dangerous_command_pattern_detection() -> Result<()> {
    let validator = AdvancedSafetyValidator::new().await?;
    
    // Test filesystem destruction patterns
    let dangerous_commands = vec![
        "rm -rf /",
        "rm -rf ~/",
        "rm -rf /*",
        "sudo rm -rf /home",
        "find / -delete",
        "rm -rf $HOME",
    ];
    
    for cmd in dangerous_commands {
        let start = std::time::Instant::now();
        let result = validator.validate(cmd, "/home/user", None).await?;
        let duration = start.elapsed();
        
        // Verify constitutional performance requirement (<50ms)
        assert!(duration.as_millis() < 50, 
            "Safety validation {} ms exceeds constitutional requirement", 
            duration.as_millis());
        
        // Verify high/critical risk detection
        assert!(!result.is_safe, "Command '{}' should be flagged as unsafe", cmd);
        assert!(matches!(result.risk_level, RiskLevel::High | RiskLevel::Critical));
        assert!(result.confidence > 0.8);
        assert!(!result.explanation.is_empty());
        assert!(!result.required_confirmations.is_empty());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_system_modification_detection() -> Result<()> {
    let validator = AdvancedSafetyValidator::new().await?;
    
    let system_commands = vec![
        "chmod 777 /etc/passwd",
        "chown root:root /bin/bash",
        "mv /etc/passwd /tmp/",
        "cp /etc/shadow /tmp/backup",
        "dd if=/dev/zero of=/dev/sda",
        "mkfs.ext4 /dev/sda1",
        "fdisk /dev/sda",
    ];
    
    for cmd in system_commands {
        let result = validator.validate(cmd, "/", None).await?;
        
        assert!(!result.is_safe, "System command '{}' should be flagged", cmd);
        assert!(matches!(result.risk_level, RiskLevel::High | RiskLevel::Critical));
        assert!(result.patterns_matched.contains(&"system_modification".to_string()));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_behavioral_analysis_with_command_sequences() -> Result<()> {
    let validator = AdvancedSafetyValidator::new().await?;
    
    // Simulate behavioral pattern that could be suspicious
    let command_sequence = vec![
        "ps aux | grep ssh",
        "netstat -tulpn | grep :22",
        "cat /etc/passwd",
        "ls -la /home",
        "find /home -name '*.key'",
    ];
    
    // Each command might be safe individually, but sequence shows pattern
    let mut previous_commands = Vec::new();
    
    for cmd in command_sequence {
        let result = validator.validate_with_context(
            cmd, 
            "/home/user", 
            Some(&previous_commands)
        ).await?;
        
        previous_commands.push(cmd.to_string());
        
        // As sequence builds, risk should increase
        if previous_commands.len() >= 3 {
            assert!(matches!(result.risk_level, RiskLevel::Moderate | RiskLevel::High));
            assert!(result.behavioral_flags.contains(&"information_gathering".to_string()));
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_context_aware_risk_assessment() -> Result<()> {
    let validator = AdvancedSafetyValidator::new().await?;
    
    let command = "rm *.log";
    
    // Same command, different contexts should yield different risk levels
    let contexts = vec![
        ("/tmp", RiskLevel::Safe), // Safe in temp directory
        ("/var/log", RiskLevel::Moderate), // Moderate in system logs
        ("/", RiskLevel::High), // High risk in root
        ("/etc", RiskLevel::Critical), // Critical in system config
    ];
    
    for (working_dir, expected_min_risk) in contexts {
        let result = validator.validate(command, working_dir, None).await?;
        
        // Risk level should be at least the expected minimum
        let risk_value = match result.risk_level {
            RiskLevel::Safe => 0,
            RiskLevel::Moderate => 1,
            RiskLevel::High => 2,
            RiskLevel::Critical => 3,
        };
        
        let expected_value = match expected_min_risk {
            RiskLevel::Safe => 0,
            RiskLevel::Moderate => 1,
            RiskLevel::High => 2,
            RiskLevel::Critical => 3,
        };
        
        assert!(risk_value >= expected_value, 
            "Risk level in {} should be at least {:?}, got {:?}", 
            working_dir, expected_min_risk, result.risk_level);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_safe_command_validation() -> Result<()> {
    let validator = AdvancedSafetyValidator::new().await?;
    
    let safe_commands = vec![
        "ls -la",
        "cat README.md",
        "echo 'hello world'",
        "grep -r 'pattern' src/",
        "find . -name '*.rs'",
        "ps aux",
        "df -h",
        "du -sh *",
        "git status",
        "cargo build",
    ];
    
    for cmd in safe_commands {
        let result = validator.validate(cmd, "/home/user/project", None).await?;
        
        assert!(result.is_safe, "Command '{}' should be safe", cmd);
        assert_eq!(result.risk_level, RiskLevel::Safe);
        assert!(result.confidence > 0.8);
        assert!(result.required_confirmations.is_empty());
        assert!(result.suggested_alternatives.is_empty());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_custom_safety_patterns() -> Result<()> {
    let mut validator = AdvancedSafetyValidator::new().await?;
    
    // Add custom safety patterns
    let custom_patterns = vec![
        r"curl.*\|\s*bash".to_string(), // Dangerous pipe to bash
        r"wget.*\|\s*sh".to_string(),   // Dangerous pipe to sh
        r"docker.*--privileged".to_string(), // Privileged containers
    ];
    
    validator.add_custom_patterns(custom_patterns).await?;
    
    // Test custom pattern detection
    let test_commands = vec![
        ("curl https://example.com/script.sh | bash", true),
        ("wget https://example.com/install.sh | sh", true),
        ("docker run --privileged ubuntu", true),
        ("curl https://example.com/data.json", false),
    ];
    
    for (cmd, should_match) in test_commands {
        let result = validator.validate(cmd, "/home/user", None).await?;
        
        if should_match {
            assert!(!result.is_safe, "Custom pattern should detect '{}'", cmd);
            assert!(matches!(result.risk_level, RiskLevel::High | RiskLevel::Critical));
        } else {
            assert!(result.is_safe || result.risk_level == RiskLevel::Moderate, 
                "Safe command '{}' should not trigger custom patterns", cmd);
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_suggested_alternatives() -> Result<()> {
    let validator = AdvancedSafetyValidator::new().await?;
    
    let commands_with_alternatives = vec![
        ("rm -rf *", vec!["rm -i *", "trash *", "mv * ~/.trash/"]),
        ("chmod 777 file.txt", vec!["chmod 644 file.txt", "chmod 755 file.txt"]),
        ("sudo su", vec!["sudo -i", "su -"]),
    ];
    
    for (dangerous_cmd, expected_alternatives) in commands_with_alternatives {
        let result = validator.validate(dangerous_cmd, "/home/user", None).await?;
        
        assert!(!result.is_safe);
        assert!(!result.suggested_alternatives.is_empty());
        
        // Check that suggested alternatives are actually safer
        for alternative in &result.suggested_alternatives {
            let alt_result = validator.validate(alternative, "/home/user", None).await?;
            assert!(alt_result.risk_level <= result.risk_level, 
                "Alternative '{}' should be safer than '{}'", alternative, dangerous_cmd);
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_confidence_scoring() -> Result<()> {
    let validator = AdvancedSafetyValidator::new().await?;
    
    let commands_with_expected_confidence = vec![
        ("rm -rf /", 0.95), // Very confident this is dangerous
        ("rm temp.txt", 0.90), // Confident this is safe
        ("rm -rf temp*", 0.85), // Moderately confident (depends on context)
        ("some_unknown_command --weird-flag", 0.60), // Low confidence
    ];
    
    for (cmd, expected_min_confidence) in commands_with_expected_confidence {
        let result = validator.validate(cmd, "/home/user", None).await?;
        
        assert!(result.confidence >= expected_min_confidence, 
            "Confidence for '{}' should be at least {}, got {}", 
            cmd, expected_min_confidence, result.confidence);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_performance_under_load() -> Result<()> {
    let validator = AdvancedSafetyValidator::new().await?;
    
    // Test performance with many validation requests
    let commands = vec![
        "ls -la", "rm temp.txt", "find . -name '*.rs'", "grep pattern file.txt",
        "ps aux", "df -h", "cat /etc/passwd", "chmod 777 file", "rm -rf /tmp/*"
    ];
    
    let mut total_duration = Duration::new(0, 0);
    let iterations = 100;
    
    for _ in 0..iterations {
        for cmd in &commands {
            let start = std::time::Instant::now();
            let _result = validator.validate(cmd, "/home/user", None).await?;
            total_duration += start.elapsed();
        }
    }
    
    let avg_duration = total_duration / (iterations * commands.len() as u32);
    
    // Each validation should average well under constitutional requirement
    assert!(avg_duration.as_millis() < 25, 
        "Average validation time {} ms should be well under 50ms requirement", 
        avg_duration.as_millis());
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_safety_validation() -> Result<()> {
    let validator = std::sync::Arc::new(AdvancedSafetyValidator::new().await?);
    
    // Test concurrent validation requests
    let mut handles = vec![];
    
    for i in 0..20 {
        let validator_clone = validator.clone();
        let handle = tokio::spawn(async move {
            let cmd = format!("echo 'test {}'", i);
            let result = validator_clone.validate(&cmd, "/home/user", None).await.unwrap();
            assert!(result.is_safe);
            assert_eq!(result.risk_level, RiskLevel::Safe);
        });
        handles.push(handle);
    }
    
    // Wait for all validations to complete
    for handle in handles {
        handle.await?;
    }
    
    Ok(())
}

#[tokio::test]
async fn test_validation_result_serialization() -> Result<()> {
    let validator = AdvancedSafetyValidator::new().await?;
    
    let result = validator.validate("rm -rf /", "/", None).await?;
    
    // Test that ValidationResult can be serialized for storage/transmission
    let serialized = serde_json::to_string(&result)?;
    let deserialized: ValidationResult = serde_json::from_str(&serialized)?;
    
    assert_eq!(result.is_safe, deserialized.is_safe);
    assert_eq!(result.risk_level, deserialized.risk_level);
    assert_eq!(result.confidence, deserialized.confidence);
    assert_eq!(result.explanation, deserialized.explanation);
    assert_eq!(result.patterns_matched, deserialized.patterns_matched);
    
    Ok(())
}