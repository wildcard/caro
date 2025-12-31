# MVP Milestones and Task Breakdown

## Overview

This document breaks down the shell integration feature into phases, milestones, and actionable stories. Each story is sized as S (small), M (medium), or L (large) based on complexity.

---

## Phase 1: Core Infrastructure

**Objective**: Build the foundational daemon, IPC layer, and basic shell hooks

### Milestone 1.1: Shell Daemon Foundation

| Story | Size | Description |
|-------|------|-------------|
| S1.1.1 | M | Create `src/shell/mod.rs` module structure with submodules |
| S1.1.2 | L | Implement Unix socket server in `src/shell/ipc/socket.rs` |
| S1.1.3 | M | Define IPC message protocol in `src/shell/ipc/protocol.rs` |
| S1.1.4 | M | Implement session management in `src/shell/session.rs` |
| S1.1.5 | M | Create daemon lifecycle (start/stop/status) in `src/shell/daemon.rs` |
| S1.1.6 | S | Add `caro daemon` CLI subcommand |
| S1.1.7 | S | Write unit tests for IPC protocol serialization |
| S1.1.8 | M | Write integration tests for daemon lifecycle |

**Acceptance Criteria**:
- [ ] `caro daemon start` starts background process
- [ ] `caro daemon stop` gracefully terminates daemon
- [ ] `caro daemon status` reports running/stopped
- [ ] Socket created with correct permissions (600)
- [ ] Multiple clients can connect concurrently
- [ ] Sessions tracked per-connection

### Milestone 1.2: bash Integration

| Story | Size | Description |
|-------|------|-------------|
| S1.2.1 | M | Create `shell-scripts/bash.init` with hook infrastructure |
| S1.2.2 | S | Implement `__caro_preexec` using DEBUG trap |
| S1.2.3 | S | Implement `__caro_postcmd` using PROMPT_COMMAND |
| S1.2.4 | S | Add IPC client helper function (netcat-based fallback) |
| S1.2.5 | S | Implement interactive detection (`[[ $- == *i* ]]`) |
| S1.2.6 | S | Add CARO_DISABLE environment variable support |
| S1.2.7 | M | Write bash integration tests |
| S1.2.8 | S | Test with bash 4.x and 5.x |

**Acceptance Criteria**:
- [ ] Hooks fire on every command in interactive bash
- [ ] Non-interactive scripts unaffected
- [ ] CARO_DISABLE=1 disables all hooks
- [ ] No noticeable latency on command execution
- [ ] Exit code captured correctly

### Milestone 1.3: zsh Integration

| Story | Size | Description |
|-------|------|-------------|
| S1.3.1 | M | Create `shell-scripts/zsh.init` with hook infrastructure |
| S1.3.2 | S | Implement preexec hook using add-zsh-hook |
| S1.3.3 | S | Implement precmd hook using add-zsh-hook |
| S1.3.4 | S | Ensure compatibility with oh-my-zsh |
| S1.3.5 | S | Ensure compatibility with prezto |
| S1.3.6 | M | Write zsh integration tests |
| S1.3.7 | S | Test with zsh 5.x |

**Acceptance Criteria**:
- [ ] Works alongside oh-my-zsh plugins
- [ ] Works alongside prezto
- [ ] Hooks use add-zsh-hook (not direct override)
- [ ] Custom prompts not affected

### Milestone 1.4: fish Integration

| Story | Size | Description |
|-------|------|-------------|
| S1.4.1 | M | Create `shell-scripts/fish/` directory structure |
| S1.4.2 | S | Implement fish_preexec event handler |
| S1.4.3 | S | Implement fish_postexec event handler |
| S1.4.4 | S | Create fish functions for IPC |
| S1.4.5 | M | Write fish integration tests |
| S1.4.6 | S | Test with fish 3.x |

**Acceptance Criteria**:
- [ ] Events fire correctly in fish
- [ ] CMD_DURATION captured
- [ ] Works with fisher and oh-my-fish
- [ ] Auto-loads from conf.d

