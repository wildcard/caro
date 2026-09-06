//! Execution-grounded evaluator
//!
//! Every other evaluator in this module judges the generated command *string*.
//! This one actually runs the command in a disposable sandbox and scores what
//! happened: exit code, stdout, and filesystem effects. The sandbox is reached
//! over the provider-neutral JSONL protocol in `tools/exec-harness/PROTOCOL.md`;
//! tier 0 is `just-bash` in a local Node child process (in-memory filesystem,
//! nothing ever touches the host).
//!
//! Grading philosophy: an engine gap is never a command failure. When the tier
//! cannot run at all (disabled, node missing, `npm ci` not run) or the engine
//! reports the command `unsupported`, the case is SKIPPED (passes with an
//! explanatory `actual_behavior`), so pass-rates measure command quality, not
//! harness availability. See `docs/adr/ADR-017-cloud-assisted-verification.md`.

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::evaluation::errors::Result;
use crate::evaluation::evaluators::{CommandResult, Evaluator};
use crate::evaluation::{ErrorType, EvaluationResult, TestCase, TestCategory, Tier0Support};

/// Per-command execution budget sent to the runner. Kept well under the
/// Evaluator trait's 5-second contract (the runner adds snapshot overhead).
const EXEC_TIMEOUT_MS: u64 = 3_000;

/// Which execution tier backs `TestCategory::Execution` cases.
///
/// `Off` is the default everywhere: execution grounding is strictly opt-in
/// (`--execution-tier tier0` on the eval CLI) and its absence never fails a
/// run. Remote tiers (Cloudflare containers) are future variants — see
/// ADR-017 for the policy on which tiers may back CI-blocking jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionTier {
    /// Execution cases are skipped (default)
    #[default]
    Off,
    /// just-bash in a local Node child process (no network, no secrets)
    Tier0,
}

/// Response shape of the exec-harness protocol (PROTOCOL.md).
#[derive(Debug, Clone, Deserialize)]
struct ExecResponse {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    exit_code: i64,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    unsupported: bool,
    #[serde(default)]
    timed_out: bool,
    #[serde(default)]
    fs_diff: FsDiff,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct FsDiff {
    #[serde(default)]
    created: Vec<String>,
    #[serde(default)]
    removed: Vec<String>,
    #[serde(default)]
    modified: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ExecRequest<'a> {
    id: &'a str,
    command: &'a str,
    fixture_files: &'a HashMap<String, String>,
    timeout_ms: u64,
}

/// A running tier-0 server child process.
struct Tier0Runner {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Tier0Runner {
    /// Locates `tools/exec-harness` — env override first, then the repo the
    /// binary was compiled from, then the current directory. Internal tooling:
    /// eval runs happen from a checkout.
    fn harness_dir() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var("CARO_EXEC_HARNESS_DIR") {
            return Some(PathBuf::from(dir));
        }
        [
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tools/exec-harness"),
            PathBuf::from("tools/exec-harness"),
        ]
        .into_iter()
        .find(|candidate| candidate.join("src/serve.mjs").exists())
    }

