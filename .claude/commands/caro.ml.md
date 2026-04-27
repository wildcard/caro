---
description: ML/DS engineering skill for Caro fine-tuning pipeline — dataset growth, base-model selection, training experiments, eval harness extension. Targets M4 Max 48 GB.
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `tests/evaluation/src/dataset_export.rs`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/caro.ml` | Run a full ML/DS pass — triage → audit → propose → file/implement → report |
| `/caro.ml --dry-run` | Print the phase plan and what *would* happen, take no action |
| `/caro.ml audit` | Just run AUDIT phase — print dataset stats, coverage matrix, recent eval pass-rate |
| `/caro.ml propose <area>` | Skip triage, jump to PROPOSE for a specific area: `dataset`/`fine-tune`/`model-selection`/`eval-harness` |
| `/caro.ml issue "<title>"` | File a single ML issue with the right labels and template, then exit |
| `/caro.ml log` | Print the last 5 entries from `.claude/memory/ml-session-log.md` |

---

## What This Command Does

`/caro.ml` is the operator console for the **caro-ml-ds-engineer** role (defined in [.claude/agents/ml-ds-engineer.md](.claude/agents/ml-ds-engineer.md)). It is the user-invocable entry point that mirrors the `/ml-fine-tune-loop` daily routine but is interactive — the user runs it when they want a focused ML pass *now* instead of waiting for midnight.

The skill spawns the `ml-ds-engineer` sub-agent with the right context bundle and lets it execute one of the action categories below.

---

## Phases

### 1. CONTEXT
- Read [docs/ROADMAP.md](docs/ROADMAP.md) (or `.claude/memory/ROADMAP.md`) for current strategic direction
- Read `.claude/memory/ml-session-log.md` for what was last done (creates the file on first run)
- Read [docs/fine-tune-pipeline.md](docs/fine-tune-pipeline.md) for current pipeline state (creates a stub on first run)
- `gh issue list --label ml --state open --limit 30` for open ML asks
- `gh pr list --state merged --search "label:ml" --limit 10` for recent ML PRs
- `git log --since='1 week ago' --oneline -- src/evaluation/ src/backends/embedded/ src/prompts/`

### 2. AUDIT
- Dataset count + coverage matrix (per shell × OS × intent), via `cargo run --bin caro-eval -- --stats` or direct read of YAML test cases under `tests/evaluation/datasets/`
- Most recent eval pass-rate per backend from the harness ([src/evaluation/harness.rs](src/evaluation/harness.rs))
- Largest coverage gap — surface the dimension with the lowest count
- Open beads epics tagged `ml` if present

### 3. PROPOSE
Pick **exactly one** action for this pass, ranked by leverage:

| Category | When to pick |
|---|---|
| **Dataset growth** (PR) | Coverage gap is the bottleneck on eval; have a clean public-domain or consented source |
| **Dataset hook** (PR) | Caro is missing telemetry/logging that would let users opt-in to growing the dataset |
| **Base-model bake-off** (issue + plan) | Shortlist hasn't been re-evaluated in >30 days, or a notable new OSS model dropped |
| **LoRA experiment** (issue with config + baseline; PR if approved) | Have a winning candidate base + eval baseline + a hypothesis for what fine-tuning would improve |
| **Eval metric** (PR) | Current harness misses a Caro-specific signal (e.g. shell-portability, safety-validator pass rate alongside correctness) |
| **Pipeline infra** (issue, often P0/P1) | A blocker that prevents the loop from running end-to-end (no training script, no model packaging path, etc.) |

State the choice in plain English with one sentence on **why this and not the others**.

### 4. IMPLEMENT or FILE

**Implement path** (PR):
- Branch: `feat/ml-<short-slug>` off `main`
- Worktree if other sessions are active: `git worktree add .worktrees/ml-<slug> -b feat/ml-<slug>`
- Open as **draft PR** by default, mark ready for review only when CI passes locally
- PR title: `feat(ml): <one-line>` (conventional commits)
- PR body sections: Goal / Why now / What changed / Eval before-after (if applicable) / Follow-ups
- Reference the originating issue if there was one (`Closes #N`)

**File path** (issue):
```bash
gh issue create \
  --title "<area>: <one-line ask>" \
  --label "ml,<sub-label>,<priority>" \
  --body "$(cat <<'EOF'
## Goal
...

## Why now
...

## Proposed approach
...

## Acceptance criteria
- [ ] ...

## Risks / open questions
...

## Effort
T-shirt: S | M | L
EOF
)"
```

Sub-labels: `dataset`, `fine-tune`, `eval`, `model-selection`, `pipeline-infra`.
Priorities: `P0` (loop is broken), `P1` (blocks the next step), `P2` (improves throughput), `P3` (parking lot).
Never include $ / GPU-hours / cost numbers in the issue body — project rule.

### 5. REPORT

Append to `.claude/memory/ml-session-log.md`:
```markdown
## YYYY-MM-DD — <one-line summary>
- **Move:** <category> — <slug>
- **Artifact:** <PR or issue link>
- **Eval delta:** <if applicable>
- **Next:** <one sentence>
```

Echo to user with a session summary that includes:
- `**Today's move:**` <link>
- `**Needs user input:**` <if any>
- Quick actions footer

---

## Output Conventions

- All PR comments follow `~/.claude/rules/pr-comment-structure.md`
- All commits follow `~/.claude/rules/git-commit-rules.md` (conventional, with `Co-Authored-By` trailer)
- Never commit on `main` — feature branch + PR is mandatory (`.claude/rules/git-workflow.md`)
- Never push to remote without user approval for the first push of a new branch

---

## See Also

- [.claude/agents/ml-ds-engineer.md](.claude/agents/ml-ds-engineer.md) — sub-agent persona
- [.claude/commands/ml-fine-tune-loop.md](.claude/commands/ml-fine-tune-loop.md) — autonomous daily routine using the same agent
- [.claude/commands/prompt-tuner.md](.claude/commands/prompt-tuner.md) — companion skill for system-prompt iteration (different surface, related goal)
- [.claude/commands/qa-bundle-validation.md](.claude/commands/qa-bundle-validation.md) — runs eval against shipped bundles; consumer of this role's outputs
