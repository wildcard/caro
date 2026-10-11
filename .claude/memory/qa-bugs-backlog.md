# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | **closed** (fixed v1.5.0) | 2026-05-07 |
| [#1272](https://github.com/wildcard/caro/issues/1272) | P1 | ai | `caro ai --once` hangs silently with zero output when no backend is available | open | 2026-06-26 |
| [#1408](https://github.com/wildcard/caro/issues/1408) | P1 | ai | `caro ai --once` provides zero user feedback during model initialization | open | 2026-08-15 |
| [#1603](https://github.com/wildcard/caro/issues/1603) | P2 | docs/ops | QA memory files stuck at 2026-05-07 bootstrap — 25+ update PRs unmerged | open | 2026-10-11 |

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
- **#1044**: CLAUDE.md version banner drift — Fixed 2026-05-09 (version field now references Cargo.toml dynamically)
