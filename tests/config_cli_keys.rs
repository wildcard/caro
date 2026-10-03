//! Regression guard for #1177 / #1216: every key `caro config show` prints
//! must be settable and readable through `caro config set/get`, including the
//! `telemetry.enabled` command the first-run consent screen tells users to run.
//!
//! Unix-only: on Windows `dirs::config_dir()` resolves the roaming AppData
//! folder via the known-folder API, ignoring env vars, so the test could not
//! be isolated from the runner's real config.
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

fn ok(xdg: &Path, args: &[&str]) -> String {
    let out = caro(xdg, args);
    assert!(
        out.status.success(),
        "`caro {}` failed\nstdout: {}\nstderr: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Path from the `Config saved to: <path>` line `config set` prints.
fn saved_path(stdout: &str) -> String {
    let line = stdout
        .lines()
        .find_map(|l| l.split_once("Config saved to: "))
        .unwrap_or_else(|| panic!("no saved path in: {stdout}"))
        .1;
    // Drop any trailing ANSI color reset.
    line.split('\u{1b}').next().unwrap().trim().to_string()
}

#[test]
fn consent_screen_command_disables_telemetry() {
    let dir = TempDir::new().unwrap();
    let set = ok(dir.path(), &["config", "set", "telemetry.enabled", "false"]);
    let got = ok(dir.path(), &["config", "get", "telemetry.enabled"]);
    assert!(got.contains("false"), "got: {got}");
    // The explicit choice must count as consent so the first-run prompt
    // cannot later overwrite it. Read the file `config set` reports, since
    // the config dir differs per platform.
    let saved = std::fs::read_to_string(saved_path(&set)).unwrap();
    assert!(saved.contains("first_run = false"), "config: {saved}");

    ok(dir.path(), &["config", "set", "telemetry", "true"]);
    let got = ok(dir.path(), &["config", "get", "telemetry"]);
    assert!(got.contains("true"), "got: {got}");

    ok(dir.path(), &["config", "set", "telemetry-enabled", "false"]);
    let got = ok(dir.path(), &["config", "get", "telemetry-enabled"]);
    assert!(got.contains("false"), "got: {got}");
}

#[test]
fn keys_shown_by_config_show_are_settable() {
    let dir = TempDir::new().unwrap();
    for (key, value) in [
        ("log_level", "debug"),
        ("cache_max_size", "20"),
        ("log_rotation", "14"),
        ("log-level", "debug"),
        ("cache-max-size", "20"),
        ("log-rotation", "14"),
        ("model_name", "test-model"),
    ] {
        ok(dir.path(), &["config", "set", key, value]);
        let got = ok(dir.path(), &["config", "get", key]).to_lowercase();
        assert!(got.contains(value), "{key}: got {got}");
    }
    let show = ok(dir.path(), &["config", "show"]).to_lowercase();
    assert!(show.contains("debug") && show.contains("20 gb") && show.contains("14 days"));
}

#[test]
fn invalid_values_are_rejected() {
    let dir = TempDir::new().unwrap();
    for (key, value) in [
        ("telemetry.enabled", "maybe"),
        ("log_level", "loud"),
        ("cache_max_size", "0"),
        ("log_rotation", "999"),
    ] {
        let out = caro(dir.path(), &["config", "set", key, value]);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !out.status.success(),
            "`config set {key} {value}` should fail"
        );
        // No success check mark for a value that is then rejected.
        assert!(
            !String::from_utf8_lossy(&out.stdout).contains('✓'),
            "`config set {key} {value}` printed success"
        );
        // The key must be recognised: the rejection is about the value.
        assert!(
            !stderr.contains("Unknown config key"),
            "`{key}` should be a known key, got: {stderr}"
        );
    }
}
