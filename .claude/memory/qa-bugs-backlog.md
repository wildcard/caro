# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1277](https://github.com/wildcard/caro/issues/1277) | P1 | embedded | CPU backend stub always returns echo clarify; broken on Linux/x86 (QA-confirmed; other non-MLX platforms untested) | open | 2026-06-29 |
| [#1334](https://github.com/wildcard/caro/issues/1334) | P1 | embedded | `caro ai --once` broken (CPU stub, confirmed v1.5.0) | open | 2026-07-18 |
| [#1366](https://github.com/wildcard/caro/issues/1366) | P2 | docs | CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0 | open | 2026-07-22 |

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
- [**#1044**](https://github.com/wildcard/caro/issues/1044) — CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 — Closed 2026-05-09
