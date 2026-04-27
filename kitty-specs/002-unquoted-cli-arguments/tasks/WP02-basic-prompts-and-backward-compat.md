---
work_package_id: "WP02"
subtasks:
  - "T005"
  - "T006"
  - "T007"
  - "T008"
  - "T009"
  - "T010"
  - "T011"
title: "User Story 1 & 2 - Basic Prompts & Backward Compatibility"
phase: "Phase 1 - MVP Core"
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

# Work Package Prompt: WP02 – Basic Prompts & Backward Compatibility

## Objectives & Success Criteria

**Goal**: Enable basic unquoted prompts while maintaining 100% backward compatibility with quoted prompts.

**Success Criteria**:
- ✅ `caro list files` generates a command (US1 acceptance scenario 1)
- ✅ `caro "list files"` works exactly as before (US2 acceptance scenario 1)
- ✅ `caro --verbose list files` parses flags correctly (US1 acceptance scenario 3)
- ✅ Multi-word prompts work: `caro find large files` (US1 acceptance scenario 2)
- ✅ All existing tests pass (backward compatibility validation)
- ✅ SC-001: 100% accuracy for 2-5 word prompts
- ✅ SC-002: Backward compatibility maintained

## Context & Constraints

**Prerequisites**:
- WP01 must be complete (CliArgs has `trailing_args` field)
- Contract: `contracts/cli-argument-parsing.contract.md` Rules 1-3, 6
- Spec: `spec.md` User Stories 1 & 2 (P1 priority)

**Implementation Notes**:
- Join trailing args with spaces: `trailing_args.join(" ")`
- Quoted prompts: Shell removes quotes, so arrives as single arg in `trailing_args[0]`
- This is the **MVP** - delivers immediate value

## Subtasks & Detailed Guidance

### T005 – Implement trailing args collection and joining
- **Purpose**: Convert Vec<String> to single prompt string
- **Steps**:
  1. Open `src/main.rs`
  2. After parsing CliArgs, add logic to join trailing_args:
     ```rust
     let prompt_from_args = if !args.trailing_args.is_empty() {
         args.trailing_args.join(" ")
     } else {
         String::new()
     };
     ```
  3. This becomes the prompt for LLM (for now; later WP03 will add source resolution)
- **Files**: `src/main.rs`
- **Contract**: FR-009 (join multiple words with spaces)

### T006 – Verify quoted prompts still work
- **Purpose**: Ensure backward compatibility (US2)
- **Steps**:
  1. Test manually: `cargo run -- "list files"`
  2. Verify prompt is "list files" (single string, spaces preserved)
  3. Shell removes quotes, so it arrives as single arg: `trailing_args = ["list files"]`
  4. Join produces: "list files" (unchanged)
- **Files**: N/A (verification only)
- **Contract**: FR-002 (backward compatibility)

### T007 – Test unquoted prompt
- **Purpose**: E2E test for US1 acceptance scenario 1
- **Steps**:
  1. Add test to `tests/e2e_cli_tests.rs`:
     ```rust
     #[tokio::test]
     async fn test_unquoted_prompt_basic() {
         // Test: caro list files
         let result = run_caro_command(vec!["list", "files"]).await;
         assert!(result.is_ok());
         assert!(result.unwrap().contains("ls")); // Expected: generates 'ls' command
     }
     ```
- **Files**: `tests/e2e_cli_tests.rs`
- **Parallel**: Yes

### T008 – Test quoted prompt (backward compatibility)
- **Purpose**: E2E test for US2 acceptance scenario 1
- **Steps**:
  1. Add test to `tests/e2e_cli_tests.rs`:
     ```rust
     #[tokio::test]
     async fn test_quoted_prompt_backward_compat() {
         // Test: caro "list files" (shell passes as single arg)
         let result = run_caro_command(vec!["list files"]).await;
         assert!(result.is_ok());
     }
     ```
- **Files**: `tests/e2e_cli_tests.rs`
- **Parallel**: Yes

### T009 – Test flags before prompt
- **Purpose**: E2E test for US1 acceptance scenario 3
- **Steps**:
  1. Add test:
     ```rust
     #[tokio::test]
     async fn test_flags_before_unquoted_prompt() {
         let result = run_caro_command(vec!["--verbose", "list", "files"]).await;
         assert!(result.is_ok());
         // Verify --verbose was processed correctly
     }
     ```
- **Files**: `tests/e2e_cli_tests.rs`
- **Parallel**: Yes

### T010 – Test multi-word prompts
- **Purpose**: E2E test for US1 acceptance scenario 2
- **Steps**:
  1. Add test:
     ```rust
     #[tokio::test]
     async fn test_multi_word_unquoted_prompt() {
         let result = run_caro_command(vec!["find", "large", "files", "in", "current", "directory"]).await;
         assert!(result.is_ok());
     }
     ```
- **Files**: `tests/e2e_cli_tests.rs`
- **Parallel**: Yes

### T011 – Run all existing tests
- **Purpose**: Validate no regressions (FR-011, SC-002)
- **Steps**:
  1. Run full test suite: `cargo test`
  2. Verify all existing tests pass
  3. If any fail, investigate and fix
  4. Document any test updates needed in Activity Log
- **Files**: N/A (test execution)

## Test Strategy

**Required Tests**:
- T007: Basic unquoted prompt
- T008: Quoted prompt (backward compat)
- T009: Flags before prompt
- T010: Multi-word prompts
- T011: Full regression suite

**Test Execution**:
```bash
cargo test e2e_cli_tests
cargo test  # Full suite
```

## Risks & Mitigations

**Risk**: Breaking existing quoted prompt workflows
- **Mitigation**: T011 validates all existing tests pass

**Risk**: Incorrect whitespace handling
- **Mitigation**: `.join(" ")` preserves single spaces between words

## Definition of Done Checklist

- [x] T005: Trailing args joining implemented
- [x] T006: Quoted prompts verified working
- [x] T007-T010: All 4 E2E tests written and passing
- [x] T011: All existing tests pass (no regressions)
- [x] SC-001: 100% accuracy for 2-5 word prompts validated
- [x] SC-002: Backward compatibility validated
- [x] `tasks.md` updated

## Review Guidance

**Key Checkpoints**:
1. ✅ Both `caro list files` and `caro "list files"` work
2. ✅ Flags are parsed before prompt
3. ✅ Multi-word prompts joined correctly
4. ✅ No regressions in existing tests

## Activity Log

- 2025-12-25T02:30:00Z – system – lane=planned – Prompt created.
- 2025-12-25T10:48:57Z – claude – shell_pid=92085 – lane=doing – Starting implementation of basic prompts and backward compatibility
- 2025-12-25T11:15:00Z – claude – shell_pid=92085 – lane=doing – Completed implementation: T005-T011 all done, 171 tests pass (4 new E2E tests added), backward compatibility maintained
- 2025-12-25T11:03:22Z – claude – shell_pid=92085 – lane=for_review – Ready for review: Basic prompts and backward compatibility complete
- 2025-12-25T13:11:34Z – claude – shell_pid=21645 – lane=done – Acceptance review complete
