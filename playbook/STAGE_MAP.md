# Caro Stage Map

**Last Updated**: 2026-10-03
**Source framework**: [Anthropic, "The Founder's Playbook: Building an AI-native startup"](https://claude.com/blog/the-founders-playbook) (May 14, 2026)

This is Caro's location on the playbook's four-stage map. Each stage names
the goal, the exit criteria the playbook proposes, the evidence Caro has
against those criteria, and the failure modes we've already observed (or
risk observing). When two readers disagree about where Caro is, this
document is the tiebreaker.

> **TL;DR**: Caro is **late MVP → early Launch**. We have a working core
> loop, shipped distribution, and a public website, but we cannot yet show
> a flat retention curve or a defended PMF cohort. The Launch-stage exit
> gate is retention, not features.

---

## Stage 1 — Idea (✅ exited)

**Playbook goal**: Validate a problem worth solving.
**Playbook exit criterion**: 10 target users willing to pay.
**Playbook failure modes**: obsessing over solutions, surveys over
interviews, confusing competition-absence with opportunity, AI
confirmation bias.

### Caro's evidence

- **Problem statement**: codified in [`MISSION.md`](../MISSION.md) — "the
  barrier between intent and action shouldn't require memorizing syntax
  from decades of accumulated conventions"
- **Personas + jobs**: [`docs/PERSONAS_JTBD.md`](../docs/PERSONAS_JTBD.md),
  [`docs/JOBS_TO_BE_DONE.md`](../docs/JOBS_TO_BE_DONE.md)
- **Category positioning**: guardian-agent execution layer
  ([`docs/GUARDIAN_AGENT.md`](../docs/GUARDIAN_AGENT.md)) — distinct from
  policy / identity / audit categories
- **Willingness-to-pay signal**: live waitlist (~247 signups), ADR-001
  Community/Enterprise split with [enterprise value
  proposition](../docs/enterprise/ENTERPRISE-VALUE-PROPOSITION.md) drafted

### What we observed (failure-mode honesty)

- The 10-paying-users criterion is **not** yet evidenced by transcripts —
  the waitlist is a quantitative signal, not 10 first-hand interview
  records. This is discovery-debt we accept for the core product line;
  see `docs/discovery/` (lands Phase 3) for the format we'll use
  going forward.
- v2.0 product lines (Karo, Dogma, voice synthesis) **have not been
  through Stage-1 validation**. They are flagged for retroactive audit
  in [`docs/discovery/v2.0-validation-audit.md`](../docs/discovery/v2.0-validation-audit.md)
  (Phase 3).

---

## Stage 2 — MVP (✅ exited)

**Playbook goal**: Maintain engineering discipline under AI acceleration.
**Playbook exit criterion**: demonstrable core loop + minimal security
checklist (auth, API-key management, dependency audits).
**Playbook failure modes**: "demoware trap" (impressive demos that don't
survive real-world load); "mistaking building for validating".

### Caro's evidence

- **Core loop**: natural-language → safety-validated POSIX command,
  shipped through 8 minor releases (v1.0 → v1.4)
- **Quality bar**: 94.8% Command Success Rate (baseline from v1.1.0 beta,
  vs. 75% plan target); 700+ eval cases in `tests/evaluation/`
- **Engineering discipline**: tiered constitution
  ([`.claude/rules/constitution.md`](../.claude/rules/constitution.md)),
  Spec-Kit workflow, TDD-mandatory safety patterns, 33 domain agents,
  feature-branch enforcement via PreToolUse hook
- **Multi-backend resilience** (no single-model lock-in): MLX, CPU,
  Ollama, vLLM, OpenRouter, embedded — explicit insulation against the
  playbook's "single provider dependency" failure mode
- **Security gates**: `cargo audit` in CI, secret scanning, AGPL-3.0
  license discipline. Codified end-to-end in
  [`docs/SECURITY-CHECKLIST.md`](../docs/SECURITY-CHECKLIST.md) (Phase 4).

