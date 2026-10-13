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
        // The real CVE rule files: the script derives what a build must contain
        // from them.
        let rules = dir.path().join("data/cve_rules");
        fs::create_dir_all(&rules).unwrap();
        for entry in fs::read_dir(root().join("data/cve_rules")).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), rules.join(entry.file_name())).unwrap();
        }
        // Tripwire: the dangerous CVE-2024-3094 fixture command must only ever
        // be validator input, never run. Any call to `xz` is recorded.
        let xz = dir.path().join("bin/xz");
        fs::write(&xz, "#!/bin/sh\necho \"xz $*\" >> \"$FAKE_DIR/xz.log\"\n").unwrap();
        make_executable(&xz);
        let gh = dir.path().join("bin/gh");
        fs::write(
            &gh,
            "#!/bin/sh\n\
             n=$(( $(cat \"$FAKE_DIR/count\" 2>/dev/null || echo 0) + 1 )); echo $n > \"$FAKE_DIR/count\"\n\
             echo \"gh $*\" >> \"$FAKE_LOG\"\n\
             case \"$1\" in\n\
               api) cat \"$FAKE_ASSETS\" ;;\n\
               release)\n\
                 [ \"$2\" = upload ] || { echo \"unexpected gh call: $*\" >&2; exit 99; }\n\
                 shift 3\n\
                 if [ \"$(cat \"$FAKE_DIR/die_mid_pair_on_call\" 2>/dev/null)\" = \"$n\" ]; then\n\
                   basename \"$1\" >> \"$FAKE_ASSETS\"; echo \"stub: upload died after the first file\" >&2; exit 1; fi\n\
                 if [ \"$(cat \"$FAKE_DIR/fail_on_call\" 2>/dev/null)\" = \"$n\" ]; then\n\
                   echo \"stub: upload failed before any file\" >&2; exit 1; fi\n\
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

    /// Make the n-th `gh` call (1-based; it must be an upload) fail before it stores any file.
    fn fail_gh_on_call(&self, n: u32) {
        fs::write(self.p().join("fail_on_call"), n.to_string()).unwrap();
    }

    /// Make the n-th `gh release upload` die after storing only its first file.
    fn gh_dies_mid_pair_on_call(&self, n: u32) {
        fs::write(self.p().join("die_mid_pair_on_call"), n.to_string()).unwrap();
    }

    fn upload_calls(&self) -> Vec<String> {
        self.gh_log()
            .lines()
            .filter(|l| l.starts_with("gh release upload"))
            .map(str::to_string)
            .collect()
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
            .env("FAKE_DIR", self.p())
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

/// What the fixture's `guard` subcommand does with the CVE-2024-3094 trigger.
#[derive(Clone, Copy, PartialEq)]
enum Cve {
    /// Critical, deny, cites the CVE: a build with the ruleset compiled in.
    Ok,
    /// Safe, no opinion: a build whose ruleset compiled to zero rules.
    ZeroRules,
    /// Denies, but without citing the CVE rule.
    NoCveText,
    /// Denies everything, even a benign command.
    AlwaysDeny,
}

/// Source URLs of the compiled (non-draft) `CVE-*.yaml` rules in the repo.
fn rule_sources() -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(root().join("data/cve_rules"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with("CVE-") && n.ends_with(".yaml")
        })
        .filter_map(|e| {
            let body = fs::read_to_string(e.path()).unwrap();
            if body.contains("TODO_NIMBLE_AUTHORED") {
                return None;
            }
            body.lines()
                .find_map(|l| l.strip_prefix("source:"))
                .map(|u| u.split_whitespace().next().unwrap_or("").to_string())
        })
        .collect();
    v.sort();
    v
}

fn fixture(compiled: bool, cve: Cve, with_rule_urls: bool) -> String {
    let mut s = String::from("#!/bin/sh\n");
    if with_rule_urls {
        for u in rule_sources() {
            s.push_str(&format!("# compiled rule source: {u}\n"));
        }
    }
    let cites = match cve {
        Cve::NoCveText => "some-other-pattern",
        _ => "cve-2024-3094: xz-utils backdoor trigger",
    };
    let (trigger, trigger_rc) = match cve {
        Cve::ZeroRules => (
            r#"{"matched_patterns":[],"risk":"safe","verdict":"none"}"#.to_string(),
            0,
        ),
        _ => (
            format!(r#"{{"matched_patterns":["{cites}"],"risk":"critical","verdict":"deny"}}"#),
            2,
        ),
    };
    let (benign, benign_rc) = if cve == Cve::AlwaysDeny {
        (trigger.clone(), 2)
    } else {
        (
            r#"{"matched_patterns":[],"risk":"safe","verdict":"none"}"#.to_string(),
            0,
        )
    };
    s.push_str(&format!(
        "if [ \"$1\" = guard ]; then\n\
         \x20 payload=$(cat)\n\
         \x20 case \"$payload\" in\n\
         \x20   *'xz --lzma1'*) echo '{trigger}'; exit {trigger_rc} ;;\n\
         \x20   *) echo '{benign}'; exit {benign_rc} ;;\n\
         \x20 esac\n\
         fi\n"
    ));
    s.push_str("echo 'Available inference backends'\n");
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

fn backend_info_fixture(compiled: bool) -> String {
    fixture(compiled, Cve::Ok, true)
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
    ws.write_fixture(&format!("target/{target}/release/{bin}"), body);
}

fn arm64_remote_fixture() -> String {
    arm64_fixture(true)
}

fn arm64_fixture(with_rule_urls: bool) -> String {
    let mut s = String::from("#!/bin/sh\n# ANTHROPIC_API_KEY environment variable not set\n");
    if with_rule_urls {
        for u in rule_sources() {
            s.push_str(&format!("# compiled rule source: {u}\n"));
        }
    }
    s
}

fn prepare_ok(ws: &Ws, target: &str, platform: &str, bin: &str) {
    let body = if target == "aarch64-unknown-linux-gnu" {
        arm64_remote_fixture()
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
    git(dir, &["tag", "-a", "-m", "release", tag]); // real release tags are annotated
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
    let needs: Vec<String> = match get(publish_job, "needs") {
        Value::Sequence(l) => l.iter().map(scalar).collect(),
        other => vec![scalar(other)],
    };
    assert!(
        needs.contains(&"build".to_string()),
        "publish must need build"
    );

    // The build job never writes a release.
    let raw = fs::read_to_string(root().join(WORKFLOW)).unwrap();
    assert!(!raw.contains("softprops/action-gh-release"));
    assert!(!raw.contains("--clobber"));
    // The build job runs code from a tag: no cache may be saved from it.
    for (_, j) in get(&wf, "jobs").as_mapping().unwrap() {
        for st in steps(j) {
            let uses = st.get("uses").and_then(Value::as_str).unwrap_or("");
            assert!(
                uses.is_empty() || uses.starts_with("actions/"),
                "only first-party actions may run in this workflow: {uses}"
            );
            assert!(
                !uses.contains("cache"),
                "no cache may be saved from a job that builds tag source: {uses}"
            );
        }
    }
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

fn pin_exprs<'a>(
    ref_type: &'a str,
    ref_name: &'a str,
    sha: &'a str,
    input_tag: &'a str,
) -> Vec<(&'a str, &'a str)> {
    vec![
        ("REF_TYPE", ref_type),
        ("REF_NAME", ref_name),
        ("SHA", sha),
        ("INPUT_TAG", input_tag),
    ]
}

#[test]
fn pin_block_accepts_exactly_the_tagged_commit_in_both_jobs() {
    for job_name in ["build", "publish"] {
        let ws = Ws::new();
        let commit = make_repo(ws.p(), "1.5.0", TAG);
        // The tag object differs from the commit it peels to; pin must use the commit.
        assert_eq!(git(ws.p(), &["cat-file", "-t", TAG]), "tag");
        assert_ne!(git(ws.p(), &["rev-parse", TAG]), commit);
        let out = ws.run(job_name, "pin", &pin_exprs("tag", TAG, &commit, TAG));
        assert!(out.status.success(), "{job_name}: {}", text(&out));
        let got = ws.github_output();
        for line in [
            format!("tag={TAG}"),
            format!("version={VERSION}"),
            format!("commit={commit}"),
        ] {
            assert!(got.lines().any(|l| l == line), "{line} not in:\n{got}");
        }
    }
}

#[test]
fn pin_block_rejects_a_mismatched_ref_input_commit_version_or_moved_tag() {
    for job_name in ["build", "publish"] {
        let ws = Ws::new();
        let commit = make_repo(ws.p(), "1.5.0", TAG);
        git(ws.p(), &["tag", "-a", "-m", "x", "v1.5.1"]); // same commit, Cargo.toml still says 1.5.0

        type Exprs<'a> = Vec<(&'a str, &'a str)>;
        let cases: Vec<(&str, Exprs, &str)> = vec![
            (
                "started from a branch",
                pin_exprs("branch", "main", &commit, "main"),
                "start this workflow from the release tag",
            ),
            (
                "tag input names another tag",
                pin_exprs("tag", TAG, &commit, "v1.5.1"),
                "does not match the tag this run was started from",
            ),
            (
                "ref name is not a v-tag",
                pin_exprs("tag", "1.5.0", &commit, "1.5.0"),
                "must start with 'v'",
            ),
            (
                "dispatched commit is not a full SHA",
                pin_exprs("tag", TAG, "abc123", TAG),
                "not a full SHA",
            ),
            (
                "checkout is not the dispatched commit",
                pin_exprs("tag", TAG, COMMIT, TAG),
                "is not the dispatched commit",
            ),
            (
                "tag does not exist",
                pin_exprs("tag", "v1.5.9", &commit, "v1.5.9"),
                "does not exist",
            ),
            (
                "tag version differs from Cargo.toml",
                pin_exprs("tag", "v1.5.1", &commit, "v1.5.1"),
                "Cargo.toml",
            ),
        ];
        for (label, exprs, expect) in cases {
            let out = ws.run(job_name, "pin", &exprs);
            assert!(!out.status.success(), "{job_name}/{label} must fail");
            assert!(
                text(&out).contains(expect),
                "{job_name}/{label}: want '{expect}' in:\n{}",
                text(&out)
            );
            assert!(
                ws.github_output().is_empty(),
                "{job_name}/{label}: a failed pin must output nothing"
            );
        }

        // The tag moved after the run started: HEAD is still the dispatched
        // commit, but the tag now points elsewhere.
        git(ws.p(), &["checkout", "-q", "-b", "other"]);
        fs::write(
            ws.p().join("Cargo.toml"),
            "[package]\nname = \"caro\"\nversion = \"1.5.0\"\n# moved\n",
        )
        .unwrap();
        git(ws.p(), &["commit", "-q", "-am", "moved"]);
        git(ws.p(), &["tag", "-f", "-a", "-m", "moved", TAG]);
        git(ws.p(), &["checkout", "-q", "--detach", &commit]);
        let moved = ws.run(job_name, "pin", &pin_exprs("tag", TAG, &commit, TAG));
        assert!(!moved.status.success(), "{job_name}: moved tag must fail");
        assert!(text(&moved).contains("moved"), "{}", text(&moved));
    }
}

#[test]
fn workflow_checks_out_one_commit_and_never_an_input_ref() {
    let wf = workflow();
    let jobs: Vec<String> = get(&wf, "jobs")
        .as_mapping()
        .unwrap()
        .keys()
        .map(scalar)
        .collect();
    assert_eq!(
        jobs,
        ["build", "publish"],
        "no resolve job, no second checkout"
    );
    for (jname, j) in get(&wf, "jobs").as_mapping().unwrap() {
        let jname = scalar(jname);
        let st = steps(j);
        // The first step is the only checkout; the second pins the source
        // before any toolchain, build or download step.
        assert!(
            st[0]
                .get("uses")
                .and_then(Value::as_str)
                .unwrap()
                .starts_with("actions/checkout@"),
            "{jname}: step 1 must be the checkout"
        );
        assert_eq!(
            st[1].get("id").and_then(Value::as_str),
            Some("pin"),
            "{jname}: step 2 must pin the source"
        );
        let checkouts = st
            .iter()
            .filter(|s| {
                s.get("uses")
                    .and_then(Value::as_str)
                    .is_some_and(|u| u.starts_with("actions/checkout@"))
            })
            .count();
        assert_eq!(checkouts, 1, "{jname}: exactly one checkout");
        let with = get(&st[0], "with").as_mapping().unwrap();
        let keys: Vec<String> = with.keys().map(scalar).collect();
        assert_eq!(
            keys,
            ["persist-credentials"],
            "{jname}: checkout takes no ref/path/repository"
        );
        assert_eq!(
            with.get("persist-credentials").and_then(Value::as_bool),
            Some(false)
        );
    }
    // Inputs are used in two places only: the tag confirmation passed to `pin`
    // through env, and the publish gate. Any other mention of `inputs` fails,
    // whatever the syntax (`inputs['tag']`, `toJSON(inputs)`, ...).
    let raw = fs::read_to_string(root().join(WORKFLOW)).unwrap();
    let word = regex::Regex::new(r"\binputs\b").unwrap();
    for line in raw.lines().filter(|l| !l.trim_start().starts_with('#')) {
        if word.is_match(line) && line.trim() != "inputs:" {
            assert!(
                line.contains("INPUT_TAG: ${{ inputs.tag }}")
                    || line.contains("inputs.publish == true"),
                "unexpected use of an input: {line}"
            );
        }
    }
    // The pin step gets its facts from the dispatch context, in both jobs, and
    // no run block moves the checkout.
    let moves =
        regex::Regex::new(r"\bgit\s+(checkout|fetch|switch|worktree|clone|pull)\b").unwrap();
    for (jname, j) in get(&wf, "jobs").as_mapping().unwrap() {
        let jname = scalar(jname);
        let pin = get(step(j, "pin"), "env").as_mapping().unwrap();
        let got: Vec<(String, String)> = pin.iter().map(|(k, v)| (scalar(k), scalar(v))).collect();
        let want = [
            ("REF_TYPE", "${{ github.ref_type }}"),
            ("REF_NAME", "${{ github.ref_name }}"),
            ("SHA", "${{ github.sha }}"),
            ("INPUT_TAG", "${{ inputs.tag }}"),
        ];
        assert_eq!(got.len(), want.len(), "{jname}: pin env");
        for (k, v) in want {
            assert!(
                got.iter().any(|(gk, gv)| gk == k && gv == v),
                "{jname}: pin env {k} must be {v}"
            );
        }
        for st in steps(j) {
            if let Some(run) = st.get("run").and_then(Value::as_str) {
                assert!(
                    !moves.is_match(run),
                    "{jname}: a run block moves the checkout: {run}"
                );
            }
        }
    }
    assert!(!raw.contains("needs.resolve"));
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

    // A platform with neither file is missing, not partial: still refused.
    let gone = remote_name("macos-intel");
    fs::remove_file(ws.p().join("dist").join(format!("{gone}.sha256"))).unwrap();
    let absent = ws.run("publish", "verify_all", &e);
    assert!(!absent.status.success());
    assert!(text(&absent).contains("missing asset"), "{}", text(&absent));
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

    // A platform that never reached the release is not 'complete'.
    let mut four = ws.release_assets();
    four.retain(|n| !n.starts_with(&remote_name("linux-arm64")));
    ws.set_release_assets(&four);
    let short = ws.run("publish", "after", &e);
    assert!(!short.status.success());
    assert!(
        text(&short).contains("not all 'complete'"),
        "{}",
        text(&short)
    );

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

// ---------------------------------------------------------------------------
// Publish path under failure (stub gh; no network, no token, no upload)
// ---------------------------------------------------------------------------

#[test]
fn upload_stops_at_the_first_failure_and_a_repeat_run_refuses_the_leftovers() {
    let ws = Ws::new();
    full_dist(&ws);
    let e = publish_exprs();

    // The second `gh` call fails before storing anything: platform 1 is on the
    // release, platforms 2 to 5 are not, and the step exits non-zero.
    ws.fail_gh_on_call(2);
    let up = ws.run("publish", "upload", &e);
    assert!(!up.status.success(), "a failed upload must fail the step");
    assert_eq!(
        ws.upload_calls().len(),
        2,
        "no upload after the failure:\n{}",
        ws.gh_log()
    );
    assert_eq!(
        ws.release_assets().len(),
        2,
        "only the first pair was stored"
    );

    // Repeat run: the release already holds -remote assets, so `before` stops it
    // before any upload; `after` also refuses the incomplete set.
    let calls_before = ws.upload_calls().len();
    let again = ws.run("publish", "before", &e);
    assert!(!again.status.success());
    assert!(
        text(&again).contains("not all 'absent'"),
        "{}",
        text(&again)
    );
    assert_eq!(
        ws.upload_calls().len(),
        calls_before,
        "a repeat must not upload"
    );
    let after = ws.run("publish", "after", &e);
    assert!(!after.status.success());
    assert!(
        text(&after).contains("not all 'complete'"),
        "{}",
        text(&after)
    );
}

#[test]
fn upload_dying_inside_a_pair_leaves_a_partial_pair_that_a_repeat_run_refuses() {
    let ws = Ws::new();
    full_dist(&ws);
    let e = publish_exprs();

    // Call 3 stores the binary of platform 3 but not its checksum, then fails.
    ws.gh_dies_mid_pair_on_call(3);
    let up = ws.run("publish", "upload", &e);
    assert!(!up.status.success());
    assert_eq!(ws.upload_calls().len(), 3);
    let held = ws.release_assets();
    assert_eq!(
        held.len(),
        5,
        "2 complete pairs and 1 lone binary: {held:?}"
    );
    assert!(held.contains(&remote_name("macos-intel")));
    assert!(!held.contains(&format!("{}.sha256", remote_name("macos-intel"))));

    // The leftover is reported as a partial pair, and nothing is uploaded again.
    let again = ws.run("publish", "before", &e);
    assert!(!again.status.success());
    assert!(
        text(&again).contains("partial binary/checksum pair"),
        "{}",
        text(&again)
    );
    assert_eq!(ws.upload_calls().len(), 3, "a repeat must not upload");
}

#[test]
fn upload_touches_only_remote_names_and_leaves_stable_assets_alone() {
    let ws = Ws::new();
    full_dist(&ws);
    let e = publish_exprs();
    let stable = [
        "caro-1.5.0-linux-amd64",
        "caro-1.5.0-linux-amd64.sha256",
        "caro-1.5.0-macos-silicon",
        "caro-1.5.0-macos-silicon.sha256",
        "README.md",
    ];
    ws.set_release_assets(&stable.map(String::from));

    // Stable assets on the release are not -remote assets: `before` passes.
    let before = ws.run("publish", "before", &e);
    assert!(before.status.success(), "{}", text(&before));
    let up = ws.run("publish", "upload", &e);
    assert!(up.status.success(), "{}", text(&up));

    let calls = ws.upload_calls();
    assert_eq!(calls.len(), 5);
    for c in &calls {
        for file in c.split_whitespace().skip(4) {
            assert!(file.contains("-remote"), "uploaded a non-remote file: {c}");
        }
        assert!(!c.contains("--clobber"), "{c}");
    }
    let held = ws.release_assets();
    for name in stable {
        assert_eq!(
            held.iter().filter(|n| n.as_str() == name).count(),
            1,
            "stable asset {name} must stay exactly once: {held:?}"
        );
    }
    assert!(
        ws.gh_log()
            .lines()
            .all(|l| l.starts_with("gh api") || l.starts_with("gh release upload")),
        "only `api` and `release upload` are ever called:\n{}",
        ws.gh_log()
    );
}

#[test]
fn prepare_makes_no_gh_call_and_a_checksum_mismatch_stops_at_verify_all_before_any_gh_call() {
    let ws = Ws::new();
    full_dist(&ws);
    let e = publish_exprs();

    // The prepare step (all five platforms) makes no gh call. The publish=false
    // gate itself is pinned by the `if` check in the isolation test.
    assert_eq!(ws.gh_log(), "", "building and preparing must not call gh");

    // A checksum mismatch stops at verify_all, with no gh call at all. (Step
    // order and gating are pinned by the publish-order test.)
    fs::write(
        ws.p().join("dist").join(remote_name("linux-amd64")),
        "#!/bin/sh\necho tampered\n",
    )
    .unwrap();
    let bad = ws.run("publish", "verify_all", &e);
    assert!(!bad.status.success());
    assert!(text(&bad).contains("checksum mismatch"), "{}", text(&bad));
    assert_eq!(ws.gh_log(), "", "a bad dist must stop before any gh call");
}

#[test]
fn publish_steps_run_in_order_and_nothing_can_skip_a_failed_check() {
    let wf = workflow();
    let publish = job(&wf, "publish");
    let ids: Vec<String> = steps(publish)
        .iter()
        .filter_map(|s| s.get("id").and_then(Value::as_str).map(str::to_string))
        .collect();
    assert_eq!(
        ids,
        ["pin", "verify_all", "before", "upload", "after"],
        "publish steps must verify, check the release, upload, then confirm"
    );
    for st in steps(publish) {
        assert!(
            st.get("continue-on-error").is_none(),
            "no publish step may continue on error: {st:?}"
        );
        assert!(
            st.get("if").is_none(),
            "no publish step may carry an `if` (a failed step must stop the job): {st:?}"
        );
    }
    // And the build job never reaches the release: no step there calls the
    // script's gh-backed subcommands.
    for st in steps(job(&wf, "build")) {
        if let Some(run) = st.get("run").and_then(Value::as_str) {
            for sub in ["release-assets", "release-state", " upload "] {
                assert!(
                    !run.contains(&format!("release-remote.sh{sub}"))
                        && !run.contains(&format!("release-remote.sh {}", sub.trim())),
                    "build job must not call `{sub}`: {run}"
                );
            }
        }
    }
}

#[test]
fn pin_rejects_a_version_with_build_metadata() {
    let ws = Ws::new();
    let commit = make_repo(ws.p(), "1.6.0+meta", "v1.6.0+meta");
    let out = ws.run(
        "build",
        "pin",
        &pin_exprs("tag", "v1.6.0+meta", &commit, "v1.6.0+meta"),
    );
    assert!(!out.status.success(), "build metadata must be refused");
    assert!(text(&out).contains("not a version"), "{}", text(&out));
}

#[test]
fn the_repair_hint_names_every_leftover_asset_not_just_the_stray_one() {
    let ws = Ws::new();
    let e = publish_exprs();
    // Two complete pairs and one lone binary, as left by an upload that died.
    let mut names = names_for(&["linux-amd64", "linux-arm64"], true);
    names.push(remote_name("macos-intel"));
    ws.set_release_assets(&names);
    let out = ws.run("publish", "before", &e);
    assert!(!out.status.success());
    let t = text(&out);
    assert!(t.contains("partial binary/checksum pair"), "{t}");
    assert!(t.contains("remove every caro-1.5.0-*-remote* asset"), "{t}");

    // Complete leftovers (no lone file) get the same instruction.
    ws.set_release_assets(&names_for(&["linux-amd64"], true));
    let out = ws.run("publish", "before", &e);
    assert!(!out.status.success());
    assert!(text(&out).contains("not all 'absent'"), "{}", text(&out));
    assert!(
        text(&out).contains("remove every caro-1.5.0-*-remote* asset"),
        "{}",
        text(&out)
    );
}

// ---------------------------------------------------------------------------
// CVE ruleset contract in the optional release path (#1652 robustness)
// ---------------------------------------------------------------------------

const NATIVE: (&str, &str, &str) = ("x86_64-unknown-linux-gnu", "linux-amd64", "caro");
const ARM64: (&str, &str, &str) = ("aarch64-unknown-linux-gnu", "linux-arm64", "caro");

fn smoke_with(ws: &Ws, t: (&str, &str, &str), body: &str) -> Output {
    place_binary(ws, t.0, t.2, body);
    let e = build_exprs(t.0, t.1, t.2);
    let p = ws.run("build", "prepare", &refs(&e));
    assert!(p.status.success(), "prepare: {}", text(&p));
    ws.run("build", "smoke", &refs(&e))
}

#[test]
fn smoke_block_rejects_a_build_whose_cve_ruleset_is_missing_or_empty() {
    let cases: Vec<(&str, String, &str)> = vec![
        (
            "ruleset compiled to zero rules (trigger rated safe)",
            fixture(true, Cve::ZeroRules, true),
            "was not blocked",
        ),
        (
            "trigger denied without citing the CVE rule",
            fixture(true, Cve::NoCveText, true),
            "does not cite cve-2024-3094",
        ),
        (
            "guard denies a benign command too (the check would prove nothing)",
            fixture(true, Cve::AlwaysDeny, true),
            "benign control was denied",
        ),
        (
            "rule sources absent from the binary",
            fixture(true, Cve::Ok, false),
            "not found in the binary",
        ),
    ];
    for (label, body, expect) in cases {
        let ws = Ws::new();
        let out = smoke_with(&ws, NATIVE, &body);
        assert!(
            !out.status.success(),
            "{label}: smoke must fail:\n{}",
            text(&out)
        );
        assert!(
            text(&out).contains(expect),
            "{label}: want '{expect}' in:\n{}",
            text(&out)
        );
    }
    // Cross-built arm64 cannot run on the runner: only the static check applies.
    let ws = Ws::new();
    let out = smoke_with(&ws, ARM64, &arm64_fixture(false));
    assert!(
        !out.status.success(),
        "arm64 without rule sources must fail"
    );
    assert!(
        text(&out).contains("not found in the binary"),
        "{}",
        text(&out)
    );
}

#[test]
fn smoke_block_accepts_a_build_with_the_cve_ruleset_and_never_runs_the_trigger() {
    let ws = Ws::new();
    let out = smoke_with(&ws, NATIVE, &fixture(true, Cve::Ok, true));
    assert!(out.status.success(), "{}", text(&out));
    let t = text(&out);
    assert!(t.contains("CVE ruleset ok"), "{t}");
    // The CVE-2024-3094 trigger string is validator input only. The tripwire
    // `xz` on PATH was never called.
    let log = fs::read_to_string(ws.p().join("xz.log")).unwrap_or_default();
    assert_eq!(log, "", "the trigger command must never be executed");

    let ws = Ws::new();
    let out = smoke_with(&ws, ARM64, &arm64_fixture(true));
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("not run: cross-built"),
        "the arm64 limitation must be printed: {}",
        text(&out)
    );
}

#[test]
fn preflight_block_requires_cve_rules_and_a_fresh_build_directory() {
    let ws = Ws::new();
    let ok = ws.run("build", "preflight", &[]);
    assert!(ok.status.success(), "{}", text(&ok));
    assert!(text(&ok).contains("compiled CVE rule"), "{}", text(&ok));

    // A build directory that already exists could hold a stale build script.
    let ws = Ws::new();
    fs::create_dir_all(ws.p().join("target")).unwrap();
    let stale = ws.run("build", "preflight", &[]);
    assert!(!stale.status.success());
    assert!(text(&stale).contains("not fresh"), "{}", text(&stale));

    // Rules directory missing (vendored tarball, moved checkout).
    let ws = Ws::new();
    fs::remove_dir_all(ws.p().join("data/cve_rules")).unwrap();
    let missing = ws.run("build", "preflight", &[]);
    assert!(!missing.status.success());
    assert!(
        text(&missing).contains("no CVE rules"),
        "{}",
        text(&missing)
    );

    // Rules directory present but nothing compilable: template and draft only.
    let ws = Ws::new();
    let dir = ws.p().join("data/cve_rules");
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().unwrap().to_string_lossy() != "EXAMPLE-TEMPLATE.yaml" {
            fs::remove_file(path).unwrap();
        }
    }
    fs::write(
        dir.join("CVE-9999-0001.yaml"),
        "id: CVE-9999-0001\nsource: https://example.test/x\npattern: \"TODO_NIMBLE_AUTHORED\"\n",
    )
    .unwrap();
    let drafts = ws.run("build", "preflight", &[]);
    assert!(!drafts.status.success());
    assert!(text(&drafts).contains("no CVE rules"), "{}", text(&drafts));
}

#[test]
fn build_job_pins_then_preflights_then_builds_and_runs_the_cve_contract_on_linux() {
    let wf = workflow();
    let build = job(&wf, "build");
    let st = steps(build);
    let idx = |pred: &dyn Fn(&Value) -> bool, what: &str| {
        st.iter()
            .position(pred)
            .unwrap_or_else(|| panic!("no step: {what}"))
    };
    let run_has = |needle: &'static str| {
        move |s: &Value| {
            s.get("run")
                .and_then(Value::as_str)
                .is_some_and(|r| r.contains(needle))
        }
    };
    let pin = idx(
        &|s| s.get("id").and_then(Value::as_str) == Some("pin"),
        "pin",
    );
    let pre = idx(
        &|s| s.get("id").and_then(Value::as_str) == Some("preflight"),
        "preflight",
    );
    let tool = idx(&run_has("rustup toolchain install"), "toolchain");
    let cargo = idx(&run_has("cargo build"), "build");
    let contract = idx(&run_has("safety_text_contract"), "cve contract");
    let prepare = idx(
        &|s| s.get("id").and_then(Value::as_str) == Some("prepare"),
        "prepare",
    );
    assert!(
        pin < pre && pre < tool && tool < cargo,
        "pin, preflight, toolchain, build"
    );
    assert!(
        cargo < contract && contract < prepare,
        "contract runs after the build, before prepare"
    );

    let c = &st[contract];
    let run = get(c, "run").as_str().unwrap();
    assert!(run.contains("--features remote-backends"), "{run}");
    assert!(run.contains("--test safety_text_contract"), "{run}");
    let cond = get(c, "if").as_str().unwrap();
    assert!(
        cond.contains("x86_64-unknown-linux-gnu"),
        "the source-level contract runs on the native Linux leg only: {cond}"
    );
    assert!(c.get("continue-on-error").is_none());
    // Every leg gets the artifact-level check through the smoke step.
    let smoke = get(step(build, "smoke"), "run").as_str().unwrap();
    assert!(smoke.contains("release-remote.sh smoke"), "{smoke}");
}
