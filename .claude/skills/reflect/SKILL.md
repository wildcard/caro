---
name: "reflect"
description: "Review session learnings and suggest CLAUDE.md or skill improvements. Use at session end to convert mistakes and insights into permanent guardrails."
version: "1.0.0"
allowed-tools: "Read, Grep, Glob, Edit, Write"
license: "AGPL-3.0"
---

# Reflect Skill

## What This Skill Does

At session end, reviews what went wrong and what was learned, then suggests concrete improvements to CLAUDE.md or proposes new skills. This converts per-session mistakes into permanent system improvements.

Inspired by hboon's `reflect-agents.md-file` skill. See `docs/research/agentic-workflow-patterns-hboon.md`.

**Key Principle:** "If the agent made a mistake once, it should never make it again. The fix isn't just fixing the code — it's updating the instructions."

## When to Use This Skill

- At the end of a coding session before handoff
- After fixing a bug caused by agent misunderstanding
- After discovering that the agent repeatedly ignores a convention
- When you notice a pattern that should be documented

**Triggers:**
- `reflect`
- "What did we learn this session?"
- "Suggest CLAUDE.md improvements"

## Process

### Phase 1: Session Review

1. Review the conversation history for:
   - Mistakes the agent made (wrong commands, bad patterns, missed conventions)
   - Corrections the user made
   - Repeated clarifications on the same topic
   - Tools or commands the agent had to discover mid-session
   - Patterns that worked well and should be reinforced

2. Categorize findings:
   - **Prohibitions** — things the agent did that it shouldn't (add to CLAUDE.md)
   - **Missing commands** — dev/test/build commands the agent guessed at (add to CLAUDE.md)
   - **Missing conventions** — code style or patterns the agent didn't follow (add to CLAUDE.md)
   - **Recurring procedures** — multi-step workflows repeated manually (extract to a new skill)
   - **Model-specific quirks** — behaviors that changed after a model update (note in CLAUDE.md)

### Phase 2: Draft Suggestions

For each finding, draft a concrete suggestion:

**For CLAUDE.md additions:**
- Write the exact text to add
- Specify which section it belongs in
- Keep it behavior-changing and concise — no prose that doesn't affect agent behavior
- Prefer imperative directives: "Never X" or "Always Y" or "When X, do Y"

**For new skills:**
- Write a one-paragraph skill proposal
- Explain what triggers it and what it produces
- Only propose if the procedure has 3+ steps and is likely to recur

### Phase 3: Present for Review

Output a structured report:

```markdown
## Session Reflection

### CLAUDE.md Suggestions

#### 1. [Section: X] — Add prohibition
> Never do Y because Z.

#### 2. [Section: X] — Add command
> `command here` — explanation

### New Skill Proposals

#### 1. skill-name
One paragraph describing the skill, its trigger, and its output.

### No Action Needed
- [List things that went well / are already documented]
```

### Phase 4: Apply (with approval)

- Only apply changes if the user approves
- Use Edit tool for CLAUDE.md modifications
- Use Write tool for new skill files
- Keep CLAUDE.md lean — if it's getting bloated, suggest moving content to skills instead

## Quality Criteria

A good reflection output:
- Contains at least one actionable suggestion (or explicitly states "nothing to improve")
- Each suggestion is specific enough to copy-paste into CLAUDE.md
- Doesn't add prose or documentation — only behavior-changing directives
- Doesn't duplicate existing CLAUDE.md content
- Proposes skills only for genuinely recurring multi-step procedures