### What we observed (failure-mode honesty)

- The "demoware trap" is the watch-out for v2.0. CaroML preview shipped
  cleanly, but Karo distributed intelligence is at the demo stage where
  the playbook warns most loudly.
- "Mistaking building for validating" is a real risk: we have shipped
  features without explicit transcript evidence. The new
  `.claude/rules/validation-discipline.md` rule (Phase 2) is the
  systemic answer.

---

## Stage 3 — Launch (⏳ in progress — **current stage**)

**Playbook goal**: distinguish genuine traction from early enthusiasm.
**Playbook exit criteria**: (a) retention curve flattens, (b) users
proactively recall the product, (c) sustainable marginal cost of paid
conversion.
**Playbook failure modes**: "false prosperity trap" (mistaking early
enthusiasm for sustainable PMF); Sean Ellis test misapplied to the wrong
user cohort.

### Caro's evidence

- **Distribution shipped**: crates.io, Homebrew, npm, NuGet, WinGet
  pending, one-line install script at setup.caro.sh
- **Public surface**: caro.sh website (Astro 5, 58 pages, 15 locales),
  docs site, [PITCH_DECK.md](../docs/PITCH_DECK.md),
  [GTM_EXECUTION_PLAYBOOK.md](../docs/GTM_EXECUTION_PLAYBOOK.md),
  [GTM_STRATEGY_FRAMEWORK.md](../docs/GTM_STRATEGY_FRAMEWORK.md)
