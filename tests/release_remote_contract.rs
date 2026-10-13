//! Offline contract for the opt-in `release-remote` workflow (#1081).
//!
//! The optional `-remote` build lives in `.github/workflows/release-remote.yml`,
//! a separate manual workflow, so it cannot hold back the stable release
//! (`release.yml` -> Homebrew and packages). These tests pin that isolation
//! and run the workflow's actual `run:` blocks against fixtures:
//!
//! - fixture shell scripts stand in for the built binary;
//! - a stub `gh` on PATH stands in for GitHub and records every call;
//! - a throwaway git repo stands in for the tagged source.
//!
//! No network, no provider call, no cargo build, no upload. They run on Linux
//! and macOS CI. They do not run on Windows, so the Windows `certUtil`/`.exe`
//! path and CRLF handling stay unrun here. They also do not prove that a real
//! `--features remote-backends` build passes the smoke step on any platform:
//! `tests/backend_features_contract.rs` covers the debug builds on Linux and
//! macOS, and the first real `workflow_dispatch` run is the evidence for the rest.
#![cfg(unix)]

use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

const WORKFLOW: &str = ".github/workflows/release-remote.yml";
const WORKFLOW_NAME: &str = "Release Remote (optional)";
const SCRIPT: &str = "scripts/release-remote.sh";
const VERSION: &str = "1.5.0";
const TAG: &str = "v1.5.0";
const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const BACKENDS: [&str; 9] = [
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

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn yaml(path: &Path) -> Value {
    serde_yaml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn workflow() -> Value {
    yaml(&root().join(WORKFLOW))
}

fn get<'a>(v: &'a Value, key: &str) -> &'a Value {
    v.get(key)
        .unwrap_or_else(|| panic!("missing key '{key}' in {v:?}"))
}

fn job<'a>(wf: &'a Value, name: &str) -> &'a Value {
    get(get(wf, "jobs"), name)
}

fn steps(job: &Value) -> &Vec<Value> {
    get(job, "steps").as_sequence().unwrap()
}

fn step<'a>(job: &'a Value, id: &str) -> &'a Value {
    steps(job)
        .iter()
        .find(|s| s.get("id").and_then(Value::as_str) == Some(id))
        .unwrap_or_else(|| panic!("no step with id '{id}'"))
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn scalar(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => panic!("not a scalar: {other:?}"),
    }
}

/// One matrix entry of the build job: (target, platform, bin).
fn matrix() -> Vec<(String, String, String)> {
    let wf = workflow();
    get(get(get(job(&wf, "build"), "strategy"), "matrix"), "include")
        .as_sequence()
        .unwrap()
        .iter()
        .map(|e| {
            (
                scalar(get(e, "target")),
                scalar(get(e, "platform")),
                scalar(get(e, "bin")),
            )
        })
        .collect()
}

/// A scratch workspace holding the real script, optional fixtures and a stub `gh`.
struct Ws {
    dir: TempDir,
}

