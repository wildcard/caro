//! Regression guard for #1499: `caro ai --once "<prompt>"` must not block on
//! stdin when the prompt is already given as trailing words, even if stdin is
//! a pipe that never reaches EOF (scripts, CI, subprocess calls).

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[test]
fn trailing_prompt_does_not_wait_for_open_stdin() {
    let dir = TempDir::new().unwrap();
    let caro = env!("CARGO_BIN_EXE_caro");
    let env = [
        ("XDG_CONFIG_HOME", dir.path()),
        ("XDG_DATA_HOME", dir.path()),
        ("HOME", dir.path()),
    ];

    // Write a default config, then disable the AI feature so `--once` returns
    // right after prompt resolution instead of initializing a model backend.
    let out = Command::new(caro)
        .args(["config", "set", "safety", "moderate"])
        .envs(env)
        .env("CARO_TELEMETRY_ENABLED", "false")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let path = stdout
        .lines()
        .find_map(|l| l.split_once("Config saved to: "))
        .expect("config path")
        .1
        .split('\u{1b}')
        .next()
        .unwrap()
        .trim()
        .to_string();
    let cfg = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        cfg.replacen("[ai]\nenabled = true", "[ai]\nenabled = false", 1),
    )
    .unwrap();

    // stdin is a pipe we hold open and never write to or close.
    let mut child = Command::new(caro)
        .args(["ai", "--once", "list", "files"])
        .envs(env)
        .env("CARO_TELEMETRY_ENABLED", "false")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _stdin = child.stdin.take();

    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if start.elapsed() > Duration::from_secs(20) {
            child.kill().ok();
            panic!("`caro ai --once list files` blocked on an open stdin pipe");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    // It reached the config check, i.e. it resolved the trailing prompt.
    assert!(!status.success());
    assert!(
        stderr.contains("AI feature is disabled"),
        "stderr: {stderr}"
    );
}
