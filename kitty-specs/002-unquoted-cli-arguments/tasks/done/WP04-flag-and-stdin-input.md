---
work_package_id: "WP04"
subtasks:
  - "T016"
  - "T017"
  - "T018"
  - "T019"
  - "T020"
  - "T021"
  - "T022"
title: "User Story 3 & 4 - Flag and Stdin Input"
phase: "Phase 2 - Automation"
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

# Work Package Prompt: WP04 – Flag and Stdin Input

## Objectives & Success Criteria

**Goal**: Implement `-p`/`--prompt` flag and stdin piping for automation (US3, US4).

**Success Criteria**:
- ✅ `caro -p "list files"` works (US3)
- ✅ `echo "list files" | caro` works (US4)
- ✅ Priority order respected in all combinations
- ✅ SC-005: Non-interactive mode with `-p` flag
- ✅ SC-006: Stdin input processing works

## Context

**Prerequisites**: WP03 (resolve_prompt exists)
**Spec**: User Stories 3 & 4 (P2 priority)

## Subtasks

### T016 – Add -p/--prompt flag
```rust
#[arg(short = 'p', long = "prompt")]
pub prompt_flag: Option<String>,
```
**Files**: `src/cli/mod.rs`

### T017 – Implement stdin reading
```rust
fn read_stdin() -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
```
**Files**: `src/main.rs`

### T018 – Integrate source resolution in main
```rust
let stdin_content = if is_stdin_available() {
    Some(read_stdin()?)
} else {
    None
};

let resolved = resolve_prompt(args.prompt_flag, stdin_content, args.trailing_args);
// Use resolved.text as the prompt
```
**Files**: `src/main.rs`

### T019-T022 – E2E Tests
- T019: Test `-p` flag
- T020: Test stdin piping
- T021: Test all precedence combinations
- T022: Test non-interactive mode (no confirmation when using `-p`)

**Files**: `tests/e2e_cli_tests.rs`
**Parallel**: Yes

## Definition of Done

- [x] T016-T018: Implementation complete
- [x] T019-T022: All tests passing
- [x] SC-005, SC-006 validated

## Activity Log

- 2025-12-25T02:30:00Z – system – lane=planned – Prompt created.
- 2025-12-25T12:03:57Z – claude – shell_pid=92085 – lane=doing – Starting implementation of flag and stdin input integration
- 2025-12-25T12:30:00Z – claude – shell_pid=92085 – lane=doing – Completed T016-T022: All implementation and tests complete. Fixed prompt validation to handle empty strings and updated e2e_empty_input_handling test. All 29 E2E tests passing.
- 2025-12-25T12:29:16Z – claude – shell_pid=37674 – lane=for_review – Completed all subtasks T016-T022. All 29 E2E tests passing.
- 2025-12-25T13:11:51Z – claude – shell_pid=21645 – lane=done – Acceptance review complete
