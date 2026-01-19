#!/usr/bin/env bash
# PreToolUse hook to block git commits on main branch
# Enforces feature branch workflow to prevent conflicts across parallel sessions

set -euo pipefail

# Only check Bash tool usage
TOOL_NAME="${CLAUDE_TOOL_NAME:-}"
if [[ "$TOOL_NAME" != "Bash" ]]; then
  exit 0
fi

# Get the command being run
COMMAND="${CLAUDE_TOOL_PARAMS_COMMAND:-}"

# Check if it's a git commit command
if [[ ! "$COMMAND" =~ git[[:space:]]+commit ]]; then
  exit 0
fi

# Check current branch
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")

# Block commits on main branch
if [[ "$CURRENT_BRANCH" == "main" ]]; then
  cat >&2 <<'EOF'

❌ BLOCKED: Cannot commit directly to main branch

You are attempting to commit on the main branch, which is not allowed.
The user runs 4-5 parallel Claude sessions and direct commits to main
cause constant merge conflicts.

Required workflow:
1. Create a feature branch with worktree:
   bin/sk-new-feature "description"
   cd .worktrees/NNN-feature-name/

2. OR create a hotfix branch:
   git worktree add .worktrees/hotfix-name -b hotfix/name
   cd .worktrees/hotfix-name/

3. Make changes and commit in the worktree
4. Push branch and create PR
5. Merge via GitHub after review

See .claude/rules/git-workflow.md for details.

EOF
  exit 1
fi

# Allow commits on feature/hotfix branches
exit 0
