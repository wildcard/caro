//! E2E regression guard for `caro guard` (spec 011 / ADR-018).
//!
//! Drives the real binary with harness hook payloads on stdin and pins the
//! output protocol each harness relies on.
use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

/// A `caro guard` invocation isolated from the developer's own config and
/// data dirs.
fn guard(home: &TempDir, args: &[&str], stdin: &str) -> std::process::Output {
    Command::cargo_bin("caro")
        .unwrap()
        .arg("guard")
        .args(args)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_DATA_HOME", home.path().join("data"))
        .env_remove("CARO_GUARD_MODE")
        .write_stdin(stdin)
        .output()
        .unwrap()
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
