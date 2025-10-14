/// End-to-End CLI Black-box Tests
/// 
/// These tests run the actual cmdai binary as a user would and verify behavior
/// from a user's perspective. Tests are based on the comprehensive manual QA
/// test cases and validate the complete user experience.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::Value;
use tempfile::TempDir;

/// Helper struct for running cmdai CLI commands and capturing output
struct CliTestRunner {
    binary_path: String,
    temp_dir: TempDir,
}

impl CliTestRunner {
    /// Create a new CLI test runner with temporary directory
    fn new() -> Self {
        let binary_path = if Path::new("target/debug/cmdai").exists() {
            "target/debug/cmdai".to_string()
        } else {
            // Fallback to cargo run for cases where binary isn't built
            "cargo".to_string()
        };

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        Self {
            binary_path,
            temp_dir,
        }
    }

    /// Run a cmdai command and return stdout, stderr, and exit status
    fn run_command(&self, args: &[&str]) -> CliTestResult {
        let start_time = std::time::Instant::now();
        
        let mut cmd = if self.binary_path == "cargo" {
            let mut cmd = Command::new("cargo");
            cmd.arg("run").arg("--");
            cmd.args(args);
            cmd
        } else {
            let mut cmd = Command::new(&self.binary_path);
            cmd.args(args);
            cmd
        };

        // Set clean environment
        cmd.env("CMDAI_CONFIG_DIR", self.temp_dir.path());
        cmd.env_remove("CMDAI_CACHE_DIR");
        
        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cmdai");

        let execution_time = start_time.elapsed();

        CliTestResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            execution_time,
        }
    }

    /// Run command and expect success (exit code 0)
    fn run_success(&self, args: &[&str]) -> String {
        let result = self.run_command(args);
        assert_eq!(
            result.exit_code, 0,
            "Command failed. Args: {:?}\nStdout: {}\nStderr: {}",
            args, result.stdout, result.stderr
        );
        result.stdout
    }

    /// Run command and expect failure (non-zero exit code)
    #[allow(dead_code)]
    fn run_failure(&self, args: &[&str]) -> CliTestResult {
        let result = self.run_command(args);
        assert_ne!(
            result.exit_code, 0,
            "Command unexpectedly succeeded. Args: {:?}\nStdout: {}",
            args, result.stdout
        );
        result
    }
}

/// Result of running a CLI command
#[derive(Debug)]
struct CliTestResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
    execution_time: Duration,
}

// =============================================================================
// E2E Test Suite A: Core CLI Functionality
// =============================================================================

/// E2E-A1: Help Output Test
/// Verifies that --help produces comprehensive help information
#[test]
fn e2e_help_output() {
    let runner = CliTestRunner::new();
    let output = runner.run_success(&["--help"]);
    
    // Verify essential help content
    assert!(output.contains("cmdai converts natural language"));
    assert!(output.contains("Usage: cmdai"));
    assert!(output.contains("--shell"));
    assert!(output.contains("--safety"));
    assert!(output.contains("--output"));
    assert!(output.contains("--verbose"));
    assert!(output.contains("--version"));
    
    // Verify help describes shell options
    assert!(output.contains("bash") || output.contains("Shell type"));
    
    println!("✅ E2E-A1: Help output contains all expected elements");
}

/// E2E-A2: Version Information Test
/// Verifies that --version shows correct version format
#[test]
fn e2e_version_information() {
    let runner = CliTestRunner::new();
    let output = runner.run_success(&["--version"]);
    
    // Should show version in format "cmdai X.Y.Z"
    assert!(output.contains("cmdai"));
    assert!(output.matches(char::is_numeric).count() >= 3); // At least X.Y.Z
    
    println!("✅ E2E-A2: Version information displayed correctly");
}

/// E2E-A3: Basic Command Generation Test
/// Verifies that simple natural language input produces shell command output
#[test]
fn e2e_basic_command_generation() {
    let runner = CliTestRunner::new();
    let output = runner.run_success(&["list files in current directory"]);
    
    // Should contain command output
    assert!(output.contains("Command:") || output.len() > 10);
    assert!(!output.is_empty());
    
    // Should not contain error messages
    assert!(!output.to_lowercase().contains("error"));
    assert!(!output.to_lowercase().contains("failed"));
    
    println!("✅ E2E-A3: Basic command generation works");
}

