---
name: caro.discovery
description: Use this skill to run a structured user-discovery interview for Caro and save the transcript into docs/discovery/transcripts/ under the project's anonymization rules. Use when a new product line or major feature spec needs to clear Gate 1 of .claude/rules/validation-discipline.md (the 20-transcripts rule). Also use to synthesize transcripts into the hypothesis ledger after each batch of 5 interviews. Triggers - "run a discovery interview for <feature>", "log this user conversation as a Caro discovery transcript", "synthesize this week's transcripts into the hypothesis ledger".
---

# Caro Discovery Skill

This skill turns a user conversation into a properly-formatted
discovery transcript that counts toward the 20-transcripts gate in
[`.claude/rules/validation-discipline.md`](../../rules/validation-discipline.md),
and synthesizes batches of transcripts into the
[`docs/discovery/hypothesis-ledger.md`](../../../docs/discovery/hypothesis-ledger.md).

## When this skill activates

- **Run a discovery interview** — guides you through the structured
  question script in
  [`docs/discovery/interview-template.md`](../../../docs/discovery/interview-template.md)
- **Log an organic conversation** — formats an existing Discord / DM /
  GitHub Discussions exchange as a Caro discovery transcript (with
  consent)
- **Synthesize a batch** — after every 5 transcripts, distill the
  signal into the hypothesis ledger

## What this skill does NOT do

- **It does not invent transcripts.** Every transcript must reference
  a real conversation with a real person. Fabricated transcripts are
  the fastest way to corrupt the entire validation discipline.
- **It does not skip consent.** Logging an existing conversation
  requires the participant's explicit consent (recorded as a yes/no
  field in the transcript front matter).
- **It does not collect PII.** Transcripts are anonymized at write
  time. See the anonymization rules in
  [`docs/discovery/README.md`](../../../docs/discovery/).
- **It does not gate features by itself.** This skill produces the
  evidence; `.claude/rules/validation-discipline.md` is the gate that
  uses the evidence.

## Interview flow

1. **Confirm the product line under discovery.** "We're validating the
   hypothesis that <X>. Today's interview is one of 20 for this
   hypothesis." Reference the relevant entry in
   `docs/discovery/hypothesis-ledger.md`.
2. **Get consent.** "May I record this conversation as an anonymized
   discovery transcript? It goes in a public repo under
   docs/discovery/transcripts/ but your handle is hashed." Record
   yes/no.
3. **Run the interview template** at
   [`docs/discovery/interview-template.md`](../../../docs/discovery/interview-template.md).
   The template enforces the "what do users do" framing (over "what
   would users say"), captures pain frequency / current alternatives
   / willingness to pay.
4. **Write the transcript** to
   `docs/discovery/transcripts/YYYY-MM-DD-<hash>.md` with front
   matter:
   ```
   ---
   date: 2026-MM-DD
   hypothesis: <hypothesis-ledger-id>
   participant_hash: <sha256-truncated-12>
   consent: yes
   channel: interview | discord | dm | github-discussion
   pain_patterns: [<pattern-tag>, ...]
   ---
   ```
5. **Update the hypothesis ledger** with the new evidence count.

## Synthesis flow (after every 5 transcripts)

1. Read the 5 most-recent transcripts under the same hypothesis.
2. Identify **distinct pain patterns** (not 5 variations of the same
   one). The validation-discipline rule requires ≥3 distinct patterns
   from the 20-transcript set.
3. Append a synthesis entry to
   [`docs/discovery/hypothesis-ledger.md`](../../../docs/discovery/hypothesis-ledger.md):
   - hypothesis id
   - transcript count so far
   - pattern count surfaced
   - 1-paragraph summary of the strongest signal
   - any pattern that would *invalidate* the hypothesis (if surfaced)
4. If a hypothesis is invalidated by the synthesis, flag it — don't
   silently move on. An invalidated hypothesis is signal; it saves a
   build cycle.

## Anti-patterns this skill refuses

- **Surveys-as-transcripts.** A Google Form response is not a
  transcript. Skill will not log it.
- **Filling in the blanks.** If the participant didn't answer the
  willingness-to-pay question, that field is left empty. Don't
  guess.
- **Skipping the hash.** Participant handles are sha256-truncated.
  Skill will not write a transcript with a raw handle.
- **Self-interviews.** The interviewer cannot be the participant.
  Caro maintainers' opinions go in spec discussion, not in the
  transcript ledger.

## Integration with the validation-discipline rule

A new spec that claims to clear Gate 1 (20 transcripts) must
reference transcript filenames from `docs/discovery/transcripts/`.
The reviewer can `ls docs/discovery/transcripts/ | wc -l` to confirm
the count, and `grep -l "hypothesis: <id>" docs/discovery/transcripts/`
to confirm the 20 are for the right hypothesis.

This skill is one of the few sanctioned paths to producing those
transcripts.
