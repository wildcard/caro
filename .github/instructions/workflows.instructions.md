---
applyTo: ".github/workflows/**/*.yml,.github/workflows/**/*.yaml"
---

# GitHub Actions Workflow Review Instructions

Apply these guidelines when reviewing GitHub Actions workflow files.

## Security Requirements

### Critical: pull_request_target Security
The `pull_request_target` trigger has write access to secrets and the repository. Use with extreme caution:

```yaml
# DANGEROUS: Checkout and run code from fork PRs
on: pull_request_target

jobs:
  build:
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.pull_request.head.sha }}  # DANGER: Fork code
      - run: npm install  # Runs untrusted code with secrets access
```

### Safe Patterns for pull_request_target
```yaml
# SAFE: Only label or comment, no code execution
on: pull_request_target

jobs:
  label:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/labeler@v5  # No code checkout

# SAFE: Checkout base branch only
on: pull_request_target

jobs:
  check:
    steps:
      - uses: actions/checkout@v4  # Default: base branch, safe
```

### Permission Scoping
Always use minimal permissions:

```yaml
permissions:
  contents: read    # Default to read-only
  pull-requests: write  # Only if needed

jobs:
  build:
    permissions:
      contents: read  # Override at job level
```

## Common Issues from Past Reviews

### Missing Permissions for Release Drafter
```yaml
# BAD: Missing required permission
on:
  push:
    branches: [main]

jobs:
  release-drafter:
    runs-on: ubuntu-latest
    steps:
      - uses: release-drafter/release-drafter@v5

# GOOD: Include contents: write
permissions:
  contents: write
  pull-requests: read

jobs:
  release-drafter:
    # ...
```

### Pagination Limits in API Calls
```yaml
# BAD: Limited to 100 results without pagination
- name: Get PRs
  run: |
    gh api repos/${{ github.repository }}/pulls --per-page 100

# GOOD: Use pagination for complete data
- name: Get all PRs
  run: |
    gh api repos/${{ github.repository }}/pulls --paginate
```

### Bot Detection
```yaml
# BAD: Fragile string matching
if: contains(github.actor, '[bot]') || contains(github.actor, 'bot')

# GOOD: Use user type check
- name: Check if actor is bot
  id: check-bot
  run: |
    USER_TYPE=$(gh api users/${{ github.actor }} --jq '.type')
    echo "is_bot=$([[ "$USER_TYPE" == "Bot" ]] && echo true || echo false)" >> $GITHUB_OUTPUT
  env:
    GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

- name: Skip if bot
  if: steps.check-bot.outputs.is_bot != 'true'
  run: echo "Running for human users"
```

## Workflow Triggers

### Avoid Spammy Triggers
```yaml
# BAD: Runs on every issue edit (very noisy)
on:
  issues:
    types: [opened, edited, reopened, labeled, unlabeled]

# GOOD: Limit to meaningful events
on:
  issues:
    types: [opened, reopened]
```

### Duplicate Detection Thresholds
```yaml
# BAD: Low threshold causes false positives
similarity_threshold: 0.5

# GOOD: Higher threshold for accuracy
similarity_threshold: 0.65
```

## Required Workflow Patterns

### Use Latest Action Versions
```yaml
# BAD: Old major version
- uses: actions/checkout@v3

# GOOD: Latest stable version
- uses: actions/checkout@v4
```

### Pin Actions with SHA for Security-Critical Workflows
```yaml
# For security-sensitive workflows, pin to commit SHA
- uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11  # v4.1.1
```

### Error Handling
```yaml
# GOOD: Continue on error with status check
- name: Run optional check
  id: optional-check
  continue-on-error: true
  run: npm run optional-task

- name: Report status
  if: steps.optional-check.outcome == 'failure'
  run: echo "Optional check failed, continuing..."
```

## Label and Automation Best Practices

### Timing for Label Enforcement
```yaml
# BAD: Enforce labels before auto-labeler runs
on: pull_request

jobs:
  enforce-labels:
    runs-on: ubuntu-latest
    steps:
      - name: Check labels  # May fail before auto-labeler adds labels
        run: ...

# GOOD: Wait for labeler or run on labeled event
on:
  pull_request:
    types: [labeled, synchronize]

jobs:
  enforce-labels:
    # Now runs after labels are applied
```

## Testing Requirements

### Workflow Validation
- Ensure workflows pass `actionlint`
- Test workflows in a fork before merging
- Verify secrets are not exposed in logs

### Required Checks
```yaml
# Include these checks for Rust projects
jobs:
  ci:
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo audit
```

## Documentation

### Required Comments
```yaml
# Document non-obvious triggers or conditions
on:
  workflow_dispatch:  # Allow manual trigger for emergency releases
  push:
    branches: [main]
    paths:
      - 'src/**'  # Only run on source changes, not docs
```

### Describe Complex Conditionals
```yaml
# Run only for human users on feature branches
if: |
  github.actor != 'dependabot[bot]' &&
  github.actor != 'github-actions[bot]' &&
  startsWith(github.ref, 'refs/heads/feature/')
```
