# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1375](https://github.com/wildcard/caro/issues/1375) | P1 | ai | `caro ai --once` CpuBackend always returns placeholder on Linux x86_64 | open | 2026-07-26 (from #1373 branch) |
| [#1442](https://github.com/wildcard/caro/issues/1442) | P2 | docs | CLAUDE.md version 1.4.0 vs 1.5.0; MSRV 1.83 vs 1.85 (recurrence #1044) | open | 2026-09-07 (from #1443 branch) |
| [#1372](https://github.com/wildcard/caro/issues/1372) | P2 | docs | CLAUDE.md version 1.4.0 vs 1.5.0 (recurrence #1044; duplicate of #1442) | open | 2026-07-25 (from #1373 branch) |

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
- [#1044](https://github.com/wildcard/caro/issues/1044) — CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 — Closed 2026-05-07 (recurrence tracked in #1372, #1442)
