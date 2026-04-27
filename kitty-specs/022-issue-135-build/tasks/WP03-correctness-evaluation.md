---
work_package_id: "WP03"
subtasks:
  - "T013"
  - "T014"
  - "T015"
  - "T016"
  - "T017"
  - "T018"
  - "T019"
  - "T020"
  - "T021"
title: "Command Correctness Evaluation"
phase: "Phase 2 - MVP Evaluation"
lane: "for_review"
assignee: ""
agent: "claude"
shell_pid: "12556"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP03 – Command Correctness Evaluation (🎯 MVP)

## ⚠️ IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately (right below this notice).
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Mark as acknowledged**: When you understand the feedback and begin addressing it, update `review_status: acknowledged` in the frontmatter.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** – Reviewers add detailed feedback here when work needs changes. Implementation must address every item listed below before returning for re-review.

*[This section is empty initially. Reviewers will populate it if the work is returned from review. If you see feedback here, treat each item as a must-do before completion.]*

---

## Objectives & Success Criteria

**Goal**: Implement command correctness validation (User Story 1) with exact match and semantic equivalence.

**Success Criteria**:
- Run 20-prompt correctness dataset
- Generate accuracy report showing match rate
- Support exact match and semantic equivalence
- Produce diffs for mismatches
- Complete evaluation in < 5 minutes (SC-001)

**Independent Test**: Load correctness dataset, run through caro CLI, generate report with accuracy percentage and diff for each mismatch.

## Context & Constraints

**Prerequisites**: WP02 (dataset loader exists)

**Supporting Documents**:
- **Spec**: `kitty-specs/022-issue-135-build/spec.md` - User Story 1, FR-001, FR-003, SC-001
- **Plan**: `kitty-specs/022-issue-135-build/plan.md` - CLI invocation strategy (black-box via std::process::Command)
- **Data Model**: `kitty-specs/022-issue-135-build/data-model.md` - EvaluationResult, PerformanceMetrics

**Key Design Decisions** (from plan.md):
- **CLI Invocation**: Use `std::process::Command` to invoke caro as black-box
- **Semantic Equivalence**: Start simple (whitespace normalization, flag ordering), expand iteratively
- **Performance**: Tokio async with parallel test execution to meet < 5 min target

**Constraints**:
- Must use existing caro CLI (no direct backend API calls)
- Timeout: 30s per command
- Semantic equivalence rules start conservative

## Subtasks & Detailed Guidance

### T013 – Implement `tests/evaluation/src/executor.rs` for CLI invocation

**Purpose**: Provide interface for executing caro CLI and capturing results.

**Steps**:
1. Create `tests/evaluation/src/executor.rs`
2. Define Executor struct:
   ```rust
   use std::process::Command;
   use std::time::Duration;

   pub struct Executor {
       caro_binary_path: std::path::PathBuf,
       timeout: Duration,
   }

   impl Executor {
       pub fn new() -> Result<Self, ExecutorError> {
           let binary_path = std::env::current_exe()?
               .parent()
               .ok_or_else(|| ExecutorError::BinaryNotFound)?
               .join("../../target/release/caro");

           Ok(Self {
               caro_binary_path: binary_path,
               timeout: Duration::from_secs(30),
           })
       }

       pub async fn execute(&self, prompt: &str) -> Result<String, ExecutorError> {
           // Implementation in T014
           todo!()
       }
   }
   ```
