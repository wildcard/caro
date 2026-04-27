# ML Fine-Tune Loop

Autonomous daily routine for the **caro-ml-ds-engineer** role. Runs at 12 AM PST per `.claude/automation/config/schedule.yaml`. One pass per 24h, capped.

The loop is the unattended counterpart to `/caro.ml`: same agent, same phases, same filing rails, but kicked off by cron instead of the user, with a strict "one move per pass" budget so the project's GitHub doesn't drown in agent-filed issues.

## Usage

```
/ml-fine-tune-loop [options]
```

## Options

- `--dry-run` — Run all phases but file no issues, open no PRs. Print what *would* happen.
- `--verbose` — Stream phase output to stdout instead of summarizing.
- `--focus <area>` — Force the PROPOSE phase to consider only one category (`dataset` | `fine-tune` | `model-selection` | `eval-harness` | `pipeline-infra`).
- `--skip-audit` — Skip AUDIT (use the cached audit from yesterday's session log). Faster but stale.

## Process

### 1. Initialize

- Refuse to run if a non-dry pass already ran in the last 23 hours (check `.claude/memory/ml-session-log.md` last entry timestamp).
- Read `.claude/agents/ml-ds-engineer.md` for the persona.
- Read `~/.claude/projects/-Users-kobik-private-workspace-caro/memory/ml_ds_engineer_role.md` for the standing operational rules.
- Confirm working directory is clean enough to spawn a worktree (`git worktree list`; if `.worktrees/ml-*` is stale, abort with a notice).

### 2. Spawn `ml-ds-engineer` Sub-Agent

Hand the sub-agent a context bundle assembled from:

```bash
# Last 24h of activity in the relevant subtrees
git log --since='1 day ago' --oneline -- src/evaluation/ src/backends/embedded/ src/prompts/ tests/evaluation/

# Open ML asks
gh issue list --label ml --state open --limit 30 --json number,title,labels,createdAt

# Recent merged PRs in scope
gh pr list --state merged --search 'label:ml' --limit 10 --json number,title,mergedAt

# Yesterday's session log entry
tail -40 .claude/memory/ml-session-log.md
```

The spawn prompt is structured: "You are caro-ml-ds-engineer. Bundle attached. Pick the highest-leverage move for today (one only). Execute or file. Report back in the strict format below."

### 3. Execute (one of)

| Move | Output |
|---|---|
| **Dataset growth PR** | New branch `feat/ml-data-<slug>`, draft PR, dataset YAML or JSONL diff |
| **Dataset hook PR** | New branch `feat/ml-hook-<slug>`, draft PR adding logging/telemetry plumbing (always opt-in, never default-on) |
| **Base-model issue** | `gh issue create` with `ml,model-selection,P{0-3}`, body includes shortlist + acceptance criteria |
| **LoRA experiment plan** | `gh issue create` with `ml,fine-tune,P{0-3}`, body includes config + baseline reference |
| **Eval metric PR** | New branch `feat/ml-eval-<slug>`, draft PR extending [src/evaluation/harness.rs](src/evaluation/harness.rs) |
| **Pipeline infra issue** | `gh issue create` with `ml,pipeline-infra,P0` or `P1`, body explains the blocker |

Hard cap: **one filed artifact per pass** (PR or issue, not both). If the agent thinks two moves are needed, it picks the most blocking and notes the second under `**Needs user input:**` for human decision.

### 4. Log

Append to `.claude/memory/ml-session-log.md`:

```markdown
## YYYY-MM-DD — <one-line>
- **Move:** <category> — <slug>
- **Artifact:** <PR or issue link>
- **Eval delta:** <if any>
- **Next:** <one-sentence pointer>
```

### 5. Report

Exit with a structured summary on stdout:

```
ml-fine-tune-loop: <date> — <move category>
  artifact: <link>
  needs user input: <yes|no>
  next pass: <date+1>
```

The schedule layer surfaces this on the Slack/email channel configured in `.claude/automation/config/schedule.yaml` (`notify_on_failure: true`, plus the loop's own `notify_on_completion` if the user opts in).

## Failure Modes

| Symptom | Cause | Fix |
|---|---|---|
| "Refusing: ran 2h ago" | The 23h-cap is doing its job | Wait, or pass `--force` (only acceptable interactively, not from cron) |
| `gh: not authenticated` | Cron environment missing token | Ensure `GH_TOKEN` is in the schedule runner's env |
| "No clean main" | Local main has uncommitted state from another session | Loop aborts cleanly; user resolves |
| Sub-agent spawned but produced 0 moves | Empty inbox + no audit gap | Acceptable. Log "no-op pass" with rationale. The loop is allowed to skip days. |

The loop is **allowed to be a no-op**. Doing nothing on a quiet day is correct behavior; filing an issue for the sake of filing is not.

## Constraints

Inherits all from [.claude/agents/ml-ds-engineer.md](.claude/agents/ml-ds-engineer.md). Notably:
- No commits to `main`. Worktree + draft PR only.
- No financial / GPU-hour numbers in any caro file or issue.
- No PII / user transcripts pushed to the public repo.
- Every issue/PR comment follows `~/.claude/rules/pr-comment-structure.md`.

## See Also

- [.claude/agents/ml-ds-engineer.md](.claude/agents/ml-ds-engineer.md) — the agent this loop spawns
- [.claude/commands/caro.ml.md](.claude/commands/caro.ml.md) — interactive twin of this loop
- [.claude/automation/config/schedule.yaml](.claude/automation/config/schedule.yaml) — cron entry: `ml_fine_tune_loop @ 0 0 * * *`
- [.claude/commands/qa-automation-loop.md](.claude/commands/qa-automation-loop.md) — sibling loop for QA; same architectural pattern
