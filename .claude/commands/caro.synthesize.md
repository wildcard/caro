---
description: Synthesize cognitive state from all project sources into a unified "what matters now" view
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `ROADMAP.md`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/caro.synthesize` | Full synthesis of all state sources |
| `/caro.synthesize --quick` | Quick refresh of project-state.md only |
| `/caro.synthesize --priorities` | Show just the top 3 priorities |

---

## What This Command Does

`/caro.synthesize` reads across all project state sources and produces a unified cognitive state - a single "what matters right now" summary that any session can use to orient itself instantly.

This solves the "50 First Dates" problem: instead of re-briefing each session from scratch, every session reads the cognitive state and knows what's happening.

**Output**: Updates `.claude/memory/cognitive-state.md`

---

## Process

### Step 1: Gather State from All Sources

Read these files (use parallel agents where possible):

1. **Project metadata**:
   - `Cargo.toml` - current version
   - `git branch -a` - active branches
   - `git log --oneline -10` - recent commits

2. **Roadmap & priorities**:
   - `ROADMAP.md` - active milestones and items marked `active` or in-progress

3. **Quality state**:
   - `.claude/memory/qa-bugs-backlog.md` - active bug count
   - `.claude/memory/known-bugs.md` - recently resolved bugs

4. **Feedback pipeline**:
   - `.claude/memory/feedback-insights.yaml` - total insights, untriaged count, top themes

5. **Active work**:
   - `thoughts/shared/handoffs/` - any handoff docs from last 7 days
   - `thoughts/ledgers/` - any active continuity ledgers

6. **Open PRs** (if GitHub MCP available):
   - List open PRs with status

### Step 2: Synthesize Priorities

Determine the **top 3 priorities** based on:

1. **Urgency**: Active blockers > critical bugs > high-severity feedback > roadmap items
2. **Momentum**: Work already in-progress should generally continue
3. **Impact**: Items affecting users or blocking releases rank higher
4. **Feedback signal**: Themes appearing across multiple feedback sources rank higher

### Step 3: Identify Risks

Look for:
- Stale handoffs (> 7 days old with status != complete)
- Critical/high severity untriaged feedback
- Failing CI or build issues
- Overdue roadmap items

### Step 4: Write Cognitive State

Update `.claude/memory/cognitive-state.md` with this structure:

```markdown
# What Matters Now

> Synthesized by `/caro.synthesize`. Reads across all state sources.

**Last synthesized**: [ISO date]

## Top 3 Priorities

1. **[Title]** - [Why this matters, what to do next]
2. **[Title]** - [Why this matters, what to do next]
3. **[Title]** - [Why this matters, what to do next]

## Active Risks

- [Risk description and mitigation]

## Recent Wins

- [Completed items from last 7 days]

## Feedback Themes

- [Top themes from feedback-insights.yaml, or "No feedback collected yet"]

## Active Work

- [Branch]: [Description and status]

## Quick Reference

| Source File | What It Tracks |
|-------------|---------------|
| `.claude/memory/project-state.md` | Version, branches, PRs, bugs |
| `.claude/memory/feedback-insights.yaml` | User feedback insights |
| `.claude/memory/qa-bugs-backlog.md` | Active bugs |
| `ROADMAP.md` | Milestones and timeline |
| `thoughts/shared/handoffs/` | Session handoff documents |
| `thoughts/ledgers/` | Within-session continuity ledgers |
```

### Step 5: Update Project State

Also refresh `.claude/memory/project-state.md` with current data from Step 1.

### Step 6: Report

Output a concise summary:

```
Cognitive state synthesized.

Top 3 priorities:
1. [Priority 1]
2. [Priority 2]
3. [Priority 3]

Risks: [count] | Feedback: [untriaged]/[total] | Bugs: [active count]
Updated: .claude/memory/cognitive-state.md
```
