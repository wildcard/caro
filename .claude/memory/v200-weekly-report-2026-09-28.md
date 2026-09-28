# v2.0.0 Weekly Planning Report — 2026-09-28

## Completion

- **GitHub milestone "v2.0.0 - Distributed Autonomy"**
  - Last confirmed state (2026-07-06): 47 items total (18 open + 29 closed)
  - Last confirmed completion: **61.7%** (29/47)
  - Note: GitHub milestone API not directly queryable via label; state extrapolated from
    2026-07-06 report + no known closures since then
- **Core gantt feature completion: 0%** (0/5 implemented — all in research/blocked phase)
- **Release date**: June 30, 2026 → **90 DAYS OVERDUE** as of 2026-09-28

### Open items from 2026-07-06 report (18 — confirmed list, presumed unchanged)

> **Note**: #1075, #1151, #1152 are v2.0.0 research items but were **not assigned** to
> milestone #3 as of 2026-07-06 (per decision record D1, assignment was a follow-up that
> required API access unavailable at that time). The confirmed open list below matches the
> 2026-07-06 report exactly; total/completion figures (47 items, 61.7%) are consistent with
> this list.

| # | Title | Category |
|---|-------|----------|
| #1172 | [v2.0.0] Integrate Continuous Claude | Dev Experience |
| #672 | Interactive TUI Welcome Screen | Dev Experience |
| #668 | [EPIC] Automated Development Flow System | Dev Experience |
| #667 | [EPIC] Autocoder Integration | Dev Experience |
| #666 | [EPIC] Ralph Playbook — Autonomous AI Development | Dev Experience |
| #665 | [EPIC] P2P Distributed Networking Layer | Distributed |
| #664 | [EPIC] Skills Extension System (ADR-004) | Core |
| #663 | [EPIC] vLLM Jukebox Multi-Model Server Backend | Backend |
| #662 | [EPIC] Handy.Computer Integration | Backend |
| #661 | [EPIC] Azure Foundry Backend Integration | Backend |
| #162 | Add Exo cluster connection support | Backend |
| #160 | Research voice synthesis for Caro character | Research (unvalidated) |
| #154 | Plan Jazz integration for cross-device sync | Research (unvalidated) |
| #153 | Research Yappus-Term project and features | Dev Experience |
| #133 | Define Karo distributed terminal intelligence | Research (unvalidated) |
| #6 | Security hardening: cache/manifest permissions | Code Quality |
| #5 | Implement FromStr traits | Code Quality (stale) |
| #4 | Align config/logging contract tests | Code Quality (stale) |

### Core gantt features — all blocked by validation-discipline

| Feature | Tracking issue | Gantt window | Status |
|---------|---------------|--------------|--------|
| Karo Distributed Intelligence | #133 | Apr 1–May 1 (OVERDUE 150d) | research/unvalidated, 0/20 transcripts |
| Dogma Rule Engine | #1075 | Apr 1–Apr 25 (OVERDUE 156d) | research/unvalidated, 0/20 transcripts |
| Voice Synthesis | #160 | Apr 15–May 5 (OVERDUE 146d) | research/unvalidated, 0/20 transcripts |
| Self-Healing | #1151 | May 1–May 26 (OVERDUE 125d) | research/unvalidated, 0/20 transcripts |
| Local Context Indexing | #1152 | May 15–Jun 15 (OVERDUE 105d) | research/unvalidated, 0/20 transcripts |

## This week

### New issues created

None — all ROADMAP items already have tracking issues (confirmed in 2026-07-06 report).

### Stale planning PRs (ESCALATING PROBLEM)

| PR | Title | Age | Status |
|----|-------|-----|--------|
| #1452 | chore(v2.0.0): weekly planning report 2026-09-07 | 21 days | Open |
| #1437 | chore(v2.0.0): weekly planning report 2026-08-31 | 28 days | Open |
| #1350 | chore(v2.0.0): weekly planning report 2026-07-13 | 77 days | Open |
| #1173 | chore(v2.0.0): weekly planning report 2026-05-25 | 126 days | Open |

**4 planning PRs are open and unmerged.** These represent accumulated state that the
owner has not reviewed. The reports are idempotent (no code changes), so they can be
bulk-merged without review risk.

