---
work_package_id: "WP05"
subtasks:
  - "T023"
  - "T024"
  - "T025"
  - "T026"
  - "T027"
  - "T028"
title: "Validation & Help Display"
phase: "Phase 0 - Foundation"
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

# Work Package Prompt: WP05 – Validation & Help Display

## Objectives & Success Criteria

**Goal**: Validate prompts and show help for empty/whitespace input (FR-004, SC-004).

**Success Criteria**:
- ✅ `caro` (no args) shows help, not error
- ✅ `caro   ` (whitespace) shows help
- ✅ Special characters preserved (FR-010)
- ✅ SC-004: Help message displays correctly

## Context

**Prerequisites**: WP03 (resolved prompt available)
**Contract**: `contracts/validation.contract.md`

## Subtasks

### T023 – Implement validate_prompt()
```rust
pub enum ValidationAction {
    ShowHelp,
    ProceedWithPrompt,
}

pub fn validate_prompt(prompt: &str) -> ValidationAction {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        ValidationAction::ShowHelp
    } else {
        ValidationAction::ProceedWithPrompt
    }
}
```
**Files**: `src/main.rs`

### T024 – Handle empty string
Use `validate_prompt("")` → `ShowHelp`

### T025 – Handle whitespace-only
Use `validate_prompt("   ")` → `ShowHelp`

### T026 – Preserve special characters
Test: `validate_prompt("find *.txt")` → `ProceedWithPrompt` (unchanged)

### T027 – Write validation tests
```rust
#[test]
fn test_empty_shows_help() {
    assert!(matches!(validate_prompt(""), ValidationAction::ShowHelp));
}

#[test]
fn test_whitespace_shows_help() {
    assert!(matches!(validate_prompt("   "), ValidationAction::ShowHelp));
}

#[test]
fn test_valid_prompt_proceeds() {
    assert!(matches!(validate_prompt("list files"), ValidationAction::ProceedWithPrompt));
}
```
**Files**: `tests/execution_prompt_behavior.rs`
**Parallel**: Yes

### T028 – Integrate help display
```rust
match validate_prompt(&resolved.text) {
    ValidationAction::ShowHelp => {
        print_help();  // Existing caro help function
        return Ok(());
    }
    ValidationAction::ProceedWithPrompt => {
        // Continue to inference
    }
}
```
**Files**: `src/main.rs`

## Definition of Done

- [x] T023-T026: Implementation complete
- [x] T027: All validation tests passing
- [x] T028: Help integration working
- [x] SC-004 validated

## Activity Log

- 2025-12-25T02:30:00Z – system – lane=planned – Prompt created.
- 2025-12-25T12:33:56Z – claude – shell_pid=50833 – lane=doing – Starting implementation of validation and help display
- 2025-12-25T12:45:00Z – claude – shell_pid=50833 – lane=doing – Completed T023-T028: ValidationAction enum, validate_prompt() function, help integration, 4 unit tests, and 2 E2E tests updated. All 8 unit tests and 29 E2E tests passing. Empty/whitespace input now displays help (SC-004).
- 2025-12-25T12:37:50Z – claude – shell_pid=50833 – lane=for_review – Completed all subtasks T023-T028. All tests passing.
- 2025-12-25T13:11:51Z – claude – shell_pid=21645 – lane=done – Acceptance review complete
