//! Regression guard for #1081: which backends a build can reach must match its
//! cargo features, and an unreachable backend must fail loudly.
//!
//! Run both shapes:
//!   cargo test --test backend_features_contract
//!   cargo test --test backend_features_contract --features remote-backends
//!
//! The manual `release-remote` workflow builds a second, opt-in `-remote`
//! binary with `--features remote-backends`. The default build stays minimal. These tests
//! pin both contracts. They make no network calls: no API key is set, so the
//! hosted backends fail on configuration before any request.
//!
//! Unix-only: on Windows `dirs::config_dir()` ignores env vars, so the test
//! could not be isolated from the runner's real config.
#![cfg(unix)]

use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

#[cfg(not(feature = "remote-backends"))]
const REMOTE_BACKENDS: [&str; 9] = [
    "ollama",
    "exo",
    "vllm",
    "mesh",
    "ai-horde",
    "hybrid",
    "grok",
    "claude",
    "openrouter",
];

fn caro(home: &Path, args: &[&str]) -> Output {
    // `env_clear` drops every inherited variable (API keys, CARO_* settings),
    // so a developer's environment cannot change the output or reach a
    // provider. Only PATH is kept, then HOME and the XDG dirs point at the
    // temp dir.
    let path = std::env::var_os("PATH").unwrap_or_default();
    Command::new(env!("CARGO_BIN_EXE_caro"))
        .args(args)
        .env_clear()
        .env("PATH", path)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env("XDG_DATA_HOME", home)
        .env("XDG_CACHE_HOME", home)
        .stdin(Stdio::null())
        .output()
        .expect("failed to run caro")
}

fn combined(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn backend_info(home: &Path) -> String {
    let out = caro(home, &["--backend-info"]);
    assert!(
        out.status.success(),
        "--backend-info failed: {}",
        combined(&out)
    );
    combined(&out)
}

fn hosted_backend_run(home: &Path, name: &str) -> Output {
    // `--no-telemetry` keeps the first-run consent prompt out of the way, so the
    // `-p` prompt path reaches backend selection.
    caro(
        home,
        &[
            "--no-telemetry",
            "--backend",
            name,
            "--dry-run",
            "-p",
            "list pdf files",
        ],
    )
}

#[cfg(not(feature = "remote-backends"))]
#[test]
fn default_build_marks_every_remote_backend_not_compiled() {
    let home = TempDir::new().unwrap();
    let info = backend_info(home.path());
    for name in REMOTE_BACKENDS {
        let row = info
            .lines()
            .find(|l| l.trim_start().starts_with(name))
            .unwrap_or_else(|| panic!("no --backend-info row for {name}:\n{info}"));
        assert!(
            row.contains("not compiled"),
            "default build must mark {name} 'not compiled': {row}"
        );
    }
}

#[cfg(not(feature = "remote-backends"))]
#[test]
fn default_build_rejects_remote_backend_loudly_with_remedy() {
    let home = TempDir::new().unwrap();
    for name in REMOTE_BACKENDS {
        let out = hosted_backend_run(home.path(), name);
        let text = combined(&out);
        assert!(
            !out.status.success(),
            "--backend {name} must not exit 0:\n{text}"
        );
        assert!(
            text.contains("requires the 'remote-backends' feature"),
            "--backend {name} must name the missing feature:\n{text}"
        );
        assert!(
            text.contains("--features remote-backends"),
            "--backend {name} must print the remedy:\n{text}"
        );
    }
}

#[cfg(feature = "remote-backends")]
#[test]
fn remote_build_marks_no_backend_not_compiled() {
    let home = TempDir::new().unwrap();
    let info = backend_info(home.path());
    assert!(
        !info.contains("not compiled"),
        "remote-backends build must compile every listed backend:\n{info}"
    );
}

#[cfg(feature = "remote-backends")]
#[test]
fn remote_build_reports_missing_api_key_for_hosted_backends() {
    let home = TempDir::new().unwrap();
    for (name, var) in [
        ("claude", "ANTHROPIC_API_KEY"),
        ("openrouter", "OPENROUTER_API_KEY"),
        ("grok", "XAI_API_KEY"),
    ] {
        let out = hosted_backend_run(home.path(), name);
        let text = combined(&out);
        assert!(
            !out.status.success(),
            "--backend {name} must not exit 0:\n{text}"
        );
        assert!(
            text.contains(var),
            "--backend {name} must name {var} when it is unset:\n{text}"
        );
        assert!(
            !text.contains("requires the 'remote-backends' feature"),
            "--backend {name} is compiled in; it must not blame the feature:\n{text}"
        );
    }
}
