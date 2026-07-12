# Caro's Agentic Stack

**Last Updated**: 2026-05-25
**Source framework**: [Anthropic's Founder's Playbook — Product Matrix](https://claude.com/blog/the-founders-playbook)

This document is how Caro itself uses the Anthropic product matrix
(Chat, Cowork, Code, Platform) for its own operations. The playbook
recommends a particular product per use-case; we record where we
follow the recommendation, where we deviate, and why.

> **TL;DR**: We are heavy on **Code** (the agentic core of Caro's
> own development), starting on **Platform** (the multi-backend
> InferenceBackend abstraction inside Caro itself), light on **Chat**
> (used informally), and **deliberately not adopting Cowork** for
> internal KM until its audit-log limitation closes.

## Product matrix

### Claude Code — heavy use ✅

**Playbook recommendation**: "Continuous product iteration."
**Caro adoption**: primary. The entire `.claude/` directory is
Code-shaped (agents, skills, commands, rules). Every PR in this repo
is touched by Claude Code via the `caro-coder-loop` skill,
`pr-management-loop` skill, or direct human-in-the-loop sessions.

**Where it works**:
- The 33+ domain agents in [`.claude/agents/`](../.claude/agents/)
  give Code coherent personas to spawn for specialized work (Rust
  CLI architect, safety pattern developer, frustrated beta tester,
  release expert, design engineer, devil's advocate, etc.)
- The tiered constitution
  ([`.claude/rules/constitution.md`](../.claude/rules/constitution.md))
  encodes precedence so parallel sessions don't tear at each other
- Spec-Kit + beads queue keeps work units coherent across sessions

**Where it doesn't**:
- Multi-session coordination of edits to the same file → covered by
  the worktree convention + git-workflow rule
- Long-running agent loops without supervision → covered by Hermes
  + integration-health monitoring, but the failure mode is real
  (see Stage-4 "multi-agent observability" gap in
  [`launch-os.md`](./launch-os.md))

### Claude Platform (API + multi-agent orchestration) — starting ✅

**Playbook recommendation**: "Backend API invocation and multi-agent
orchestration."
**Caro adoption**: starting. Caro's own backend abstraction
(`InferenceBackend` trait in `src/inference/mod.rs`) is the
Platform-shaped layer *inside Caro the product* — it supports
multiple model providers with runtime switching, deliberately
insulating against the playbook's "single provider dependency"
Stage-4 failure mode.

For operational AI (not product AI):
- We use Anthropic's APIs from Claude Code and from the
  `ai-marketing-engineering` skill
- We use OpenRouter as an alternate path for the embedded backend's
  cloud fallback
- Multi-agent orchestration *inside the project's dev ops* is the
  Hermes agent + the constellation of `.claude/skills/`; Hermes is
  the orchestration layer

**Where we deliberately differ from the playbook**:
- The playbook treats Platform as the company's *operational* AI
  layer. Caro treats it as both operational AND the product
  itself — `caro` the CLI IS a Platform-shaped consumer (multi-
  backend, runtime-switchable). This makes us more sensitive to
  vendor-lock-in failure modes than the playbook's median reader.

### Claude Chat — light, informal use ✅

**Playbook recommendation**: "Customer support entry point."
**Caro adoption**: informal. We don't currently route support intake
through a Chat instance. GitHub Issues + GitHub Discussions + DMs
are the support surface. Chat-shaped automation may land at Stage-4
scale.

**Where it works**:
- Founder + maintainer occasional Chat use for one-off research,
  competitive scans, drafting copy
- The
  [`claude-design-frontend-engineer`](../.claude/agents/claude-design-frontend-engineer.md)
  agent's bidirectional dialogue with the UI/UX persona at
  claude.ai/design is Chat-shaped (governed by
  [`.claude/rules/design-dialogue-protocol.md`](../.claude/rules/design-dialogue-protocol.md))

**Where it doesn't**:
- Public-facing support intake — defer to Stage-4
- Automated triage of GitHub Discussions — gap in `launch-os.md`;
  may be addressed via Chat or via Code, TBD

### Claude Cowork — deliberately not adopted ⛔

**Playbook recommendation**: "Internal knowledge management."
**Caro adoption**: **deliberately not adopted at this time.**

**Why**:
The playbook recommends Cowork for internal KM and even for SOC 2 /
GDPR / HIPAA compliance workstreams. Reviewers of the playbook noted
that **Cowork activity is excluded from audit logs and compliance
APIs**, making it structurally unsuitable for regulated workloads —
despite the playbook's recommendation. (See the techtimes.com
critical review cited in the playbook's research notes.)

Caro's positioning is split:

- **Community Edition**: local-first, single-user, no organizational
  audit need. Cowork-shaped KM isn't load-bearing here; we use the
  repo itself (docs/, .claude/, .hermes/, beads) as the knowledge
  base.
- **Enterprise Edition** (ADR-001): explicitly designed for
  regulated-workload customers — CISOs, audit-trail forwarding,
  centralized policy distribution. We **cannot** sit our own
  internal KM on a tool that's structurally unsuitable for the
  customer's audit requirements; that would be hypocritical and
  would create awkward conversations.

**Re-evaluation trigger**: if Anthropic closes Cowork's audit-log
gap and publishes a compliance API, we re-evaluate this decision.
Until then, our KM lives in the repo, and Hermes serves the
"synthesis across knowledge" role Cowork might otherwise serve.

## Internal AI ops by surface area

| Surface | Heavy use | Light use | Not used |
| --- | --- | --- | --- |
| Product development (`src/`) | Code, Platform (inside Caro) | — | Cowork, Chat |
| Documentation (`docs/`, `playbook/`, `website/`) | Code | Chat (drafts) | Cowork, Platform |
| Marketing copy + content | Code (via `ai-marketing-engineering` skill) | Chat | Cowork |
| Strategic synthesis | Hermes (on Platform) | Chat (founder) | Cowork |
| Translations | `gh workflow run translate.yml` (Platform) | — | Code, Cowork, Chat |
| Beta testing | Code (via `caro-frustrated-beta`, `unbiased-beta-tester` skills) | — | Cowork, Chat |
| Customer support intake | (manual via GH Discussions / DMs) | — | Code, Chat, Cowork (yet) |

## The "dogfooding" claim

Caro's category is **execution safety for AI-generated commands**.
Our entire dev process is AI-generated commands meeting an execution
layer (the repo, the CLI). We are an honest user of our own product
shape:

- Every PR has shell commands in it (build, test, lint, deploy)
- Every commit is created via tooling that runs shell commands
- Every release runs a sequence of validated shell commands per
  `.claude/rules/release-version-alignment.md`

We don't (yet) use Caro itself to safety-validate the shell commands
inside our own dev loop. That's a dogfooding gap; tracked as a
discovery candidate (low priority, since maintainers are themselves
the safety check).

## Where this stack is going

When Caro graduates to Stage 4 ([`playbook/STAGE_MAP.md`](../playbook/STAGE_MAP.md)),
this document expands:

- **Chat** picks up customer-support intake routing
- **Platform** picks up Enterprise rollout automation (per ADR-001)
- **Cowork** — re-evaluated only if the audit-log limitation closes
- **Code** — continues, but with multi-agent observability per the
  gap in `launch-os.md`

## See also

- [`launch-os.md`](./launch-os.md) — the broader Launch-stage
  agent/skill inventory; this document focuses on the Anthropic
  product matrix specifically
- [`.hermes/AGENT.md`](../.hermes/AGENT.md) — the strategic-intel
  agent that's the closest thing to our "Cowork" today
- [`.claude/rules/external-sdk-integration.md`](../.claude/rules/external-sdk-integration.md) —
  multi-vendor resilience as code (the same shape as the playbook's
  "no single provider dependency" advice)
- [`playbook/STAGE_MAP.md`](../playbook/STAGE_MAP.md) — where this
  stack is taking us next
- The Founder's Playbook critical review (techtimes.com) — source of
  the Cowork audit-log caveat we honor
