# PR Management Loop - Design Requirements Specification

> **Document Type**: DRS
> **Version**: 1.0.0
> **Status**: Active
> **Parent**: [AUTOMATED_DEV_FLOW_DRS.md](./AUTOMATED_DEV_FLOW_DRS.md)
> **Pack**: Management

---

## 1. Overview

The PR Management Loop monitors open PRs, interacts with external review agents (Kubic, Copilot), keeps PRs fresh, and ensures work flows to completion.

### 1.1 Objectives

1. **Prevent Stale PRs**: Keep PRs moving toward merge
2. **External Agent Coordination**: Respond to Kubic/Copilot feedback
3. **Roadmap Alignment**: Ensure work aligns with priorities
4. **Automated Maintenance**: Rebase, conflict resolution, CI monitoring

---

## 2. System Design

### 2.1 Component Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    PR MANAGEMENT LOOP                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  TRIGGER: Every 4 hours (or manual)                             │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    PR SCANNER                              │  │
│  │                 gh pr list --state open                    │  │
│  └─────────────────────────┬─────────────────────────────────┘  │
│                            │                                     │
│                            ▼                                     │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    PR ANALYZER                             │  │
│  │  For each PR:                                              │  │
│  │  ├── Last activity time                                    │  │
│  │  ├── CI status                                             │  │
│  │  ├── Review status                                         │  │
│  │  ├── External agent comments                               │  │
│  │  ├── Merge conflicts                                       │  │
│  │  └── Roadmap alignment                                     │  │
│  └─────────────────────────┬─────────────────────────────────┘  │
│                            │                                     │
│         ┌──────────────────┼──────────────────┐                  │
│         ▼                  ▼                  ▼                  │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐            │
│  │   STALE     │   │  CI/REVIEW  │   │   READY     │            │
│  │   HANDLER   │   │   HANDLER   │   │   HANDLER   │            │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘            │
│         │                 │                 │                    │
│         ▼                 ▼                 ▼                    │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐            │
│  │ Ping author │   │ Fix issues  │   │ Auto-merge  │            │
│  │ Rebase      │   │ Respond to  │   │ (if enabled)│            │
│  │ Close if old│   │ agents      │   │             │            │
│  └─────────────┘   └─────────────┘   └─────────────┘            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 PR Classification

```yaml
# Classification criteria
classifications:
  stale:
    criteria:
      - last_activity_days: ">3"
      - no_pending_review: true
    actions:
      - rebase_with_main
      - ping_author
      - request_reviews

  stale_critical:
    criteria:
      - last_activity_days: ">7"
      - no_pending_review: true
    actions:
      - rebase_with_main
      - ping_author
      - add_stale_label
      - consider_closing

  ci_failing:
    criteria:
      - ci_status: "failing"
    actions:
      - analyze_failure
      - comment_with_analysis
      - suggest_fix

  needs_review:
    criteria:
      - ci_status: "passing"
      - reviews: 0
    actions:
      - request_reviews
      - ping_codeowners

  has_feedback:
    criteria:
      - has_review_comments: true
      - not_addressed: true
    actions:
      - summarize_feedback
      - ping_author

  ready_to_merge:
    criteria:
      - ci_status: "passing"
      - reviews_approved: ">=1"
      - no_conflicts: true
    actions:
      - auto_merge (if enabled)
      - notify_author

  has_conflicts:
    criteria:
      - merge_conflict: true
    actions:
      - attempt_rebase
      - if_fail: notify_author
```

---

## 3. External Agent Integration

### 3.1 Supported Agents

```yaml
# .claude/automation/config/pr_agents.yaml
external_agents:
  kubic:
    name: "Kubic"
    type: "github_bot"
    comment_prefix: "@kubic"
    capabilities:
      - code_review
      - security_analysis
      - performance_suggestions
    response_required: true
    response_timeout_hours: 24

  copilot:
    name: "GitHub Copilot"
    type: "github_bot"
    comment_prefix: "@github-actions"  # or @github-advanced-security
    capabilities:
      - code_review
      - security_scan
    response_required: false
    track_suggestions: true

  coderabbit:
    enabled: false
    name: "CodeRabbit"
    type: "github_bot"
    # Future integration
```

