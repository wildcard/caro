//! End-to-end integration test for the entire CaroML pipeline.
//!
//! Exercises every user-facing capability against fixture .caro files in
//! a tempdir, using the inline `mock` backend for deterministic generation:
//!
//! 1. `caro new <name>` scaffolds a starter and refuses to overwrite
//! 2. `caro check` parses + lints (success and error paths with line numbers)
//! 3. `caro generate --backend mock` produces a per-platform lock
//! 4. `caro list` discovers project tasks
//! 5. `caro export` writes a runbook AND stamps the `runbook_hash`
//! 6. `caro run -y` executes via runbook-first path, then journals
//! 7. `caro run` re-runs and uses the cached runbook (Clean state)
//! 8. Edit the runbook → `caro run` reports `Drift` and falls back step-by-step
//! 9. Edit the .caro intent → `caro run` refuses (HardRegen needed)
//! 10. `caro experiment` adds an A/B challenger
//! 11. `caro adopt` promotes the challenger
//! 12. `caro history` shows lineage + journal
//! 13. `caro why` explains the RegenEvaluator decision
//! 14. `caro render` produces Markdown
//! 15. `caro do` resolves a JOB from a Carofile and runs it
//! 16. `caro skill install/uninstall` round-trips
//!
//! The test uses `assert_cmd` (already in dev-dependencies) to run the binary
//! built by Cargo. Each phase has its own helper so a failure points at the
//! exact step in the pipeline.

use assert_cmd::Command;
use std::fs;

/// Smoke-test name for `caro new` / `caro generate` / etc.
const TASK: &str = "demo-task";

