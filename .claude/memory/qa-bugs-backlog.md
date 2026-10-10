# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| pending-2 | P2 | ai | `caro ai --once` hangs silently (no output, no timeout msg) when no model; `doctor` doesn't warn | deferred — gh auth failed 2026-10-10 | 2026-10-10 |
| pending-1 | P2 | docs | CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0 (recurrence of #1044) | deferred — gh auth failed 2026-10-10 | 2026-10-10 |
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | open (recurrence observed 2026-10-10; now 1.4.0 vs 1.5.0) | 2026-05-07 |

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
