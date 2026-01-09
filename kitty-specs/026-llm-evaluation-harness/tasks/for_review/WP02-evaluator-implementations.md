---
work_package_id: "WP02"
subtasks: ["T009", "T010", "T011", "T012", "T013", "T014", "T015", "T016", "T017", "T018"]
title: "Evaluator Trait & Implementations"
phase: "Phase 1 - Core Logic"
lane: "for_review"
agent: "claude"
shell_pid: "51796"
history:
  - timestamp: "2026-01-09T11:00:00Z"
    lane: "planned"
    agent: "system"
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP02 – Evaluator Trait & Implementations

## Objectives & Success Criteria

**Goal**: Implement Evaluator trait and all four category-specific evaluators (Correctness, Safety, POSIX, Consistency).

**Success Criteria**:
- Evaluator trait defined with async_trait support
- All four evaluators pass unit tests with mock data
- SafetyEvaluator reuses existing src/safety/ patterns
- Command equivalence and pattern matching utilities work correctly

## Context & Constraints

**Prerequisites**:
- WP01 complete (models, TestCase, EvaluationResult)
- Read contracts/evaluation-api.md for Evaluator trait specification
- SafetyEvaluator must use existing src/safety/patterns.rs for consistency

**High Parallelization**: All four evaluators can be implemented completely independently.

## Subtasks

### T009 – Define Evaluator Trait
Create `src/evaluation/mod.rs` trait:
```rust
use async_trait::async_trait;

#[async_trait]
pub trait Evaluator: Send + Sync {
    fn category(&self) -> TestCategory;
    async fn evaluate(&self, test_case: &TestCase, result: &CommandResult) -> Result<EvaluationResult>;
}
```

### T010 – Create Evaluator Utilities (`src/evaluation/utils.rs`)
- `command_equivalence(cmd1, cmd2) -> bool`: Handles "find ." vs "find . -type f"
- `matches_pattern(cmd, pattern) -> bool`: Regex matching with proper escaping
- `check_posix_compliance(cmd) -> Vec<String>`: Returns list of POSIX violations

### T011-T012 – CorrectnessEvaluator (`src/evaluation/evaluators/correctness.rs`)
Implements validation rules: ExactMatch, CommandEquivalence, PatternMatch.
Uses utils for equivalence checking. Unit tests cover all validation rules.

### T013-T014 – SafetyEvaluator (`src/evaluation/evaluators/safety.rs`)
**Critical**: Wraps `src/safety/` module for consistency with existing safety validation.
Validates MustBeBlocked and MustExecute rules. Unit tests with dangerous/safe commands.

### T015-T016 – POSIXEvaluator (`src/evaluation/evaluators/posix.rs`)
Checks for bash-specific features, GNU-only flags. Uses utils::check_posix_compliance().
Unit tests with POSIX-compliant and non-compliant commands.

### T017-T018 – ConsistencyEvaluator (`src/evaluation/evaluators/consistency.rs`)
Compares outputs from multiple backends for functional equivalence.
Requires multiple CommandResults to compare. Unit tests with matching/mismatching outputs.

## Test Strategy

**Unit Tests**: Each evaluator in separate test module.
```bash
cargo test --package caro --lib evaluation::evaluators
```

**Test Coverage**: Each validation rule, edge cases (None values, empty commands).

## Risks & Mitigations

- **Command equivalence complexity**: Start simple, iterate based on test failures
- **Safety pattern consistency**: Reuse existing patterns, don't duplicate logic
- **POSIX validation accuracy**: Document known limitations, use shellcheck as reference

## Definition of Done

- [x] Evaluator trait defined with async_trait
- [x] Utils module with command equivalence, pattern matching, POSIX checking
- [x] All four evaluators implemented with comprehensive logic
- [x] Unit tests pass for each evaluator (80%+ coverage)
- [x] SafetyEvaluator confirmed to use existing safety module
- [x] Documentation comments on all public APIs

## Activity Log

- 2026-01-09T11:00:00Z – system – lane=planned – Prompt created
- 2026-01-09T10:13:00Z – claude – shell_pid=43803 – lane=doing – Starting evaluator implementations
- 2026-01-09T10:23:38Z – claude – shell_pid=51796 – lane=for_review – Completed all 4 evaluator implementations with 52 tests passing
