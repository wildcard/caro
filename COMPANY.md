# Caro as a Company

**Last Updated**: 2026-07-12
**Companion docs**: [`MISSION.md`](./MISSION.md) (values),
[`playbook/STAGE_MAP.md`](./playbook/STAGE_MAP.md) (where we are),
[`docs/PITCH_DECK.md`](./docs/PITCH_DECK.md) (long-form pitch)

This is the one-page version of how Caro becomes a company. It exists
to give a contributor, a reviewer, or a future-self enough context to
make a strategic decision in five minutes.

## Category

Caro is a **guardian-agent execution layer**. Not policy. Not identity.
Not audit. The narrow slice between "an LLM (or a human) drafted a
shell command" and "that command runs on a real machine". Today that
slice is mostly trust-by-default — we sit in it and apply deterministic
validation against 52+ dangerous patterns, CVE rules, and platform-
aware safety checks.

The category is named in [`docs/GUARDIAN_AGENT.md`](./docs/GUARDIAN_AGENT.md).
Gartner's 2026 Guardian Agents framing puts this layer at the same
infrastructure tier as policy and audit; we differ by being the
**execution-time** enforcement point, not the post-hoc one.

## Two products, one core

Per [ADR-001 (Enterprise / Community Architecture)](./docs/adr/ADR-001-enterprise-community-architecture.md):

### Community Edition — free forever, AGPL-3.0

- The Caro CLI everyone installs from crates.io / Homebrew / npm /
  NuGet / setup.caro.sh
- Full safety validator, all backends, full CaroML, full telemetry
  (opt-in, local-first)
- The proof that the category exists, and the on-ramp for everything
  else

### Enterprise Edition — premium plugin suite

- CISO dashboard, centralized policy distribution, audit-trail
  forwarding, machine correlation, IT rollout integration
- Implemented as plugins (per [ADR-001](./docs/adr/ADR-001-enterprise-community-architecture.md))
  so the open-source core stays unburdened
- The value proposition is risk math: one prevented security incident
  pays for years of licensing — full argument in
  [`docs/enterprise/ENTERPRISE-VALUE-PROPOSITION.md`](./docs/enterprise/ENTERPRISE-VALUE-PROPOSITION.md)
  and the defensive moat in [`docs/enterprise/MOAT.md`](./docs/enterprise/MOAT.md)

The community core is **never** crippled to upsell. The enterprise
plugins extend; they do not gate.

## What stage we're in

**Late MVP → early Launch.** See
[`playbook/STAGE_MAP.md`](./playbook/STAGE_MAP.md) for the evidence.
The exit gate is **retention**, not features. Until we publish a
defended D7 retention curve, we do not claim product-market fit.

## What "Scale" looks like for Caro

Scale isn't more features. Scale is:

1. **Horizontal adoption in orgs that already deploy AI coding agents** —
   the 60% of organizations that ship AI-assisted code with no
   governance policy. Caro plugs into the existing AI-agent workflow as
   the execution-safety layer.
2. **Replicable agentic operating system inside the company** — Hermes
   (strategic intel), frustrated-beta (QA), coder-loop (development),
   pr-management-loop (PR ops), and friends already cover most of what
   the playbook calls a "launch operating system". Scale extends this
   to support intake and outbound discovery. Gaps: see
   [`docs/launch-os.md`](./docs/launch-os.md).
3. **Multi-model, multi-vendor resilience by construction** — already
   real; explicit insulation against the playbook's "single provider
   dependency" failure mode. We support MLX, CPU, Ollama, vLLM,
   OpenRouter, and embedded models, swappable at runtime.

## What we will not do

- **Crippling the community core to upsell.** Enterprise extends;
  it does not gate.
- **Telemetry-as-business-model.** Telemetry is opt-in, local-first,
  privacy-audited. It exists to make Caro better, not to be the
  product. See [`docs/TELEMETRY.md`](./docs/TELEMETRY.md).
- **Pivoting into a consumer category.** The Anthropic playbook lists
  9 consumer opportunities (health, careers, money, etc.). They're
  real opportunities. They're not ours.
- **Closed source for safety patterns.** Our 52+ dangerous-pattern
  library is in `src/safety/patterns.rs` under AGPL-3.0. If we close
  it, the category we created collapses back into vendor opinion.

## Funding

This document does not commit to a specific funding posture. Whether
Caro stays bootstrapped, raises a seed, or remains an open-source
project with an enterprise sponsor is a founder decision tracked
outside this repo. What this document does commit to is that any
funding choice must be compatible with the four "what we will not do"
items above. If a term sheet would require closing the safety patterns,
the term sheet loses.

## Decision log

Strategic decisions made while routing around normal review (per
`validation-discipline.md`, these get recorded so the next
decision-maker inherits the reasoning):

- **2026-07-12 — v1.5.0 now; v2.0.0 stays gated; autonomous-ops
  protocol.** With the owner preoccupied, an autonomous session chose
  to ship two months of unreleased work as a semver-honest v1.5.0
  rather than a hollow "v2.0.0 - Distributed Autonomy" (whose 5
  defining features are all at 0/20 discovery transcripts), and
  codified the evidence/demo/regression-guard requirement for every
  feature PR. Full record with alternatives:
  [`docs/decisions/2026-07-12-autonomous-mode-release-scope.md`](./docs/decisions/2026-07-12-autonomous-mode-release-scope.md).

- **2026-09-06 — Cloudflare as assistive verification infrastructure,
  not a runtime dependency.** Adopted Cloudflare compute (Sandbox
  containers, Browser Run/Kitesurf, `@cloudflare/computer` preview) for
  the *dev harness only*: execution-grounded eval, safety-corpus
  detonation, and structural website QA — internal tooling, exempt from
  validation gates. The product keeps zero runtime dependency on any
  cloud vendor; every user-facing cloud-execution idea was parked as an
  unvalidated hypothesis (`sandbox-preview-ux`, `live-playground`,
  `agentic-loop` — 0/20 each), consistent with the local-first and
  multi-vendor-resilience commitments above. Vendor account and API
  tokens are human-created (per the 2026-07-12 D5 human-required
  limits). Architecture, tier policy (GA-only for anything that could
  gate CI), and vendor seam:
  [`docs/adr/ADR-017-cloud-assisted-verification.md`](./docs/adr/ADR-017-cloud-assisted-verification.md).

## How to contribute to "the company part"

- **Marketing / DevRel work** lives in the
  [Marketing & DevRel project](https://github.com/users/wildcard/projects/3)
- **Enterprise positioning** is iterated in
  [`docs/enterprise/`](./docs/enterprise/) — RFCs welcome
- **Discovery work** (interviews, validation evidence) lands in
  [`docs/discovery/`](./docs/discovery/) per the contributor guide
  there
- **Strategic memos** to Hermes go through `.hermes/messages/` per
  [`.hermes/PROTOCOL.md`](./.hermes/PROTOCOL.md)
