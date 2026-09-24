#!/usr/bin/env bash
# PreToolUse hook to block force deletion of git worktrees
# Prevents environment corruption from destroying active worktrees

set -euo pipefail

# Only check Bash tool usage
TOOL_NAME="${CLAUDE_TOOL_NAME:-}"
if [[ "$TOOL_NAME" != "Bash" ]]; then
  exit 0
fi

# Get the command being run
COMMAND="${CLAUDE_TOOL_PARAMS_COMMAND:-}"

# Check if it's a git worktree remove command with --force flag
if [[ "$COMMAND" =~ git[[:space:]]+worktree[[:space:]]+remove ]] && [[ "$COMMAND" =~ (--force|-f) ]]; then
  cat >&2 <<'EOF'

⚠️ BLOCKED: Force-deleting worktree

Git warned about uncommitted changes for a reason!

Force-deleting an active worktree can corrupt the git environment and cause
shell failures. This happened during the PR batch merge session and broke
the entire environment.

**Before removing a worktree:**

1. cd into the worktree
   cd <worktree-path>

2. Check what's uncommitted
   git status

3. Save or discard your work:
   - To save: git stash OR git commit
   - To discard: git checkout . && git clean -fd

4. Return to main repo and remove normally:
   cd /Users/kobik-private/workspace/caro
   git worktree remove <path>  # WITHOUT --force

**Only use --force if:**
- The worktree directory is corrupted/deleted
- You've verified there's nothing to save

See the post-mortem in session transcript for details on why this matters.

EOF
  exit 1
fi

# Allow all other commands
exit 0
