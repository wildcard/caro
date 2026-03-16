---
description: Multi-model code review — dispatch code to external models for review perspectives, Claude triages
---

## User Input

```text
$ARGUMENTS
```

---

## What This Command Does

`/mw.review` sends code to external CLI models (Codex, Gemini) for review, collects structured feedback, and lets Claude triage each suggestion. External models see code as text input only — they never access files directly or apply changes.

**Security**: All suggestions are text-only. Claude decides what to apply. External models are reviewers, never actors.

---

## Execution Steps

### 1. Parse Review Target

From `$ARGUMENTS`, determine what to review:

| Input | Action |
|-------|--------|
| File path(s) | Read specified files |
| PR number (e.g., `#123`) | Fetch PR diff via `gh pr diff 123` |
| "staged" or "diff" | Use `git diff --staged` |
| Empty | Use `git diff` (unstaged changes) |

### 2. Collect Code to Review

Read the target code using the Read tool. For large files, focus on:
- Changed sections (if reviewing a diff)
- Public API surfaces
- Safety-critical code paths
- Complex logic sections

**Size limit**: Cap at 500 lines per model invocation. For larger reviews, split into focused sections.

### 3. Check Model Availability

```bash
CODEX_OK=$(command -v codex >/dev/null 2>&1 && echo "true" || echo "false")
GEMINI_OK=$(command -v gemini >/dev/null 2>&1 && echo "true" || echo "false")
echo "codex:$CODEX_OK gemini:$GEMINI_OK"
```

If neither available, Claude performs the review solo (still valuable, just single-perspective).

### 4. Dispatch for Review

**Review Prompt Template:**
```
CODE REVIEW TASK

Review the following Rust code for:
1. Correctness - Logic errors, edge cases, off-by-one errors
2. Safety - Security vulnerabilities, unsafe patterns, OWASP concerns
3. Performance - Unnecessary allocations, blocking calls, O(n) improvements
4. Idiomatic Rust - Clippy-level issues, better patterns, error handling
5. Maintainability - Naming, structure, documentation needs

CONTEXT:
Project: Caro - Rust CLI for natural language to shell command conversion
This code is part of: <module description>

CODE:
```rust
<code content>
```

OUTPUT FORMAT:
## Issues Found

### Critical
- [CRIT-1] <description> (line ~N)
  Suggestion: <fix>

### Important
- [IMP-1] <description> (line ~N)
  Suggestion: <fix>

### Minor
- [MIN-1] <description> (line ~N)
  Suggestion: <fix>

## Positive Observations
- <what's done well>

## Overall Assessment
<1-2 sentence summary>
```

Dispatch to both available models in parallel via Bash:

**Codex:**
```bash
timeout 90 codex --approval-mode full-auto -q "<review prompt>" 2>/dev/null
```

**Gemini:**
```bash
echo "<review prompt>" | timeout 60 gemini 2>/dev/null
```

### 5. Collect and Deduplicate

Parse both model outputs and:
1. Group findings by file/line location
2. Identify duplicate findings (both models flagged same issue → high confidence)
3. Tag each finding with source model
4. Sort by severity: Critical → Important → Minor

### 6. Claude Triage

For each finding, Claude decides:

| Decision | Meaning | Action |
|----------|---------|--------|
| **Accept** | Finding is valid and actionable | Apply the fix using Edit tool |
| **Reject** | Finding is incorrect or not applicable | Note reason, skip |
| **Modify** | Direction is right, but fix needs adjustment | Apply modified version |
| **Defer** | Valid but out of scope for this review | Create TODO or note |

Present the triage as a formatted table:

```markdown
## Review Triage

| # | Severity | Finding | Source | Decision | Reason |
|---|----------|---------|--------|----------|--------|
| 1 | CRIT | Buffer overflow in X | codex+gemini | ACCEPT | Both models agree, confirmed |
| 2 | IMP | Missing error handling | codex | MODIFY | Good catch, but use anyhow not thiserror |
| 3 | MIN | Naming convention | gemini | REJECT | Follows project convention |
| 4 | IMP | Potential race condition | codex | DEFER | Out of scope, filed as TODO |
```

### 7. Apply Accepted Changes

For each ACCEPT or MODIFY decision:
1. Use the Edit tool to apply the fix
2. Verify the fix doesn't break existing tests: `cargo test`
3. Note the change in the review summary

### 8. Review Summary

```markdown
## Review Complete

### Stats
- External models used: [codex, gemini, both, none]
- Total findings: N
- Accepted: N | Modified: N | Rejected: N | Deferred: N

### Changes Applied
1. <file:line> — <description of change>
2. <file:line> — <description of change>

### Deferred Items
- [ ] <deferred finding for future work>

### Confidence Assessment
<How confident Claude is in the review quality based on model agreement/disagreement>
```

---

## Examples

### Review Specific File
```
User: /mw.review src/safety/patterns.rs
→ Reads file, dispatches to both models, triages findings
```

### Review PR
```
User: /mw.review #42
→ Fetches PR diff, dispatches changed code to models, triages
```

### Review Staged Changes
```
User: /mw.review staged
→ Gets git diff --staged, dispatches to models, triages
```

---

## Important: Security Boundaries

1. External models receive code as **text in the prompt**, never as file paths
2. External models **never execute** any commands
3. External models **suggest** changes as text; Claude **applies** them via Edit tool
4. All suggested commands go through Caro's **safety validator** before execution
5. Claude can **reject any suggestion** regardless of model confidence