---

## Phase 2: Safety and Policy Engine

**Objective**: Implement configurable safety levels and command validation

### Milestone 2.1: Policy Engine Core

| Story | Size | Description |
|-------|------|-------------|
| S2.1.1 | M | Create `src/shell/policy/mod.rs` with SafetyLevel enum |
| S2.1.2 | M | Implement PolicyEngine with rule evaluation |
| S2.1.3 | M | Integrate existing SafetyValidator from `src/safety/` |
| S2.1.4 | S | Add blocklist pattern support |
| S2.1.5 | S | Add allowlist pattern support |
| S2.1.6 | M | Implement directory-based policy overrides |
| S2.1.7 | M | Write policy engine unit tests |
| S2.1.8 | M | Write property-based tests for edge cases |

**Acceptance Criteria**:
- [ ] Three safety levels (off, passive, active) working
- [ ] Blocklist patterns block matching commands
- [ ] Allowlist patterns bypass safety checks
- [ ] Directory overrides apply correctly
- [ ] Existing safety patterns from `src/safety/` reused

### Milestone 2.2: Pre-execution Workflow

| Story | Size | Description |
|-------|------|-------------|
| S2.2.1 | M | Implement preexec handler in daemon |
| S2.2.2 | S | Return warnings to shell for passive mode |
| S2.2.3 | M | Implement confirmation prompt for active mode |
| S2.2.4 | S | Implement blocking for critical-risk commands |
| S2.2.5 | M | Update shell scripts to handle responses |
| S2.2.6 | M | Write integration tests for pre-exec flow |

**Acceptance Criteria**:
- [ ] Passive mode shows warnings but allows execution
- [ ] Active mode prompts for confirmation on high-risk
- [ ] Active mode blocks critical-risk
- [ ] Off mode has no safety checks
- [ ] Timeout falls through (allows command)

### Milestone 2.3: Configuration System

| Story | Size | Description |
|-------|------|-------------|
| S2.3.1 | M | Define config schema in `src/shell/config.rs` |
| S2.3.2 | S | Implement config loading from TOML |
| S2.3.3 | S | Add environment variable overrides |
| S2.3.4 | S | Create default configuration template |
| S2.3.5 | S | Add `caro config show` command |
| S2.3.6 | S | Add `caro config edit` command |
| S2.3.7 | M | Write config validation and tests |

**Acceptance Criteria**:
- [ ] Config file at `~/.config/caro/config.toml`
- [ ] Environment variables override config
- [ ] Invalid config shows helpful errors
- [ ] Default config created on first run

---

## Phase 3: Fix-It Engine

**Objective**: Implement post-failure suggestion system ("thefuck"-like)

### Milestone 3.1: Pattern-Based Fixes

| Story | Size | Description |
|-------|------|-------------|
| S3.1.1 | M | Create `src/shell/fixit/mod.rs` engine |
| S3.1.2 | M | Build pattern database in `src/shell/fixit/patterns.rs` |
| S3.1.3 | S | Implement typo correction (Levenshtein distance) |
| S3.1.4 | S | Implement sudo suggestion for permission denied |
| S3.1.5 | S | Implement path correction for file not found |
| S3.1.6 | M | Add confidence scoring |
| S3.1.7 | M | Write fix-it unit tests |

**Common Patterns to Implement**:
- [ ] Command not found → suggest similar command
- [ ] Permission denied → suggest sudo
- [ ] File not found → check typos in path
- [ ] Common typos (gti→git, sl→ls)
- [ ] Git-specific errors
- [ ] Package manager errors

**Acceptance Criteria**:
- [ ] Suggestions appear after non-zero exit
- [ ] Confidence score filters low-quality suggestions
- [ ] User can configure typo corrections
- [ ] No suggestions for successful commands

### Milestone 3.2: Apply Fix UX

