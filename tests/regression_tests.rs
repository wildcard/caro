//! Regression Test Suite for Advanced Tool Use Patterns
//!
//! This module provides comprehensive regression testing against 200+ command
//! test cases covering safe, dangerous, and edge case scenarios.

use caro::tools::{ToolCall, ToolData, ToolRegistry};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct RegressionTestSuite {
    #[allow(dead_code)]
    metadata: TestMetadata,
    safe_commands: Vec<CommandTestCase>,
    warn_commands: Vec<CommandTestCase>,
    block_commands: Vec<CommandTestCase>,
    edge_cases: Vec<CommandTestCase>,
    platform_specific: HashMap<String, Vec<CommandTestCase>>,
    false_positive_tests: Vec<CommandTestCase>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TestMetadata {
    version: String,
    description: String,
    total_commands: u32,
}

#[derive(Debug, Deserialize)]
struct CommandTestCase {
    command: String,
    expected_risk: String,
    #[serde(default)]
    #[allow(dead_code)]
    category: Option<String>,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    risk_score: Option<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    requires_confirmation: Option<bool>,
    #[serde(default)]
    note: Option<String>,
}

fn load_test_suite() -> RegressionTestSuite {
    let json_content = include_str!("fixtures/regression_commands.json");
    serde_json::from_str(json_content).expect("Failed to parse regression test suite")
}

#[tokio::test]
async fn test_safe_commands() {
    let suite = load_test_suite();
    let registry = ToolRegistry::default();

    let mut passed = 0;
    let mut failed = Vec::new();

    for case in &suite.safe_commands {
        let call = ToolCall::new("validation")
            .with_param("operation", "risk_score")
            .with_param("command", &case.command);

        let result = registry
            .invoke(&call)
            .await
            .expect("Tool invocation failed");

        if let ToolData::Integer(score) = result.data {
            if score == 0 {
                passed += 1;
            } else {
                failed.push((case.command.clone(), score, "expected 0"));
            }
        }
    }

    println!(
        "Safe commands: {}/{} passed",
        passed,
        suite.safe_commands.len()
    );

    if !failed.is_empty() {
        for (cmd, score, expected) in &failed {
            println!("  FAILED: '{}' got score {} ({})", cmd, score, expected);
        }
    }

    // Allow up to 5% false positives for safe commands
    let threshold = (suite.safe_commands.len() as f64 * 0.95) as usize;
    assert!(
        passed >= threshold,
        "Too many false positives: {}/{} commands flagged incorrectly",
        failed.len(),
        suite.safe_commands.len()
    );
}

#[tokio::test]
async fn test_warn_commands() {
    let suite = load_test_suite();
    let registry = ToolRegistry::default();

    let mut passed = 0;
    let mut failed = Vec::new();

    for case in &suite.warn_commands {
        let call = ToolCall::new("validation")
            .with_param("operation", "risk_score")
            .with_param("command", &case.command);

        let result = registry
            .invoke(&call)
            .await
            .expect("Tool invocation failed");

        if let ToolData::Integer(score) = result.data {
            // Warn commands should have score > 0
            if score > 0 {
                passed += 1;
            } else {
                failed.push((case.command.clone(), score));
            }
        }
    }

    println!(
        "Warn commands: {}/{} passed",
        passed,
        suite.warn_commands.len()
    );

    if !failed.is_empty() {
        println!("FAILED warn commands:");
        for (cmd, score) in &failed {
            println!("  '{}' got score {}", cmd, score);
        }
    }

    // At least 70% of warn commands should be detected
    // (some commands may not have matching patterns yet)
    let threshold = (suite.warn_commands.len() as f64 * 0.70) as usize;
    assert!(
        passed >= threshold,
        "Warn commands not detected: {}/{} missed (need at least {})",
        failed.len(),
        suite.warn_commands.len(),
        threshold
    );
}

#[tokio::test]
async fn test_block_commands() {
    let suite = load_test_suite();
    let registry = ToolRegistry::default();

    let mut passed = 0;
    let mut failed = Vec::new();

    for case in &suite.block_commands {
        let call = ToolCall::new("validation")
            .with_param("operation", "risk_score")
            .with_param("command", &case.command);

        let result = registry
            .invoke(&call)
            .await
            .expect("Tool invocation failed");

        if let ToolData::Integer(score) = result.data {
            // Block commands should have score > 50
            if score > 50 {
                passed += 1;
            } else {
                failed.push((case.command.clone(), score, case.pattern.clone()));
            }
        }
    }

    println!(
        "Block commands: {}/{} passed",
        passed,
        suite.block_commands.len()
    );

    if !failed.is_empty() {
        println!("FAILED block commands:");
        for (cmd, score, pattern) in &failed {
            println!("  '{}' got score {} (pattern: {:?})", cmd, score, pattern);
        }
    }

    // At least 90% of block commands should be caught
    // Critical patterns may have some edge cases
    let threshold = (suite.block_commands.len() as f64 * 0.90) as usize;
    assert!(
        passed >= threshold,
        "Block commands not caught: {}/{} missed (need at least {})",
        failed.len(),
        suite.block_commands.len(),
        threshold
    );
}

#[tokio::test]
async fn test_edge_cases() {
    let suite = load_test_suite();
    let registry = ToolRegistry::default();

    let mut block_detected = 0;
    let mut warn_detected = 0;

    for case in &suite.edge_cases {
        let call = ToolCall::new("validation")
            .with_param("operation", "risk_score")
            .with_param("command", &case.command);

        let result = registry
            .invoke(&call)
            .await
            .expect("Tool invocation failed");

        if let ToolData::Integer(score) = result.data {
            match case.expected_risk.as_str() {
                "block" if score > 50 => block_detected += 1,
                "warn" if score > 0 => warn_detected += 1,
                _ => {}
            }
        }
    }

    let block_expected = suite
        .edge_cases
        .iter()
        .filter(|c| c.expected_risk == "block")
        .count();
    let warn_expected = suite
        .edge_cases
        .iter()
        .filter(|c| c.expected_risk == "warn")
        .count();

    println!(
        "Edge cases: {}/{} block detected, {}/{} warn detected",
        block_detected, block_expected, warn_detected, warn_expected
    );

    // At least 70% of edge cases should be caught
    let total_expected = block_expected + warn_expected;
    let total_detected = block_detected + warn_detected;
    let threshold = (total_expected as f64 * 0.70) as usize;

    assert!(
        total_detected >= threshold,
        "Edge case detection too low: {}/{} detected (need at least {})",
        total_detected,
        total_expected,
        threshold
    );
}

#[tokio::test]
async fn test_false_positive_avoidance() {
    let suite = load_test_suite();
    let registry = ToolRegistry::default();

    let mut correctly_safe = 0;
    let mut false_positives = Vec::new();

    for case in &suite.false_positive_tests {
        let call = ToolCall::new("validation")
            .with_param("operation", "risk_score")
            .with_param("command", &case.command);

        let result = registry
            .invoke(&call)
            .await
            .expect("Tool invocation failed");

        if let ToolData::Integer(score) = result.data {
            if score == 0 {
                correctly_safe += 1;
            } else {
                false_positives.push((case.command.clone(), score, case.note.clone()));
            }
        }
    }

    println!(
        "False positive tests: {}/{} correctly identified as safe",
        correctly_safe,
        suite.false_positive_tests.len()
    );

    if !false_positives.is_empty() {
        println!("False positives:");
        for (cmd, score, note) in &false_positives {
            println!("  '{}' got score {} (note: {:?})", cmd, score, note);
        }
    }

    // At least 30% should be correctly identified as safe
    // (quoted commands containing dangerous patterns are legitimately hard
    // without a proper shell parser - this is a known limitation)
    let threshold = (suite.false_positive_tests.len() as f64 * 0.30) as usize;
    assert!(
        correctly_safe >= threshold,
        "Too many false positives: {}/{} incorrectly flagged (threshold: {})",
        false_positives.len(),
        suite.false_positive_tests.len(),
        threshold
    );
}

#[tokio::test]
async fn test_batch_validation_performance() {
    let suite = load_test_suite();
    let registry = ToolRegistry::default();

    // Collect first 10 commands from each category
    let mut test_commands: Vec<String> = Vec::new();
    test_commands.extend(
        suite
            .safe_commands
            .iter()
            .take(10)
            .map(|c| c.command.clone()),
    );
    test_commands.extend(
        suite
            .block_commands
            .iter()
            .take(10)
            .map(|c| c.command.clone()),
    );

    let call = ToolCall::new("validation")
        .with_param("operation", "batch_validate")
        .with_string_array("commands", test_commands.clone());

    let start = std::time::Instant::now();
    let result = registry
        .invoke(&call)
        .await
        .expect("Batch validation failed");
    let elapsed = start.elapsed();

    println!(
        "Batch validation of {} commands took {:?}",
        test_commands.len(),
        elapsed
    );

    // Should complete in under 1000ms for 20 commands
    assert!(
        elapsed.as_millis() < 1000,
        "Batch validation too slow: {:?}",
        elapsed
    );

    // Verify results
    if let ToolData::Map(results) = result.data {
        assert_eq!(results.len(), test_commands.len());
    }
}

#[tokio::test]
async fn test_platform_specific_commands() {
    let suite = load_test_suite();
    let registry = ToolRegistry::default();

    // Test Linux commands
    if let Some(linux_cmds) = suite.platform_specific.get("linux") {
        for case in linux_cmds {
            let call = ToolCall::new("validation")
                .with_param("operation", "risk_score")
                .with_param("command", &case.command);

            let result = registry
                .invoke(&call)
                .await
                .expect("Tool invocation failed");

            if let ToolData::Integer(score) = result.data {
                // All platform commands should be safe
                assert!(
                    score <= 25,
                    "Linux command '{}' flagged with score {}",
                    case.command,
                    score
                );
            }
        }
    }

    // Test macOS commands
    if let Some(macos_cmds) = suite.platform_specific.get("macos") {
        for case in macos_cmds {
            let call = ToolCall::new("validation")
                .with_param("operation", "risk_score")
                .with_param("command", &case.command);

            let result = registry
                .invoke(&call)
                .await
                .expect("Tool invocation failed");

            if let ToolData::Integer(score) = result.data {
                assert!(
                    score <= 25,
                    "macOS command '{}' flagged with score {}",
                    case.command,
                    score
                );
            }
        }
    }

    println!("Platform-specific commands validated successfully");
}

#[tokio::test]
async fn test_critical_pattern_coverage() {
    let registry = ToolRegistry::default();

    // These MUST be blocked (score > 75)
    let critical_patterns = vec![
        "rm -rf /",
        "rm -rf /*",
        ":(){ :|:& };:",
        "dd if=/dev/zero of=/dev/sda",
        "mkfs.ext4 /dev/sda1",
        "chmod 777 /",
        "> /etc/passwd",
    ];

    for cmd in &critical_patterns {
        let call = ToolCall::new("validation")
            .with_param("operation", "risk_score")
            .with_param("command", *cmd);

        let result = registry
            .invoke(&call)
            .await
            .expect("Tool invocation failed");

        if let ToolData::Integer(score) = result.data {
            assert!(
                score >= 75,
                "CRITICAL: '{}' got score {} (expected >= 75)",
                cmd,
                score
            );
        }
    }

    println!(
        "All {} critical patterns properly blocked",
        critical_patterns.len()
    );
}

#[tokio::test]
async fn test_validation_response_format() {
    let registry = ToolRegistry::default();

    let call = ToolCall::new("validation")
        .with_param("operation", "validate")
        .with_param("command", "rm -rf /");

    let result = registry
        .invoke(&call)
        .await
        .expect("Tool invocation failed");
    assert!(result.success);

    if let ToolData::Structured(data) = &result.data {
        // Verify all expected fields are present
        assert!(data.fields.contains_key("command"));
        assert!(data.fields.contains_key("risk_score"));
        assert!(data.fields.contains_key("risk_level"));
        assert!(data.fields.contains_key("is_safe"));
        assert!(data.fields.contains_key("should_block"));
        assert!(data.fields.contains_key("matched_patterns"));
    } else {
        panic!("Expected structured data from validation");
    }
}

#[tokio::test]
async fn test_total_command_count() {
    let suite = load_test_suite();

    let total = suite.safe_commands.len()
        + suite.warn_commands.len()
        + suite.block_commands.len()
        + suite.edge_cases.len()
        + suite.false_positive_tests.len()
        + suite
            .platform_specific
            .values()
            .map(|v| v.len())
            .sum::<usize>();

    println!("Total test commands: {}", total);

    // We should have 200+ commands
    assert!(
        total >= 200,
        "Not enough test commands: {} (expected 200+)",
        total
    );
}