### v1.5.0 post-release context

v1.5.0 shipped July 12, 2026 (safety floor hardening + CI repair). The v2.0.0 milestone
explicitly notes it "stays open pending validation-discipline Gate 1 (user discovery)".
No scope change to v2.0.0 is recorded in the roadmap since the July 6 planning run.

### PRs of note (open, non-planning)

| PR | Title | Notes |
|----|-------|-------|
| #1470 | fix: enforce declared limits + google/ax research | `path:refuse-list` label |
| #1478 | feat(ci): lint Claude Code harness (ADR-017) | `path:refuse-list` label |
| #1459 | feat(decision): Jev gap analysis, calibration metrics | `path:scoped` |

### Blockers identified

1. **Validation discipline gate (CRITICAL, UNCHANGED)**: All 5 core features have 0/20
   user interview transcripts. Per `.claude/rules/validation-discipline.md`, no
   implementation PRs can open until Gate 1 is cleared. 90 days have passed with no
   discovery work started.
2. **Release date 90 days overdue**: Target was June 30, 2026. v2.0.0 is in an
   indefinite research hold until discovery gates clear.
3. **5 stale planning PRs**: PRs #1173, #1350, #1437, #1452 are all open and unmerged
   (4 with known numbers); the 2026-07-06 planning run's PR is also open and stale (PR
   number not recorded in this session — see Weekly Reports table in release-state.md).
   This run is the 6th consecutive planning cycle with no merge. The owner has not
   reviewed these reports.
4. **No discovery progress**: The `discovery-debt-v2.0` beads epic shows 0/20
   transcripts across all 5 features. Note: #133 (Karo) and #160 (Voice) have had open
   tracking issues since before May 2026 (gantt windows start Apr 1 and Apr 15); the
   gating issues #1075, #1151, #1152 were filed 2026-05-25 by the planning agent (126 days
   ago) with no discovery work started on any of them.

## Next milestone items

**Top 3 unstarted items to work on next:**

1. **Reset v2.0.0 target date** — The 2026-07-12 decision record
   (`docs/decisions/2026-07-12-autonomous-mode-release-scope.md` D1) already accepted this
   path: keep the milestone for the validated distributed-autonomy features; set a new target
   date (Q1 2027 proposed). Shipping the 29 completed items as "v2.0.0" was explicitly
   rejected in D1 (semver violation; misrepresents the release to users). This decision
   needs an owner sign-off and a ROADMAP update.

2. **Start discovery per audit sequencing** — `docs/discovery/v2.0-validation-audit.md`
   (revised 2026-05-31) sequences interview work as: enterprise-dashboard (drives
   dogma-rules scope), local-context-indexing, karo-distributed, self-healing,
   voice-synthesis. Self-Healing (#1151) is 4th; Voice (#160) is flagged highest
   a-priori-risk. Before interview design begins for voice-synthesis and self-healing,
   run the devil's-advocate review gate the audit requires for those two hypotheses. Run
   `caro.discovery` skill; target 20
   interviews per feature. Store transcripts flat under `docs/discovery/transcripts/`
   using `YYYY-MM-DD-<anon-handle>-<hypothesis-slug>.md` (per transcripts/README.md).

3. **Merge or close stale planning PRs** — PRs #1173, #1350, #1437, #1452 are
   documentation-only and idempotent. Bulk-merge all 4 to clear the backlog. This
   planning run's PR (#5, this cycle) is the same.

## Status

**BLOCKED / SCOPE REVIEW REQUIRED**

Release date passed 90 days ago. All 5 core features are at 0% implementation. No
discovery work has started. The milestone needs an explicit scope decision before
any further implementation planning is meaningful.

The 29 completed items represent real shipped value (v1.2.0–v1.5.0 features). The
5 research-gated items are genuine future work that cannot start without user interviews.
Per `docs/decisions/2026-07-12-autonomous-mode-release-scope.md` D1, the accepted path is
to reset the v2.0.0 target date (Q4 2026/Q1 2027) and begin discovery work now; shipping
the completed items as "v2.0.0" was explicitly rejected as a semver violation.
