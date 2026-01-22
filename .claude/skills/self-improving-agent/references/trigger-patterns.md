# Self-Improving Agent Trigger Patterns

Quick reference for detecting when to capture learnings.

## User Correction Signals

Watch for these phrases that indicate a correction:

```
"No, that's wrong"
"Actually..."
"That's not right"
"That's not what I meant"
"Not quite"
"Close, but..."
"The correct way is..."
"You should use X instead"
"That won't work because..."
```

## Error Indicators

System/tool failures to capture:

- Command returns non-zero exit code
- API returns 4xx/5xx status
- "Error:", "Failed:", "not found"
- Timeout or connection failures
- Permission denied
- Syntax errors in generated code

## Capability Gap Signals

User requests that can't be fulfilled:

```
"Can you...?" (followed by something not possible)
"Is there a way to...?"
"I wish it could..."
"Does it support...?"
"Why can't you...?"
```

## Outdated Knowledge Signals

Indicators of stale information:

- "That API endpoint no longer exists"
- "That command is deprecated"
- "The new version uses..."
- Version mismatches
- Documentation URLs that 404

## Improvement Discovery Signals

Finding better approaches:

- "There's a simpler way..."
- Realizing a pattern could be reused
- Finding existing code that does the same thing
- User shows more efficient approach
- Post-implementation "should have done X"

## When to Capture

**Always capture:**
- Errors that took > 1 attempt to fix
- User corrections
- Knowledge gaps that caused confusion
- Better patterns discovered mid-task

**Skip capturing:**
- Trivial typos
- One-off mistakes with obvious cause
- User preference without general applicability
- Already documented in CLAUDE.md
