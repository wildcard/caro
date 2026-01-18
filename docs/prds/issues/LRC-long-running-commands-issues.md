# GitHub Issues: Long-Running Command UX

**Feature**: Long-Running Command UX Improvements (v1.2.0)
**PRD**: PRD-001
**ADR**: ADR-011
**Created**: January 18, 2026

---

## Issue Summary

This document defines GitHub issues for the Long-Running Command UX feature. Issues are organized by priority and dependency order.

To create these issues, use:
```bash
gh issue create --title "..." --body "..." --label "..." --milestone "v1.2.0"
```

---

## Epic Issue

### LRC-000: Long-Running Command UX Improvements (Epic)

**Labels**: `enhancement`, `epic`, `ux`
**Milestone**: v1.2.0

```markdown
## Summary
Implement comprehensive improvements to how caro handles long-running commands, addressing user feedback about commands appearing "stuck" or unresponsive.

## Problem Statement
Users experience a "stuck" or "unresponsive" feeling when caro executes commands that take significant time to complete (e.g., `find . -type f -size +100M` from home directory). No output appears, no progress is shown, and users cannot tell if the command is running or hung.

## Solution Overview
Three interconnected improvements:
1. **Streaming Output**: Real-time display of command output
2. **Intelligent Timeout**: Preemptive timeout with partial results
3. **Smart Scope**: Better command inference with appropriate scope

## Success Criteria
- [ ] Commands show activity within 100ms of execution
- [ ] Timeout cleanly terminates commands with partial output
- [ ] Users understand when to use Edit option for long-running tasks
- [ ] Zero "stuck" perception reports from beta testers

## Related Documents
- PRD: docs/prds/PRD-001-long-running-command-ux.md
- ADR: docs/adr/ADR-011-streaming-execution-architecture.md

## Child Issues
- [ ] #LRC-001: Implement StreamingExecutor with Tokio
- [ ] #LRC-002: Add real-time output display
- [ ] #LRC-003: Implement Ctrl+C cancellation
- [ ] #LRC-004: Add elapsed time and activity indicator
- [ ] #LRC-005: Implement preemptive timeout
- [ ] #LRC-006: Display partial results on timeout
- [ ] #LRC-007: Add educational timeout messaging
- [ ] #LRC-008: Detect broad-scope commands
```

---

## Streaming Execution Issues

### LRC-001: Implement StreamingExecutor with Tokio

**Labels**: `enhancement`, `core`, `execution`
**Milestone**: v1.2.0
**Priority**: Critical
**Estimate**: Large

```markdown
## Summary
Create a new `StreamingExecutor` that uses Tokio's async process module for non-blocking command execution with real-time output streaming.

## Context
The current `CommandExecutor` uses synchronous `std::process::Command::output()` which blocks until completion. This prevents streaming output and makes timeout enforcement impossible.

## Requirements
- [ ] Create `src/execution/streaming.rs` with `StreamingExecutor` struct
- [ ] Use `tokio::process::Command` with piped stdout/stderr
- [ ] Implement async streaming via `AsyncBufReadExt`
- [ ] Return `StreamingResult` with partial output support
- [ ] Add `TerminationReason` enum (Completed, Timeout, Cancelled, Error)
- [ ] Maintain backward compatibility with existing `ExecutionResult`

## Technical Design
```rust
pub struct StreamingExecutor {
    timeout: Duration,
    show_progress: bool,
}

pub struct StreamingResult {
    pub exit_code: Option<i32>,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub execution_time_ms: u64,
    pub terminated_by: TerminationReason,
}
```

## Testing
- [ ] Unit tests for timeout behavior
- [ ] Unit tests for cancellation
- [ ] Integration tests with real commands
- [ ] Cross-platform tests (macOS, Linux)

## Files to Modify
- Create: `src/execution/streaming.rs`
- Modify: `src/execution/mod.rs` (add exports)
- Modify: `src/main.rs` (integrate new executor)

## Dependencies
- None (foundational)

## References
- ADR-011: Streaming Execution Architecture
- Tokio process docs: https://docs.rs/tokio/latest/tokio/process/
```

---

### LRC-002: Add real-time output display

**Labels**: `enhancement`, `ux`, `display`
**Milestone**: v1.2.0
**Priority**: Critical
**Estimate**: Medium

```markdown
## Summary
Display command stdout/stderr in real-time as it's produced, rather than waiting for command completion.

## Context
Currently, all output is buffered and displayed only after command exits. Users see nothing during execution.

## Requirements
- [ ] Create output display module (`src/display/stream.rs`)
- [ ] Stream stdout lines as they arrive
- [ ] Stream stderr lines (with visual distinction)
- [ ] Handle interleaved stdout/stderr gracefully
- [ ] Respect terminal width for line wrapping
- [ ] Support both TTY and non-TTY contexts

## UX Design
```
Executing command...
[stdout] ./node_modules/.cache/large-bundle.js (156MB)
[stdout] ./data/training-set.csv (234MB)
[stderr] find: ./private: Permission denied
```

## Testing
- [ ] Test with commands producing mixed stdout/stderr
- [ ] Test with high-volume output
- [ ] Test non-TTY behavior (piped output)

## Files to Modify
- Create: `src/display/stream.rs`
- Modify: `src/display/mod.rs`
- Modify: `src/main.rs` (wire up display)

## Dependencies
- LRC-001: StreamingExecutor
```

