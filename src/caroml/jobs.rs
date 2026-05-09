//! `caro do` resolver + JOB runner.
//!
//! Resolves a name (typed by the user as `caro do <name>`) into one of:
//!
//! - **Job**: a `JOB <name>` declared in the project's Carofile — runs the
//!   sequence of `RUN <alias>` lines, dispatching each via `dispatch_alias`.
//! - **External alias**: a `USE "<command>" AS <name>` registers `<name>` as
//!   a shell-command alias; `dispatch_alias` runs the command directly.
//! - **Native task alias**: a `USE <path-to-.caro> AS <name>` aliases a
//!   CaroML task; `dispatch_alias` runs it via `caro run`.
//! - **Fallback**: if none of the above match, treat `<name>` as a bare task
//!   name and run via `caro run <name>` (this is the no-Carofile path).
//!
//! All execution goes through the existing `CommandExecutor` so the safety
//! validator can scan the resolved command (per the plan: "the safety
//! pipeline still runs on the resolved command").

use crate::caroml::carofile::{Carofile, UseDecl, UseTarget};
use crate::caroml::lock::Lock;
use crate::caroml::runner::{self, RunError};
use crate::execution::{CommandExecutor, ExecutionResult, ExecutorError};
use crate::models::ShellType;
use std::path::Path;
use thiserror::Error;

/// What `caro do <name>` resolved to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Run a JOB declared in the Carofile.
    Job { name: String, runs: Vec<String> },
    /// Run an external command alias (e.g. `npm test`).
    ExternalAlias { alias: String, command: String },
    /// Run a native task — caller dispatches to `caro run <task>`.
    NativeAlias {
        alias: String,
        task_path: std::path::PathBuf,
    },
    /// No Carofile match — treat as a bare task name.
    BareTask { name: String },
}

