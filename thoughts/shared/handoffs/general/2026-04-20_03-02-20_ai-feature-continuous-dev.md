---
date: 2026-04-20T03:02:20+0000
session_name: general
researcher: Claude (Opus 4.7)
git_commit: 09187dc72e6970269d695d69547dc449325c67e4
branch: claude/add-ai-feature-2Kbzx
repository: wildcard/caro
topic: "AI Feature (Atuin-AI-style) — continuous development + automation routines"
tags: [implementation, ai, shell-integration, automation, scheduled-tasks, roadmap]
status: complete
last_updated: 2026-04-20
last_updated_by: Claude (Opus 4.7)
type: implementation_strategy
root_span_id:
turn_span_id:
---

# Handoff: Caro AI feature — continuous dev + 2 automation routines

## Task(s)

**Completed (this session)**
- Studied Atuin AI (https://docs.atuin.sh/cli/ai/) and shipped a Caro-equivalent
  MVP: `caro ai --once` + `caro shell-init <shell>` with a `?` keybinding,
  privacy-first `[ai]` TOML config, JSON-file session store, every generation
  routed through the existing 52-pattern `SafetyValidator`.
- PR #861 opened on branch `claude/add-ai-feature-2Kbzx` with 20 new tests, all
  green; clippy + fmt clean.
- Plan file written to `/root/.claude/plans/study-this-feature-https-docs-atuin-sh-c-whimsical-hare.md`
  documenting the full 6-phase roadmap.

**Handoff scope (next agent)**
1. Continuous development of the AI feature toward GA — Phases 4-6 of the plan
   (interactive REPL, telemetry hooks, docs).
2. Stand up two recurring automation routines (see `## Routines` below):
   - **Backlog grooming** — convert pushed branches / open PRs / GH issues into
     trackable backlog tasks aligned with the roadmap.
   - **Release cycle integration** — review-merge-test-release loop that pulls
     ready features into the active milestone.

## Critical References

- `/root/.claude/plans/study-this-feature-https-docs-atuin-sh-c-whimsical-hare.md`
  — full implementation plan, recommended approach, phased breakdown, verification.
- `ROADMAP.md` (top of repo) — milestones; AI feature targets v1.2.0 (Mar 2026)
  alongside Website & Docs, with stretch into v2.0.0 (Karo Distributed Intelligence).
- `.claude/rules/git-workflow.md` — branch hygiene; never commit on main.
- `.claude/rules/adr-numbering.md` — ADR sequencing (next available is `ADR-004`+).

## Recent changes

- `src/models/mod.rs:627-720` — added `AiConfig`, `AiCapabilities`, `AiOpening`
  with privacy-first defaults; wired into `UserConfiguration` + builder + validator.
- `src/ai/mod.rs`, `src/ai/session.rs`, `src/ai/store.rs`, `src/ai/privacy.rs`,
  `src/ai/runner.rs`, `src/ai/shell_init.rs` — new module (~1100 LOC + tests).
- `src/cli/mod.rs:42-47` — exposed `CliApp::backend_arc()` so the runner can
  reuse the configured backend without re-detecting.
- `src/main.rs:455-499` — added `Commands::Ai` and `Commands::ShellInit`.
- `src/main.rs:847-1000` — `handle_shell_init` + `run_ai_once` dispatchers.
- `src/setup/mod.rs:183-194` — initialize `ai: AiConfig::default()` in setup wizard.
- `src/lib.rs:31` — `pub mod ai;`.

## Learnings

- **Existing `Commands::Integration`** (`src/main.rs:751`) already emits a
  shell wrapper; we deliberately added a separate `ShellInit` to keep the AI
  surface composable. The next agent may want to *deprecate* `Integration`
  once `shell-init` is broadly adopted, but DO NOT delete it — there are users
  with `eval "$(caro init bash)"` in their dotfiles.
- **`ExecutionContext`** real shape: `{os, arch, os_version, distribution,
  cwd: PathBuf, shell, user, available_commands}` — `cwd` is `PathBuf` not
  `String`. Initial draft of `privacy.rs` used wrong field names; fixed.
- **`BackendType`** has `Mock | Embedded | Ollama | VLlm | Mlx` — *no* `Static`
  variant despite a `StaticMatcher` backend existing. The static matcher reports
  itself as `Mock` for `BackendInfo`. Keep this in mind for any telemetry.
- **`CARO_LAST_COMMAND` env var** is the integration point for shell hooks to
  pass the prior command into the AI session — the runner reads it only when
  `ai.opening.send_last_command = true`. Document this when teaching shell hooks.
- **`src/models` is in `.gitignore`** (line 75: `models/`) but `src/models/mod.rs`
  is already tracked. Use `git add -u src/models/mod.rs` (not `git add src/models/`)
  or pass `-f`. Otherwise `git add` silently drops it.
- **JSON session store vs SQLite** — chose JSON because sessions are small and
  ordered; the trade-off is persisted in the plan. If session count or write
  contention grows, migrate to `rusqlite` (already in dep tree transitively).

## Post-Mortem

### What Worked

- **Plan-mode workflow with parallel Explore + Plan agents** — gave a precise
  map of reusable infrastructure (`AgentLoop`, `KnowledgeIndex`, `SafetyValidator`)
  before any code was written. Avoided rebuilding what already existed.
- **Privacy-first defaults** — every `[ai.opening]` / `[ai.capabilities]` toggle
  defaults to `false`, with one centralized `privacy::build_context` gate. Makes
  audit trivial: golden-string tests prove substrings are/aren't in the prompt.
- **One-shot (`--once`) + shell widget** as MVP — sidesteps the full TUI/REPL
  complexity (Atuin's "Hex" PTY proxy) while still delivering the `?` UX.
- **Reuse `CliApp::with_overrides`** to construct the backend — the AI runner
  inherits `--backend`, `CARO_BACKEND`, and config-file precedence for free.

### What Failed

- **Tried**: passing `prompt: Vec<String>` directly out of the match arm
  → Failed because partial move conflicts with `&cli` borrow → Fixed by
  destructuring with `ref prompt` and cloning before the call (`src/main.rs:2113`).
- **Tried**: `git add src/models/` to stage the modified `mod.rs`
  → Failed because `.gitignore` line 75 ignores the `models/` path → Fixed by
  staging the file directly with `git add -u src/models/mod.rs`.
- **Initial `ExecutionContext` mock used hypothetical field names** (`username`,
  `hostname`, `capability_profile`) — caught by compile failure in the privacy
  test. Real fields documented in **Learnings**.

### Key Decisions

- **Decision**: JSON file at `$XDG_DATA_HOME/caro/ai_sessions.json` for sessions.
  - Alternatives considered: rusqlite SQLite DB; reuse `KnowledgeIndex` (LanceDB).
  - Reason: sessions are small, ordered, ephemeral; flat file is debuggable by
    humans; no new SQL surface in the hot CLI path; `KnowledgeIndex` conflates
    "learned successes" with "conversation state" and forces every AI user to
    pay the vector-DB cost.
- **Decision**: `ai.enabled = true` by default (vs Atuin's `false`).
  - Alternatives considered: opt-in like Atuin.
  - Reason: Caro's default backend is local (embedded MLX/CPU), so the privacy
    risk Atuin guards against (cloud transit) doesn't apply to the default
    install. The `[ai.opening]` / `[ai.capabilities]` sub-toggles still default
    `false`, preserving privacy-first when the user moves to a remote backend.
- **Decision**: ship MVP without the interactive REPL.
  - Alternatives considered: full Enter/Tab/`f`/`q` TUI on day one.
  - Reason: scope cost is high; the shell widget + `--once` already delivers
    90% of Atuin's UX from the user's perspective; REPL becomes Phase 4 and
    can iterate against real telemetry.

## Artifacts

**Plan & spec**
- `/root/.claude/plans/study-this-feature-https-docs-atuin-sh-c-whimsical-hare.md` — full plan
- PR: https://github.com/wildcard/caro/pull/861

**Source (this branch)**
- `src/ai/mod.rs:1-19` — module surface + re-exports
- `src/ai/session.rs:1-180` — `AiSession`, `Turn`, `Role`, TTL helpers, history rendering
- `src/ai/store.rs:1-185` — JSON file store, `default_store_path`, atomic flush, TTL resume
- `src/ai/privacy.rs:1-200` — `build_context`, `may_leak_context_offhost`, golden tests
- `src/ai/runner.rs:1-285` — `run_once`, `AiInvocation`, `AiOutcome`, `SessionMode`, `build_validator`
- `src/ai/shell_init.rs:1-200` — bash/zsh/fish snippet renderers + `?` keybinding
- `src/main.rs:455-499` — `Commands::Ai`, `Commands::ShellInit` enum entries
- `src/main.rs:847-1000` — `handle_shell_init`, `run_ai_once` dispatchers
- `src/models/mod.rs:627-720` — `AiConfig` and friends
- `src/cli/mod.rs:42-47` — `CliApp::backend_arc()` accessor

**Tests touching the new surface**
- `src/ai/privacy.rs:121-198` (5 tests)
- `src/ai/session.rs:140-185` (4 tests)
- `src/ai/store.rs:130-180` (3 tests)
- `src/ai/shell_init.rs:170-218` (6 tests)
- `src/ai/runner.rs:200-285` (2 tests)

## Action Items & Next Steps

### Phase 4 — Interactive REPL (`src/ai/repl.rs`)

Reuse `crossterm` (consider adding) or `dialoguer` (already a dep) to drive an
in-process loop:
- `Enter` → execute through `caro::execution::Executor`; HIGH/CRITICAL risk
  requires a second `Enter` within 2 s (mirrors Atuin).
- `Tab` → emit command to stdout (current `--once` behavior).
- `f` → prompt for follow-up; reuse `runner::run_once` with the same session id.
- `q` / Ctrl-C → exit cleanly, persist final session state.
- Drive with a `KeySource` trait so it's unit-testable without a TTY.

### Phase 5 — Knowledge integration

Behind `feature = "knowledge"`, when `ai.capabilities.enable_history_search = true`:
- In `privacy::build_context`, call `KnowledgeIndex::find_similar(prompt, 3)`
  and prepend results.
- After a successful Enter execution in the REPL, call
  `KnowledgeIndex::record_success(prompt, command, ...)`.
- Add a `caro ai history index` subcommand that one-shot indexes shell history
  files (`$HISTFILE` / `$ZSH_HISTORY` / fish history JSON) — gated on user confirm.

### Phase 6 — Docs & telemetry

- New file: `docs/ai.md` covering the `[ai]` config, the `?` keybinding,
  privacy model, and example `eval "$(caro shell-init bash)"`.
- README.md: add a `caro ai` section under "Features".
- Add opt-in telemetry events in `src/telemetry/events.rs`:
  `ai.session.created`, `ai.turn.completed{risk, confidence_bucket}`,
  `ai.command.blocked{risk_level}` — *event names only, no command text*.

### Roadmap alignment

- **v1.2.0 (Mar 2026)** — AI feature lands here alongside docs site + website.
  Open `docs/adr/ADR-004-ai-conversational-command-generation.md` summarising
  the trade-offs above (use ADR template; bump number based on what's already
  merged at the time per `.claude/rules/adr-numbering.md`).
- **v2.0.0 (post-Karo)** — multi-machine session sync; PTY-proxy (Hex-style)
  popup UI; LLM-side danger scoring (the local 52 patterns are necessary but
  not sufficient).

## Routines (scheduled tasks for next agent to set up)

These two routines run on a recurring schedule. Use the existing
`/loop` skill (`.claude/skills/loop/`) and bind each to a slash command so a
human can trigger them manually too.

### Routine 1 — Backlog grooming (every 6 h)

**Goal**: keep `.claude/memory/current-tasks.md` and the GitHub Project board
synchronized with reality. Auto-create issues for branches & PRs that lack one.

**Trigger**: `/loop 6h /caro.backlog-groom`

**Process** (define in new `.claude/commands/caro.backlog-groom.md`):
1. **Pull state**:
   - `git ls-remote --heads origin` → list of pushed branches.
   - `mcp__github__list_pull_requests` (state=all, last 30 days) → PRs.
   - `mcp__github__list_issues` (state=open) → issues.
   - `git log main..@{u} --since='7 days ago' --pretty=format:'%h %s %an'`
     → recent merged work attributable to Claude Code.
2. **Reconcile**:
   - For each pushed branch with no open PR & no recent activity > 14 d:
     create a `cleanup/stale-branch` issue listing the branch and last commit.
   - For each open PR with no linked issue: create an issue mirroring the PR
     summary and link it via `Closes #<pr>` body line.
   - For each merged PR in the window without a CHANGELOG entry: open a doc
     issue tagged `docs/changelog`.
3. **Rank against roadmap** (`ROADMAP.md`):
   - Tag each new/updated issue with the milestone it aligns to (`v1.2.0`,
     `v2.0.0`, etc.). Use `mcp__github__issue_write`.
   - Anything off-roadmap → label `needs-roadmap-decision` for human triage.
4. **Update** `.claude/memory/current-tasks.md` with a delta block:
   `### Backlog grooming YYYY-MM-DD HH:MM\n- +N new issues\n- ~M issues retitled\n- !K stale branches flagged`.
5. **Idempotency guard**: keep a state file at
   `.claude/state/backlog-grooming.json` recording the last-seen sha & PR
   number per repo so re-runs don't duplicate work.

**Stop conditions**: dry-run if `> 20` actions queued — escalate to human via
`mcp__github__add_issue_comment` on a tracking issue.

### Routine 2 — Release cycle integration (every 12 h)

**Goal**: shepherd ready features through review → merge → integration test →
release-branch promotion, in line with the milestone calendar.

**Trigger**: `/loop 12h /caro.release-integrate`

**Process** (define in new `.claude/commands/caro.release-integrate.md`):
1. **Identify candidates**:
   - PRs with: ≥1 approval, all required checks green, no `do-not-merge` /
     `wip` labels, milestone matches the active release in `ROADMAP.md`.
   - Use `/pr-management-loop` (already exists) for staleness/CI rebasing
     side-effects, then filter the resulting list.
2. **Auto-merge** the candidate set in PR-number order (per ADR rule):
   - `mcp__github__merge_pull_request` with `merge_method: "squash"`.
   - On conflict: rebase via `mcp__github__update_pull_request_branch`; if it
     still fails, comment with the conflict file list and skip.
3. **Run integration tests** on `main` after each merge:
   - Worktree: `git worktree add .worktrees/release-integrate origin/main`
   - `cargo test --workspace --all-features` (~5 min budget; abort + revert
     last merge if it fails — never leave main red).
   - On green: prune the worktree.
4. **Promote to release branch**:
   - If active milestone is in its final week (per ROADMAP.md gantt), open or
     fast-forward a `release/<version>` branch from `main` and tag a
     `<version>-rc.N` release candidate.
   - Trigger `/caro.release.prepare` for the human to take over the
     ship-or-rollback decision.
5. **Report**:
   - Post a summary comment on the milestone issue: PRs merged this run,
     integration test result, RC tag (if produced).
   - Update `CHANGELOG.md` Unreleased section in a follow-up PR.

**Safety rails**:
- Never merge a PR authored by the routine itself (avoid loops).
- Never push to `main` directly — only merge PRs.
- Honor `merge-window-closed` repo label (humans can pause the routine).
- Never publish to crates.io / GitHub Releases — that stays human-driven via
  `/caro.release.publish`.

### Setup commands for the next agent

```bash
# 1. Read the existing loop skill to confirm syntax
cat .claude/skills/loop/SKILL.md

# 2. Create the two new commands
mkdir -p .claude/commands
$EDITOR .claude/commands/caro.backlog-groom.md       # use Routine 1 above as the body
$EDITOR .claude/commands/caro.release-integrate.md   # use Routine 2 above as the body

# 3. Wire up the schedules (in a new shell or tmux pane)
/loop 6h /caro.backlog-groom
/loop 12h /caro.release-integrate

# 4. State directory
mkdir -p .claude/state
echo '{}' > .claude/state/backlog-grooming.json
echo '{}' > .claude/state/release-integrate.json
git add .claude/commands/caro.backlog-groom.md .claude/commands/caro.release-integrate.md
```

Both commands should follow the existing patterns in
`.claude/commands/pr-management-loop.md` and `.claude/commands/idea-sourcing-loop.md`.

## Other Notes

- Existing automation skills available to compose with:
  - `pr-management-loop` (`.claude/skills/pr-management-loop/`) — covers
    staleness/rebasing for Routine 2.
  - `automation.orchestrate` (`.claude/skills/automation.orchestrate/`) —
    higher-level orchestrator if both routines should report into a single
    dashboard.
- ADR registry lives at `docs/adr/`; sequential numbering enforced by
  `.claude/rules/adr-numbering.md` — check the latest merged ADR PR before
  picking a number.
- Branch protection: feature branches *must* be created via
  `bin/sk-new-feature "..."` (see `.claude/rules/git-workflow.md`). The local
  coder agent must NOT commit directly to `main`; a hook in
  `.claude/hooks/block-main-commits.sh` enforces this but routines should also
  refuse on principle.
- Constitution validator (`.claude/skills/validate-constitution/`) runs on
  push and will catch link/config-pattern regressions — both routines should
  invoke it before opening PRs.
