# ADR-017: Cloud-Assisted Verification (execution-grounded eval, detonation, browser QA)

**Status**: Proposed
**Date**: 2026-09-06
**Deciders**: Autonomous session (per D5 protocol in
[`docs/decisions/2026-07-12-autonomous-mode-release-scope.md`](../decisions/2026-07-12-autonomous-mode-release-scope.md));
recorded in `COMPANY.md` decision log
**Target audience**: Community Edition (dev infrastructure); no user-facing change

## Context

Caro's two central claims were, until this ADR, never verified by execution:

1. **"The generated command actually works."** Every eval layer stops at
   string comparison (`src/evaluation/evaluators/correctness.rs`); the
   POSIX/BSD checks are regex heuristics; `tests/quickstart_scenarios.rs`
   admits "full execution testing requires execution module (future work)".
2. **"The 52+ safety patterns block real danger."** Safety is tested as
   *classification* only; `tests/red_team/` was promised (`HELP_WANTED.md`)
   and `docs/SECURITY-CHECKLIST.md` Gate 6 pointed at a directory that did
   not exist. Nothing was ever detonated to confirm a pattern's risk label
   corresponds to real destructive effect.

Separately, caro.sh (58 pages × 15 locales) had zero browser-level QA: the
documented L1 regression class (raw i18n keys rendered on production) could
only be caught by a human looking.

Two 2026 Cloudflare releases change the cost of closing these gaps:
[`@cloudflare/sandbox`](https://developers.cloudflare.com/sandbox/) (GA
Apr 2026: disposable real-Linux containers with an exec/file API) and
[Kitesurf](https://blog.cloudflare.com/kitesurf/) (Aug 2026 free beta: an
agent-first browser engine on Workers, CDP-compatible, 3–7× cheaper than
Chromium). The preview-stage
[`@cloudflare/computer`](https://github.com/cloudflare/computer) unifies
isolate/container execution behind one workspace API, and its isolate-shell
backend is [`just-bash`](https://github.com/vercel-labs/just-bash) — a
pure-TypeScript bash with an in-memory filesystem that also runs in plain
Node with **no cloud account at all**.

Governance context that shapes the decision: dev-harness/internal tooling is
explicitly exempt from `validation-discipline.md`; every user-facing
cloud-execution idea is hard-blocked at Gate 1 (0/20 transcripts);
[ADR-010](./ADR-010-bubblewrap-sandbox-execution.md) (Proposed) is the
standing *local* sandbox position; `COMPANY.md` commits to local-first
privacy and multi-vendor resilience; and the 2026-05-15 strategy memo
already scored Cloudflare sandboxes "Adjacent — complementary layers" with
Opportunity 4 ("Trusted Execution Target Manifest") as the blessed
product-side direction.

## Decision

Adopt cloud execution **as assistive verification infrastructure for the dev
harness**, behind a provider-neutral protocol, in three tiers. Build no
user-facing cloud execution.

### One protocol, three tiers

All execution flows through the JSONL contract in
[`tools/exec-harness/PROTOCOL.md`](../../tools/exec-harness/PROTOCOL.md)
(request: command + fixture files + timeout; response: exit code, output,
fs_diff, `unsupported` flag). The protocol — not any vendor SDK — is the
integration surface.

| Tier | Engine | Cost / secrets | Allowed role |
|------|--------|----------------|--------------|
| 0 | just-bash in plain Node (`tools/exec-harness/`) | $0, none | CI-blocking-eligible smoke; runs today |
| 1 | `@cloudflare/sandbox` **GA** containers (`tools/exec-harness/worker/`) | ~$5/mo Workers Paid + pay-per-use | non-blocking nightly; may graduate to blocking only after a quarter of nightly stability |
| 2 | `@cloudflare/computer` **preview** + Kitesurf **beta** | free (beta) | experimental / non-blocking lanes only, pinned versions, never load-bearing |

Consumers landed with this ADR:

- **Execution-grounded eval**: `TestCategory::Execution` +
  `ExecutionEvaluator` (`src/evaluation/evaluators/execution.rs`) grade
  *observed behavior* (exit code, stdout, filesystem effects) instead of
  command strings, via `--execution-tier tier0` on the eval CLI, against
  `tests/evaluation/datasets/posix/exec_grounded.json`. Grading philosophy:
  an engine gap is a SKIP, never a FAIL — pass-rates measure command
  quality, not harness availability.
- **Detonation lane**: `tests/red_team/` executes the dangerous-command
  corpus in egress-restricted throwaway containers (`/detonate` route) and
  fails on risk-label contradictions (a "critical" command that completes
  exit-0 with zero observable effects). Evidence artifacts per run.
  This makes SECURITY-CHECKLIST Gate 6 real.
- **Website structural QA**: `website/e2e/` drives Kitesurf over CDP for
  raw-i18n-key leak scans, link integrity, DOM structure, and
  renders-at-all screenshots. **Explicitly not visual/brand fidelity** —
  Kitesurf is not pixel-perfect; the claude-design Chromium flow
  (`.claude/rules/design-dialogue-protocol.md`) keeps brand audits. The
  Remotion demo re-render also stays on real Chromium. Kitesurf's REST
  screenshot endpoint MAY replace the human-local-Chrome loop for
  *structural* audits of public pages only.

### Relation to ADR-010 (bubblewrap)

Complementary, not competing. ADR-010 remains the plan for the *local*
execution boundary on a user's machine. This ADR's remote tiers answer
ADR-010's own stated trade-offs — "Linux-first" (a remote Linux container
serves macOS/Windows developers) and the rejected local-Docker-daemon
requirement (a remote API needs no daemon) — for *harness* use. When
ADR-010's `SandboxBackend` trait lands, `RemoteSandbox` (a thin reqwest
client speaking PROTOCOL.md) becomes one more implementation beside
`bubblewrap.rs`.

