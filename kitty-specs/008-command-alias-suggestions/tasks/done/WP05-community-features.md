# WP05: Community Features

**Work Package**: WP05
**Status**: planned
**Priority**: low
**Estimated Effort**: 5-7 days
**Depends On**: WP03

## Objective

Enable community contribution of cheatsheets and aliases, including website integration, moderation workflow, and CLI contribution tools.

## Tasks

### T5.1: Contribution Format and Validation
- [ ] Finalize cheatsheet YAML schema
- [ ] Create JSON Schema for validation
- [ ] Document contribution guidelines
- [ ] Create template cheatsheet
- [ ] Build schema validator tool

### T5.2: CLI Contribution Commands
- [ ] Add `caro community contribute` wizard
- [ ] Export local aliases to cheatsheet format
- [ ] Preview contribution locally
- [ ] Submit contribution (create GitHub issue/PR)
- [ ] Check contribution status

### T5.3: Website Community Pages
- [ ] Create `/community/contribute` page
- [ ] Build cheatsheet upload form
- [ ] Implement YAML syntax validation
- [ ] Preview contribution in browser
- [ ] GitHub OAuth for attribution

### T5.4: Community Browser
- [ ] Create `/community/tips` page
- [ ] Browse tips by category
- [ ] Search tips and aliases
- [ ] Filter by shell type
- [ ] View contributor profiles

### T5.5: Moderation Dashboard
- [ ] Create `/admin/moderation` page (protected)
- [ ] Queue of pending contributions
- [ ] Approve/reject with feedback
- [ ] Edit before approval
- [ ] Batch operations

### T5.6: GitHub Integration
- [ ] Auto-create PRs for contributions
- [ ] Label contributions for review
- [ ] Notify moderators on new submissions
- [ ] Auto-merge approved contributions
- [ ] Trigger KB rebuild on merge

### T5.7: Contributor Attribution
- [ ] Track contributor GitHub usernames
- [ ] Credit contributors in KB metadata
- [ ] Display attribution in tip display
- [ ] Contributor leaderboard (optional)

### T5.8: Rate Limiting and Spam Prevention
- [ ] Rate limit submissions per user
- [ ] Basic spam detection (duplicate content)
- [ ] Report inappropriate content
- [ ] Block abusive users

## Acceptance Criteria

- [ ] Users can submit cheatsheets via website
- [ ] Submissions appear in moderation queue
- [ ] Moderators can approve/reject contributions
- [ ] Approved contributions merged to KB
- [ ] Contributors credited in tips
- [ ] CLI can export and submit aliases

## Technical Notes

**CLI Contribution Flow**:
```bash
# Export local aliases to cheatsheet
$ caro community export-aliases
Exporting aliases from ~/.zshrc...
Found 15 aliases.
Created: ~/caro-my-aliases.yaml

# Preview before submitting
$ caro community preview ~/caro-my-aliases.yaml
Previewing cheatsheet...
- 15 aliases
- 0 tips
- Shell: zsh
Looks good!

# Submit (opens browser for GitHub OAuth)
$ caro community submit ~/caro-my-aliases.yaml
Opening browser for GitHub authentication...
Submitting to Caro community...
Success! Your contribution is pending review.
Track status: https://github.com/wildcard/caro/issues/XXX
```

**Website Contribution Form**:
```tsx
// website/src/pages/community/contribute.tsx
export default function ContributePage() {
  const [yaml, setYaml] = useState('');
  const [errors, setErrors] = useState<string[]>([]);

  const validate = async () => {
    const result = await validateCheatsheet(yaml);
    setErrors(result.errors);
    return result.valid;
  };

  const submit = async () => {
    if (await validate()) {
      await createPullRequest({
        title: `Community: ${parseTitle(yaml)}`,
        body: yaml,
        labels: ['community', 'needs-review'],
      });
    }
  };

  return (
    <form onSubmit={submit}>
      <YamlEditor value={yaml} onChange={setYaml} />
      <ValidationErrors errors={errors} />
      <button type="submit">Submit for Review</button>
    </form>
  );
}
```

**Moderation Workflow**:
```
1. User submits cheatsheet
   -> Creates GitHub Issue with "community" label

2. GitHub Action validates YAML
   -> Adds "valid" or "invalid" label
   -> Comments with validation results

3. Moderator reviews
   -> Views in moderation dashboard
   -> Can edit YAML inline
   -> Approves or rejects with reason

4. On approval
   -> GitHub Action creates PR
   -> PR auto-merged if CI passes
   -> KB rebuild triggered

5. User notified
   -> GitHub issue updated
   -> Email notification (if enabled)
```

**Spam Prevention**:
```rust
pub fn check_spam(submission: &Submission) -> SpamCheckResult {
    let mut score = 0.0;

    // Check for duplicates
    if has_duplicate_content(&submission.content) {
        score += 0.5;
    }

    // Check submission rate
    if submissions_in_last_hour(&submission.user) > 3 {
        score += 0.3;
    }

    // Check content quality
    if submission.aliases.is_empty() && submission.tips.is_empty() {
        score += 0.4;
    }

    SpamCheckResult {
        is_spam: score > 0.7,
        score,
        reasons: vec![...],
    }
}
```

## Dependencies

- WP03 (Knowledge Base) for KB format
- Website infrastructure
- GitHub OAuth integration
- GitHub API access

## Files to Create

```
# CLI additions
src/commands/
└── community.rs

# Website pages
website/src/pages/community/
├── index.tsx           # Community hub
├── contribute.tsx      # Submit form
├── tips.tsx            # Browse tips
└── aliases.tsx         # Browse aliases

website/src/pages/admin/
└── moderation.tsx      # Moderation dashboard

# Documentation
docs/
└── community/
    ├── contributing.md # Contribution guide
    └── cheatsheet-format.md # Format reference
```

## Privacy Considerations

1. **GitHub Identity**: Contributors identified by GitHub username only
2. **No Email Collection**: Use GitHub notifications
3. **Opt-in Attribution**: Contributors can choose anonymous
4. **Content Ownership**: Contributions licensed under project license
5. **Data Retention**: Rejected submissions not stored long-term
