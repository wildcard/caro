---
description: Continuous v1.2.0 feature development — claims next ready beads task, implements via kraken (TDD), opens PR. Loop until no ready work remains.
---

# Caro Coder Loop (v1.2.0)

You are the **local coder agent** for the caro v1.2.0 milestone (Epic #792, due 2026-04-30). Your job: drain the beads queue by implementing one task per iteration, opening a PR per task, then looping.

## Preflight (run once at loop entry)

```bash
cd /Users/kobik-private/workspace/caro
bd stats | grep -E "Ready to Work|In Progress"
git fetch origin main
```

If `Ready to Work` == 0 on v1.2.0 tasks, exit gracefully with: `"No ready v1.2.0 work remaining. Backlog-grooming routine will re-populate on next 6h tick."`

## Per-iteration workflow

### 1. Claim

```bash
# Get next ready v1.2.0 task (highest priority, Phase-1 first by dependency order)
NEXT=$(bd ready --json 2>/dev/null | jq -r '.[] | select(.labels[]? == "v1.2.0") | .id' | head -1)
[ -z "$NEXT" ] && { echo "no ready v1.2.0 work"; exit 0; }
bd claim "$NEXT"
bd show "$NEXT"
bin/notify coder "claim $NEXT"
```

Capture:
- `NEXT` — beads ID (e.g. `caro-xk0.1`)
- `GH_ISSUE` — from `external_ref` field (e.g. `gh-793`)
- `TITLE`, `DESCRIPTION`, `LABELS`

### 2. Create isolated worktree

Follow `.claude/rules/git-workflow.md` — NEVER work on main.

```bash
SLUG=$(echo "$TITLE" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9' '-' | sed 's/--*/-/g' | cut -c1-40)
cd /Users/kobik-private/workspace/caro
bin/sk-new-feature "$SLUG"
cd .worktrees/*"$SLUG"*/
```

### 3. Gather context

Before delegating to kraken:

```bash
# Read the linked GitHub issue body
gh issue view "${GH_ISSUE#gh-}" > /tmp/issue-body.md
# Read the v1.2.0 tech spec if referenced
ls specs/v1.2-delivering-on-the-promise/ 2>/dev/null && cat specs/v1.2-delivering-on-the-promise/tech-spec.md
```

### 4. Pick the right specialist agent

**Consult `.github/STAKEHOLDERS.yml` first.** The map pairs codebase paths
with the specialist agents that should own changes there. Pick the most
specific glob match for the paths the task is likely to touch — derived
from the issue body, beads labels (`area:safety`, `area:ml`, …), or the
title.

```bash
# Heuristic: extract candidate paths from the issue body
PATHS=$(grep -oE '(src/[a-z_/]+|tests/[a-z_]+|website/[^ ]+|\.github/[^ ]+|Cargo\.[a-z]+)' /tmp/issue-body.md | sort -u)

# Look up agents per path; the first concrete glob match wins
yq -r --arg p "$PATHS" '
  .areas | to_entries[] | select(.key as $k | $p | test($k))
  | "\(.key) -> \(.value.agents | join(\",\"))"
' .github/STAKEHOLDERS.yml | head -3
```

The agent named on the most-specific match becomes the **primary** specialist
for the task. Examples:

| Path touched | Primary specialist |
|---|---|
| `src/safety/patterns.rs` | `safety-pattern-developer` (TDD discipline required) |
| `src/safety/**` | `safety-pattern-developer`, fall back to `safety-pattern-auditor` |
| `src/backends/embedded_backend.rs` | `llm-integration-expert` |
| `src/ai/**`, `src/eval/**` | `ml-ds-engineer` |
| `src/cli/**`, `src/main.rs` | `rust-cli-expert` |
| `Cargo.toml`, `.github/workflows/release.yml` | `caro-release-expert` |
| `website/**` | `technical-writer` |
| `website/src/i18n/**` | `technical-writer` + `cultural-heritage-expert` |
| `specs/**` | `spec-driven-dev-guide` |

**Fallback rules** (when STAKEHOLDERS.yml has no match):
- `labels` contains `documentation` → use **spark** (Sonnet, fast)
- Otherwise → use **kraken** (Opus, TDD for Rust safety-critical code)

**Critical**: never bypass `safety-pattern-developer` for `src/safety/**`
changes — that path requires the TDD-first workflow encoded in the skill.

### 5. Delegate to the specialist

Spawn via Task tool with this prompt template:

```
You are implementing beads task $NEXT (GitHub #${GH_ISSUE#gh-}) for caro v1.2.0.

**Working directory**: $(pwd) (a git worktree on branch $(git branch --show-current))

**Task**: $TITLE

**Issue body**: [contents of /tmp/issue-body.md]

**Acceptance criteria**: see the task description and the v1.2.0 tech spec at specs/v1.2-delivering-on-the-promise/tech-spec.md.

**Conventions** (from /Users/kobik-private/workspace/caro/CLAUDE.md):
- Rust edition 2021, MSRV 1.85
- Use `thiserror` for error types, `anyhow::Result` for application errors
- TDD: write failing test FIRST, then make it pass
- All commit messages conventional: `<type>(<scope>): <subject>` + Co-Authored-By
- Run `cargo test` + `cargo clippy` before declaring done
- For safety-related changes (src/safety/), add test cases to patterns.rs tests

**Apply the Good Boy Scout rule** (.claude/rules/good-boy-scout.md) — fix obvious
nearby issues but don't gold-plate.

When done:
1. Commit all changes with conventional message
2. Report: files changed, tests added, clippy status, test pass count
```

### 6. Reviewer gate (swarm-forge pattern)

Before validating or opening a PR, spawn an independent **Reviewer** sub-agent
via the Task tool. Borrowed from swarm-forge's Architect → Coder → **Reviewer**
pipeline: the coder cannot self-approve.

**Subagent**: `qa-testing-expert` (model: opus)

**Reviewer prompt template:**

```
You are the REVIEWER gate for caro v1.2.0 task $NEXT (PR not yet opened).

Working directory: $(pwd)
Branch: $(git branch --show-current)
Diff to review: `git diff main...HEAD`

You MUST run, and your verdict MUST cite the actual output of:
  1. `cargo fmt --all -- --check`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo test`
  4. Constitution check: any change under `src/safety/` MUST have a
     corresponding test added in the same diff (see the
     `safety-pattern-developer` skill for the source-of-truth
     requirement).
  5. Good-boy-scout check: the diff does not introduce obvious
     nearby brokenness left unfixed.

Return JSON on the LAST line of your reply:
  {"verdict":"pass"} | {"verdict":"block","reasons":["...","..."]}

Be terse. No PR is opened on `block` — instead the loop sends your
reasons back to the coder for a fix pass.
```

Capture the verdict (concrete extraction):

```bash
# 1. Pipe the agent reply (stdout) to a file: $REVIEW_OUT
# 2. Grab the LAST non-empty line — that's the verdict JSON
LAST_LINE=$(grep -v '^$' "$REVIEW_OUT" | tail -1)

# 3. Validate it parses as JSON; if not, treat as block (defensive default)
if ! echo "$LAST_LINE" | jq -e . >/dev/null 2>&1; then
  VERDICT="block"
  REASONS="reviewer did not return valid JSON on last line: $LAST_LINE"
else
  VERDICT=$(echo "$LAST_LINE" | jq -r '.verdict // "block"')
  REASONS=$(echo "$LAST_LINE" | jq -r '.reasons // [] | join("; ")')
fi

# 4. Branch on verdict
if [ "$VERDICT" = "block" ]; then
  bin/notify reviewer "block $NEXT: $REASONS"
  # re-delegate to coder with $REASONS appended; on third block, mark blocked.
  bd update "$NEXT" --status blocked --notes "reviewer blocked: $REASONS"
  exit 1
fi
bin/notify reviewer "pass $NEXT"
```

### 7. Validate locally

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

If any fail: `bd update "$NEXT" --status blocked --notes "validation failed: <err>"`, push branch for debugging, exit loop.

### 8. Open PR + close beads task

```bash
BRANCH=$(git branch --show-current)
git push -u origin "$BRANCH"

PR_URL=$(gh pr create \
  --title "$(echo "$TITLE" | cut -c1-70)" \
  --body "$(cat <<EOF
## Summary
Implements beads task \`$NEXT\` → closes #${GH_ISSUE#gh-}.

Part of Epic #792 (v1.2.0: Delivering on the Promise).

## Changes
<agent-filled summary from kraken report>

## Test plan
- [x] \`cargo test\` passes
- [x] \`cargo clippy -- -D warnings\` passes
- [x] \`cargo fmt --check\` passes
- [ ] Manual smoke test on target platform

🤖 Generated by /caro-coder-loop
EOF
)" --json url --jq .url)

bd close "$NEXT" --notes "PR: $PR_URL"
bin/notify coder "pr opened $PR_URL ($NEXT)"
echo "✓ closed $NEXT → $PR_URL"
```

### 9. Loop

Return to step 1 until `bd ready` has no v1.2.0 entries OR user interrupts.

## Safety rails

- **Never merge** the PR yourself — Routine B (`caro-merge-review-integrate`) owns merging after human review.
- **Never touch `src/safety/patterns.rs` without TDD** — if kraken tries to skip, reject and mark task blocked.
- **Never work on main** — enforced by `.claude/rules/git-workflow.md` hook.
- **Atomic claim** — `bd claim` is atomic across processes, so scheduled tasks and live sessions won't collide.
- **Exit on first failure** — don't cascade broken state into the next task.

## Invocation

Manual (live session):
```
/caro-coder-loop
```

As a `/loop` skill:
```
/loop /caro-coder-loop
```
(the loop skill will self-pace between iterations)
