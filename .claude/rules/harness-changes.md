---
paths:
  - ".claude/**"
  - "CLAUDE.md"
---

# Harness Changes (`CLAUDE.md`, `.claude/`)

Path-scoped: this rule loads when a session touches the harness itself.

The agent harness is code: CI lints it with `scripts/check-harness.py`
(Harness Lint; rationale in `docs/adr/ADR-018-autoharness-harness-hygiene.md`).
Its warning budget lives in `scripts/harness-budget.json` and must equal the
warning count, so the PR that fixes a warning also lowers the budget; nothing
raises it.

- **Search, then compare.** Before adding a skill, agent, command, rule or
  routine, search open PRs for the same work, then extend the closest existing
  file. Add a file only when nothing covers that class of work.
- **Least privilege.** An agent that says it is read-only declares a `tools:`
  allowlist without Write, Edit, or Bash (the shell can write too).
- **No unwired promises.** Call a check "enforced" (hook, CI job, "blocks
  pushes") only once it is wired. Until then, document it as manual.
- **Retire, don't let rot.** Removal is a `git rm` whose commit body states
  the reason and the evidence (git history is the archive). A deprecation
  notice names a removal date, and the removal lands by then.
