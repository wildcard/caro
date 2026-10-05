# Weekly Demo — 2026-10-05 (week of Sep 28 – Oct 5)

Per [`.claude/rules/feature-evidence.md`](../../.claude/rules/feature-evidence.md):
every feature merged this week, with demo, evidence, and regression guard.

## No feature merges this week

No feature PRs merged in the Sep 28–Oct 5 window. This report satisfies the standing
requirement so absence of a demo file never signals a failed routine.

Open planning PR: [#1528](https://github.com/wildcard/caro/pull/1528)
(v2.0.0 weekly planning report — docs-only memory files; not a feature).

## Pareto View by Backend

Evaluation suite was not run by this planning routine (no build or feature work this session).
No evaluation run link is available for this week; the last recorded results are from the
2026-07-12 feature session. See the evaluate workflow for the current baseline.

| Backend | Pass Rate | ECE | Cost/task | p95 Latency | Risk-label agreement |
|---------|-----------|-----|-----------|-------------|----------------------|
| *Not available — re-run `cargo test --test evaluation` on a build session for current data* | — | — | — | — | — |

A fresh Pareto table is produced whenever a feature PR triggers an evaluation CI run.
See [GitHub Actions → Evaluate jobs](https://github.com/wildcard/caro/actions/workflows/evaluate.yml)
for the latest automated results.

## Known red (not regressions, tracked)

- **Vercel caro-foss-website deploy** — pre-existing Astro 5→6 migration issue; present on
  `main` and every PR; root-caused in #1246's body. Not this PR's regression.

## Next week

- Begin user discovery for Self-Healing (#1151) — highest PMF potential; run devil's-advocate review on any AI-drafted hypothesis spec first (Gate 4), then run `caro.discovery` to collect 20 transcripts.
- Begin user discovery for Local Context Indexing (#1152) — Gates 3–4 remain.
- Reset v2.0.0 milestone target date (D1 default: Q4 2026).
