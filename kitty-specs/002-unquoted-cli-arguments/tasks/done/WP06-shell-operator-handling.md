---
work_package_id: "WP06"
subtasks:
  - "T029"
  - "T030"
  - "T031"
  - "T032"
  - "T033"
  - "T034"
title: "User Story 5 - Shell Operator Handling"
phase: "Phase 3 - Advanced"
lane: "done"
assignee: "claude"
agent: "claude"
shell_pid: "21645"
review_status: ""
reviewed_by: "claude"
history:
  - timestamp: "2025-12-25T02:30:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP06 – Shell Operator Handling

## Objectives & Success Criteria

**Goal**: Detect POSIX shell operators and truncate prompt correctly (US5, FR-007, SC-007).

**Success Criteria**:
- ✅ All 7 POSIX operators detected: `>`, `|`, `<`, `>>`, `2>`, `&`, `;`
- ✅ Truncation stops at first operator
- ✅ Embedded operators (not standalone) ignored
- ✅ SC-007: 100% accuracy on operator detection

## Context

**Contract**: `contracts/shell-operator-detection.contract.md`
**Note**: In normal usage, shell handles operators before caro sees them. This handles edge cases (quoted commands, scripts).

## Subtasks

### T029 – Implement truncate_at_shell_operator()
```rust
pub fn truncate_at_shell_operator(args: Vec<String>) -> Vec<String> {
    const SHELL_OPERATORS: &[&str] = &[">", "|", "<", ">>", "2>", "&", ";"];

    args.into_iter()
        .take_while(|arg| !SHELL_OPERATORS.contains(&arg.as_str()))
        .collect()
}
```
**Files**: `src/cli/mod.rs` or `src/main.rs`

### T030 – Define POSIX operators array
See T029 implementation (7 operators)

### T031-T034 – Tests
```rust
#[test]
fn test_all_operators() {
    for op in &[">", "|", "<", ">>", "2>", "&", ";"] {
        let args = vec!["cmd".into(), op.to_string(), "arg".into()];
        let result = truncate_at_shell_operator(args);
        assert_eq!(result, vec!["cmd"]);
    }
}

#[test]
fn test_embedded_operator_not_detected() {
    let args = vec!["find".into(), "files>output.txt".into()];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["find", "files>output.txt"]);  // Not truncated
}

#[test]
fn test_operator_first() {
    let result = truncate_at_shell_operator(vec![">".into(), "file".into()]);
    assert!(result.is_empty());
}

#[test]
fn test_multiple_operators() {
    let args = vec!["cmd".into(), ">".into(), "out".into(), "|".into(), "grep".into()];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["cmd"]);  // Stops at first '>'
}
```
**Files**: `tests/` (unit or integration)
**Parallel**: Yes (all 4 tests)

## Definition of Done

- [x] T029-T030: Implementation complete
- [x] T031-T034: All tests passing
- [x] SC-007: 100% operator detection validated

## Activity Log

- 2025-12-25T02:30:00Z – system – lane=planned – Prompt created.
- 2025-12-25T12:39:43Z – claude – shell_pid=64078 – lane=doing – Starting implementation of shell operator handling
- 2025-12-25T12:55:00Z – claude – shell_pid=64078 – lane=doing – Completed T029-T034: truncate_at_shell_operator() function, 7 POSIX operators defined, 4 unit tests, 2 E2E tests. All 12 unit tests and 31 E2E tests passing. Shell operators (>, |, <, >>, 2>, &, ;) now correctly truncate prompts (SC-007).
- 2025-12-25T12:43:03Z – claude – shell_pid=64078 – lane=for_review – Completed all subtasks T029-T034. All tests passing.
- 2025-12-25T13:11:51Z – claude – shell_pid=21645 – lane=done – Acceptance review complete
