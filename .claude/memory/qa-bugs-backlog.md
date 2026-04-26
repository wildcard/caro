# QA Bugs Backlog

**Last updated**: 2026-04-26 by caro-qa-agent

---

This file used to hold a `BUG-XXX` template. As of 2026-04-26 the project standardizes on **GitHub Issues** as the single source of truth for QA-filed defects (beads is reserved for epics + dependency graphs). The template has been retired.

## Where bugs live now

```bash
# Open bugs filed by the QA agent
gh issue list --label qa --state open

# All open bugs (any author)
gh issue list --label bug --state open

# Bugs that block the next release
gh issue list --label release-gap --state open

# Recent regressions
gh issue list --label regression --state open
```

## Watch list (issues I filed but want to monitor)

QA-filed issues that haven't been closed yet, with the rotation slot they came from. When an issue closes, remove the row.

| GH issue | Filed | Slot | Surface | Severity | Status |
|---|---|---|---|---|---|
| [#884](https://github.com/wildcard/caro/issues/884) | 2026-04-26 | C | Website caro.sh landing — EN/DE/JA copy drift + EN title/H1 mismatch | P2 | open |

## Resolved (historical)

- **BUG-001** (2026-01-02): Search highlight double-counting with global regex — fixed pre-GH-issue-flow. Kept here for historical continuity.

---

## Cross-references

- Coverage matrix (drives random rotation): `.claude/memory/qa-coverage-matrix.md`
- Session log (chronicle of every pass): `.claude/memory/qa-session-log.md`
- Known flakes (intermittent — not yet regressions): `.claude/memory/qa-known-flakes.md`
- QA persona / role: `~/.claude/projects/-Users-kobik-private-workspace-caro/memory/qa_agent_role.md`
