# Validation Discipline

**APPLIES TO**: Any new product line, any major feature spec (>2-week
build), any spec that introduces a new user-facing capability.
**DOES NOT APPLY TO**: bug fixes, refactors, dependency updates,
documentation, internal tooling.

Adapted from [Anthropic's Founder's Playbook](https://claude.com/blog/the-founders-playbook)
(May 14, 2026). The playbook's central warning: when a prototype takes
an afternoon instead of a quarter, founders mistake the existence of
the prototype for proof that anyone wants it. The playbook calls this
**"mistaking building for validating"**. AI amplifies it because models
trained to be helpful will happily justify whatever idea you ask them
to justify — confirmation bias on demand.

This rule is the systemic answer: evidence requirements that gate
features at the spec stage, before code starts.

## The Five Gates

A new product line or major feature spec must clear all five gates
before its implementation PR can be opened. The gates are cumulative,
not alternatives.

### Gate 1 — Twenty Transcripts

The spec must reference **20 first-hand user conversation transcripts**
collected before the spec was written. Transcripts live in
[`docs/discovery/transcripts/`](../../docs/discovery/) under the
anonymization rules documented there.

- "Conversation" means a real interview (synchronous or async chat),
  not a survey response. Surveys are quantitative confirmation, not
  qualitative validation.
- "First-hand" means *you* talked to them. Not "20 GitHub issues we
  scanned". Not "20 tweets we found". Not "Hermes summarized 20
  competitive products". The transcript carries the timestamp, the
  participant's anonymized handle, and the raw exchange.
- The 20 transcripts must collectively name **at least 3 distinct
  user pain patterns**, not 20 variations of the same one. Repetition
  inside a single pattern is noise, not signal.

The number 20 is deliberate. 5 is anecdote; 10 is selection bias; 20
is enough to see the pattern repeat in voices you didn't pre-select.

### Gate 2 — No Surveys (As The Sole Evidence)

A survey result, NPS score, "would you use this?" form, or waitlist
signup count is **never** sufficient evidence by itself. These
instruments answer "do users say yes when asked?", which is the wrong
question. The right question is "what do users do when they have the
problem?" — and that answer comes from interviews.

Surveys are welcome as **quantitative follow-up** after qualitative
validation: once the 20 transcripts surface 3 pain patterns, a survey
can size them. But the survey alone closes no gate.

### Gate 3 — Demoware-Trap Section

Every feature spec must contain a section titled
**"What breaks at 100 real users"** that names:

- The data model assumption that holds at demo scale but fails at
  100 concurrent users / 100 GB of state / 100 platforms
- The failure mode if the assumption breaks
- The instrumentation that would tell us the assumption is breaking
- The fallback if the failure mode triggers in production

If the answer is "nothing breaks", the section must explain *why*
nothing breaks (e.g. "feature is local-first, single-user, no shared
state"). "Nothing breaks" without explanation fails the gate.

The playbook's demoware-trap warning: "impressive demos with
underlying data models that cannot handle real-world usage at scale".
This section forces the failure mode into the spec before code is
written, when it's cheap to redesign.

### Gate 4 — Devil's-Advocate Review

Any AI-drafted or AI-assisted feature proposal must pass review by the
[`devils-advocate`](../agents/devils-advocate.md) agent before merge.
The agent is read-only, adversarial by construction, and instructed
to interrogate (not justify) the proposal.

- The review goes in the PR as a comment block titled
  `## Devil's Advocate Review`
- It must include the agent's concrete objections, not just "passed"
- The spec author addresses each objection in the PR description (either
  by revising the spec or by recording the deliberate decision to ignore
  the objection with a one-line justification)

The point isn't to block features. The point is to surface
confirmation bias before it ships.

### Gate 5 — Sean Ellis With A Defended Cohort

If the spec claims **product-market fit** (explicitly or implicitly —
"users love this", "this is what they've been asking for", "high
retention expected"), the claim must reference:

- The Sean Ellis >40%-would-be-very-disappointed result, AND
- A **defended cohort definition** that names: who counts as a user
  for this measurement, why that cohort is the right one, and what
  signal excludes a user from the cohort

The playbook explicitly corrects the common Sean Ellis misuse:
"requires an adequate sample drawn from the right user cohort to
mean anything". A 47% result among "everyone who installed and
opened once" is meaningless. A 47% result among "users who completed
≥5 commands in week 1" is signal.

PMF claims without both ingredients get rejected and reframed as
"early enthusiasm" until the gate is cleared.

## How to use this rule

### As a spec author

- Plan the 20 interviews *before* you start writing the spec, not
  after. The transcripts shape the spec.
- The five gates aren't a checklist you tick at the end. They're
  the structure of the spec itself.
- If clearing the gates feels expensive, the feature probably
  doesn't pass them — that's the rule working as intended.

### As a reviewer

- A PR that opens an implementation against an ungated spec gets
  marked **blocked** with a link to this rule. The spec PR must
  land first.
- The five gates are evidence requirements, not opinions. "I don't
  buy the 20 transcripts" is the kind of feedback that lands as a
  concrete count (e.g. "12 of these are from the same Discord
  channel, gate 1 isn't actually met").

### As a founder making a strategic decision

- This rule applies to *Caro's own* feature planning. Every v2.0
  item (Karo, Dogma, voice synthesis, self-healing, local context
  indexing) gets audited under it; see
  [`docs/discovery/v2.0-validation-audit.md`](../../docs/discovery/).
- If a strategic decision routes around the rule ("we're shipping
  this without 20 transcripts because…"), the routing-around gets a
  recorded decision in `COMPANY.md` so the next decision-maker
  inherits the reasoning.

## What this rule does NOT do

- It does not require interviews for bug fixes, refactors,
  performance work, security patches, or internal tooling. Those are
  responses to evidence we already have (broken thing, slow thing,
  vulnerability, friction).
- It does not require interviews to ship a *prototype*. It requires
  interviews to ship a **feature spec** that says "we should build
  X". You can prototype anything; you can graduate-to-feature only
  what's gated.
- It does not require interviews for the **core product loop** of
  natural-language → safety-validated POSIX command. That ship has
  sailed (1.4.0 GA, 94.8% CSR, 247 waitlist signups). The rule
  applies forward from May 25, 2026.

## Why this matters

The playbook's framing: AI removed three historical startup
bottlenecks (capital, headcount, technical skill). The new
constraint is **what a founder chooses to build**. Without an
evidence requirement at the spec stage, every feature feels equally
worth building because the build cost is roughly zero. The 20
transcripts, the demoware-trap section, the devil's advocate, and
the defended cohort are not bureaucracy — they're the only thing
distinguishing "we built it because we could" from "we built it
because we should".

## See also

- [`playbook/STAGE_MAP.md`](../../playbook/STAGE_MAP.md) — Caro's
  current location and the exit criteria the gates serve
- [`.claude/agents/devils-advocate.md`](../agents/devils-advocate.md) —
  the agent that gates the AI-confirmation-bias check
- [`.claude/skills/caro.discovery/SKILL.md`](../skills/caro.discovery/SKILL.md) —
  the skill that runs a structured interview and saves the
  transcript
- [`docs/discovery/`](../../docs/discovery/) — interview templates,
  hypothesis ledger, anonymization rules
- [`.claude/rules/release-version-alignment.md`](./release-version-alignment.md) —
  the same checklist-as-grep pattern this rule follows
