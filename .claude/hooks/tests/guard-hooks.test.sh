#!/usr/bin/env bash
# Regression guard for the PreToolUse Bash guard hooks.
#
# Claude Code passes hook input as JSON on stdin and only exit code 2 blocks
# a tool call. These tests feed real hook payloads and assert the exit code,
# so a hook that silently stops enforcing (the pre-2026-09 state) fails here.
#
# Usage: .claude/hooks/tests/guard-hooks.test.sh

set -uo pipefail

HOOKS="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

pass=0
fail=0

payload() {
  # payload <tool> <command> <cwd>
  jq -n --arg t "$1" --arg c "$2" --arg d "$3" \
    '{hook_event_name:"PreToolUse", tool_name:$t, tool_input:{command:$c}, cwd:$d}'
}

expect() {
  # expect <exit-code> <hook> <description> <tool> <command> <cwd>
  local want="$1" hook="$2" desc="$3" got
  payload "$4" "$5" "$6" | (cd "$6" && "$HOOKS/$hook" >/dev/null 2>&1)
  got=$?
  if [[ "$got" == "$want" ]]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL [$hook] $desc: want exit $want, got $got" >&2
  fi
}

git_init() {
  git init -q -b main "$1"
  git -C "$1" -c user.email=t@t -c user.name=t commit -q --allow-empty -m init
}

# --- block-main-commits.sh -------------------------------------------------
REPO="$TMP/repo"
git_init "$REPO"
git -C "$REPO" worktree add -q "$REPO/.worktrees/feat" -b feat/x

expect 2 block-main-commits.sh "commit on main is blocked" \
  Bash "git commit -m x" "$REPO"
expect 0 block-main-commits.sh "commit on feature branch is allowed" \
  Bash "git commit -m x" "$REPO/.worktrees/feat"
expect 0 block-main-commits.sh "cd into feature worktree then commit is allowed" \
  Bash "cd .worktrees/feat && git commit -m x" "$REPO"
expect 0 block-main-commits.sh "git -C feature worktree commit is allowed" \
  Bash "git -C .worktrees/feat commit -m x" "$REPO"
expect 0 block-main-commits.sh "non-commit command on main is allowed" \
  Bash "git status" "$REPO"
expect 0 block-main-commits.sh "non-Bash tool is ignored" \
  Write "git commit -m x" "$REPO"

# --- worktree-protection.sh ------------------------------------------------
expect 2 worktree-protection.sh "--force removal is blocked" \
  Bash "git worktree remove --force .worktrees/feat" "$REPO"
expect 2 worktree-protection.sh "-f removal is blocked" \
  Bash "git worktree remove -f .worktrees/feat" "$REPO"
expect 0 worktree-protection.sh "plain removal of a path containing -f is allowed" \
  Bash "git worktree remove .worktrees/foo-feature" "$REPO"
expect 0 worktree-protection.sh "unrelated command is allowed" \
  Bash "ls -f" "$REPO"

# --- block-budget-leaks.sh -------------------------------------------------
LEAKY="$TMP/leaky"
git_init "$LEAKY"
git -C "$LEAKY" remote add origin https://github.com/wildcard/caro.git
mkdir -p "$LEAKY/.beads"
echo 'spent $42 on inference' > "$LEAKY/.beads/notes.md"
git -C "$LEAKY" add .beads/notes.md

CLEAN="$TMP/clean"
git_init "$CLEAN"
git -C "$CLEAN" remote add origin https://github.com/wildcard/caro.git
mkdir -p "$CLEAN/.beads"
echo 'shipped v1.3.0' > "$CLEAN/.beads/notes.md"
git -C "$CLEAN" add .beads/notes.md

expect 2 block-budget-leaks.sh "dollar amount in .beads is blocked" \
  Bash "git commit -m x" "$LEAKY"
expect 0 block-budget-leaks.sh "clean .beads change is allowed" \
  Bash "git commit -m x" "$CLEAN"

echo "guard hooks: $pass passed, $fail failed"
[[ "$fail" == 0 ]]
