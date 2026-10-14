//! Process-level contract for `--backend select-files` (deterministic, no model).
//!
//! Runs the real binary with a private HOME. Nothing here starts a model,
//! opens a network connection or executes a generated command: every run is
//! `--dry-run`.
#![cfg(target_os = "linux")]

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

// ------------------------------------------------ eval route and flags ----

fn tiny_suite(dir: &TempDir) -> std::path::PathBuf {
    let p = dir.path().join("suite.yaml");
    std::fs::write(
        &p,
        "metadata:\n  name: tiny\n  description: select-files conflict check\ntest_cases:\n  - input: please\n    expected_outputs: [\"x\"]\n    category: edge_case\n",
    )
    .unwrap();
    p
}

#[test]
fn eval_route_rejects_every_model_request_before_initialization() {
    let h = TempDir::new().unwrap();
    let suite = tiny_suite(&h);
    let s = suite.to_str().unwrap();
    // `Cli` uses args_conflicts_with_subcommands: a top-level flag before
    // `test` sends the words down the normal route, not the subcommand. Those
    // forms are therefore pinned together with `--backend select-files`, where
    // the main route rejects them before any backend is built. A form without
    // `--backend` would start the default (embedded) backend and is NOT run.
    let cases: Vec<Vec<&str>> = vec![
        vec![
            "test",
            "--backend",
            "select-files",
            "--model-name",
            "m",
            "--suite",
            s,
        ],
        vec![
            "--backend",
            "select-files",
            "--model-name",
            "m",
            "test",
            "--suite",
            s,
        ],
        vec![
            "--backend",
            "select-files",
            "--force-llm",
            "test",
            "--suite",
            s,
        ],
        vec![
            "--backend",
            "select-files",
            "--advisor",
            "claude",
            "test",
            "--suite",
            s,
        ],
    ];
    for args in cases {
        let out = caro(&h, &[], &args);
        let all = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!out.status.success(), "{args:?}: {all}");
        assert!(
            all.contains("cannot be used with --backend select-files"),
            "{args:?}: {all}"
        );
        // refused before the suite was loaded or any generator was built
        assert!(!all.contains("Loaded test suite"), "{args:?}: {all}");
        assert!(!all.contains("Download"), "{args:?}: {all}");
    }
}

#[test]
fn eval_route_ignores_inherited_model_settings() {
    let h = TempDir::new().unwrap();
    let suite = tiny_suite(&h);
    // Config file model name and env model name are inherited, not requests.
    std::fs::create_dir_all(h.path().join("caro")).ok();
    let out = caro(
        &h,
        &[("CARO_MODEL", "some-model")],
        &[
            "test",
            "--backend",
            "select-files",
            "--suite",
            suite.to_str().unwrap(),
        ],
    );
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        all.contains("Running evaluation tests with backend: select-files"),
        "{all}"
    );
    assert!(all.contains("Loaded test suite: tiny"), "{all}");
    assert!(!all.contains("cannot be used"), "{all}");
}

// ------------------------------------------- real CLI output bytes ----

fn fixture_with_non_utf8() -> TempDir {
    use std::os::unix::ffi::OsStrExt;
    let d = TempDir::new().unwrap();
    std::fs::write(d.path().join("a.py"), b"").unwrap();
    std::fs::write(d.path().join("sp ace.py"), b"").unwrap();
    let odd = std::ffi::OsStr::from_bytes(b"\xff\xfe.py");
    std::fs::write(d.path().join(odd), b"").unwrap();
    d
}

fn caro_in(home: &TempDir, cwd: &std::path::Path, args: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_caro"));
    c.args(args)
        .current_dir(cwd)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .env("XDG_CACHE_HOME", home.path().join("cache"))
        .env("CARO_TELEMETRY_ENABLED", "false");
    c.output().expect("run caro")
}

