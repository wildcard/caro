---
description: Capture and structure user feedback from any source (manual, GitHub, beta tests) into actionable insights
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
| `/feedback-intake` | Interactive mode - asks for feedback source and content |
| `/feedback-intake <raw text>` | Process pasted feedback directly |
| `/feedback-intake --github #123` | Extract feedback from GitHub issue #123 |
| `/feedback-intake --beta-cycle <path>` | Import failures from a beta test cycle result |
| `/feedback-intake --list` | Show all insights, grouped by status |
| `/feedback-intake --stats` | Show feedback pipeline statistics |

---

## What This Command Does

`/feedback-intake` is the universal entry point for the feedback pipeline. It accepts raw, unstructured feedback from any source and transforms it into structured insights stored in `.claude/memory/feedback-insights.yaml`.

This is the first step in the Canopy-lite pipeline:
```
feedback --> /feedback-intake --> insights.yaml --> /insight-to-issue --> GitHub issue --> /caro.roadmap --> implementation
```

---

## Process

### Step 1: Determine Source

Based on arguments:

- **No arguments or raw text**: Ask the user what type of feedback they have
- **`--github #NNN`**: Read the GitHub issue/discussion using MCP tools
- **`--beta-cycle <path>`**: Read the beta test cycle result file
- **Raw text as argument**: Treat as manual/pasted feedback

### Step 2: Read Existing Insights

```bash
cat .claude/memory/feedback-insights.yaml
```

Parse to determine:
- Current `total_insights` count (for generating next ID)
- Existing insights (for deduplication check)

### Step 3: Extract Structured Insight

For each piece of feedback, determine:

| Field | How to Determine |
|-------|-----------------|
| `id` | `fb-YYYY-MM-DD-NNN` where NNN is sequential |
| `source` | `manual`, `github`, or `beta-test` |
| `source_ref` | Issue number, cycle ID, or "pasted feedback" |
| `raw` | Verbatim text (truncated to 500 chars if longer) |
| `category` | Classify as: `bug`, `ux`, `feature`, `docs`, `confusion` |
| `severity` | Assess: `critical`, `high`, `medium`, `low` |
| `user_segment` | Match to personas: `devops`, `data-scientist`, `sysadmin`, `student`, `security` |
| `insight` | Rewrite as: "Users expect X but get Y because Z" |
| `actionable` | Is there a concrete action we can take? |
| `action` | Brief description of what to do |
| `status` | Always starts as `new` |
| `linked_issue` | `null` until `/insight-to-issue` creates one |
| `created` | Today's date |

### Step 4: Deduplication Check

Before adding, check if a similar insight already exists:
- Same category + similar raw text = likely duplicate
- If duplicate found, note it and ask user whether to merge or add separately

### Step 5: Write to Insights Store

Append the new insight to `.claude/memory/feedback-insights.yaml`:
- Update `metadata.total_insights` count
- Update `metadata.untriaged` count
- Update `metadata.last_updated` date
- Add new insight entry to `insights` array

### Step 6: Report

Output:

```
Feedback captured!

ID: fb-2026-04-12-001
Category: [category] | Severity: [severity]
Segment: [user_segment]
Insight: [structured insight statement]
Action: [suggested action]

Pipeline status: [untriaged]/[total] insights pending triage
Next step: Run /insight-to-issue to create GitHub issues from insights
```

---

## Source-Specific Handling

### Manual Paste

When the user pastes raw text:
1. Read the text carefully
2. Identify the core complaint, request, or observation
3. Determine if it contains multiple distinct insights (split if so)
4. Ask clarifying questions only if the feedback is genuinely ambiguous

### GitHub Issue/Discussion

When `--github #NNN` is provided:
1. Read the issue using GitHub MCP tools
2. Extract the core feedback from issue body and comments
3. Note the issue author's apparent experience level for `user_segment`
4. Set `source_ref` to the issue number
5. If the issue already tracks the feedback, set `linked_issue` to the issue number

### Beta Test Cycle

When `--beta-cycle <path>` is provided:
1. Read the cycle result file
2. For each test failure:
   - Create one insight per distinct failure mode
   - Set `source` to `beta-test`
   - Set `source_ref` to the cycle identifier and profile ID
   - Map beta tester profile to `user_segment`:
     - bt_001 (Alex, novice) -> `student`
     - bt_002 (Jordan, power user) -> `devops`
     - bt_003 (Sam, corporate IT) -> `security`
     - bt_005 (Taylor, SRE) -> `devops`
     - bt_006 (Riley, data scientist) -> `data-scientist`
     - bt_007 (Yuki, Japanese dev) -> `devops`
     - bt_009 (Jamie, accessibility) -> `student`
     - bt_010 (Chris, SSH-only) -> `sysadmin`
   - Categorize failure as `bug` (wrong output), `ux` (confusing), or `feature` (missing capability)

---

## User Segments (from User Research Strategy)

| Segment | Persona | Characteristics |
|---------|---------|----------------|
| `devops` | Sarah (DevOps Engineer) | kubectl, docker, terraform, 8yr exp |
| `data-scientist` | Marcus (Data Scientist) | Python expert, CLI novice, 3yr exp |
| `sysadmin` | Alex (System Administrator) | Server management, monitoring |
| `student` | Priya (CS Student) | Learning CLI, needs explanations |
| `security` | Jordan (Security Engineer) | Pentesting, auditing, safety-conscious |

---

## Examples

### Manual feedback
```
/feedback-intake Users are confused about what --dry-run does. They expect it to show the command but not execute it, but it's not clear from the help text.
```

### GitHub issue
```
/feedback-intake --github #456
```

### Beta test results
```
/feedback-intake --beta-cycle .claude/beta-testing/cycles/2026-04-12-bt_005.md
```

### Stats
```
/feedback-intake --stats
```
