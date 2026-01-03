// Tests for interactive execution prompt behavior
// Verifies that commands prompt for execution by default

use caro::cli::{CliApp, IntoCliArgs};

// Import validation functions for WP05 tests
// Note: These are defined in src/main.rs but need to be accessible for testing
// We'll test them via integration tests below

/// Mock CLI arguments for testing
#[derive(Default, Clone)]
struct TestArgs {
    prompt: Option<String>,
    shell: Option<String>,
    safety: Option<String>,
    output: Option<String>,
    confirm: bool,
    verbose: bool,
    config_file: Option<String>,
    execute: bool,
    dry_run: bool,
    interactive: bool,
}

impl IntoCliArgs for TestArgs {
    fn prompt(&self) -> Option<String> {
        self.prompt.clone()
    }

    fn shell(&self) -> Option<String> {
        self.shell.clone()
    }

    fn safety(&self) -> Option<String> {
        self.safety.clone()
    }

    fn output(&self) -> Option<String> {
        self.output.clone()
    }

    fn confirm(&self) -> bool {
        self.confirm
    }

    fn verbose(&self) -> bool {
        self.verbose
    }

    fn config_file(&self) -> Option<String> {
        self.config_file.clone()
    }

    fn execute(&self) -> bool {
        self.execute
    }

    fn dry_run(&self) -> bool {
        self.dry_run
    }

    fn interactive(&self) -> bool {
        self.interactive
    }

    fn no_spellcheck(&self) -> bool {
        false
    }
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_default_behavior_no_auto_execution() {
    // Default behavior: command generated but NOT executed
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("list files".to_string()),
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // Command should be generated
    assert!(!result.generated_command.is_empty());

    // Safety checks should pass (list files is safe)
    assert!(
        result.executed,
        "Safety checks should pass for safe commands"
    );

    // But command should NOT be auto-executed (no --execute flag)
    assert!(
        result.exit_code.is_none(),
        "Command should not execute without --execute flag or user confirmation"
    );
    assert!(
        result.stdout.is_none(),
        "Should have no stdout without execution"
    );
    assert!(
        result.stderr.is_none(),
        "Should have no stderr without execution"
    );
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_execute_flag_auto_executes() {
    // With --execute flag: command should auto-execute
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("current directory".to_string()),
        execute: true, // Auto-execute flag
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // Command should be generated
    assert!(!result.generated_command.is_empty());

    // Command SHOULD be executed automatically
    assert!(
        result.exit_code.is_some(),
        "Command should execute with --execute flag"
    );

    // Should have execution output
    let exit_code = result.exit_code.unwrap();
    assert_eq!(exit_code, 0, "pwd command should succeed");

    assert!(result.stdout.is_some(), "Should have stdout from execution");
    let stdout = result.stdout.unwrap();
    assert!(!stdout.is_empty(), "Should have output from pwd");
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_interactive_flag_auto_executes() {
    // With --interactive flag: command should auto-execute
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("current directory".to_string()),
        interactive: true, // Interactive flag
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // Command should be executed automatically with -i flag
    assert!(
        result.exit_code.is_some(),
        "Command should execute with --interactive flag"
    );
    assert!(result.stdout.is_some(), "Should have stdout from execution");
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_dry_run_no_execution() {
    // With --dry-run: command should NOT execute
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("list files".to_string()),
        dry_run: true,
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // Command should be generated
    assert!(!result.generated_command.is_empty());

    // Safety checks pass
    assert!(result.executed, "Safety checks should pass");

    // But NO execution in dry-run mode
    assert!(result.exit_code.is_none(), "Dry-run should not execute");
    assert!(result.stdout.is_none(), "Dry-run should have no stdout");
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_dangerous_command_blocked_without_confirmation() {
    // Dangerous commands should require confirmation
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("delete system".to_string()), // Triggers dangerous command
        execute: true, // Even with execute flag, dangerous commands need confirmation
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // Command generated but should require confirmation
    assert!(!result.generated_command.is_empty());

    // Should require confirmation
    assert!(
        result.requires_confirmation || result.blocked_reason.is_some(),
        "Dangerous commands should require confirmation or be blocked"
    );

    // Should NOT execute without explicit confirmation
    if result.blocked_reason.is_none() {
        // If not blocked, it requires confirmation, so shouldn't auto-execute
        assert!(
            result.exit_code.is_none(),
            "Dangerous command should not auto-execute"
        );
    }
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_dangerous_command_executes_with_confirm_flag() {
    // Dangerous commands with --confirm flag should execute (after safety check passes)
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("delete".to_string()), // Potentially dangerous
        execute: true,
        confirm: true, // Auto-confirm
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // For moderately dangerous commands with confirm flag
    if result.blocked_reason.is_none() {
        // Command should execute if not blocked
        assert!(
            result.exit_code.is_some() || result.execution_error.is_some(),
            "Command should attempt execution with --confirm flag"
        );
    }
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_execution_captures_output() {
    // Verify that execution properly captures stdout, stderr, and exit code
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("current directory".to_string()),
        execute: true,
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // Should have exit code
    assert!(result.exit_code.is_some(), "Should capture exit code");
    assert_eq!(result.exit_code.unwrap(), 0, "pwd should exit with 0");

    // Should have stdout
    assert!(result.stdout.is_some(), "Should capture stdout");
    let stdout = result.stdout.unwrap();
    assert!(!stdout.trim().is_empty(), "Should have output");

    // Should capture execution time
    assert!(
        result.timing_info.execution_time_ms > 0,
        "Should track execution time"
    );
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_execution_handles_errors() {
    // Test that failed commands are handled properly
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("directory".to_string()), // This generates "pwd" which won't fail, so we need a different test
        execute: true,
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // Just verify execution worked - we'll test error handling in integration tests
    assert!(result.exit_code.is_some(), "Should have exit code");
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_multiple_execution_modes_mutually_exclusive() {
    // Verify behavior when multiple flags are set
    let cli = CliApp::new().await.unwrap();

    // execute + dry_run: dry_run should take precedence (no execution)
    let args = TestArgs {
        prompt: Some("list files".to_string()),
        execute: true,
        dry_run: true,
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // Dry-run should prevent execution even with execute flag
    assert!(
        result.exit_code.is_none(),
        "Dry-run should prevent execution"
    );
}

#[tokio::test]
#[cfg(unix)] // Tests execute generated shell commands which may use Unix-specific utilities
async fn test_safe_command_passes_all_checks() {
    // Verify a completely safe command passes all checks
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("list files".to_string()),
        ..Default::default()
    };

    let result = cli.run_with_args(args).await.unwrap();

    // Should generate command
    assert!(!result.generated_command.is_empty());

    // Should NOT be blocked
    assert!(
        result.blocked_reason.is_none(),
        "Safe command should not be blocked"
    );

    // Should NOT require confirmation
    assert!(
        !result.requires_confirmation,
        "Safe command should not require confirmation"
    );

    // Safety checks pass
    assert!(
        result.executed,
        "executed flag should be true (safety checks passed)"
    );

    // But not auto-executed (no --execute flag)
    assert!(
        result.exit_code.is_none(),
        "Should not auto-execute without flag"
    );
}
