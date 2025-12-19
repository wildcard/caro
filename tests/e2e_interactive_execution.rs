// End-to-end black box tests for interactive execution
// These tests spawn the actual binary and interact with stdin/stdout

use std::io::Write;
use std::process::{Command, Stdio};

/// Helper to run cmdai binary with input
fn run_cmdai_with_input(args: &[&str], input: &str) -> (String, String, i32) {
    let mut child = Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cmdai");

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }

    // Wait for completion and capture output
    let output = child.wait_with_output().expect("Failed to wait for cmdai");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    (stdout, stderr, exit_code)
}

/// Helper to run cmdai binary without input (for non-interactive tests)
fn run_cmdai(args: &[&str]) -> (String, String, i32) {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cmdai")
        .wait_with_output()
        .expect("Failed to wait for cmdai");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    (stdout, stderr, exit_code)
}

#[test]
fn test_e2e_execute_flag_runs_command() {
    // Test: --execute flag should auto-execute the command
    let (stdout, _stderr, exit_code) = run_cmdai(&["--execute", "current directory"]);

    // Should succeed
    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should show the command
    assert!(
        stdout.contains("pwd") || stdout.contains("Command:"),
        "Should show the generated command"
    );

    // Should show execution results
    assert!(
        stdout.contains("Execution Results:") || stdout.contains("Success"),
        "Should show execution results"
    );

    // Should contain actual output from pwd command
    assert!(
        stdout.contains("cmdai") || stdout.contains("/home/") || stdout.contains("/"),
        "Should contain output from pwd command"
    );
}

