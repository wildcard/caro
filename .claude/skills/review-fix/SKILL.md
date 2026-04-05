---
name: "review-fix"
description: "Iterative code review and fix loop using a second model as reviewer. Cycles until no issues remain."
version: "1.0.0"
allowed-tools: "Bash, Read, Edit, Grep, Glob, Agent"
license: "AGPL-3.0"
---

# Review+Fix Skill

## What This Skill Does

Runs an iterative review-then-fix loop on uncommitted changes. A sub-agent (different model) reviews the dirty diff, reports issues, then the primary agent fixes them. Repeats until clean.

Inspired by hboon's `review+fix` and `review-plus-fix-relentlessly` skills. See `docs/research/agentic-workflow-patterns-hboon.md`.

**Typical convergence:** 2-3 cycles.

## When to Use This Skill

- After completing a feature before committing
- As part of the `take-over` chain
- When you want a fresh perspective on your changes
- Before creating a PR

**Triggers:**
- `review-fix` or `review+fix`
- "Review and fix my changes"
- "Clean up before commit"

## Process

### Phase 1: Capture Dirty Diff

```bash
git diff
git diff --cached
git diff --stat
```

Collect the full uncommitted diff (staged + unstaged).

### Phase 2: Spawn Reviewer

Launch a sub-agent (use **haiku** or **sonnet** model for cost efficiency) with this prompt:

```
You are a code reviewer. Review the following diff for:
1. Bugs or logic errors
2. Missing error handling at system boundaries
3. Security issues (OWASP top 10)
4. Dead code or unnecessary changes
5. Violations of project conventions (check CLAUDE.md)

Do NOT flag:
- Style preferences or formatting (handled by formatters)
- Missing documentation or comments
- Hypothetical future issues

For each issue found, report:
- File and line number
- What's wrong
- Suggested fix (one sentence)

If no issues found, respond with exactly: "LGTM"

Diff:
[paste diff here]
```

### Phase 3: Fix Issues

For each issue the reviewer flagged:
1. Read the relevant file
2. Apply the fix
3. Mark the issue as resolved

Skip any reviewer suggestions that:
- Contradict project conventions in CLAUDE.md
- Add unnecessary complexity
- Are style-only preferences

### Phase 4: Re-review (if needed)

If fixes were applied:
1. Capture the new diff
2. Spawn reviewer again with updated diff
3. Repeat until reviewer responds "LGTM" or no actionable issues remain

**Max iterations:** 5 (to prevent infinite loops)

Report cycle count: "Review+fix complete after N cycles."

### Phase 5: Summary

Output a brief summary:

```markdown
## Review+Fix Complete

- **Cycles:** N
- **Issues found:** X (Y fixed, Z skipped)
- **Skipped reasons:** [if any]
```

## Composability

This skill is designed to be called by other skills:
- `take-over` calls `review-fix` as its first step
- Can be followed by `session-cleanup` and then commit

## Configuration

The reviewer model can be adjusted based on cost/quality tradeoff:
- **haiku** — fastest, cheapest, good for obvious issues
- **sonnet** — balanced, catches more subtle issues
- **opus** — most thorough, use for critical code
