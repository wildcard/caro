# GitHub Infrastructure Import Guide

> **Step-by-step instructions for setting up all GitHub infrastructure for cmdai launch**

---

## Overview

This guide helps you import:
1. GitHub Labels (29 labels)
2. GitHub Milestones (5 milestones)
3. Initial Issues (15 V1.0 issues)
4. Issue Templates (already in `.github/ISSUE_TEMPLATE/`)

**Time required**: ~30 minutes

---

## Step 1: Import Labels

### Option A: Using GitHub CLI (Recommended)

```bash
# Install GitHub CLI if not already installed
# macOS: brew install gh
# Linux: https://github.com/cli/cli/blob/trunk/docs/install_linux.md

# Authenticate
gh auth login

# Navigate to project directory
cd /home/user/cmdai

# Import labels from JSON
cat launch/GITHUB_LABELS.json | jq -r '.[] | @json' | while read label; do
  name=$(echo $label | jq -r '.name')
  color=$(echo $label | jq -r '.color')
  description=$(echo $label | jq -r '.description')

  gh label create "$name" \
    --color "$color" \
    --description "$description" \
    --repo wildcard/cmdai || echo "Label $name already exists"
done
```

### Option B: Using GitHub Web UI

1. Go to https://github.com/wildcard/cmdai/labels
2. Click **"New label"** for each label
3. Copy from `launch/GITHUB_LABELS.json`:
   - Name
   - Color (without `#`)
   - Description

**Total labels to create**: 29

**Label Categories**:
- Type labels (6): feature, bug, docs, refactor, test, infra
- Priority labels (4): P0, P1, P2, P3
- Component labels (7): cli, backend, safety, cloud, enterprise, workflows, integrations
- Phase labels (6): mvp, v1.0, q1-2025, q2-2025, q3-2025, q4-2025
- Other labels (6): good-first-issue, help-wanted, blocked, needs-design, needs-review, breaking-change

### Option C: Using a Script (Fastest)

Save this script as `import_labels.sh`:

```bash
#!/bin/bash

# Import GitHub labels from JSON file
# Usage: ./import_labels.sh

set -e

REPO="wildcard/cmdai"
LABELS_FILE="launch/GITHUB_LABELS.json"

if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed"
    echo "Install: brew install gh (macOS) or see https://cli.github.com/"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "Error: jq is not installed"
    echo "Install: brew install jq (macOS) or apt install jq (Linux)"
    exit 1
fi

echo "Importing labels to $REPO..."

cat "$LABELS_FILE" | jq -c '.[]' | while read -r label; do
    name=$(echo "$label" | jq -r '.name')
    color=$(echo "$label" | jq -r '.color')
    description=$(echo "$label" | jq -r '.description')

    echo "Creating label: $name"
    gh label create "$name" \
        --color "$color" \
        --description "$description" \
        --repo "$REPO" 2>&1 | grep -v "already exists" || true
done

echo "✅ Labels imported successfully!"
```

Run it:
```bash
chmod +x import_labels.sh
./import_labels.sh
```

---

## Step 2: Create Milestones

### Using GitHub Web UI

1. Go to https://github.com/wildcard/cmdai/milestones
2. Click **"New milestone"**
3. For each milestone in `launch/GITHUB_MILESTONES.md`:
   - Copy the **Title**
   - Set the **Due Date**
   - Copy the **Description** (full markdown)
   - Click **"Create milestone"**

**Milestones to create**:
1. `v1.0 - Production CLI` (Due: December 31, 2025)
2. `Q1 2025 - Cloud Launch` (Due: March 31, 2025)
3. `Q2 2025 - Enterprise Features` (Due: June 30, 2025)
4. `Q3 2025 - Platform` (Due: September 30, 2025)
5. `Q4 2025 - Scale & Fundraise` (Due: December 31, 2025)

### Using GitHub CLI

```bash
# Create v1.0 milestone
gh api repos/wildcard/cmdai/milestones \
  -f title="v1.0 - Production CLI" \
  -f state="open" \
  -f description="$(cat launch/GITHUB_MILESTONES.md | sed -n '/## Milestone 1/,/^---$/p' | tail -n +3 | head -n -1)" \
  -f due_on="2025-12-31T23:59:59Z"

# Repeat for other milestones...
# (Note: You may need to extract each milestone's description programmatically)
```

---

## Step 3: Create Initial Issues

### Process

For each of the 15 issues in `launch/GITHUB_ISSUES.md`:

1. Go to https://github.com/wildcard/cmdai/issues/new
2. Copy the **Title**
3. Copy the **Description** (everything under "Description:")
4. Add **Labels** (listed in each issue)
5. Set **Milestone** (listed in each issue)
6. (Optional) Add **Assignee** if you know who will work on it
7. Click **"Submit new issue"**

### Issues to Create

**P0 Issues (Critical)**:
1. #1: Performance Optimization - <100ms startup
2. #2: Binary Size Reduction - <50MB
3. #3: Homebrew Formula
4. #4: Publish to crates.io
5. #5: E2E Integration Testing
6. #7: Security Audit
7. #15: Launch Checklist

**P1 Issues (High Priority)**:
8. #6: Documentation Polish
9. #10: GitHub Release Automation
10. #11: Performance Benchmarking
11. #12: Debian Package
12. #13: Config Validation

**P2 Issues (Medium Priority)**:
13. #8: Shell Completions (good-first-issue)
14. #9: Man Pages (good-first-issue)
15. #14: Logging and Verbosity

### Using a Helper Script (Semi-automated)

