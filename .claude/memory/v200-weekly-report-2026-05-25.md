# v2.0.0 Weekly Planning Report — 2026-05-25

## Completion

- Total milestone items tracked: 46 (17 open + 29 closed)
- Core gantt features tracked: 5 (Karo, Dogma, Voice, Self-Healing, Local Context)
- Core gantt features with substantive code: 0
- Closed/done (milestone-wide): 29 (mostly infrastructure + code-quality items)
- Milestone-wide completion: 63% (29/46)
- Core-feature completion: 0% (0/5 gantt items merged)

## This Week

### New Issues Created

| # | Title | Notes |
|---|-------|-------|
| [#1172](https://github.com/wildcard/caro/issues/1172) | [v2.0.0] Integrate Continuous Claude into development process | Only roadmap item with no prior tracking issue |

### Issues Already Tracked by Previous Planning Runs

| # | Title | Created |
|---|-------|---------|
| [#1075](https://github.com/wildcard/caro/issues/1075) | [v2.0.0] Research Dogma rule engine architecture | 2026-05-11 |
| [#1151](https://github.com/wildcard/caro/issues/1151) | [v2.0.0] Self-Healing Implementation | 2026-05-18 |
| [#1152](https://github.com/wildcard/caro/issues/1152) | [v2.0.0] Local Context Indexing | 2026-05-18 |

### PRs with CI Failures

No open PRs tagged to v2.0.0 milestone were found. The 46 open PRs across the repo are mostly i18n bot updates, dependency bumps, brand/design work, and QA rotations — none assigned to v2.0.0.

### Blockers Identified

| Feature | Gantt Window | Status | Risk |
|---------|-------------|--------|------|
| Karo Distributed Intelligence (#133) | Apr 1 – May 1 | ⚠️ OVERDUE | HIGH |
| Dogma Rule Engine (#1075) | Apr 1 – Apr 25 | ⚠️ OVERDUE | HIGH |
| Voice Synthesis (#160) | Apr 15 – May 5 | ⚠️ OVERDUE | HIGH |
| Self-Healing (#1151) | May 1 – May 26 | ⚠️ DUE TOMORROW | CRITICAL |
| Local Context Indexing (#1152) | May 15 – Jun 15 | Active window | MEDIUM |

**All 5 core features are behind the original gantt schedule.** No implementation code has been merged for any of them. The v2.0.0 release date is **June 30, 2026 — 36 days away**.

## Next Milestone Items (Top 3 to Work On)

1. **#1151 Self-Healing** — Gantt says due tomorrow (May 26). Implement `src/agent/heal.rs` with `SelfHealEngine`, error classification, and 2-retry loop. This is the highest-urgency deliverable.

2. **#1075 Dogma Rule Engine** — Research phase is overdue since April 25. Start with the ADR and TOML schema spec — no code needed yet, just architecture decisions to unblock later implementation phases.

3. **#1152 Local Context Indexing** — Active gantt window (May 15 – Jun 15). Build on the ChromaDB foundation (#504 Phases 1-3 done) to deliver Phase 4 multi-source indexing.

## Status

**AT RISK** — 4 of 5 core features are past their gantt windows with no implementation merged. The schedule requires aggressive delivery in the next 5 weeks to hit June 30. A gantt reassessment or scope reduction decision is needed before the next planning cycle.

## Milestone Tracking URLs

- Milestone: https://github.com/wildcard/caro/milestone/3
- Open issues: https://github.com/wildcard/caro/issues?q=milestone%3A%22v2.0.0+-+Distributed+Autonomy%22+is%3Aopen
