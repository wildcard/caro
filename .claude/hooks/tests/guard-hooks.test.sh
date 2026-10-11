#!/usr/bin/env bash
# Regression guard for the PreToolUse Bash guard hooks.
#
# Claude Code passes hook input as JSON on stdin and only exit code 2 blocks
# a tool call. These tests feed real hook payloads and assert the exit code,
# so a hook that silently stops enforcing (the pre-2026-09 state) fails here.
#
# Usage: .claude/hooks/tests/guard-hooks.test.sh
#        CARO_HOOKS_NO_JQ=1 .claude/hooks/tests/guard-hooks.test.sh  # python3 path

set -uo pipefail

HOOKS="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

pass=0
fail=0

payload() {
  # payload <tool> <command> <cwd>
  if [[ -z "${CARO_HOOKS_NO_JQ:-}" ]] && command -v jq >/dev/null 2>&1; then
    jq -n --arg t "$1" --arg c "$2" --arg d "$3" \
      '{hook_event_name:"PreToolUse", tool_name:$t, tool_input:{command:$c}, cwd:$d}'
  else
    python3 -c 'import json,sys
t,c,d=sys.argv[1:4]
print(json.dumps({"hook_event_name":"PreToolUse","tool_name":t,"tool_input":{"command":c},"cwd":d}))' "$1" "$2" "$3"
  fi
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
expect 2 block-main-commits.sh "git -C <main checkout> commit is blocked" \
  Bash "git -C $REPO commit -m x" "$REPO/.worktrees/feat"
expect 2 block-main-commits.sh "-C on an earlier git command doesn't redirect the commit" \
  Bash "git -C .worktrees/feat status && git commit -m x" "$REPO"

SPACED="$TMP/repo with spaces"
git_init "$SPACED"
expect 2 block-main-commits.sh "cd into a quoted path on main then commit is blocked" \
  Bash "cd \"$SPACED\" && git commit -m x" "$REPO/.worktrees/feat"
expect 2 block-main-commits.sh "git -C quoted path on main commit is blocked" \
  Bash "git -C '$SPACED' commit -m x" "$REPO/.worktrees/feat"

# --- worktree-protection.sh ------------------------------------------------
expect 2 worktree-protection.sh "--force removal is blocked" \
  Bash "git worktree remove --force .worktrees/feat" "$REPO"
expect 2 worktree-protection.sh "-f removal is blocked" \
  Bash "git worktree remove -f .worktrees/feat" "$REPO"
expect 2 worktree-protection.sh "-ff removal is blocked" \
  Bash "git worktree remove -ff .worktrees/feat" "$REPO"
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

PLUSSY="$TMP/plussy"
git_init "$PLUSSY"
git -C "$PLUSSY" remote add origin https://github.com/wildcard/caro.git
mkdir -p "$PLUSSY/.beads"
echo '++ spent $42' > "$PLUSSY/.beads/notes.md"
git -C "$PLUSSY" add .beads/notes.md

CLEAN="$TMP/clean"
git_init "$CLEAN"
git -C "$CLEAN" remote add origin https://github.com/wildcard/caro.git
mkdir -p "$CLEAN/.beads"
echo 'shipped v1.3.0' > "$CLEAN/.beads/notes.md"
git -C "$CLEAN" add .beads/notes.md

expect 2 block-budget-leaks.sh "dollar amount in .beads is blocked" \
  Bash "git commit -m x" "$LEAKY"
expect 2 block-budget-leaks.sh "added line starting with ++ is still scanned" \
  Bash "git commit -m x" "$PLUSSY"
expect 2 block-budget-leaks.sh "git -C <dir> commit is scanned" \
  Bash "git -C $LEAKY commit -m x" "$TMP"
expect 0 block-budget-leaks.sh "clean .beads change is allowed" \
  Bash "git commit -m x" "$CLEAN"

# A merge stages lines the other branch already committed. Lines that are
# already on the remote are public, so only lines this side adds are new.
# Regression: merging origin/main into #1478 was blocked by main's
# "~1 GB default model" memory note.
MERGING="$TMP/merging"
git_init "$MERGING"
git -C "$MERGING" remote add origin https://github.com/wildcard/caro.git
mkdir -p "$MERGING/.claude/memory"
echo 'downloads the ~1 GB default model' > "$MERGING/.claude/memory/notes.md"
git -C "$MERGING" add .claude/memory/notes.md
git -C "$MERGING" -c user.email=t@t -c user.name=t commit -q -m note
git -C "$MERGING" update-ref refs/remotes/origin/main main  # main is pushed
git -C "$MERGING" checkout -q -b feat/y HEAD~1
git -C "$MERGING" -c user.email=t@t -c user.name=t merge -q --no-ff --no-commit main >/dev/null 2>&1

expect 0 block-budget-leaks.sh "line merged in from the other branch is allowed" \
  Bash "git commit -m x" "$MERGING"
mkdir -p "$MERGING/.beads"
echo 'spent $42 on inference' > "$MERGING/.beads/notes.md"
git -C "$MERGING" add .beads/notes.md
expect 2 block-budget-leaks.sh "line this side adds during a merge is blocked" \
  Bash "git commit -m x" "$MERGING"

# An unpushed branch is not public, and its commits may never have passed this
# hook (a commit made outside Claude skips it), so a merge scans its lines.
TOPIC="$TMP/topic"
git_init "$TOPIC"
git -C "$TOPIC" remote add origin https://github.com/wildcard/caro.git
git -C "$TOPIC" checkout -q -b topic
mkdir -p "$TOPIC/.beads"
echo 'spent $42 on inference' > "$TOPIC/.beads/notes.md"
git -C "$TOPIC" add .beads/notes.md
git -C "$TOPIC" -c user.email=t@t -c user.name=t commit -q -m unscanned
git -C "$TOPIC" checkout -q -b feat/z main
git -C "$TOPIC" -c user.email=t@t -c user.name=t merge -q --no-ff --no-commit topic >/dev/null 2>&1
expect 2 block-budget-leaks.sh "line merged in from an unpushed branch is blocked" \
  Bash "git commit -m x" "$TOPIC"

echo "guard hooks: $pass passed, $fail failed"
[[ "$fail" == 0 ]]
