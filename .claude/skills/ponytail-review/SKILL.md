---
name: "ponytail-review"
description: "Run the pragmatic-skeptic 'ponytail' reviewer over the current diff to surface over-engineering — code, abstractions, dependencies, files, or process steps the problem does not need. Read-only and advisory; never trims safety, security, accessibility, data-loss handling, or their tests. Use before pushing a PR, when a change feels heavier than the task warranted, or when boilerplate/a new dependency crept in. Triggers: 'ponytail review', 'is this over-engineered', 'what can I delete here', 'did I over-build this'."
version: "1.0.0"
allowed-tools: "Bash, Read, Grep, Glob, Task"
license: "AGPL-3.0"
---

# Ponytail Review Skill

## What This Skill Does

Spawns the [`ponytail-reviewer`](../../agents/ponytail-reviewer.md) agent
against the current change set and returns its 6-section critique. The
reviewer is the "lazy senior dev" — it looks for the simplest thing that
could work and names what is surplus, without ever touching safety,
security, accessibility, data-loss handling, the tests that guard them,
or the release/ADR process.

It is the active companion to the passive
[`good-boy-scout`](../../rules/good-boy-scout.md) rule, and the
deliberate opposite of [`devils-advocate`](../../agents/devils-advocate.md):
devils-advocate asks "should this be built?"; ponytail asks "is this
bigger than it needs to be?"

**This skill is advisory. It proposes reductions; it never applies them.**
The human or parent agent decides what to act on.

## When to Use This Skill

- Before pushing a PR, as a bloat check.
- When a change feels heavier than the task warranted.
- When boilerplate, a new abstraction, or a new dependency crept in.
- When reviewing whether a *process* change is ceremony or signal.

Do **not** use it as a gate that blocks merges, and do **not** let it
override safety-pattern TDD or any constitution carve-out.

## How to Run

1. Determine the change set under review (in order of preference):
   - An explicit diff/PR the user named.
   - `git diff` (unstaged) and `git diff --staged` on the working branch.
   - `git diff main...HEAD` for the whole branch when reviewing a PR.

2. Spawn the reviewer with the Task tool:
   - `subagent_type: "ponytail-reviewer"`
   - Prompt: paste the diff (or name the files) plus the stated intent of
     the change, and ask for the standard 6-section critique.

3. Return the agent's critique verbatim. Do not soften the verdict.

## What You Get Back

The standard 6-section contract:

1. **What I read** — the change and its intent.
2. **The simplest thing that could work** — the YAGNI baseline.
3. **Reduction objections (numbered)** — what could go, which ladder rung
   it fails, why it's surplus, what would justify keeping it.
4. **Carve-outs respected** — the safety/security/a11y/data-loss/test/
   release lines it deliberately did not trim.
5. **Net effect** — lines/files/deps/steps removable vs. the cost.
6. **Verdict** — `Keep as-is` / `Trim` / `Over-built`.

## Guardrails

- **Read-only.** The skill and the agent never edit, delete, format, or
  commit. They report.
- **Carve-outs are absolute.** If the agent ever suggests trimming a
  safety pattern, a security check, an accessibility affordance, a
  data-loss safeguard, a test for new functionality, or a release-
  checklist step — discard that objection; it violated the contract.
- **Pragmatic, not zealous.** A few extra lines that aid clarity are not
  a finding. Shorter is not always simpler.

## See Also

- `.claude/agents/ponytail-reviewer.md` — the agent this skill drives.
- `.claude/agents/devils-advocate.md` — the opposite critical axis.
- `.claude/rules/good-boy-scout.md` — the passive principle this
  operationalizes.
- `docs/adr/ADR-016-ponytail-pragmatic-reviewer.md` — why this exists and
  what we deliberately did not adopt from the upstream ponytail project.