impl Ws {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("scripts")).unwrap();
        fs::copy(root().join(SCRIPT), dir.path().join(SCRIPT)).unwrap(); // keeps the exec bit
        fs::create_dir_all(dir.path().join("bin")).unwrap();
        let gh = dir.path().join("bin/gh");
        fs::write(
            &gh,
            "#!/bin/sh\n\
             echo \"gh $*\" >> \"$FAKE_LOG\"\n\
             case \"$1\" in\n\
               api) cat \"$FAKE_ASSETS\" ;;\n\
               release)\n\
                 [ \"$2\" = upload ] || { echo \"unexpected gh call: $*\" >&2; exit 99; }\n\
                 shift 3\n\
                 for f in \"$@\"; do basename \"$f\" >> \"$FAKE_ASSETS\"; done ;;\n\
               *) echo \"unexpected gh call: $*\" >&2; exit 99 ;;\n\
             esac\n",
        )
        .unwrap();
        make_executable(&gh);
        fs::write(dir.path().join("assets.txt"), "").unwrap();
        fs::write(dir.path().join("gh.log"), "").unwrap();
        Ws { dir }
    }

    fn p(&self) -> &Path {
        self.dir.path()
    }

    fn write_fixture(&self, rel: &str, body: &str) {
        let path = self.p().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, body).unwrap();
        make_executable(&path);
    }

    fn set_release_assets(&self, names: &[String]) {
        fs::write(self.p().join("assets.txt"), names.join("\n") + "\n").unwrap();
    }

    fn gh_log(&self) -> String {
        fs::read_to_string(self.p().join("gh.log")).unwrap()
    }

    fn release_assets(&self) -> Vec<String> {
        fs::read_to_string(self.p().join("assets.txt"))
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn github_output(&self) -> String {
        fs::read_to_string(self.p().join("github_output")).unwrap_or_default()
    }

    /// Run the real `run:` block of `job`/`step_id` under `bash -eo pipefail`, the
    /// shell GitHub uses for `shell: bash`. `${{ ... }}` values in the step's
    /// `env:` come from `exprs`; the block itself must contain none.
    fn run(&self, job_name: &str, step_id: &str, exprs: &[(&str, &str)]) -> Output {
        let wf = workflow();
        let st = step(job(&wf, job_name), step_id);
        assert_eq!(
            st.get("shell").and_then(Value::as_str),
            Some("bash"),
            "{step_id}: steps that run the script must use `shell: bash` (pipefail)"
        );
        let run = get(st, "run").as_str().unwrap();
        assert!(
            !run.contains("${{"),
            "{step_id}: run block holds an expression"
        );
        let given: HashMap<&str, &str> = exprs.iter().copied().collect();
        let mut env: Vec<(String, String)> = Vec::new();
        if let Some(map) = st.get("env").and_then(Value::as_mapping) {
            for (k, v) in map {
                let k = scalar(k);
                let raw = scalar(v);
                if raw.contains("${{") {
                    let val = given
                        .get(k.as_str())
                        .unwrap_or_else(|| panic!("{step_id}: no fixture for env {k} = {raw}"));
                    env.push((k, (*val).to_string()));
                } else {
                    env.push((k, raw));
                }
            }
        }
        let path = format!(
            "{}:{}",
            self.p().join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        Command::new("bash")
            .args(["--noprofile", "--norc", "-eo", "pipefail", "-c", run])
            .current_dir(self.p())
            .env_clear()
            .env("PATH", path)
            .env("HOME", self.p())
            .env("GITHUB_OUTPUT", self.p().join("github_output"))
            .env("FAKE_LOG", self.p().join("gh.log"))
            .env("FAKE_ASSETS", self.p().join("assets.txt"))
            .envs(env)
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perm = fs::metadata(path).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(path, perm).unwrap();
}

fn backend_info_fixture(compiled: bool) -> String {
    let mut s = String::from("#!/bin/sh\necho 'Available inference backends'\n");
    s.push_str("echo '  embedded      available         local'\n");
    for b in BACKENDS {
        let status = if compiled {
            "available"
        } else {
            "not compiled"
        };
        s.push_str(&format!("echo '  {b}  {status}  notes'\n"));
    }
    s
}

fn build_exprs(target: &str, platform: &str, bin: &str) -> Vec<(String, String)> {
    vec![
        ("VERSION".into(), VERSION.into()),
        ("TAG".into(), TAG.into()),
        ("COMMIT".into(), COMMIT.into()),
        ("PLATFORM".into(), platform.into()),
        ("TARGET".into(), target.into()),
        ("BIN".into(), bin.into()),
    ]
}

fn refs(v: &[(String, String)]) -> Vec<(&str, &str)> {
    v.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
}

fn remote_name(platform: &str) -> String {
    if platform.starts_with("windows") {
        format!("caro-{VERSION}-{platform}-remote.exe")
    } else {
        format!("caro-{VERSION}-{platform}-remote")
    }
}

/// Place a fixture binary where the build step would leave it.
fn place_binary(ws: &Ws, target: &str, bin: &str, body: &str) {
    ws.write_fixture(&format!("source/target/{target}/release/{bin}"), body);
}

fn arm64_remote_fixture() -> &'static str {
    "#!/bin/sh\n# ANTHROPIC_API_KEY environment variable not set\n"
}

fn prepare_ok(ws: &Ws, target: &str, platform: &str, bin: &str) {
    let body = if target == "aarch64-unknown-linux-gnu" {
        arm64_remote_fixture().to_string()
    } else {
        backend_info_fixture(true)
    };
    place_binary(ws, target, bin, &body);
    let e = build_exprs(target, platform, bin);
    let out = ws.run("build", "prepare", &refs(&e));
    assert!(out.status.success(), "prepare failed:\n{}", text(&out));
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
        .args(["-c", "user.email=t@example.com", "-c", "user.name=t"])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {}", text(&out));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A git repo whose Cargo.toml says `version`, tagged `tag`. Returns the commit.
fn make_repo(dir: &Path, version: &str, tag: &str) -> String {
    fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q"]);
    fs::write(
        dir.join("Cargo.toml"),
        format!("[package]\nname = \"caro\"\nversion = \"{version}\"\n\n[features]\n"),
    )
    .unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "release"]);
    git(dir, &["tag", tag]);
    git(dir, &["rev-parse", "HEAD"])
}

