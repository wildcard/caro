# Legible output: what Caro can learn from Karpathy's post and ASD-STE100

**Date:** 2026-10-03
**Source:** [Andrej Karpathy, 2026-10-02](https://x.com/karpathy/status/2105819303471976479)
**Status:** Research, plus two small adoptions in the same PR: the STE-lite
explain mode and the [`legible-output`](../../.claude/rules/legible-output.md)
rule. Anything that grows into a new product line must clear the gates in
[`validation-discipline.md`](../../.claude/rules/validation-discipline.md).

This note uses the style that it recommends. Most sentences have fewer
than 20 words.

## 1. What the post says

Karpathy's claim: *"We'll be spending a lot more time trying to understand
the outputs of language models."* He ranks output formats from good to best:

| # | Format | His tip |
|---|---|---|
| 1 | Writing | Ask for ASD-STE100, or "80% of the way to ASD-STE100" when the full spec is too strict. |
| 2 | Diagram | A picture is often faster to parse than prose. |
| 3 | Web page | Ask for output "in HTML" to get an interactive page. |
| 4 | Explainer video | A bespoke "3b1b style" video on any topic, with generated narration. |

His summary has two parts:

- As models do more of the work, *"a lot more of our work will rise up the
  abstractions into oversight and understanding."*
- Code is now cheap. Ask for *"large, custom, discardable software
  artifacts"* that would not have made sense to build before.

## 2. The underlying problem

Generation is no longer the bottleneck. **Human understanding is.** An agent
can write 2,000 lines in an hour. A person still reads at the same speed as
before. Oversight fails when the reader cannot understand the output fast
enough to judge it.

Caro has this problem twice:

```
                 ┌────────────────────┐
  user prompt ──▶│ caro generates cmd │──▶ end user must understand it
                 └────────────────────┘    BEFORE they press Enter
                                           (product: explain mode, safety text)

                 ┌────────────────────┐
  owner task ───▶│ agents write code  │──▶ owner must understand the result
                 └────────────────────┘    to give feedback
                                           (dev process: PRs, summaries, digests)
```

The first reader is the user. A shell command that the user does not
understand is a safety risk, even when the validator passes it. The second
reader is the owner. The owner runs 4–5 parallel sessions and has the least
time of anyone in the loop. In both cases the scarce resource is the reader's
attention, not compute.

## 3. ASD-STE100 in depth

### Origin

AECMA, the European aerospace industry association, started work on it in
the early 1980s. The first guide came out in 1985. The problem was specific.
Aircraft maintenance manuals came from many manufacturers. Each manufacturer
wrote a different English (American, British, or a non-native house style).
Most of the readers were maintenance technicians whose first language was not
English. When a technician misread a procedure, an aircraft could be
damaged and people could be hurt.

So STE is a **safety tool**, not a style preference. It controls language
so that a non-native reader under time pressure gets one meaning only.
That is the same position as a Caro user who reads a `rm` or `dd` command
before running it.

### Current state

- **Issue 9** was released on 2025-01-15. It changed from a specification
  to an international standard. It has no new rules, but it rewords 31 of
  the 53 rules and updates 555 dictionary entries.
- **53 writing rules** plus a **dictionary of about 900 approved words**.
- One word has one meaning and one part of speech. For example, "test" is
  a noun only, so you write "do a test", not "test the unit".
- Technical names and technical verbs from your own subject are allowed.
  For Caro this means command names, flags, file paths and POSIX terms.
- As of December 2024, 64% of the people who requested the standard came
  from outside aerospace and defence.

### The rules that matter most for us

| STE rule | Limit | Why it helps the reader |
|---|---|---|
| Procedural sentence length | ≤ 20 words | One instruction fits in working memory. |
| Descriptive sentence length | ≤ 25 words | Same, with room for one qualifier. |
| One instruction per sentence | — | The reader can do step 1, then read step 2. |
| Active voice | Passive only if the agent is unknown | The reader knows *who* or *what* does the action. |
| Simple tenses | Present, past, future, imperative | No "would have been" chains. |
| `-ing` forms | Only as technical nouns or modifiers | Removes many ambiguous phrases. |
| Paragraphs | ≤ 6 sentences, one topic | The reader can skip a topic. |
| Warnings and cautions | Start with a clear command or condition | The reader sees the risk before the detail. |
| Noun clusters | ≤ 3 words | "file size check result list" stops being a puzzle. |
| Approved words | One word, one meaning | "use", not "utilize"; "before", not "prior to". |

### Why LLMs follow it well

A vague instruction like "be concise" gives a model nothing to check against.
STE gives it numbers and lists. A model can count to 20. A model can know that
"utilize" is not approved. LLM prose has a known failure mode: long, hedged,
filler-heavy sentences. STE targets exactly that failure mode.

### Limits (devil's advocate)

- **No hard comprehension data in our sources.** The case for STE rests on
  40 years of adoption in safety-critical documentation, not on a study we
  can cite. Treat "STE is more readable" as a strong prior, not a measured
  result.
- **The ASD says that STE alone is not enough.** It controls sentences. It
  does not tell you what to write or in what order.
- **Full STE is hard and can read as stilted.** Karpathy himself softens it
  to "80%". We do the same: **STE-lite** means the sentence, voice, paragraph
  and word rules, but not the full 900-word dictionary.
- **Software checkers cannot write STE.** They can only flag mechanical
  violations, and some flags are false positives. Our checker
  (`src/prompts/ste.rs`) is a regression guard, not a judge of quality.

## 4. The format ladder

Karpathy's list is a ladder. Each step costs more to make and is cheaper to
read. Code is now cheap, so the cost to make one is small. Pick the lowest
step that the reader can understand in one pass:

| Step | Use it when | Caro tooling |
|---|---|---|
| STE-lite text | ≤ 5 facts, no flow | — |
| Diagram (ASCII or Mermaid) | A pipeline, a state machine, or more than 3 parts that interact | Plain Markdown |
| HTML page | The owner must compare options, review a large change, or explore data | Artifact tool (private, discardable) |
| Video | A user-facing feature that the weekly demo must show | `caro-demo-video` skill (Remotion) |

## 5. What we adopt now

### Product: explain mode (`caro --explain`)

The audit of `create_explanation()` found these problems:

1. **Doubled verb in the headline.** The summary was `"Uses find to <intent>"`
   and the display adds `Use \`find\``. The result was
   ``Use `find` Uses find to find recent files:``.
2. **Wrong flags.** Flag detection used substring matching:
   - `grep --include` matched `-i`. The output said "case-insensitive" for a
     case-sensitive search. That is a wrong explanation, which is worse than
     none.
   - `ls -la` did not match `-a`, and `grep -rn` did not match `-n`, because
     combined short flags were not split.
3. **The "what it does" text was not shown**, and it repeated the full
   command in parentheses.
4. **The LLM explainer prompt had no writing constraints.** It asked for
   "concise yet educational" output, which is the vague instruction that STE
   replaces.

Fixes in this PR:

- An STE-lite checker, `src/prompts/ste.rs`. It checks sentence length,
  paragraph length, a short list of unapproved words, and passive voice.
- All static explanation text is rewritten in STE-lite. A contract test
  (`tests/explain_ste_contract.rs`) runs the checker on every explanation
  for a set of commands, so the text cannot drift back.
- Flag parsing splits combined short flags and ignores long options. It no
  longer reports a flag that is not there.
- The explainer system prompt now has a `WRITING RULES (STE-lite)` section,
  so LLM-generated explanations get the same constraints when that path is
  enabled.
- Explain mode shows a `What it does:` line.

### Dev process: the `legible-output` rule

[`.claude/rules/legible-output.md`](../../.claude/rules/legible-output.md)
applies the same idea to agent output that the owner reads. It covers
summaries, PR bodies, handoffs and digests. The UserPromptSubmit hook
prints a three-line reminder, so every session gets the rule without a
manual load. The rule asks for four things:

1. Put what needs the owner first.
2. Write in STE-lite.
3. Climb the format ladder when text is not enough.
4. Make feedback cheap: number the items and give a default.

## 6. Follow-ups (not in this PR)

| Item | Why | Gate |
|---|---|---|
| Rewrite safety warnings in STE warning format | STE's original purpose. Warnings must start with the command or condition. | Safety text change: use `safety-pattern-developer` TDD. |
| Validate LLM explainer output with `ste::check` at runtime | Catch long, hedged model output before display | Needs the LLM explain path enabled first. |
| `--explain=diagram` for pipelines | Step 2 of the ladder for multi-stage commands | Prototype first. It is a new user-facing capability, so the validation gates apply. |
| STE-lite for website docs and i18n source strings | Controlled source text is easier to translate across 15 locales | Docs change, no gate. |
