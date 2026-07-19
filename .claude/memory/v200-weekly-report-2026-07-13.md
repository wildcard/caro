# v2.0.0 Weekly Planning Report — 2026-07-13

## Completion

- **GitHub milestone "v2.0.0 - Distributed Autonomy"** (milestone #3)
  - Total items: ~47 (baseline from July 6 run; no new issues created)
  - Closed items: ~30+ (v1.5.0 landed July 12, closing safety-floor items)
  - Completion: **~64%** (approx)
- **Core gantt feature completion: 0%** (0/5 implemented — all in research/discovery phase)
- **Release date**: June 30, 2026 → **13 DAYS OVERDUE** as of today
- **Discovery debt**: 0/20 transcripts across all 5 blocked features (unchanged)

### Open v2.0.0 issues (no change from last week)

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

### Discovery debt issues (all open, all 0/20 transcripts)

| # | Hypothesis | Priority | Status |
|---|-----------|---------|--------|
| #1188 | Discovery debt epic (umbrella) | — | Open |
| #1189 | local-context-indexing | P0 | Open — do first |
| #1190 | karo-distributed | P2 | Open |
| #1191 | dogma-rules | P1 | Open (after enterprise-dashboard) |
| #1192 | self-healing | P2 | Open — devil's-advocate FIRST |
| #1193 | voice-synthesis | P2 | Open — devil's-advocate FIRST |

### Core gantt features — all blocked by validation-discipline

| Feature | Tracking issues | Gantt window | Status |
|---------|----------------|--------------|--------|
| Karo Distributed Intelligence | #133, #1190 | Apr 1–May 1 (OVERDUE 104d) | 0/20 transcripts |
| Dogma Rule Engine | #1075, #1191 | Apr 1–Apr 25 (OVERDUE 104d) | 0/20 transcripts |
| Voice Synthesis | #160, #1193 | Apr 15–May 5 (OVERDUE 89d) | 0/20 transcripts |
| Self-Healing | #155, #1151, #1192 | May 1–May 26 (OVERDUE 78d) | 0/20 transcripts |
| Local Context Indexing | #152, #1152, #1189 | May 15–Jun 15 (OVERDUE 58d) | 0/20 transcripts |

---

## This week (2026-07-07 to 2026-07-13)

### Blockers cleared

| Item | What happened |
|------|--------------|
| **PR #1173 closed** | Stale May-25 planning PR closed July 12 (not merged — superseded). Backlog clear. |
| **PR #1246 merged** | CI repair PR merged July 12: test_allowlist_functionality + test_cache_roundtrip fixed; catastrophic floor hardened with 5 adversarial-review rounds. CI green gate restored. |
| **v1.5.0 released** | Safety floor hardening shipped July 12 (RUSTSEC-2026-0204/0185 patched, ethnum/rust-1.97 compile break fixed, MSRV 1.85, cache-test de-flaked). |

### New issues created

None — full coverage confirmed in the July-6 run.

### Blockers remaining

1. **Validation discipline gate (CRITICAL)**: All 5 core features have 0/20 user interview
   transcripts. Gate 1 of .claude/rules/validation-discipline.md is unmet. Implementation
   PRs cannot open. Discovery work has not started.
2. **Release date +13 days overdue**: Target was June 30, 2026. No scope decision recorded.
3. **Scope decision deferred (3rd consecutive week)**: Two prior reports recommended a
   milestone split (ship completed items as v2.0.0, create v2.1.0 for discovery-gated
   features). No owner has acted on either recommendation.

---

## Next milestone items

**Top 3 priority actions:**

1. **Make the scope decision.** Options:
   - (A) Reset v2.0.0 release date to Q4 2026; begin discovery now
   - (B) Ship v2.0.0 on the 30+ completed items; create v2.1.0 for 5 discovery-gated features (recommended)
   - (C) Accept the slip and wait for discovery to complete
   Option B delivers the 4 validated core-extensions (Azure Foundry, vLLM Jukebox,
   Skills Extension, Handy.Computer) and unblocks the milestone.

2. **Begin user discovery for local-context-indexing (#1189)** — P0, strongest signal.
   Run caro.discovery skill. Re-interview GitHub Issue authors of #152 and #166 as warm
   leads. Target: 5 transcripts this week, 20 within 4 weeks.

3. **Begin user discovery for self-healing (#1192)** — but run devils-advocate agent
   against the proposal FIRST before designing interview questions (per issue requirement).

---

## Status

**BLOCKED — 13 days overdue.** Two blockers resolved this week (CI repair merged, stale PR
closed). Core blocker unchanged: Gate 1 (user interview transcripts) unmet for all 5 core
features. The scope decision is the only unblocking action available to a human owner.

**Recommendation (third consecutive week)**: Choose Option B. Ship v2.0.0 with what is
done; move the 5 discovery-gated features to v2.1.0. Start discovery interviews immediately.