```bash
#!/bin/bash
# create_issues.sh - Helper to create issues faster

REPO="wildcard/cmdai"

# Issue #1
gh issue create \
  --repo "$REPO" \
  --title "[v1.0] Optimize CLI startup time to <100ms" \
  --body-file <(sed -n '/^## Issue #1:/,/^---$/p' launch/GITHUB_ISSUES.md | tail -n +5 | head -n -1) \
  --label "type: feature,priority: P0,component: cli,phase: v1.0" \
  --milestone "v1.0 - Production CLI"

# Repeat for other issues...
```

**Note**: This requires manually extracting each issue's body. It's often faster to use the web UI for the initial 15 issues.

---

## Step 4: Verify Issue Templates

Issue templates are already in `.github/ISSUE_TEMPLATE/`:
- ✅ `bug_report.yml`
- ✅ `feature_request.yml`
- ✅ `backend_integration.yml`
- ✅ `safety_pattern.yml`
- ✅ `good_first_issue.yml` (newly added)

No action needed - these are already committed to the repository.

---

## Step 5: Set Up GitHub Projects (Optional)

### Create Project Board

1. Go to https://github.com/wildcard/cmdai/projects
2. Click **"New project"**
3. Choose **"Board"** template
4. Name: **"cmdai Roadmap 2025"**

### Add Columns

Create these columns:
1. **Backlog** - Not yet prioritized
2. **Ready** - Prioritized, ready to start
3. **In Progress** - Currently being worked on
4. **In Review** - PR submitted, needs review
5. **Done** - Completed and merged

### Add Custom Fields

1. **Quarter**: Single select
   - Options: Q1 2025, Q2 2025, Q3 2025, Q4 2025
2. **Effort**: Number
   - Story points: 1, 2, 3, 5, 8, 13
3. **Revenue Impact**: Single select
   - Options: None, Low, Medium, High, Critical

### Add Issues to Project

1. Click **"Add items"**
2. Search for issues by milestone: `milestone:"v1.0 - Production CLI"`
3. Bulk add all V1.0 issues
4. Drag to appropriate columns (most start in "Backlog" or "Ready")

---

## Step 6: Configure Repository Settings

### Enable Features

1. Go to https://github.com/wildcard/cmdai/settings
2. **Features**:
   - ✅ Issues
   - ✅ Projects
   - ✅ Discussions (enable for community Q&A)
   - ✅ Wiki (optional)

### Branch Protection

1. Go to **Settings → Branches**
2. Add rule for `main`:
   - ✅ Require pull request before merging
   - ✅ Require approvals (1)
   - ✅ Require status checks to pass (all CI tests)
   - ✅ Require branches to be up to date

### Discussion Categories

1. Go to https://github.com/wildcard/cmdai/discussions
2. Create categories:
   - **General**: General discussions
   - **Ideas**: Feature requests and ideas
   - **Q&A**: Questions and help
   - **Show and Tell**: Community projects
   - **Announcements**: Project announcements

---

## Step 7: Add Project Metadata

### Update README Badges

Add these badges to `README.md`:

```markdown
[![GitHub release](https://img.shields.io/github/v/release/wildcard/cmdai)](https://github.com/wildcard/cmdai/releases)
[![Crates.io](https://img.shields.io/crates/v/cmdai)](https://crates.io/crates/cmdai)
[![CI](https://github.com/wildcard/cmdai/workflows/CI/badge.svg)](https://github.com/wildcard/cmdai/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/YOUR_SERVER_ID)](https://discord.gg/YOUR_INVITE)
```

### Create CONTRIBUTING.md Link

Add to README:
```markdown
## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Good first issues**: [View beginner-friendly issues](https://github.com/wildcard/cmdai/issues?q=is%3Aissue+is%3Aopen+label%3A%22good-first-issue%22)
```

---

## Verification Checklist

After completing all steps, verify:

- [ ] **Labels**: 29 labels created in https://github.com/wildcard/cmdai/labels
- [ ] **Milestones**: 5 milestones created in https://github.com/wildcard/cmdai/milestones
- [ ] **Issues**: 15 V1.0 issues created in https://github.com/wildcard/cmdai/issues
- [ ] **Templates**: 5 issue templates available when creating new issues
- [ ] **Project**: Project board created with columns and custom fields
- [ ] **Settings**: Discussions enabled, branch protection configured
- [ ] **README**: Badges and contributing section updated

---

## Troubleshooting

### Problem: GitHub CLI authentication fails

**Solution**:
```bash
gh auth login
# Follow the prompts to authenticate
```

### Problem: Labels already exist

**Solution**: This is fine. The script will skip existing labels. If you need to update a label:
```bash
gh label edit "label-name" --color "new-color" --description "new description"
```

### Problem: jq not found

**Solution**:
```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt install jq

# Fedora
sudo dnf install jq
```

### Problem: Can't create milestones via CLI

**Solution**: Use the web UI. It's faster for one-time setup of 5 milestones.

---

## Next Steps

After importing everything:

1. **Prioritize issues**: Review the backlog and ensure P0 issues are in "Ready" column
2. **Assign issues**: Tag team members or contributors
3. **Start working**: Pick the first P0 issue and move it to "In Progress"
4. **Track progress**: Update the project board daily
5. **Triage new issues**: Label and assign new community issues within 48 hours

---

## Questions?

- GitHub Discussions: https://github.com/wildcard/cmdai/discussions
- Project maintainers: Tag @maintainer-username in issues

---

**Ready to launch! 🚀**