#[derive(Debug, Error)]
pub enum DoError {
    #[error("executor: {0}")]
    Executor(#[from] ExecutorError),
    #[error("alias `{alias}` resolved to a JOB but JOBs cannot be re-aliased")]
    InvalidAliasing { alias: String },
    #[error("step run error: {0}")]
    StepRun(#[from] RunError),
    #[error("{0}")]
    Other(String),
}

/// Resolve `name` against an optional Carofile. Returns the matched variant.
pub fn resolve(name: &str, carofile: Option<&Carofile>) -> Resolution {
    if let Some(cf) = carofile {
        if let Some(job) = cf.jobs.iter().find(|j| j.name == name) {
            return Resolution::Job {
                name: job.name.clone(),
                runs: job.runs.clone(),
            };
        }
        if let Some(use_decl) = cf.uses.iter().find(|u| u.alias == name) {
            return resolution_for_use(use_decl);
        }
    }
    Resolution::BareTask {
        name: name.to_string(),
    }
}

fn resolution_for_use(use_decl: &UseDecl) -> Resolution {
    match &use_decl.target {
        UseTarget::ExternalCommand(cmd) => Resolution::ExternalAlias {
            alias: use_decl.alias.clone(),
            command: cmd.clone(),
        },
        UseTarget::NativeTask(path) => Resolution::NativeAlias {
            alias: use_decl.alias.clone(),
            task_path: path.clone(),
        },
    }
}

/// Render the dispatch plan for `caro do --dry-run` — no execution.
pub fn render_plan(name: &str, resolution: &Resolution, carofile: Option<&Carofile>) -> String {
    let mut s = String::new();
    s.push_str(&format!("caro do {} →\n", name));
    match resolution {
        Resolution::Job { name, runs } => {
            s.push_str(&format!("  JOB {} ({} runs):\n", name, runs.len()));
            for (i, alias) in runs.iter().enumerate() {
                let nested = carofile
                    .and_then(|cf| cf.uses.iter().find(|u| u.alias == *alias))
                    .map(resolution_for_use)
                    .unwrap_or(Resolution::BareTask {
                        name: alias.clone(),
                    });
                s.push_str(&format!("    {}. {}", i + 1, render_inline(&nested)));
                s.push('\n');
            }
        }
        other => {
            s.push_str(&format!("  {}\n", render_inline(other)));
        }
    }
    s
}

fn render_inline(r: &Resolution) -> String {
    match r {
        Resolution::Job { name, .. } => format!("JOB {}", name),
        Resolution::ExternalAlias { alias, command } => {
            format!("alias `{}` → `{}`", alias, command)
        }
        Resolution::NativeAlias { alias, task_path } => {
            format!("alias `{}` → caro run {}", alias, task_path.display())
        }
        Resolution::BareTask { name } => format!("caro run {}", name),
    }
}

/// Dispatch one resolved alias. For a Job, recursively iterates its runs.
/// For external commands, executes via `CommandExecutor`. For native
/// aliases / bare tasks, calls `task_runner` (caller-supplied — keeps this
/// module free of CLI / I/O choice).
pub fn dispatch<F>(
    name: &str,
    carofile: Option<&Carofile>,
    mut task_runner: F,
) -> Result<Vec<ExecutionResult>, DoError>
where
    F: FnMut(&str, Option<&Path>) -> Result<Vec<ExecutionResult>, DoError>,
{
    let resolution = resolve(name, carofile);
    match resolution {
        Resolution::Job { runs, .. } => {
            let mut all_results = Vec::new();
            for alias in &runs {
                let nested =
                    match carofile.and_then(|cf| cf.uses.iter().find(|u| u.alias == *alias)) {
                        Some(use_decl) => resolution_for_use(use_decl),
                        None => Resolution::BareTask {
                            name: alias.clone(),
                        },
                    };
                let mut step_results = dispatch_resolution(&nested, &mut task_runner)?;
                all_results.append(&mut step_results);
            }
            Ok(all_results)
        }
        other => dispatch_resolution(&other, &mut task_runner),
    }
}

fn dispatch_resolution<F>(
    resolution: &Resolution,
    task_runner: &mut F,
) -> Result<Vec<ExecutionResult>, DoError>
where
    F: FnMut(&str, Option<&Path>) -> Result<Vec<ExecutionResult>, DoError>,
{
    match resolution {
        Resolution::Job { name, .. } => Err(DoError::InvalidAliasing {
            alias: name.clone(),
        }),
        Resolution::ExternalAlias { command, .. } => {
            let exec = CommandExecutor::new(ShellType::Bash);
            let result = exec.execute(command)?;
            if result.exit_code != 0 {
                return Err(DoError::Other(format!(
                    "external alias exited {}: {}\nstderr: {}",
                    result.exit_code, command, result.stderr
                )));
            }
            Ok(vec![result])
        }
        Resolution::NativeAlias { alias, task_path } => task_runner(alias, Some(task_path)),
        Resolution::BareTask { name } => task_runner(name, None),
    }
}

/// Convenience runner used by `caro do <bare-task>`: load the task's lock
/// and execute it on `platform`. The lock must already exist.
pub fn run_native_task(task_path: &Path, platform: &str) -> Result<Vec<ExecutionResult>, DoError> {
    let lock_path = task_path.with_extension("caro.lock");
    let lock = Lock::read_path(&lock_path).map_err(|e| DoError::Other(e.to_string()))?;
    let plan = runner::plan_run(&lock, platform).map_err(DoError::StepRun)?;
    runner::execute_plan(&plan).map_err(DoError::StepRun)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caroml::carofile::{parse, Carofile};

    fn cf(src: &str) -> Carofile {
        parse(src).expect("parse")
    }

    #[test]
    fn resolve_finds_job() {
        let c = cf("\
TASK demo
USE \"npm test\" AS test
JOB ci
  RUN test
");
        match resolve("ci", Some(&c)) {
            Resolution::Job { name, runs } => {
                assert_eq!(name, "ci");
                assert_eq!(runs, vec!["test".to_string()]);
            }
            other => panic!("expected Job, got {:?}", other),
        }
    }

    #[test]
    fn resolve_finds_external_alias() {
        let c = cf("TASK demo\nUSE \"npm test\" AS test\n");
        match resolve("test", Some(&c)) {
            Resolution::ExternalAlias { alias, command } => {
                assert_eq!(alias, "test");
                assert_eq!(command, "npm test");
            }
            other => panic!("expected ExternalAlias, got {:?}", other),
        }
    }

    #[test]
    fn resolve_finds_native_alias() {
        let c = cf("TASK demo\nUSE tasks/cleanup-logs.caro AS cleanup-logs\n");
        match resolve("cleanup-logs", Some(&c)) {
            Resolution::NativeAlias { alias, task_path } => {
                assert_eq!(alias, "cleanup-logs");
                assert_eq!(task_path.to_string_lossy(), "tasks/cleanup-logs.caro");
            }
            other => panic!("expected NativeAlias, got {:?}", other),
        }
    }

    #[test]
    fn resolve_falls_back_to_bare_task() {
        let c = cf("TASK demo\n");
        match resolve("nonexistent", Some(&c)) {
            Resolution::BareTask { name } => assert_eq!(name, "nonexistent"),
            other => panic!("expected BareTask, got {:?}", other),
        }
    }

    #[test]
    fn resolve_without_carofile_returns_bare_task() {
        match resolve("anything", None) {
            Resolution::BareTask { name } => assert_eq!(name, "anything"),
            other => panic!("expected BareTask, got {:?}", other),
        }
    }

    #[test]
    fn render_plan_job_lists_each_run() {
        let c = cf("\
TASK demo
USE \"npm test\" AS test
USE \"make build\" AS build
JOB ci
  RUN test
  RUN build
");
        let resolution = resolve("ci", Some(&c));
        let rendered = render_plan("ci", &resolution, Some(&c));
        assert!(rendered.contains("JOB ci (2 runs)"));
        assert!(rendered.contains("npm test"));
        assert!(rendered.contains("make build"));
    }
}
