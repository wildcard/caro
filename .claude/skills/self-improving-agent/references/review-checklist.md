# Pre-Task Learning Review Checklist

Use this checklist before starting significant work.

## Quick Review Commands

### List Recent Learnings (Last 7 Days)
```bash
find .claude/memory/learnings -name "*.md" -mtime -7 -type f | grep -v INDEX | grep -v archive
```

### Search Learnings by Topic
```bash
grep -ril "<topic>" .claude/memory/learnings/*.md 2>/dev/null | grep -v INDEX
```

### Count Learnings by Severity
```bash
grep -l "severity: critical" .claude/memory/learnings/*.md 2>/dev/null | wc -l
```

### View Critical Learnings
```bash
grep -l "severity: critical" .claude/memory/learnings/*.md 2>/dev/null
```

## Pre-Task Checklist

Before starting a major task:

- [ ] **Check related learnings**: Search for keywords related to the task
- [ ] **Review critical learnings**: Skim any critical-severity items
- [ ] **Check recent corrections**: Look at learnings from last 7 days
- [ ] **Note applicable patterns**: Identify any learnings to apply

## Post-Task Capture

After completing a task with issues:

- [ ] **Capture errors**: Document any unexpected failures
- [ ] **Record corrections**: Note any user corrections received
- [ ] **Update index**: Add new learnings to INDEX.md
- [ ] **Tag appropriately**: Ensure good searchability

## Monthly Maintenance

- [ ] Archive learnings older than 90 days if resolved
- [ ] Review and consolidate similar learnings
- [ ] Update INDEX.md with accurate categorization
- [ ] Remove learnings that are now in CLAUDE.md
