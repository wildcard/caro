# Launch Operating System

**Last Updated**: 2026-05-25
**Source framework**: [Anthropic's Founder's Playbook, Stage 3 — Launch](https://claude.com/blog/the-founders-playbook)

The playbook's term for the constellation of agents/skills/automation
that handles support, content, community, ops, and outbound discovery
during the Launch stage — so the founder doesn't burn out doing 80%
of it manually. This document is Caro's inventory: what we already
have, what's a gap, and what's deliberately out of scope.

> **TL;DR**: Caro has ~70% of a Launch OS. Hermes covers PR triage,
> competitive intel, weekly briefings, and integration health.
> Specialist agents and skills cover QA loop, beta feedback fixing,
> PR management, idea sourcing, content marketing. The four named
> gaps below are the highest-leverage Launch-stage builds.

## Inventory

### What we already have

| Playbook capability | Caro implementation | Lives in |
| --- | --- | --- |
| PR triage (daily) | Hermes PR digest + routing comments | [`.hermes/AGENT.md`](../.hermes/AGENT.md) §1; `pr-management-loop` skill |
| Competitive intel (weekly) | Hermes weekly market scan | [`.hermes/AGENT.md`](../.hermes/AGENT.md) §2 |
| Cross-agent coordination | Hermes branch-conflict + duplicate-work detection | [`.hermes/AGENT.md`](../.hermes/AGENT.md) §3; [`.hermes/PROTOCOL.md`](../.hermes/PROTOCOL.md) |
| Integration health monitoring | Hermes nightly + `caro-integrator-nightly` | [`.hermes/AGENT.md`](../.hermes/AGENT.md) §4 |
| Release readiness assessment | `caro.release.acceptance` skill + Hermes audit | [`.claude/skills/`](../.claude/skills/) |
| Executive briefings (weekly) | Hermes weekly digest in `.hermes/digests/` | [`.hermes/AGENT.md`](../.hermes/AGENT.md) §6 |
| QA loop (continuous) | `qa-automation-loop` skill + `caro-frustrated-beta` agent | [`.claude/skills/`](../.claude/skills/), [`.claude/agents/`](../.claude/agents/) |
| Beta-feedback triage | `beta-feedback-fixer` skill | [`.claude/skills/`](../.claude/skills/beta-feedback-fixer/) |
| Content marketing | `ai-marketing-engineering` + `social-queue` skills | [`.claude/skills/`](../.claude/skills/) |
| Idea sourcing | `idea-sourcing-loop` skill | [`.claude/skills/`](../.claude/skills/) |
| Continuous product iteration | `caro-coder-loop` + Claude Code | [`.claude/skills/`](../.claude/skills/caro-coder-loop/) |
| Translation pipeline | `gh workflow run translate.yml` (15 locales) | [`website/I18N_TRANSLATION_GUIDE.md`](../website/I18N_TRANSLATION_GUIDE.md) |
| Beta cycle orchestration | `beta-test-cycles` + `unbiased-beta-tester` skills | [`.claude/skills/`](../.claude/skills/) |
| Adversarial review (NEW) | `devils-advocate` agent | [`.claude/agents/devils-advocate.md`](../.claude/agents/devils-advocate.md) |
| User discovery (NEW) | `caro.discovery` skill + `docs/discovery/` | [`.claude/skills/caro.discovery/`](../.claude/skills/caro.discovery/) |

### What's a gap

| Playbook capability | Why it matters at Launch | Current state | Next step |
| --- | --- | --- | --- |
| **Retention dashboard** | Stage-3 exit gate is "retention curve flattens"; we currently cannot compute D1/D7/D30 | Telemetry events collected (opt-in), no dashboard | Implement against [`retention-dashboard-spec.md`](./retention-dashboard-spec.md) |
| **GitHub Discussions response triage** | First-touch latency on Discussions degrades community signal; founder shouldn't be the triage layer | Manual founder + occasional Hermes nudge | Spin out a `discussions-triage` skill modeled on `pr-management-loop` |
| **Outbound discovery automation** | Gate-1 evidence (20 transcripts per hypothesis) requires identifying candidate users from public signals | Manual + Hermes competitive intel scans | Extend Hermes weekly scan with `candidate-users` output section |
| **Multi-agent observability** | Hermes coordinates ~15 agents/skills; failure modes (agent drift, eval regression, conflicting outputs) are not currently instrumented | None | Stage-4 prerequisite; spec lives in a future ADR |

### What's deliberately out of scope at Launch

| Playbook capability | Why we're not building it yet |
| --- | --- |
| Live Discord with engagement automation | Community at our scale (~247 waitlist) fits in GitHub Discussions + DMs; Discord adds ops overhead that exceeds its value until a step-change in community size |
| Personalized email lifecycle nurture | The waitlist Turso backend captures source / interests; one-touch follow-up is a Phase 5 pricing-page wedge, not a full nurture pipeline |
| Outbound sales motion | ADR-001 Enterprise edition is in research; sales work is gated by pricing-page demand signal |
| Cowork-based knowledge management | The playbook recommends Cowork for internal KM; we've documented in [`agentic-stack.md`](./agentic-stack.md) that Cowork's audit-log limitation makes it unsuitable for our Enterprise-bound positioning — re-evaluate after the audit-log gap closes |

## How the existing pieces compose

```
Founder / contributor
        │
        ▼
   beads queue ◄────────────── idea-sourcing-loop
        │                              ▲
        ▼                              │
   coder-loop (Claude Code) ──────► PR opens
        │                              │
        ▼                              ▼
   PR opens ──► pr-management-loop ──► Hermes triage ──► reviewer
                       │                    │
                       ▼                    ▼
                  qa-automation-loop   .hermes/digests/
                       │                    │
                       ▼                    ▼
                  release-prepare       executive briefing
```

The user-facing surface (caro.sh + the CLI itself) sits below this
diagram; telemetry flows back into the retention dashboard (gap) and
informs the next discovery cycle.

## When you operate on this document

- **Adding a new agent or skill**: append a row to the inventory
  with a link to its definition file. If it closes a gap, move the
  gap to "What we already have".
- **Identifying a new gap**: append to "What's a gap" with a one-
  sentence rationale and a concrete next step. Don't list gaps
  without ownership.
- **Closing a gap**: PR that closes the gap removes the row from
  "What's a gap" and adds it to "What we already have", in the same
  commit.
- **Reclassifying scope**: if "deliberately out of scope" stops
  being so, move the row up and explain in the PR description why
  the calculus changed.

## Stage-4 (Scale) extensions

When Caro graduates to Stage 4, this document expands to cover:

- Support intake (Claude Chat as entry point per [`agentic-stack.md`](./agentic-stack.md))
- Multi-agent observability (the named gap above becomes a build)
- Enterprise deployment ops (rollout automation, audit-trail
  forwarding, policy distribution — per ADR-001)
- Hiring + onboarding (the playbook's Stage 4 includes "team
  expansion"; for Caro this means contributor onboarding, not
  full-time hires)

Until then, those rows are tracked elsewhere or simply not
applicable.

## See also

- [`.hermes/AGENT.md`](../.hermes/AGENT.md) — the strategic-intel
  agent that runs most of the existing Launch OS
- [`.hermes/PROTOCOL.md`](../.hermes/PROTOCOL.md) — inter-agent
  communication contract
- [`retention-dashboard-spec.md`](./retention-dashboard-spec.md) —
  the Stage-3 exit-gate measurement spec
- [`agentic-stack.md`](./agentic-stack.md) — how Caro uses the
  Anthropic product matrix internally
- [`SECURITY-CHECKLIST.md`](./SECURITY-CHECKLIST.md) — the MVP-stage
  security gates that the Launch OS does NOT relax
- [`playbook/STAGE_MAP.md`](../playbook/STAGE_MAP.md) — where this
  Launch OS is taking us next
