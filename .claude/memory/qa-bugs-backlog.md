# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | closed (version drift recurred as #1098, #1520) | 2026-05-07 |
| [#1248](https://github.com/wildcard/caro/issues/1248) | P2 | ai | caro ai silently hangs in non-TTY; no error or --once suggestion | open | pre-2026-10-05 |
| [#1520](https://github.com/wildcard/caro/issues/1520) | P2 | docs | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (recurring drift) | open | pre-2026-10-05 |

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
