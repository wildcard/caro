# v2.0.0 Weekly Planning Report — 2026-10-12

## Completion

- **GitHub milestone "v2.0.0 - Distributed Autonomy"**
  - Total items: ~47 (18 open + 29 closed, per July 2026 census; GitHub `milestone:v2.0.0` label not in use — milestone object #3 tracks them)
  - Completion: **~61.7%** (29/47, unchanged from July 6 run — no new v2.0.0 items closed)
- **Core gantt feature completion: 0%** (0/5 implemented — all in research phase)
- **Release date**: June 30, 2026 → **104 DAYS OVERDUE** as of today
- **Context**: v1.5.0 shipped October 11, 2026 (yesterday). This was a consolidation of post-July work; it did not include any v2.0.0 core features.

## This week

### New issues created

None — all ROADMAP items already have tracking issues. v2.0.0 items tracked via GitHub milestone #3, not the `milestone:v2.0.0` label (which doesn't exist). No untracked items found.

### Notable open items (confirmed open as of today)

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

No open PRs with `milestone:v2.0.0` label. Previously flagged PRs:
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

1. **Milestone scope decision** — Choose Option A (reset to Q4 2026 / Q1 2027 with discovery
   gate) OR Option B (ship v2.0.0 on the 29 completed items; move 5 discovery-gated features
   to v2.1.0). This is an owner decision, not an agent decision. Without it, the milestone
   target date is meaningless and all core features remain permanently blocked.

2. **Begin user discovery for Self-Healing (#155, #1151)** — Highest concrete PMF potential.
   Run `caro.discovery` skill; target 20 first-hand interviews with power-CLI users. Store
   transcripts in `docs/discovery/transcripts/self-healing/`. Required before any code.

3. **Begin user discovery for Local Context Indexing (#504)** — ChromaDB Phases 1–3 are
   already complete. 20 interviews would unlock Phase 4 (the actual feature work). Store
   transcripts in `docs/discovery/transcripts/local-context-indexing/`.

## Status

**BLOCKED / AT RISK** — Release date passed 104 days ago (June 30 → October 12, 2026).
All 5 core gantt features are at 0% implementation. Validation-discipline Gate 1 has not been
cleared for any hypothesis. The milestone scope has not been formally reset despite the July 6
recommendation. No discovery work has started.

v1.5.0 shipped yesterday (October 11), which demonstrates the project is healthy overall —
but v2.0.0's core features remain entirely gated on user discovery, not engineering capacity.

**Owner decision required**: Scope reset or discovery kickoff.
