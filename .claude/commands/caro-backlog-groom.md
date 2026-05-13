---
description: Scan GH issues / PRs / branches / recent Claude-Code commits and sync them into the beads queue for v1.2.0. Idempotent — safe to run on any cadence.
---

# Caro Backlog Grooming

Refreshes the beads queue (`.beads/beads.db`) against GitHub and git state. Runs every 6 hours via scheduled task `caro-backlog-grooming`, or manually via `/caro-backlog-groom`.

**Design goal**: `bd ready` should always reflect the *latest* real-world queue. No orphaned tasks, no duplicates, no stale priorities.

## Preflight

```bash
cd /Users/kobik-private/workspace/caro
git fetch origin --prune --tags --quiet
bd stats
```

## Phase A — Scan sources (parallel where possible)

Produce four JSON streams:

### A1. Open v1.2.0 GH issues
```bash
gh issue list \
  --state open \
  --milestone "v1.2.0 - Global Launch" \
  --limit 200 \
  --json number,title,labels,updatedAt,body \
  > /tmp/groom-issues.json
```

### A2. Open PRs
```bash
gh pr list \
  --state open \
  --limit 200 \
  --json number,title,headRefName,updatedAt,author,labels,milestone \
  > /tmp/groom-prs.json
```

### A3. Remote branches (possible WIP)
```bash
git ls-remote --heads origin | \
  awk '{print $2}' | \
  sed 's|refs/heads/||' | \
  grep -E '^(feat|fix|docs|feature|chore)/' \
  > /tmp/groom-branches.txt
```

### A4. Claude-Code-authored commits in last 6 hours
```bash
git log --all --since="6 hours ago" \
  --pretty=format:'%H|%an|%ae|%s|%D' | \
  grep -iE 'claude|co-authored-by.*claude' \
  > /tmp/groom-claude-commits.txt || true
```

### A5. Current beads state
```bash
bd list --status open --json > /tmp/groom-beads.json
```

## Phase B — Reconcile (create / update / dedup)

For each scanned item, decide: `CREATE`, `UPDATE`, `SKIP`, or `DEDUP`.

**Dedup key**: `external_ref` field on the beads task (format `gh-<issue_number>` or `branch-<branch_name>`).

### B1. GH issues → beads tasks

```python
# Pseudocode — actual impl inline
for issue in groom_issues.json:
    existing = bd_find_by_external_ref(f"gh-{issue.number}")
    priority = derive_priority(issue.labels)  # P0→0, security→0, P1→1, docs→3
    phase = derive_phase_from_milestone_or_labels(issue)
    # Compute STAKEHOLDERS.yml suggestions up-front so the coder-loop
    # doesn't have to re-derive them on every claim.
    suggested_agents = stakeholders_lookup(issue.body, issue.title)

    if not existing:
        bd create <issue.title> \
          --type feature \
          --priority $priority \
          --parent caro-xk0 \
          --external-ref gh-$issue.number \
          --labels "v1.2.0,$phase,auto-groomed" \
          --metadata "suggested_agents=$suggested_agents" \
          --description "GH: https://github.com/wildcard/caro/issues/$issue.number"
        log_create(issue.number, beads_id)
    elif existing.title != issue.title or existing.priority != priority:
        bd update $existing.id --title "$issue.title" --priority $priority \
          --metadata "suggested_agents=$suggested_agents"
        log_update(issue.number, existing.id)
```

**`stakeholders_lookup` reference implementation**:

```bash
stakeholders_lookup() {
  local body="$1" title="$2"
  # Extract candidate paths from the issue body + title
  local paths=$(printf '%s\n%s' "$body" "$title" \
    | grep -oE '(src/[a-z_/]+|tests/[a-z_]+|website/[^ ]+|\.github/[^ ]+|Cargo\.[a-z]+)' \
    | sort -u)
  # For each path, find the longest STAKEHOLDERS glob match. Emit a
  # comma-separated agent list, deduped, longest-prefix-wins.
  for p in $paths; do
    yq -r --arg p "$p" '
      .areas | to_entries[]
      | select(.key as $k | $p | test($k))
      | (.key | length) as $score
      | "\($score)|\(.value.agents | join(\",\"))"
    ' .github/STAKEHOLDERS.yml
  done | sort -rn | head -1 | cut -d'|' -f2
}
```