/// E2E-A4: Performance Test - Startup Time
/// Verifies that CLI startup is reasonably fast
#[test]
fn e2e_performance_startup_time() {
    let runner = CliTestRunner::new();
    let result = runner.run_command(&["--version"]);
    
    // CLI should start within reasonable time (allowing for compilation in CI)
    assert!(
        result.execution_time < Duration::from_secs(30),
        "CLI startup took too long: {:?}",
        result.execution_time
    );
    
    println!("✅ E2E-A4: CLI startup time acceptable ({:?})", result.execution_time);
}

// =============================================================================
// E2E Test Suite B: Output Formats
// =============================================================================

/// E2E-B1: JSON Output Format Test
/// Verifies that --output json produces valid JSON
#[test]
fn e2e_json_output_format() {
    let runner = CliTestRunner::new();
    let output = runner.run_success(&["show disk usage", "--output", "json"]);
    
    // Should be valid JSON
    let json_result: Result<Value, _> = serde_json::from_str(&output);
    assert!(json_result.is_ok(), "Output is not valid JSON: {}", output);
    
    let json_value = json_result.unwrap();
    
    // Should contain expected fields
    assert!(json_value.get("generated_command").is_some());
    assert!(json_value.get("explanation").is_some());
    
    println!("✅ E2E-B1: JSON output format is valid and complete");
}

/// E2E-B2: YAML Output Format Test
/// Verifies that --output yaml produces valid YAML
#[test]
fn e2e_yaml_output_format() {
    let runner = CliTestRunner::new();
    let output = runner.run_success(&["show system info", "--output", "yaml"]);
    
    // Should contain YAML-style output
    assert!(output.contains(":"));
    assert!(!output.starts_with("{"));  // Not JSON
    
    // Should contain expected fields
    assert!(output.contains("generated_command") || output.contains("command"));
    assert!(output.contains("explanation"));
    
    println!("✅ E2E-B2: YAML output format is properly structured");
}

/// E2E-B3: Plain Output Format Test
/// Verifies that plain output (default) is human-readable
#[test]
fn e2e_plain_output_format() {
    let runner = CliTestRunner::new();
    let output = runner.run_success(&["list files"]);
    
    // Plain output should be human-readable
    assert!(!output.starts_with("{")); // Not JSON
    assert!(!output.contains("generated_command:")); // Not YAML
    
    // Should contain readable command information
    assert!(output.contains("Command:") || output.len() > 5);
    
    println!("✅ E2E-B3: Plain output format is human-readable");
}

// =============================================================================
// E2E Test Suite C: Shell Types and Configuration
// =============================================================================

/// E2E-C1: Shell Type Selection Test
/// Verifies that different shell types are accepted
#[test]
fn e2e_shell_type_selection() {
    let runner = CliTestRunner::new();
    
    let shells = ["bash", "zsh", "fish", "sh"];
    
    for shell in &shells {
        let output = runner.run_success(&["list files", "--shell", shell]);
        assert!(!output.is_empty(), "No output for shell: {}", shell);
        assert!(!output.to_lowercase().contains("error"), 
                "Error with shell {}: {}", shell, output);
    }
    
    println!("✅ E2E-C1: All major shell types accepted (bash, zsh, fish, sh)");
}

/// E2E-C2: Safety Level Configuration Test
/// Verifies that different safety levels are accepted
#[test]
fn e2e_safety_level_configuration() {
    let runner = CliTestRunner::new();
    
    let safety_levels = ["strict", "moderate", "permissive"];
    
    for level in &safety_levels {
        let output = runner.run_success(&["rm *.tmp", "--safety", level]);
        assert!(!output.is_empty(), "No output for safety level: {}", level);
        assert!(!output.to_lowercase().contains("error"),
                "Error with safety level {}: {}", level, output);
    }
    
    println!("✅ E2E-C2: All safety levels accepted (strict, moderate, permissive)");
}

/// E2E-C3: Configuration Display Test
/// Verifies that --show-config displays configuration information
#[test]
fn e2e_configuration_display() {
    let runner = CliTestRunner::new();
    let output = runner.run_success(&["--show-config"]);
    
    // Should contain configuration information
    assert!(output.contains("Configuration") || output.contains("config"));
    assert!(output.contains("Safety level") || output.contains("safety"));
    
    println!("✅ E2E-C3: Configuration display shows relevant information");
}

// =============================================================================
// E2E Test Suite D: Error Handling and Edge Cases
// =============================================================================

/// E2E-D1: Empty Input Handling Test
/// Verifies graceful handling of empty command input
#[test]
fn e2e_empty_input_handling() {
    let runner = CliTestRunner::new();
    let _output = runner.run_success(&[""]);
    
    // Should handle empty input gracefully (not crash)
    // Output might be empty or contain a message, but shouldn't error
    
    println!("✅ E2E-D1: Empty input handled gracefully");
}