    fn start() -> std::result::Result<Self, String> {
        let dir = Self::harness_dir().ok_or_else(|| {
            "tools/exec-harness not found (set CARO_EXEC_HARNESS_DIR to override)".to_string()
        })?;
        let node = std::env::var("CARO_NODE_BIN").unwrap_or_else(|_| "node".to_string());

        let mut child = Command::new(&node)
            .arg(dir.join("src/serve.mjs"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to spawn `{node}`: {e}"))?;

        let stdin = child.stdin.take().expect("stdin piped");
        let stdout = BufReader::new(child.stdout.take().expect("stdout piped"));
        let mut runner = Self {
            child,
            stdin,
            stdout,
        };

        // Handshake proves the engine loaded (a missing `npm ci` fails here).
        let pong = runner
            .round_trip(&serde_json::json!({"op": "ping"}).to_string())
            .map_err(|e| format!("handshake failed: {e} (run `npm ci` in tools/exec-harness)"))?;
        if !pong.contains("\"pong\"") {
            return Err(format!("unexpected handshake response: {pong}"));
        }
        Ok(runner)
    }

    fn round_trip(&mut self, line: &str) -> std::result::Result<String, String> {
        writeln!(self.stdin, "{line}").map_err(|e| format!("write failed: {e}"))?;
        self.stdin
            .flush()
            .map_err(|e| format!("flush failed: {e}"))?;
        let mut response = String::new();
        let read = self
            .stdout
            .read_line(&mut response)
            .map_err(|e| format!("read failed: {e}"))?;
        if read == 0 {
            return Err("runner exited (EOF)".to_string());
        }
        Ok(response)
    }
}

impl Drop for Tier0Runner {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

enum RunnerState {
    NotStarted,
    Ready(Box<Tier0Runner>),
    /// Start failed once; remembered so every subsequent case skips with the
    /// same reason instead of re-spawning node per case.
    Unavailable(String),
}

/// Evaluator for `TestCategory::Execution`: delegates to the configured
/// execution tier and grades observed behavior against
/// `TestCase::execution.expected`.
pub struct ExecutionEvaluator {
    tier: ExecutionTier,
    runner: Arc<Mutex<RunnerState>>,
}

impl ExecutionEvaluator {
    pub fn new(tier: ExecutionTier) -> Self {
        Self {
            tier,
            runner: Arc::new(Mutex::new(RunnerState::NotStarted)),
        }
    }

    /// Runs one command through the tier-0 runner. `Err(reason)` means the
    /// harness itself is unavailable/broken — callers turn that into a SKIP.
    fn run_tier0(
        runner: &Mutex<RunnerState>,
        test_case: &TestCase,
        command: &str,
    ) -> std::result::Result<ExecResponse, String> {
        let empty = HashMap::new();
        let fixture_files = test_case
            .execution
            .as_ref()
            .map(|e| &e.fixture_files)
            .unwrap_or(&empty);
        let request = serde_json::to_string(&ExecRequest {
            id: &test_case.id,
            command,
            fixture_files,
            timeout_ms: EXEC_TIMEOUT_MS,
        })
        .map_err(|e| format!("request encode failed: {e}"))?;

        let mut state = runner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if matches!(*state, RunnerState::NotStarted) {
            *state = match Tier0Runner::start() {
                Ok(r) => RunnerState::Ready(Box::new(r)),
                Err(reason) => RunnerState::Unavailable(reason),
            };
        }
        let active = match &mut *state {
            RunnerState::Ready(r) => r,
            RunnerState::Unavailable(reason) => return Err(reason.clone()),
            RunnerState::NotStarted => unreachable!("initialized above"),
        };

        match active.round_trip(&request) {
            Ok(line) => serde_json::from_str::<ExecResponse>(&line)
                .map_err(|e| format!("bad runner response: {e}")),
            Err(reason) => {
                // A dead runner poisons the tier for the rest of the run.
                *state = RunnerState::Unavailable(reason.clone());
                Err(reason)
            }
        }
    }

    fn skip(&self, test_case: &TestCase, result: &CommandResult, reason: &str) -> EvaluationResult {
        EvaluationResult {
            test_id: test_case.id.clone(),
            backend_name: result.backend_name.clone(),
            passed: true,
            actual_command: result.command.clone(),
            actual_behavior: Some(format!("skipped: {reason}")),
            failure_reason: None,
            execution_time_ms: result.execution_time_ms,
            timestamp: Utc::now(),
            error_type: None,
            est_tokens_in: 0,
            est_tokens_out: 0,
            est_cost_usd: 0.0,
            criteria_passed: 0,
            criteria_total: 0,
        }
    }

    fn fail(
        &self,
        test_case: &TestCase,
        result: &CommandResult,
        error_type: ErrorType,
        reason: String,
    ) -> EvaluationResult {
        EvaluationResult {
            test_id: test_case.id.clone(),
            backend_name: result.backend_name.clone(),
            passed: false,
            actual_command: result.command.clone(),
            actual_behavior: None,
            failure_reason: Some(reason),
            execution_time_ms: result.execution_time_ms,
            timestamp: Utc::now(),
            error_type: Some(error_type),
            est_tokens_in: 0,
            est_tokens_out: 0,
            est_cost_usd: 0.0,
            criteria_passed: 0,
            criteria_total: 0,
        }
    }
}

/// Pure grading of a runner response against a case's expected effects.
/// Returns (passed, criteria_passed, criteria_total, failure_reason, behavior).
fn grade(
    test_case: &TestCase,
    response: &ExecResponse,
) -> (bool, u32, u32, Option<String>, String) {
    let expected = test_case
        .execution
        .as_ref()
        .map(|e| e.expected.clone())
        .unwrap_or_default();

    let mut failures: Vec<String> = Vec::new();
    let mut passed_count: u32 = 0;
    let mut total: u32 = 0;

    // Exit code is always one criterion; omitted expectation means 0.
    total += 1;
    let want_exit = i64::from(expected.exit_code.unwrap_or(0));
    if response.exit_code == want_exit {
        passed_count += 1;
    } else {
        failures.push(format!(
            "exit code {} (expected {})",
            response.exit_code, want_exit
        ));
    }

    if let Some(pattern) = &expected.stdout_pattern {
        total += 1;
        match regex::Regex::new(pattern) {
            Ok(re) if re.is_match(&response.stdout) => passed_count += 1,
            Ok(_) => failures.push(format!("stdout did not match /{pattern}/")),
            Err(e) => failures.push(format!("bad stdout_pattern /{pattern}/: {e}")),
        }
    }

    for (label, wanted, observed) in [
        (
            "created",
            &expected.files_created,
            &response.fs_diff.created,
        ),
        (
            "removed",
            &expected.files_removed,
            &response.fs_diff.removed,
        ),
        (
            "modified",
            &expected.files_modified,
            &response.fs_diff.modified,
        ),
    ] {
        for path in wanted {
            total += 1;
            // Dataset paths are workspace-relative; the runner reports absolute.
            let matched = observed
                .iter()
                .any(|p| p == path || p.strip_prefix("/work/") == Some(path.as_str()));
            if matched {
                passed_count += 1;
            } else {
                failures.push(format!("expected {path} to be {label}, it was not"));
            }
        }
    }

    let behavior = format!(
        "exit {}; created {:?}; removed {:?}; modified {:?}",
        response.exit_code,
        response.fs_diff.created,
        response.fs_diff.removed,
        response.fs_diff.modified
    );
    let passed = failures.is_empty();
    let failure_reason = if passed {
        None
    } else {
        Some(failures.join("; "))
    };
    (passed, passed_count, total, failure_reason, behavior)
}

#[async_trait]
impl Evaluator for ExecutionEvaluator {
    fn category(&self) -> TestCategory {
        TestCategory::Execution
    }

    async fn evaluate(
        &self,
        test_case: &TestCase,
        result: &CommandResult,
    ) -> Result<EvaluationResult> {
        if self.tier == ExecutionTier::Off {
            return Ok(self.skip(
                test_case,
                result,
                "execution tier disabled (--execution-tier off)",
            ));
        }

        // Execution cases assert runtime behavior, so the command must exist.
        if result.blocked {
            return Ok(self.fail(
                test_case,
                result,
                ErrorType::ValidationFailure,
                "command was blocked by safety validation; execution case expects it to run"
                    .to_string(),
            ));
        }
        let command = match &result.command {
            Some(c) => c.clone(),
            None => {
                return Ok(self.fail(
                    test_case,
                    result,
                    ErrorType::GenerationFailure,
                    format!(
                        "backend produced no command: {}",
                        result.error.as_deref().unwrap_or("unknown error")
                    ),
                ));
            }
        };

        // Cases labeled unsupported for this tier are dialect gaps, not bugs.
        if test_case.execution.as_ref().and_then(|e| e.tier0) == Some(Tier0Support::Unsupported) {
            return Ok(self.skip(test_case, result, "case labeled tier0=unsupported"));
        }

        let runner = Arc::clone(&self.runner);
        let case = test_case.clone();
        let outcome =
            tokio::task::spawn_blocking(move || Self::run_tier0(&runner, &case, &command))
                .await
                .map_err(|e| {
                    crate::evaluation::errors::EvaluationError::Other(format!(
                        "execution task panicked: {e}"
                    ))
                })?;

        let response = match outcome {
            Ok(r) => r,
            Err(reason) => {
                return Ok(self.skip(
                    test_case,
                    result,
                    &format!("tier0 exec harness unavailable: {reason}"),
                ))
            }
        };

        if !response.ok {
            return Ok(self.skip(
                test_case,
                result,
                &format!(
                    "harness error: {}",
                    response.error.as_deref().unwrap_or("unknown")
                ),
            ));
        }
        if response.unsupported {
            return Ok(self.skip(
                test_case,
                result,
                "command not implemented by tier0 engine (exit 127)",
            ));
        }
        if response.timed_out {
            return Ok(self.fail(
                test_case,
                result,
                ErrorType::Timeout,
                format!("execution exceeded {EXEC_TIMEOUT_MS}ms budget"),
            ));
        }

        let (passed, criteria_passed, criteria_total, failure_reason, behavior) =
            grade(test_case, &response);

        Ok(EvaluationResult {
            test_id: test_case.id.clone(),
            backend_name: result.backend_name.clone(),
            passed,
            actual_command: result.command.clone(),
            actual_behavior: Some(behavior),
            failure_reason,
            execution_time_ms: result.execution_time_ms,
            timestamp: Utc::now(),
            error_type: if passed {
                None
            } else {
                Some(ErrorType::IncorrectOutput)
            },
            est_tokens_in: 0,
            est_tokens_out: 0,
            est_cost_usd: 0.0,
            criteria_passed,
            criteria_total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::{ExecutionSpec, ExpectedEffects, ValidationRule};

    fn exec_case(expected: ExpectedEffects) -> TestCase {
        TestCase {
            id: "exec-test".to_string(),
            category: TestCategory::Execution,
            input_request: "sort data".to_string(),
            expected_command: None,
            expected_behavior: None,
            validation_rule: ValidationRule::MustExecute,
            validation_pattern: None,
            tags: vec![],
            difficulty: None,
            source: None,
            notes: None,
            execution: Some(ExecutionSpec {
                fixture_files: HashMap::new(),
                expected,
                tier0: None,
            }),
        }
    }

    fn generated(command: &str) -> CommandResult {
        CommandResult::success(command.to_string(), 5, "static_matcher".to_string())
    }

    fn response(exit_code: i64, stdout: &str, created: &[&str]) -> ExecResponse {
        ExecResponse {
            ok: true,
            error: None,
            exit_code,
            stdout: stdout.to_string(),
            unsupported: false,
            timed_out: false,
            fs_diff: FsDiff {
                created: created.iter().map(|s| s.to_string()).collect(),
                removed: vec![],
                modified: vec![],
            },
        }
    }

    #[test]
    fn grade_passes_on_default_exit_zero() {
        let case = exec_case(ExpectedEffects::default());
        let (passed, ok, total, reason, _) = grade(&case, &response(0, "", &[]));
        assert!(passed);
        assert_eq!((ok, total), (1, 1));
        assert!(reason.is_none());
    }

    #[test]
    fn grade_fails_on_unexpected_exit_code() {
        let case = exec_case(ExpectedEffects::default());
        let (passed, ok, total, reason, _) = grade(&case, &response(2, "", &[]));
        assert!(!passed);
        assert_eq!((ok, total), (0, 1));
        assert!(reason.unwrap().contains("exit code 2"));
    }

    #[test]
    fn grade_matches_workspace_relative_created_files() {
        let case = exec_case(ExpectedEffects {
            files_created: vec!["out/sorted.txt".to_string()],
            ..Default::default()
        });
        let (passed, ok, total, _, _) = grade(&case, &response(0, "", &["/work/out/sorted.txt"]));
        assert!(passed);
        assert_eq!((ok, total), (2, 2));
    }

    #[test]
    fn grade_scores_stdout_pattern() {
        let case = exec_case(ExpectedEffects {
            stdout_pattern: Some(r"^\s*2\b".to_string()),
            ..Default::default()
        });
        let (passed, ..) = grade(&case, &response(0, "2 matches\n", &[]));
        assert!(passed);
        let (failed, _, _, reason, _) = grade(&case, &response(0, "nope\n", &[]));
        assert!(!failed);
        assert!(reason.unwrap().contains("stdout"));
    }

    #[tokio::test]
    async fn off_tier_skips_and_passes() {
        let evaluator = ExecutionEvaluator::new(ExecutionTier::Off);
        let case = exec_case(ExpectedEffects::default());
        let result = evaluator.evaluate(&case, &generated("true")).await.unwrap();
        assert!(result.passed);
        assert!(result.actual_behavior.unwrap().starts_with("skipped:"));
    }

    #[tokio::test]
    async fn unavailable_runner_skips_and_passes() {
        std::env::set_var("CARO_EXEC_HARNESS_DIR", "/nonexistent/exec-harness");
        let evaluator = ExecutionEvaluator::new(ExecutionTier::Tier0);
        let case = exec_case(ExpectedEffects::default());
        let result = evaluator.evaluate(&case, &generated("true")).await.unwrap();
        std::env::remove_var("CARO_EXEC_HARNESS_DIR");
        assert!(result.passed);
        assert!(result.actual_behavior.unwrap().contains("unavailable"));
    }

    #[tokio::test]
    async fn blocked_command_fails_execution_case() {
        let evaluator = ExecutionEvaluator::new(ExecutionTier::Tier0);
        let case = exec_case(ExpectedEffects::default());
        let blocked = CommandResult::blocked(5, "static_matcher".to_string());
        let result = evaluator.evaluate(&case, &blocked).await.unwrap();
        assert!(!result.passed);
        assert_eq!(result.error_type, Some(ErrorType::ValidationFailure));
    }

    /// Full round-trip through the real Node runner. Requires `npm ci` in
    /// tools/exec-harness (matches the repo convention for env-gated tests).
    #[tokio::test]
    #[ignore = "requires node + npm ci in tools/exec-harness"]
    async fn live_tier0_round_trip() {
        let evaluator = ExecutionEvaluator::new(ExecutionTier::Tier0);
        let mut case = exec_case(ExpectedEffects {
            files_created: vec!["sorted.txt".to_string()],
            ..Default::default()
        });
        case.execution.as_mut().unwrap().fixture_files =
            HashMap::from([("data.txt".to_string(), "b\na\nb\n".to_string())]);
        let result = evaluator
            .evaluate(&case, &generated("sort -u data.txt > sorted.txt"))
            .await
            .unwrap();
        assert!(result.passed, "failure: {:?}", result.failure_reason);
        assert_eq!(result.criteria_total, 2);
    }
}