If no path match is found, fall back to the legacy label heuristic
(`documentation` → `spark`; otherwise `kraken`).

### B2. Pushed branches without PR → WIP beads task

```python
branch_set = set(groom_branches.txt)
pr_branch_set = {pr.headRefName for pr in groom_prs.json}
orphan_branches = branch_set - pr_branch_set

for branch in orphan_branches:
    existing = bd_find_by_external_ref(f"branch-{branch}")
    last_commit_subject = git log -1 origin/$branch --pretty=format:'%s'
    commit_count = git rev-list --count origin/main..origin/$branch

    if not existing and commit_count > 0:
        bd create "WIP: $last_commit_subject" \
          --type chore \
          --priority 3 \
          --external-ref branch-$branch \
          --labels "wip,auto-groomed,branch-$branch" \
          --description "Untracked WIP on branch '$branch' ($commit_count commits). Needs PR or cleanup."
```

### B3. Claude-Code-authored commits → trace to existing PR/branch

For each commit in `/tmp/groom-claude-commits.txt`:
- If commit is on a branch with a PR → already tracked, `SKIP`.
- If commit is on an orphan branch → covered by B2.
- If commit is on main → flag in delta report as "unexpected main commit" (should have been a PR).

### B4. Close beads tasks for merged PRs

```python
for pr in gh pr list --state merged --search "merged:>$(date -v-6H -u +%Y-%m-%dT%H:%M:%SZ)":
    # Parse "closes #N" from PR body → find beads task with gh-N external ref
    for issue_num in closed_issues_from_pr(pr):
        bead = bd_find_by_external_ref(f"gh-{issue_num}")
        if bead and bead.status != "closed":
            bd close $bead.id --notes "PR #$pr.number merged"
```

### B5. Detect duplicate beads tasks

Group open tasks by normalized title. If 2+ open tasks with same title and no dedup link:
```bash
bd dep $dup1 --blocks $dup2   # or use bd relate
# Close the auto-groomed one, keep the manually-curated one
```

### B6. Repair stranded canonical beads (post-dedup)

When a prior dedup event closed a duplicate bead, the dupe **retains its
`external_ref`** while the canonical (open) bead has none. The canonical
becomes structurally invisible to ref-keyed scans, so B2 won't find it
and will either silently miss work or fail to create a fresh WIP bead
with `UNIQUE constraint failed: issues.external_ref`.

**Detection** (run as part of every cycle, not just on `UNIQUE` failures):

```sql
-- Closed beads holding a branch-* ref where the branch is still on origin
SELECT
  closed.id        AS closed_dupe_id,
  closed.external_ref AS stranded_ref,
  closed.close_reason
FROM issues closed
WHERE closed.status = 'closed'
  AND closed.external_ref LIKE 'branch-%'
  AND closed.close_reason LIKE '%dedup%';
```

For each row, the `close_reason` typically names the canonical
(e.g. `"dedup: caro-ai5 is canonical WIP for docs/adr-008"`). Parse it
to identify the open canonical.

