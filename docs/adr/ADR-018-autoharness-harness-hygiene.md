# ADR-018: Harness Hygiene Lessons from autoharness (Additive Adoption)

**Status**: Proposed

**Date**: 2026-09-26

**Authors**: Caro maintainers

**Target**: Community

## Context

We evaluated [`tigerless-labs/autoharness`](https://github.com/tigerless-labs/autoharness)
(MIT, v0.5.3, Python 3.11+ with zero third-party dependencies): a Claude Code
plugin that learns skills from real sessions and keeps its skill layer
maintained without a human. How it is built:

- **The model proposes; deterministic code decides.** A reflector agent has no
  Write/Edit/Bash tools. It can only stage *intents* through one MCP tool. A
  deterministic promoter lints every intent against a single spec (safety scan,
  structure, trigger-first description budget, body-length "altitude" cap,
  no placeholders, referenced subfiles exist) and lands it with an atomic
  rename. A rejected intent means zero writes.
- **Contracts are tests.** The agent prompts are pinned by unit tests: least
  privilege tools, hooks routed to files that exist, and prompt clauses added
  after specific failures. Example: *"forbid claiming a verdict it cannot
  observe"* (their PR #99) came with a test.
- **Keep what is used, not what is scored.** Per-skill `use`/`view`/`patch`
  counters come from hook events. New skills get a probation period, each
  layer has a capacity cap, and a retired skill is **archived, never deleted**.
  Their experiment E8 found that models mostly consume skills by `Read`-ing
  `SKILL.md`, not through the `Skill` tool, so both paths are counted.
- **Merge before you add.** "Compare-first" means patching an existing skill
  before creating one. New skills are born as class-level umbrellas, and a
  periodic curator folds near-duplicates together.
- **Anti-silence.** Every run writes an account of what landed and what was
  rejected, surfaced at the next session start. A metrics module answers "is
  this layer doing anything?" and never feeds a decision.
- **Process:** small single-purpose PRs. Commit bodies state the root cause and
  the evidence (named experiments). `remove(...)` commits delete features that
  shipped but were never enabled. Release bumps list the PRs they ship.

Caro's harness is the same kind of artifact at a larger scale: 34 agents, 46
skills, 27 commands and 12 rules, maintained by hand across four or five
parallel sessions. **No automated check runs on any of it.** An audit on
2026-09-26 found:

1. `design-dialogue-protocol.md` was missing from the constitution's
   precedence index, although the constitution says to index new rules "in the
   same PR".
2. `devils-advocate` and `ponytail-reviewer` say "You are read-only" but had no
   `tools:` allowlist, so they inherited Write/Edit. ADR-016's safety argument
   ("read-only neutralizes the conflict") rested on prose alone.
3. **Enforcement was claimed but not wired.** `.claude/memory/consolidated-knowledge-rules.md`
   cited a pre-push hook that "blocks pushes", and `validate-constitution`
   cited a hookify `PostPush` hook. Neither exists: Claude Code has no
   `PostPush` event, `settings.json` does not reference the script, and the
   script ends in `exit 0` anyway.
4. `CLAUDE.md`, which loads into every session, said v1.4.0 and MSRV 1.83.
   `Cargo.toml` says 1.5.0 and 1.85. It also sent every session to a
   nonexistent `.claude/memory/current-tasks.md`.
5. There were 27 dangling file references: 3 in always-loaded files and 24 in
   skills, agents and commands.
6. `.claude/skills/code-parts-syncer/` has no `SKILL.md`, so Claude Code
   silently ignores it. It is in fact support material for `/caro.sync`.
7. `caro-shell-helper` is eight weeks past its own announced removal date. It is
   still the only entry in `.claude-plugin/marketplace.json` and is still
   advertised on the website.
8. **Context budget:** the agent descriptions listed in every session total
   about 47.7K characters. 27 of the 34 exceed 1,024 characters, and about a
   dozen describe overlapping Rust/systems personas.

Each of these is a failure autoharness is designed to prevent: a promise made in
prose with nothing deterministic behind it, and a layer that grows but never
shrinks.

## Decision

**Do not install autoharness into this repository.** Instead, harvest its
mechanisms additively, in phases:

- **Phase 1 (this ADR's PR): deterministic harness lint in CI.**
  `scripts/check-harness.py` (stdlib only) runs in
  `.github/workflows/harness-lint.yml`. The findings above are fixed, and a
  path-scoped rule, `.claude/rules/harness-changes.md`, states the conventions.
- **Phase 2: measure before pruning.** Add hook-based usage counters (`Skill`
  and `Agent` tool calls, `Read` into `.claude/skills/`, slash commands via
  `UserPromptSubmit`) that write to a **durable** sink (routine runs already
  get one: the `automation/routine-status` branch from #1506). Then run one curator-style
  consolidation pass on the overlapping agents, backed by that data.
- **Phase 3 (optional): a personal pilot.** One maintainer runs the autoharness
  plugin at user scope for a time-box and judges it by its own `metrics` output
  (recall rate, landed/rejected funnel). It is not enabled for the project.

| autoharness mechanism | Caro adaptation (Phase 1) |
|---|---|
| Promoter lints every intent against one spec | CI lints the harness against `check-harness.py` |
| Tests pin agent contracts (reflector has no write tools) | Agent `tools:` allowlists are left to #1534's Rust contract test (`tests/agent_tools_contract.rs`), so this linter does not duplicate it |
| Manifest tests: hooks route to files that exist | Every repo script a `settings.json` hook names must exist; the one it runs directly must be executable |
| Structure check: referenced files exist | Dangling references: **error** in always-loaded files (CLAUDE.md, rules), ratcheted **warning** in skills, agents and commands |
| Rejected intents stay visible, never silent | Warnings print on every run. Their count must equal the budget in `scripts/harness-budget.json`, so the PR that fixes one also lowers the budget and the slack can't be spent again |
| Curator folds near-duplicate skills | Skills whose descriptions share 5+ word trigrams get a notice (ported from #1196's `bin/verify-skills`) |
| Promoter rejects malformed specs | #1196's other checks, ported with it: a skill description under 20 chars is a warning, and an empty command `description:` is an error |
| No wall-clock lifecycle ("a closed laptop ages no one out") | Overdue deprecations are **notices** and never fail CI, so the calendar alone cannot turn a PR red |
| `metrics.py` is observation-only | A context-budget notice prints on every run |
| Never promise a path that does not exist (their PRs #99 and #105) | Corrected enforcement claims, plus the rule "document a check as enforced only once it is wired" |
| Compare-first, umbrella skills, archive not delete, ledger (reason + evidence) | Before adding a skill, agent or rule, extend an existing one. Retire things with `git rm` in a commit whose body carries the reason and evidence; git history is the archive |

## Rationale

- **Prose drifts; code does not.** Findings 1 to 4 each break a rule written
  down in this repo. The rules were fine. Nothing checked them.
- **Errors only where drift costs every session.** Registry breakage and drift
  in always-loaded files fail CI. Legacy drift in on-demand files is ratcheted,
  so this PR does not have to fix 24 judgment calls to go green, and no new
  ones can slip in.
- **Least privilege belongs in frontmatter, not prose.** autoharness learned
  this from a live end-to-end run; we found it in an audit.
- **Measure before cutting.** Consolidating the Rust personas is the largest
  context win available. It stays in Phase 2 because "no evidence of use is not
  evidence of no use" (autoharness MNG). Guessing would also break ADR-016's
  additive steer.

## Consequences

### Benefits

- Harness drift is caught at PR time, including drift caused by renames
  outside `.claude/`.
- Files loaded into every session stay truthful.
- The "read-only" claims of reviewer agents are enforced.
- The context budget and overdue deprecations are visible on every run.

### Trade-offs

- One more CI job (about 10 seconds, stdlib Python and no dependencies).
- A PR that fixes a warning must also lower `max_warnings` in
  `scripts/harness-budget.json` (CI says so and fails until it does).
- The checks are regex heuristics. A support directory occasionally needs an
  entry in `SUPPORT_DIRS`, with a stated reason.

### Risks

- **Risk**: a false positive blocks an unrelated PR. **Mitigation**: errors are
  limited to registry integrity and always-loaded files. Fenced code, URLs,
  anchors, placeholders (`YYYY`, `vX`, `<name>`) and bare directory shorthand
  are ignored, and fixture-based unit tests pin the behavior.
- **Risk**: someone raises the budget to get green. **Mitigation**: the workflow
  comment says never raise it, and the number sits in a reviewed diff.

## Alternatives Considered

### Alternative 1: Install the autoharness plugin for the project

- Pros: it maintains its own learned skills automatically.
- Cons:
  - It manages **only skills it authored**, so it would not touch the 107
    hand-written harness files where the sprawl is.
  - Its background processes write into the git-tracked `.claude/skills/`,
    which clashes with the parallel-session, branch-per-change workflow.
  - Reflection runs `claude -p --dangerously-skip-permissions` child sessions.
    That is acceptable for a personal pilot, but not a default for every
    contributor to a safety-critical repo (compare ADR-016, Alternative 1).
  - Its calibration values are still labelled placeholders.
- **Rejected** for project scope. Phase 3 keeps a personal pilot open.

### Alternative 2: Rely on the existing prose rules

- **Rejected.** This audit is the evidence that prose alone drifts.

### Alternative 3: Fix the drift once, without CI

- **Rejected.** The same drift would return. autoharness's central point is that
  the gate is code.

### Alternative 4: Measure usage by parsing session transcripts

- **Rejected.** Claude Code documents its transcript JSONL as internal and
  version-unstable. Hook payloads are documented, so Phase 2 counts through
  hooks, as autoharness does.

## Implementation Notes

- `+ scripts/check-harness.py`: the linter. Levels are error, warn
  (ratcheted) and notice.
- `+ scripts/tests/test_check_harness.py`: `unittest` cases for every check,
  each built on a throwaway fixture tree.
- `+ .github/workflows/harness-lint.yml`: runs the tests, then the linter.
- `+ scripts/harness-budget.json`: the warning budget (16). Keeping it out of
  the workflow means lowering it never edits a Tier-1 path.
- The read-only reviewers' `tools:` lines came from #1470 (merged); #1534
  adds allowlists for every other agent.
- `~ .claude/rules/constitution.md`: index `design-dialogue-protocol.md` and
  `harness-changes.md`, appended to Tier 3 so no existing numbers shift.
- `+ .claude/rules/harness-changes.md`: the conventions (search before adding,
  least privilege, no unwired promises, retire with evidence). Its `paths:`
  frontmatter loads it only when harness files are touched, and the linter
  treats such rules as on-demand.
- `~ CLAUDE.md`, rules, the `validate-constitution` skill and the
  `constitution-validator` agent,
  `.claude/memory/consolidated-knowledge-rules.md`, six commands and one skill: the drift
  fixes listed in Context.

**Phase 2 constraint.** Local logs (like `.claude/notifications.log`) do not
survive ephemeral cloud sessions. Pick a durable sink before trusting any
"unused" verdict, or "unused" will only mean "used in the cloud".

**Left for follow-up** (each needs a decision, not a mechanical fix):

- The 16 remaining dangling references.
- Retiring `caro-shell-helper`, which requires updating the website and the
  marketplace entry first.
- `.claude-plugin/marketplace.json`, which lacks the `owner` and `plugins`
  fields the plugin marketplace schema requires.
- The duplicate `ADR-004` number.
- The failing `scripts/tests/test_pattern_gap_analyzer.py` suite, which no CI
  job runs (the wider gap, 23 of 46 test targets, is #1537).

## Success Metrics

- Harness Lint stays green on `main`, and the warning budget only decreases
  (16 → 0).
- After merge, no orphan rule or loader-invisible skill directory lands on
  `main` (read-only agents with write tools are #1534's contract test).
- Phase 2: at least 30 days of durable usage data before any consolidation PR.
  The consolidation should cut the per-session agent-description budget
  (about 47.7K characters) roughly in half.

## References

- [`tigerless-labs/autoharness`](https://github.com/tigerless-labs/autoharness).
  See especially `src/autoharness/lib/validate.py`, `lib/format_spec.md`,
  `hook/promoter.py`, `lib/lifecycle.py`, `lib/metrics.py`,
  `tests/test_reflector_agent.py` and `tests/test_plugin_manifest.py`.
- [ADR-016](./ADR-016-ponytail-pragmatic-reviewer.md): precedent for adopting an
  external project's idea additively.
- [`constitution.md`](../../.claude/rules/constitution.md),
  [`harness-changes.md`](../../.claude/rules/harness-changes.md),
  [`good-boy-scout.md`](../../.claude/rules/good-boy-scout.md).

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-09-26 | Caro maintainers | Initial draft, Proposed (as ADR-017) |
| 2026-10-09 | Caro maintainers | Renumbered to ADR-018 (#1459 took 017). Least-privilege check handed to #1534; #1196's collision, minimum-description and command-description checks ported; budget moved to `scripts/harness-budget.json`; conventions moved to a path-scoped rule |
