# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | open | 2026-05-07 |
| [#1361](https://github.com/wildcard/caro/issues/1361) | P1 | ai | `caro ai --once` returns `echo 'Please clarify your request'` with confidence=0.85 and exit 0 when model unavailable; CPU stub `prompt.contains("rm")` fires on system-prompt words | open | 2026-07-21 |
| [#1362](https://github.com/wildcard/caro/issues/1362) | P2 | cli | Static matcher Pattern 43 regex `^(list|show).*(all)?.*(files?)\s*$` rejects "list files in current directory"; `\s*$` over-anchored | open | 2026-07-21 |

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
