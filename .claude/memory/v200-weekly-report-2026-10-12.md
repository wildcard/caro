# v2.0.0 Weekly Planning Report — 2026-10-12

## Completion

- **GitHub milestone "v2.0.0 - Distributed Autonomy"**
  - Total items: ~47 (18 open + 29 closed, per July 2026 census; GitHub `milestone:v2.0.0` label not in use — milestone object #3 tracks them)
  - Completion: **~61.7%** (29/47, unchanged from July 6 run — no new v2.0.0 items closed)
- **Core gantt feature completion: 0%** (0/5 implemented — all in research phase)
- **Release date**: June 30, 2026 → **104 DAYS OVERDUE** as of today
- **Context**: v1.5.0 release PR merged October 11, 2026; tag and crates.io publish still pending
  (credential scope blocks `git push origin v1.5.0` and `gh workflow run tag-release.yml` — see
  `.claude/memory/v150-release-handoff.md`). The release commit is on `main` but not yet visible
  to users on crates.io.

## This week

### New issues created

None — all ROADMAP items already have tracking issues. v2.0.0 items tracked via GitHub milestone #3, not the `milestone:v2.0.0` label (which doesn't exist). No untracked items found.

### Notable open items (from ROADMAP.md; not reconciled live against milestone #3 this run)

| # | Title | Labels |
|---|-------|--------|
| #133 | Define Karo distributed terminal intelligence | enhancement, safety |
| #154 | Plan Jazz integration for cross-device sync | enhancement |
| #155 | Add self-healing feature to CARO | bug, enhancement |
| #160 | Research voice synthesis for Caro character | enhancement |
| #162 | Add Exo cluster connection support | enhancement, backend |
| #504 | [EPIC] ChromaDB Integration — Pluggable Vector Backend | enhancement, backend |
| #661 | [EPIC] Azure Foundry Backend Integration | bug, enhancement, backend |
| #662 | [EPIC] Handy.Computer Integration | bug, enhancement, backend |
| #663 | [EPIC] vLLM Jukebox Multi-Model Server Backend | bug, enhancement, backend |
| #664 | [EPIC] Skills Extension System (ADR-004) | enhancement, security |
| #665 | [EPIC] P2P Distributed Networking Layer | enhancement, security |
| #666 | [EPIC] Ralph Playbook — Autonomous AI Development | enhancement, safety |
| #667 | [EPIC] Autocoder Integration | enhancement |
| #668 | [EPIC] Automated Development Flow System | enhancement, safety |
| #672 | Interactive TUI Welcome Screen | enhancement |
| #4 | Align config/logging contract tests | stale |
| #5 | Implement FromStr traits | stale |
| #6 | Add security hardening: cache/manifest permissions | stale |

### Research (unvalidated) — blocked by validation-discipline Gate 1

| Hypothesis ID | Tracking Issues | Transcripts | Status |
|---|---|---|---|
| `karo-distributed` | #133, #171, #154 | 0 / 20 | `unvalidated` |
| `dogma-rules` | #126, #1075 | 0 / 20 | `unvalidated` |
| `voice-synthesis` | #160, #187 | 0 / 20 | `unvalidated` |
| `self-healing` | #155, #1151 | 0 / 20 | `unvalidated` |
| `local-context-indexing` | #152, #166, #504, #1152 | 0 / 20 | `unvalidated` |

### PRs with CI failures

The `milestone:v2.0.0` label does not exist; PRs are checked via milestone #3.
No open PRs confirmed assigned to milestone #3. Previously flagged PRs:
- PR #1173 — previous weekly planning PR (May 25, 2026). Status unconfirmed; check if merged.
- PR #1246 — CI fix PR. Status unconfirmed; check if merged.

### Blockers identified

1. **Validation discipline gate (CRITICAL)**: All 5 core features have 0/20 user interview
   transcripts. Per `.claude/rules/validation-discipline.md`, implementation PRs cannot open
   until Gate 1 (20 transcripts) is cleared. This gate has been uncleared for 20+ weeks.
2. **Release date passed by 104 days**: Target was June 30, 2026. Now October 12, 2026.
3. **Milestone scope still undefined**: No scope-reset decision taken (see July recommendation).
   The ROADMAP status table shows "0% - Research Phase" but the issue tracker shows 61.7%.
4. **Discovery work not started**: `docs/discovery/transcripts/` has no entries for any of the
   5 hypothesis IDs per `v2.0-validation-audit.md`.

## Next milestone items

**Top 3 actions to unblock v2.0.0:**

1. **Reset the v2.0.0 target date** — Per `docs/decisions/2026-07-12-autonomous-mode-release-scope.md`
   (Decision D1, accepted), v2.0.0 stays scoped to validated distributed-autonomy features. Shipping
   the 29 completed items as v2.0.0 was explicitly rejected (Option A in D1). The accepted decision
   was to keep v2.0.0 for validated features and reset its target date. Owner action needed:
   set a new date in GitHub milestone #3 (Q4 2026 or Q1 2027) and begin discovery.

2. **Begin user discovery for Self-Healing (#155, #1151)** — Highest concrete PMF potential.
   Run `caro.discovery` skill; target 20 first-hand interviews with power-CLI users. Store
   transcripts in `docs/discovery/transcripts/` using filename convention
   `YYYY-MM-DD-<anon-handle>-self-healing.md`. Required before any code.

3. **Begin user discovery for Local Context Indexing (#504)** — ChromaDB Phases 1–3 are
   already complete. Store transcripts in `docs/discovery/transcripts/` using filename convention
   `YYYY-MM-DD-<anon-handle>-local-context-indexing.md`. Phase 4 proceeds only after all five
   validation-discipline gates clear (20 transcripts with ≥3 distinct pain patterns, no-surveys,
   demoware-trap section, devil's-advocate review, Sean Ellis defended cohort).

## Status

**BLOCKED / AT RISK** — Release date passed 104 days ago (June 30 → October 12, 2026).
All 5 core gantt features are at 0% implementation. Validation-discipline Gate 1 has not been
cleared for any hypothesis. No discovery work has started.

Per Decision D1 (2026-07-12), v2.0.0 stays scoped to validated distributed-autonomy features.
The next step is: reset the milestone target date and begin user discovery.

v1.5.0 release PR merged October 11, 2026; the project is healthy overall —
v2.0.0's core features are gated on user discovery, not engineering capacity.

**Owner action required**: Reset milestone target date in GitHub; begin discovery kickoff.
