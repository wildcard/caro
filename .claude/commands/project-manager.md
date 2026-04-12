---
description: Strategic project manager agent for PR triage, merge queue, milestone alignment, and cross-session dispatch
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `ROADMAP.md`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/project-manager` | Default: run morning briefing |
| `/project-manager briefing` | Full morning briefing with merge queue + dispatch |
| `/project-manager triage` | Triage open PRs only (no dispatch, no merge queue) |
| `/project-manager triage --dry-run` | Triage with scoring but no actions |
| `/project-manager merge-queue` | Generate prioritized merge queue only |
| `/project-manager dispatch` | Dispatch sub-agents for fixable PRs only |
| `/project-manager status` | Show last briefing and open queue |
| `/project-manager --pr 123` | Focus on a single PR |
| `/project-manager --milestone v1.2.0` | Filter to a specific milestone |

---

## What This Command Does

`/project-manager` is the strategic coordinator for the caro project. It sits **above** the tactical `/pr-management-loop` and provides:

- **Strategic triage** — Score and rank open PRs by roadmap alignment + milestone urgency
- **Merge queue** — Produce a prioritized list of PRs ready for human approval
- **Targeted dispatch** — Spawn sub-agents for mechanical fixes (rebase, simple CI failures)
- **Morning briefing** — Structured report for the maintainer at the start of each day
- **Cross-session awareness** — Prevents dispatch conflicts across parallel Claude sessions

**Boundaries:**
- **Never auto-merges.** Produces recommendations only — human always approves merges.
- **Never dispatches complex fixes.** Only mechanical operations. Complex issues get flagged for humans.
- **Builds on pr-management-loop.** Does not duplicate PR classification logic; consumes its output.

---

## Core Workflow

```
STATE → CLASSIFY → PRIORITIZE → MERGE QUEUE → DISPATCH → BRIEFING
```

---

## Pre-flight Checks

1. Verify GitHub MCP tools are available (`mcp__github__list_pull_requests`, `mcp__github__pull_request_read`)
2. Verify `ROADMAP.md` exists at project root
3. Verify `.claude/agent-profiles.yaml` exists
4. If any check fails, stop with a clear error message

---

## Phase 1: Gather State

### 1.1 Fetch Open PRs

Use `mcp__github__list_pull_requests` for repo `wildcard/caro`:
- State: `open`
- Include: labels, reviewers, head ref, base ref, created/updated timestamps

### 1.2 For Each PR, Fetch Details

Use `mcp__github__pull_request_read` to get:
- CI status (statusCheckRollup)
- Review status (approved, changes requested, pending)
- Merge readiness (mergeable flag, conflicts)
- Last activity (comments, pushes)
- Milestone assignment

Batch these calls in parallel where possible to minimize API calls.

### 1.3 Read Roadmap Context

```bash
# Read ROADMAP.md for milestone data
cat ROADMAP.md
```

Extract:
- Active milestone and due date
- Next milestone
- Priority themes per milestone
- Total items and completion percentage

### 1.4 Read Agent Profiles

```bash
cat .claude/agent-profiles.yaml
```

Extract area-to-profile mapping for dispatch decisions.

### 1.5 Read Last PR Management Loop State

```bash
# Read most recent pr_management state file
ls -t .claude/automation/state/pr_management/*.yaml 2>/dev/null | head -1
```

If present, consume existing classifications rather than re-classifying.

---

## Phase 2: Classify & Score

### 2.1 Use Existing Classifications

Reuse classifications from `/pr-management-loop`:
- `healthy` — CI passing, recent activity
- `stale` — No activity > 3 days
- `stale_critical` — No activity > 7 days
- `ci_failing` — CI checks failing
- `needs_review` — CI passing, no reviews
- `has_feedback` — Unaddressed review comments
- `ready_to_merge` — Approved, CI passing, no conflicts
- `has_conflicts` — Merge conflicts present

### 2.2 Calculate Strategic Priority Score

For each PR, compute:

```
priority_score =
    (milestone_urgency_weight × 3)  +
    (roadmap_alignment_weight × 2)  +
    (age_penalty_weight × 1)        +
    (ci_status_weight × 1)          +
    (review_readiness_weight × 1)
```

**Weights:**

| Factor | Values |
|--------|--------|
| milestone_urgency | Due in <7d: 10, <14d: 7, <30d: 4, >30d: 2, none: 0 |
| roadmap_alignment | Active milestone: 10, next milestone: 5, no milestone: 0 |
| age_penalty | <3d: 0, 3-7d: -2, 7-14d: -5, >14d: -10 |
| ci_status | passing: 5, pending: 0, failing: -5 |
| review_readiness | approved: 10, changes_requested: -3, no reviews: 0 |

### 2.3 Group into Action Tiers

| Tier | Criteria | Action |
|------|----------|--------|
| **Tier 1 — Merge Now** | `ready_to_merge` + score ≥ 15 | Add to merge queue |
| **Tier 2 — Unblock** | `ci_failing`, `stale`, `has_conflicts` (simple) | Dispatch sub-agent fix |
| **Tier 3 — Review Needed** | `needs_review` + milestone-aligned | Request reviews |
| **Tier 4 — Strategic Hold** | No milestone or score < 0 | Flag for human decision |

---

## Phase 3: Generate Merge Queue

For Tier 1 PRs, produce an ordered merge queue.

### 3.1 Sort by Priority

Sort Tier 1 PRs descending by `priority_score`.

### 3.2 Detect Conflicts Between PRs

For each pair of Tier 1 PRs, check if they touch the same files:
- Use `mcp__github__pull_request_read` to get changed files
- Flag pairs that share files as potential conflicts
- Recommend merging the higher-priority PR first

### 3.3 Output Structured Merge Queue

Save to `.claude/automation/state/project_manager/merge-queue.yaml`:

```yaml
generated_at: "2026-04-12T07:00:00Z"
queue:
  - position: 1
    pr: 234
    title: "Add fish shell support"
    priority_score: 32
    milestone: "v1.2.0"
    conflicts_with: []
  - position: 2
    pr: 241
    title: "Update installation docs"
    priority_score: 28
    milestone: "v1.2.0"
    conflicts_with: [245]
```

---

## Phase 4: Dispatch (Limited Auto-fix)

**Only dispatch sub-agents for mechanical operations.** Anything requiring judgment stays in the briefing for humans.

### 4.1 Safe Dispatch Operations

| Issue | Operation | MCP Tool / Agent |
|-------|-----------|------------------|
| Branch out of date with main | `update_pull_request_branch` | `mcp__github__update_pull_request_branch` |
| Simple CI failure (lint, fmt, typo) | Spawn agent to analyze + fix | Task tool with `rust-cli-expert` or `technical-writer` |
| Merge conflict (non-overlapping) | Rebase via git | Task tool with `general-purpose` agent |

### 4.2 NEVER Dispatch For

- Complex CI failures (test logic errors, flaky tests)
- Review feedback requiring code changes
- Architectural concerns
- Security-sensitive PRs
- PRs without a clear owner

### 4.3 Dispatch Process

For each eligible Tier 2 PR:

1. **Check for active sessions on the same PR** (via session-wrangler state if available)
2. **Spawn sub-agent via Task tool** with a focused prompt:
   - Goal: "Fix the specific issue on PR #NNN"
   - Context: PR details, CI logs, conflict details
   - Constraints: Make minimal changes, don't refactor
3. **Record dispatch** in `.claude/automation/state/project_manager/dispatches.yaml`

### 4.4 Throttle Dispatches

Maximum 3 concurrent dispatches per run to avoid overwhelming CI and parallel sessions.

---

## Phase 5: Morning Briefing Report

Generate a structured report and save it to:
`.claude/automation/state/project_manager/YYYY-MM-DD.md`

### 5.1 Report Format

```markdown
# Caro Project Manager — Morning Briefing

**Date**: YYYY-MM-DD
**Generated**: HH:MM UTC
**Run ID**: pm-YYYY-MM-DD-HHMMSS

---

## Executive Summary

- **Open PRs**: NN total
- **Ready to merge**: N (awaiting human approval)
- **Unblocked today**: N (automated fixes applied)
- **Needs attention**: N (manual review required)
- **Active milestone**: vX.Y.Z (XX days remaining, YY% complete)

---

## Merge Queue (Human Approval Required)

| # | PR | Title | Score | Milestone | Notes |
|---|----|----|-------|-----------|-------|
| 1 | #234 | Add fish shell support | 32 | v1.2.0 | approved, CI ✓ |
| 2 | #241 | Update installation docs | 28 | v1.2.0 | approved, CI ✓ |

**Recommended merge order**: #234 → #241 → #245
**Conflicts detected**: None

---

## Unblocked (Automated Fixes Applied)

- **PR #235**: Rebased with main (was 4 days stale)
- **PR #237**: Applied clippy fix for `unused_import`

---

## Needs Human Attention

### Tier 2 — Unresolved
- **PR #236**: CI failing — flaky integration test (3 retries failed)
  - Recommended owner: @maintainer
  - Last activity: 5 days ago

### Tier 3 — Review Requested
- **PR #240**: Passing CI, no reviews (2 days old)
  - Area: Backends → AI/ML profile match
  - Blocks: v1.2.0 milestone deliverable

### Tier 4 — Strategic Hold
- **PR #199**: No milestone assigned, 30+ days stale
  - Recommendation: Close or assign to v2.0.0

---

## Strategic Overview

### Milestone Progress
- **v1.2.0** (due Mar 31): 12/24 complete (50%)
- **v2.0.0** (due Jun 30): 0/30 complete (0%)

### Velocity
- Merged this week: N PRs
- Merged last week: M PRs
- Trend: ↑ / ↓ / stable

### Blockers
- **Release blockers**: N open (see `label:release-blocker`)
- **Critical bugs**: N open

---

## Recommended Actions (Top 3)

1. **Merge Tier 1 queue** — 2 PRs ready, no conflicts
2. **Review PR #240** — Blocks v1.2.0 milestone
3. **Close PR #199** — Superseded, 30+ days stale

---

## Dispatch Log

| PR | Action | Agent | Status |
|----|--------|-------|--------|
| #235 | rebase | general-purpose | ✓ complete |
| #237 | clippy-fix | rust-cli-expert | ✓ complete |

---

*Next briefing: tomorrow 7 AM PT*
```

### 5.2 Notify Completion

If running in scheduled mode, emit completion notification per `schedule.yaml` settings.

---

## Mode-Specific Logic

### briefing (default)
Execute all 5 phases. Produces full report with merge queue and dispatches.

### triage
Execute Phases 1-2 only. Produces classification without actions.

### triage --dry-run
Execute Phases 1-2 + calculate scoring but **do not write state files**.

### merge-queue
Execute Phases 1-3. Produces merge queue only, no dispatch.

### dispatch
Execute Phases 1-2 + 4. Dispatch sub-agents for fixable PRs.

### status
Read last briefing from `.claude/automation/state/project_manager/` and display.

### --pr NNN
Focus on single PR — fetch details, compute score, output recommendations.

### --milestone vX.Y.Z
Filter all PRs to a specific milestone before classification.

---

## Integration with Existing Systems

| System | Integration |
|--------|-------------|
| `/pr-management-loop` | Consumes classification state if recent (<6h old) |
| `/caro.roadmap` | Reads milestone priorities |
| `/modulization` | Optional: consume module classifications for Tier 4 decisions |
| `/automation/orchestrate` | Runs as `project_manager` loop in Management pack |
| `agent-profiles.yaml` | Matches PRs to expert agents for dispatch |
| Session Wrangler | Checks active sessions to prevent dispatch conflicts |

---

## Configuration

Default config is embedded. Override via `.claude/automation/config/project_manager.yaml`:

```yaml
project_manager:
  enabled: true
  schedule: "0 7 * * *"

  scoring:
    milestone_urgency_weight: 3
    roadmap_alignment_weight: 2
    age_penalty_weight: 1
    ci_status_weight: 1
    review_readiness_weight: 1

  tiers:
    merge_now_threshold: 15
    strategic_hold_threshold: 0

  dispatch:
    enabled: true
    max_concurrent: 3
    allowed_operations:
      - rebase
      - clippy_fix
      - format_fix
      - branch_update
    forbidden_operations:
      - refactor
      - architectural_change
      - security_fix

  merge:
    auto_merge: false  # NEVER auto-merge
    recommend_only: true
```

---

## Output Artifacts

```
.claude/automation/state/project_manager/
├── YYYY-MM-DD.md              # Daily briefing (human-readable)
├── merge-queue.yaml           # Current merge queue (structured)
├── dispatches.yaml            # Active dispatch log
└── history/                   # Historical briefings
    └── YYYY-MM-DD.yaml        # Structured run data
```

---

## Example Session

```
> /project-manager briefing

Caro Project Manager — Morning Briefing
══════════════════════════════════════════

Fetching open PRs... found 23
Fetching PR details... (batched)
Reading ROADMAP.md... active: v1.2.0 (48 days remaining)
Reading pr-management state... consuming from 4h ago

Classifying & scoring...
  Tier 1 (merge now):    3 PRs
  Tier 2 (unblock):      5 PRs
  Tier 3 (review):       8 PRs
  Tier 4 (strategic):    7 PRs

Generating merge queue...
  Position 1: PR #234 (score: 32)
  Position 2: PR #241 (score: 28)
  Position 3: PR #238 (score: 22)
  No conflicts detected.

Dispatching automated fixes...
  PR #235: Spawning rebase agent...  ✓
  PR #237: Spawning clippy-fix agent... ✓
  PR #242: Spawning branch-update... ✓
  (3/3 dispatches, throttle reached)

Writing briefing to .claude/automation/state/project_manager/2026-04-12.md...
✓ Complete (2m 18s)

Summary:
  Ready for merge:  3 PRs
  Auto-fixes:       3 PRs
  Needs attention:  5 PRs (see briefing)
  Strategic review: 7 PRs

Next run: tomorrow 7 AM PT
```

---

## Related Skills

- `/pr-management-loop` — Tactical PR hygiene (every 4h)
- `/caro.roadmap` — Roadmap status and work selection
- `/modulization` — Unfinished work discovery
- `/stale-revival-loop` — Weekly deep cleanup
- `/automation/orchestrate` — Schedule and run loops

## DRS Reference

See [PROJECT_MANAGER_DRS.md](../../.claude/automation/specs/PROJECT_MANAGER_DRS.md)
