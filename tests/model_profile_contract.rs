//! Contract for `[models.<name>]` profiles (google/ax "models as named
//! resources"): `--model-profile` / `CARO_MODEL_PROFILE` select a profile,
//! an unknown name is refused, and the profile's `api_key_env` reaches the
//! backend that reads it.
//!
//! Unix-only for the same reason as `config_cli_keys.rs`: on Windows the
//! config dir ignores env vars, so the test could not be isolated.
#![cfg(unix)]

use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn caro(xdg: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_caro"));
    cmd.args(args)
        .env("XDG_CONFIG_HOME", xdg)
        .env("HOME", xdg)
        .env("CARO_TELEMETRY_ENABLED", "false")
        .env_remove("CARO_MODEL_PROFILE")
        .env_remove("CARO_BACKEND")
        .env_remove("CARO_MODEL");
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.output().expect("failed to run caro")
}

/// Write `body` as the config.toml `config set` reports, and return the dir.
fn with_config(body: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    let out = caro(dir.path(), &["config", "set", "log_level", "info"], &[]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let path = stdout
        .lines()
        .find_map(|l| l.split_once("Config saved to: "))
        .unwrap_or_else(|| panic!("no saved path in: {stdout}"))
        .1
        .split('\u{1b}')
        .next()
        .unwrap()
        .trim()
        .to_string();
    std::fs::write(path, body).unwrap();
    dir
}

const CONFIG: &str = r#"
[telemetry]
enabled = false
first_run = false

[models.horde]
backend = "ai-horde"
api_key_env = "CARO_TEST_PROFILE_KEY_UNSET"
"#;

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn unknown_profile_is_refused_and_names_the_defined_ones() {
    let dir = with_config(CONFIG);
    let out = caro(
        dir.path(),
        &["--model-profile", "nope", "show disk usage"],
        &[],
    );
    assert!(!out.status.success());
    let err = stderr(&out);
    assert!(err.contains("\"nope\"") && err.contains("horde"), "{err}");
}

#[test]
fn env_var_selects_the_profile_too() {
    let dir = with_config(CONFIG);
    let out = caro(
        dir.path(),
        &["show disk usage"],
        &[("CARO_MODEL_PROFILE", "nope")],
    );
    assert!(!out.status.success());
    assert!(stderr(&out).contains("\"nope\""), "{}", stderr(&out));
}

#[cfg(feature = "remote-backends")]
#[test]
fn profile_key_env_reaches_the_backend() {
    // The named variable is unset, so building the AI-Horde backend must
    // fail and name it. That proves the profile chose the backend and
    // its key source; no request leaves the machine.
    let dir = with_config(CONFIG);
    let out = caro(
        dir.path(),
        &["--model-profile", "horde", "show disk usage"],
        &[],
    );
    assert!(!out.status.success());
    let err = stderr(&out);
    assert!(err.contains("CARO_TEST_PROFILE_KEY_UNSET"), "{err}");
}

#[test]
fn bad_profile_is_refused_at_load() {
    let dir = with_config(concat!(
        "[telemetry]\nfirst_run = false\n\n",
        "[models.p]\nbackend = \"ollama\"\napi_key_env = \"OLLAMA_KEY\"\n",
    ));
    let out = caro(dir.path(), &["show disk usage"], &[]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("only ai-horde"), "{}", stderr(&out));
}
