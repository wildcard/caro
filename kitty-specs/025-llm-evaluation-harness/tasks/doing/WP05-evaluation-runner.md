---
work_package_id: "WP05"
subtasks:
  - "T033"
  - "T034"
  - "T035"
  - "T036"
  - "T037"
  - "T038"
  - "T039"
  - "T040"
  - "T041"
  - "T042"
title: "Evaluation Runner Core Logic"
phase: "Phase 1 - Core Implementation"
lane: "doing"
assignee: ""
agent: "claude"
shell_pid: "31030"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-09T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP05 – Evaluation Runner Core Logic

## Objectives & Success Criteria

**Goal**: Implement evaluation harness that runs test cases against backends and collects results.

**Success Criteria**:
- Harness can execute single test case end-to-end
- Backend integration works (MLX primary)
- Command generation, normalization, and validation pipeline functions
- Timeout errors handled gracefully (30s limit)
- Malformed responses handled without crashing
- Integration tests validate happy path and error scenarios

## Context & Constraints

**References**:
- [plan.md](../../plan.md) - Evaluation flow architecture (lines 226-254)
- [data-model.md](../../data-model.md) - TestCase and TestResult structs (lines 18-73)
- Existing modules: `caro::backends::BackendTrait`, `caro::backends::mlx::MlxBackend`

**Key Architecture** (from plan.md):
> Evaluation Flow:
> 1. Load TestDataset from TOML
> 2. For each TestCase:
>    a. Generate command using backend
>    b. Normalize both expected and actual commands
>    c. Validate: correctness, safety, POSIX compliance
>    d. Collect result (pass/fail + reason)
> 3. Aggregate results into EvaluationResult

**Constraints**:
- MLX hardcoded for MVP (multi-backend in WP10)
- 30s timeout per test case
- No command execution (evaluation only)
- Use existing `tokio` runtime

## Subtasks & Detailed Guidance

### T033 – Define run_evaluation() Signature

Create in `tests/evaluation/harness.rs`:

```rust
use crate::dataset::TestDataset;
use std::path::Path;

/// Intermediate result structure for single test case
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub prompt: String,
    pub expected: String,
    pub actual: Option<String>,  // None if backend error
    pub passed: bool,
    pub reason: FailureReason,
}

#[derive(Debug, Clone)]
pub enum FailureReason {
    Pass,
    IncorrectCommand,
    SafetyMismatch { expected: bool, actual: bool },
    PosixMismatch { expected: bool, actual: bool },
    BackendError(String),
}

/// Run evaluation against test dataset
///
/// Returns Vec<TestResult> for further processing by metrics calculation.
pub async fn run_evaluation(dataset_path: &Path) -> Result<Vec<TestResult>, String> {
    // Implementation in following subtasks
    todo!()
}
```

### T034-T036 – Backend Integration and Test Execution Loop

Implement core logic in `harness.rs`:

```rust
use caro::backends::{BackendTrait, mlx::MlxBackend};
use tokio::time::{timeout, Duration};

pub async fn run_evaluation(dataset_path: &Path) -> Result<Vec<TestResult>, String> {
    // Load dataset
    let dataset = TestDataset::from_toml(dataset_path)?;

    // Initialize MLX backend
    let backend = MlxBackend::new()
        .map_err(|e| format!("Failed to initialize MLX backend: {}", e))?;

    let mut results = Vec::new();

    // Execute each test case
    for test_case in &dataset.test_cases {
        let result = execute_test_case(&backend, test_case).await;
        results.push(result);
    }

    Ok(results)
}

async fn execute_test_case(
    backend: &MlxBackend,
    test_case: &TestCase,
) -> TestResult {
    // Generate command with timeout
    let timeout_duration = Duration::from_secs(30);

    let actual_command = match timeout(
        timeout_duration,
        backend.generate_command(&test_case.prompt)
    ).await {
        Ok(Ok(cmd)) => Some(cmd),
        Ok(Err(e)) => {
            // Backend error (not timeout)
            return TestResult {
                test_id: test_case.id.clone(),
                prompt: test_case.prompt.clone(),
                expected: test_case.expected_command.clone(),
                actual: None,
                passed: false,
                reason: FailureReason::BackendError(format!("Backend error: {}", e)),
            };
        }
        Err(_) => {
            // Timeout
            return TestResult {
                test_id: test_case.id.clone(),
                prompt: test_case.prompt.clone(),
                expected: test_case.expected_command.clone(),
                actual: None,
                passed: false,
                reason: FailureReason::BackendError("Request timeout after 30s".to_string()),
            };
        }
    };

    // Continue to validation (next subtask)
    validate_command(test_case, actual_command.unwrap())
}
```

