---
work_package_id: "WP04"
subtasks: ["T029", "T030", "T031", "T032", "T033", "T034", "T035", "T036"]
title: "Baseline Storage & Regression Detection"
phase: "Phase 2 - Quality Gate"
lane: "doing"
agent: "claude"
shell_pid: "62373"
history:
  - timestamp: "2026-01-09T11:00:00Z"
    lane: "planned"
    agent: "system"
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP04 – Baseline Storage & Regression Detection

## Objectives & Success Criteria

**Goal**: Implement baseline comparison for CI/CD regression detection.

**Success Criteria**:
- Can store/load baselines as JSON
- Compares two BenchmarkReports and detects regressions
- Configurable threshold (default 5%)
- Identifies significant regressions by category/backend

## Context & Constraints

**Dependencies**: WP01 (BenchmarkReport model), WP03 (produces reports)
**Storage**: `tests/evaluation/baselines/main-{timestamp}.json`

## Key Subtasks

### T029-T030 – Baseline Storage (`src/evaluation/baseline.rs`)
Store BenchmarkReport as pretty-printed JSON. Create symlink `main-latest.json`.

### T031-T032 – Comparison & Regression Detection
Calculate deltas: `current.pass_rate - baseline.pass_rate`
Detect regression: delta < -threshold (default -0.05 for 5% drop)

### T033 – Regression Reporting
Populate `significant_regressions` list in BaselineDelta with failing categories/backends.

### T034-T036 – Tests
Unit tests for CRUD, comparison logic, threshold detection.

## Test Strategy

```bash
cargo test --package caro --lib evaluation::baseline
```

## Definition of Done

- [x] Baseline store/load with JSON serialization
- [x] Comparison produces BaselineDelta with per-category/backend deltas
- [x] Regression detection with configurable threshold
- [x] Unit tests cover happy path and edge cases
- [x] Integration test: store → load → compare → detect

## Activity Log

- 2026-01-09T11:00:00Z – system – lane=planned – Prompt created
- 2026-01-09T10:38:55Z – claude – shell_pid=62373 – lane=doing – Starting baseline storage and regression detection