### 3.2 Agent Response Flow

```
External Agent Comments on PR
    │
    ▼
┌─────────────────────────────────────┐
│ Detect agent comment                │
│ (via comment author/prefix)         │
└─────────────────┬───────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│ Parse feedback                      │
│ - Type: suggestion, warning, error  │
│ - Severity: low, medium, high       │
│ - Actionable: yes/no                │
└─────────────────┬───────────────────┘
                  │
           ┌──────┴──────┐
           ▼             ▼
    ┌───────────┐ ┌───────────────────┐
    │ Automated │ │ Requires Human    │
    │ Response  │ │ Intervention      │
    └─────┬─────┘ └─────────┬─────────┘
          │                 │
          ▼                 ▼
    ┌───────────┐ ┌───────────────────┐
    │ Apply fix │ │ Create TODO       │
    │ Comment   │ │ Ping author       │
    │ "Applied" │ │ Add to handoff    │
    └───────────┘ └───────────────────┘
```

---

## 4. Execution Flow

### 4.1 Step-by-Step Process

```
1. FETCH OPEN PRS
   │
   ├── gh pr list --state open --json number,title,author,...
   └── Include: labels, reviews, CI status, comments

2. ANALYZE EACH PR
   │
   For each PR:
   │
   ├── Calculate staleness
   │   └── days since last meaningful activity
   │
   ├── Check CI status
   │   ├── Passing: ✓
   │   ├── Failing: Analyze logs
   │   └── Pending: Skip
   │
   ├── Check reviews
   │   ├── Count approvals
   │   ├── Count requests for changes
   │   └── Identify pending reviewers
   │
   ├── Check external agents
   │   ├── Kubic comments
   │   ├── Copilot suggestions
   │   └── Unaddressed feedback
   │
   ├── Check merge status
   │   ├── Conflicts: yes/no
   │   └── Behind main by: N commits
   │
   └── Check roadmap alignment
       └── Linked to milestone/issue?

3. CLASSIFY PR
   │
   └── Assign to category: stale, ci_failing, needs_review, etc.

4. EXECUTE ACTIONS
   │
   Based on classification:
   │
   ├── Stale:
   │   ├── git fetch origin main
   │   ├── git rebase origin/main
   │   ├── git push --force-with-lease
   │   └── gh pr comment "Rebased with main"
   │
   ├── CI Failing:
   │   ├── Analyze failure logs
   │   ├── Identify root cause
   │   └── Comment with suggestion
   │
   ├── Has External Feedback:
   │   ├── Parse agent comments
   │   ├── Generate response
   │   └── Apply fixes if possible
   │
   └── Ready to Merge:
       └── gh pr merge --auto (if enabled)

5. REPORT SUMMARY
   │
   ├── PRs processed: N
   ├── Actions taken: [list]
   ├── Issues found: [list]
   └── Store in state/pr_management/{date}.yaml
```

### 4.2 Staleness Algorithm

```python
def calculate_staleness_score(pr):
    """
    Calculate how 'stale' a PR is on a 0-100 scale.
    Higher = more stale, needs more attention.
    """
    score = 0

    # Days since last meaningful activity
    # (not just bot comments)
    days_inactive = get_days_since_human_activity(pr)
    score += min(days_inactive * 10, 50)

    # Review status
    if pr.reviews_requested > 0 and pr.reviews_received == 0:
        score += 20  # Waiting for reviews
    if pr.has_change_requests and not pr.addressed:
        score += 15  # Has unaddressed feedback

    # CI status
    if pr.ci_status == "failing":
        score += 20
    elif pr.ci_status == "pending":
        score += 5

    # Merge conflicts
    if pr.has_conflicts:
        score += 15

    # Age of PR
    days_old = (now - pr.created_at).days
    if days_old > 14:
        score += 10
    if days_old > 30:
        score += 10

    return min(score, 100)
```

---

## 5. Actions Reference

### 5.1 Rebase Action

