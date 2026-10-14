//! Process-level contract for `--backend select-files` (deterministic, no model).
//!
//! Runs the real binary with a private HOME. Nothing here starts a model,
//! opens a network connection or executes a generated command: every run is
//! `--dry-run`.
#![cfg(unix)]

use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

fn caro(home: &TempDir, envs: &[(&str, &str)], args: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_caro"));
    c.args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .env("XDG_CACHE_HOME", home.path().join("cache"))
        .env("CARO_TELEMETRY_ENABLED", "false");
    for (k, v) in envs {
        c.env(k, v);
    }
    c.output().expect("run caro")
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn json(home: &TempDir, envs: &[(&str, &str)], req: &str) -> serde_json::Value {
    let out = caro(
        home,
        envs,
        &[
            "--backend",
            "select-files",
            "--dry-run",
            "-o",
            "json",
            "-v",
            req,
        ],
    );
    assert!(
        out.status.success(),
        "{req}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let start = text.find('{').expect("json");
    serde_json::from_str(&text[start..]).unwrap_or_else(|e| panic!("{e}: {text}"))
}

fn command(v: &serde_json::Value) -> &str {
    v["generated_command"].as_str().unwrap()
}

#[test]
fn seven_day_python_request_compiles_a_bounded_window() {
    let h = TempDir::new().unwrap();
    let before = now();
    let v = json(&h, &[], "find python files modified in the last 7 days");
    let after = now();
    let cmd = command(&v);
    let re = regex::Regex::new(
        r"^/(usr/)?bin/find -P \. -type f -iname '\*\.py' -newermt @(\d+) '!' -newermt @(\d+) -print0$",
    )
    .unwrap();
    let c = re.captures(cmd).unwrap_or_else(|| panic!("shape: {cmd}"));
    let cutoff: i64 = c[2].parse().unwrap();
    let t0: i64 = c[3].parse().unwrap();
    assert_eq!(t0 - cutoff, 7 * 24 * 3600, "elapsed seven days");
    assert!((before..=after).contains(&t0), "T0 is the generation time");
    let ex = v["explanation"].as_str().unwrap();
    assert!(ex.starts_with("output-framing: nul-separated"));
    assert!(ex.contains("window frozen at generation"));
    assert_eq!(v["blocked_reason"], serde_json::Value::Null);
    assert_eq!(v["requires_confirmation"], false);
    let dbg = v["debug_info"].as_str().unwrap();
    assert!(dbg.contains("intent-deterministic"));
    assert!(dbg.contains("Model: none"));
}

#[test]
fn plain_python_and_pdf_listings() {
    let h = TempDir::new().unwrap();
    for (req, pat) in [
        ("list all python files", "'*.py'"),
        ("list all PDF files", "'*.pdf'"),
    ] {
        let v = json(&h, &[], req);
        let cmd = command(&v);
        assert!(cmd.ends_with(&format!("-iname {pat} -print0")), "{cmd}");
        assert!(!cmd.contains("-newermt"), "{cmd}");
    }
}

#[test]
fn filler_only_and_root_only_requests_never_compile_a_listing() {
    let h = TempDir::new().unwrap();
    for req in ["please", "in docs", "find me please", "named foo"] {
        let out = caro(&h, &[], &["--backend", "select-files", "--dry-run", req]);
        let all = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!out.status.success(), "{req}: {all}");
        assert!(!all.contains("-print0"), "{req}: {all}");
        assert!(all.contains("NoSubject"), "{req}: {all}");
    }
}

#[test]
fn calendar_windows_ask_for_a_rolling_window() {
    let h = TempDir::new().unwrap();
    let out = caro(
        &h,
        &[],
        &[
            "--backend",
            "select-files",
            "--dry-run",
            "python files modified today",
        ],
    );
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(all.contains("rolling window"), "{all}");
    assert!(!all.contains("-print0"), "{all}");
}

#[test]
fn model_flags_conflict_and_inherited_models_are_ignored() {
    let h = TempDir::new().unwrap();
    for extra in [
        vec!["--force-llm"],
        vec!["--advisor", "claude"],
        vec!["--model-name", "some-model"],
    ] {
        let mut args = vec!["--backend", "select-files", "--dry-run"];
        args.extend(extra.iter().copied());
        args.push("list all python files");
        let out = caro(&h, &[], &args);
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "{extra:?}");
        assert!(
            err.contains("cannot be used with --backend select-files"),
            "{extra:?}: {err}"
        );
        assert!(!String::from_utf8_lossy(&out.stdout).contains("-print0"));
    }
    // The same conflict through the environment-selected backend.
    let out = caro(
        &h,
        &[("CARO_BACKEND", "select-files")],
        &["--dry-run", "--force-llm", "list all python files"],
    );
    assert!(!out.status.success());
    // An inherited model name is not an explicit request: ignored, truthfully.
    let v = json(&h, &[("CARO_MODEL", "some-model")], "list all python files");
    let dbg = v["debug_info"].as_str().unwrap();
    assert!(dbg.contains("Model: none"), "{dbg}");
    assert!(!dbg.contains("some-model"), "{dbg}");
}

#[test]
fn smart_approval_needs_no_network_or_model() {
    let h = TempDir::new().unwrap();
    // A dead proxy would fail any network attempt; the run must still succeed.
    let v = json(
        &h,
        &[
            ("CARO_APPROVAL", "smart"),
            ("HTTPS_PROXY", "http://127.0.0.1:9"),
            ("HTTP_PROXY", "http://127.0.0.1:9"),
        ],
        "list all python files",
    );
    assert_eq!(v["requires_confirmation"], false);
    assert!(v["warnings"].as_array().unwrap().is_empty());
    assert!(v["explanation"].as_str().unwrap().contains("no model"));
}

#[test]
fn benign_literals_that_look_dangerous_are_one_shell_word() {
    let h = TempDir::new().unwrap();
    for (req, lit) in [
        ("find files named used.txt", "used.txt"),
        (
            "find files named \"awk '{print $1}' | xargs echo.txt\"",
            "'awk '\\''{print $1}'\\'' | xargs echo.txt'",
        ),
        (
            "find files named \"delete me.txt\" in \"old rm backups\"",
            "'delete me.txt'",
        ),
    ] {
        let v = json(&h, &[], req);
        let cmd = command(&v);
        assert!(cmd.contains(lit), "{req}: {cmd}");
        assert_eq!(v["blocked_reason"], serde_json::Value::Null, "{req}");
        assert_eq!(v["requires_confirmation"], false, "{req}: {cmd}");
        // exactly one find and one -print0: a second generation or a
        // refinement would show up as a different or longer command
        assert_eq!(cmd.matches("-print0").count(), 1, "{cmd}");
        assert!(
            cmd.starts_with("/usr/bin/find") || cmd.starts_with("/bin/find"),
            "{cmd}"
        );
    }
}

#[test]
fn backend_roster_lists_select_files() {
    let h = TempDir::new().unwrap();
    let out = caro(&h, &[], &["--backend-info"]);
    let s = String::from_utf8_lossy(&out.stdout);
    let line = s
        .lines()
        .find(|l| l.contains("select-files"))
        .expect("roster row");
    assert!(line.contains("available"), "{line}");
}

#[test]
fn unknown_backend_names_are_still_rejected() {
    let h = TempDir::new().unwrap();
    let out = caro(
        &h,
        &[],
        &[
            "--backend",
            "select-file",
            "--dry-run",
            "list all python files",
        ],
    );
    assert!(!out.status.success());
}
