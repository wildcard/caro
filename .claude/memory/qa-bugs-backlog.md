# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1277](https://github.com/wildcard/caro/issues/1277) | P1 | embedded | CPU placeholder misfires on all queries (system prompt contains "rm", triggers danger branch) | open | — |
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version stale (1.4.0 vs 1.5.0); 5 additional tracking issues: #1319, #1335, #1359, #1366, #1368 | open | 2026-05-07 |

---

## Template

```markdown
## BUG-XXX: [Short title]

**Reported:** YYYY-MM-DD
**Severity:** Critical / High / Medium / Low
**Component:** [file path or feature area]
**Reproducible:** Always / Sometimes / Rarely

### Steps to Reproduce
1.
2.
3.

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happens]

### Screenshots/Logs
[If applicable]

### Environment
- OS:
- caro version:

### Notes
[Additional context]
```

---

## Resolved (closed issues)

- **BUG-001**: Search highlight double-counting with global regex — Fixed 2026-01-02
