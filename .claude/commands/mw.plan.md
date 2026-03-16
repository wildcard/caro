---
description: Multi-model enriched planning — wraps spec-kitty.plan with external model pre-research
---

## User Input

```text
$ARGUMENTS
```

---

## What This Command Does

`/mw.plan` enhances the standard `/spec-kitty.plan` workflow by injecting external model research before the planning phase begins. It decomposes the feature into research questions, dispatches them to Codex and/or Gemini, synthesizes findings, and then feeds the enriched context into spec-kitty's Phase 0.

**Flow:**
```
Feature description
  → Decompose into research questions
  → Dispatch to external models (parallel)
  → Synthesize Research Digest
  → Feed into spec-kitty Phase 0 (research.md)
  → Continue with /spec-kitty.plan
```

---

## Execution Steps

### 1. Parse Feature Description

Extract feature description from `$ARGUMENTS`.

If empty, ask: "What feature are you planning? Provide a description."

### 2. Check Context

Detect if we're in a spec-kitty worktree:
```bash
git branch --show-current 2>/dev/null
test -d kitty-specs/ 2>/dev/null && echo "kitty-specs-exists"
```

If `spec.md` doesn't exist yet, suggest running `/spec-kitty.specify` first.

### 3. Decompose Into Research Questions

Based on the feature description, identify 2-3 targeted research questions:

**Question Categories:**
- **Architecture**: "What patterns exist for <X> in Rust CLI tools?"
- **Implementation**: "What are the trade-offs between <A> and <B> for this use case?"
- **Safety**: "What security considerations apply to <X>?"
- **Ecosystem**: "What crates/libraries support <X>? Compare maturity and maintenance."

**Example decomposition for "Add Redis caching with TTL support":**
1. [Codex] "Analyze Rust Redis crate options (redis-rs vs fred vs deadpool-redis) for a CLI tool with async support. Focus on connection pooling, TTL operations, and error handling patterns."
2. [Gemini] "Research TTL-based caching patterns for CLI tools. Compare in-memory vs external cache trade-offs. Consider cold-start latency implications."
3. [Both] "Review existing Caro backend architecture (provided below) and recommend integration points for a caching layer."

### 4. Dispatch Research

Run `/mw.research` with each decomposed question. If both models are available, dispatch in parallel.

If no external models are available, perform Claude-only research using:
- Grep/Read to analyze existing codebase
- WebSearch for ecosystem research
- Agent tool for deep code exploration

### 5. Produce Enriched Planning Context

Combine all research findings into an enriched planning context:

```markdown
## Pre-Planning Research

### Feature: <description>

### Research Questions Investigated
1. <question 1> → <summary finding>
2. <question 2> → <summary finding>
3. <question 3> → <summary finding>

### Architecture Recommendations
<Cross-validated recommendations from external models + Claude analysis>

### Implementation Approach
<Recommended approach with rationale>

### Risks & Mitigations
| Risk | Mitigation | Source |
|------|-----------|--------|
| <risk> | <mitigation> | [codex/gemini/claude] |

### Dependencies
- <crate or system dependency identified>

### Open Questions for Planning Phase
- <anything that needs user input during planning>
```

### 6. Feed Into Spec-Kitty

If in a worktree with `kitty-specs/`:
1. Write enriched context to `research.md` (or append if it exists)
2. Invoke `/spec-kitty.plan` — the planning phase will read `research.md` during Phase 0

If NOT in a worktree:
1. Display the enriched context
2. Suggest: "Run `/spec-kitty.plan` to continue with this enriched context"

---

## When to Use `/mw.plan` vs `/spec-kitty.plan`

| Scenario | Use |
|----------|-----|
| Complex feature with unknowns | `/mw.plan` — external research reduces unknowns |
| Simple feature, well-understood | `/spec-kitty.plan` — skip external research |
| Architecture decision needed | `/mw.plan` — cross-validate with multiple models |
| Time-critical planning | `/spec-kitty.plan` — faster without external calls |
| Unfamiliar technology | `/mw.plan` — leverage model-specific strengths |

---

## Example

```
User: /mw.plan "Add distributed locking for concurrent caro instances"

Claude decomposes:
  Q1 [Codex]: "Analyze Rust distributed locking patterns (file locks vs advisory locks vs named pipes)"
  Q2 [Gemini]: "Research POSIX file locking guarantees across Linux/macOS for CLI tools"
  Q3 [Both]: "Review caro's current execution model and identify race conditions"

Dispatches to models, collects findings...

Research Digest written to research.md:
  - Codex recommends flock() with timeout
  - Gemini confirms POSIX portability of flock()
  - Both agree on atomic PID file pattern as fallback
  - Claude resolves: Use flock() primary, PID file fallback

Continues with /spec-kitty.plan using enriched context.
```
