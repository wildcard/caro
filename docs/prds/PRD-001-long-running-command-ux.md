# PRD-001: Long-Running Command UX Improvements

**Status**: Proposed
**Created**: January 18, 2026
**Author**: @wildcard
**Target Release**: v1.2.0 or v1.3.0
**Priority**: High

---

## Executive Summary

Users experience a "stuck" or "unresponsive" feeling when caro executes commands that take significant time to complete (e.g., `find . -type f -size +100M` from home directory). This PRD addresses three interconnected improvements to transform long-running command handling into a delightful, informative experience.

### The Problem (Observed)

```
kobi@MacBook-Air ~ % caro -v --force-llm files larger than 100mb
Command:
  find . -type f -size +100M

Explanation:
  Generated using MLX backend

Execute this command?: Yes - execute

Executing command...
█  <- Appears stuck/frozen, no feedback
```

**User Impact**: Confusion, lost trust, potential force-quit of process

---

## Problem Statement

### Current State

1. **No streaming output** - Commands execute synchronously, blocking until complete
2. **No progress indication** - Users cannot tell if a command is running or stuck
3. **No timeout enforcement** - Long-running commands never terminate
4. **Overly broad scope** - Commands often target larger scope than intended
5. **No graceful exit** - Users have no option to cancel or hand off to native shell

### User Personas Affected

| Persona | Pain Point |
|---------|------------|
| Terminal Novice | Thinks app is broken, force-quits |
| Power User | Annoyed by lack of control, switches to manual commands |
| SSH User | Timeout anxiety on slow connections |
| Data Scientist | Large file operations freeze workflow |

---

## Goals & Success Metrics

### Goals

1. **Transparency** - Users always know what's happening
2. **Control** - Users can cancel, modify, or delegate commands
3. **Intelligence** - System proactively suggests better scopes
4. **Trust** - Build confidence through predictable behavior

### Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| User force-quit rate | Unknown | <2% | Telemetry |
| Time-to-realization (stuck) | >30s | 3s | User testing |
| Command scope accuracy | ~60% | 85% | Evaluation suite |
| User satisfaction (surveys) | N/A | 4.2/5.0 | Post-beta surveys |

---

## Feature Requirements

### Feature 1: Smart Command Scope Inference

**Priority**: High
**Complexity**: Medium

#### Description

Improve command generation to infer appropriate scope from context, user intent, and environment.

#### User Stories

1. As a user, when I say "files larger than 100MB", I expect caro to ask about the target directory or infer it from context
2. As a user, I want caro to recognize when a command might have a very large scope and warn me
3. As a user, I want suggested alternatives with different scopes

#### Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| F1.1 | Detect potentially large-scope commands (find, grep -r, du) | Must |
| F1.2 | Prompt for target directory when scope is ambiguous | Must |
| F1.3 | Suggest adding file type filters when appropriate | Should |
| F1.4 | Show estimated scope impact (e.g., "This will search ~50,000 files") | Should |
| F1.5 | Learn from user patterns (e.g., user often works in ~/Projects) | Could |

#### Example Flow

```
User: files larger than 100mb

Caro: I'll search for large files. Where should I look?
  [1] Current directory (./caro - ~500 files)
  [2] Home directory (~/ - ~50,000 files)
  [3] Enter custom path
  [4] Use current directory with depth limit

> 1

Command:
  find . -type f -size +100M
```

---

### Feature 2: Streaming Output for Long-Running Commands

**Priority**: Critical
**Complexity**: High

#### Description

Stream command output in real-time, showing users that the command is actively running and producing results.

#### User Stories

1. As a user, I want to see output appearing as the command runs, not all at once at the end
2. As a user, I want a visual indicator that the command is actively running
3. As a user, I want to cancel a running command with Ctrl+C

#### Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| F2.1 | Stream stdout/stderr in real-time during execution | Must |
| F2.2 | Show elapsed time indicator during execution | Must |
| F2.3 | Support Ctrl+C cancellation with graceful process termination | Must |
| F2.4 | Display spinner/activity indicator when no output | Must |
| F2.5 | Buffer and display partial output on interruption | Should |
| F2.6 | Optional "quiet mode" for non-interactive contexts | Should |

#### Technical Considerations

- Use `tokio::process::Command` for async execution
- Pipe stdout/stderr through async readers
- Implement proper signal handling for Ctrl+C
- Consider terminal capability detection (dumb terminals, CI)

#### Example Flow

```
Executing command...
[00:00] Searching...
[00:02] ./node_modules/.cache/large-bundle.js (156MB)
[00:05] ./data/training-set.csv (234MB)
[00:08] ▌ Still running... (press Ctrl+C to cancel)
[00:12] Complete. Found 2 files in 12.3 seconds.
```

---

### Feature 3: Intelligent Timeout & Handoff

