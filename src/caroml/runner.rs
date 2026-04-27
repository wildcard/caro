//! `caro run` runner — plan-then-confirm + execution against a runbook.
//!
//! Two execution paths in v0.1:
//!
//! 1. **Runbook-first** (default): if the per-platform `.<plat>.sh` runbook
//!    exists and is hash-clean, just `bash` it. This is the cheapest path
//!    and the same artifact a non-Caro user would use.
//! 2. **Step-by-step from lock**: if the runbook is missing or drift-detected,
//!    execute each step's active variant via `CommandExecutor` directly.
//!
//! v0.1 keeps execution simple: sequential, stop on first non-zero exit,
//! preserve the lock unchanged regardless of execution outcome (execution
//! failure ≠ generation failure). Track-record updates land in PR 6.

use crate::caroml::lock::Lock;
use crate::execution::{CommandExecutor, ExecutionResult, ExecutorError};
use crate::models::ShellType;
use std::path::Path;
use thiserror::Error;

/// Decision from `plan_run` that the CLI uses to ask for confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPlan {
    pub platform: String,
    pub steps: Vec<PlanStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStep {
    pub line: usize,
    pub intent: String,
    pub command: String,
    pub risk_level: String,
    pub generation_id: String,
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("no active variant for platform `{platform}` on step at line {line}")]
    MissingVariant { line: usize, platform: String },
    #[error("step `{intent}` (line {line}) failed: exit {exit_code}\nstderr: {stderr}")]
    StepFailed {
        line: usize,
        intent: String,
        exit_code: i32,
        stderr: String,
    },
    #[error("executor error: {0}")]
    Executor(#[from] ExecutorError),
}

/// Build the run plan for `platform` (no execution).
pub fn plan_run(lock: &Lock, platform: &str) -> Result<RunPlan, RunError> {
    let mut steps = Vec::with_capacity(lock.steps.len());
    for step in &lock.steps {
        let variant = step
            .active_variant(platform)
            .ok_or_else(|| RunError::MissingVariant {
                line: step.line,
                platform: platform.to_string(),
            })?;
        steps.push(PlanStep {
            line: step.line,
            intent: step.intent.clone(),
            command: variant.command.clone(),
            risk_level: variant.risk_level.clone(),
            generation_id: variant.generation_id.clone(),
        });
    }
    Ok(RunPlan {
        platform: platform.to_string(),
        steps,
    })
}

/// Execute the plan step-by-step via `CommandExecutor`. Returns the per-step
/// results in order. On the first non-zero exit, returns `Err(StepFailed)`.
pub fn execute_plan(plan: &RunPlan) -> Result<Vec<ExecutionResult>, RunError> {
    let executor = CommandExecutor::new(shell_for(&plan.platform));
    let mut results = Vec::with_capacity(plan.steps.len());
    for step in &plan.steps {
        let result = executor.execute(&step.command)?;
        if result.exit_code != 0 {
            return Err(RunError::StepFailed {
                line: step.line,
                intent: step.intent.clone(),
                exit_code: result.exit_code,
                stderr: result.stderr.clone(),
            });
        }
        results.push(result);
    }
    Ok(results)
}

/// Execute a runbook file (`.<platform>.sh`) directly via `bash`.
/// Returns the exit code on success.
pub fn execute_runbook(path: &Path) -> Result<i32, RunError> {
    let executor = CommandExecutor::new(ShellType::Bash);
    let cmd = format!("bash {}", quote_path(path));
    let result = executor.execute(&cmd)?;
    Ok(result.exit_code)
}

/// Format a runbook plan for human display (used by `caro run` confirmation prompt).
pub fn render_plan(plan: &RunPlan) -> String {
    let mut s = String::new();
    s.push_str(&format!("Plan ({} steps on {}):\n", plan.steps.len(), plan.platform));
    for (i, step) in plan.steps.iter().enumerate() {
        s.push_str(&format!(
            "  {}. [{}] {} (line {})\n     {} {}\n",
            i + 1,
            step.risk_level,
            step.intent,
            step.line,
            "$",
            step.command
        ));
    }
    s
}

