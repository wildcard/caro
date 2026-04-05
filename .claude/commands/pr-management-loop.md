# PR Management Loop

Automated PR management that monitors open PRs, interacts with external review agents (Kubic, Copilot), keeps PRs fresh through rebasing, and ensures work flows to completion.

**Philosophy**: Follows the [Vibe Maintainer](https://steve-yegge.medium.com/vibe-maintainer-a2273a841040) workflow (see ADR-015). Default to "yes." Fix issues rather than bouncing them back. Target 88%+ merge rate.

## Usage

```
/pr-management-loop [options]
```

## Options

- `--dry-run` - Analyze without taking actions
- `--pr <number>` - Process specific PR only
- `--verbose` - Show detailed analysis

## Process

### 1. Scan Open PRs

```bash
gh pr list --state open --json number,title,author,createdAt,updatedAt,
  labels,reviews,statusCheckRollup,mergeable,headRefName
```

### 2. Analyze Each PR

For each PR, calculate:

| Metric | How |
|--------|-----|
| Staleness | Days since last human activity |
| CI Status | Check status from statusCheckRollup |
| Review Status | Approved, changes requested, pending |
| Agent Comments | Parse Kubic/Copilot comments |
| Merge Readiness | Conflicts, approvals, CI |

### 3. Classify

```
healthy       - CI passing, recent activity
stale         - No activity > 3 days
stale_critical - No activity > 7 days
ci_failing    - CI checks failing
needs_review  - CI passing, no reviews
has_feedback  - Unaddressed review comments
ready_to_merge - Approved, CI passing, no conflicts
has_conflicts - Merge conflicts present
```

### 3.5 Vibe Triage (PR Akita Disposition)

After classification, assign a **disposition** based on the Vibe Maintainer philosophy (ADR-015).
The default is "yes" -- look for reasons to merge, not reasons to reject.

| Classification | Default Disposition | Action |
|---|---|---|
| ready_to_merge | **Merge** | Auto-merge if enabled, manual merge otherwise |
| healthy + needs_review | **Needs-Review** | Flag for human maintainer with summary |
| ci_failing (simple fix) | **Fix-Merge** | Checkout, fix CI issue, push, merge with attribution |
| ci_failing (complex) | **Needs-Review** | Flag for human with CI analysis |
| stale (< 7 days) | **Fix-Merge** | Rebase + fix issues yourself |
| stale_critical (> 7 days) | **Retire** | Close kindly with gratitude |
| has_conflicts (simple) | **Fix-Merge** | Rebase on main yourself |
| has_conflicts (complex) | **Needs-Review** | Flag for human review |
| has_feedback (unaddressed) | **Fix-Merge** | Address feedback yourself, merge |

**Simple vs Complex**: A fix is "simple" if it can be resolved in ~30 minutes (formatting, clippy, imports, rebase conflicts). Anything requiring changes to core logic or architectural understanding is "complex."

**Fix-Merge Attribution**: Always use `Co-authored-by: Name <email>` when fixing someone else's PR. See `.claude/rules/vibe-maintainer.md`.

### 4. Execute Actions

**Stale PRs:**
```bash
# Rebase with main
git fetch origin main
git checkout <branch>
git rebase origin/main
git push --force-with-lease

# Comment
gh pr comment <number> --body "🔄 Rebased with main"
```

**CI Failing (Fix-Merge disposition):**
```bash
# Checkout PR branch
git fetch origin pull/<number>/head:pr-<number>
git checkout pr-<number>

# Fix the issue (clippy, fmt, imports, etc.)
cargo fmt
cargo clippy --fix --allow-dirty

# Commit with attribution
git add -A
git commit -m "fix: resolve CI issues

Co-authored-by: Original Author <email@users.noreply.github.com>"

# Push and merge
git push origin pr-<number> --force-with-lease
gh pr merge <number> --squash

# Comment
gh pr comment <number> --body "Thanks for this! I fixed the CI issue and merged. You get full credit as co-author."
```

**CI Failing (Needs-Review disposition):**
```bash
# Analyze failure
gh run view <run_id> --log-failed

# Comment with analysis
gh pr comment <number> --body "## CI Analysis
The build failed because...
Suggested fix: ..."
```

**Needs Review:**
```bash
# Request reviews
gh pr edit <number> --add-reviewer @codeowners
gh pr comment <number> --body "👋 Ready for review!"
```

**External Agent Feedback:**
```bash
# Parse agent comment
# Generate response
# Apply fixes if possible
gh pr comment <number> --body "Applied Kubic suggestion..."
```

**Stale Critical (Retire disposition):**
```bash
# Close kindly with gratitude
gh pr comment <number> --body "Thanks for this contribution, @author! This PR has been inactive for a while and the codebase has moved on. I'm closing it, but the idea was appreciated. Feel free to open a fresh PR if you'd like to revisit this."
gh pr close <number>
```

**Ready to Merge:**
```bash
# If auto-merge enabled
gh pr merge <number> --squash --auto
```

## Example Session

```
> /pr-management-loop

PR Management Loop
══════════════════

Scanning open PRs...
Found 12 open PRs

Analyzing...

PR #234: Add fish shell support
  Author: @alice
  Age: 5 days
  Last activity: 3 days ago
  CI: ✓ passing
  Reviews: 1 approved
  Status: ready_to_merge

  → Action: Auto-merge eligible (manual merge required)

PR #235: Fix pipe command parsing
  Author: @claude-bot
  Age: 4 days
  Last activity: 4 days ago
  CI: ✓ passing
  Reviews: 0
  Status: stale, needs_review

  → Actions:
    ✓ Rebased with main
    ✓ Requested review from @maintainers

PR #236: Update documentation
  Author: @bob
  Age: 2 days
  CI: ✗ failing
  Status: ci_failing

  → Action: Analyzed CI failure
    Error: Missing import in docs-site/astro.config.mjs
    Commented with fix suggestion

PR #237: Security improvements
  Author: @carol
  Age: 1 day
  CI: ✓ passing
  Reviews: Kubic commented
  Status: has_feedback

  → Actions:
    ✓ Responded to Kubic feedback
    ✓ Applied suggested security fix

Summary:
  PRs processed: 12
  Actions taken: 8
    - Rebased: 3
    - Review requested: 2
    - CI analyzed: 1
    - Agent responses: 2

Next run: 4 hours
```

## External Agent Integration

### Kubic Bot

```yaml
# Detection
comment_author: "kubic[bot]"

# Parse feedback
- type: security_warning
  severity: high
  suggestion: "Add input validation"

# Response
- If auto-fix possible: Apply and comment
- If manual needed: Create TODO, ping author
```

### GitHub Copilot

```yaml
# Detection
comment_author: "github-advanced-security[bot]"

# Types
- code_scanning_alert
- dependency_review
- secret_scanning

# Response
- Log alert
- Comment acknowledgment if needed
```

## Claude Code Web Sessions

Scan for orphan branches from Claude Code Web:

```bash
# Find Claude branches without PRs
git branch -r | grep 'claude/' | while read branch; do
  if ! gh pr list --head "${branch#origin/}" --state all | grep -q .; then
    echo "Orphan: $branch"
  fi
done
```

Options:
- Create PR automatically
- Notify owner
- Archive if old

## Configuration

```yaml
# .claude/automation/config/pr_management.yaml
pr_management:
  enabled: true
  schedule: "0 */4 * * *"

  staleness:
    warn_after_days: 3
    critical_after_days: 7

  rebase:
    auto_rebase: true
    force_push: true

  reviews:
    auto_request: true
    ping_after_days: 2

  merge:
    auto_merge: false  # Require manual
    require_approvals: 1

  external_agents:
    respond_to_kubic: true
    auto_apply_fixes: false

  # Vibe Maintainer settings (ADR-015)
  vibe_maintainer:
    enabled: true
    auto_fix_merge: false  # Require manual approval for fix-merges initially
    merge_rate_target: 88
    retire_after_days: 14
    attribution:
      always_coauthor: true
      preserve_original_author: true
```

## Output Report

```yaml
# .claude/automation/state/pr_management/2026-01-11.yaml
run:
  id: "pr-mgmt-2026-01-11-120000"
  started: "2026-01-11T12:00:00Z"
  completed: "2026-01-11T12:05:34Z"

  prs_scanned: 12

  by_classification:
    healthy: 5
    stale: 3
    ci_failing: 2
    needs_review: 1
    ready_to_merge: 1

  actions_taken:
    - pr: 235
      action: "rebased"
      result: "success"
    - pr: 235
      action: "requested_reviews"
      reviewers: ["@maintainers"]

  by_disposition:
    merge: 1
    fix_merge: 3
    needs_review: 2
    retire: 1
    reject: 0
    request_changes: 0

  metrics:
    avg_pr_age_days: 4.2
    merge_rate_7d: 85%
    merge_rate_30d: 88%
    fix_merge_ratio: 42%
```

## Related Skills

- `skill: pr-akita` - AI-powered PR triage with vibe-maintainer philosophy
- `/stale-revival-loop` - Weekly deep stale cleanup
- `/caro.roadmap` - Roadmap-aligned PR prioritization
- `/create_handoff` - Create handoff for stale PRs
- ADR-015 - Vibe Maintainer Workflow architectural decision
- `.claude/rules/vibe-maintainer.md` - Disposition hierarchy and attribution rules

## DRS Reference

See [PR_MANAGEMENT_DRS.md](../.claude/automation/specs/PR_MANAGEMENT_DRS.md)