### Forward design (specced, not built): CaroML runbook verification

The one product-adjacent wedge, gated behind feature flag `sandbox-verify`
(never default): CaroML (shipped v1.4.0) compiles `.caro` tasks to
per-platform runbooks it cannot verify on platforms the user isn't on.
A `RemoteSandbox` implementation of ADR-010's trait shape lets
`regen_evaluator` record `verified_on: [linux]` in the lock and rank
`--explore` challenger variants by actual execution success. Egress
hygiene, in order: CaroML `validators/secrets.rs` (refuse on findings) →
`ContextSanitizer` (`src/backends/hybrid/sanitizer.rs`, the shipped ADR-015
pattern) → send → `restore()`. Opt-in flag + config only; macOS/Windows
runbooks surface as `unverified`, never silently trusted. This extends the
validated core loop (exempt per the v2.0 audit's extension principle); it
ships as a `feat:` PR with the full feature-evidence trio when built.

### Product-side posture

- **Parked as unvalidated hypotheses** (0/20, see
  [`hypothesis-ledger.md`](../discovery/hypothesis-ledger.md)):
  `sandbox-preview-ux` (verified preview instead of y/N via
  `SuggestedRouting`), `live-playground` (execution-backed try-caro; the
  current scripted mock stays a marketing surface), `agentic-loop`
  (execute→observe→retry). Standard Gate 1 evidence graduates them.
- **Blessed export direction** (strategy-memo Opportunity 4, future work):
  `caro-profile.json` — caro exports safety policy INTO sandbox runtimes
  (Cloudflare's included) with zero runtime dependency. Tracked separately;
  not part of this ADR's build.

### Non-negotiables (the "must-nots")

1. No Rust vendor SDK — the product's only HTTP surface stays `reqwest`
   (<50 MB binary, MSRV 1.85 untouched).
2. No default-on features; everything cloud-touching is opt-in.
3. No command/runbook content leaves a machine without secrets-validator +
   sanitizer + explicit opt-in.
4. Nothing CI-blocking may depend on preview (`computer`) or beta
   (Kitesurf) APIs. Only GA `@cloudflare/sandbox` may ever back a required
   check, and only after a quarter of nightly stability.
5. Vendor accounts and API tokens are created by humans (D5). All
   workflows skip green when secrets are absent.
6. Multi-vendor seam: PROTOCOL.md is the boundary. E2B, local Docker, or
   bwrap slot in as alternative protocol servers / `SandboxBackend` impls
   without touching evaluators or tests.

## Consequences

**Positive**: caro's core claims become measurable (execution-grounded pass
rates; detonation-verified risk labels); the missing `tests/red_team/`
exists; the L1 website regression class is caught nightly for free; tier 0
gives contributors a zero-setup local execution harness; the seam keeps
vendor lock-in at one directory.

**Negative / accepted costs**: ~$5/mo + sub-$10/mo container usage once
activated; tier-0 dialect gaps require honest `tier0` labels in datasets;
two dormant workflows until secrets exist; just-bash (Apache-2.0) and
`@cloudflare/sandbox` (worker-side) enter the dev-dependency surface
(license-compatible with AGPL-3.0; nothing links into the shipped binary).

**Risks**: Kitesurf/beta churn (mitigated: non-blocking + `BROWSER_MODE=chromium`
fallback via the same endpoint); `computer` API instability (mitigated:
experimental lane only); sandbox blind spots hiding real blast radius
(mitigated: the detonation suite fails on label contradictions so blind
spots surface as findings, and the canary tree + system-intact probe cover
the common classes).

## Activation

Human steps (D5): create the Cloudflare account, Workers Paid, API token;
add repo secrets `CARO_CF_ACCOUNT_ID`, `CARO_CF_API_TOKEN`, and after
worker deploy `CARO_DETONATION_URL`, `CARO_DETONATION_TOKEN`. Full
checklist: [`tools/exec-harness/worker/README.md`](../../tools/exec-harness/worker/README.md).
Until then: tier 0 and its eval lane run everywhere with zero secrets.

## See also

- [`tools/exec-harness/PROTOCOL.md`](../../tools/exec-harness/PROTOCOL.md) — the contract
- [ADR-010](./ADR-010-bubblewrap-sandbox-execution.md) — local sandbox (complementary)
- [ADR-015](./ADR-015-distributed-llm-backends-hybrid-privacy.md) — the sanitizer pattern P5 reuses
- [`market-scans/2026-05-15-ai-agent-strategy-memo.md`](../../market-scans/2026-05-15-ai-agent-strategy-memo.md) — Opportunity 4
- [`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md) — why product-side work is parked
- [`docs/discovery/hypothesis-ledger.md`](../discovery/hypothesis-ledger.md) — the parked hypotheses