### T037-T038 – Validation Pipeline and Result Collection

Add validation logic in `harness.rs`:

```rust
use crate::validators::{commands_match, validate_safety, is_posix_compliant};

fn validate_command(test_case: &TestCase, actual: String) -> TestResult {
    // Normalize and compare commands
    let commands_equivalent = commands_match(&test_case.expected_command, &actual);

    // If command is incorrect, fail immediately
    if !commands_equivalent {
        return TestResult {
            test_id: test_case.id.clone(),
            prompt: test_case.prompt.clone(),
            expected: test_case.expected_command.clone(),
            actual: Some(actual),
            passed: false,
            reason: FailureReason::IncorrectCommand,
        };
    }

    // Check safety label matches
    let actual_safe = validate_safety(&actual);
    if actual_safe != test_case.safe {
        return TestResult {
            test_id: test_case.id.clone(),
            prompt: test_case.prompt.clone(),
            expected: test_case.expected_command.clone(),
            actual: Some(actual),
            passed: false,
            reason: FailureReason::SafetyMismatch {
                expected: test_case.safe,
                actual: actual_safe,
            },
        };
    }

    // Check POSIX compliance matches
    let actual_posix = is_posix_compliant(&actual);
    if actual_posix != test_case.posix_compliant {
        return TestResult {
            test_id: test_case.id.clone(),
            prompt: test_case.prompt.clone(),
            expected: test_case.expected_command.clone(),
            actual: Some(actual),
            passed: false,
            reason: FailureReason::PosixMismatch {
                expected: test_case.posix_compliant,
                actual: actual_posix,
            },
        };
    }

    // All checks passed
    TestResult {
        test_id: test_case.id.clone(),
        prompt: test_case.prompt.clone(),
        expected: test_case.expected_command.clone(),
        actual: Some(actual),
        passed: true,
        reason: FailureReason::Pass,
    }
}
```

### T039-T040 – Error Handling (already implemented above)

The timeout and backend error handling is already included in the `execute_test_case()` function. No additional code needed.

### T041-T042 – Integration Tests

Add to `tests/evaluation/harness.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_run_single_test_case() {
        // Create test dataset with single test case
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"
version = "1.0.0"
[[test_cases]]
id = "test_01"
prompt = "list all files"
expected_command = "ls -la"
category = "correctness"
safe = true
posix_compliant = true
        "#).unwrap();

        let results = run_evaluation(file.path()).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].test_id, "test_01");
        // Note: actual result depends on backend response
    }

    #[tokio::test]
    async fn test_handle_backend_timeout() {
        // This test would require mocking the backend to force a timeout
        // Placeholder for now - implement when backend mocking is available
        // TODO: Add mock backend that hangs, verify timeout handling
    }
}
```

## Dependencies

**Cargo.toml additions required**:
```toml
[dev-dependencies]
tokio = { version = "1.35", features = ["macros", "time"] }
tempfile = "3.8"
```

**Module imports**:
```rust
use caro::backends::{BackendTrait, mlx::MlxBackend};
use crate::dataset::TestDataset;
use crate::validators::{commands_match, validate_safety, is_posix_compliant};
```

## Definition of Done Checklist

- [ ] `run_evaluation()` successfully loads dataset and initializes MLX backend
- [ ] Test execution loop processes all test cases
- [ ] Backend timeout enforced at 30s per test case
- [ ] Backend errors captured in FailureReason::BackendError
- [ ] Validation pipeline checks correctness, safety, POSIX compliance
- [ ] TestResult Vec returned for metrics calculation
- [ ] Integration test validates single test case execution
- [ ] Error handling tested (timeout scenario)

## Activity Log

- 2026-01-09T00:00:00Z – system – shell_pid= – lane=planned – Prompt created
- 2026-01-09T10:01:51Z – claude – shell_pid=31030 – lane=doing – Started implementation