// ---------------------------------------------------------------------------
// Isolation and permissions
// ---------------------------------------------------------------------------

#[test]
fn workflow_is_manual_only_and_publication_is_off_by_default() {
    let wf = workflow();
    let on = get(&wf, "on").as_mapping().unwrap();
    let triggers: Vec<String> = on.keys().map(scalar).collect();
    assert_eq!(triggers, ["workflow_dispatch"], "must be manual only");

    let publish = get(get(get(&wf, "on"), "workflow_dispatch"), "inputs");
    let publish = get(publish, "publish");
    assert_eq!(get(publish, "type").as_str(), Some("boolean"));
    assert_eq!(get(publish, "default").as_bool(), Some(false));

    // Least privilege: read-only workflow; only the publish job can write.
    let perms = get(&wf, "permissions").as_mapping().unwrap();
    assert_eq!(perms.len(), 1);
    assert_eq!(perms.get("contents").and_then(Value::as_str), Some("read"));
    for (name, j) in get(&wf, "jobs").as_mapping().unwrap() {
        let name = scalar(name);
        match (name.as_str(), j.get("permissions")) {
            ("publish", Some(p)) => {
                assert_eq!(p.get("contents").and_then(Value::as_str), Some("write"));
                assert_eq!(p.as_mapping().unwrap().len(), 1);
            }
            ("publish", None) => panic!("publish job must declare its own permissions"),
            (other, Some(_)) => panic!("job {other} must not widen permissions"),
            (_, None) => {}
        }
    }

    let publish_job = job(&wf, "publish");
    let cond = get(publish_job, "if").as_str().unwrap();
    assert!(
        cond.contains("inputs.publish == true"),
        "publish gate: {cond}"
    );
    let needs: Vec<String> = get(publish_job, "needs")
        .as_sequence()
        .unwrap()
        .iter()
        .map(scalar)
        .collect();
    assert!(
        needs.contains(&"build".to_string()),
        "publish must need build"
    );

    // The build job never writes a release.
    let raw = fs::read_to_string(root().join(WORKFLOW)).unwrap();
    assert!(!raw.contains("softprops/action-gh-release"));
    assert!(!raw.contains("--clobber"));
}

