---
name: discuss
description: Optional pre-planning discussion phase for complex features. Use when implementation preferences, constraints, or trade-offs need clarification before diving into specifications. Inspired by GSD's discuss phase.
---

# DISCUSS Phase (Optional)

You are facilitating a **pre-planning discussion** to capture implementation preferences, constraints, and trade-offs before specification work begins.

## When This Phase is Recommended

Agents should recommend `/discuss` when:
- Feature has multiple valid implementation approaches
- Significant architectural decisions needed
- User preferences about trade-offs are unclear
- Cross-cutting concerns affect multiple systems
- Performance vs simplicity trade-offs exist
- External dependencies or integrations involved
- Feature scope is ambiguous

## Discussion Framework

### 1. Scope Clarification
Ask about boundaries:
- What's explicitly IN scope?
- What's explicitly OUT of scope?
- What are the "nice to haves" vs "must haves"?

### 2. Implementation Preferences
Capture opinions on:
- **Performance vs Simplicity**: "Should this be optimized for speed or maintainability?"
- **Flexibility vs Constraints**: "Should this be configurable or opinionated?"
- **Completeness vs Speed**: "MVP first or full feature?"

### 3. Technical Constraints
Identify limitations:
- Must integrate with existing system X?
- Cannot use dependency Y?
- Must maintain backwards compatibility?
- Performance requirements?

### 4. Trade-off Decisions
Document explicit choices:
- "We chose X over Y because..."
- "We're accepting limitation Z because..."

## Output Format

After discussion, create a **DISCUSS.md** file in the feature directory:

```markdown
# Discussion Summary: [Feature Name]

**Date**: YYYY-MM-DD
**Participants**: [who was involved]

## Scope

### In Scope
- [item 1]
- [item 2]

### Out of Scope
- [item 1]
- [item 2]

### Nice to Have (if time permits)
- [item 1]

## Implementation Preferences

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Performance vs Simplicity | Simplicity | MVP phase, optimize later |
| Config vs Convention | Convention | Reduce user decisions |

## Technical Constraints

- Must use existing `Backend` trait
- Cannot add new dependencies > 1MB
- Must maintain CLI backwards compatibility

## Trade-off Decisions

1. **Chose sync over async** because the operation is fast and async adds complexity
2. **Chose JSON over YAML** because serde_json is already a dependency

## Open Questions (for spec phase)

- [ ] How should errors be surfaced to users?
- [ ] What's the fallback if external service unavailable?

## Next Steps

Proceed to `/spec-kitty.specify` with these constraints documented.
```

## Integration with Workflow

```
Feature Idea
    ↓
[Optional] /discuss  ← YOU ARE HERE (if complex)
    ↓
/spec-kitty.specify → spec.md
    ↓
/spec-kitty.plan → plan.md
    ↓
... (rest of workflow)
```

## Agent Recommendation Triggers

When processing a feature request, recommend `/discuss` if you detect:

1. **Ambiguous scope**: "Add caching" (what kind? where? how much?)
2. **Multiple approaches**: Could use Redis, file-based, or in-memory
3. **Trade-off language**: "fast but simple", "flexible but easy"
4. **Integration complexity**: Touches 3+ existing systems
5. **User uncertainty**: "I'm not sure if...", "maybe we should..."

Example recommendation:
```
This feature has several valid implementation approaches (in-memory vs
file-based caching, eager vs lazy loading). I recommend running `/discuss`
first to capture your preferences before we create the specification.

Would you like to start a discussion phase?
```

## Skip Discussion When

- Feature is well-defined with clear scope
- It's a bug fix with obvious solution
- User has already specified all constraints
- Small enhancement (< 1 day work)
- Following an existing pattern in codebase

Remember: Discussion is **optional** but valuable for complex work. The goal is to prevent rework by capturing decisions upfront.
