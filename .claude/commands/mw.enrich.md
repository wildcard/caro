---
description: Synthesize external model outputs into structured context for Claude's decision-making
---

## User Input

```text
$ARGUMENTS
```

---

## What This Command Does

`/mw.enrich` takes raw output from external model invocations (Codex, Gemini) and synthesizes it into a structured "Enriched Context" block. This is the synthesis step between raw research and Claude's decision-making.

Use this when:
- You've manually run external models and want to structure the output
- You have research from a previous session to incorporate
- You want to merge findings from multiple sources into a single context block

---

## Execution Steps

### 1. Collect Raw Input

Input can come from:
- `$ARGUMENTS` — Raw text pasted by user
- Previous `/mw.research` output — Already structured, needs merging
- External model output copied from another terminal

If `$ARGUMENTS` is empty, ask:
```
Paste the external model output you want to enrich, or specify a file path containing the output.
```

### 2. Identify Sources

Parse the input to identify which model(s) produced it:
- Look for `[source: codex]` or `[source: gemini]` tags
- If no tags, ask: "Which model produced this output? (codex/gemini/other)"
- If multiple sources, separate and tag each section

### 3. Structure the Output

Transform raw output into the Enriched Context format:

```markdown
## Enriched Context

### Source Attribution
| Source | Status | Confidence |
|--------|--------|------------|
| Codex | [provided/missing] | [high/medium/low] |
| Gemini | [provided/missing] | [high/medium/low] |
| Claude | synthesizing | — |

### Key Findings (Cross-Validated)
<Findings that appear in multiple sources or align with codebase patterns>
- Finding 1 [codex + gemini agree]
- Finding 2 [codex, confirmed by codebase analysis]

### Source-Specific Findings

#### From Codex
- <tagged finding>
- <tagged finding>

#### From Gemini
- <tagged finding>
- <tagged finding>

### Contradictions & Resolutions
| Topic | Codex Says | Gemini Says | Claude's Decision |
|-------|-----------|-------------|-------------------|
| <topic> | <position> | <position> | <resolution + reasoning> |

### Actionable Recommendations
1. <Specific recommendation with confidence level>
2. <Specific recommendation>

### Context for Next Step
<Summary paragraph ready to be consumed by planning, implementation, or review phases>
```

### 4. Validate Against Codebase

Where possible, validate external findings against the actual codebase:
- Use Grep to check if recommended patterns already exist
- Use Read to verify claims about current implementation
- Flag any findings that contradict what's actually in the code

### 5. Output or Append

**Default**: Display the Enriched Context in the conversation for Claude to use.

**If in spec-kitty workflow**: Offer to append to `research.md`:
```
Enriched context ready. Append to research.md? [Y/n]
```

**If user specifies a file**: Write to the specified path.

---

## Integration Notes

- This command is automatically invoked by `/mw.research` after collecting external outputs
- Can also be used standalone to process manually-collected research
- The Enriched Context format is designed to be consumed by `/mw.plan` and `/spec-kitty.plan`
- All contradictions MUST include Claude's resolution — never leave unresolved conflicts
