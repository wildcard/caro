#!/usr/bin/env bash
# SessionStart (startup) readiness check for Claude Code on the web.
#
# Modeled on google/ax's runner /readyz: don't let an agent start work in an
# environment that only *looks* ready. Pre-fetch crates (cached with the
# container), then verify the tools the repo's checks depend on and print one
# READY / NOT READY summary into the session context. Never blocks startup.
#
# Remote-only: local machines manage their own toolchains.
set -uo pipefail

if [[ "${CLAUDE_CODE_REMOTE:-}" != "true" ]]; then
  exit 0
fi

cd "${CLAUDE_PROJECT_DIR:-$(pwd)}" || exit 0

missing=()
notes=()

need() {
  # need <command> <why>
  command -v "$1" >/dev/null 2>&1 || missing+=("$1 ($2)")
}

need cargo "build and test"
need rustfmt "cargo fmt --check"
need cargo-clippy "cargo clippy"
need git "branch and PR workflow"
need shellcheck "ShellCheck CI job"
need python3 "hook JSON fallback"
command -v jq >/dev/null 2>&1 || notes+=("jq missing: guard hooks fall back to python3")

# Pre-fetch dependencies so builds and tests don't stall on the network later.
# `cargo fetch` is idempotent and its cache persists with the container.
if command -v cargo >/dev/null 2>&1; then
  if ! cargo fetch --quiet >/dev/null 2>&1; then
    notes+=("cargo fetch failed: first build will download crates")
  fi
fi

# The PreToolUse guards only protect anything if they can actually run.
for hook in block-main-commits.sh worktree-protection.sh block-budget-leaks.sh; do
  [[ -x ".claude/hooks/$hook" ]] || missing+=(".claude/hooks/$hook (not executable)")
done
[[ -f .claude/hooks/lib/hook-input.sh ]] || missing+=(".claude/hooks/lib/hook-input.sh (guard input parser)")

branch="$(git branch --show-current 2>/dev/null || echo unknown)"
[[ "$branch" == "main" ]] && notes+=("on main: create a feature branch before committing")

if ((${#missing[@]} == 0)); then
  echo "caro env: READY (branch: $branch). Checks: cargo fmt --check; cargo clippy --no-default-features --features embedded-cpu -- -D warnings; cargo test --no-default-features --features embedded-cpu --lib"
else
  echo "caro env: NOT READY (branch: $branch). Missing:"
  printf '  - %s\n' "${missing[@]}"
fi
if ((${#notes[@]} > 0)); then
  printf '  note: %s\n' "${notes[@]}"
fi
exit 0
