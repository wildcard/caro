# Leftover Issue Template

Use this template when creating GitHub issues for leftover/incomplete work.

## Template

```markdown
## Context

| Field | Value |
|-------|-------|
| **Origin Branch** | `{branch_name}` |
| **Origin PR** | #{pr_number} - {pr_title} |
| **Work Plan** | `{plan_file_path}` |
| **Created By** | Claude Agent Session |
| **Date** | {iso_date} |

## Original User Request

{user_request_summary}

> Capture the essence of what the user originally asked for.
> Include any specific requirements or constraints mentioned.

## What Was Planned

{planned_work_summary}

- [ ] Item 1 that was planned
- [x] Item 2 that was completed
- [ ] Item 3 that remains (this issue)

## What Was Completed

{completed_items}

Describe what was actually accomplished during the session:
- Files modified: `src/foo.rs`, `src/bar.rs`
- Tests added: `tests/test_foo.rs`
- Documentation updated: `docs/feature.md`

## What Remains

{remaining_items}

**This issue tracks the following incomplete work:**

1. {specific_task_1}
2. {specific_task_2}

### Acceptance Criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Tests pass
- [ ] Documentation updated

## Suggested Next Steps

{next_steps}

1. Start by reading `{relevant_file}:{line_range}`
2. Consider approach X because {reason}
3. Watch out for edge case Y
4. Run tests with `cargo test {test_name}`

## Technical Notes

{technical_notes}

Any discoveries, gotchas, or context that would help someone picking this up:
- Pattern found at `src/module.rs:42`
- Related issue: #{related_issue}
- Dependency: Requires X to be merged first

## Correlation References

```
Branch: {branch_name}
PR: #{pr_number}
Plan: {plan_file_path}
Handoff: {handoff_file_path}
Session: {braintrust_trace_id}
```

---
_This issue was automatically created by `/caro.leftovers` to track incomplete work from an agent session._
```

## Placeholder Reference

| Placeholder | Source | Example |
|-------------|--------|---------|
| `{branch_name}` | `git branch --show-current` | `feature/safety-validation` |
| `{pr_number}` | `gh pr list --head $BRANCH --jq '.[0].number'` | `145` |
| `{pr_title}` | `gh pr list --head $BRANCH --jq '.[0].title'` | `Add command safety validation` |
| `{plan_file_path}` | Search in `thoughts/shared/plans/` | `thoughts/shared/plans/safety.md` |
| `{iso_date}` | `date -u +"%Y-%m-%dT%H:%M:%SZ"` | `2026-01-14T10:30:00Z` |
| `{user_request_summary}` | From conversation context | User's original request |
| `{planned_work_summary}` | From plan file or session | What was intended |
| `{completed_items}` | From session | What got done |
| `{remaining_items}` | User input | What's left |
| `{next_steps}` | Agent analysis | Suggested approach |
| `{handoff_file_path}` | If exists | `thoughts/shared/handoffs/...` |
| `{braintrust_trace_id}` | From state file | Trace ID for debugging |

## Label Mapping

| Category | Labels |
|----------|--------|
| Critical blocker | `leftover`, `from-agent`, `critical`, `priority-high` |
| Technical debt | `leftover`, `from-agent`, `technical-debt` |
| Missing tests | `leftover`, `from-agent`, `testing` |
| Documentation | `leftover`, `from-agent`, `docs` |
| Future enhancement | `leftover`, `from-agent`, `enhancement` |
| Bug discovered | `leftover`, `from-agent`, `bug` |

## gh CLI Command

```bash
# Write body to temp file (handles multiline properly)
cat > /tmp/leftover-issue.md << 'ISSUE_EOF'
{rendered_body}
ISSUE_EOF

# Create issue
gh issue create \
  --title "[Leftover]: {title}" \
  --body-file /tmp/leftover-issue.md \
  --label "leftover,from-agent,{category_label}"
```
