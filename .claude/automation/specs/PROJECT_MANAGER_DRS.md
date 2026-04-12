# Project Manager Agent - Design Requirements Specification

> **Document Type**: DRS
> **Version**: 1.0.0
> **Status**: Active
> **Created**: 2026-04-12
> **Parent**: [AUTOMATED_DEV_FLOW_DRS.md](./AUTOMATED_DEV_FLOW_DRS.md)
> **Pack**: Management

---

## 1. Overview

The Project Manager Agent is a **strategic coordination layer** that sits above the tactical PR Management Loop. It triages open PRs by roadmap alignment and milestone urgency, produces a daily prioritized merge queue for human approval, and dispatches sub-agents for mechanical fixes.

### 1.1 Objectives

1. **Reduce PR backlog** — Ensure open PRs move toward merge or closure, not limbo
2. **Roadmap alignment** — Prioritize work that advances the active milestone
3. **Human-in-the-loop merges** — Never auto-merge; produce recommendations only
4. **Limited auto-dispatch** — Fix mechanical issues (rebase, lint) automatically
5. **Cross-session coordination** — Prevent dispatch conflicts across 4-5 parallel Claude sessions

### 1.2 Non-Objectives

- Does not replace human review judgment
- Does not make architectural decisions
- Does not duplicate PR Management Loop classifications
- Does not run more than once per day in scheduled mode

---

## 2. System Design

### 2.1 Component Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                     PROJECT MANAGER AGENT                             │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  TRIGGER: Daily 7 AM PT (or manual /project-manager)                 │
│                                                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │                    STATE GATHERER                               │  │
│  │  mcp__github__list_pull_requests → open PRs                    │  │
│  │  mcp__github__pull_request_read  → PR details (batched)        │  │
│  │  ROADMAP.md                      → milestone data              │  │
│  │  pr_management state             → existing classifications    │  │
│  │  agent-profiles.yaml             → dispatch routing            │  │
│  └────────────────────────┬───────────────────────────────────────┘  │
│                           │                                          │
│                           ▼                                          │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │                  STRATEGIC SCORER                               │  │
│  │  For each PR:                                                   │  │
│  │  ├── milestone_urgency  (×3)                                   │  │
│  │  ├── roadmap_alignment  (×2)                                   │  │
│  │  ├── age_penalty        (×1)                                   │  │
│  │  ├── ci_status          (×1)                                   │  │
│  │  └── review_readiness   (×1)                                   │  │
│  └────────────────────────┬───────────────────────────────────────┘  │
│                           │                                          │
│         ┌─────────────────┼─────────────────┐                        │
│         ▼                 ▼                 ▼                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │ TIER 1      │  │ TIER 2      │  │ TIER 3+4    │                  │
│  │ MERGE QUEUE │  │ DISPATCH    │  │ FLAG FOR    │                  │
│  │ GENERATOR   │  │ ENGINE      │  │ HUMAN       │                  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                  │
│         │                │                │                          │
│         ▼                ▼                ▼                          │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │                   MORNING BRIEFING                              │  │
│  │  Structured markdown report saved to automation state           │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

### 2.2 Action Tiers

```yaml
tiers:
  tier_1_merge_now:
    criteria:
      - classification: "ready_to_merge"
      - priority_score: ">=15"
    actions:
      - add_to_merge_queue
      - detect_inter_pr_conflicts

  tier_2_unblock:
    criteria:
      - classification: ["ci_failing", "stale", "has_conflicts"]
      - issue_type: "mechanical"
    actions:
      - dispatch_sub_agent
      - record_dispatch
    constraints:
      - max_concurrent: 3
      - only_mechanical_fixes: true

  tier_3_review_needed:
    criteria:
      - classification: "needs_review"
      - milestone_aligned: true
    actions:
      - flag_for_human_review

  tier_4_strategic_hold:
    criteria:
      - milestone_aligned: false
      - priority_score: "<0"
    actions:
      - recommend_close_or_defer
```

---

## 3. Priority Scoring Algorithm

### 3.1 Formula