**Repair** (atomic SQL batch — required because of upstream bug
[gastownhall/beads#3902](https://github.com/gastownhall/beads/issues/3902)):

```bash
# Step 1 — clear ALL stranded refs in one transaction.
# Why SQL: `bd update --external-ref ""` writes empty string (not NULL),
# and UNIQUE applies to ''. Second CLI clear fails. SQL NULL bypasses this.
sqlite3 .beads/beads.db <<'SQL'
BEGIN;
UPDATE issues SET external_ref = NULL
WHERE id IN ('caro-u7j','caro-6bh','caro-avr');  -- list closed dupes here
COMMIT;
SQL

# Step 2 — attach refs to canonicals via CLI (preserves audit trail).
bd update caro-ai5 --external-ref branch-docs/adr-008-self-update
bd update caro-4w3 --external-ref branch-feature/vercel-slidev-deployment
bd update caro-226 --external-ref branch-fix/vercel-root-directory-conflict

# Step 3 — flush JSONL.
bd sync
```

**Idempotent**: if no stranded refs are detected, this phase is a no-op.

**Future**: when [gastownhall/beads#3902](https://github.com/gastownhall/beads/issues/3902)
is fixed (CLI maps `--external-ref ""` to SQL NULL), step 1 can be
rewritten as `bd update <dupe> --external-ref ""` per row, and the
SQL fallback drops out.

## Phase C — Re-compute priorities

Priority drift is common (issue labels change, milestones shift). Re-apply the rule:

| Signal | Priority |
|--------|----------|
| Label `security` or `P0` | 0 |
| Label `P1` or `critical` | 1 |
| Epic #792 Phase 1-3 | inherit phase priority |
| Label `documentation` | 3 |
| Everything else | 2 |

```bash
for bead in /tmp/groom-beads.json:
    new_prio = compute_priority(...)
    if bead.priority != new_prio:
        bd update $bead.id --priority $new_prio
```

## Phase D — Emit delta report

Post a comment on GH Epic #792 summarizing changes. Follow `~/.claude/rules/pr-comment-structure.md`:

```bash
cat > /tmp/groom-delta.md <<EOF
\`[agent]\`

**Agent:** Claude Code (\`claude-opus-4-7\`)

---

## Backlog Grooming Delta — $(date -u +%Y-%m-%dT%H:%MZ)

**Beads state**: $(bd stats | grep 'Total Issues' | awk '{print $NF}') total · $(bd stats | grep 'Ready to Work' | awk '{print $NF}') ready · $(bd stats | grep 'Blocked' | awk '{print $NF}') blocked

### Created this cycle
$(cat /tmp/groom-created.log 2>/dev/null | sed 's/^/- /' | head -20)

### Updated this cycle
$(cat /tmp/groom-updated.log 2>/dev/null | sed 's/^/- /' | head -20)

### Closed this cycle
$(cat /tmp/groom-closed.log 2>/dev/null | sed 's/^/- /' | head -20)

### Orphan branches (need PR or cleanup)
$(cat /tmp/groom-orphan-branches.log 2>/dev/null | sed 's/^/- /' | head -10)

### Next ready work
\`\`\`
$(bd ready --limit 5)
\`\`\`

---

<details>
<summary>Prompt used to generate this comment</summary>

\`\`\`
.claude/commands/caro-backlog-groom.md — scheduled every 6h
\`\`\`

</details>
EOF

gh issue comment 792 --body-file /tmp/groom-delta.md
```

## Guardrails

- **Read-only where possible**: only write to beads when diff detected.
- **Rate-limit GH API**: `gh` CLI handles this; use `--limit 200` as ceiling.
- **Respect manual edits**: if a beads task has label `manual-curated`, skip automatic priority rewrite.
- **Idempotent**: safe to run back-to-back; no-op if nothing changed.
- **Single writer**: `.beads/bd.sock` ensures only one bd process writes at a time.

## Manual invocation

```
/caro-backlog-groom
```

## Scheduled invocation

Runs via `scheduled-tasks::create_scheduled_task` with cron `0 */6 * * *`. See task id `caro-backlog-grooming`.

## Known schema gaps (upstream)

- **[gastownhall/beads#3902](https://github.com/gastownhall/beads/issues/3902)** —
  `bd update --external-ref ""` writes empty string, not SQL NULL, and the
  UNIQUE constraint treats `''` as a value. Repair workflow in **B6** uses a
  SQL fallback to clear stranded refs in batch. Drop the SQL fallback when
  fixed upstream.