fn shell_for(platform: &str) -> ShellType {
    match platform {
        "windows" => ShellType::PowerShell,
        _ => ShellType::Bash,
    }
}

fn quote_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.contains(' ') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.into_owned()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caroml::lock::{Lock, Meta, Step as LockStep, Variant};
    use chrono::Utc;
    use std::collections::BTreeMap;

    fn variant(platform: &str, command: &str) -> Variant {
        Variant {
            platform: platform.into(),
            active: true,
            generation_id: "gen_a".into(),
            command: command.into(),
            reasoning: "".into(),
            exports: vec![],
            imports: vec![],
            risk_level: "safe".into(),
            matched_patterns: vec![],
            warnings: vec![],
            confidence: 1.0,
            iterations: 1,
            validations: vec![],
            generated_at: Utc::now(),
            model: "m".into(),
            backend: "b".into(),
            tool_versions: BTreeMap::new(),
            track_record: Default::default(),
            retired_at: None,
        }
    }

    fn lock_with_steps(commands: &[&str], platform: &str) -> Lock {
        let mut lock = Lock::default();
        lock.meta = Meta {
            caro_version: "1.4.0".into(),
            intent_path: "tasks/test.caro".into(),
            intent_hash: "sha256:test".into(),
            supported_platforms: vec![platform.into()],
            last_full_regen: Some(Utc::now()),
        };
        for (i, cmd) in commands.iter().enumerate() {
            lock.steps.push(LockStep {
                line: i + 2,
                intent: format!("step {}", i + 1),
                intent_hash: format!("sha256:s{}", i),
                notes: vec![],
                variants: vec![variant(platform, cmd)],
            });
        }
        lock
    }

    #[test]
    fn plan_run_collects_all_steps_for_platform() {
        let lock = lock_with_steps(&["echo a", "echo b", "echo c"], "linux");
        let plan = plan_run(&lock, "linux").unwrap();
        assert_eq!(plan.steps.len(), 3);
        assert_eq!(plan.steps[0].command, "echo a");
        assert_eq!(plan.steps[2].command, "echo c");
        assert_eq!(plan.platform, "linux");
    }

    #[test]
    fn plan_run_errors_on_missing_platform_variant() {
        let lock = lock_with_steps(&["echo a"], "linux");
        match plan_run(&lock, "macos") {
            Err(RunError::MissingVariant { line: 2, .. }) => {}
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn render_plan_includes_step_count_and_platform() {
        let lock = lock_with_steps(&["echo a", "echo b"], "linux");
        let plan = plan_run(&lock, "linux").unwrap();
        let rendered = render_plan(&plan);
        assert!(rendered.contains("Plan (2 steps on linux)"));
        assert!(rendered.contains("echo a"));
        assert!(rendered.contains("echo b"));
    }

    #[test]
    fn execute_plan_runs_each_step_against_bash() {
        // We use a fixture lock with `true` (always exit 0) — keeps the test
        // deterministic and dependency-free.
        let lock = lock_with_steps(&["true", "true", "true"], "linux");
        let plan = plan_run(&lock, "linux").unwrap();
        let results = execute_plan(&plan).unwrap();
        assert_eq!(results.len(), 3);
        for r in results {
            assert_eq!(r.exit_code, 0);
        }
    }

    #[test]
    fn execute_plan_stops_on_first_non_zero_exit() {
        let lock = lock_with_steps(&["true", "false", "true"], "linux");
        let plan = plan_run(&lock, "linux").unwrap();
        match execute_plan(&plan) {
            Err(RunError::StepFailed { exit_code, .. }) => {
                assert_ne!(exit_code, 0);
            }
            other => panic!("expected StepFailed, got {:?}", other),
        }
    }
}
