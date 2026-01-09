---
work_package_id: "WP03"
subtasks: ["T019", "T020", "T021", "T022", "T023", "T024", "T025", "T026", "T027", "T028"]
title: "Harness Orchestration & Parallel Execution"
phase: "Phase 2 - Integration"
lane: "planned"
history:
  - timestamp: "2026-01-09T11:00:00Z"
    lane: "planned"
    agent: "system"
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP03 – Harness Orchestration & Parallel Execution

## Objectives & Success Criteria

**Goal**: Implement EvaluationHarness that orchestrates parallel backend execution and aggregates results.

**Success Criteria**:
- Full evaluation completes in <5 minutes with 100 tests × 4 backends
- Handles backend failures gracefully (timeouts, crashes, unavailability)
- Produces valid BenchmarkReport with aggregated results
- Platform detection skips unavailable backends (e.g., MLX on Linux)

## Context & Constraints

**Dependencies**: WP01 (models, dataset), WP02 (evaluators)
**Performance Target**: <5 min with parallel execution via tokio

## Key Subtasks

### T019-T020 – EvaluationHarness Struct (`src/evaluation/harness.rs`)
```rust
pub struct EvaluationHarness {
    dataset: TestDataset,
    backends: Vec<Box<dyn Backend>>,
    evaluators: HashMap<TestCategory, Box<dyn Evaluator>>,
    config: HarnessConfig,
}
```
Initialize with dataset, backends from existing caro modules, evaluators from WP02.

### T021 – Parallel Backend Execution
Strategy: Outer loop over test cases, inner tokio::spawn for each backend.
Timeout: 30s per backend using tokio::time::timeout.
Collect results as they complete, don't block on failures.

### T022-T023 – Filtering (per-category, per-backend)
`run_category(category)` and `run_backend(backend_name)` methods for selective evaluation.

### T024 – Result Aggregation
Collect all EvaluationResults → calculate pass rates → build BenchmarkReport.
Category breakdown, backend breakdown, overall metrics.

### T025-T026 – Failure Handling & Platform Detection
- Backend timeout → record as failure, continue
- Backend crash → log error, continue with others
- Platform detection: Check uname for MLX (requires macOS arm64)

### T027-T028 – Integration Tests & Performance Validation
Mock backends for fast integration tests. Real performance test validates <5min target.

## Test Strategy

```bash
cargo test --package caro --test evaluation_tests
```

Integration tests with mock backends verify orchestration logic without real inference calls.

## Definition of Done

- [x] Harness orchestrates parallel evaluation across all backends
- [x] Performance validated: <5 min for 100 tests × 4 backends
- [x] Backend failures handled gracefully
- [x] Integration tests pass with mock backends
- [x] Platform detection works (MLX macOS-only)

## Activity Log

- 2026-01-09T11:00:00Z – system – lane=planned – Prompt created
