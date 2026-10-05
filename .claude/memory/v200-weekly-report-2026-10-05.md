# v2.0.0 Weekly Planning Report — 2026-10-05

## Completion

- **GitHub milestone "v2.0.0 - Distributed Autonomy"** (milestone #3)
  - Total items: 47 (18 open + 29 closed)
  - Completion: **61.7%** (29/47) — unchanged since 2026-07-06
- **Core gantt feature completion: 0%** (0/5 implemented — all in research phase)
- **Release date**: June 30, 2026 → **97 DAYS OVERDUE** as of today
- **Status**: BLOCKED

### Open issues (18)

| # | Title | Notes |
|---|-------|-------|
| #1172 | [v2.0.0] Integrate Continuous Claude | In milestone |
| #672 | Interactive TUI Welcome Screen | In milestone |
| #668 | [EPIC] Automated Development Flow System | In milestone |
| #667 | [EPIC] Autocoder Integration | In milestone |
| #666 | [EPIC] Ralph Playbook — Autonomous AI Development | In milestone |
| #665 | [EPIC] P2P Distributed Networking Layer | In milestone |
| #664 | [EPIC] Skills Extension System (ADR-004) | In milestone |
| #663 | [EPIC] vLLM Jukebox Multi-Model Server Backend | In milestone |
| #662 | [EPIC] Handy.Computer Integration | In milestone |
| #661 | [EPIC] Azure Foundry Backend Integration | In milestone |
| #162 | Add Exo cluster connection support | In milestone |
| #160 | Research voice synthesis for Caro character | In milestone (unvalidated) |
| #154 | Plan Jazz integration for cross-device sync | In milestone (unvalidated) |
| #153 | Research Yappus-Term project and features | In milestone |
| #133 | Define Karo distributed terminal intelligence | In milestone (unvalidated) |
| #6 | Security hardening: cache/manifest permissions | In milestone (labeled stale) |
| #5 | Implement FromStr traits | In milestone (labeled stale) |
| #4 | Align config/logging contract tests | In milestone (labeled stale) |

**Plus 3 issues referencing v2.0.0 NOT on GitHub milestone object:**
- #1151 Self-Healing Implementation
- #1152 Local Context Indexing
- #1075 Dogma Rule Engine Research

### Core gantt features — all blocked by validation-discipline

| Feature | Tracking issue | Gantt window | Status |
|---------|---------------|--------------|--------|
| Karo Distributed Intelligence | #133 | Apr 1–May 1 (OVERDUE) | unvalidated, 0/20 transcripts |
| Dogma Rule Engine | #1075 | Apr 1–Apr 25 (OVERDUE) | unvalidated, 0/20 transcripts |
| Voice Synthesis | #160 | Apr 15–May 5 (OVERDUE) | unvalidated, 0/20 transcripts |
| Self-Healing | #1151 | May 1–May 26 (OVERDUE) | unvalidated, 0/20 transcripts |
| Local Context Indexing | #1152 | May 15–Jun 15 (OVERDUE) | unvalidated, 0/20 transcripts |

## This week

### New issues created

None — all ROADMAP.md items confirmed tracked as of 2026-09-28 run.

### PRs with CI status

| PR | Title | Age | Status |
|----|-------|-----|--------|
| #1482 | chore(v2.0.0): weekly planning report 2026-09-28 | 7d open | Unmerged; safe to merge (docs only) |

Note: PR #1173 (May 25 planning) was closed unmerged on 2026-07-12. PR #1482 (Sept 28 planning)
remains open with `mergeable_state: unstable` — likely base branch drift, not a test failure.

### Blockers identified

1. **Validation discipline gate (CRITICAL)**: All 5 core features have 0/20 user interview
   transcripts. Per `.claude/rules/validation-discipline.md` Gate 1, no implementation PRs
   can open until 20 first-hand transcripts are collected per feature.
2. **Release date passed**: Target was June 30, 2026 — now 97 days overdue, with 0/5 core
   features implemented.
3. **PR #1482 unmerged**: 7-day-old planning PR still open; docs-only, safe to merge.
4. **Scope decision deferred**: Since July 6 the owner has not acted on the
   recommendation to reset the target date or ship on 29 completed items.

## Next milestone items

**Top 3 unstarted items to work on next:**

1. **Begin user discovery for Self-Healing (#1151)** — highest PMF potential; run
   `caro.discovery` skill, target 20 first-hand interviews. Store transcripts in the
   flat layout defined by `docs/discovery/transcripts/README.md` (not in a subdirectory)
   so they register in the Gate 1 count. Note: validation-discipline requires Gates 1–5
   before implementation; a devil's-advocate review (Gate 4) is also needed once the
   hypothesis spec is drafted.

2. **Begin user discovery for Local Context Indexing (#1152)** — ChromaDB foundation
   (Phases 1–3 of #504) are complete, but Gates 3–4 remain; target 20
   interviews plus the remaining validation gates before Phase 4 implementation.

3. **Action the scope decision** — Ship v2.0.0 on 29 completed items OR reset target
   date to Q1 2027. Both options remain open; the decision requires owner input.
   See `v200-release-state.md` for the options.

## Status

**BLOCKED** — 97 days past June 30 target. All 5 core gantt features have 0% implementation
because none has cleared validation-discipline Gate 1 (20 user interview transcripts).

The milestone trajectory has not changed since July 6: no discovery work started, no
transcripts collected, no scope decision made. The reporting state is now stale by 3 months.

**Owner action needed:**
- Option A: Reset milestone target to Q1 2027 and start user discovery now.
- Option B: Ship v2.0.0 on the 29 already-completed items; move 5 core features to v2.1.0.
  (Note: previous run flagged this may be a semver concern — v2.0.0 should carry the
  major-version features that justify the bump.)
