# Caro Discovery Interview Template

The question script for a Caro user-discovery interview. Used by the
[`caro.discovery`](../../.claude/skills/caro.discovery/SKILL.md) skill
and counted toward the 20-transcripts gate in
[`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md).

**Time budget**: 20–30 minutes synchronous, or 24 hours async-chat.
**Format**: 1:1, qualitative, recorded with consent.

## Before the interview

- [ ] Confirm the **hypothesis under test**. Today's interview is
  one of 20 for hypothesis `<id>` from
  [`hypothesis-ledger.md`](./hypothesis-ledger.md). Knowing the
  hypothesis shapes which probes you push on — but you do NOT share
  the hypothesis with the participant during the interview (it
  primes them).
- [ ] Get explicit consent. "May I record this conversation as an
  anonymized discovery transcript? It goes in a public repo, your
  handle is hashed, your company is generalized. You can ask me to
  delete it at any time."
- [ ] Decide on the `anon_handle` (consistent if this participant
  has been interviewed before).

## Question 1 — Context (≤3 min)

> "Tell me about your work. What do you build, in what kind of team,
> what's a typical day in the terminal look like for you?"

**Listen for**: role, seniority, daily shell hours, OS, shell
flavor, team size. These shape the cohort.

## Question 2 — Last hard command (5–7 min)

> "Tell me about the last time you had to look up a command, ask a
> colleague, or copy-paste from StackOverflow / ChatGPT / Cursor.
> What were you trying to do? What did you actually do? How long
> did it take from "I need this" to "it ran"?"

**Listen for**: the trigger, the workaround, the time-to-command,
the emotional valence (frustration / resignation / amusement). Push
for **the specific incident**, not the abstract category.

**Bad answer to follow up on**: "Oh, I do that all the time."
**Better follow-up**: "What did you do yesterday morning? Walk me
through it."

## Question 3 — Current alternatives (3–5 min)

> "When you don't remember a flag / pipeline / one-liner, what's
> your default path? Manpages? `tldr`? `cheat`? Cursor inline?
> ChatGPT in another tab? Slack the team? GitHub Copilot? Just
> Google it?"

**Listen for**: the path of least resistance (revealed-preference
signal), the path they wish they took (aspirational), the path
they actively avoid (anti-pattern). All three are signal.

## Question 4 — Pain frequency (≤2 min)

> "Per day or per week, how often does this happen?"

**Listen for**: an actual integer. "All the time" is not an
answer; push for "5 times a day" or "twice a week". Pain frequency
predicts willingness-to-adopt; you cannot synthesize without it.

## Question 5 — Willingness signal (3–5 min)

> "If a tool fixed this in 2 seconds, with safety validation
> blocking destructive commands before they run, what's it worth
> to you? Free-tier nice-to-have? Worth $5/month, $20/month,
> $100/month? Would your company pay for it?"

**Listen for**: a number AND a tier choice (personal vs.
company-paid). Both signals matter — Caro's dual-track
Community/Enterprise model needs both demand curves.

**Be honest**: if they say "I'd never pay personally but my
employer might", that's an Enterprise signal, not a Community one.

## Question 6 — Dealbreakers (5–7 min) ⭐ most important

> "What would a tool in this space do that would make you
> uninstall it within a day? What's the thing that would make you
> not even try it?"

**Listen for**: the failure modes nobody tells you in a survey.
Telemetry phoning home, slow startup, hallucinated commands, vendor
lock-in, no offline mode, no audit log, requiring a Anthropic API
key, requiring a cloud account, the wrong shell support. The
dealbreakers from this question are the demoware-trap inputs for
the feature spec.

## Optional — Beta interest

> "Would you want early access if/when this exists? If yes, what
> would you want to test first?"

**Listen for**: enthusiasm vs. politeness. "Sure, sounds cool" is
politeness; "yes, here's my email, please ping me Tuesday" is
enthusiasm.

## After the interview

- [ ] Write the transcript to `transcripts/YYYY-MM-DD-<anon-handle>-<topic-slug>.md`
  with the required frontmatter (see
  [`README.md`](./README.md))
- [ ] Anonymize aggressively per the rules in
  [`README.md`](./README.md)
- [ ] Tag with 1–3 `pain_patterns` slugs
- [ ] Update the `transcripts` count for the hypothesis in
  [`hypothesis-ledger.md`](./hypothesis-ledger.md)
- [ ] If this is the 5th, 10th, 15th, or 20th transcript under this
  hypothesis, run a synthesis pass into the ledger

## Calibration notes

- **The question that earns the discovery, not the answer.**
  Discovery is about uncovering what the participant believes; if
  the participant has nothing surprising to tell you, you didn't
  pick the right participant.
- **Push for specific incidents.** "I always do X" is a story;
  "yesterday at 3pm I did Y" is data. Demand the second.
- **Cohort variance matters.** If all 20 transcripts come from
  senior backend engineers in Pacific timezone, the resulting
  hypothesis is validated for *that cohort*, not "users in
  general". Note this explicitly in the synthesis.
- **Negative signal is signal.** A transcript where the
  participant says "I never have this problem" doesn't get
  discarded — it goes in the ledger as a cohort exclusion.

## What this template is NOT

- **It is not a sales script.** Don't pitch Caro during the
  interview. Pitching primes the participant and corrupts the
  evidence.
- **It is not a closed-form survey.** The questions are anchors;
  the conversation is the data. Follow the participant's lead
  when they surface something unexpected.
- **It is not the same as a beta-test session.** A beta test
  evaluates an existing product; a discovery interview evaluates
  whether a hypothetical product is worth building. Don't mix
  them.

## See also

- [`README.md`](./README.md) — directory purpose, anonymization rules
- [`hypothesis-ledger.md`](./hypothesis-ledger.md) — where the
  hypothesis IDs live
- [`.claude/skills/caro.discovery/SKILL.md`](../../.claude/skills/caro.discovery/SKILL.md) —
  the skill that walks this template programmatically
- [`docs/PERSONAS_JTBD.md`](../PERSONAS_JTBD.md) — existing persona
  framing (use it to pre-select interview participants per cohort)
