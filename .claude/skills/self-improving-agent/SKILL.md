---
name: self-improving-agent
description: Captures learnings, errors, and corrections to enable continuous improvement
version: 1.0.1
source: https://clawdhub.com/pskoett/self-improving-agent
---

# Self-Improving Agent

Captures learnings, errors, and corrections to enable continuous improvement across sessions.

## When to Use

Activate this skill when:

1. **Command or operation fails unexpectedly** - Capture what went wrong and the fix
2. **User corrects Claude** - "No, that's wrong...", "Actually...", "That's not right"
3. **User requests a capability that doesn't exist** - Feature gap identified
4. **External API or tool fails** - Integration issues discovered
5. **Knowledge is outdated or incorrect** - Stale information found
6. **Better approach discovered** - Improved pattern for recurring task

Also: **Review learnings before major tasks** to avoid repeating past mistakes.

## Workflow

### Phase 1: Detect Learning Opportunity

Watch for triggers:
- Error messages or unexpected behavior
- User corrections ("No", "Actually", "That's not how...")
- Missing capabilities ("Can you...", "Is there a way to...")
- Failed commands or API calls
- Discovering better approaches mid-task

### Phase 2: Capture the Learning

Create a learning entry at `.claude/memory/learnings/YYYY-MM-DD_<topic>.md`:

```markdown
---
date: <ISO timestamp>
type: <error|correction|gap|outdated|improvement>
category: <command|api|pattern|knowledge|capability>
severity: <critical|important|minor>
---

# Learning: <brief title>

## Context
<What was being attempted when this was discovered?>

## What Happened
<Describe the error, correction, or discovery>

## Root Cause
<Why did this happen?>

## Resolution
<What was the fix or improvement?>

## Prevention
<How to avoid this in the future?>

## Related Files
- <file:line references>

## Tags
<relevant-tags for searchability>
```

### Phase 3: Index and Categorize

Update the learning index at `.claude/memory/learnings/INDEX.md`:

```markdown
# Learnings Index

Last updated: <timestamp>

## By Category
### Commands
- [YYYY-MM-DD] <title> - <brief summary>

### APIs
- ...

### Patterns
- ...

## By Severity
### Critical
- ...

### Important
- ...
```

### Phase 4: Review Before Major Tasks

Before starting significant work:

1. **Check relevant learnings:**
   ```bash
   grep -l "<topic>" .claude/memory/learnings/*.md
   ```

2. **Review recent learnings:**
   ```bash
   ls -lt .claude/memory/learnings/*.md | head -10
   ```

3. **Check the index:**
   Read `.claude/memory/learnings/INDEX.md` for categorized learnings

## Learning Types

| Type | Description | Example |
|------|-------------|---------|
| `error` | Operation failed | Command syntax error on BSD vs GNU |
| `correction` | User corrected output | Wrong file path assumption |
| `gap` | Missing capability | No support for X format |
| `outdated` | Stale information | API endpoint changed |
| `improvement` | Better approach found | More efficient pattern discovered |

## Categories

| Category | Description |
|----------|-------------|
| `command` | Shell commands, CLI usage |
| `api` | External API interactions |
| `pattern` | Code patterns, architecture |
| `knowledge` | Domain knowledge, facts |
| `capability` | Feature limitations/gaps |

## Example Learnings

### Example 1: Command Error

```markdown
---
date: 2026-01-22T10:30:00Z
type: error
category: command
severity: important
---

# Learning: BSD find uses -depth, not -maxdepth

## Context
Generating a find command to search within 2 directory levels on macOS.

## What Happened
Generated `find . -maxdepth 2 -name "*.py"` which failed with "illegal option -- maxdepth"

## Root Cause
macOS uses BSD find, which uses `-depth` differently than GNU find's `-maxdepth`.

## Resolution
Use `-d` or check platform before generating:
- BSD: `find . -d 2 -name "*.py"` (macOS, FreeBSD)
- GNU: `find . -maxdepth 2 -name "*.py"` (Linux)

## Prevention
Always check platform before generating find commands. Use caro's platform detection.

## Related Files
- src/platform/mod.rs:45

## Tags
find, bsd, gnu, macos, linux, compatibility
```

### Example 2: User Correction

```markdown
---
date: 2026-01-22T11:00:00Z
type: correction
category: knowledge
severity: minor
---

# Learning: User prefers kebab-case for branch names

## Context
Creating a feature branch for the user.

## What Happened
Created branch `feature/addUserAuth` but user corrected: "Actually, use kebab-case"

## Root Cause
Did not know user's naming convention preference.

## Resolution
Changed to `feature/add-user-auth`

## Prevention
Check existing branch names for convention, or ask user for preference.

## Related Files
- .claude/rules/git-workflow.md

## Tags
git, branch, naming, convention
```

## Integration with Handoffs

When creating handoffs, include recent learnings in the "Learnings" section:

```markdown
## Learnings
- BSD vs GNU find differences (see .claude/memory/learnings/2026-01-22_bsd-find.md)
- User prefers kebab-case branch names
```

## Proactive Review

Before major tasks, Claude should:

1. Check if similar work has been done before
2. Review learnings related to the domain
3. Apply past corrections to current approach
4. Note any patterns that have improved over time

## Tips

1. **Capture immediately** - Don't wait; capture learnings as they happen
2. **Be specific** - Include exact error messages, file paths, commands
3. **Link context** - Reference related files and previous learnings
4. **Tag well** - Good tags make learnings findable later
5. **Review often** - Learnings are only valuable if reviewed
6. **Prune stale** - Remove learnings that are no longer relevant

## Storage Location

All learnings stored in: `.claude/memory/learnings/`

- Individual learnings: `YYYY-MM-DD_<topic>.md`
- Index file: `INDEX.md`
- Archive (old/resolved): `archive/`
