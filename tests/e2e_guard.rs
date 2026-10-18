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

/// The six evasions measured in spec 011 ("Known gaps") reached the shell
/// with no opinion before #1054. Segment- and wrapper-aware validation must
/// now give each one a verdict the harness acts on.
#[test]
fn enforce_catches_segment_and_wrapper_evasions() {
    let home = TempDir::new().unwrap();
    for (cmd, want) in [
        ("bash -c 'rm -rf /'", "deny"),
        ("git status; chmod -R 777 ~", "deny"),
        ("find / -delete", "deny"),
        ("echo cm0gLXJmIC8= | base64 -d | sh", "deny"),
        ("eval \"$X\"", "ask"),
        ("python3 -c 'import shutil;shutil.rmtree(\"/\")'", "ask"),
    ] {
        let out = guard(
            &home,
            &["--mode", "enforce", "--no-log"],
            &claude_payload(cmd),
        );
        assert_eq!(out.status.code(), Some(0), "{cmd:?}");
        assert_eq!(
            stdout_json(&out)["hookSpecificOutput"]["permissionDecision"],
            want,
            "{cmd:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// spec 011 §P2.1 / §P2.2: `caro guard test` and `caro guard setup`
// ---------------------------------------------------------------------------

/// `caro guard <args>` with HOME and XDG dirs pointed at a temp dir, no stdin.
fn guard_cmd(home: &TempDir, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("caro")
        .unwrap()
        .arg("guard")
        .args(args)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_DATA_HOME", home.path().join("data"))
        .env_remove("CARO_GUARD_MODE")
        .output()
        .unwrap()
}

#[test]
fn guard_test_denies_critical_with_exit_2_and_json() {
    let home = TempDir::new().unwrap();
    let out = guard_cmd(&home, &["test", "--output", "json", "rm -rf /"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_eq!(v["verdict"], "deny");
    assert_eq!(v["risk"], "critical");
    assert_eq!(v["floor_applied"], true);
    assert_eq!(v["source"], "static");
    assert!(v["latency_us"].is_u64());
    assert!(!out.stdout.windows(7).any(|w| w == b"\"allow\""));
}

#[test]
fn guard_test_asks_on_high_and_is_silent_on_safe() {
    let home = TempDir::new().unwrap();
    let ask = guard_cmd(&home, &["test", "chmod -R 777 /etc"]);
    assert_eq!(ask.status.code(), Some(3));
    let text = String::from_utf8_lossy(&ask.stdout);
    assert!(text.contains("verdict:  ask"), "{text}");

    let safe = guard_cmd(&home, &["test", "ls -la"]);
    assert_eq!(safe.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&safe.stdout).contains("verdict:  none"));
}

#[test]
fn guard_test_redacts_secrets_in_its_own_output() {
    let home = TempDir::new().unwrap();
    let marker = ["hunter", "2", "secret"].concat();
    let out = guard_cmd(
        &home,
        &[
            "test",
            "--output",
            "json",
            &format!("export GITHUB_TOKEN={marker}; ls"),
        ],
    );
    assert!(!String::from_utf8_lossy(&out.stdout).contains(&marker));
}

#[test]
fn guard_setup_dry_run_writes_nothing_and_prints_hook() {
    let home = TempDir::new().unwrap();
    let out = guard_cmd(&home, &["setup", "claude", "--dry-run"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("caro guard --harness claude --mode shadow"),
        "{text}"
    );
    assert!(
        !home.path().join(".claude").exists(),
        "dry run must not write"
    );
}

#[test]
fn guard_setup_is_idempotent_keeps_other_hooks_and_verifies() {
    let home = TempDir::new().unwrap();
    let settings = home.path().join(".claude").join("settings.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(
        &settings,
        r#"{"permissions":{"allow":["Bash(ls:*)"]},"hooks":{"PreToolUse":[{"matcher":"Write","hooks":[{"type":"command","command":"lint.sh"}]}]}}"#,
    )
    .unwrap();

    let first = guard_cmd(&home, &["setup", "claude", "--mode", "enforce"]);
    assert_eq!(
        first.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let second = guard_cmd(&home, &["setup", "claude", "--mode", "enforce"]);
    assert!(String::from_utf8_lossy(&second.stdout).contains("already up to date"));

    let v: Value = serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    assert_eq!(
        v["permissions"]["allow"][0], "Bash(ls:*)",
        "foreign keys kept"
    );
    let pre = v["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(pre.len(), 2, "one foreign hook + one caro hook: {pre:?}");
    assert_eq!(pre[0]["hooks"][0]["command"], "lint.sh");
    assert_eq!(
        pre[1]["hooks"][0]["command"],
        "caro guard --harness claude --mode enforce"
    );

    // --verify drives the `caro` on $PATH; point PATH at the test binary's dir.
    let bin = Command::cargo_bin("caro").unwrap().get_program().to_owned();
    let bin_dir = std::path::Path::new(&bin).parent().unwrap().to_owned();
    let verify = Command::cargo_bin("caro")
        .unwrap()
        .args(["guard", "setup", "claude", "--verify"])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("PATH", &bin_dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&verify.stdout);
    assert_eq!(
        verify.status.code(),
        Some(0),
        "{text}\n{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(text.contains("ADAPTER VERIFIED"), "{text}");
    assert!(
        !text.contains("HOST VERIFIED"),
        "caro cannot observe host ingestion"
    );
}

#[test]
fn guard_setup_verify_fails_when_not_installed() {
    let home = TempDir::new().unwrap();
    let out = guard_cmd(&home, &["setup", "claude", "--verify"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("NOT VERIFIED"));
}

#[test]
fn guard_setup_opencode_writes_embedded_plugin() {
    let home = TempDir::new().unwrap();
    let out = guard_cmd(&home, &["setup", "opencode"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let plugin = home
        .path()
        .join("config")
        .join("opencode")
        .join("plugin")
        .join("caro-guard.ts");
    let text = std::fs::read_to_string(&plugin).unwrap();
    assert!(text.contains("tool.execute.before"));
}

// ---------------------------------------------------------------------------
// spec 011 §P2.3: file-tool coverage (protected paths)
// ---------------------------------------------------------------------------

fn file_payload(tool: &str, path: &str) -> String {
    serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": tool,
        "tool_input": { "file_path": path },
        "session_id": "e2e",
        "cwd": "/repo",
    })
    .to_string()
}

#[test]
fn enforce_asks_on_protected_path_read() {
    let home = TempDir::new().unwrap();
    let key = format!("{}/.ssh/id_rsa", home.path().display());
    let out = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        &file_payload("Read", &key),
    );
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "ask");
    let reason = v["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .unwrap();
    assert!(reason.contains("SSH"), "{reason}");
    assert!(
        !reason.contains("id_rsa"),
        "reason must not echo the path: {reason}"
    );

    // Relative dotenv resolved against cwd; Grep/Glob use `path`.
    let dotenv = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        &file_payload("Edit", "apps/web/.env.local"),
    );
    assert_eq!(
        stdout_json(&dotenv)["hookSpecificOutput"]["permissionDecision"],
        "ask"
    );
    let grep = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        &serde_json::json!({"tool_name":"Grep","tool_input":{"pattern":"AKIA","path":format!("{}/.aws", home.path().display())}}).to_string(),
    );
    assert_eq!(
        stdout_json(&grep)["hookSpecificOutput"]["permissionDecision"],
        "ask"
    );
}

#[test]
fn routine_read_stays_silent_and_unlogged() {
    let home = TempDir::new().unwrap();
    let log = home.path().join("decisions.jsonl");
    for (tool, path) in [
        ("Read", "src/main.rs"),
        ("Edit", "/repo/README.md"),
        ("Write", "Cargo.toml"),
        ("Grep", "/repo/src"),
        ("Glob", "**/*.rs"),
    ] {
        let out = guard(
            &home,
            &["--mode", "enforce", "--log", log.to_str().unwrap()],
            &file_payload(tool, path),
        );
        assert!(
            out.stdout.is_empty(),
            "{tool} {path}: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        assert_eq!(out.status.code(), Some(0));
    }
    assert!(!log.exists(), "routine file calls must not be logged");
}

#[test]
fn writing_own_decision_log_is_denied_and_shadow_logs_tool_and_path() {
    let home = TempDir::new().unwrap();
    let log = home.path().join("decisions.jsonl");
    let log_arg = log.to_str().unwrap();

    let deny = guard(
        &home,
        &["--mode", "enforce", "--log", log_arg],
        &file_payload("Write", log_arg),
    );
    assert_eq!(
        stdout_json(&deny)["hookSpecificOutput"]["permissionDecision"],
        "deny"
    );

    let key = format!("{}/.ssh/id_ed25519", home.path().display());
    let shadow = guard(&home, &["--log", log_arg], &file_payload("Read", &key));
    assert!(shadow.stdout.is_empty(), "shadow emits nothing");
    assert!(String::from_utf8_lossy(&shadow.stderr).contains("would ask"));

    let last = std::fs::read_to_string(&log).unwrap();
    let rec: Value = serde_json::from_str(last.lines().last().unwrap()).unwrap();
    assert_eq!(rec["tool"], "Read");
    assert_eq!(rec["verdict"], "ask");
    assert_eq!(rec["emitted"], "none");
    assert!(rec["path"].as_str().unwrap().ends_with("id_ed25519"));
    assert!(rec.get("command").is_none());

    let report = Command::cargo_bin("caro")
        .unwrap()
        .args(["guard", "report", "--log", log_arg])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&report.stdout);
    assert!(text.contains("by tool:"), "{text}");
    assert!(text.contains("read 1"), "{text}");
}

#[test]
fn user_protected_paths_are_additive() {
    let home = TempDir::new().unwrap();
    let cfg_dir = home.path().join("config").join("caro");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::write(
        cfg_dir.join("config.toml"),
        "safety_level = \"moderate\"\nlog_level = \"info\"\ncache_max_size_gb = 10\nlog_rotation_days = 7\n\n[safety]\nprotected_paths = [\"**/vault/**\"]\n",
    )
    .unwrap();
    let user = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        &file_payload("Read", "/srv/vault/token"),
    );
    assert_eq!(
        stdout_json(&user)["hookSpecificOutput"]["permissionDecision"],
        "ask"
    );
    let builtin = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        &file_payload("Read", "/repo/.env"),
    );
    assert_eq!(
        stdout_json(&builtin)["hookSpecificOutput"]["permissionDecision"],
        "ask",
        "built-ins stay when the user adds globs"
    );
}

// ---------------------------------------------------------------------------
// spec 011 §P2.4: hash-chained log and `caro guard audit`
// ---------------------------------------------------------------------------

#[test]
fn audit_verify_detects_a_one_byte_edit_and_tail_prints_records() {
    let home = TempDir::new().unwrap();
    let log = home.path().join("decisions.jsonl");
    let log_arg = log.to_str().unwrap();
    for cmd in ["rm -rf /", "chmod -R 777 /etc", "ls"] {
        guard(&home, &["--log", log_arg], &claude_payload(cmd));
    }

    let text = std::fs::read_to_string(&log).unwrap();
    let first: Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
    assert_eq!(
        first["prev_hash"],
        Value::Null,
        "first record has no predecessor"
    );
    assert_eq!(first["hash"].as_str().unwrap().len(), 64);
    let second: Value = serde_json::from_str(text.lines().nth(1).unwrap()).unwrap();
    assert_eq!(second["prev_hash"], first["hash"]);

    let ok = guard_cmd(&home, &["audit", "verify", "--log", log_arg]);
    assert_eq!(
        ok.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&ok.stdout)
    );
    assert!(String::from_utf8_lossy(&ok.stdout).contains("CHAIN INTACT"));

    let tail = guard_cmd(&home, &["audit", "tail", "--log", log_arg, "-n", "2"]);
    let tail_text = String::from_utf8_lossy(&tail.stdout);
    assert_eq!(tail_text.lines().count(), 2, "{tail_text}");
    assert!(tail_text.contains("chmod"), "{tail_text}");

    // Flip one byte inside the second record's command.
    let tampered = text.replacen("chmod -R 777", "chmod -R 700", 1);
    assert_ne!(text, tampered);
    std::fs::write(&log, tampered).unwrap();
    let bad = guard_cmd(&home, &["audit", "verify", "--log", log_arg]);
    assert_eq!(bad.status.code(), Some(1));
    let out = String::from_utf8_lossy(&bad.stdout);
    assert!(out.contains("CHAIN BROKEN at line 2"), "{out}");

    // `report` still works under `audit`.
    let report = guard_cmd(&home, &["audit", "report", "--log", log_arg]);
    assert!(String::from_utf8_lossy(&report.stdout).contains("decisions: 3"));
}

#[test]
fn guard_setup_verify_rejects_a_malformed_stored_command() {
    let home = TempDir::new().unwrap();
    let settings = home.path().join(".claude").join("settings.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(
        &settings,
        r#"{"hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"caro guard --harness claude --mode broken"}]}]}}"#,
    )
    .unwrap();
    let bin = Command::cargo_bin("caro").unwrap().get_program().to_owned();
    let bin_dir = std::path::Path::new(&bin).parent().unwrap().to_owned();
    let verify = Command::cargo_bin("caro")
        .unwrap()
        .args(["guard", "setup", "claude", "--verify"])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("PATH", &bin_dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&verify.stdout);
    assert_eq!(verify.status.code(), Some(1), "{text}");
    assert!(text.contains("unknown mode `broken`"), "{text}");
    assert!(text.contains("NOT VERIFIED"), "{text}");
}

#[test]
fn glob_pattern_into_protected_dir_asks_and_broken_config_asks_for_file_tools() {
    let home = TempDir::new().unwrap();
    let glob = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        &serde_json::json!({"tool_name":"Glob","tool_input":{"pattern":format!("{}/.ssh/**", home.path().display())}}).to_string(),
    );
    assert_eq!(
        stdout_json(&glob)["hookSpecificOutput"]["permissionDecision"],
        "ask"
    );

    // Malformed config: a file call must fail toward the human, not drop
    // the user's protected_paths silently.
    let cfg_dir = home.path().join("config").join("caro");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::write(cfg_dir.join("config.toml"), "[safety\nbroken").unwrap();
    let out = guard(
        &home,
        &["--mode", "enforce", "--no-log"],
        &file_payload("Read", "src/main.rs"),
    );
    assert_eq!(
        stdout_json(&out)["hookSpecificOutput"]["permissionDecision"],
        "ask"
    );
}
