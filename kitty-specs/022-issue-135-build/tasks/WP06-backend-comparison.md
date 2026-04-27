---
work_package_id: "WP06"
subtasks:
  - "T039"
  - "T040"
  - "T041"
  - "T042"
  - "T043"
  - "T044"
  - "T045"
  - "T046"
title: "Multi-Backend Comparison"
phase: "Phase 3 - Extended Validation"
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

# Work Package Prompt: WP06 – Multi-Backend Comparison

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

**Goal**: Implement backend comparison analysis (User Story 4) with statistical significance testing.

**Success Criteria**:
- Run same 50-prompt dataset through MLX/Ollama/vLLM
- Generate variance report across backends
- Detect statistically significant differences
- Report per-backend metrics

**Independent Test**: Execute dataset through all configured backends, aggregate results by backend, produce comparison report with statistical tests.

## Context & Constraints

**Prerequisites**: WP03 (correctness evaluator), WP04 (safety validator), WP05 (POSIX checker)

**Supporting Documents**:
- **Spec**: `kitty-specs/022-issue-135-build/spec.md` - User Story 4, FR-008, FR-009, SC-005
- **Data Model**: `kitty-specs/022-issue-135-build/data-model.md` - BackendMetrics

**Key Design Decisions**:
- Test same prompts through each configured backend
- Statistical significance: chi-square for categorical (correctness), t-test for continuous (latency)
- Skip unavailable backends with warning

## Subtasks & Detailed Guidance

### T039 – Extend executor to support backend selection via environment variable

**Purpose**: Allow specifying which backend to test.

**Steps**:
1. Read `CARO_BACKEND` environment variable
2. Pass to caro CLI: `caro --backend mlx <prompt>`

**Files**: `tests/evaluation/src/executor.rs` (modify)

### T040 – Implement backend configuration loading from caro config

**Purpose**: Discover which backends are available.

**Steps**:
1. Read caro config file to find configured backends
2. Return list of available backends for testing

**Files**: `tests/evaluation/src/executor.rs` (modify)

### T041 – Implement parallel execution across multiple backends

**Purpose**: Test all backends concurrently for speed.

**Steps**:
1. Use tokio::spawn for parallel backend execution
2. Aggregate results when all backends complete

**Files**: `tests/evaluation/src/executor.rs` (modify)

### T042 – Implement backend-specific result aggregation

**Purpose**: Group results by backend for comparison.

**Steps**:
1. Collect all EvaluationResults per backend
2. Calculate per-backend statistics (mean, std dev)

**Files**: Create `tests/evaluation/src/backend_comparison.rs`

### T043 – Implement statistical comparison

**Purpose**: Determine if backend differences are significant.

**Steps**:
1. Chi-square test for correctness rates
2. T-test for latency differences
3. Report p-values and confidence intervals

**Files**: `tests/evaluation/src/backend_comparison.rs` (modify)

### T044 – Implement variance detection and significance testing

**Purpose**: Identify when backends produce meaningfully different results.

**Steps**:
1. Calculate variance across backends
2. Flag high-variance test cases
3. Report significance levels

**Files**: `tests/evaluation/src/backend_comparison.rs` (modify)

### T045 – Create backend comparison dataset

**Purpose**: Test cases suitable for cross-backend testing.

**Files**: `tests/evaluation/datasets/backend_comparison/cross_backend.json` (create)

### T046 – Add integration test `tests/evaluation/tests/test_backends.rs`

**Purpose**: End-to-end backend comparison flow.

**Files**: `tests/evaluation/tests/test_backends.rs` (create)

## Risks & Mitigations

**Risk**: Backends not configured
**Mitigation**: Skip unavailable backends with warning

**Risk**: Statistical power low
**Mitigation**: Require minimum 30 samples per backend for significance tests

## Definition of Done Checklist

- [ ] Executor supports backend selection
- [ ] Backend configuration loading implemented
- [ ] Parallel execution across backends working
- [ ] Backend-specific result aggregation implemented
- [ ] Statistical comparison (chi-square, t-test) implemented
- [ ] Variance detection working
- [ ] Backend comparison dataset created
- [ ] Integration test runs comparison across all backends
- [ ] All tests pass
- [ ] `tasks.md` updated with WP06 completion status

## Activity Log

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created.
