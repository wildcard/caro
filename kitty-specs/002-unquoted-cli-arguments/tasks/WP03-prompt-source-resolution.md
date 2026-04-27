---
work_package_id: "WP03"
subtasks:
  - "T012"
  - "T013"
  - "T014"
  - "T015"
title: "Prompt Source Resolution"
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

# Work Package Prompt: WP03 – Prompt Source Resolution

## Objectives & Success Criteria

**Goal**: Implement input prioritization (flag > stdin > trailing args) with clean Library-First separation.

**Success Criteria**:
- ✅ PromptSource enum defined (Flag, Stdin, TrailingArgs)
- ✅ `resolve_prompt()` function implements FR-008 priority
- ✅ stdin availability check works correctly
- ✅ Unit tests validate all priority combinations

## Context

**Prerequisites**:
- WP01 (CliArgs structure exists)
- Contract: `contracts/prompt-source-resolution.contract.md`
- Constitution: Principle II (Library-First Architecture)

**Priority Order**: `-p` flag > stdin > trailing args

## Subtasks

### T012 – Create PromptSource enum
```rust
pub enum PromptSource {
    Flag,           // From -p/--prompt
    Stdin,          // From piped stdin
    TrailingArgs,   // From command-line words
}

pub struct ResolvedPrompt {
    pub text: String,
    pub source: PromptSource,
}
```
**Files**: `src/main.rs`

### T013 – Implement resolve_prompt() function
```rust
fn resolve_prompt(flag: Option<String>, stdin: Option<String>, trailing_args: Vec<String>) -> ResolvedPrompt {
    if let Some(text) = flag {
        ResolvedPrompt { text, source: PromptSource::Flag }
    } else if let Some(text) = stdin {
        ResolvedPrompt { text, source: PromptSource::Stdin }
    } else {
        ResolvedPrompt { text: trailing_args.join(" "), source: PromptSource::TrailingArgs }
    }
}
```
**Files**: `src/main.rs`
**Contract**: Rules 2-5

### T014 – Add stdin availability check
```rust
fn is_stdin_available() -> bool {
    use std::io::IsTerminal;
    !std::io::stdin().is_terminal()
}
```
**Files**: `src/main.rs`
**Contract**: Rule 7

### T015 – Write unit tests
```rust
#[test]
fn test_flag_overrides_all() {
    let resolved = resolve_prompt(Some("flag".into()), Some("stdin".into()), vec!["trailing".into()]);
    assert_eq!(resolved.text, "flag");
    assert!(matches!(resolved.source, PromptSource::Flag));
}

#[test]
fn test_stdin_overrides_trailing() {
    let resolved = resolve_prompt(None, Some("stdin".into()), vec!["trailing".into()]);
    assert_eq!(resolved.text, "stdin");
    assert!(matches!(resolved.source, PromptSource::Stdin));
}

#[test]
fn test_trailing_args_default() {
    let resolved = resolve_prompt(None, None, vec!["list".into(), "files".into()]);
    assert_eq!(resolved.text, "list files");
    assert!(matches!(resolved.source, PromptSource::TrailingArgs));
}
```
**Files**: `src/main.rs` (inline tests) or `tests/unit/`
**Parallel**: Yes

## Definition of Done

- [x] T012-T014: All functions implemented
- [x] T015: All 3 unit tests passing
- [x] Code is pure functions (testable without IO)
- [x] Contract Rules 1-7 satisfied

## Review Guidance

Verify: Contract `contracts/prompt-source-resolution.contract.md` all rules pass

## Activity Log

- 2025-12-25T02:30:00Z – system – lane=planned – Prompt created.
- 2025-12-25T11:04:44Z – claude – shell_pid=92085 – lane=doing – Starting implementation of prompt source resolution
- 2025-12-25T11:20:00Z – claude – shell_pid=92085 – lane=doing – Completed implementation: T012-T015 all done, 4 unit tests passing, pure functions with Library-First architecture
- 2025-12-25T11:07:09Z – claude – shell_pid=92085 – lane=for_review – Ready for review: Prompt source resolution complete with pure functions
- 2025-12-25T13:11:51Z – claude – shell_pid=21645 – lane=done – Acceptance review complete
