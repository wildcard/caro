---
description: Convert structured feedback insights into GitHub issues with deduplication, labeling, and roadmap linking
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `.claude/memory/feedback-insights.yaml`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/insight-to-issue` | Process all untriaged insights into GitHub issues |
| `/insight-to-issue --dry-run` | Show what issues would be created without creating them |
| `/insight-to-issue --id fb-2026-04-12-001` | Process a specific insight by ID |
| `/insight-to-issue --stats` | Show pipeline statistics |

---

## What This Command Does

`/insight-to-issue` is the second step in the feedback pipeline. It reads structured insights from `.claude/memory/feedback-insights.yaml` and converts them into GitHub issues, with deduplication, labeling, and roadmap linking.

```
/feedback-intake --> insights.yaml --> /insight-to-issue --> GitHub issues --> /caro.roadmap
```

---

## Process

### Step 1: Read Insights Store

Read `.claude/memory/feedback-insights.yaml` and filter for:
- `status: "new"` (untriaged insights)
- Or specific ID if `--id` flag provided

If no untriaged insights exist, report:
```
No untriaged insights found. Use /feedback-intake to capture feedback first.
```

### Step 2: Group Related Insights

Before creating issues, group insights that relate to the same underlying problem:

- Same `category` + similar `insight` text = likely same issue
- Same `action` suggestion = likely same issue
- Multiple insights from different `user_segment` values about the same topic = stronger signal

For each group, note:
- Number of insights in the group (signal strength)
- Which user segments are affected
- Highest severity in the group

### Step 3: Check for Existing Issues

For each group, search existing GitHub issues to avoid duplicates:
- Search by keywords from the insight text
- Check for issues with matching labels
- If a matching open issue exists, add a comment with the new feedback data instead of creating a duplicate

### Step 4: Create GitHub Issues

For each insight group that doesn't have an existing issue, create one:

**Title**: Concise description of the problem or request (under 70 chars)

**Body**:
```markdown
## Feedback Summary

[Structured insight statement from the group]

## Source Data

| Field | Value |
|-------|-------|
| Insights | [count] related feedback items |
| Category | [category] |
| Severity | [highest in group] |
| Segments affected | [list of user segments] |
| Sources | [list of sources: manual, github, beta-test] |

## Raw Feedback

[Verbatim quotes from the `raw` fields, attributed by source]

## Suggested Action

[Action from the insight, refined based on group analysis]

---
*Created by `/insight-to-issue` from feedback pipeline*
*Insight IDs: [list of fb-YYYY-MM-DD-NNN]*
```

**Labels** (create if they don't exist):
- `feedback` (always)
- Category label: `feedback:bug`, `feedback:ux`, `feedback:feature`, `feedback:docs`, `feedback:confusion`
- Severity label: `severity:critical`, `severity:high`, `severity:medium`, `severity:low`

**Milestone**: Map to roadmap milestone based on category:
- `bug` with severity critical/high -> current milestone
- `feature` -> next milestone
- `ux`/`docs`/`confusion` -> current or next based on severity

### Step 5: Update Insights Store

For each processed insight:
- Set `status` to `triaged`
- Set `linked_issue` to the created issue number (e.g., `#456`)
- Update `metadata.untriaged` count
- Update `metadata.last_updated`

Write updated YAML back to `.claude/memory/feedback-insights.yaml`.

### Step 6: Report

Output:

```
Insight-to-issue pipeline complete!

Created:
  - #456: [title] (3 insights, severity: high, segments: devops, sysadmin)
  - #457: [title] (1 insight, severity: medium, segments: student)

Updated existing:
  - #123: Added feedback from 2 new insights

Skipped:
  - fb-2026-04-12-003: Duplicate of #456

Pipeline status: [remaining untriaged]/[total] insights
```

---

## Dry Run Mode

When `--dry-run` is used:
- Go through the full process but don't create issues or update the YAML
- Show what would be created/updated
- Useful for reviewing before committing to issue creation

---

## Stats Mode

When `--stats` is used, read the insights store and report:

```
Feedback Pipeline Stats
=======================
Total insights: [N]
By status:
  - new (untriaged): [N]
  - triaged: [N]
  - in-progress: [N]
  - resolved: [N]
  - wont-fix: [N]

By category:
  - bug: [N]
  - ux: [N]
  - feature: [N]
  - docs: [N]
  - confusion: [N]

By severity:
  - critical: [N]
  - high: [N]
  - medium: [N]
  - low: [N]

By source:
  - manual: [N]
  - github: [N]
  - beta-test: [N]

Top themes: [most common insight patterns]
```