#[test]
fn stable_release_workflows_do_not_depend_on_the_optional_workflow() {
    let dir = root().join(".github/workflows");
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if name == "release-remote.yml" {
            continue;
        }
        let wf = yaml(&path);
        let waits_for_it = wf
            .get("on")
            .and_then(|on| on.get("workflow_run"))
            .and_then(|wr| wr.get("workflows"))
            .and_then(Value::as_sequence)
            .map(|l| l.iter().any(|w| w.as_str() == Some(WORKFLOW_NAME)))
            .unwrap_or(false);
        assert!(!waits_for_it, "{name} must not wait for {WORKFLOW_NAME}");
    }
    // The stable Release workflow carries no optional work at all.
    let release = fs::read_to_string(dir.join("release.yml")).unwrap();
    for needle in [
        "remote-backends",
        "-remote",
        "release-remote",
        "continue-on-error",
    ] {
        assert!(
            !release.contains(needle),
            "release.yml must not contain '{needle}'"
        );
    }
}

#[test]
fn run_blocks_hold_no_expressions_and_never_call_gh_directly() {
    let wf = workflow();
    for (jname, j) in get(&wf, "jobs").as_mapping().unwrap() {
        for st in steps(j) {
            if let Some(run) = st.get("run").and_then(Value::as_str) {
                assert!(
                    !run.contains("${{"),
                    "{}: a run block holds an expression (pass it through env:)",
                    scalar(jname)
                );
                assert!(
                    !run.contains("gh "),
                    "{}: gh calls belong in {SCRIPT}",
                    scalar(jname)
                );
            }
        }
    }
}

#[test]
fn the_build_job_builds_with_remote_backends_for_all_five_platforms() {
    let m = matrix();
    let mut platforms: Vec<&str> = m.iter().map(|(_, p, _)| p.as_str()).collect();
    platforms.sort();
    assert_eq!(
        platforms,
        [
            "linux-amd64",
            "linux-arm64",
            "macos-intel",
            "macos-silicon",
            "windows-amd64"
        ]
    );
    let wf = workflow();
    let build = get(job(&wf, "build"), "steps")
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|s| s.get("run").and_then(Value::as_str))
        .find(|r| r.contains("cargo build"))
        .expect("a cargo build step");
    assert!(build.contains("--features remote-backends"));
    assert!(build.contains("cross build --release --features remote-backends"));
}

// ---------------------------------------------------------------------------
// Build job blocks: prepare, smoke, verify
// ---------------------------------------------------------------------------

#[test]
fn prepare_smoke_and_verify_blocks_accept_a_remote_build_on_every_platform() {
    for (target, platform, bin) in matrix() {
        let ws = Ws::new();
        prepare_ok(&ws, &target, &platform, &bin);
        let e = build_exprs(&target, &platform, &bin);

        let name = remote_name(&platform);
        let dist = ws.p().join("dist");
        assert!(dist.join(&name).is_file(), "{name} missing");
        let sums = fs::read_to_string(dist.join(format!("{name}.sha256"))).unwrap();
        assert_eq!(sums.lines().count(), 1);
        assert!(sums.trim_end().ends_with(&format!("  {name}")), "{sums}");
        let manifest =
            fs::read_to_string(dist.join(format!("{}.manifest", name.trim_end_matches(".exe"))))
                .unwrap();
        for line in [
            format!("tag={TAG}"),
            format!("version={VERSION}"),
            format!("commit={COMMIT}"),
            format!("platform={platform}"),
            format!("asset={name}"),
        ] {
            assert!(
                manifest.lines().any(|l| l == line),
                "{line} not in manifest:\n{manifest}"
            );
        }
        // Only the -remote name exists: no default-named asset is created.
        let default_name = name.replace("-remote", "");
        assert!(!dist.join(default_name).exists());

        let smoke = ws.run("build", "smoke", &refs(&e));
        assert!(
            smoke.status.success(),
            "{platform} smoke:\n{}",
            text(&smoke)
        );
        let verify = ws.run("build", "verify", &refs(&e));
        assert!(
            verify.status.success(),
            "{platform} verify:\n{}",
            text(&verify)
        );
    }
}

