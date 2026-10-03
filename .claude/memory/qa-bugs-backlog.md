# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1221](https://github.com/wildcard/caro/issues/1221) | P1 | cli | --backend-info lists invalid backends (static, claude) and omits valid ones (exo, mesh, ai-horde, hybrid) | open | 2026-06-13 |
| [#1222](https://github.com/wildcard/caro/issues/1222) | P1 | cli | e2e_safety_level_configuration test fails — cargo fallback missing --bin caro with multiple binaries | open | 2026-06-13 |
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | **closed** 2026-05-09 | 2026-05-07 |

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
