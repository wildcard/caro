# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1494](https://github.com/wildcard/caro/issues/1494) | P1 | ai | `caro ai --once` CPU backend always returns deletion-clarification (system prompt contains "rm") | open | 2026-10-01 |
| [#1495](https://github.com/wildcard/caro/issues/1495) | P2 | docs | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (regression of #1044) | open | 2026-10-01 |

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
- [#1044](https://github.com/wildcard/caro/issues/1044) — CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 — Closed 2026-05-09 (recurred as #1495)