3. Add error type:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum ExecutorError {
       #[error("Caro binary not found at expected location")]
       BinaryNotFound,
       #[error("Command execution failed: {0}")]
       ExecutionFailed(String),
       #[error("Command timeout after {0:?}")]
       Timeout(Duration),
       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
   }
   ```

**Files**:
- `tests/evaluation/src/executor.rs` (create)

**Parallel?**: No (foundation for evaluation)

**Notes**: Binary path assumes standard cargo build layout. Timeout prevents hung evaluations.

### T014 – Implement command output capture and error handling

**Purpose**: Execute caro CLI subprocess and capture stdout/stderr reliably.

**Steps**:
1. Implement `execute` method in `executor.rs`:
   ```rust
   pub async fn execute(&self, prompt: &str) -> Result<String, ExecutorError> {
       let output = tokio::process::Command::new(&self.caro_binary_path)
           .arg(prompt)
           .output()
           .await?;

       if !output.status.success() {
           return Err(ExecutorError::ExecutionFailed(
               String::from_utf8_lossy(&output.stderr).to_string()
           ));
       }

       let command = String::from_utf8_lossy(&output.stdout)
           .trim()
           .to_string();

       Ok(command)
   }
   ```
2. Add timeout support using `tokio::time::timeout`
3. Handle edge cases:
   - Empty output
   - Non-UTF8 output
   - Signal termination

**Files**:
- `tests/evaluation/src/executor.rs` (modify)

**Parallel?**: No (depends on T013)

### T015 – Implement `tests/evaluation/src/evaluator.rs` with exact match logic

**Purpose**: Compare generated commands against expected commands.

**Steps**:
1. Create `tests/evaluation/src/evaluator.rs`
2. Define comparison logic:
   ```rust
   use crate::dataset::TestCase;

   pub struct Evaluator;

   #[derive(Debug, Clone)]
   pub struct CorrectnessResult {
       pub score: f64,
       pub method: CorrectnessMethod,
       pub diff: Option<String>,
   }

   #[derive(Debug, Clone, PartialEq)]
   pub enum CorrectnessMethod {
       ExactMatch,
       SemanticEquivalent,
       NoMatch,
   }

   impl Evaluator {
       pub fn evaluate_correctness(
           &self,
           generated: &str,
           expected: &str,
       ) -> CorrectnessResult {
           // Exact match first
           if generated == expected {
               return CorrectnessResult {
                   score: 1.0,
                   method: CorrectnessMethod::ExactMatch,
                   diff: None,
               };
           }

           // Semantic equivalence in T016
           todo!()
       }
   }
   ```

**Files**:
- `tests/evaluation/src/evaluator.rs` (create)

**Parallel?**: Yes (can proceed alongside T013-T014 if types are committed)

### T016 – Implement semantic equivalence rules

**Purpose**: Handle command variations that are functionally equivalent.

**Steps**:
1. Add normalization functions:
   ```rust
   fn normalize_whitespace(cmd: &str) -> String {
       cmd.split_whitespace().collect::<Vec<_>>().join(" ")
   }

   fn normalize_flags(cmd: &str) -> String {
       // Parse command into base + flags
       // Sort flags alphabetically
       // Reconstruct command
       // Example: "ls -la" == "ls -al"
       todo!()
   }
   ```
2. Update `evaluate_correctness`:
   ```rust
   // After exact match check
   let gen_normalized = normalize_whitespace(generated);
   let exp_normalized = normalize_whitespace(expected);

   if gen_normalized == exp_normalized {
       return CorrectnessResult {
           score: 0.95,
           method: CorrectnessMethod::SemanticEquivalent,
           diff: None,
       };
   }

   // Try flag normalization
   let gen_flags = normalize_flags(&gen_normalized);
   let exp_flags = normalize_flags(&exp_normalized);

   if gen_flags == exp_flags {
       return CorrectnessResult {
           score: 0.90,
           method: CorrectnessMethod::SemanticEquivalent,
           diff: Some(format!("Flag order differs: {} vs {}", generated, expected)),
       };
   }

   // No match
   CorrectnessResult {
       score: 0.0,
       method: CorrectnessMethod::NoMatch,
       diff: Some(self.generate_diff(generated, expected)),
   }
   ```

**Files**:
- `tests/evaluation/src/evaluator.rs` (modify)

**Parallel?**: No (depends on T015)

**Notes**: Start with simple rules. Flag normalization is complex; use heuristics initially.

### T017 – Implement correctness scoring (0.0-1.0)

**Purpose**: Provide quantitative measure of command correctness.

**Steps**:
1. Define scoring policy in evaluator:
   - Exact match: 1.0
   - Whitespace normalized: 0.95
   - Flag order normalized: 0.90
   - Partial match (future): 0.5-0.8
   - No match: 0.0
2. Add score aggregation:
   ```rust
   impl Evaluator {
       pub fn aggregate_scores(results: &[CorrectnessResult]) -> f64 {
           if results.is_empty() {
               return 0.0;
           }
           results.iter().map(|r| r.score).sum::<f64>() / results.len() as f64
       }
   }
   ```

**Files**:
- `tests/evaluation/src/evaluator.rs` (modify)

**Parallel?**: Yes (can proceed alongside T016 implementation)

### T018 – Implement diff generation using `similar` crate

**Purpose**: Show clear comparison for failed matches.

**Steps**:
1. Add similar crate usage:
   ```rust
   use similar::{ChangeTag, TextDiff};

   impl Evaluator {
       fn generate_diff(&self, generated: &str, expected: &str) -> String {
           let diff = TextDiff::from_lines(expected, generated);

           let mut output = String::new();
           for change in diff.iter_all_changes() {
               let sign = match change.tag() {
                   ChangeTag::Delete => "-",
                   ChangeTag::Insert => "+",
                   ChangeTag::Equal => " ",
               };
               output.push_str(&format!("{}{}", sign, change));
           }
           output
       }
   }
   ```
2. Format diff for readability:
   ```
   Expected: ls -la /tmp
   Generated: ls -al /tmp

   Diff:
   -ls -la /tmp
   +ls -al /tmp
   ```

**Files**:
- `tests/evaluation/src/evaluator.rs` (modify)

**Parallel?**: Yes (can proceed alongside T016-T017)

### T019 – Create correctness test datasets

**Purpose**: Provide comprehensive test coverage for file operations, text processing, and network commands.

**Steps**:
1. Create `datasets/correctness/file_operations.json` (if not already from WP02)
2. Create `datasets/correctness/text_processing.json`:
   - grep patterns
   - sed transformations
   - awk processing
   - cut/sort/uniq pipelines
3. Create `datasets/correctness/network_commands.json`:
   - curl/wget downloads
   - ping/traceroute diagnostics
   - ssh connections
   - netstat/ss queries

**Files**:
- `tests/evaluation/datasets/correctness/file_operations.json` (verify exists)
- `tests/evaluation/datasets/correctness/text_processing.json` (create)
- `tests/evaluation/datasets/correctness/network_commands.json` (create)

**Parallel?**: Yes (can proceed independently of implementation)

**Notes**: Each dataset should have 10-20 test cases. Mix basic and complex examples. Ensure risk_level and posix_compliant fields are accurate.

### T020 – Add unit tests for equivalence rules

**Purpose**: Verify normalization logic works correctly.

**Steps**:
1. Add test module to `evaluator.rs`:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_exact_match() {
           let eval = Evaluator;
           let result = eval.evaluate_correctness("ls", "ls");
           assert_eq!(result.score, 1.0);
           assert_eq!(result.method, CorrectnessMethod::ExactMatch);
       }

       #[test]
       fn test_whitespace_normalization() {
           let eval = Evaluator;
           let result = eval.evaluate_correctness("ls  -la", "ls -la");
           assert_eq!(result.score, 0.95);
           assert_eq!(result.method, CorrectnessMethod::SemanticEquivalent);
       }

       #[test]
       fn test_flag_order() {
           let eval = Evaluator;
           let result = eval.evaluate_correctness("ls -la", "ls -al");
           assert!(result.score >= 0.90);
       }

       #[test]
       fn test_no_match() {
           let eval = Evaluator;
           let result = eval.evaluate_correctness("ls", "pwd");
           assert_eq!(result.score, 0.0);
           assert_eq!(result.method, CorrectnessMethod::NoMatch);
           assert!(result.diff.is_some());
       }
   }
   ```

