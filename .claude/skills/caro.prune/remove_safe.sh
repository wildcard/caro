#!/usr/bin/env bash
# remove_safe.sh — execute removal of worktrees marked SAFE by audit_wts.sh
#
# Usage: bash remove_safe.sh <worktree-root-subpath>
#
# Reads /tmp/wts_safe.txt produced by audit_wts.sh and removes each entry.
# Probes both <root>/<name> and <root>/pr_review/<name> to handle the
# nested PR-review layout.

set -euo pipefail

WT_ROOT="${1:?usage: remove_safe.sh <worktree-root-subpath>}"
SAFE="${SAFE:-/tmp/wts_safe.txt}"

if [ ! -s "$SAFE" ]; then
  echo "error: $SAFE is empty or missing — run audit_wts.sh first" >&2
  exit 2
fi

REMOVED=0
FAILED=0

while IFS='|' read -r size name br merged remote; do
  hit=0
  for wt in "$WT_ROOT/$name" "$WT_ROOT/pr_review/$name"; do
    if [ -d "$wt" ]; then
      if git worktree remove "$wt" >/dev/null 2>&1; then
        echo "  ✓ removed $wt ($size)"
        REMOVED=$((REMOVED+1))
        hit=1
        break
      fi
    fi
  done
  if [ "$hit" = "0" ]; then
    echo "  ✗ FAILED $name — not found at $WT_ROOT/$name or $WT_ROOT/pr_review/$name"
    FAILED=$((FAILED+1))
  fi
done < "$SAFE"

echo
echo "Summary: removed=$REMOVED failed=$FAILED"

git worktree prune --verbose
