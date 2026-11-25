# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | open | 2026-05-07 |
| [#1786](https://github.com/wildcard/caro/issues/1786) | P1 | docs | /docs/installation install commands are dead (caro.sh/install.sh 404, `cargo install cmdai`, caro-darwin-arm64 404) | open | 2026-11-25 |
| [#1787](https://github.com/wildcard/caro/issues/1787) | P1 | install | install.sh writes `caro --completion` to the rc; errors on every new shell | open | 2026-11-25 |
| [#1788](https://github.com/wildcard/caro/issues/1788) | P1 | install | Installer claims Strict via `CARO_SAFETY_LEVEL`; caro ignores it and runs Moderate | open | 2026-11-25 |

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