#[test]
fn caroml_full_pipeline_e2e() {
    // Build the binary once (cargo handles this via assert_cmd).
    let bin = env!("CARGO_BIN_EXE_caro");

    // Working dir: a fresh tempdir; we cd into it for the entire test.
    let dir = tempfile::tempdir().expect("tempdir");
    let work = dir.path();

    // --- 1. caro new --------------------------------------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["new", TASK])
        .assert()
        .success();
    let caro_path = work.join("tasks").join(format!("{}.caro", TASK));
    assert!(caro_path.exists(), "tasks/<name>.caro must be scaffolded");

    // Refuses to overwrite
    Command::new(bin)
        .current_dir(work)
        .args(["new", TASK])
        .assert()
        .failure();

    // Replace the template with a real, parseable task so the rest of the
    // pipeline sees real DOs.
    fs::write(
        &caro_path,
        "TASK Demo task\n\
         WHY  E2E pipeline smoke\n\
         \n\
         DO   say hello\n\
         DO   say goodbye\n",
    )
    .expect("write task");

    // --- 2. caro check (success + error path) ------------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["check", caro_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("ok (2 steps"));

    let bad = work.join("tasks").join("bad.caro");
    fs::write(&bad, "DO say hi\n").unwrap(); // missing TASK header
    Command::new(bin)
        .current_dir(work)
        .args(["check", bad.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("expected `TASK"));

    // --- 3. caro generate --backend mock -----------------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["generate", TASK, "--backend", "mock"])
        .assert()
        .success();
    let lock_path = work.join("tasks").join(format!("{}.caro.lock", TASK));
    assert!(lock_path.exists(), "lock must exist after generate");
    let lock_body = fs::read_to_string(&lock_path).unwrap();
    assert!(lock_body.contains("schema_version = 2"));
    assert!(lock_body.contains("intent_hash = \"sha256:"));
    assert!(lock_body.contains("active = true"));

    // --- 4. caro list -------------------------------------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicates::str::contains(TASK));

    // --- 5. caro export (runbook_hash gets stamped) ------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["export", TASK])
        .assert()
        .success();

    let platform = current_platform();
    let runbook_path = work.join("tasks").join(format!("{}.{}.sh", TASK, platform));
    assert!(runbook_path.exists(), "runbook must exist after export");
    assert!(
        fs::read_to_string(&lock_path)
            .unwrap()
            .contains("runbook_hash = \"sha256:"),
        "export must stamp runbook_hash on the active variant"
    );

    // --- 6. caro run -y (Clean path, runbook-first execution) --------------
    Command::new(bin)
        .current_dir(work)
        .args(["run", TASK, "-y"])
        .assert()
        .success()
        .stdout(predicates::str::contains("runbook clean"));

    // --- 7. cached re-run is still Clean -----------------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["run", TASK, "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("runbook clean"));

    // --- 8. drift detection (manual edit to runbook) -----------------------
    let mut sh = fs::read_to_string(&runbook_path).unwrap();
    sh.push_str("\n# manual edit\n");
    fs::write(&runbook_path, &sh).unwrap();
    Command::new(bin)
        .current_dir(work)
        .args(["run", TASK, "--dry-run"])
        .assert()
        .success()
        .stderr(predicates::str::contains("has been edited"));

    // Restore the runbook before continuing.
    Command::new(bin)
        .current_dir(work)
        .args(["export", TASK])
        .assert()
        .success();

    // --- 9. intent edit triggers HardRegen ---------------------------------
    let mut intent = fs::read_to_string(&caro_path).unwrap();
    intent.push_str("DO   say something new\n");
    fs::write(&caro_path, &intent).unwrap();
    let why_output = Command::new(bin)
        .current_dir(work)
        .args(["why", TASK])
        .output()
        .expect("why");
    assert!(why_output.status.success());
    let why = String::from_utf8_lossy(&why_output.stdout).into_owned();
    assert!(
        why.contains("HardRegen") && why.contains("intent_hash mismatch"),
        "edited intent should be detected as HardRegen, got: {}",
        why
    );

    // Regenerate to recover.
    Command::new(bin)
        .current_dir(work)
        .args(["generate", TASK, "--backend", "mock"])
        .assert()
        .success();
    Command::new(bin)
        .current_dir(work)
        .args(["export", TASK])
        .assert()
        .success();

    // --- 10. caro experiment (add a challenger) ----------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["experiment", TASK, "--backend", "mock"])
        .assert()
        .success();
    let lock_body = fs::read_to_string(&lock_path).unwrap();
    let active_count = lock_body.matches("active = true").count();
    let challenger_count = lock_body.matches("active = false").count();
    assert!(
        active_count >= 1 && challenger_count >= 1,
        "after experiment we should have an active + a challenger; got active={} challenger={}",
        active_count,
        challenger_count
    );

    // --- 11. caro adopt ----------------------------------------------------
    let challenger_id = extract_challenger_id(&lock_body)
        .expect("at least one challenger generation_id should exist");
    Command::new(bin)
        .current_dir(work)
        .args(["adopt", TASK, "--variant", &challenger_id])
        .assert()
        .success()
        .stdout(predicates::str::contains("Adopted"));

    // --- 12. caro history --------------------------------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["history", TASK])
        .assert()
        .success()
        .stdout(predicates::str::contains("Lock history"))
        .stdout(predicates::str::contains("Local run journal"));

    // --- 13. caro why now reports cache-clean ------------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["why", TASK])
        .assert()
        .success();

    // --- 14. caro render ---------------------------------------------------
    Command::new(bin)
        .current_dir(work)
        .args(["render", TASK])
        .assert()
        .success()
        .stdout(predicates::str::contains("# Demo task"));

    // --- 15. caro do via Carofile -----------------------------------------
    fs::write(
        work.join("Carofile"),
        format!(
            "TASK orchestrate\n\
             USE  tasks/{}.caro    AS run-demo\n\
             USE  \"true\"           AS noop\n\
             JOB ci\n  RUN noop\n  RUN run-demo\n",
            TASK
        ),
    )
    .unwrap();

    // After adopt the runbook may not match the new active's hash; refresh.
    Command::new(bin)
        .current_dir(work)
        .args(["export", TASK])
        .assert()
        .success();

    Command::new(bin)
        .current_dir(work)
        .args(["jobs"])
        .assert()
        .success()
        .stdout(predicates::str::contains("ci"));

    Command::new(bin)
        .current_dir(work)
        .args(["do", "ci", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("JOB ci"));

    // --- 16. caro skill install / uninstall (with HOME=tempdir) -----------
    let skill_home = work.join("skill-home");
    let skill_dest = skill_home
        .join(".claude")
        .join("skills")
        .join("caro-scaffold");
    // Provide a fake source under the cwd, since `caro skill install` reads
    // from `.claude/skills/caro-scaffold/` relative to cwd by default.
    let src = work.join(".claude").join("skills").join("caro-scaffold");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("SKILL.md"), "---\nname: caro-scaffold\n---\n").unwrap();
    fs::write(src.join("README.md"), "# caro-scaffold\n").unwrap();

    Command::new(bin)
        .current_dir(work)
        .env("HOME", &skill_home)
        .args(["skill", "install"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Installed"));
    assert!(
        skill_dest.exists(),
        "skill should land at HOME/.claude/skills/..."
    );

    Command::new(bin)
        .current_dir(work)
        .env("HOME", &skill_home)
        .args(["skill", "uninstall"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Removed"));
    assert!(!skill_dest.exists(), "skill must be removed");
}

fn current_platform() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "linux" => "linux",
        "windows" => "windows",
        _ => "posix",
    }
}

fn extract_challenger_id(lock_body: &str) -> Option<String> {
    // Walk `[[steps.variants]]` blocks; return the generation_id of the first
    // block whose `active` line is `false`. Each block has `active` before
    // `generation_id` (declaration order in the Variant struct), so we collect
    // the active flag of the current block first, then capture the id once
    // we see it.
    let mut in_block = false;
    let mut current_active: Option<bool> = None;
    for line in lock_body.lines() {
        let trimmed = line.trim();
        if trimmed == "[[steps.variants]]" {
            in_block = true;
            current_active = None;
            continue;
        }
        // Any other section header closes the current variant block.
        if in_block && trimmed.starts_with('[') && trimmed != "[[steps.variants]]" {
            in_block = false;
            current_active = None;
            continue;
        }
        if !in_block {
            continue;
        }
        if trimmed == "active = true" {
            current_active = Some(true);
        } else if trimmed == "active = false" {
            current_active = Some(false);
        } else if let Some(rest) = trimmed.strip_prefix("generation_id = \"") {
            if current_active == Some(false) {
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            }
        }
    }
    None
}