#[test]
fn smoke_block_rejects_a_default_build_and_unusable_output() {
    let (target, platform, bin) = ("x86_64-unknown-linux-gnu", "linux-amd64", "caro");
    let e = build_exprs(target, platform, bin);
    let cases: [(&str, String, &str); 4] = [
        ("default build", backend_info_fixture(false), "not compiled"),
        ("empty output", "#!/bin/sh\n".to_string(), "no ollama row"),
        (
            "non-zero exit",
            "#!/bin/sh\necho partial output\nexit 3\n".to_string(),
            "exited non-zero",
        ),
        (
            "missing backend row",
            backend_info_fixture(true).replace("openrouter", "other"),
            "no openrouter row",
        ),
    ];
    for (label, body, expect) in cases {
        let ws = Ws::new();
        place_binary(&ws, target, bin, &body);
        let p = ws.run("build", "prepare", &refs(&e));
        assert!(p.status.success(), "{label} prepare: {}", text(&p));
        let out = ws.run("build", "smoke", &refs(&e));
        assert!(
            !out.status.success(),
            "{label} must fail smoke:\n{}",
            text(&out)
        );
        assert!(
            text(&out).contains(expect),
            "{label}: want '{expect}' in:\n{}",
            text(&out)
        );
    }
}

#[test]
fn smoke_block_static_check_for_the_cross_built_arm64_binary() {
    let (target, platform, bin) = ("aarch64-unknown-linux-gnu", "linux-arm64", "caro");
    let e = build_exprs(target, platform, bin);
    let cases = [
        ("default marker present", "which is not compiled into this build\nANTHROPIC_API_KEY environment variable not set\n"),
        ("remote marker absent", "nothing relevant\n"),
    ];
    for (label, body) in cases {
        let ws = Ws::new();
        place_binary(&ws, target, bin, &format!("#!/bin/sh\n# {body}"));
        let p = ws.run("build", "prepare", &refs(&e));
        assert!(p.status.success(), "{label} prepare: {}", text(&p));
        let out = ws.run("build", "smoke", &refs(&e));
        assert!(!out.status.success(), "{label} must fail:\n{}", text(&out));
    }
}

#[test]
fn verify_block_detects_partial_pairs_checksum_mismatch_and_stray_names() {
    let (target, platform, bin) = ("x86_64-unknown-linux-gnu", "linux-amd64", "caro");
    let e = build_exprs(target, platform, bin);
    let name = remote_name(platform);

    type Mutate = fn(&Path, &str);
    let cases: [(&str, Mutate, &str); 6] = [
        (
            "tampered binary",
            |d, n| fs::write(d.join(n), "#!/bin/sh\necho tampered\n").unwrap(),
            "checksum mismatch",
        ),
        (
            "binary without .sha256",
            |d, n| fs::remove_file(d.join(format!("{n}.sha256"))).unwrap(),
            "partial pair",
        ),
        (
            ".sha256 without binary",
            |d, n| fs::remove_file(d.join(n)).unwrap(),
            "partial pair",
        ),
        (
            ".sha256 names another file",
            |d, n| {
                let s = fs::read_to_string(d.join(format!("{n}.sha256"))).unwrap();
                fs::write(d.join(format!("{n}.sha256")), s.replace(n, "caro-other")).unwrap();
            },
            "is not",
        ),
        (
            "default-named asset next to it",
            |d, n| fs::write(d.join(n.replace("-remote", "")), "x").unwrap(),
            "unexpected file",
        ),
        (
            "asset of another version",
            |d, _| fs::write(d.join("caro-9.9.9-linux-amd64-remote"), "x").unwrap(),
            "unexpected file",
        ),
    ];
    for (label, mutate, expect) in cases {
        let ws = Ws::new();
        prepare_ok(&ws, target, platform, bin);
        mutate(&ws.p().join("dist"), &name);
        let out = ws.run("build", "verify", &refs(&e));
        assert!(
            !out.status.success(),
            "{label} must fail verify:\n{}",
            text(&out)
        );
        assert!(
            text(&out).contains(expect),
            "{label}: want '{expect}' in:\n{}",
            text(&out)
        );
    }
}