**Files**:
- `tests/evaluation/src/evaluator.rs` (modify)

**Parallel?**: Yes (alongside T016-T018 implementation)

### T021 – Add integration test `tests/evaluation/tests/test_correctness.rs`

**Purpose**: Verify end-to-end correctness evaluation flow.

**Steps**:
1. Create `tests/evaluation/tests/test_correctness.rs`:
   ```rust
   use caro_evaluation::dataset::TestDataset;
   use caro_evaluation::executor::Executor;
   use caro_evaluation::evaluator::Evaluator;

   #[tokio::test]
   async fn test_evaluate_file_operations_dataset() {
       let dataset = TestDataset::load_from_file(
           "datasets/correctness/file_operations.json".as_ref()
       ).expect("Failed to load dataset");

       let executor = Executor::new().expect("Failed to create executor");
       let evaluator = Evaluator;

       let mut results = Vec::new();

       for test_case in &dataset.test_cases {
           let generated = executor.execute(&test_case.prompt).await
               .expect("Execution failed");

           let correctness = evaluator.evaluate_correctness(
               &generated,
               &test_case.expected_command
           );

           results.push((test_case.id.clone(), correctness));
       }

       let avg_score = results.iter()
           .map(|(_, r)| r.score)
           .sum::<f64>() / results.len() as f64;

       println!("Average correctness: {:.2}%", avg_score * 100.0);

       // Report failures
       for (id, result) in &results {
           if result.score < 1.0 {
               println!("Test {} scored {:.2}", id, result.score);
               if let Some(diff) = &result.diff {
                   println!("{}", diff);
               }
           }
       }

       assert!(avg_score >= 0.70, "Correctness below 70%");
   }
   ```
