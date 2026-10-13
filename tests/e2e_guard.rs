//! E2E regression guard for `caro guard` (spec 011 / ADR-020).
//!
//! Drives the real binary with harness hook payloads on stdin and pins the
//! output protocol each harness relies on.
use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

/// A `caro guard` invocation isolated from the developer's own config and
/// data dirs.
fn guard(home: &TempDir, args: &[&str], stdin: &str) -> std::process::Output {
    guard_cmd(home, args).write_stdin(stdin).output().unwrap()
}

fn guard_cmd(home: &TempDir, args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("caro").unwrap();
    cmd.arg("guard")
        .args(args)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_DATA_HOME", home.path().join("data"))
        .env_remove("CARO_GUARD_MODE")
        .env_remove("CARO_GUARD_JUDGE");
    cmd
}

fn claude_payload(cmd: &str) -> String {
    serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": { "command": cmd },
        "session_id": "e2e",
    })
    .to_string()
}

fn stdout_json(out: &std::process::Output) -> Value {
    serde_json::from_slice(&out.stdout).expect("stdout is JSON")
}

#[test]
fn enforce_denies_critical_for_claude() {
    let home = TempDir::new().unwrap();
    let out = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        &claude_payload("rm -rf /"),
    );
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
}

#[test]
fn enforce_asks_on_high_for_grok_camel_case() {
    let home = TempDir::new().unwrap();
    let payload = r#"{"hookEventName":"pre_tool_use","hook_event_name":"PreToolUse",
        "toolName":"run_terminal_command","toolInput":{"command":"chmod -R 777 /etc"},
        "sessionId":"g"}"#;
    let out = guard(
        &home,
        &["--harness", "grok", "--mode", "enforce", "--no-log"],
        payload,
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        stdout_json(&out)["hookSpecificOutput"]["permissionDecision"],
        "ask"
    );
}

#[test]
fn enforce_is_silent_for_safe_and_non_shell_calls() {
    let home = TempDir::new().unwrap();
    let safe = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        &claude_payload("ls -la"),
    );
    assert!(safe.stdout.is_empty());
    assert_eq!(safe.status.code(), Some(0));

    let read = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        r#"{"tool_name":"Read","tool_input":{"file_path":"/etc/shadow"}}"#,
    );
    assert!(read.stdout.is_empty());
}

#[test]
fn shadow_is_default_logs_and_changes_nothing() {
    let home = TempDir::new().unwrap();
    let log = home.path().join("decisions.jsonl");
    let log_arg = log.to_str().unwrap();

    let out = guard(&home, &["--log", log_arg], &claude_payload("rm -rf /"));
    assert!(out.stdout.is_empty(), "shadow must not emit a decision");
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stderr).contains("would deny"));

    let line = std::fs::read_to_string(&log).unwrap();
    let rec: Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(rec["mode"], "shadow");
    assert_eq!(rec["verdict"], "deny");
    assert_eq!(rec["emitted"], "none");
    assert_eq!(rec["floor_applied"], true);
    assert_eq!(rec["source"], "static");

    let report = Command::cargo_bin("caro")
        .unwrap()
        .args(["guard", "report", "--log", log_arg])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&report.stdout);
    assert!(text.contains("decisions: 1"), "{text}");
    assert!(text.contains("deny"), "{text}");
}

#[test]
fn generic_harness_uses_exit_codes() {
    let home = TempDir::new().unwrap();
    let deny = guard(
        &home,
        &["--harness", "generic", "--mode", "enforce", "--no-log"],
        r#"{"command":"rm -rf /"}"#,
    );
    assert_eq!(deny.status.code(), Some(2));
    assert_eq!(stdout_json(&deny)["verdict"], "deny");

    let ask = guard(
        &home,
        &["--harness", "opencode", "--mode", "enforce", "--no-log"],
        r#"{"command":"chmod -R 777 /etc"}"#,
    );
    assert_eq!(ask.status.code(), Some(3));
}

#[test]
fn env_var_selects_enforce_and_bad_json_fails_toward_ask() {
    let home = TempDir::new().unwrap();
    let out = Command::cargo_bin("caro")
        .unwrap()
        .args(["guard", "--no-log"])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("CARO_GUARD_MODE", "enforce")
        .write_stdin("{not json")
        .output()
        .unwrap();
    assert_eq!(
        stdout_json(&out)["hookSpecificOutput"]["permissionDecision"],
        "ask"
    );
}

#[test]
fn never_emits_allow() {
    let home = TempDir::new().unwrap();
    for cmd in [
        "ls",
        "echo ok",
        "cargo test",
        "rm -rf /",
        "chmod -R 777 /etc",
    ] {
        let out = guard(
            &home,
            &["--mode", "enforce", "--no-log"],
            &claude_payload(cmd),
        );
        assert!(
            !String::from_utf8_lossy(&out.stdout).contains("\"allow\""),
            "{cmd}"
        );
    }
}

#[test]
fn broken_config_asks_without_leaking_config_text() {
    let home = TempDir::new().unwrap();
    let cfg_dir = home.path().join("config").join("caro");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    // Malformed TOML whose offending line carries a token-like value.
    let marker = ["leak", "Marker", "7731"].concat();
    std::fs::write(
        cfg_dir.join("config.toml"),
        format!("[ai]\napi_token = \"{marker}\n[broken"),
    )
    .unwrap();
    let log = home.path().join("d.jsonl");

    let out = guard(
        &home,
        &["--mode", "enforce", "--log", log.to_str().unwrap()],
        &claude_payload("ls"),
    );
    assert_eq!(
        stdout_json(&out)["hookSpecificOutput"]["permissionDecision"],
        "ask"
    );
    let logged = std::fs::read_to_string(&log).unwrap();
    assert!(!logged.contains(&marker), "{logged}");
    assert!(!String::from_utf8_lossy(&out.stderr).contains(&marker));
}