| Story | Size | Description |
|-------|------|-------------|
| S3.2.1 | M | Implement Esc Esc keybinding in bash |
| S3.2.2 | M | Implement Esc Esc keybinding in zsh (ZLE) |
| S3.2.3 | M | Implement Esc Esc keybinding in fish |
| S3.2.4 | S | Store last suggestion in session state |
| S3.2.5 | S | Apply fix by modifying command line buffer |
| S3.2.6 | M | Write keybinding integration tests |

**Acceptance Criteria**:
- [ ] Esc Esc applies last suggestion
- [ ] Command line buffer updated (user can review)
- [ ] Works consistently across shells
- [ ] Clear visual feedback when fix applied

---

## Phase 4: Installer and Deployment

**Objective**: Create user-friendly installation and management experience

### Milestone 4.1: Installer Implementation

| Story | Size | Description |
|-------|------|-------------|
| S4.1.1 | M | Create `caro shell install` command |
| S4.1.2 | S | Implement shell detection |
| S4.1.3 | S | Create XDG-compliant directories |
| S4.1.4 | M | Copy shell scripts to config directory |
| S4.1.5 | M | Add source line to rc files (marked block) |
| S4.1.6 | S | Handle existing installations (idempotent) |
| S4.1.7 | M | Create `caro shell uninstall` command |
| S4.1.8 | S | Implement backup before modifications |

**Acceptance Criteria**:
- [ ] `caro shell install` works on fresh system
- [ ] Re-running install doesn't duplicate config
- [ ] Uninstall removes all traces
- [ ] Backups created before rc file modification

### Milestone 4.2: Doctor and Diagnostics

| Story | Size | Description |
|-------|------|-------------|
| S4.2.1 | M | Implement `caro doctor` command |
| S4.2.2 | S | Check shell script presence |
| S4.2.3 | S | Check rc file integration |
| S4.2.4 | S | Check daemon status |
| S4.2.5 | S | Check socket connectivity |
| S4.2.6 | S | Provide fix recommendations |

**Acceptance Criteria**:
- [ ] Doctor identifies common issues
- [ ] Clear output showing status of each component
- [ ] Actionable recommendations for fixes

---

## Phase 5: Interactive Features

**Objective**: Add Caro interactive prompt and UI polish

### Milestone 5.1: Caro Interactive Prompt

| Story | Size | Description |
|-------|------|-------------|
| S5.1.1 | M | Implement `__caro_invoke` function |
| S5.1.2 | M | Add Ctrl+X Ctrl+C keybinding (bash) |
| S5.1.3 | M | Add Ctrl+X Ctrl+C keybinding (zsh ZLE) |
| S5.1.4 | M | Add Ctrl+X Ctrl+C keybinding (fish) |
| S5.1.5 | S | Integrate with existing caro command |
| S5.1.6 | S | Preserve command line buffer on cancel |

**Acceptance Criteria**:
- [ ] Hotkey opens simple "Caro>" prompt
- [ ] User can type natural language
- [ ] Generated command shown
- [ ] Previous command line buffer restored after

### Milestone 5.2: UI Polish

| Story | Size | Description |
|-------|------|-------------|
| S5.2.1 | S | Add colored output for warnings |
| S5.2.2 | S | Add colored output for blocked commands |
| S5.2.3 | S | Add colored output for suggestions |
| S5.2.4 | S | Implement CARO_NO_COLOR support |
| S5.2.5 | S | Add configurable icons/emoji |

**Acceptance Criteria**:
- [ ] Colors distinguish warning/error/success
- [ ] CARO_NO_COLOR disables colors
- [ ] Terminal compatibility (falls back gracefully)

---

## Phase 6: Documentation and Release

**Objective**: Prepare for release with documentation and final polish

### Milestone 6.1: Documentation

| Story | Size | Description |
|-------|------|-------------|
| S6.1.1 | M | Write user documentation |
| S6.1.2 | S | Document installation process |
| S6.1.3 | S | Document configuration options |
| S6.1.4 | S | Document keybindings |
| S6.1.5 | S | Add troubleshooting guide |
| S6.1.6 | M | Update README with shell integration |

### Milestone 6.2: Release Preparation

