#!/usr/bin/env bash
# PreToolUse hook to block force deletion of git worktrees
# Prevents environment corruption from destroying active worktrees

set -euo pipefail

source "$(dirname "$0")/lib/hook-input.sh"

# Only check Bash tool usage
if [[ "$TOOL_NAME" != "Bash" ]]; then
  exit 0
fi

# Check if it's a git worktree remove command with --force flag
if is_git_cmd worktree remove && [[ "$COMMAND" =~ (^|[[:space:]])(--force|-f+)([[:space:]]|$) ]]; then
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
   cd "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"
   git worktree remove <path>  # WITHOUT --force

**Only use --force if:**
- The worktree directory is corrupted/deleted
- You've verified there's nothing to save

See the post-mortem in session transcript for details on why this matters.

EOF
  exit "$BLOCK"
fi

# Allow all other commands
exit 0
