# Discovery

This directory holds the **evidence** that gates feature work per
[`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md).

Specifically: first-hand user-interview transcripts, the hypothesis
ledger that tracks pain patterns surfaced across transcripts, and
retroactive validation audits for in-flight product lines.

Adapted from [Anthropic's Founder's Playbook](https://claude.com/blog/the-founders-playbook):
the 20-transcript rule, the no-surveys-as-primary-evidence rule, the
demoware-trap gate, and the defended-cohort requirement for PMF
claims all live here as operational artifacts.

## Contents

| File | Purpose |
| --- | --- |
| [`README.md`](./README.md) | This file: directory purpose, anonymization rules, contribution guide |
| [`interview-template.md`](./interview-template.md) | Question script + signal taxonomy for `caro.discovery` skill |
| [`hypothesis-ledger.md`](./hypothesis-ledger.md) | Append-only ledger: every product-line hypothesis with evidence count, validation status, last review date |
| [`v2.0-validation-audit.md`](./v2.0-validation-audit.md) | Retroactive audit of Karo, Dogma, voice synthesis, self-healing, local context indexing |
| `transcripts/` | Anonymized first-hand user-interview transcripts (one file per interview) |

## How to add a transcript

1. Run the interview using the [`caro.discovery`
   skill](../../.claude/skills/caro.discovery/SKILL.md), which uses
   the [`interview-template.md`](./interview-template.md) question
   script.
2. Get **explicit consent** from the participant for the transcript
   to be logged. Note the consent in the transcript frontmatter.
3. Save the transcript at:
   ```
   transcripts/YYYY-MM-DD-<anon-handle>-<topic-slug>.md
   ```
4. Anonymize aggressively per the rules below.
5. Tag with 1–3 `pain_patterns` slugs in the frontmatter. Reuse
   existing slugs from `hypothesis-ledger.md` if one fits; coin a
   new slug if none does (and add it to the ledger).
6. After every 5 transcripts under the same hypothesis, run a
   synthesis pass — append a synthesis entry to
   `hypothesis-ledger.md`.

## Anonymization rules

Transcripts live in a public repo. They must never carry
identifying information.

| Information type | What to do |
| --- | --- |
| Real names | Replace with a stable `anon_handle` (e.g. `ds-engineer-pacific`). The handle is consistent across that participant's multiple transcripts so we can track them as a cohort. |
| Company names | Replace with `<industry>-co` (e.g. `fintech-co`, `healthtech-co`) unless the participant gave explicit consent to name the company. |
| Internal tool names | Replace with `<internal-tool>` if naming the tool would identify the participant. |
| File paths with usernames | Replace with `~/` or `<home>/`. |
| Email addresses | Never log. Don't even quote them. |
| Discord display names, GitHub handles, Twitter handles | Pseudonymize into the `anon_handle`. |
| Geographic identifiers more specific than country | Generalize ("US west coast" not "Mountain View"). |

If you can't anonymize without destroying the signal, the
transcript isn't usable in this directory. Either get explicit
consent for the identifying detail, or don't log it. Don't
compromise — escalate to a human reviewer.

## Hypothesis ledger format

Every product-line hypothesis under validation gets a row in
[`hypothesis-ledger.md`](./hypothesis-ledger.md). Columns:

- **Hypothesis ID** — short slug, e.g. `karo-distributed`, `voice-synthesis`
- **Claim** — one sentence: "users want X so they can Y"
- **Transcripts** — count of transcripts referencing this hypothesis
- **Distinct pain patterns** — count of unique pain patterns observed
  across those transcripts
- **Validation status** — `unvalidated` / `partial` / `validated` /
  `invalidated`
- **Last review** — date of last synthesis pass
- **Stage gate** — which gate of `validation-discipline.md` this
  hypothesis has cleared

A hypothesis graduates from `unvalidated` only when ≥20 transcripts
cite it AND ≥3 distinct pain patterns have emerged AND the devil's
advocate review (Gate 4) returns Pass or Revise (not Block).

## What this directory does NOT hold

- **Surveys, NPS data, waitlist signups.** Those are quantitative
  signals and live elsewhere (telemetry, Turso DB). Per Gate 2 they
  cannot serve as primary validation evidence.
- **Marketing testimonials.** Public testimonials require their own
  consent flow and live in `website/src/components/landing/`
  (e.g. `AITestimonials.astro`).
- **Bug reports.** Those are GitHub Issues. A bug is evidence that
  something we shipped is broken — different from evidence that
  something we want to ship is needed.
- **Telemetry data.** Telemetry is opt-in, privacy-first, no command
  content (see `docs/TELEMETRY.md`). It serves retention/usage
  measurement, not problem validation.

## Why this directory exists

The playbook's central warning: when a prototype takes an afternoon,
founders mistake the prototype's existence for proof of demand. The
counter is evidence requirements that gate feature work *at the spec
stage*, before code starts. This directory is where the evidence
lives. Without it, the validation-discipline rule is a piece of
paper.

## See also

- [`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md) —
  the rule this directory serves
- [`.claude/skills/caro.discovery/SKILL.md`](../../.claude/skills/caro.discovery/SKILL.md) —
  the skill that walks an interview and writes a transcript here
- [`.claude/agents/devils-advocate.md`](../../.claude/agents/devils-advocate.md) —
  the agent that audits whether a spec's transcript citations
  actually clear Gate 1
- [`playbook/STAGE_MAP.md`](../../playbook/STAGE_MAP.md) — what stage
  the evidence is serving