```bash
# Automated rebase workflow
git fetch origin main
git checkout pr-branch
git rebase origin/main

if [ $? -eq 0 ]; then
    git push --force-with-lease
    gh pr comment $PR_NUMBER --body "🔄 Rebased with main to resolve conflicts."
else
    gh pr comment $PR_NUMBER --body "⚠️ Automatic rebase failed. Manual intervention required.

Conflicts in:
$(git diff --name-only --diff-filter=U)

Please run:
\`\`\`
git fetch origin main
git rebase origin/main
# resolve conflicts
git push --force-with-lease
\`\`\`"
fi
```

### 5.2 Review Request Action

```bash
# Request reviews from CODEOWNERS
gh pr edit $PR_NUMBER --add-reviewer @org/codeowners

# Comment to ping
gh pr comment $PR_NUMBER --body "👋 Friendly ping for review!

This PR has been open for ${DAYS_OPEN} days.

**Summary**: ${PR_TITLE}
**Changes**: ${FILES_CHANGED} files changed

cc @maintainers"
```

### 5.3 CI Analysis Action

```bash
# Analyze CI failure
LOGS=$(gh run view $RUN_ID --log-failed)

# Use LLM to analyze
ANALYSIS=$(analyze_ci_logs "$LOGS")

gh pr comment $PR_NUMBER --body "## CI Analysis

The CI run failed with:

\`\`\`
${ERROR_SUMMARY}
\`\`\`

**Suggested fix:**
${ANALYSIS}

<details>
<summary>Full logs</summary>

\`\`\`
${LOGS}
\`\`\`
</details>"
```

---

## 6. Configuration

```yaml
# .claude/automation/config/pr_management.yaml
pr_management:
  enabled: true
  schedule: "0 */4 * * *"  # Every 4 hours

  staleness:
    warn_after_days: 3
    critical_after_days: 7
    close_after_days: 30
    close_enabled: false  # Require manual close

  rebase:
    auto_rebase: true
    force_push: true  # Uses --force-with-lease
    notify_on_failure: true

  reviews:
    auto_request: true
    ping_after_days: 2
    max_pings: 2

  merge:
    auto_merge: false  # Require manual merge
    require_approvals: 1
    require_ci_pass: true
    squash_commits: true

  external_agents:
    respond_to_kubic: true
    respond_to_copilot: false
    auto_apply_fixes: false  # Require human review

  claude_code_web:
    scan_orphan_branches: true
    create_prs_for_orphans: false  # Just notify

  notifications:
    on_stale: true
    on_ci_failure: true
    on_agent_feedback: true
```

---

## 7. Output Artifacts

### 7.1 Run Report

```yaml
# .claude/automation/state/pr_management/{date}.yaml
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
    - pr: 123
      action: "rebased"
      result: "success"

    - pr: 124
      action: "requested_reviews"
      reviewers: ["@alice", "@bob"]

    - pr: 125
      action: "commented_ci_analysis"
      error_type: "test_failure"

    - pr: 126
      action: "responded_to_kubic"
      suggestion: "accepted"

  claude_code_web:
    orphan_branches_found: 2
    branches:
      - name: "claude/feature-xyz-abc123"
        age_days: 5
        action: "notified"

  metrics:
    avg_pr_age_days: 4.2
    avg_time_to_first_review_hours: 18
    merge_rate_7d: 85%
```

---

## 8. Integration with Dev Loop

```
PR Management Loop
    │
    ├── Identifies PRs needing work
    │
    ├── Creates handoffs for stale PRs
    │   └── /create_handoff with PR context
    │
    └── Feeds into /caro.roadmap
        │
        ▼
    Dev Loop picks up work
        │
        ├── /resume_handoff
        │
        └── Completes PR
```

---

## 9. Related Documents

- [PR_MANAGEMENT_TEST.md](../tests/PR_MANAGEMENT_TEST.md) - Test cases
- [STALE_REVIVAL_DRS.md](./STALE_REVIVAL_DRS.md) - Weekly stale revival
- [caro.roadmap](../../commands/caro.roadmap.md) - Roadmap integration
