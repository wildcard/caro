#!/usr/bin/env bash
# audit_wts.sh — audit caro git worktrees for safe removal
#
# Usage: bash audit_wts.sh <worktree-root-subpath>
#   e.g.  bash audit_wts.sh .claude/worktrees
#         bash audit_wts.sh .worktrees
#
# Writes:
#   /tmp/wts_safe.txt  — worktrees safe to remove (pipe-delimited)
#   /tmp/wts_keep.txt  — worktrees that need user review
#
# Safety invariants — never marks safe:
#   1. Locked worktrees
#   2. Worktrees with uncommitted changes (`git status --porcelain` non-empty)
#   3. Worktrees whose HEAD is unreachable from origin/main AND not on any remote
#
# See SKILL.md in the same directory for the full procedure.

set -euo pipefail

WT_ROOT="${1:?usage: audit_wts.sh <worktree-root-subpath>}"
MAIN_REF="${MAIN_REF:-origin/main}"

if ! MAIN=$(git rev-parse "$MAIN_REF" 2>/dev/null); then
  echo "error: cannot resolve $MAIN_REF — did you 'git fetch origin'?" >&2
  exit 2
fi

LIST=/tmp/wts_list.txt
SAFE=/tmp/wts_safe.txt
KEEP=/tmp/wts_keep.txt

git worktree list --porcelain \
  | awk '/^worktree/ {wt=$2}
         /^branch/   {br=$2}
         /^detached/ {br="DETACHED"}
         /^locked/   {locked=1}
         /^$/        {print wt"|"br"|"locked; wt=""; br=""; locked=0}
         END         {if(wt) print wt"|"br"|"locked}' \
  | grep "/${WT_ROOT}/" > "$LIST"

: > "$SAFE"
: > "$KEEP"

while IFS='|' read -r wt br locked; do
  name=$(basename "$wt")
  if [ "$locked" = "1" ]; then
    echo "LOCKED|$name|$br" >> "$KEEP"
    continue
  fi
  br_short=${br#refs/heads/}
  dirty=$(git -C "$wt" status --porcelain 2>/dev/null | wc -l | tr -d ' ')
  head=$(git -C "$wt" rev-parse HEAD 2>/dev/null || echo "")
  if [ -z "$head" ]; then
    echo "size=? dirty=? merged=? remote=? UNREADABLE | $name | $br_short" >> "$KEEP"
    continue
  fi
  merged="no"
  git merge-base --is-ancestor "$head" "$MAIN" 2>/dev/null && merged="yes"
  on_remote=$(git branch -r --contains "$head" 2>/dev/null | head -1 | tr -d ' ')
  size=$(du -sh "$wt" 2>/dev/null | cut -f1)

  if [ "$dirty" = "0" ] && { [ "$merged" = "yes" ] || [ -n "$on_remote" ]; }; then
    echo "$size|$name|$br_short|merged=$merged|remote=$on_remote" >> "$SAFE"
  else
    echo "size=$size dirty=$dirty merged=$merged remote=$on_remote | $name | $br_short" >> "$KEEP"
  fi
done < "$LIST"

echo "=== SAFE TO REMOVE ($(wc -l < "$SAFE" | tr -d ' ')) ==="
cat "$SAFE"
echo
echo "=== KEEP/REVIEW ($(wc -l < "$KEEP" | tr -d ' ')) ==="
cat "$KEEP"
echo
echo "Next step: review $SAFE then run:"
echo "  bash .claude/skills/caro.prune/remove_safe.sh $WT_ROOT"
