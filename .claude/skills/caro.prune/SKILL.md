---
name: caro.prune
description: Safely reclaim disk space in the caro project. Cleans Rust build cache, stale git worktrees (both .claude/worktrees/ and .worktrees/), empty stubs, and node_modules using a tiered audit that preserves any worktree with uncommitted changes, missing remotes, or a lock file. Use when du shows the project > 30 GB or disk free is low.
---

# caro.prune Skill

**Purpose:** Reclaim disk space without losing any work. Caro's worktree-heavy
workflow + Rust build caches grow the repo to 100 GB+ within weeks of active
development. This skill turns that back into ~5–15 GB while preserving every
worktree that has dirty files, an unpushed branch, or a `locked` flag.

**Codified from:** the 2026-05-16 cleanup pass that reduced the project from
**101 GB → 18 GB** in ~5 min by removing 53 stale worktrees + the entire
`target/` cache.

---

## When to Use

- `du -sh .` shows the project > 30 GB
- `df -h .` shows < 100 GB free on the volume
- Before a long session where the repo will be growing (releases, big refactors)
- After finishing a multi-week sprint with many merged PRs (their worktrees linger)
- Periodic hygiene — once a month is a good cadence

## When NOT to Use

- Mid-build (`cargo build` running) — Tier 1 will wipe the cache out from under it
- Right before a benchmark run — you want the warm cache
- If `git worktree list` shows < 10 worktrees — not enough fat to be worth the audit
- On a worktree branch that isn't `main` and isn't your own — leave the parent alone

---

## Safety Invariants

The skill **never** removes:
1. A worktree marked `locked` (held by another session, cua-driver, etc.)
2. A worktree with **any** uncommitted changes (`git status --porcelain` non-empty)
3. A worktree whose `HEAD` is **not** reachable from `origin/main` AND **not**
   contained in **any** remote branch (i.e. work that exists nowhere else)
4. The main checkout at `/Users/kobik-private/workspace/caro`
5. Files outside the project root

If you want to remove a flagged worktree anyway, **do it manually after
verifying with the user** — never extend the heuristic to be more aggressive.

---

## The Five Tiers

### Tier 1 — `cargo clean` (~60–70 GB, zero audit, 1 cmd)

```bash
cargo clean
```

Rebuild after: `cargo build` takes ~3–5 min for debug, ~5–8 min for release on
M-series Apple Silicon.

### Tier 2 — Prune `.claude/worktrees/` (~15–18 GB)

These are ephemeral session worktrees from Claude Code's worktree integration.
Most are named `claude/<adjective-name-hash>` and tied to merged or pushed
branches. Run the audit script in `scripts/audit_worktrees.sh` (below) targeting
`.claude/worktrees/`.

### Tier 3 — Prune `.worktrees/` (~7–10 GB)

Feature/PR worktrees. Same audit, different path. Watch for nested worktrees
under `.worktrees/pr_review/<N>/` — they're listed as `.worktrees/<N>` by
`basename` but actually live one level deeper.

### Tier 4 — Empty stubs (0 GB, cosmetic)

```bash
find .worktrees -maxdepth 1 -type d -empty -not -path .worktrees -exec rmdir {} +
```

Leftover empty dirs from previous `git worktree remove` calls.

### Tier 5 — `node_modules/` (~1 GB)

```bash
rm -rf node_modules        # reinstall with `pnpm install` when needed
```

---

## The Audit Heuristic (run this before any worktree removal)

Save the following as `/tmp/audit_wts.sh` or paste into Bash directly. It
takes the worktree root (`.claude/worktrees` or `.worktrees`) as `$1` and
writes two files:

- `/tmp/wts_safe.txt` — pipe-delimited list of worktrees safe to remove
- `/tmp/wts_keep.txt` — flagged for user review (dirty / unmerged / locked)

```bash
WT_ROOT="${1:?usage: audit_wts.sh <worktree-root>}"
MAIN=$(git rev-parse origin/main)

git worktree list --porcelain \
  | awk '/^worktree/ {wt=$2}
         /^branch/   {br=$2}
         /^detached/ {br="DETACHED"}
         /^locked/   {locked=1}
         /^$/        {print wt"|"br"|"locked; wt=""; br=""; locked=0}
         END         {if(wt) print wt"|"br"|"locked}' \
  | grep "/${WT_ROOT}/" > /tmp/wts_list.txt

: > /tmp/wts_safe.txt
: > /tmp/wts_keep.txt

while IFS='|' read -r wt br locked; do
  name=$(basename "$wt")
  if [ "$locked" = "1" ]; then
    echo "LOCKED|$name|$br" >> /tmp/wts_keep.txt
    continue
  fi
  br_short=${br#refs/heads/}
  dirty=$(git -C "$wt" status --porcelain 2>/dev/null | wc -l | tr -d ' ')
  head=$(git -C "$wt" rev-parse HEAD 2>/dev/null)
  merged="no"
  git merge-base --is-ancestor "$head" "$MAIN" 2>/dev/null && merged="yes"
  on_remote=$(git branch -r --contains "$head" 2>/dev/null | head -1 | tr -d ' ')
  size=$(du -sh "$wt" 2>/dev/null | cut -f1)

  if [ "$dirty" = "0" ] && { [ "$merged" = "yes" ] || [ -n "$on_remote" ]; }; then
    echo "$size|$name|$br_short|merged=$merged|remote=$on_remote" >> /tmp/wts_safe.txt
  else
    echo "size=$size dirty=$dirty merged=$merged remote=$on_remote | $name | $br_short" >> /tmp/wts_keep.txt
  fi
done < /tmp/wts_list.txt

echo "=== SAFE TO REMOVE ($(wc -l < /tmp/wts_safe.txt)) ==="
cat /tmp/wts_safe.txt
echo
echo "=== KEEP/REVIEW ($(wc -l < /tmp/wts_keep.txt)) ==="
cat /tmp/wts_keep.txt
```

