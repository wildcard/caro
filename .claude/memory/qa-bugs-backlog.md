# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1269](https://github.com/wildcard/caro/issues/1269) | P1 | embedded | CPU stub `rm`-in-system-prompt: all non-static-matched queries return clarification stub | open | 2026-06-26 |
| [#1098](https://github.com/wildcard/caro/issues/1098) | P2 | docs | CLAUDE.md version drift (shows 1.4.0, Cargo.toml is 1.5.0) | open | 2026-05-07 |

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
- [#1044](https://github.com/wildcard/caro/issues/1044) — CLAUDE.md version banner drift — Closed as duplicate of #1098 (2026-10-09 confirmed closed)