---

### LRC-003: Implement Ctrl+C cancellation

**Labels**: `enhancement`, `core`, `signals`
**Milestone**: v1.2.0
**Priority**: Critical
**Estimate**: Medium

```markdown
## Summary
Allow users to cancel running commands with Ctrl+C, gracefully terminating the child process and preserving partial output.

## Context
Currently, Ctrl+C kills the entire caro process. Users cannot cancel just the running command.

## Requirements
- [ ] Install signal handler for SIGINT/SIGTERM
- [ ] Propagate cancellation to child process
- [ ] Wait for child process cleanup
- [ ] Preserve partial output collected before cancellation
- [ ] Display cancellation message with partial results
- [ ] Restore terminal state after cancellation

## UX Design
```
Executing command...
[00:05] ./data/file1.txt (150MB)
^C
Cancelled by user after 5.2 seconds.

Partial results (1 file found):
  ./data/file1.txt (150MB)
```

## Testing
- [ ] Test Ctrl+C during execution
- [ ] Test process cleanup (no zombies)
- [ ] Test partial output preservation

## Files to Modify
- Create: `src/execution/signals.rs`
- Modify: `src/execution/streaming.rs`
- Modify: `src/main.rs`

## Dependencies
- LRC-001: StreamingExecutor
- LRC-002: Real-time output display
```

---

### LRC-004: Add elapsed time and activity indicator

**Labels**: `enhancement`, `ux`, `display`
**Milestone**: v1.2.0
**Priority**: High
**Estimate**: Small

```markdown
## Summary
Show elapsed time and a spinner/activity indicator during command execution, especially when the command produces no output.

## Context
Commands like `find` on large directories may produce no output for extended periods, making users think the app is frozen.

## Requirements
- [ ] Display elapsed time counter `[00:05]`
- [ ] Show spinner animation when no recent output
- [ ] Update spinner at 100ms intervals
- [ ] Pause spinner when output appears
- [ ] Support non-TTY mode (disable animations)

## UX Design
```
Executing command...
[00:00] ⠋ Searching...
[00:02] ./data/file1.txt (150MB)
[00:05] ⠙ Still running...
[00:08] ./data/file2.txt (200MB)
[00:12] Complete. Found 2 files.
```

## Testing
- [ ] Test spinner animation timing
- [ ] Test elapsed time accuracy
- [ ] Test non-TTY fallback

## Files to Modify
- Modify: `src/display/stream.rs`
- Add: spinner animation logic

## Dependencies
- LRC-002: Real-time output display
```

---

## Timeout Issues

### LRC-005: Implement preemptive timeout

**Labels**: `enhancement`, `core`, `execution`
**Milestone**: v1.2.0
**Priority**: Critical
**Estimate**: Medium

```markdown
## Summary
Implement actual timeout enforcement that kills commands exceeding the time limit, rather than just checking duration after completion.

## Context
Current timeout check happens AFTER execution completes, which cannot prevent hanging commands.

## Requirements
- [ ] Use `tokio::time::timeout()` to wrap execution
- [ ] Kill child process when timeout expires
- [ ] Configurable timeout (default: 30 seconds)
- [ ] Config option: `execution.timeout`
- [ ] CLI flag: `--timeout <seconds>`
- [ ] Preserve partial output on timeout

## Configuration
```toml
[execution]
timeout = 30  # seconds
```

## CLI
```bash
caro --timeout 60 "find large files"
```

## Testing
- [ ] Test timeout triggers at correct time
- [ ] Test process is actually killed
- [ ] Test partial output preserved
- [ ] Test custom timeout values

## Files to Modify
- Modify: `src/execution/streaming.rs`
- Modify: `src/cli/mod.rs` (add flag)
- Modify: `src/config/mod.rs` (add setting)

## Dependencies
- LRC-001: StreamingExecutor
```

---

### LRC-006: Display partial results on timeout

**Labels**: `enhancement`, `ux`, `display`
**Milestone**: v1.2.0
**Priority**: High
**Estimate**: Small

```markdown
## Summary
When a command times out, display any partial results collected before the timeout occurred.

## Context
Users want to see what was found even if the command didn't complete. This provides value and helps users understand command behavior.

## Requirements
- [ ] Buffer output during execution
- [ ] On timeout, format and display buffered output
- [ ] Show count of results found
- [ ] Indicate more results may exist
- [ ] Handle large output gracefully (truncate if needed)

## UX Design
```
Timeout after 30 seconds.

Partial results (15 files found):
  ./cache/model-weights.bin (2.1GB)
  ./data/dataset.tar (1.8GB)
  ... and 13 more

This command may find additional results given more time.
```

## Testing
- [ ] Test partial output display
- [ ] Test with empty partial output
- [ ] Test with large partial output

## Files to Modify
- Modify: `src/display/stream.rs`
- Add: timeout result formatting

## Dependencies
- LRC-005: Preemptive timeout
```

