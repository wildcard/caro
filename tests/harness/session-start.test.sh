#!/usr/bin/env bash
# Regression guard for .claude/hooks/session-start.sh (SessionStart readiness).
#
# The hook must: stay silent off-remote, report READY when tools and guard
# hooks are present, report NOT READY (not crash, not block) when a guard hook
# is not executable, and always exit 0.
#
# Usage: tests/harness/session-start.test.sh
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HOOK="$ROOT/.claude/hooks/session-start.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

pass=0
fail=0
check() {
  local desc="$1"
  shift
  if "$@"; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL: $desc" >&2; fi
}

# A minimal project: the guard hooks and lib, plus a stub cargo so the test
# doesn't depend on (or wait for) a real toolchain fetch.
mkdir -p "$TMP/proj/.claude/hooks/lib" "$TMP/bin"
for h in block-main-commits.sh worktree-protection.sh block-budget-leaks.sh; do
  printf '#!/bin/sh\nexit 0\n' > "$TMP/proj/.claude/hooks/$h"
  chmod +x "$TMP/proj/.claude/hooks/$h"
done
touch "$TMP/proj/.claude/hooks/lib/hook-input.sh"
git init -q -b feat "$TMP/proj"
for t in cargo rustfmt cargo-clippy shellcheck; do
  printf '#!/bin/sh\nexit 0\n' > "$TMP/bin/$t"
  chmod +x "$TMP/bin/$t"
done

run() { (cd "$TMP/proj" && env PATH="$TMP/bin:$PATH" CLAUDE_PROJECT_DIR="$TMP/proj" "$@" "$HOOK"); }

out="$(run env -u CLAUDE_CODE_REMOTE)"; rc=$?
check "off-remote exits 0 (rc=$rc)" test "$rc" -eq 0
check "off-remote prints nothing" test -z "$out"

out="$(run env CLAUDE_CODE_REMOTE=true)"; rc=$?
check "remote exits 0 (rc=$rc)" test "$rc" -eq 0
check "remote reports READY (got: $out)" grep -q "caro env: READY (branch: feat)" <<<"$out"

chmod -x "$TMP/proj/.claude/hooks/block-main-commits.sh"
out="$(run env CLAUDE_CODE_REMOTE=true)"; rc=$?
check "non-executable guard still exits 0 (rc=$rc)" test "$rc" -eq 0
check "non-executable guard reports NOT READY" grep -q "NOT READY" <<<"$out"
check "names the broken guard" grep -q "block-main-commits.sh (not executable)" <<<"$out"

echo "session-start: $pass passed, $fail failed"
[[ "$fail" == 0 ]]