// ---------------------------------------------------------------------------
// Tag and commit identity
// ---------------------------------------------------------------------------

#[test]
fn resolve_block_pins_the_tag_to_one_commit_and_rejects_a_version_mismatch() {
    let ws = Ws::new();
    let commit = make_repo(ws.p(), "1.5.0", TAG);
    let ok = ws.run("resolve", "resolve", &[("TAG", TAG)]);
    assert!(ok.status.success(), "{}", text(&ok));
    let out = ws.github_output();
    for line in [
        format!("tag={TAG}"),
        format!("version={VERSION}"),
        format!("commit={commit}"),
    ] {
        assert!(out.lines().any(|l| l == line), "{line} not in:\n{out}");
    }

    let missing = ws.run("resolve", "resolve", &[("TAG", "v1.5.9")]);
    assert!(!missing.status.success());
    assert!(text(&missing).contains("does not exist"));

    git(ws.p(), &["tag", "v1.5.1"]); // Cargo.toml still says 1.5.0
    let wrong = ws.run("resolve", "resolve", &[("TAG", "v1.5.1")]);
    assert!(!wrong.status.success());
    assert!(text(&wrong).contains("Cargo.toml"), "{}", text(&wrong));

    let bad = ws.run("resolve", "resolve", &[("TAG", "1.5.0")]);
    assert!(!bad.status.success());
}

#[test]
fn check_source_and_recheck_blocks_catch_a_wrong_checkout_and_a_moved_tag() {
    let ws = Ws::new();
    let commit = make_repo(&ws.p().join("source"), "1.5.0", TAG);
    let exprs = [("COMMIT", commit.as_str()), ("VERSION", VERSION)];
    let ok = ws.run("build", "check_source", &exprs);
    assert!(ok.status.success(), "{}", text(&ok));

    let wrong_commit = ws.run(
        "build",
        "check_source",
        &[("COMMIT", COMMIT), ("VERSION", VERSION)],
    );
    assert!(!wrong_commit.status.success());
    let wrong_version = ws.run(
        "build",
        "check_source",
        &[("COMMIT", commit.as_str()), ("VERSION", "1.5.1")],
    );
    assert!(!wrong_version.status.success());

    // publish job: the tag must still point at the built commit.
    let ws2 = Ws::new();
    let first = make_repo(ws2.p(), "1.5.0", TAG);
    let exprs = [("TAG", TAG), ("COMMIT", first.as_str())];
    assert!(ws2.run("publish", "recheck", &exprs).status.success());
    fs::write(
        ws2.p().join("Cargo.toml"),
        "[package]\nname = \"caro\"\nversion = \"1.5.0\"\n# moved\n",
    )
    .unwrap();
    git(ws2.p(), &["commit", "-q", "-am", "moved"]);
    git(ws2.p(), &["tag", "-f", TAG]);
    let moved = ws2.run("publish", "recheck", &exprs);
    assert!(!moved.status.success());
    assert!(text(&moved).contains("moved"), "{}", text(&moved));
}

// ---------------------------------------------------------------------------
// Publish job blocks, against a stub gh
// ---------------------------------------------------------------------------

fn all_platforms() -> Vec<(String, String, String)> {
    matrix()
}

/// Build verified dist/ for all five platforms the way five build jobs would.
fn full_dist(ws: &Ws) {
    for (target, platform, bin) in all_platforms() {
        prepare_ok(ws, &target, &platform, &bin);
    }
}

fn publish_exprs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("VERSION", VERSION),
        ("TAG", TAG),
        ("COMMIT", COMMIT),
        ("GH_TOKEN", "fake-token"),
        ("GH_REPO", "o/r"),
        ("GITHUB_REPOSITORY", "o/r"),
    ]
}