// ── `--judge` (spec 011 follow-up 3) ─────────────────────────────────────

fn last_record(log: &std::path::Path) -> Value {
    let text = std::fs::read_to_string(log).unwrap();
    serde_json::from_str(text.lines().last().unwrap()).unwrap()
}

#[test]
fn unknown_judge_keeps_the_static_verdict_and_logs_why() {
    let home = TempDir::new().unwrap();
    let log = home.path().join("d.jsonl");
    let args = [
        "--harness",
        "generic",
        "--mode",
        "enforce",
        "--judge",
        "nonsense",
        "--log",
        log.to_str().unwrap(),
    ];

    let pass = guard(&home, &args, r#"{"command":"ls -la"}"#);
    assert_eq!(pass.status.code(), Some(0), "a broken judge never asks");
    let rec = last_record(&log);
    assert_eq!(rec["source"], "static+judge_error");
    assert!(rec["judge_error"]
        .as_str()
        .unwrap()
        .contains("unknown judge"));

    let deny = guard(&home, &args, r#"{"command":"rm -rf /"}"#);
    assert_eq!(deny.status.code(), Some(2), "static rules still deny");
    assert_eq!(last_record(&log)["source"], "static");
}

/// `$CARO_GUARD_JUDGE` selects the judge; a build without the remote
/// backends says so in the log instead of failing the hook.
#[cfg(not(feature = "remote-backends"))]
#[test]
fn judge_from_env_needs_remote_backends() {
    let home = TempDir::new().unwrap();
    let log = home.path().join("d.jsonl");
    let out = Command::cargo_bin("caro")
        .unwrap()
        .args(["guard", "--harness", "generic", "--mode", "enforce"])
        .args(["--log", log.to_str().unwrap()])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_DATA_HOME", home.path().join("data"))
        .env_remove("CARO_GUARD_MODE")
        .env("CARO_GUARD_JUDGE", "ollama")
        .write_stdin(r#"{"command":"ls"}"#)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let rec = last_record(&log);
    assert_eq!(rec["source"], "static+judge_error");
    assert!(rec["judge_error"]
        .as_str()
        .unwrap()
        .contains("remote-backends"));
}

/// A local Ollama judge escalates a command the static rules let through.
///
/// Not on Windows: `dirs::config_dir()` there is the Known Folder for roaming
/// AppData, which the test cannot point at a temporary config.
#[cfg(all(feature = "remote-backends", not(windows)))]
#[tokio::test(flavor = "multi_thread")]
async fn ollama_judge_escalates_a_static_pass_to_ask() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": r#"{"risk": "high", "reason": "deletes the cache", "confidence": 0.9}"#,
            "done": true
        })))
        .mount(&server)
        .await;

    let home = TempDir::new().unwrap();
    let toml = format!("[backends]\nollama_url = \"{}\"\n", server.uri());
    // dirs::config_dir(): XDG on Linux, ~/Library/Application Support on macOS.
    for dir in [
        home.path().join("config").join("caro"),
        home.path().join("Library/Application Support/caro"),
    ] {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.toml"), &toml).unwrap();
    }
    let log = home.path().join("d.jsonl");
    let out = guard(
        &home,
        &[
            "--harness",
            "generic",
            "--mode",
            "enforce",
            "--judge",
            "ollama",
            "--log",
            log.to_str().unwrap(),
        ],
        r#"{"command":"ls -la"}"#,
    );
    assert_eq!(out.status.code(), Some(3), "{:?}", out);
    let rec = last_record(&log);
    assert_eq!(rec["source"], "static+judge");
    assert_eq!(rec["judge_risk"], "high");
    assert_eq!(rec["verdict"], "ask");
    let body = stdout_json(&out);
    assert!(!body["reason"]
        .as_str()
        .unwrap()
        .contains("deletes the cache"));
}

/// #1647: a loopback judge is called directly even when a proxy is set, so
/// the endpoint Caro checks is the endpoint it talks to. Not on Windows, for
/// the same config-dir reason as the test above.
#[cfg(all(feature = "remote-backends", not(windows)))]
#[tokio::test(flavor = "multi_thread")]
async fn loopback_judge_bypasses_an_http_proxy() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": r#"{"risk": "high", "reason": "r", "confidence": 0.9}"#,
            "done": true
        })))
        .mount(&server)
        .await;

    let home = TempDir::new().unwrap();
    let toml = format!("[backends]\nollama_url = \"{}\"\n", server.uri());
    for dir in [
        home.path().join("config").join("caro"),
        home.path().join("Library/Application Support/caro"),
    ] {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.toml"), &toml).unwrap();
    }
    let log = home.path().join("d.jsonl");
    // Port 1 refuses connections: a proxied judge call fails.
    let dead_proxy = "http://127.0.0.1:1";
    let out = guard_cmd(
        &home,
        &[
            "--harness",
            "generic",
            "--mode",
            "enforce",
            "--judge",
            "ollama",
            "--log",
            log.to_str().unwrap(),
        ],
    )
    .env("HTTP_PROXY", dead_proxy)
    .env("http_proxy", dead_proxy)
    .env("ALL_PROXY", dead_proxy)
    .env("all_proxy", dead_proxy)
    .env_remove("NO_PROXY")
    .env_remove("no_proxy")
    .write_stdin(r#"{"command":"ls -la"}"#)
    .output()
    .unwrap();
    let rec = last_record(&log);
    assert_eq!(rec["source"], "static+judge", "{:?} {rec}", out);
    assert_eq!(out.status.code(), Some(3), "{:?}", out);
}