| Story | Size | Description |
|-------|------|-------------|
| S6.2.1 | S | Update CHANGELOG |
| S6.2.2 | S | Version bump |
| S6.2.3 | M | End-to-end testing on fresh systems |
| S6.2.4 | M | Performance benchmarking |
| S6.2.5 | S | Security audit (cargo audit) |
| S6.2.6 | S | Create release notes |

---

## Story Size Definitions

| Size | Complexity | Story Points |
|------|------------|--------------|
| S (Small) | Straightforward, well-defined | 1-2 |
| M (Medium) | Some complexity, clear approach | 3-5 |
| L (Large) | Complex, may need spike | 8-13 |

---

## Dependency Graph

```
Phase 1 (Core Infrastructure)
├── Milestone 1.1: Shell Daemon Foundation
│   └── Milestone 1.2, 1.3, 1.4 (shell integrations) [depends on 1.1]
│
Phase 2 (Safety and Policy)
├── Milestone 2.1: Policy Engine Core [depends on 1.1]
│   └── Milestone 2.2: Pre-execution Workflow [depends on 2.1, 1.2-1.4]
│       └── Milestone 2.3: Configuration System [depends on 2.1]
│
Phase 3 (Fix-It Engine)
├── Milestone 3.1: Pattern-Based Fixes [depends on 1.1]
│   └── Milestone 3.2: Apply Fix UX [depends on 3.1, 1.2-1.4]
│
Phase 4 (Installer)
├── Milestone 4.1: Installer Implementation [depends on 1.2-1.4]
│   └── Milestone 4.2: Doctor and Diagnostics [depends on 4.1]
│
Phase 5 (Interactive)
├── Milestone 5.1: Caro Interactive Prompt [depends on 1.2-1.4]
│   └── Milestone 5.2: UI Polish [depends on 5.1]
│
Phase 6 (Documentation)
└── All documentation [depends on all features]
```

---

## MVP Definition

**MVP includes**:
- Phase 1: All milestones (daemon + bash + zsh + fish)
- Phase 2: Milestones 2.1, 2.2, 2.3 (safety with config)
- Phase 3: Milestones 3.1, 3.2 (fix-it with apply)
- Phase 4: Milestones 4.1, 4.2 (installer + doctor)
- Phase 5: Milestone 5.1 only (basic interactive)
- Phase 6: Basic documentation

**MVP excludes** (v1.5/v2.0):
- Inline suggestions while typing
- Advanced TUI overlay
- LLM-powered suggestions
- Cross-machine config sync
- Plugin system

---

## Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| bash DEBUG trap conflicts | High | Thorough testing with various configs |
| zsh framework incompatibility | Medium | Test with oh-my-zsh, prezto |
| Performance regression | High | Benchmark in CI |
| Socket permission issues | Medium | Test on multiple Linux distros |
| fish scripting differences | Low | Fish-native implementation |

---

## Sprint Suggestions

### Sprint 1: Foundation
- S1.1.1 through S1.1.8 (Daemon)
- S1.2.1 through S1.2.4 (bash basics)

### Sprint 2: Shell Coverage
- S1.2.5 through S1.2.8 (bash complete)
- S1.3.1 through S1.3.7 (zsh complete)

### Sprint 3: fish + Policy
- S1.4.1 through S1.4.6 (fish complete)
- S2.1.1 through S2.1.4 (policy core)

### Sprint 4: Safety Workflow
- S2.1.5 through S2.1.8 (policy complete)
- S2.2.1 through S2.2.6 (pre-exec workflow)

### Sprint 5: Fix-It Engine
- S3.1.1 through S3.1.7 (pattern fixes)
- S3.2.1 through S3.2.3 (keybindings)

### Sprint 6: Installer
- S4.1.1 through S4.1.8 (installer)
- S4.2.1 through S4.2.6 (doctor)

### Sprint 7: Polish + Release
- S5.1.1 through S5.1.6 (interactive)
- S6.1.1 through S6.2.6 (docs + release)