fn names_for(platforms: &[&str], sums: bool) -> Vec<String> {
    let mut v = Vec::new();
    for p in platforms {
        v.push(remote_name(p));
        if sums {
            v.push(format!("{}.sha256", remote_name(p)));
        }
    }
    v
}

#[test]
fn publish_blocks_verify_identity_and_check_the_release_before_uploading() {
    let ws = Ws::new();
    full_dist(&ws);
    let e = publish_exprs();

    let ok = ws.run("publish", "verify_all", &e);
    assert!(ok.status.success(), "{}", text(&ok));
    assert!(text(&ok).contains("verified 5 pair(s)"));
    assert!(text(&ok).contains("identity ok for 5 platform(s)"));

    // A different commit than the one the manifests recorded is refused.
    let mut other = e.clone();
    other.retain(|(k, _)| *k != "COMMIT");
    other.push(("COMMIT", "ffffffffffffffffffffffffffffffffffffffff"));
    let bad = ws.run("publish", "verify_all", &other);
    assert!(!bad.status.success());
    assert!(text(&bad).contains("lacks 'commit="), "{}", text(&bad));

    // One missing platform is refused: publication is all-or-nothing.
    fs::remove_file(ws.p().join("dist").join(remote_name("macos-intel"))).unwrap();
    let missing = ws.run("publish", "verify_all", &e);
    assert!(!missing.status.success());
    assert!(
        text(&missing).contains("partial pair"),
        "{}",
        text(&missing)
    );
}

#[test]
fn before_block_refuses_a_release_that_already_has_remote_assets() {
    let ws = Ws::new();
    let e = publish_exprs();

    ws.set_release_assets(&[
        "caro-1.5.0-linux-amd64".to_string(),
        "caro-1.5.0-linux-amd64.sha256".to_string(),
    ]);
    let clean = ws.run("publish", "before", &e);
    assert!(
        clean.status.success(),
        "default assets only must pass:\n{}",
        text(&clean)
    );

    ws.set_release_assets(&names_for(&["linux-amd64"], false));
    let partial = ws.run("publish", "before", &e);
    assert!(!partial.status.success());
    assert!(
        text(&partial).contains("partial binary/checksum pair"),
        "{}",
        text(&partial)
    );

    ws.set_release_assets(&names_for(&["linux-amd64"], true));
    let done = ws.run("publish", "before", &e);
    assert!(!done.status.success());
    assert!(text(&done).contains("not all 'absent'"), "{}", text(&done));
}

#[test]
fn upload_block_never_overwrites_creates_or_deletes_and_after_block_confirms_pairs() {
    let ws = Ws::new();
    full_dist(&ws);
    let e = publish_exprs();

    let up = ws.run("publish", "upload", &e);
    assert!(up.status.success(), "{}", text(&up));
    let log = ws.gh_log();
    assert_eq!(
        log.lines().count(),
        5,
        "one upload call per platform:\n{log}"
    );
    for l in log.lines() {
        assert!(
            l.starts_with(&format!("gh release upload {TAG} dist/caro-{VERSION}-")),
            "{l}"
        );
        assert!(!l.contains("--clobber"), "{l}");
    }
    assert!(
        !log.contains("release create")
            && !log.contains("release delete")
            && !log.contains("release edit")
    );

    // The stub recorded the uploads; every pair is now on the release.
    assert_eq!(ws.release_assets().len(), 10);
    let after = ws.run("publish", "after", &e);
    assert!(after.status.success(), "{}", text(&after));

    // An interrupted upload (binary without its checksum) is reported as partial.
    let mut names = ws.release_assets();
    names.retain(|n| n != &format!("{}.sha256", remote_name("windows-amd64")));
    ws.set_release_assets(&names);
    let cut = ws.run("publish", "after", &e);
    assert!(!cut.status.success());
    assert!(
        text(&cut).contains("partial binary/checksum pair"),
        "{}",
        text(&cut)
    );
}
