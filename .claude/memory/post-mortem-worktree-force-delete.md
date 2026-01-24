# Post-Mortem: Force Worktree Deletion Environment Corruption

**Date**: 2026-01-24
**Incident**: Shell environment corruption during PR batch merge session
**Impact**: Session became unusable, all bash commands failed with posix_spawn errors
**Prevention**: Implemented `worktree-protection.sh` hook

## Executive Summary

A session attempting to merge 50+ PRs encountered critical environment corruption when force-deleting a worktree that contained uncommitted changes. The force deletion triggered a cascade of hook failures and rendered the shell environment unusable.

## Root Cause Chain

```
1. Created worktree for PR #601 resolution
2. Did rebase, made code edits (JsonSchema fix)
3. Lost directory context → ran commands in main repo instead of worktree
4. Eventually committed/pushed from correct location, merged PR
5. Tried to delete worktree → Git warned: "contains modified or untracked files"
6. FORCE DELETED the worktree anyway (--force flag)
7. posix_spawn '/bin/sh' failure → shell environment corrupted
```

**The Critical Mistake**: When git warned about uncommitted changes, the agent used `--force` to bypass the warning instead of investigating what needed to be saved or discarded.

## Why Force Deletion Is Dangerous

Force-deleting an active worktree can:
- Corrupt git's worktree tracking (`.git/worktrees/`)
- Leave orphaned references
- Trigger cascade of hook failures
- Break the shell environment completely

## Prevention Mechanism

Created `.claude/hooks/worktree-protection.sh` PreToolUse hook that:

1. **Blocks** `git worktree remove --force` and `git worktree remove -f`
2. **Provides guidance** on proper worktree cleanup procedure
3. **Forces investigation** of uncommitted changes before deletion

## Proper Worktree Cleanup Procedure

**When git warns about uncommitted changes:**

```bash
# 1. Investigate what's uncommitted
cd <worktree-path>
git status

# 2. Save or discard the changes
# Option A: Save the work
git stash
# OR
git commit -m "WIP: save before cleanup"

# Option B: Discard the changes
git checkout .      # Discard modified files
git clean -fd       # Remove untracked files

# 3. Return to main repo and remove normally
cd /Users/kobik-private/workspace/caro
git worktree remove <path>  # WITHOUT --force
```

## Secondary Issues Identified

### 1. Working Directory Confusion
**Problem**: Commands ran in main repo instead of worktree after checkout.

**Prevention**: Always use absolute paths with cd:
```bash
cd /Users/kobik-private/workspace/caro/.worktrees/NNN-feature/
```

### 2. Overly Aggressive Conflict Resolution
**Problem**: Used `git checkout --ours` without reviewing incoming changes.

**Prevention**: Always read conflict markers before resolving.

### 3. No Verification Checkpoints
**Problem**: No systematic verification between operations.

**Prevention**: After rebase, run `cargo check` before pushing.

## Recommended Workflow for PR Batch Operations

### Phase 1: Triage (Read-Only)
```bash
gh pr list --json number,mergeable,mergeStateStatus,title
# Categorize: CLEAN, CONFLICTING, UNSTABLE
```

### Phase 2: Merge Clean PRs (Small Batches)
```bash
# Merge in batches of 5-10
for pr in 601 602 603; do
  gh pr merge $pr --squash
done

# Verify after each batch
git pull origin main
cargo check
```

### Phase 3: Resolve Conflicts (One at a Time)
```bash
# 1. Create worktree with ABSOLUTE path
WORKTREE_PATH="/Users/kobik-private/workspace/caro/.worktrees/resolve-pr-NNN"
git worktree add "$WORKTREE_PATH" -b resolve-pr-NNN origin/main

# 2. Stay in the worktree
cd "$WORKTREE_PATH"
gh pr checkout NNN

# 3. Rebase and resolve
git fetch origin main
git rebase origin/main
# Manually resolve conflicts

# 4. Verify BEFORE pushing
cargo check
pwd && git branch --show-current

# 5. Push and merge
git push --force-with-lease
gh pr merge NNN --squash

# 6. Clean up (git will warn if uncommitted changes exist)
cd /Users/kobik-private/workspace/caro
git worktree remove "$WORKTREE_PATH"
```

## Key Learnings

| Issue | Prevention | Enforcement |
|-------|------------|-------------|
| Force-deleting active worktree | Never `--force` without investigating | `worktree-protection.sh` hook blocks it |
| Directory confusion | Stay in worktree until work complete | Use absolute paths |
| Blind conflict resolution | Review both sides before resolving | Training/discipline |
| Cascade merge errors | `cargo check` after batches | Process discipline |

## Hook Implementation

**File**: `.claude/hooks/worktree-protection.sh`
**Type**: PreToolUse hook on Bash tool
**Registration**: Added to `.claude/settings.json`

**Behavior**:
- Detects `git worktree remove` with `--force` or `-f` flag
- Blocks the command (exit code 1)
- Provides detailed guidance on proper cleanup procedure

**Testing**:
```bash
# Blocked (exit 1)
git worktree remove --force .worktrees/test
git worktree remove -f .worktrees/test

# Allowed (exit 0)
git worktree remove .worktrees/test
```

## Philosophy

**Rules are passive documentation. Hooks inject context at decision points when the agent is about to make a mistake.**

Git warnings exist for a reason. `--force` flags should trigger investigation, not bypass the safety mechanism.

---

*This post-mortem documents the incident and prevention mechanisms. The worktree-protection hook prevents recurrence.*
