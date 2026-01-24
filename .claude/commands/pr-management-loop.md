# PR Management Loop

Automated PR management that monitors open PRs, interacts with external review agents (Kubic, Copilot), keeps PRs fresh through rebasing, and ensures work flows to completion.

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

**CI Failing:**
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

  metrics:
    avg_pr_age_days: 4.2
    merge_rate_7d: 85%
```

## Related Skills

- `/stale-revival-loop` - Weekly deep stale cleanup
- `/caro.roadmap` - Roadmap-aligned PR prioritization
- `/create_handoff` - Create handoff for stale PRs

## DRS Reference

See [PR_MANAGEMENT_DRS.md](../.claude/automation/specs/PR_MANAGEMENT_DRS.md)
