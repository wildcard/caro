---
work_package_id: "WP07"
subtasks:
  - "T047"
  - "T048"
  - "T049"
  - "T050"
  - "T051"
  - "T052"
  - "T053"
  - "T054"
  - "T055"
  - "T056"
title: "Report Generation & Integration Tests"
phase: "Phase 4 - Polish"
lane: "planned"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP07 – Report Generation & Integration Tests

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

**Goal**: Implement JSON/Markdown report generation and end-to-end integration tests.

**Success Criteria**:
- Run full evaluation
- Generate timestamped report files (JSON + Markdown)
- Verify report format matches specification
- Complete end-to-end integration test

**Independent Test**: Execute complete evaluation workflow, generate reports in `results/`, validate JSON schema and Markdown formatting.

## Context & Constraints

**Prerequisites**: All previous work packages (WP03-WP06 evaluation logic)

**Supporting Documents**:
- **Spec**: `kitty-specs/022-issue-135-build/spec.md` - FR-006, FR-010, SC-007
- **Data Model**: `kitty-specs/022-issue-135-build/data-model.md` - EvaluationRun, EvaluationSummary, BackendMetrics

**Key Design Decisions**:
- JSON format follows data-model.md schema
- Markdown format: summary tables, per-category breakdowns, regressions
- Capture git commit via `git rev-parse HEAD`

## Subtasks & Detailed Guidance

### T047 – Implement `tests/evaluation/src/reporter.rs` module

**Purpose**: Provide reporting infrastructure.

**Steps**:
1. Create reporter module with Report struct
2. Define output format interfaces (JSON, Markdown)

**Files**: `tests/evaluation/src/reporter.rs` (create)

### T048 – Implement EvaluationResult struct serialization

**Purpose**: Convert internal results to JSON format.

**Steps**:
1. Use serde to serialize EvaluationResult
2. Match data-model.md schema exactly

**Files**: `tests/evaluation/src/reporter.rs` (modify)

### T049 – Implement EvaluationSummary aggregation from results

**Purpose**: Calculate summary statistics.

**Steps**:
1. Aggregate correctness scores across all test cases
2. Calculate per-category breakdowns
3. Compute safety accuracy, POSIX compliance rate

**Files**: `tests/evaluation/src/reporter.rs` (modify)

### T050 – Implement BackendMetrics calculation

**Purpose**: Per-backend statistics.

**Steps**:
1. Group results by backend_name
2. Calculate mean, p50, p95, p99 latencies
3. Compute per-backend correctness and compliance rates

**Files**: `tests/evaluation/src/reporter.rs` (modify)

### T051 – Implement JSON report generation to `tests/evaluation/results/run_<timestamp>.json`

**Purpose**: Machine-readable evaluation results.

**Steps**:
1. Create `results/` directory if missing
2. Generate filename with ISO 8601 timestamp
3. Write EvaluationRun struct as JSON

**Files**: `tests/evaluation/src/reporter.rs` (modify)

### T052 – Implement Markdown report generation with tables and formatting

**Purpose**: Human-readable evaluation results.

**Steps**:
1. Generate summary section with overall metrics
2. Create per-category breakdown tables
3. List regressions (if comparing to previous run)
4. Format with GitHub-flavored Markdown

**Files**: `tests/evaluation/src/reporter.rs` (modify)

### T053 – Add performance metrics capture (latency, memory usage)

**Purpose**: Track evaluation performance.

**Steps**:
1. Measure inference latency per test case
2. Capture total evaluation duration
3. Optional: memory usage tracking

**Files**: `tests/evaluation/src/reporter.rs` (modify)

### T054 – Add timestamp and git commit tracking to reports

**Purpose**: Associate results with codebase version.

**Steps**:
1. Capture run timestamp (UTC)
2. Run `git rev-parse HEAD` to get commit hash
3. Include in report metadata

**Files**: `tests/evaluation/src/reporter.rs` (modify)

### T055 – Create full end-to-end integration test

**Purpose**: Verify complete evaluation workflow.

**Steps**:
1. Load all datasets
2. Execute evaluations (correctness, safety, POSIX)
3. Generate reports
4. Validate report contents

**Files**: `tests/evaluation/tests/test_full_evaluation.rs` (create)

### T056 – Add CLI wrapper script for easy evaluation execution

**Purpose**: User-friendly interface for running evaluations.

**Steps**:
1. Create `tests/evaluation/bin/evaluate.rs`
2. Add CLI arguments: dataset selection, backend selection, output path
3. Invoke evaluation and report generation

**Files**: `tests/evaluation/bin/evaluate.rs` (create)

**Parallel?**: Yes

## Risks & Mitigations

**Risk**: Report file size too large
**Mitigation**: Limit detailed output, use summary aggregations

**Risk**: Markdown formatting breaks in different viewers
**Mitigation**: Use GitHub-flavored markdown standard

## Definition of Done Checklist

- [ ] Reporter module implemented
- [ ] EvaluationResult serialization working
- [ ] EvaluationSummary aggregation implemented
- [ ] BackendMetrics calculation implemented
- [ ] JSON report generation to timestamped file
- [ ] Markdown report generation with tables
- [ ] Performance metrics capture (latency)
- [ ] Timestamp and git commit tracking
- [ ] Full end-to-end integration test passing
- [ ] CLI wrapper script created
- [ ] All tests pass
- [ ] Example reports generated and validated
- [ ] `tasks.md` updated with WP07 completion status

## Activity Log

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created.
