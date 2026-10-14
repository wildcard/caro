#!/usr/bin/env bash
# Fail if git tracks a Cargo build output: any path with a `target/`
# directory component. Build outputs belong in .gitignore, not the index.
# #1655: 669 files under tests/evaluation/target/ were tracked by mistake.
#
# Usage: scripts/check-tracked-build-outputs.sh            # checks the index
#        scripts/check-tracked-build-outputs.sh --tree REV # checks a commit
set -euo pipefail

if [ "${1:-}" = "--tree" ]; then
  paths=$(git ls-tree -r -z --name-only "${2:?usage: --tree REV}" | tr '\0' '\n')
else
  paths=$(git ls-files -z | tr '\0' '\n')
fi

bad=$(printf '%s\n' "$paths" | grep -E '(^|/)target/' || true)
if [ -n "$bad" ]; then
  n=$(printf '%s\n' "$bad" | wc -l | tr -d ' ')
  echo "error: $n tracked build-output path(s) under a target/ directory:" >&2
  printf '%s\n' "$bad" | head -5 >&2
  echo "fix: git rm -r --cached <dir>  (keeps the files on disk), then add <dir> to .gitignore" >&2
  exit 1
fi
echo "ok: no tracked target/ build outputs"