#[test]
fn test_e2e_dry_run_no_execution() {
    // Test: --dry-run should show what would happen without executing
    let (stdout, _stderr, exit_code) = run_cmdai(&["--dry-run", "list files"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should show the command
    assert!(
        stdout.contains("ls") || stdout.contains("Command:"),
        "Should show the generated command"
    );

    // Should show dry-run message
    assert!(
        stdout.contains("Dry Run") || stdout.contains("would be executed"),
        "Should indicate dry-run mode"
    );

    // Should NOT show execution results
    assert!(
        !stdout.contains("Execution Results:"),
        "Should not show execution results in dry-run"
    );

    // Should NOT contain actual ls output (file listings)
    assert!(
        !stdout.contains("total"),
        "Should not contain ls output in dry-run mode"
    );
}

#[test]
fn test_e2e_short_execute_flag() {
    // Test: -x short flag should work the same as --execute
    let (stdout, _stderr, exit_code) = run_cmdai(&["-x", "current directory"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");
    assert!(
        stdout.contains("pwd") || stdout.contains("Command:"),
        "Should show command"
    );
    assert!(
        stdout.contains("Execution Results:") || stdout.contains("Success"),
        "Should execute with -x flag"
    );
}

#[test]
fn test_e2e_interactive_flag() {
    // Test: -i flag should auto-execute
    let (stdout, _stderr, exit_code) = run_cmdai(&["-i", "current directory"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");
    assert!(
        stdout.contains("Execution Results:") || stdout.contains("Success"),
        "Should execute with -i flag"
    );
}

#[test]
fn test_e2e_verbose_with_execute() {
    // Test: --verbose with --execute should show debug info
    let (stdout, _stderr, exit_code) = run_cmdai(&["--verbose", "--execute", "current directory"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should show execution results
    assert!(
        stdout.contains("Execution Results:") || stdout.contains("Success"),
        "Should execute command"
    );

    // Verbose mode should show additional info
    assert!(
        stdout.contains("Debug Info:") || stdout.contains("Backend:") || stdout.contains("ms"),
        "Should show verbose debug information"
    );
}

#[test]
fn test_e2e_json_output_format() {
    // Test: JSON output format
    let (stdout, _stderr, exit_code) =
        run_cmdai(&["--output", "json", "--execute", "current directory"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should be valid JSON
    assert!(
        stdout.contains("{") && stdout.contains("}"),
        "Should output JSON format"
    );

    // Should contain expected fields
    assert!(
        stdout.contains("generated_command") && stdout.contains("exit_code"),
        "Should contain expected JSON fields"
    );

    // Verify it's actually valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "Should be valid JSON");

    if let Ok(json) = parsed {
        assert!(
            json["exit_code"].as_i64().is_some(),
            "Should have exit_code in JSON"
        );
        assert!(
            json["executed"].as_bool().is_some(),
            "Should have executed field in JSON"
        );
    }
}

#[test]
fn test_e2e_no_flags_non_interactive() {
    // Test: Without flags in non-interactive mode (piped stdin)
    let (stdout, _stderr, exit_code) = run_cmdai(&["list files"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should show the command
    assert!(
        stdout.contains("ls") || stdout.contains("Command:"),
        "Should show command"
    );

    // Should show message about using --execute flag
    assert!(
        stdout.contains("--execute") || stdout.contains("-x"),
        "Should suggest using --execute flag in non-interactive mode"
    );

    // Should NOT execute (no execution results)
    assert!(
        !stdout.contains("Execution Results:"),
        "Should not auto-execute without flag in non-interactive mode"
    );
}

#[test]
fn test_e2e_multiple_commands() {
    // Test: Run multiple commands in sequence
    let commands: Vec<(Vec<&str>, &str)> = vec![
        (vec!["--execute", "current directory"], "pwd"),
        (vec!["--dry-run", "list files"], "ls"),
    ];

    for (args, expected_cmd) in commands {
        let (stdout, _stderr, exit_code) = run_cmdai(&args);

        assert_eq!(
            exit_code, 0,
            "cmdai should exit successfully for {:?}",
            args
        );
        assert!(
            stdout.contains(expected_cmd) || stdout.contains("Command:"),
            "Should show command for {:?}",
            args
        );
    }
}

#[test]
fn test_e2e_command_with_output() {
    // Test: Command that produces output
    let (stdout, _stderr, exit_code) = run_cmdai(&["-x", "current directory"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should show Standard Output section
    assert!(
        stdout.contains("Standard Output:") || stdout.contains("/"),
        "Should show stdout from command"
    );

    // Should show execution time
    assert!(
        stdout.contains("Execution time:") || stdout.contains("ms"),
        "Should show execution time"
    );
}

#[test]
fn test_e2e_help_flag() {
    // Test: --help flag
    let (stdout, _stderr, exit_code) = run_cmdai(&["--help"]);

    assert_eq!(exit_code, 0, "Should exit successfully with --help");

    // Should show usage information
    assert!(
        stdout.contains("Usage:") || stdout.contains("USAGE:"),
        "Should show usage"
    );
    assert!(
        stdout.contains("--execute") && stdout.contains("--dry-run"),
        "Should document execution flags"
    );
}

#[test]
fn test_e2e_version_flag() {
    // Test: --version flag
    let (stdout, _stderr, _exit_code) = run_cmdai(&["--version"]);

    // Should show version
    assert!(
        stdout.contains("cmdai") && (stdout.contains("0.") || stdout.contains("1.")),
        "Should show version number"
    );
}

#[test]
fn test_e2e_shell_selection() {
    // Test: Shell type selection
    let (stdout, _stderr, exit_code) = run_cmdai(&["--shell", "bash", "-x", "current directory"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");
    assert!(
        stdout.contains("Success") || stdout.contains("exit code: 0"),
        "Should execute successfully with bash shell"
    );
}

#[test]
fn test_e2e_execution_timing() {
    // Test: Verify execution timing is captured
    let (stdout, _stderr, exit_code) = run_cmdai(&["-x", "current directory"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should show execution time
    assert!(
        stdout.contains("Execution time:") && stdout.contains("ms"),
        "Should show execution time in milliseconds"
    );
}

#[test]
fn test_e2e_exit_code_display() {
    // Test: Exit code is displayed
    let (stdout, _stderr, exit_code) = run_cmdai(&["-x", "current directory"]);

    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should show exit code
    assert!(
        stdout.contains("exit code") || stdout.contains("Success"),
        "Should display exit code"
    );
    assert!(
        stdout.contains("0") || stdout.contains("✓"),
        "Should show success indicator"
    );
}

#[test]
#[ignore] // This test requires actual interactive TTY
fn test_e2e_interactive_prompt_yes() {
    // Test: Answering 'y' to interactive prompt
    let (stdout, _stderr, exit_code) = run_cmdai_with_input(&["list files"], "y\n");

    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should show execution results when user says yes
    assert!(
        stdout.contains("Execution Results:") || stdout.contains("Executing"),
        "Should execute when user confirms"
    );
}

#[test]
#[ignore] // This test requires actual interactive TTY
fn test_e2e_interactive_prompt_no() {
    // Test: Answering 'n' to interactive prompt
    let (stdout, _stderr, exit_code) = run_cmdai_with_input(&["list files"], "n\n");

    assert_eq!(exit_code, 0, "cmdai should exit successfully");

    // Should NOT execute when user says no
    assert!(
        !stdout.contains("Execution Results:"),
        "Should not execute when user declines"
    );
    assert!(
        stdout.contains("skipped") || stdout.contains("cancelled"),
        "Should indicate execution was skipped"
    );
}