- **Launch operating system** (the playbook's term): Hermes does PR
  triage + competitive intel + weekly briefings; agents handle QA loop,
  PR management, beta feedback, idea sourcing. Inventory and gap
  analysis in [`docs/launch-os.md`](../docs/launch-os.md) (Phase 4).
- **Telemetry infrastructure**: opt-in, privacy-first, session events
  collected. Enough raw material to compute D1/D7/D30 retention; the
  computation itself is spec'd in
  [`docs/retention-dashboard-spec.md`](../docs/retention-dashboard-spec.md)
  (Phase 4) but not yet shipped.

### Exit criteria — where we are

| Playbook criterion | Caro status | Gap to exit |
| --- | --- | --- |
| Retention curve flattens | **No data** — telemetry collected, no dashboard yet | Ship `retention-dashboard-spec.md`; 90 days of opt-in cohort data |
| Proactive user recall | Anecdotal (GitHub stars, waitlist) | Sean-Ellis instrument behind a documented cohort definition |
| Sustainable CAC | **N/A** (no paid acquisition; no priced tier live yet) | Pricing draft lands Phase 5; Enterprise demand signal becomes the first paid-conversion telemetry |

### What we're watching for (failure-mode honesty)

- **False prosperity**: 247 waitlist + GitHub stars are interest, not
  retention. Until D7 is published, we don't claim PMF.
- **Sean Ellis misuse**: the playbook explicitly corrects the >40%
  rule — it requires a defended cohort definition. Our cohort is
  *not* "everyone who installed Caro"; it's "users who completed at
  least 5 commands in week 1". This is named in the retention spec.

### Failure modes observed (2026-10-03 audit)

Two Stage-3 failure modes have been confirmed in the four months since
this map was last reconciled. They are named here so the next decision
on either does not restart from zero.

- **Release-reset drift.** `chore(release): v1.5.0` landed on
  2026-07-12 (commit `6f23d37`) and bumped `Cargo.toml`, `README.md`,
  `homebrew-tap/README.md` and `nuget/tools/install.ps1` to 1.5.0. The
  tag + crates.io publish step — which only the owner can authorize —
  never ran. Twelve subsequent PRs (eval consensus labels / Pareto /
  ECE gate, Jev-system-one calibration + typed decisions, declared-
  limit enforcement, reqwest 0.12 / RUSTSEC-2026-0258/-0285 bump,
  Hebrew i18n overhaul, Cloudflare Pages prep, static-matcher
  Pattern 43, `ai --once` stdin fix, `config set/get` key parity,
  Rust 1.98.1 lint pin, Vercel/Astro drift guard, ponytail-reviewer
  agent) landed on `main` on top of that bump. Users on crates.io
  have been pinned to v1.4.0 since 2026-05-09 — nearly five months —
  while the repository state says 1.5.0. The failure mode is not
  "forgot to tag" but "the step with owner-only credentials sits
  outside the agent flow, so the version bump accumulates work
  without a backstop". The release-reset decision in
  [`docs/decisions/2026-10-03-release-reset-and-discovery-decision.md`](../docs/decisions/2026-10-03-release-reset-and-discovery-decision.md)
  names the options.
- **Discovery-rule inertia.** `.claude/rules/validation-discipline.md`
  (Gate 1 — twenty transcripts) was adopted on 2026-05-25, the same
  day this map was last updated. The companion discovery-debt epic
  (#1188) and its five sub-issues (#1189–#1193) have had **zero
  human or agent activity since 2026-05-31**. `docs/discovery/transcripts/`
  contains zero transcripts. Zero devil's-advocate reviews
  (Gate 4) have been filed. The `interview-enterprise-dashboard.md`
  script prepared in PR #1223 for exactly this purpose has never
  been used. The rule applies to the author who wrote it, and the
  author did not respond. The failure mode is not "nobody cares
  about discovery" but "an evidence requirement imposed on the
  owner without a shipped mechanism for the owner to meet it in
  single-digit minutes a week will be ignored, not fought". The
  three honest options (resource it, retire the epic as
  `not_planned`, pause with a 30-day deadline) are named in the
  decision doc above.

Both failure modes are **systemic**, not individual: they describe
process gaps that would reproduce under any owner. The release-reset
gap wants a `Needs human: credential` escape hatch; the discovery-
inertia gap wants either a time cost small enough to budget or an
explicit retirement so the open epic stops reading as commitment.

---

## Stage 4 — Scale (🔭 mapped, not entered)

**Playbook goal**: build a replicable "agentic operating system" across
business functions.
**Playbook exit criterion**: stable, orchestrated AI-powered workflows
across product, support, ops, marketing.
**Playbook failure modes**: single-provider dependency; inadequate
observability of multi-agent systems.

### Caro's planned shape

- **Commercial model**: dual-track per
  [ADR-001](../docs/adr/ADR-001-enterprise-community-architecture.md) —
  Community Edition stays AGPL-3.0 free-forever; Enterprise Edition is
  a premium plugin suite (CISO dashboard, centralized governance, audit
  trails, IT rollout integration). Value proposition: one prevented
  security incident pays for years of licensing.
- **Multi-agent ops**: Hermes already runs as the non-coding strategic
  layer ([`.hermes/AGENT.md`](../.hermes/AGENT.md)). Scale-stage work
  extends this to support intake (Chat), knowledge management (Cowork
  as candidate), continuous product iteration (Code — current). See
  [`docs/agentic-stack.md`](../docs/agentic-stack.md) (Phase 4).
- **Multi-model resilience**: already real, called out explicitly above
- **Multi-agent observability**: gap — `docs/launch-os.md` names this as
  a Scale-stage prerequisite

### What this stage looks like for a guardian-agent execution layer

The 9 consumer opportunities the playbook lists (health, careers,
relationships, money, parenting, legal, life sciences, +2) are not
Caro's category. Caro's Scale-stage opportunity is **horizontal
adoption inside organizations that already deploy AI coding agents** —
the 60% of orgs that have no AI governance policy
([per ENTERPRISE-VALUE-PROPOSITION.md](../docs/enterprise/ENTERPRISE-VALUE-PROPOSITION.md)).

---

## How to update this file

- Cite evidence, not vibes. Every status claim must link to a file or
  metric that backs it.
- When a stage transition happens (e.g. retention curve flattens), the
  PR that ships the proof updates this map in the same commit. No
  silent stage promotions.
- When the playbook is amended by Anthropic or by us, this map is the
  first thing that gets reconciled.