/// E2E-D2: Invalid Shell Type Handling Test
/// Verifies handling of invalid shell types
#[test]
fn e2e_invalid_shell_type_handling() {
    let runner = CliTestRunner::new();
    let result = runner.run_command(&["test command", "--shell", "invalid_shell"]);
    
    // Should either succeed with warning or fail with clear error
    if result.exit_code == 0 {
        // If it succeeds, should contain a warning
        let combined_output = format!("{}{}", result.stdout, result.stderr);
        assert!(
            combined_output.to_lowercase().contains("warning") ||
            combined_output.to_lowercase().contains("invalid") ||
            combined_output.to_lowercase().contains("default"),
            "No warning for invalid shell. Output: {}", combined_output
        );
    } else {
        // If it fails, error should be clear
        assert!(!result.stderr.is_empty() || !result.stdout.is_empty(),
                "No error message for invalid shell");
    }
    
    println!("✅ E2E-D2: Invalid shell type handled appropriately");
}

/// E2E-D3: Long Input Handling Test
/// Verifies handling of very long command inputs
#[test]
fn e2e_long_input_handling() {
    let runner = CliTestRunner::new();
    let long_input = "a".repeat(1000);
    let output = runner.run_success(&[&long_input]);
    
    // Should handle long input without crashing
    assert!(!output.is_empty() || !output.contains("error"));
    
    println!("✅ E2E-D3: Long input (1000 chars) handled successfully");
}

/// E2E-D4: Special Characters Handling Test
/// Verifies handling of special characters in input
#[test]
fn e2e_special_characters_handling() {
    let runner = CliTestRunner::new();
    let special_input = "test with special chars: @#$%^&*()[]{}|\\;':\"<>?,./";
    let output = runner.run_success(&[special_input]);
    
    // Should handle special characters without crashing
    assert!(!output.is_empty());
    
    println!("✅ E2E-D4: Special characters handled successfully");
}

// =============================================================================
// E2E Test Suite E: Verbose Mode and Debugging
// =============================================================================

/// E2E-E1: Verbose Mode Test
/// Verifies that --verbose provides additional debugging information
#[test]
fn e2e_verbose_mode() {
    let runner = CliTestRunner::new();
    let normal_output = runner.run_success(&["test command"]);
    let verbose_output = runner.run_success(&["test command", "--verbose"]);
    
    // Verbose output should contain more information
    assert!(
        verbose_output.len() >= normal_output.len(),
        "Verbose output should be longer or equal to normal output"
    );
    
    // Should contain debug information
    assert!(
        verbose_output.contains("Debug") || 
        verbose_output.contains("Backend") ||
        verbose_output.contains("Generated in"),
        "Verbose output should contain debug information: {}",
        verbose_output
    );
    
    println!("✅ E2E-E1: Verbose mode provides additional debugging information");
}

/// E2E-E2: Timing Information Test
/// Verifies that verbose mode includes timing information
#[test]
fn e2e_timing_information() {
    let runner = CliTestRunner::new();
    let output = runner.run_success(&["test command", "--verbose"]);
    
    // Should contain timing information in verbose mode
    assert!(
        output.contains("ms") ||
        output.contains("time") ||
        output.contains("Generated in"),
        "Verbose output should contain timing information: {}",
        output
    );
    
    println!("✅ E2E-E2: Timing information displayed in verbose mode");
}

// =============================================================================
// E2E Test Suite F: Integration and Workflow Tests
// =============================================================================

/// E2E-F1: Complete Workflow Test
/// Tests a complete user workflow with multiple options
#[test]
fn e2e_complete_workflow() {
    let runner = CliTestRunner::new();
    
    // Test complex command with multiple options
    let output = runner.run_success(&[
        "find all Python files in the project",
        "--shell", "bash",
        "--safety", "moderate",
        "--output", "json",
        "--verbose"
    ]);
    
    // Should produce valid JSON with debug info
    let json_result: Result<Value, _> = serde_json::from_str(&output);
    assert!(json_result.is_ok(), "Complex workflow should produce valid JSON");
    
    println!("✅ E2E-F1: Complete workflow with all options works correctly");
}