### Execute removal (after reviewing `/tmp/wts_safe.txt`)

```bash
while IFS='|' read -r size name br merged remote; do
  # try the literal path; if that fails, try nested (pr_review/N)
  for wt in "$WT_ROOT/$name" "$WT_ROOT/pr_review/$name"; do
    if [ -d "$wt" ] && git worktree remove "$wt" 2>/dev/null; then
      echo "  ✓ removed $wt ($size)"; break
    fi
  done
done < /tmp/wts_safe.txt
```

---

## Standard Run (copy-paste)

```bash
# Baseline measurement
du -sh . && df -h . | tail -1

# Tier 1
cargo clean

# Refresh origin so the audit sees current remote state
git fetch origin --quiet

# Tier 2
bash <(cat .claude/skills/caro.prune/audit_wts.sh) .claude/worktrees
# review /tmp/wts_safe.txt then execute the removal loop above
mv /tmp/wts_safe.txt /tmp/safe_claude.txt
mv /tmp/wts_keep.txt /tmp/keep_claude.txt

# Tier 3
bash <(cat .claude/skills/caro.prune/audit_wts.sh) .worktrees
mv /tmp/wts_safe.txt /tmp/safe_legacy.txt
mv /tmp/wts_keep.txt /tmp/keep_legacy.txt

# Tier 4
find .worktrees -maxdepth 1 -type d -empty -not -path .worktrees -exec rmdir {} +

# Tier 5
rm -rf node_modules

# Clean up the git refs left behind
git worktree prune --verbose

# Verify
du -sh . && df -h . | tail -1
git worktree list | wc -l
```

---

## Known Quirks

- **APFS Time Machine snapshots delay the disk-free gain.** macOS keeps deleted
  blocks in `com.apple.TimeMachine.YYYY-MM-DD-HHMMSS.local` snapshots for up to
  24h. `du -sh .` will show the reclaim immediately; `df -h .` won't. To force:
  `sudo tmutil deletelocalsnapshots /`. Don't panic — the space comes back.
- **`grep -v ""` returns exit 1 when stdin is all blank lines.** If you wrap
  `git worktree remove` output in `if ... | grep ...; then`, the success branch
  will fire only when grep finds *something*. Use `>/dev/null 2>&1` instead.
- **Squash-merged branches aren't ancestors of `origin/main`.** The audit's
  fallback (`git branch -r --contains HEAD`) catches them only if the branch
  was still pushed to origin under its own name. A squash-merged branch that
  was deleted from origin will be flagged as "unmerged" — that's intentional
  and conservative.
- **Nested worktrees.** `git worktree list` shows `/Users/.../caro/.worktrees/pr_review/333`
  but `basename` returns `333`. Always probe both `$WT_ROOT/$name` and
  `$WT_ROOT/pr_review/$name` when removing.
- **Parallel sessions create new worktrees during the run.** Don't assume the
  count is stable. The audit is a snapshot; new worktrees added afterward
  simply wait for the next run.

---

## Reclaim Expectations

Approximate reclaim for a project that hasn't been pruned in ~30 days of
active multi-session development:

| Tier | Reclaim | Time | Risk |
|------|---------|------|------|
| 1 (cargo clean) | 60–70 GB | 1 cmd | None |
| 2 (.claude/worktrees) | 10–18 GB | ~3 min | Low |
| 3 (.worktrees) | 5–10 GB | ~3 min | Low |
| 4 (empty stubs) | 0 GB | trivial | None |
| 5 (node_modules) | 0.5–1.5 GB | 1 cmd | None |
| **Total** | **~75–100 GB** | < 10 min | |

The reclaim on the first run after long neglect can exceed 80 GB. Steady-state
weekly runs will reclaim 10–30 GB (mostly the build cache).

---

## Files Referenced

- `~/.claude/rules/destructive-commands.md` — the rule this skill operates under
- `.claude/rules/git-workflow.md` — never run on `main` branch
- `Cargo.toml` — Rust manifest (cargo clean target)
- `.git/worktrees/` — the authoritative worktree ref store (cleaned by `git worktree prune`)

## See Also

- `cleanup-session` skill — single-worktree cleanup after a session ends
- The 2026-05-16 cleanup plan: `~/.claude/plans/let-s-analze-the-space-majestic-squid.md`