2. Run: `cargo test --test test_correctness -- --nocapture`

**Files**:
- `tests/evaluation/tests/test_correctness.rs` (create)

**Parallel?**: No (requires T013-T018 complete)

## Risks & Mitigations

**Risk**: CLI invocation timeout for slow backends
**Mitigation**: Set 30s timeout per command; log slow commands for investigation

**Risk**: Semantic equivalence too broad, accepting incorrect commands
**Mitigation**: Start conservative with whitespace and flag ordering only; add rules iteratively with test coverage

**Risk**: Dataset quality issues (incorrect expected commands)
**Mitigation**: Manually review all test cases; run against known-good reference implementation

**Risk**: Performance target not met (> 5 minutes for 100 prompts)
**Mitigation**: Use tokio parallel execution; profile if needed; optimize hot paths

## Definition of Done Checklist

- [ ] Executor implements CLI invocation with timeout
- [ ] Command output capture handles stdout/stderr correctly
- [ ] Evaluator implements exact match logic
- [ ] Semantic equivalence handles whitespace and flag order
- [ ] Correctness scoring (0.0-1.0) implemented
- [ ] Diff generation uses `similar` crate
- [ ] Three correctness datasets created (file_operations, text_processing, network)
- [ ] Unit tests verify equivalence rules
- [ ] Integration test runs full evaluation and reports accuracy
- [ ] All tests pass: `cargo test --package caro-evaluation`
- [ ] Evaluation completes in < 5 minutes for 20-prompt dataset
- [ ] `tasks.md` updated with WP03 completion status

## Review Guidance

**Key Acceptance Checkpoints**:
1. Verify CLI invocation works with actual caro binary
2. Confirm semantic equivalence rules are conservative (not too permissive)
3. Check diff output is readable and actionable
4. Validate datasets cover diverse command types
5. Ensure performance meets < 5 min target
6. Test error handling (binary not found, timeout, invalid output)

**Context for Reviewers**:
- This is MVP critical path (User Story 1, Priority P1)
- Semantic equivalence will expand iteratively
- Dataset quality is more important than quantity initially

## Activity Log

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created.
- 2026-01-08T23:06:34Z – claude – shell_pid=5881 – lane=doing – Started WP03: Command Correctness Evaluation - MVP
- 2026-01-08T23:14:37Z – claude – shell_pid=12556 – lane=for_review – Implementation complete - all tests passing