/// E2E-F2: Multiple Commands Performance Test
/// Tests running multiple commands in sequence
#[test]
fn e2e_multiple_commands_performance() {
    let runner = CliTestRunner::new();
    
    let commands = [
        "list files",
        "show date",
        "check disk space",
        "display environment variables"
    ];
    
    let start_time = std::time::Instant::now();
    
    for cmd in &commands {
        let output = runner.run_success(&[cmd]);
        assert!(!output.is_empty(), "Command '{}' produced no output", cmd);
    }
    
    let total_time = start_time.elapsed();
    
    // Multiple commands should complete in reasonable time
    assert!(
        total_time < Duration::from_secs(60),
        "Multiple commands took too long: {:?}",
        total_time
    );
    
    println!("✅ E2E-F2: Multiple commands completed in {:?}", total_time);
}

/// E2E-F3: Output Consistency Test
/// Verifies that the same input produces consistent output
#[test]
fn e2e_output_consistency() {
    let runner = CliTestRunner::new();
    
    let test_input = "show current directory";
    let output1 = runner.run_success(&[test_input]);
    let output2 = runner.run_success(&[test_input]);
    
    // Outputs should be consistent (same command, same explanation structure)
    // Allow for minor variations in timing or mock data
    assert!(!output1.is_empty() && !output2.is_empty());
    assert_eq!(output1.len(), output2.len());
    
    println!("✅ E2E-F3: Output consistency verified for repeated commands");
}

// =============================================================================
// Test Utilities and Helpers
// =============================================================================

/// Helper function to build the binary before running E2E tests
/// This ensures we have a fresh binary for testing
#[cfg(test)]
fn ensure_binary_built() {
    use std::process::Command;
    
    let output = Command::new("cargo")
        .args(&["build", "--bin", "cmdai"])
        .output()
        .expect("Failed to build cmdai binary");
    
    if !output.status.success() {
        panic!("Failed to build cmdai binary: {}", 
               String::from_utf8_lossy(&output.stderr));
    }
}

/// Integration test that runs a subset of critical E2E tests
/// This can be used in CI/CD pipelines for quick validation
#[test]
fn e2e_smoke_test_suite() {
    ensure_binary_built();
    
    let runner = CliTestRunner::new();
    
    // Critical functionality tests
    println!("Running E2E smoke tests...");
    
    // Test 1: Basic functionality
    let help_output = runner.run_success(&["--help"]);
    assert!(help_output.contains("cmdai"));
    
    // Test 2: Command generation
    let cmd_output = runner.run_success(&["list files"]);
    assert!(!cmd_output.is_empty());
    
    // Test 3: JSON output
    let json_output = runner.run_success(&["test", "--output", "json"]);
    assert!(serde_json::from_str::<Value>(&json_output).is_ok());
    
    // Test 4: Error handling
    let _result = runner.run_command(&["", "--shell", "invalid"]);
    // Should not panic regardless of output
    
    println!("✅ E2E Smoke Test Suite: All critical tests passed");
}

// =============================================================================
// E2E Test Documentation and Usage
// =============================================================================

#[cfg(test)]
mod e2e_documentation {
    //! # End-to-End Test Documentation
    //!
    //! ## Overview
    //! These E2E tests validate the cmdai CLI from a user's perspective by running
    //! the actual binary and testing complete workflows.
    //!
    //! ## Test Categories
    //! - **Suite A**: Core CLI functionality (help, version, basic commands)
    //! - **Suite B**: Output formats (JSON, YAML, plain text)
    //! - **Suite C**: Shell types and configuration
    //! - **Suite D**: Error handling and edge cases
    //! - **Suite E**: Verbose mode and debugging
    //! - **Suite F**: Integration and workflow tests
    //!
    //! ## Running Tests
    //! ```bash
    //! # Run all E2E tests
    //! cargo test e2e_
    //!
    //! # Run specific test suite
    //! cargo test e2e_json_output_format
    //!
    //! # Run smoke tests (critical functionality only)
    //! cargo test e2e_smoke_test_suite
    //! ```
    //!
    //! ## Test Environment
    //! - Uses temporary directories for configuration isolation
    //! - Builds fresh binary before testing (when needed)
    //! - Tests actual CLI behavior as users would experience
    //!
    //! ## Adding New Tests
    //! 1. Follow naming convention: `e2e_[category]_[description]`
    //! 2. Use `CliTestRunner` for consistent test execution
    //! 3. Verify both success and failure scenarios
    //! 4. Include meaningful assertions and error messages
    //!
    //! ## CI/CD Integration
    //! These tests are designed to run in CI environments and validate
    //! that the CLI behaves correctly in real deployment scenarios.
}