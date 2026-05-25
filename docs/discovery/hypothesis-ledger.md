# Hypothesis Ledger

Append-only ledger of every product-line hypothesis under validation,
with its evidence count, validation status, and last review date.

This ledger is the single source of truth for "what are we trying to
prove right now" — it gates feature work per
[`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md).

A hypothesis graduates from `unvalidated` only when:
1. ≥20 first-hand transcripts cite it (Gate 1)
2. ≥3 distinct pain patterns emerge across the transcripts (Gate 1)
3. Devil's-advocate review (Gate 4) returns `Pass` or `Revise` (not `Block`)
4. If claiming PMF: Sean Ellis result with defended cohort (Gate 5)

## Status taxonomy

- **`unvalidated`** — claim exists, evidence gathering not started or
  <20 transcripts
- **`partial`** — ≥10 transcripts, signal converging but criteria not
  yet met
- **`validated`** — all four conditions met; safe to graduate from
  research to implementation
- **`invalidated`** — evidence actively refutes the hypothesis (the
  most valuable status — it saves a build cycle)
- **`grandfathered`** — claim was made before May 25, 2026 and is
  exempt from retroactive validation (e.g. the core NL→shell loop
  that shipped in v1.0)

## Active hypotheses

| ID | Claim | Transcripts | Patterns | Status | Last review | Gates cleared |
| --- | --- | ---: | ---: | --- | --- | --- |
| `caro-core` | Users want NL→safety-validated POSIX commands (the v1.x product loop) | — | — | `grandfathered` | 2026-05-25 | exempt (shipped pre-rule) |
| `karo-distributed` | Users want a distributed-terminal-intelligence layer across their devices | 0 | 0 | `unvalidated` | 2026-05-25 | none |
| `dogma-rules` | Users want a rule-engine they can customize beyond the built-in 52 safety patterns | 0 | 0 | `unvalidated` | 2026-05-25 | none |
| `voice-synthesis` | Users want Caro to speak responses aloud (mascot voice) | 0 | 0 | `unvalidated` | 2026-05-25 | none |
| `self-healing` | Users want commands to retry/recover automatically on certain failure classes | 0 | 0 | `unvalidated` | 2026-05-25 | none |
| `local-context-indexing` | Users want Caro to know about their repo / shell history / open files when generating commands | 0 | 0 | `unvalidated` | 2026-05-25 | none |
| `enterprise-dashboard` | CISOs want a centralized policy + audit-trail surface for Caro deployments | 0 | 0 | `unvalidated` | 2026-05-25 | none |

The `caro-core` row is grandfathered because the rule applies forward
from May 25, 2026. The full retroactive audit for each v2.0 hypothesis
is in [`v2.0-validation-audit.md`](./v2.0-validation-audit.md).

## Synthesis entries

Synthesis happens after every 5 transcripts under the same hypothesis.
The synthesis entry below each hypothesis lists distinct pain patterns,
willingness-signal counts, dealbreaker quotes, and any emerging cohort
definition. New synthesis entries get appended chronologically.

### `karo-distributed` synthesis

*No transcripts logged yet. First synthesis entry will appear here
after 5 interviews.*

### `dogma-rules` synthesis

*No transcripts logged yet.*

### `voice-synthesis` synthesis

*No transcripts logged yet.*

### `self-healing` synthesis

*No transcripts logged yet.*

### `local-context-indexing` synthesis

*No transcripts logged yet.*

### `enterprise-dashboard` synthesis

*No transcripts logged yet.*

## Pain pattern slugs

When a transcript surfaces a pain pattern, tag it with a slug. Reuse
existing slugs when applicable; coin new ones sparingly. Slugs in use:

*(none yet — this list grows as transcripts land)*

Suggested coining convention: `<verb>-<noun>` (e.g. `forget-flags`,
`fear-rm-rf`, `lose-context-across-devices`).

## Invalidated hypotheses

When evidence actively refutes a hypothesis, the row moves here. An
invalidated hypothesis saves a build cycle and deserves the same
ceremony as a validated one — name the evidence that killed it.

*(none yet)*

## How to update this ledger

- **After every interview**: increment the `Transcripts` count for the
  relevant hypothesis. Tag any new pain patterns observed.
- **Every 5 transcripts**: append a synthesis entry under that
  hypothesis's section. Update `Patterns` count. Update `Status` if
  the synthesis warrants it.
- **When a hypothesis graduates**: change `Status` to `validated`,
  record the date as `Last review`, and link to the spec PR that
  consumed the evidence.
- **When evidence refutes**: move the row to "Invalidated hypotheses"
  with one paragraph naming the refutation.

The ledger is append-only at the synthesis level; status transitions
are normal edits. Don't rewrite history — if a synthesis was wrong, add
a correction synthesis below it, don't delete the original.

## See also

- [`README.md`](./README.md) — directory purpose, anonymization rules
- [`interview-template.md`](./interview-template.md) — question script
  for new transcripts
- [`v2.0-validation-audit.md`](./v2.0-validation-audit.md) — retroactive
  audit of in-flight v2.0 product lines
- [`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md) —
  the rule that makes this ledger load-bearing
- [`ROADMAP.md`](../../ROADMAP.md) — items here cross-reference
  hypotheses by ID
