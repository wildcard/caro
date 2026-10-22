# Caro Playbook

> The founder-arc adaptation of [Anthropic's "The Founder's Playbook:
> Building an AI-native startup"](https://claude.com/blog/the-founders-playbook)
> for Caro specifically.

## Why this directory exists

Caro is a real product (1.4.0 GA, May 2026) with a real distribution
footprint (crates.io, Homebrew, npm, NuGet) and real users (~247 waitlist,
public telemetry baseline of 94.8% CSR). It is **also** a founder-arc in
motion: dual-track Community / Enterprise per
[ADR-001](../docs/adr/ADR-001-enterprise-community-architecture.md),
multi-agent operating layer, deliberate stage progression.

The Anthropic playbook gave us a clean four-stage frame to name where we
are and what comes next. This directory holds the Caro-specific
adaptation. It does **not** restate the playbook (read the original);
it maps the playbook onto Caro's evidence and our own existing artifacts.

## Files

- **[`STAGE_MAP.md`](./STAGE_MAP.md)** — Caro's location in the
  Idea/MVP/Launch/Scale framework, with exit criteria, evidence, and
  failure modes we've observed
- **[`../COMPANY.md`](../COMPANY.md)** — One-page founder-arc narrative:
  how Caro becomes a company, the dual-track model, what Scale looks
  like for our category
- **[`../MISSION.md`](../MISSION.md)** — Mission + values (preceded the
  playbook adaptation; remains canonical)

## What the playbook gave us (and what it didn't)

### What we adopted

| Playbook concept | Caro adaptation |
| --- | --- |
| 4-stage map | [`STAGE_MAP.md`](./STAGE_MAP.md) |
| "Mistaking building for validating" | [`.claude/rules/validation-discipline.md`](../.claude/rules/validation-discipline.md) |
| Counter AI confirmation bias | [`.claude/agents/devils-advocate.md`](../.claude/agents/devils-advocate.md) |
| 20-transcript validation rule | [`docs/discovery/`](../docs/discovery/) directory + [discovery skill](../.claude/skills/caro.discovery/SKILL.md) |
| Demoware-trap gate | "what breaks at 100 real users" section required in feature specs (see validation-discipline rule) |
| Sean Ellis test (used correctly) | [`docs/retention-dashboard-spec.md`](../docs/retention-dashboard-spec.md) |
| Launch operating system | [`docs/launch-os.md`](../docs/launch-os.md) — inventory + gaps |
| MVP-stage security checklist | [`docs/SECURITY-CHECKLIST.md`](../docs/SECURITY-CHECKLIST.md) |
| Product matrix (Chat/Cowork/Code/Platform) | [`docs/agentic-stack.md`](../docs/agentic-stack.md) |
| Multi-model resilience (no vendor lock-in) | Already real; surfaced in marketing copy |

### What we did NOT adopt

- **The 9 consumer opportunities** (health, careers, relationships, money,
  parenting, legal, life sciences, +2). Caro is in the developer-tool
  guardian-agent category. We acknowledge the playbook's framing but
  do not pivot toward consumer verticals.
- **Fundraising posture**. The playbook is agnostic about funding source;
  Caro's [`COMPANY.md`](../COMPANY.md) names dual-track community/
  enterprise but commits to no specific funding round.
- **Generic Cowork advocacy**. The playbook (and reviewers) noted that
  Cowork activity is excluded from audit logs and compliance APIs,
  making it unsuitable for some regulated workloads. Our
  [`agentic-stack.md`](../docs/agentic-stack.md) names this caveat
  explicitly and routes regulated work to Claude Code + the enterprise
  audit trail per ADR-003.

## How to use this directory

- **As a contributor**: read `STAGE_MAP.md` to understand where Caro
  is and what stage gate comes next. Your feature should make progress
  against the current stage's exit criteria, not the next one's.
- **As a reviewer**: a PR that claims to advance a stage should update
  `STAGE_MAP.md` in the same commit with the evidence.
- **As a founder making a strategic decision**: this directory is the
  reference frame. If a decision doesn't fit the frame, update the
  frame — don't ignore it.

## Living document

The playbook is dated May 14, 2026. Our adaptation is dated May 25,
2026. If Anthropic publishes a v2 of the playbook, this directory gets
reconciled in a single PR (one commit per affected file), with the
reconciliation diff visible in the PR description.