```
priority_score =
    (milestone_urgency × 3) +
    (roadmap_alignment × 2) +
    (age_penalty × 1) +
    (ci_status × 1) +
    (review_readiness × 1)
```

### 3.2 Weight Tables

| Factor | Condition | Value |
|--------|-----------|-------|
| **milestone_urgency** | Due in <7 days | 10 |
| | Due in <14 days | 7 |
| | Due in <30 days | 4 |
| | Due in >30 days | 2 |
| | No milestone | 0 |
| **roadmap_alignment** | Maps to active milestone | 10 |
| | Maps to next milestone | 5 |
| | No milestone | 0 |
| **age_penalty** | <3 days old | 0 |
| | 3-7 days old | -2 |
| | 7-14 days old | -5 |
| | >14 days old | -10 |
| **ci_status** | Passing | 5 |
| | Pending | 0 |
| | Failing | -5 |
| **review_readiness** | Approved | 10 |
| | Changes requested | -3 |
| | No reviews | 0 |

### 3.3 Score Ranges

| Score Range | Interpretation | Typical Tier |
|-------------|---------------|--------------|
| >= 25 | Urgent merge candidate | Tier 1 |
| 15-24 | Ready, high priority | Tier 1 |
| 5-14 | Moderate priority | Tier 2/3 |
| 0-4 | Low priority | Tier 3 |
| < 0 | Strategic review needed | Tier 4 |

---

## 4. Dispatch Rules

### 4.1 Allowed Operations

| Operation | Trigger | Agent Type | MCP Tool |
|-----------|---------|------------|----------|
| Branch update | Out of date with main | N/A (MCP direct) | `mcp__github__update_pull_request_branch` |
| Clippy fix | CI failing on clippy | `rust-cli-expert` | Task tool |
| Format fix | CI failing on rustfmt | `rust-cli-expert` | Task tool |
| Typo fix | CI failing on codespell | `technical-writer` | Task tool |
| Rebase | Merge conflicts (non-overlapping) | `general-purpose` | Task tool |

### 4.2 Forbidden Operations

- Refactoring or code restructuring
- Architectural changes
- Security-sensitive fixes
- Review feedback implementation
- Test logic changes
- Any change requiring judgment

### 4.3 Throttling

```yaml
dispatch:
  max_concurrent: 3           # Per run
  max_per_pr: 1              # One dispatch per PR per run
  cooldown_hours: 4          # Don't re-dispatch same PR within 4h
  check_active_sessions: true # Skip if another session is working on the PR
```

---

## 5. Integration Points

### 5.1 Upstream (Consumes From)

| Source | Data | Method |
|--------|------|--------|
| PR Management Loop | PR classifications | Read `.claude/automation/state/pr_management/` |
| GitHub | PR metadata, CI status | `mcp__github__list_pull_requests`, `mcp__github__pull_request_read` |
| `ROADMAP.md` | Milestone data, due dates | File read |
| `agent-profiles.yaml` | Expert area mapping | File read |
| Session Wrangler | Active session state | Read plugin state (if available) |

### 5.2 Downstream (Produces For)

| Consumer | Data | Location |
|----------|------|----------|
| Human maintainer | Morning briefing | `.claude/automation/state/project_manager/YYYY-MM-DD.md` |
| Orchestrator | Run status | `.claude/automation/state/last_run.json` |
| Sub-agents | Dispatch prompts | Task tool invocations |
| Merge queue consumers | Prioritized queue | `.claude/automation/state/project_manager/merge-queue.yaml` |

---

## 6. State Management

### 6.1 Output Files

```
.claude/automation/state/project_manager/
├── YYYY-MM-DD.md              # Daily briefing (human-readable)
├── merge-queue.yaml           # Current merge queue (machine-readable)
├── dispatches.yaml            # Active dispatch log
└── history/                   # Historical briefings
    └── YYYY-MM-DD.yaml        # Structured run metrics
```

### 6.2 History Record Format

