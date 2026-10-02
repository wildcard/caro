//! Black-box CLI tests for `caro config set/get telemetry.*`.
//!
//! Regression test for #1292: the consent flow advertises
//! `caro config set telemetry.enabled false`, so the CLI needs to accept,
//! persist, and display telemetry config keys consistently.

use assert_cmd::Command;

fn caro_cmd(home: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_caro"));
    cmd.env("HOME", home).env_remove("XDG_CONFIG_HOME");
    cmd
}

#[test]
fn config_set_and_get_telemetry_enabled() {
    let home = tempfile::tempdir().expect("tempdir");

    caro_cmd(home.path())
        .args(["config", "set", "telemetry.enabled", "false"])
        .assert()
        .success()
        .stdout(predicates::str::contains("telemetry.enabled"));

    caro_cmd(home.path())
        .args(["config", "get", "telemetry.enabled"])
        .assert()
        .success()
        .stdout(predicates::str::contains("telemetry.enabled: false"));
}

#[test]
fn config_set_and_get_telemetry_first_run() {
    let home = tempfile::tempdir().expect("tempdir");

    caro_cmd(home.path())
        .args(["config", "set", "telemetry.first_run", "false"])
        .assert()
        .success()
        .stdout(predicates::str::contains("telemetry.first_run"));

    caro_cmd(home.path())
        .args(["config", "get", "telemetry.first_run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("telemetry.first_run: false"));
}

#[test]
fn config_set_telemetry_underscore_alias() {
    let home = tempfile::tempdir().expect("tempdir");

    caro_cmd(home.path())
        .args(["config", "set", "telemetry_enabled", "false"])
        .assert()
        .success();

    caro_cmd(home.path())
        .args(["config", "get", "telemetry_enabled"])
        .assert()
        .success()
        .stdout(predicates::str::contains("telemetry.enabled: false"));
}

#[test]
fn config_set_telemetry_rejects_invalid_boolean() {
    let home = tempfile::tempdir().expect("tempdir");

    caro_cmd(home.path())
        .args(["config", "set", "telemetry.enabled", "not-a-bool"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid boolean"));
}

#[test]
fn config_show_includes_telemetry_first_run() {
    let home = tempfile::tempdir().expect("tempdir");

    caro_cmd(home.path())
        .args(["config", "set", "telemetry.first_run", "false"])
        .assert()
        .success();

    caro_cmd(home.path())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicates::str::contains("telemetry.first_run"));
}
