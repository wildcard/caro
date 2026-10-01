//! Regression guard for #1177 / #1216: every key `caro config show` prints
//! must be settable and readable through `caro config set/get`, including the
//! `telemetry.enabled` command the first-run consent screen tells users to run.

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

#[test]
fn consent_screen_command_disables_telemetry() {
    let dir = TempDir::new().unwrap();
    ok(dir.path(), &["config", "set", "telemetry.enabled", "false"]);
    let got = ok(dir.path(), &["config", "get", "telemetry.enabled"]);
    assert!(got.contains("false"), "got: {got}");

    ok(dir.path(), &["config", "set", "telemetry", "true"]);
    let got = ok(dir.path(), &["config", "get", "telemetry"]);
    assert!(got.contains("true"), "got: {got}");
}

#[test]
fn keys_shown_by_config_show_are_settable() {
    let dir = TempDir::new().unwrap();
    for (key, value) in [
        ("log_level", "debug"),
        ("cache_max_size", "20"),
        ("log_rotation", "14"),
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
        // The key must be recognised: the rejection is about the value.
        assert!(
            !stderr.contains("Unknown config key"),
            "`{key}` should be a known key, got: {stderr}"
        );
    }
}