```yaml
run:
  id: "pm-2026-04-12-070000"
  started: "2026-04-12T07:00:00Z"
  completed: "2026-04-12T07:02:18Z"
  duration_seconds: 138

  input:
    total_open_prs: 23
    pr_mgmt_state_age_hours: 3.5

  scoring:
    tier_1_merge_now: 3
    tier_2_unblock: 5
    tier_3_review_needed: 8
    tier_4_strategic_hold: 7

  actions:
    dispatches: 3
    dispatches_succeeded: 2
    dispatches_failed: 1
    merge_queue_size: 3

  metrics:
    avg_priority_score: 12.4
    avg_pr_age_days: 8.2
    prs_merged_this_week: 5
    milestone_progress_pct: 50
```

---

## 7. Configuration

```yaml
# .claude/automation/config/project_manager.yaml (optional override)
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
    cooldown_hours: 4
    allowed_operations:
      - rebase
      - clippy_fix
      - format_fix
      - branch_update
      - typo_fix

  merge:
    auto_merge: false
    recommend_only: true

  briefing:
    top_recommendations: 3
    include_velocity: true
    include_milestone_progress: true
```

---

## 8. Relationship to Existing Systems

```
                  ┌──────────────────────┐
                  │  HUMAN MAINTAINER    │ ◄── Reads briefings, approves merges
                  └──────────┬───────────┘
                             │
                             ▼
                  ┌──────────────────────┐
                  │  PROJECT MANAGER     │ ◄── THIS AGENT (strategic)
                  │  (Daily 7 AM)        │
                  └──────────┬───────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
    ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
    │ PR MGMT     │  │ ROADMAP     │  │ MODULIZATION│
    │ LOOP        │  │ STATUS      │  │             │
    │ (4-hourly)  │  │ (6 AM)      │  │ (on-demand) │
    └─────────────┘  └─────────────┘  └─────────────┘
         │                                    │
         │         TACTICAL LAYER             │
         ▼                                    ▼
    ┌──────────────────────────────────────────────┐
    │              GITHUB REPOSITORY               │
    │            (PRs, Issues, CI, Labels)          │
    └──────────────────────────────────────────────┘
```

The Project Manager **consumes** output from the tactical layer (pr-management-loop classifications, roadmap status sync) and **produces** strategic output (merge queues, briefings, dispatch orders) for the human maintainer.

---

## 9. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Average PR age | < 7 days | Weekly average from briefings |
| Merge queue throughput | > 5 PRs/week | Count from history records |
| Dispatch success rate | > 80% | dispatches_succeeded / dispatches |
| Time to first review | < 2 days | From PR creation to first review |
| Stale PR count | < 5 | PRs with no activity > 7 days |
| Milestone alignment | > 70% | PRs with milestone / total open PRs |

---

## 10. Future Enhancements (Not in MVP)

1. **Dependency graph** — Map PR dependency chains automatically
2. **Auto-close** — Close PRs that have been in Tier 4 for > 30 days (with human override)
3. **Slack integration** — Post morning briefing to Slack channel
4. **Velocity trending** — Week-over-week velocity charts
5. **Smart dispatch** — Use PR diff analysis to determine if a CI failure is mechanical
6. **Cross-repo coordination** — If caro grows to multiple repos

---

## 11. Testing Strategy

### 11.1 Manual Verification

1. Run `/project-manager briefing` and verify:
   - All open PRs are accounted for
   - Scores are reasonable and explainable
   - Merge queue order makes sense
   - No PRs are incorrectly classified

2. Run `/project-manager triage --dry-run` and verify:
   - No state files are written
   - No dispatches are sent
   - Scoring output is still produced

### 11.2 Integration Verification

1. Verify `/automation/orchestrate status` shows `project_manager` loop
2. Verify scheduled run at 7 AM produces a briefing file
3. Verify dispatch sub-agents actually fix the targeted issues
4. Verify merge-queue.yaml is parseable YAML

### 11.3 Edge Cases

- Zero open PRs → Empty briefing with "No open PRs" message
- All PRs failing CI → No Tier 1 items, emphasis on Tier 2 dispatch
- No ROADMAP.md → Degrade gracefully, skip milestone scoring
- PR Management Loop state missing → Classify from scratch
