# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1375](https://github.com/wildcard/caro/issues/1375) | P1 | ai | `caro ai --once` always returns `echo 'Please clarify your request'` on Linux x86_64 (CPU backend keyword collision with system-prompt safety examples) | open | 2026-07-26 |
| [#1422](https://github.com/wildcard/caro/issues/1422) | P2 | cli | `caro ai --once` rejects global `-p`/`--prompt` flag — inconsistent with top-level usage | open | 2026-07-26 |
| [#1421](https://github.com/wildcard/caro/issues/1421) | P2 | ai | `caro ai --once` returns echo placeholder instead of informative error when no backend available | open | 2026-07-26 |
| [#1369](https://github.com/wildcard/caro/issues/1369) | P2 | cli | `caro ai --once` hint for --execute omits required global flag placement | open | 2026-07-25 |
| [#1442](https://github.com/wildcard/caro/issues/1442) | P2 | docs | CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0 — recurrence of #1044 | open | 2026-09-07 |

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

- [#1044](https://github.com/wildcard/caro/issues/1044) — `docs: CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0` — Closed 2026-05-09 (partial fix: updated to 1.3.0 but CLAUDE.md not added to release-version-alignment checklist; drift recurred at v1.5.0, see #1442)
- **BUG-001**: Search highlight double-counting with global regex — Fixed 2026-01-02
