//! Regression guard for #1566: an invalid `config.toml` must never be
//! replaced with defaults. The first-run telemetry save used to load the
//! config with `.ok().unwrap_or_default()` and then write the defaults back,
//! so one typo silently erased every setting the user had.
//!
//! Unix-only for the same reason as `config_cli_keys.rs`: on Windows the
//! config dir ignores env vars, so the test could not be isolated.
#![cfg(unix)]

use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn caro(xdg: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caro"))
        .args(args)
        .env("XDG_CONFIG_HOME", xdg)
        .env("HOME", xdg)
        .env("CARO_TELEMETRY_ENABLED", "false")
        .output()
        .expect("failed to run caro")
}

/// Path from the `Config saved to: <path>` line `config set` prints.
fn config_path(xdg: &Path) -> String {
    let out = caro(xdg, &["config", "set", "log_level", "debug"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout
        .lines()
        .find_map(|l| l.split_once("Config saved to: "))
        .unwrap_or_else(|| panic!("no saved path in: {stdout}"))
        .1;
    line.split('\u{1b}').next().unwrap().trim().to_string()
}

#[test]
fn invalid_config_is_left_untouched_by_a_run() {
    let dir = TempDir::new().unwrap();
    let path = config_path(dir.path());

    // A real user file with one bad value: still first_run, so the run
    // reaches the telemetry save path.
    let invalid = "log_level = \"loud\"\nmodel_name = \"keep-me\"\n";
    std::fs::write(&path, invalid).unwrap();

    // Non-interactive output takes the silent first-run save branch.
    let out = caro(
        dir.path(),
        &["--output", "json", "--dry-run", "show disk usage"],
    );

    // The run must reach the config load and report it, so an early
    // startup failure cannot pass this guard by never touching the file.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "run with a bad config succeeded");
    assert!(
        stderr.contains("Failed to load configuration"),
        "stderr: {stderr}"
    );

    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(after, invalid, "config.toml was rewritten");
}
