# Discovery Transcripts

This directory holds first-hand user-interview transcripts captured per
[`../README.md`](../README.md). Each file is one interview.

## Filename convention

```
YYYY-MM-DD-<anon-handle>-<hypothesis-slug>.md
```

Examples:
- `2026-06-15-ciso-fintech-mid-enterprise-dashboard.md`
- `2026-06-22-eng-platform-bigco-enterprise-dashboard.md`
- `2026-07-03-ds-engineer-pacific-local-context-indexing.md`

## Required frontmatter

```yaml
---
date: 2026-MM-DD                           # interview date, not transcript-write date
anon_handle: <stable-pseudonym>            # consistent across this person's transcripts
channel: synchronous | async-chat | conference | other
duration_min: 25                           # actual minutes — 0 if async
hypothesis: <hypothesis-id-from-ledger>    # must match docs/discovery/hypothesis-ledger.md
pain_patterns: [<slug-1>, <slug-2>]        # 1-3 slugs from the ledger
willingness_signal: yes | no | conditional | not-asked
consent_to_log: yes                        # MUST be `yes` — no transcript without consent
recording: yes | no | refused              # whether you had an audio/video recording aid
wants_beta: yes | no | not-asked
---
```

## Body

The body is the conversation, paraphrased or quoted. Time-stamped if
synchronous. No PII per the anonymization rules in
[`../README.md`](../README.md#anonymization-rules).

**Format suggestion** — alternating speaker labels:

```markdown
**Kobi**: Tell me about the AI coding tools your engineering org uses today.

**P**: We're a 200-engineer fintech, mostly Cursor and the platform team uses
Claude Code. Security is fine with both because they're approved through our
Vanta-managed vendor list, but honestly, nobody's looking at what gets run.

**Kobi**: When you say "nobody's looking" — who would notice if something
destructive happened?
…
```

Don't try to be a court reporter. Capture meaning, not every "um". If you
took notes instead of recording, mark each block with `(notes, not verbatim)`
on the first quote so the synthesis honestly weights the signal.

## What goes here

- ✅ Real first-hand interviews (synchronous calls, scheduled async chats)
- ✅ Conversations that had explicit consent to log
- ✅ Properly anonymized exchanges

## What does NOT go here

- ❌ Survey responses → those live in the telemetry / waitlist DB; per
  Gate 2 they can't anchor a hypothesis
- ❌ AI-generated synthesis dressed as user signal — per Gate 1, the
  founder talked to a real person, not the model
- ❌ Internal Kobi-talks-to-Claude planning sessions (see the
  `local-context-indexing` correction in
  [`../v2.0-validation-audit.md`](../v2.0-validation-audit.md))
- ❌ Marketing testimonials — those have their own consent flow and
  live in `website/src/components/landing/`
- ❌ Bug reports → those are GitHub Issues

## Reviewer's quick audit

When a feature spec claims "Gate 1 cleared (≥20 transcripts)", the
reviewer can sanity-check in 30 seconds:

```sh
# Count transcripts for the hypothesis:
ls docs/discovery/transcripts/ | grep -E '<hypothesis-slug>\.md$' | wc -l

# Confirm none are AI-fabricated (no consent = invalid):
grep -L 'consent_to_log: yes' docs/discovery/transcripts/*-<hypothesis-slug>.md

# Distinct pain-pattern count across the set:
grep -h 'pain_patterns:' docs/discovery/transcripts/*-<hypothesis-slug>.md \
  | tr -d '[]' | tr ',' '\n' | sed 's/^[[:space:]]*//' | sort -u | wc -l
# This number must be ≥3 per validation-discipline.md Gate 1.
```

## Privacy escalation

If you ever can't anonymize without destroying the signal — escalate
to a human reviewer. Don't compromise. The transcript not existing is
better than the transcript leaking identifying information.

## Currently in this directory

`.gitkeep` only. The first real transcript lands here when the first
interview happens.
