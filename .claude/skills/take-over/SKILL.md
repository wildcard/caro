---
name: "take-over"
description: "Autonomous finish chain: review+fix → session-cleanup → commit. Use when interactive work is ~80-90% done and remaining work is mechanical."
version: "1.0.0"
allowed-tools: "Bash, Read, Edit, Write, Grep, Glob, Agent"
license: "AGPL-3.0"
---

# Take-Over Skill

## What This Skill Does

Chains together review, cleanup, and commit into a single autonomous sequence. Used when the human has made all key decisions and the remaining work is mechanical: reviewing, cleaning up, and committing.

Inspired by hboon's `take-over-finish` skill. See `docs/research/agentic-workflow-patterns-hboon.md`.

**Key insight:** "YOLO mode requires full upfront specification. Complex work demands iterative refinement. Take-over preserves interactive flexibility during exploration while enabling autonomous execution during mechanical work."

## When to Use This Skill

- After completing the core implementation interactively
- When all architectural decisions have been made
- When remaining work is review, cleanup, and commit
- When you want to switch context while the agent finishes up

**Triggers:**
- `take-over` or "take over"
- "Finish up"
- "Clean up and commit"

## Prerequisites

Before triggering take-over, ensure:
- [ ] Core implementation is complete
- [ ] Key design decisions have been made
- [ ] You're on a feature branch (not main)
- [ ] Tests pass (or you've acknowledged known failures)

## Process

### Step 1: Verify State

```bash
git branch --show-current   # Must NOT be main
git status                  # Check what's dirty
cargo test 2>&1 | tail -20  # Quick test check
```

If on main, STOP and create a feature branch first.

### Step 2: Review+Fix

Invoke the `review-fix` skill:
- Capture dirty diff
- Spawn reviewer sub-agent
- Fix issues iteratively
- Continue until LGTM or max 5 cycles

### Step 3: Session Cleanup

Invoke the `session-cleanup` skill:
- Remove debug print statements (dbg!, eprintln! for debug)
- Remove commented-out code
- Remove temporary test files
- Ensure no TODO/FIXME markers without ticket references

### Step 4: Final Test

```bash
cargo test
cargo clippy -- -D warnings
```

If tests fail, fix the failures. Do not commit broken code.

### Step 5: Commit

Stage and commit with a descriptive message:

```bash
git add [specific files]
git commit -m "descriptive message"
```

Follow the project's commit message style (check recent `git log`).

### Step 6: Report

Output a completion summary:

```markdown
## Take-Over Complete

- **Review+fix cycles:** N
- **Cleanup items removed:** X
- **Tests:** passing / N failures
- **Committed:** [commit hash] on branch [branch-name]
- **Ready for:** `git push -u origin [branch-name]` then `gh pr create`
```

Do NOT push or create PR automatically — leave that for the human.

## What This Skill Does NOT Do

- Push to remote (requires explicit human action)
- Create PRs (requires human review of PR description)
- Deploy (out of scope for caro)
- Make architectural decisions (those happen during interactive phase)

## Composability

This skill composes three existing skills:
1. `review-fix` — iterative review loop
2. `session-cleanup` — artifact removal
3. Standard git commit workflow

Each step can also be invoked independently.
