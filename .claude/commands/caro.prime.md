---
description: Load project context and prepare for development session
---

## Prime: Load Project Context

This command systematically analyzes the caro codebase to build deep understanding before starting development work.

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/caro.prime` | Full context loading |
| `/caro.prime quick` | Essential files only |
| `/caro.prime status` | Git status and recent activity |

---

## What This Command Does

`/caro.prime` prepares you for effective development by:
- Reading core documentation (CLAUDE.md, consolidated rules)
- Analyzing project structure
- Checking recent git activity
- Loading relevant reference materials
- Outputting a structured context summary

---

## Outline

### 1. Parse Arguments

```
ARGUMENTS check:
- Empty → FULL_MODE (complete context loading)
- "quick" → QUICK_MODE (essential files only)
- "status" → STATUS_MODE (git activity only)
```

### 2. Read Core Documentation

**Always read these files:**

```bash
# Primary context files
cat CLAUDE.md
cat .claude/memory/consolidated-knowledge-rules.md
```

**In FULL_MODE, also read:**

```bash
# Architecture understanding
cat README.md | head -100
cat Cargo.toml
cat src/main.rs | head -50
```

### 3. Analyze Project Structure

Run structure analysis:

```bash
# Show tracked files by type
git ls-files | head -50

# Show source structure
find src -name "*.rs" -type f | head -30

# Show test structure
find tests -name "*.rs" -type f 2>/dev/null | head -20

# Show command structure
ls -la .claude/commands/

# Show agent structure
ls -la .claude/agents/
```

### 4. Check Current State

```bash
# Current branch
git branch --show-current

# Recent commits
git log --oneline -10

# Working tree status
git status --short

# Any uncommitted changes
git diff --stat HEAD
```

### 5. Load Reference Materials (FULL_MODE only)

```bash
# Check what reference docs exist
ls -la .claude/reference/

# Read relevant best practices (optional, based on current work)
```

### 6. Generate Context Report

Output a structured summary:

```markdown
## Caro Project Context

### Project Overview
- **Name**: caro
- **Version**: [from Cargo.toml]
- **Purpose**: Convert natural language to shell commands using local LLMs
- **Tech Stack**: Rust 1.83+, tokio, clap, MLX/Candle

### Current State
- **Branch**: [current branch]
- **Status**: [clean/dirty with summary]
- **Recent Focus**: [from last 5 commits]

### Key Architecture
- **Entry Point**: `src/main.rs`
- **Backends**: `src/backends/` (MLX, CPU, Ollama, vLLM)
- **Safety**: `src/safety/` (52+ validation patterns)
- **Config**: `src/config/`

### Available Commands
- `/caro.feature` - Feature development workflow
- `/caro.qa` - Bug investigation
- `/caro.roadmap` - Project management
- `/caro.release.*` - Release workflow
- `/spec-kitty.*` - Specification-driven development

### Key Rules
- Install script: `bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)`
- Never use pipe-to-shell patterns
- Follow consolidated knowledge rules

### Reference Materials
- `.claude/reference/rust-cli-best-practices.md`
- `.claude/reference/safety-patterns.md`
- `.claude/reference/testing-patterns.md`

### Ready to Develop
Context loaded. Use `/caro.feature` to start feature work or `/caro.qa` for bug investigation.
```

---

## Mode Details

### FULL_MODE (Default)

Complete context loading for new sessions or deep dives:
1. Read all core documentation
2. Full structure analysis
3. Git history review
4. Reference materials loaded
5. Comprehensive context report

**Use when**: Starting a new session, onboarding, major feature work

### QUICK_MODE

Essential context for quick tasks:
1. Read CLAUDE.md and consolidated rules
2. Check git status
3. Brief context report

**Use when**: Quick fixes, small changes, already familiar with codebase

### STATUS_MODE

Git-focused context:
1. Current branch and status
2. Recent commits
3. Uncommitted changes

**Use when**: Checking what's in progress, reviewing recent work

---

## Examples

### Starting a New Session
```
User: /caro.prime
Claude:
  Reading CLAUDE.md...
  Reading consolidated-knowledge-rules.md...
  Analyzing project structure...
  Checking git status...

  ## Caro Project Context

  ### Project Overview
  - Name: caro v1.0.4
  - Purpose: Natural language to shell commands
  ...

  Ready for development. What would you like to work on?
```

### Quick Context Check
```
User: /caro.prime quick
Claude:
  Quick context load...

  Branch: main
  Status: clean
  Last commit: "fix(safety): Block rm -rf * pattern"

  Key files: CLAUDE.md, .claude/reference/
  Ready for quick work.
```

### Status Check
```
User: /caro.prime status
Claude:
  Branch: feature/add-caching
  Status: 2 files modified

  Recent commits:
  - abc1234 feat: Add cache initialization
  - def5678 test: Add cache unit tests

  Uncommitted:
  - src/cache/mod.rs (modified)
  - tests/cache_test.rs (modified)
```

---

## Integration with Other Commands

After priming, typically proceed to:

| Scenario | Next Command |
|----------|-------------|
| Start feature work | `/caro.feature "description"` |
| Investigate bug | `/caro.qa #123` |
| Check roadmap | `/caro.roadmap` |
| Review code | `/caro.code-review` |

---

## Notes

- Run `/caro.prime` at the start of each session for best results
- The context report helps maintain consistency across long sessions
- Quick mode is sufficient for simple tasks
- Full mode is recommended for complex feature work
