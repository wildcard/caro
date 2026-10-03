# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | closed | 2026-05-07 |
| [#1292](https://github.com/wildcard/caro/issues/1292) | P1 | cli | Telemetry consent screen advertises invalid config key `telemetry.enabled` | open | 2026-07-11 |
| [#1293](https://github.com/wildcard/caro/issues/1293) | P1 | cli | `--backend-info` shows wrong backends (static/claude shown; mesh/ai-horde/hybrid missing) | open | 2026-07-11 |
| [#1294](https://github.com/wildcard/caro/issues/1294) | P2 | cli | `--backend` help text missing mesh, ai-horde, hybrid (added PR #1209) | open | 2026-07-11 |
| [#1295](https://github.com/wildcard/caro/issues/1295) | P2 | ai | `caro ai --once` hangs silently with no feedback when model unavailable | open | 2026-07-11 |

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
