# Legible Output

**APPLIES TO**: Agent text that the owner reads to decide something:
end-of-turn summaries, PR bodies, review replies, handoffs, issue bodies,
Hermes digests and briefings.
**DOES NOT APPLY TO**: code, quoted logs and tool output, and text quoted
from other people.

Codified 2026-10-03 from [Karpathy's post on understanding LLM output](https://x.com/karpathy/status/2105819303471976479).
Research and evidence: [`docs/research/2026-10-03-legible-output-ste100.md`](../../docs/research/2026-10-03-legible-output-ste100.md).

## Why

Agents now write faster than the owner can read. The owner runs 4–5
parallel sessions. Their attention is the scarcest resource in the loop.
If the owner cannot understand an output in one pass, they cannot give
good feedback, and oversight fails. This rule makes agent output cheap to
read and cheap to answer.

## Rule 1 — Put what needs the owner first

- If you need a decision, the first line says so: `**Needs you:** <question>`.
- Then the result: what changed and whether it works.
- Then the details. A reader who stops after line 3 must still know the
  state.
- Report a failure plainly. Do not put it under a success.

## Rule 2 — Write in STE-lite

STE-lite is the sentence-level part of [ASD-STE100](https://www.asd-ste100.org/),
the controlled English of aircraft maintenance manuals. It is "80% of the
way" to full STE. Use the rules, but not the full 900-word dictionary.

| Do | Limit |
|---|---|
| Keep instructions short | ≤ 20 words per sentence |
| Keep descriptions short | ≤ 25 words per sentence |
| Give one instruction per sentence | — |
| Use the active voice | Passive only when the actor is unknown |
| Keep paragraphs to one topic | ≤ 6 sentences |
| Use one term for one concept | Do not switch between "guard", "check" and "test" for the same thing |
| Use simple words | "use", not "utilize"; "before", not "prior to"; "start", not "commence" |
| Cut filler and hedges | No "simply", "just", "basically", "obviously" |

Technical names are always allowed: commands, flags, paths, crate names,
test names.

## Rule 3 — Climb the format ladder when text is not enough

Pick the lowest step that the owner can understand in one pass:

| Step | Use when |
|---|---|
| STE-lite text | ≤ 5 facts, no flow |
| Diagram (ASCII or Mermaid) | A pipeline, a state machine, or more than 3 parts that interact |
| HTML artifact | The owner must compare options, review a large change, or explore data. Artifacts are private and discardable. |
| Video (`caro-demo-video` skill) | A user-facing feature that the weekly demo must show |

Code is cheap now. A throwaway HTML page that saves the owner 10 minutes is
worth making.

## Rule 4 — Make feedback cheap

- Number the items that need an answer, so the owner can reply `2: no`.
- Ask one question per decision. Give your recommended default.
- Keep the [Quick Actions footer](./quick-actions-footer.md) when you stop
  for input.

## Self-check before you send

- [ ] Line 1 tells the owner if they must act.
- [ ] No instruction is longer than 20 words and no description is longer than 25, except quoted text.
- [ ] Each paragraph has one topic.
- [ ] A flow or multi-part change has a diagram.
- [ ] Each open question has a number and a default.

## The product mirror

Caro's `--explain` mode follows the same rules for end users. The checker
in `src/prompts/ste.rs` and the contract test `tests/explain_ste_contract.rs`
check only the mechanical part: sentence length, paragraph length, a short
list of unapproved words and passive voice, on the built-in explanation
text. They are a regression check, not proof that every rule is met. When
you change explain text, run `cargo test --test explain_ste_contract`.

## See also

- [`quick-actions-footer.md`](./quick-actions-footer.md): the footer that
  Rule 4 builds on
- [`feature-evidence.md`](./feature-evidence.md): evidence, demo and
  regression guard in PR bodies, which Rule 1 orders
- `.claude/hooks/quick-actions-reminder.sh`: prints the short form of this
  rule on each prompt, except when the conversation used `AskUserQuestion`