**Priority**: High
**Complexity**: Medium

#### Description

Implement timeouts with intelligent messaging, suggesting users run long-running commands directly in their shell.

#### User Stories

1. As a user, when a command takes too long, I want caro to explain what happened
2. As a user, I want to see any partial results collected before timeout
3. As a user, I want caro to suggest running the command directly in my shell

#### Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| F3.1 | Implement configurable execution timeout (default: 30s) | Must |
| F3.2 | Display timeout warning at 50% of timeout duration | Should |
| F3.3 | On timeout, show partial output collected | Must |
| F3.4 | Explain why timeout occurred with educational messaging | Must |
| F3.5 | Suggest "Edit" option to run directly in shell | Must |
| F3.6 | Allow per-command timeout override | Could |
| F3.7 | Detect command patterns likely to be long-running and warn pre-execution | Should |

#### Long-Running Command Patterns

Commands likely to be long-running and should trigger proactive warnings:

- `find` without depth limits on broad paths
- `grep -r` on large directories
- `du` on home/root directories
- `rsync` large datasets
- `tar` of large archives
- `npm install` / `cargo build` (compilation)

#### Example Flow - Timeout

```
Executing command...
[00:15] Warning: This command is taking longer than expected
[00:30]

Timeout after 30 seconds.

Partial results:
  ./cache/model-weights.bin (2.1GB)
  ./data/dataset.tar (1.8GB)

This command is still running and may complete given more time.
Caro is optimized for quick shell tasks. For long-running operations:

  [E] Edit - Run this command directly in your shell
  [R] Retry with longer timeout (2 min)
  [C] Cancel

Tip: For large searches, try limiting the depth:
  find . -maxdepth 3 -type f -size +100M
```

---

## User Experience

### Command Confirmation Enhancement

Update the existing confirmation dialog to include scope awareness:

**Before:**
```
Execute this command?: Yes - execute
```

**After:**
```
Command: find . -type f -size +100M
Scope: Current directory (~500 files, max depth: unlimited)

[Y] Yes - execute
[E] Edit - modify and run in shell
[N] No - cancel
[?] Why this command?
```

### Configuration Options

New configuration keys for `~/.config/caro/config.toml`:

```toml
[execution]
# Default timeout for command execution (seconds)
timeout = 30

# Show streaming output (true/false)
stream_output = true

# Warn before executing broad-scope commands
scope_warnings = true

# Interactive scope refinement prompts
prompt_for_scope = true
```

### CLI Flags

New flags to support these features:

```bash
caro --timeout 60 "find large files"     # Override timeout
caro --no-stream "list processes"        # Disable streaming
caro --scope ./src "find todos"          # Pre-specify scope
```

---

## Technical Dependencies

| Dependency | Description | Status |
|------------|-------------|--------|
| Tokio async runtime | Async process execution | Existing |
| Signal handling | Ctrl+C cancellation | New |
| Terminal detection | Capability checking | New |
| Directory analysis | File count estimation | New |

See **ADR-011: Streaming Execution Architecture** for technical design.

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Streaming breaks pipe compatibility | High | Medium | Detect TTY vs pipe, disable streaming for pipes |
| Timeout interrupts legitimate commands | Medium | Low | Configurable timeout, educational messaging |
| Scope prompts slow down quick tasks | Medium | Medium | Remember preferences, skip for obvious scopes |
| Platform differences in process handling | Medium | Medium | Comprehensive cross-platform testing |

---

## Out of Scope

- Real-time progress bars (requires command cooperation)
- Command resume/checkpoint (too complex)
- Distributed execution (future Karo feature)
- Command prediction/autocomplete (separate feature)

---

## Rollout Plan

### Phase 1: Streaming Output (v1.2.x)
- Implement async execution with streaming
- Add timeout enforcement with partial output
- Educational timeout messaging

### Phase 2: Smart Scope (v1.3.x)
- Command scope analysis
- Interactive scope refinement
- Scope estimation display

### Phase 3: Intelligence (v2.0+)
- Learn from user patterns
- Predictive scope suggestions
- Integration with Karo distributed system

---

## Appendix

### Related Documents

- ADR-011: Streaming Execution Architecture
- GitHub Issues: #LRC-001 through #LRC-008 (to be created)
- ROADMAP.md: v1.2.0 and v1.3.0 milestones

### User Research

Initial feedback from beta testers (Jan 2026):
- "Thought the app froze when running find command"
- "Would be nice to see what it's finding as it runs"
- "I had to kill the process, didn't know if it was working"

### Competitive Analysis

| Tool | Streaming | Timeout | Scope Inference |
|------|-----------|---------|-----------------|
| GitHub Copilot CLI | No | No | No |
| Warp AI | Yes | No | Partial |
| Fig/Amazon Q | No | No | No |
| **Caro (proposed)** | Yes | Yes | Yes |
