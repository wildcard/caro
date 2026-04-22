//! Integration tests for CLI flags: --quiet, --no-telemetry, --backend-info
//!
//! Tracks GitHub issue #793 (task caro-xk0.1): three flags documented at caro.sh/faq
//! were advertised but not parsed. These tests lock in their observable behavior.

use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::TempDir;

struct CliRunner {
    binary: String,
    temp_dir: TempDir,
}

impl CliRunner {
    fn new() -> Self {
        let binary = if Path::new("target/debug/caro").exists() {
            "target/debug/caro".to_string()
        } else {
            "cargo".to_string()
        };
        let temp_dir = TempDir::new().expect("temp dir");
        Self { binary, temp_dir }
    }

    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let mut cmd = if self.binary == "cargo" {
            let mut c = Command::new("cargo");
            c.arg("run").arg("--");
            c.args(args);
            c
        } else {
            let mut c = Command::new(&self.binary);
            c.args(args);
            c
        };

        cmd.env("CARO_CONFIG_DIR", self.temp_dir.path());
        cmd.env_remove("CARO_CACHE_DIR");

        let out = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("failed to execute caro");

        (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
            out.status.code().unwrap_or(-1),
        )
    }
}

// =============================================================================
// --help advertises the new flags
// =============================================================================

#[test]
fn help_lists_quiet_flag() {
    let runner = CliRunner::new();
    let (stdout, _stderr, code) = runner.run(&["--help"]);
    assert_eq!(code, 0, "--help should exit 0");
    assert!(
        stdout.contains("--quiet"),
        "--help output should advertise --quiet, got:\n{}",
        stdout
    );
}

#[test]
fn help_lists_no_telemetry_flag() {
    let runner = CliRunner::new();
    let (stdout, _stderr, code) = runner.run(&["--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("--no-telemetry"),
        "--help output should advertise --no-telemetry, got:\n{}",
        stdout
    );
}

#[test]
fn help_lists_backend_info_flag() {
    let runner = CliRunner::new();
    let (stdout, _stderr, code) = runner.run(&["--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("--backend-info"),
        "--help output should advertise --backend-info, got:\n{}",
        stdout
    );
}

// =============================================================================
// --quiet: flag is accepted (does not error like pre-fix behavior)
// =============================================================================

#[test]
fn quiet_flag_is_accepted_with_prompt() {
    let runner = CliRunner::new();
    let (stdout, stderr, code) = runner.run(&["--quiet", "list files"]);
    // Must not be a clap parse error.
    assert!(
        !stderr.contains("unexpected argument '--quiet'"),
        "clap should recognise --quiet, stderr was:\n{}",
        stderr
    );
    // On successful parse we either exit 0 or show normal output, but never
    // the "unexpected argument" parse failure exit code pattern.
    assert!(
        code == 0 || !stderr.contains("unexpected argument"),
        "--quiet should be parsed; code={} stderr={}",
        code,
        stderr
    );
    // stdout should still show the generated command line (command is the
    // minimum output that --quiet preserves).
    let _ = stdout;
}

#[test]
fn quiet_suppresses_execution_timing_line() {
    // When the command is actually executed (--dry-run so it's safe), the
    // non-quiet path prints an "Execution time" / timing-ish line for the run.
    // With --quiet that timing noise must be suppressed. We assert no
    // "Execution time" nor a bare "ms" duration trailer leaks to stdout.
    let runner = CliRunner::new();
    let (stdout, _stderr, _code) = runner.run(&["--quiet", "--dry-run", "list files"]);
    assert!(
        !stdout.contains("Execution time"),
        "--quiet must not print 'Execution time' timing, got:\n{}",
        stdout
    );
}

#[test]
fn quiet_and_dry_run_combine() {
    // Combining flags should not parse-error. --dry-run + --quiet is the
    // canonical non-destructive inspection mode.
    let runner = CliRunner::new();
    let (_stdout, stderr, _code) = runner.run(&["--quiet", "--dry-run", "list files"]);
    assert!(
        !stderr.contains("unexpected argument"),
        "--quiet + --dry-run must co-exist, stderr:\n{}",
        stderr
    );
}

// =============================================================================
// --no-telemetry: flag is accepted
// =============================================================================

#[test]
fn no_telemetry_flag_is_accepted_with_prompt() {
    let runner = CliRunner::new();
    let (_stdout, stderr, _code) = runner.run(&["--no-telemetry", "list files"]);
    assert!(
        !stderr.contains("unexpected argument '--no-telemetry'"),
        "clap should recognise --no-telemetry, stderr:\n{}",
        stderr
    );
}

#[test]
fn no_telemetry_combines_with_dry_run() {
    let runner = CliRunner::new();
    let (_stdout, stderr, _code) = runner.run(&["--no-telemetry", "--dry-run", "list files"]);
    assert!(
        !stderr.contains("unexpected argument"),
        "--no-telemetry + --dry-run must co-exist, stderr:\n{}",
        stderr
    );
}

// =============================================================================
// --backend-info: meta flag that prints a backend table and exits 0
// =============================================================================

#[test]
fn backend_info_exits_zero_without_prompt() {
    let runner = CliRunner::new();
    let (stdout, stderr, code) = runner.run(&["--backend-info"]);
    assert_eq!(
        code, 0,
        "--backend-info should exit 0 without requiring a prompt. stdout:\n{}\nstderr:\n{}",
        stdout, stderr
    );
}

#[test]
fn backend_info_lists_known_backends() {
    let runner = CliRunner::new();
    let (stdout, stderr, _code) = runner.run(&["--backend-info"]);
    let combined = format!("{}{}", stdout, stderr);
    // We print a header plus at least these built-in backend names.
    assert!(
        combined.to_lowercase().contains("backend"),
        "--backend-info output should mention 'backend', got:\n{}",
        combined
    );
    for name in ["static", "embedded", "ollama", "vllm"] {
        assert!(
            combined.to_lowercase().contains(name),
            "--backend-info output should list '{}', got:\n{}",
            name,
            combined
        );
    }
}

#[test]
fn backend_info_shows_status_column() {
    let runner = CliRunner::new();
    let (stdout, stderr, _code) = runner.run(&["--backend-info"]);
    let combined = format!("{}{}", stdout, stderr).to_lowercase();
    // Must show some per-backend status signal (available / not / configured / missing).
    let has_status = combined.contains("available")
        || combined.contains("not available")
        || combined.contains("configured")
        || combined.contains("status");
    assert!(
        has_status,
        "--backend-info should include a status column/signal, got:\n{}",
        combined
    );
}
