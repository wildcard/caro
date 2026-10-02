# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | closed | 2026-05-07 |
| [#1499](https://github.com/wildcard/caro/issues/1499) | P1 | ai | `caro ai --once` hangs indefinitely in non-TTY environments when trailing args provided | open | 2026-10-02 |
| [#1500](https://github.com/wildcard/caro/issues/1500) | P2 | embedded | CPU backend stub returns misleading "What exactly should be deleted?" for all queries | open | 2026-10-02 |
| [#1501](https://github.com/wildcard/caro/issues/1501) | P2 | docs | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 — drift recurred after #1044 | open | 2026-10-02 |

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
