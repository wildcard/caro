# Frustrated-Beta QA Run — 2026-04-26 (smoke test, dry-run)

caro 1.3.0 (crates.io) on macOS 26.3, zsh.
Queries tested: **1** from `website/src/data/gtm-use-cases.ts` (limit=1).
Mode: `--dry-run` — no GitHub issues filed; drafts under `proposed-issues/` for review.

## Findings by class

| Class                       | P0 | P1 | P2 | P3 | Filed | Updated existing |
| --------------------------- | -- | -- | -- | -- | ----- | ---------------- |
| `WEBSITE/BROKEN_PROMISE`    | 1  | 0  | 0  | 0  | 0 (dry-run) | 0 |

**1 finding from 1 query.** 100% failure rate on the smoke test sample.

## Issues filed today

None — `--dry-run` mode. Drafts ready in `proposed-issues/`:

- `proposed-issues/01-website-broken-promise-find-and-kill.md` — title:
  `[QA/WEBSITE_BROKEN_PROMISE] "find and kill the runaway process eating CPU" generates only ps aux | sort — drops the kill half`
  Labels-to-apply on real run: `qa-routine`, `frustrated-beta`, `website-broken-promise`, `P0`

## Issues updated today (existing reproductions)

None.

## Stale PRs cross-linked

- **[PR #567](https://github.com/wildcard/caro/pull/567)** — *Improve UX for long-running commands* — open 97 days, expected to address this finding's multi-verb intent gap.
- **[Issue #449](https://github.com/wildcard/caro/issues/449)** — *🎯 [EPIC] Exploration Agent: Complete Integration & Rollout* — open 102 days, the architectural answer (clarifying questions for ambiguous multi-verb queries).

(Comments would be posted on each on a real-mode run — skipped here.)

## Community sweep

Skipped — smoke test scope. The real 5 AM run will execute the full sweep per the orchestrator spec.

## Smoke-test meta-findings (about the routine itself)

These are not caro bugs — they are observations about the QA routine that came up running it for the first time. Worth tracking before the routine fires for real:

1. **Persona agent not loaded into the harness on first session.** Calling
   `Agent` with `subagent_type: caro-frustrated-beta` failed with
   `Agent type 'caro-frustrated-beta' not found`. The new agent file
   (`.claude/agents/caro-frustrated-beta.md`) was created in the same session
   but the harness only picks up agent definitions at session start. **Action**:
   the cron driver must spawn caro-frustrated-beta via a fresh session so the
   harness sees the new agent file. Worth adding a preflight check that
   verifies the agent type is registered before spawning.

2. **Smoke-test fallback worked.** I executed the persona's procedure inline
   when the spawn failed. The artifacts produced match the format the agent
   prompt prescribes, so the agent and the orchestrator are aligned on
   schema. Once the agent is registered, no behavioural change expected.

3. **Run-dir naming**: I used `2026-04-26-smoketest` to avoid colliding with
   tomorrow's real 5 AM run. The orchestrator should use `<date>` for real
   runs and `<date>-<suffix>` for ad-hoc / smoke runs.

## Loudest signal

> **The very first query the website tells users to try produces a command that does the opposite of what they asked for: it finds the runaway process and then *doesn't* kill it.** The user has to know enough Unix to spot the missing tail and write `| head -1 | awk '{print $2}' | xargs kill` themselves — which defeats the entire premise of the tool. This is a P0 first-touch trust failure and the website's first example. Either the matcher needs the multi-verb fix, or the website needs to change the example. Today.

---

```
RUN: 2026-04-26-smoketest | queries: 1 | findings: P0=1 P1=0 P2=0 | repro_requests: 0
NEXT: file proposed-issues/01-website-broken-promise-find-and-kill.md as a real GH issue, then post the cross-link comment on PR #567 and Issue #449. The cron driver should also include a preflight that confirms `caro-frustrated-beta` is a registered Agent subtype before spawning.
SIGNAL: caro 1.3.0 fails the very first query on the landing page — "find and kill the runaway process" returns a half-pipeline that finds but never kills. P0 broken-promise.
```
