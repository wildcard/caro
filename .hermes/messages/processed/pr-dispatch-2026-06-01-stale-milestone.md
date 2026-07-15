# Dispatcher cycle — 2026-06-01 (caro-merge-review-integrate)

## Forward-path classifications (this cycle)

| PR | path | owner | bead filed | notes |
|---|---|---|---|---|
| — | — | — | — | No open PRs assigned to milestone `v1.2.0 - Global Launch` |

## Newly filed beads
- (none) — zero open milestone PRs to dispatch

## Beads closed (stale cleanup)
- `caro-jac.9` (`gh-599-rebase`) — PR #599 **MERGED**, rebase obsolete
- `caro-jac.2` (`gh-609-rebase`) — PR #609 **CLOSED**, rebase obsolete
- `caro-jac.1` (`gh-651-rebase`) — PR #651 **CLOSED**, rebase obsolete

## Coordination alerts (to Hermes) — ⚠️ ROUTINE TARGET IS SUPERSEDED
This dispatcher is hard-wired to milestone `v1.2.0 - Global Launch`, which has
been **fully superseded**. Shipped past it: **v1.3.0, v1.3.2, v1.4.0 (Latest /
GA)**; **v1.5.0** is already a draft release. The milestone retains 30 open
*issues* but **zero open PRs** — the v1.2.0 delivery pipeline is effectively
complete.

**Recommended actions (for human / Hermes triage):**
1. Retarget `caro-merge-review-integrate` SKILL.md from `v1.2.0` to the current
   active milestone (likely `v1.5.0` or `v2.0.0 - Distributed Autonomy`), OR
   retire the routine if v2.0 uses a different dispatch flow.
2. Decide the fate of the 30 stale open v1.2.0 issues (close / re-milestone).

## Release-readiness gates
- CHANGELOG [Unreleased] items: **4** (these belong to the **v1.5.0** draft, NOT v1.2.0)
- Milestone v1.2.0 issue progress: 7 closed / 37 total (**18%**)
- Threshold met (≥3 unreleased OR ≥80% closed)?: unreleased-count YES, but
  **release suggestion SUPPRESSED** — the gate would suggest preparing a v1.2.0
  release that already shipped 3 releases ago. Suggesting `/caro.release.prepare`
  for v1.2.0 against Epic #792 would be incorrect noise.
