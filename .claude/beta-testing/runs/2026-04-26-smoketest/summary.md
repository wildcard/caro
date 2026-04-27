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

After the dry-run smoke completed and the user approved the recommended
follow-up, the proposed issue was filed for real:

- **[#947](https://github.com/wildcard/caro/issues/947)** — `[QA/WEBSITE_BROKEN_PROMISE]
  "find and kill the runaway process eating CPU" generates only ps aux | sort
  — drops the kill half`. Labels: `qa-routine`, `frustrated-beta`,
  `website-broken-promise`, `bug-intent-dropped`, `P0`. Body sourced from
  `proposed-issues/01-website-broken-promise-find-and-kill.md`.

## Issues updated today (existing reproductions)

None.

## Stale PRs cross-linked

Cross-link comments **posted** on:

- **[PR #567](https://github.com/wildcard/caro/pull/567#issuecomment-4324389985)** — *Improve UX for long-running commands* — open 97 days. Asked: "what's blocking this from landing?"
- **[Issue #449](https://github.com/wildcard/caro/issues/449#issuecomment-4324390060)** — *🎯 [EPIC] Exploration Agent: Complete Integration & Rollout* — open 102 days. Asked: "what's blocking integration & rollout?"

Both comments use the canonical agent comment template with prompt disclosure. The routine will flag these again tomorrow morning unless they merge or close.

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
RUN: 2026-04-26-smoketest | queries: 1 | findings: P0=1 P1=0 P2=0 | filed: #947 | cross-linked: #567, #449
NEXT: tomorrow's 5 AM real fire will (a) confirm the agent-registration preflight (added to /caro.frustrated-qa in this same PR), (b) test all 4 advertised queries, (c) re-flag PR #567 and Issue #449 if they remain stale.
SIGNAL: caro 1.3.0 fails the very first query on the landing page — "find and kill the runaway process" returns a half-pipeline that finds but never kills. Filed as #947, cross-linked to two stale items (#567 open 97d, #449 open 102d) that should have prevented this.
```

## Follow-ups completed in same session (post-`rp` action)

- ✓ Added agent-registration preflight (0b) and label preflight (0c) to `.claude/commands/caro.frustrated-qa.md` — guards against the "agent not found" gotcha discovered during this smoke test.
- ✓ Pre-created the 9 missing GH labels (`qa-routine`, `frustrated-beta`, `bug-fallback-overmatch`, `bug-undermatch`, `bug-intent-dropped`, `ux-no-streaming`, `ux-no-clarification`, `website-broken-promise`, `safety-missed-danger`). P0–P3 already existed.
- ✓ Filed [#947](https://github.com/wildcard/caro/issues/947) as the first real-world output of the routine.
- ✓ Cross-linked [PR #567](https://github.com/wildcard/caro/pull/567) and [Issue #449](https://github.com/wildcard/caro/issues/449) with stale-work nudge comments.
