# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1387](https://github.com/wildcard/caro/issues/1387) | P1 | ai | `caro ai --once` bypasses static matcher; returns fallback for every prompt when LLM unavailable | open | 2026-08-01 |
| [#1388](https://github.com/wildcard/caro/issues/1388) | P2 | docs | CLAUDE.md version shows 1.4.0 instead of 1.5.0 (recurrence of #1044) | open | 2026-08-01 |
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner showed 1.1.0 instead of 1.3.0 | closed | 2026-05-07 |

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
- **#1044**: CLAUDE.md version drift (1.1.0→1.3.0) — Fixed 2026-05-09; recurred as #1388
