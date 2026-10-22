#!/usr/bin/env bash
# Regression guard for scripts/check-tracked-build-outputs.sh (#1655).
#
# Every case runs in a throwaway git repo under mktemp. Nothing here reads or
# changes the real repository's index or files.
#
# Usage: tests/harness/tracked-build-outputs.test.sh

set -uo pipefail

GUARD="$(cd "$(dirname "$0")/../.." && pwd)/scripts/check-tracked-build-outputs.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

pass=0
fail=0
expect() { # name want got
  if [ "$2" = "$3" ]; then echo "PASS $1"; pass=$((pass + 1))
  else echo "FAIL $1 (want exit $2, got $3)"; fail=$((fail + 1)); fi
}

# A repo with sources, look-alike names that must not be flagged, and the
# same ignore rules as the real .gitignore.
fixture() {
  d=$(mktemp -d "$TMP/repo.XXXXXX")
  cd "$d" || exit 1
  git init -q
  git config user.email t@example.invalid
  git config user.name test
  mkdir -p src tests/evaluation/src docs/targets
  echo 'fn main() {}' > src/main.rs
  echo 'x' > src/target.rs
  echo '[package]' > tests/evaluation/Cargo.toml
  echo 'fn x() {}' > tests/evaluation/src/lib.rs
  echo 'd' > docs/targets/readme.md
  printf '/target/\ntests/evaluation/target/\n' > .gitignore
  git add -A
  git commit -qm base
}

fixture; "$GUARD" >/dev/null 2>&1
expect clean_index_passes 0 $?

fixture; mkdir -p tests/evaluation/target/debug
echo b > tests/evaluation/target/debug/bin
git add -f tests/evaluation/target/debug/bin
"$GUARD" >/dev/null 2>"$TMP/err"
expect force_added_eval_output_fails 1 $?
grep -q '^error: 1 tracked' "$TMP/err"
expect error_counts_paths 0 $?

fixture; mkdir -p crates/new/target/release
echo b > crates/new/target/release/app
git add crates/new
"$GUARD" >/dev/null 2>&1
expect new_crate_target_fails 1 $?

fixture; mkdir -p tests/evaluation/target/debug target/debug
echo b > tests/evaluation/target/debug/bin
echo b > target/debug/caro
git add -A
"$GUARD" >/dev/null 2>&1
expect ignored_untracked_output_passes 0 $?

# --tree checks a commit without touching the index.
fixture; mkdir -p tests/evaluation/target/debug
echo b > tests/evaluation/target/debug/bin
git add -f tests/evaluation/target
git commit -qm bad
git rm -r --cached --quiet tests/evaluation/target
"$GUARD" --tree HEAD >/dev/null 2>&1
expect tree_mode_reads_commit 1 $?
"$GUARD" >/dev/null 2>&1
expect index_after_untrack_passes 0 $?
rc=0; [ -f tests/evaluation/target/debug/bin ] || rc=1
expect untrack_keeps_file_on_disk 0 "$rc"

# #1551: a local worktree added as a gitlink (mode 160000, no .gitmodules)
# breaks `git submodule` in every clone and checkout's post-job cleanup.
fixture; git update-index --add --cacheinfo "160000,$(git rev-parse HEAD),.worktrees/001-feature"
"$GUARD" >/dev/null 2>"$TMP/err"
expect worktree_gitlink_fails 1 $?
grep -q '\.worktrees/001-feature' "$TMP/err"
expect error_names_worktree_path 0 $?
git commit -qm bad
"$GUARD" --tree HEAD >/dev/null 2>&1
expect tree_mode_flags_worktree_gitlink 1 $?

fixture; git update-index --add --cacheinfo "160000,$(git rev-parse HEAD),.claude/worktrees/agent-x"
"$GUARD" >/dev/null 2>&1
expect claude_worktree_gitlink_fails 1 $?

fixture; git update-index --add --cacheinfo "160000,$(git rev-parse HEAD),.worktrees"
"$GUARD" >/dev/null 2>&1
expect worktree_root_gitlink_fails 1 $?

fixture; mkdir -p docs/worktrees && echo d > docs/worktrees/howto.md
git add docs/worktrees
"$GUARD" >/dev/null 2>&1
expect worktrees_lookalike_dir_passes 0 $?

echo "tracked-build-outputs: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
