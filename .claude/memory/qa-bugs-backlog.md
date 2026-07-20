# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | **closed** | 2026-05-07 |
| [#884](https://github.com/wildcard/caro/issues/884) | P2 | i18n | EN landing headline refreshed but DE/JA still on old copy; EN title/H1 inconsistent | open | 2026-04-26 |
| [#1098](https://github.com/wildcard/caro/issues/1098) | P2 | docs | CLAUDE.md version and MSRV stale after v1.4.0 release (regression of #1044); MSRV still shows 1.83 | open | 2026-05-15 |
| [#1099](https://github.com/wildcard/caro/issues/1099) | P2 | cli | `caro test --verbose` output identical to non-verbose — per-test details not shown | open | 2026-05-15 |
| [#1107](https://github.com/wildcard/caro/issues/1107) | P1 | cli | `--backend openrouter` rejected as 'Unknown backend' despite PR #1097 adding backend | open | 2026-05-16 |
| [#1162](https://github.com/wildcard/caro/issues/1162) | P2 | cli | `cargo test safety` fails — evaluation harness rejects standard cargo test filter | open | 2026-05-21 |
| [#1163](https://github.com/wildcard/caro/issues/1163) | P1 | cli | Global flags before subcommands silently bypass subcommand routing | open | 2026-05-22 |
| [#1164](https://github.com/wildcard/caro/issues/1164) | P2 | cli | `cargo run` fails without `--bin caro` (multi-binary project missing default-run) | open | 2026-05-22 |
| [#1165](https://github.com/wildcard/caro/issues/1165) | P1 | safety | User allowlist cannot override `rm -rf /tmp/...` despite explicit allowlist pattern (regression from #1110) | open | 2026-05-23 |
| [#1169](https://github.com/wildcard/caro/issues/1169) | P1 | safety | `test_allowlist_functionality` broken by PR #1110 Critical pre-scan guard | open | 2026-05-25 |
| [#1170](https://github.com/wildcard/caro/issues/1170) | P2 | cli | `cargo test safety` fails — evaluation binary rejects positional filter arg | open | 2026-05-25 |

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
