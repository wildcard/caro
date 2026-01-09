---
work_package_id: "WP06"
subtasks: ["T044", "T045", "T046", "T047", "T048", "T049", "T050", "T051", "T052"]
title: "cargo test Integration & CLI"
phase: "Phase 3 - Developer Experience"
lane: "for_review"
agent: "claude"
shell_pid: "71447"
history:
  - timestamp: "2026-01-09T11:00:00Z"
    lane: "planned"
    agent: "system"
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP06 – cargo test Integration & CLI

## Objectives & Success Criteria

**Goal**: Integrate with cargo test for familiar CI/CD workflow.

**Success Criteria**:
- `cargo test --test evaluation` runs full evaluation
- CLI arguments: --category, --backend, --format, --baseline, --threshold
- Exit codes: 0 (success), 1 (failure/regression), 2 (config error)
- JSON and table output formats

## Context & Constraints

**Dependencies**: WP03 (harness), WP04 (baseline)
**Integration**: cargo test framework with custom test harness

## Key Subtasks

### T044 – Custom Test Harness (`tests/evaluation/main.rs`)
```rust
#[cfg(test)]
fn main() {
    // Custom test runner entry point
    // Parse CLI args, run evaluation, format output
}
```

### T045 – CLI Argument Parsing (clap)
Arguments: category, backend, format (json/table), baseline path, threshold.

### T046-T047 – Output Formats
- JSON: Full BenchmarkReport for CI pipelines
- Table: Human-readable for local development

### T048-T049 – Exit Codes & Baseline Integration
Exit 0 if pass, 1 if fail or regression detected. Load baseline, compare if provided.

### T050-T052 – Filtering & Tests
Support category/backend filters. Integration tests for CLI argument combinations.

## Test Strategy

```bash
cargo test --test evaluation
cargo test --test evaluation -- --category safety
cargo test --test evaluation -- --format json
```

## Definition of Done

- [x] cargo test integration working
- [x] All CLI arguments functional
- [x] JSON and table formats implemented
- [x] Exit codes correct
- [x] Baseline comparison integrated
- [x] CLI integration tests pass

## Activity Log

- 2026-01-09T11:00:00Z – system – lane=planned – Prompt created
- 2026-01-09T10:53:03Z – claude – shell_pid=68731 – lane=doing – Starting cargo test integration and CLI implementation
- 2026-01-09T10:58:56Z – claude – shell_pid=71447 – lane=for_review – Completed cargo test integration - all CLI features working, validated with static_matcher backend