---

### LRC-007: Add educational timeout messaging

**Labels**: `enhancement`, `ux`, `docs`
**Milestone**: v1.2.0
**Priority**: High
**Estimate**: Small

```markdown
## Summary
Provide helpful, educational messages when commands timeout, explaining why and offering alternatives.

## Context
Users need to understand that caro is optimized for quick shell tasks and that long-running commands are better run directly.

## Requirements
- [ ] Explain timeout purpose in user-friendly language
- [ ] Suggest using Edit option for long-running commands
- [ ] Offer to retry with longer timeout
- [ ] Provide command improvement tips when applicable
- [ ] Keep messaging concise and actionable

## UX Design
```
Timeout after 30 seconds.

Partial results:
  ./cache/model-weights.bin (2.1GB)

Caro is optimized for quick shell tasks. For long-running operations:

  [E] Edit - Run this command directly in your shell
  [R] Retry with longer timeout (2 min)
  [C] Cancel

Tip: For large searches, try limiting the scope:
  find ./src -maxdepth 3 -type f -size +100M
```

## Testing
- [ ] Test messaging displays correctly
- [ ] Test Edit option integration
- [ ] Test tip generation for common commands

## Files to Modify
- Create: `src/display/messages.rs`
- Modify: `src/main.rs` (timeout handling)

## Dependencies
- LRC-005: Preemptive timeout
- LRC-006: Partial results display
```

---

## Scope Intelligence Issues

### LRC-008: Detect broad-scope commands

**Labels**: `enhancement`, `safety`, `ux`
**Milestone**: v1.2.0
**Priority**: Medium
**Estimate**: Medium

```markdown
## Summary
Detect commands that may have overly broad scope and warn users or prompt for scope refinement before execution.

## Context
Commands like `find ~` or `grep -r /` can take very long. Proactive warnings help users refine scope.

## Requirements
- [ ] Identify broad-scope command patterns:
  - `find` on ~, /, large directories without depth limits
  - `grep -r` on broad paths
  - `du` on home/root
  - `rsync` large datasets
- [ ] Estimate scope impact (file count approximation)
- [ ] Display warning before execution
- [ ] Suggest scope refinement options
- [ ] Config option to disable warnings

## Pattern Detection
```rust
fn is_broad_scope(command: &str) -> Option<ScopeWarning> {
    // Detect: find ~, find ., find / without -maxdepth
    // Detect: grep -r without specific path
    // Detect: du ~ or du /
}
```

## UX Design
```
Command: find . -type f -size +100M

Warning: This command searches the entire current directory (~50,000 files).
This may take several minutes.

Would you like to:
  [1] Continue anyway
  [2] Limit depth: find . -maxdepth 3 -type f -size +100M
  [3] Specify path (enter directory)
  [4] Cancel
```

## Testing
- [ ] Test pattern detection accuracy
- [ ] Test warning display
- [ ] Test scope refinement suggestions

## Files to Modify
- Create: `src/safety/scope.rs`
- Modify: `src/main.rs` (pre-execution check)

## Dependencies
- None (can be developed in parallel)
```

---

## Labels to Create

```bash
# Create labels for this feature
gh label create "long-running" --description "Long-running command handling" --color "d4c5f9"
gh label create "streaming" --description "Output streaming features" --color "c5def5"
gh label create "timeout" --description "Timeout and cancellation" --color "fbca04"
gh label create "scope" --description "Command scope analysis" --color "0e8a16"
```

---

## Milestone Setup

```bash
# Create milestone
gh api repos/{owner}/{repo}/milestones -f title="v1.2.0" -f description="Long-Running Command UX Improvements" -f due_on="2026-02-28T00:00:00Z"
```

---

## Issue Creation Script

Run this to create all issues at once:

```bash
#!/bin/bash
# Create epic first, then child issues
# Store issue numbers for linking

MILESTONE="v1.2.0"

# Epic
gh issue create \
  --title "[Epic] Long-Running Command UX Improvements" \
  --body-file docs/prds/issues/epic-body.md \
  --label "enhancement,epic,ux" \
  --milestone "$MILESTONE"

# Then create each LRC-00X issue...
```

---

## Summary

| Issue | Priority | Estimate | Dependencies |
|-------|----------|----------|--------------|
| LRC-000 | Epic | - | All |
| LRC-001 | Critical | Large | None |
| LRC-002 | Critical | Medium | LRC-001 |
| LRC-003 | Critical | Medium | LRC-001, LRC-002 |
| LRC-004 | High | Small | LRC-002 |
| LRC-005 | Critical | Medium | LRC-001 |
| LRC-006 | High | Small | LRC-005 |
| LRC-007 | High | Small | LRC-005, LRC-006 |
| LRC-008 | Medium | Medium | None |

**Total Critical Issues**: 4
**Recommended Execution Order**: LRC-001 → LRC-002 → LRC-005 → LRC-003 → LRC-004 → LRC-006 → LRC-007 → LRC-008