#[test]
fn cli_stdout_is_exactly_the_nul_records_including_non_utf8_names() {
    let home = TempDir::new().unwrap();
    let fx = fixture_with_non_utf8();
    let out = caro_in(
        &home,
        fx.path(),
        &["--backend", "select-files", "-x", "list all python files"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // stdout holds nothing but the records
    assert_eq!(*out.stdout.last().unwrap(), 0, "must end with NUL");
    let mut recs: Vec<Vec<u8>> = out.stdout[..out.stdout.len() - 1]
        .split(|b| *b == 0)
        .map(<[u8]>::to_vec)
        .collect();
    recs.sort();
    let mut want: Vec<Vec<u8>> = vec![
        b"./a.py".to_vec(),
        b"./sp ace.py".to_vec(),
        b"./\xff\xfe.py".to_vec(),
    ];
    want.sort();
    assert_eq!(recs, want);
    // and equals what the same command prints when run directly
    let direct = Command::new("/usr/bin/find")
        .args(["-P", ".", "-type", "f", "-iname", "*.py", "-print0"])
        .current_dir(fx.path())
        .output()
        .unwrap();
    let mut d: Vec<Vec<u8>> = direct.stdout[..direct.stdout.len() - 1]
        .split(|b| *b == 0)
        .map(<[u8]>::to_vec)
        .collect();
    d.sort();
    assert_eq!(recs, d);
    // human text went to stderr, not into the records
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("Command:"), "{err}");
    assert!(err.contains("NUL-separated"), "{err}");
}

#[test]
fn cli_json_keeps_semantics_and_marks_lossy_text() {
    let home = TempDir::new().unwrap();
    let fx = fixture_with_non_utf8();
    let out = caro_in(
        &home,
        fx.path(),
        &[
            "--backend",
            "select-files",
            "-x",
            "-o",
            "json",
            "list all python files",
        ],
    );
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let v: serde_json::Value = serde_json::from_str(&text[text.find('{').unwrap()..]).unwrap();
    assert_eq!(v["exit_code"], 0);
    assert_eq!(v["stdout_encoding"], "lossy-utf8");
    let s = v["stdout"].as_str().unwrap();
    assert!(s.contains("./a.py\u{0}"), "{s:?}");
    assert!(s.contains('\u{fffd}'), "{s:?}");
}

#[test]
fn cli_failure_keeps_stderr_exit_and_error() {
    let home = TempDir::new().unwrap();
    let fx = TempDir::new().unwrap();
    let out = caro_in(
        &home,
        fx.path(),
        &[
            "--backend",
            "select-files",
            "-x",
            "-o",
            "json",
            "list python files in nope",
        ],
    );
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let v: serde_json::Value = serde_json::from_str(&text[text.find('{').unwrap()..]).unwrap();
    assert_eq!(v["exit_code"], 1);
    assert!(
        v["stderr"].as_str().unwrap().contains("No such file"),
        "{v}"
    );
    assert!(v["execution_error"]
        .as_str()
        .unwrap()
        .contains("exited with code 1"));
    // plain mode: nothing on stdout, find's message on stderr
    let out = caro_in(
        &home,
        fx.path(),
        &[
            "--backend",
            "select-files",
            "-x",
            "list python files in nope",
        ],
    );
    assert!(
        out.stdout.is_empty(),
        "{:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("No such file"));
}

// ------------------------------------- env and config selected route ----

fn write_config(home: &TempDir, body: &str) {
    let dir = home.path().join("caro");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("config.toml"),
        format!("{body}\n[telemetry]\nenabled = false\nfirst_run = false\n"),
    )
    .unwrap();
}

fn json_no_flag(home: &TempDir, envs: &[(&str, &str)], extra: &[&str], req: &str) -> Output {
    let mut args = vec!["--dry-run", "-o", "json", "-v"];
    args.extend_from_slice(extra);
    args.push(req);
    caro(home, envs, &args)
}

#[test]
fn env_only_backend_selects_the_deterministic_route() {
    let h = TempDir::new().unwrap();
    let out = json_no_flag(
        &h,
        &[("CARO_BACKEND", "select-files")],
        &[],
        "list all pdf files",
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let t = String::from_utf8_lossy(&out.stdout).into_owned();
    let v: serde_json::Value = serde_json::from_str(&t[t.find('{').unwrap()..]).unwrap();
    assert!(command(&v).ends_with("-iname '*.pdf' -print0"));
    assert!(v["debug_info"]
        .as_str()
        .unwrap()
        .contains("intent-deterministic"));
}

#[test]
fn config_selected_backend_ignores_inherited_model_and_rejects_force_llm() {
    let h = TempDir::new().unwrap();
    write_config(
        &h,
        "default_model = \"select-files\"\nmodel_name = \"inherited-model\"",
    );
    let out = json_no_flag(&h, &[], &[], "list all python files");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let t = String::from_utf8_lossy(&out.stdout).into_owned();
    let v: serde_json::Value = serde_json::from_str(&t[t.find('{').unwrap()..]).unwrap();
    let dbg = v["debug_info"].as_str().unwrap();
    assert!(
        dbg.contains("Model: none") && !dbg.contains("inherited-model"),
        "{dbg}"
    );
    // The effective backend comes from the config file, so the conflicts apply.
    let out = json_no_flag(&h, &[], &["--force-llm"], "list all python files");
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("cannot be used with --backend select-files")
    );
    let out = json_no_flag(&h, &[], &["--model-name", "m"], "list all python files");
    assert!(!out.status.success());
}

#[test]
fn flag_beats_env_beats_config_for_the_selected_route() {
    let h = TempDir::new().unwrap();
    // config and env name other backends; those are never constructed because
    // a higher-priority source selects select-files.
    write_config(&h, "default_model = \"embedded\"");
    let out = json_no_flag(
        &h,
        &[("CARO_BACKEND", "select-files")],
        &[],
        "list all python files",
    );
    assert!(
        out.status.success(),
        "env over config: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = caro(
        &h,
        &[("CARO_BACKEND", "embedded")],
        &[
            "--backend",
            "select-files",
            "--dry-run",
            "-o",
            "json",
            "list all python files",
        ],
    );
    assert!(
        out.status.success(),
        "flag over env: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
