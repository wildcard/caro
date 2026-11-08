//! Regression guard for #1723: `caro "<prompt>"` and `caro -p "<prompt>"` must
//! not block on stdin when the prompt is already on the command line, even if
//! stdin is a pipe that never reaches EOF (coder-agent Bash tools, CI, scripts).
//! This is the call the bundled `caro-shell` skill makes. `caro ai --once` had
//! the same bug and is guarded by `tests/ai_once_stdin.rs` (#1499).
//!
//! The test asserts on termination, not on success, so it does not depend on
//! which backend the runner can load.
#![cfg(unix)]

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn assert_returns_with_open_stdin(args: &[&str]) {
    let dir = TempDir::new().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_caro"))
        .args(args)
        .env("XDG_CONFIG_HOME", dir.path())
        .env("XDG_DATA_HOME", dir.path())
        .env("HOME", dir.path())
        .env("CARO_TELEMETRY_ENABLED", "false")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Hold the pipe open and never write to it or close it.
    let _stdin = child.stdin.take();

    let start = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            return;
        }
        if start.elapsed() > Duration::from_secs(60) {
            child.kill().ok();
            panic!("`caro {}` blocked on an open stdin pipe", args.join(" "));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn trailing_prompt_does_not_wait_for_open_stdin() {
    assert_returns_with_open_stdin(&["--dry-run", "list", "files"]);
}

#[test]
fn flag_prompt_does_not_wait_for_open_stdin() {
    assert_returns_with_open_stdin(&["--dry-run", "-p", "list files"]);
}
