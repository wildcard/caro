# GitHub Labels Implementation Plan

**Status:** Ready for execution
**Priority:** High
**Owner:** DevRel Team
**Timeline:** 1 week

---

## Overview

Implement comprehensive GitHub label system across the Caro repository to organize contributions by lane, difficulty, type, and platform.

**Reference:** [.github/LABELS.md](../../.github/LABELS.md)

---

## Phase 1: Automated Label Creation (Day 1)

### Action Items

1. **Extract Label Creation Script**
   ```bash
   # From .github/LABELS.md, extract the gh label create commands
   # Save to scripts/create-labels.sh
   ```

2. **Create Script File**
   - File: `scripts/create-labels.sh`
   - Make executable: `chmod +x scripts/create-labels.sh`
   - Add error handling for existing labels

3. **Test in Dry-Run Mode**
   ```bash
   # First, list existing labels to avoid conflicts
   gh label list --limit 100

   # Test creation (manual verification recommended)
   # Run subset first, then full script
   ```

4. **Execute Label Creation**
   ```bash
   cd /home/user/caro
   ./scripts/create-labels.sh
   ```

### Expected Output

Total labels created: **~55 labels**

#### Lane Labels (6)
- `lane/security` (Red #d73a4a)
- `lane/runtime` (Blue #0075ca)
- `lane/inference` (Purple #7057ff)
- `lane/ux` (Light Blue #a2eeef)
- `lane/ecosystem` (Green #008672)
- `lane/distribution` (Pink #d876e3)

#### Difficulty Labels (4)
- `good-first-issue` (Purple #7057ff)
- `first-time-contributor` (Green #0e8a16)
- `advanced` (Red #b60205)
- `expert` (Dark Red #d93f0b)

#### Priority Labels (4)
- `critical` (Red #b60205)
- `high-priority` (Orange #d93f0b)
- `medium-priority` (Yellow #fbca04)
- `low-priority` (Green #0e8a16)

#### Type Labels (7)
- `bug`, `enhancement`, `documentation`, `refactor`
- `performance`, `testing`, `tooling`

#### Technical Area Labels (8)
- `safety`, `backend`, `agent`, `ui`, `cli`
- `config`, `cache`, `execution`

#### Technology Labels (6)
- `tokio`, `ratatui`, `mcp`, `mlx`, `candle`, `packaging`

#### Platform Labels (4)
- `macos`, `linux`, `windows`, `cross-platform`

#### Community Labels (8)
- `help-wanted`, `question`, `discussion`, `wontfix`
- `duplicate`, `conduct`, `hacktoberfest`, `good-for-demo`

#### Special Labels (4)
- `caro-the-shiba`, `lane-lead-application`
- `in-progress`, `blocked`, `needs-review`, `needs-testing`, `needs-docs`

---

## Phase 2: Retroactive Labeling (Days 2-3)

### 2.1 Audit Existing Issues

```bash
# Get all open issues
gh issue list --limit 100 --state open

# Export to CSV for analysis
gh issue list --limit 100 --state open --json number,title,labels > issues.json
```

### 2.2 Label Assignment Strategy

#### High Priority Issues (First)
1. **Security issues** → `lane/security` + severity label
2. **Bug reports** → `bug` + affected lane + priority
3. **Feature requests** → `enhancement` + target lane

#### Medium Priority
4. **Documentation** → `documentation` + `good-first-issue` (if applicable)
5. **Performance** → `performance` + `lane/inference` or `lane/runtime`
6. **Testing** → `testing` + relevant lane

#### Low Priority
7. **General questions** → `question`
8. **Discussions** → `discussion`

### 2.3 Labeling Workflow

**For each existing issue:**

1. **Read issue description**
2. **Determine lane** (Security, Runtime, Inference, UX, Ecosystem, Distribution)
3. **Assess difficulty** (good-first-issue, advanced, expert)
4. **Assign type** (bug, enhancement, documentation, etc.)
5. **Add technical area** (safety, backend, agent, etc.)
6. **Set priority** (critical, high, medium, low)
7. **Platform** (if platform-specific)

**Example:**
```
Issue: "Add Windows PowerShell dangerous pattern detection"

Labels:
- lane/security
- good-first-issue
- enhancement
- safety
- windows
- medium-priority
```

### 2.4 Automation Script

Create `scripts/label-existing-issues.sh`:

```bash
#!/bin/bash
# Semi-automated labeling based on keywords

# Security keywords → lane/security
gh issue list --search "security OR safety OR dangerous" --json number --limit 50 | \
  jq -r '.[].number' | \
  xargs -I {} gh issue edit {} --add-label "lane/security"

# Performance keywords → lane/inference + performance
gh issue list --search "performance OR speed OR optimization" --json number --limit 50 | \
  jq -r '.[].number' | \
  xargs -I {} gh issue edit {} --add-label "lane/inference,performance"

# MCP keywords → lane/ecosystem + mcp
gh issue list --search "MCP OR integration OR plugin" --json number --limit 50 | \
  jq -r '.[].number' | \
  xargs -I {} gh issue edit {} --add-label "lane/ecosystem,mcp"

# Continue for other common patterns...
```

---

## Phase 3: Label Existing Issues in HELP_WANTED.md (Day 4)

### 3.1 Create Issues from HELP_WANTED.md

For each "First Issue" in HELP_WANTED.md, create GitHub issue:

#### Security Lane Issues

**Issue 1: Policy Engine MVP**
```bash
gh issue create \
  --title "Policy engine MVP - Deny destructive commands unless explicit override" \
  --label "lane/security,critical,enhancement,advanced" \
  --body "$(cat <<'EOF'
## Overview
Implement minimum viable policy engine that denies destructive commands unless explicit user override + typed confirmation.

## Deliverables
- [ ] Policy rule format design
- [ ] Deny rule implementation for rm -rf, mkfs, dd
- [ ] Typed confirmation flow
- [ ] Test coverage for policy violations

## Reference
See HELP_WANTED.md - Security Lane, Deliverable #3

## Skills Needed
- Rust
- Security awareness
- Policy engine design

## Lane
Security Lane - Critical Priority
EOF
)"
```

**Issue 2: Risk Scoring System**
```bash
gh issue create \
  --title "Risk scoring system - Tag commands by risk type" \
  --label "lane/security,good-first-issue,enhancement,safety" \
  --body "..."
```

**Issue 3: CWD Hard-Binding**
```bash
gh issue create \
  --title "CWD hard-binding - Prevent traversal outside allowed roots" \
  --label "lane/security,critical,advanced,safety" \
  --body "..."
```

#### Repeat for all lanes...

### 3.2 Label Issues from .github/first-time-issues/

For each of the 10 first-time issues:

```bash
# Issue 1: Shiba ASCII Art
gh issue create \
  --title "🐕 Add Shiba-themed ASCII Art to Welcome Message" \
  --label "good-first-issue,first-time-contributor,lane/ux,ui,caro-the-shiba" \
  --body-file .github/first-time-issues/01-shiba-ascii-art.md

# Issue 2: Windows PowerShell Safety
gh issue create \
  --title "🛡️ Expand Dangerous Command Patterns for Windows PowerShell" \
  --label "good-first-issue,first-time-contributor,lane/security,safety,windows" \
  --body-file .github/first-time-issues/02-windows-powershell-safety.md

# ... continue for all 10 issues
```

---

## Phase 4: Documentation Updates (Day 5)

### 4.1 Update CONTRIBUTING.md

Add section about labels:

```markdown
### Using Labels

When creating or working on issues, use labels to categorize:

**Lane Labels** (pick ONE):
- `lane/security` - Safety, guardrails, red-team testing
- `lane/runtime` - Tokio, streaming, backend orchestration
- `lane/inference` - Performance, quantization, benchmarks
- `lane/ux` - Ratatui, TUI design, confirmations
- `lane/ecosystem` - MCP, IDE integration, plugins
- `lane/distribution` - Packaging, signing, offline bundles

**Difficulty**:
- `good-first-issue` - Beginner-friendly
- `first-time-contributor` - Extra support provided
- `advanced` - Requires significant experience
- `expert` - Expert-level contribution

See [.github/LABELS.md](/.github/LABELS.md) for complete label guide.
```

### 4.2 Update Issue Templates

Ensure all `.github/ISSUE_TEMPLATE/*.yml` files include appropriate default labels:

```yaml
labels: ["lane/security", "help-wanted"]  # Example for security lane template
```

### 4.3 Create PR Template Section

Add to `.github/PULL_REQUEST_TEMPLATE.md`:

```markdown
## Labels

Please add appropriate labels:
- [ ] Lane label (lane/*)
- [ ] Difficulty (if closing good-first-issue)
- [ ] Type (bug, enhancement, etc.)
- [ ] Platform (if platform-specific)
```

---

## Phase 5: Community Communication (Day 6-7)

### 5.1 Announcement Post

Create GitHub Discussion:

**Title:** "New Contribution Lane System - Find Your Perfect Issue"

**Body:**
```markdown
We've reorganized our issue tracker into **6 contribution lanes** to help you find issues that match your skills and interests! 🎯

## The Six Lanes

1. **Security Lane** - Guardrails, red-team testing, safety validation
2. **Runtime Lane** - Tokio, async Rust, backend orchestration
3. **Inference Lane** - Performance optimization, quantization
4. **UX Lane** - Ratatui, TUI design, beautiful terminals
5. **Ecosystem Lane** - MCP, IDE integration, plugins
6. **Distribution Lane** - Packaging, signing, offline bundles

## How to Use Lanes

**Find issues by lane:**
- Browse by label: `lane/security`, `lane/runtime`, etc.
- Check [HELP_WANTED.md](HELP_WANTED.md) for detailed lane info

**Contribute:**
1. Pick your lane
2. Find a `good-first-issue` or `advanced` issue
3. Comment "I'd like to work on this"
4. Get assigned and start coding!

**Want to lead a lane?**
- See [lane lead application template](/.github/ISSUE_TEMPLATE/lane_lead_application.yml)
- We're actively recruiting lane leads!

## Quick Links

- [HELP_WANTED.md](HELP_WANTED.md) - Detailed lane documentation
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution workflow
- [RECRUITING.md](RECRUITING.md) - For maintainers recruiting contributors

Happy contributing! 🚀
```

### 5.2 Social Media Posts

**Twitter/X:**
```
We've launched 6 contribution lanes for @CaroShell! 🎯

Find issues that match YOUR skills:
🔐 Security - Guardrails & red-team testing
⚡ Runtime - Tokio & async Rust
🚀 Inference - Performance optimization
🎨 UX - Beautiful terminal UI
🔌 Ecosystem - MCP & IDE integration
📦 Distribution - Packaging & signing

Explore: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md

#Rust #OpenSource #DevRel
```

**Reddit /r/rust:**
```
**New Contribution Lanes for Caro - Find Your Perfect Issue**

We've organized the Caro project into 6 contribution lanes to help contributors find issues matching their expertise.

If you love: Tokio and async Rust → Runtime Lane
If you love: Performance tuning and quantization → Inference Lane
If you love: MCP and IDE integration → Ecosystem Lane

Each lane has good-first-issue through expert-level tasks.

Check it out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md
```

### 5.3 Blog Post (Website)

Create `website/src/pages/blog/contribution-lanes.astro`:

**Title:** "Introducing Contribution Lanes - A Better Way to Contribute"

**Outline:**
1. Problem: Hard to find the right issue
2. Solution: 6 specialized lanes
3. Benefits: Match skills, clear ownership, lane leads
4. How to get started
5. Call to action: Pick your lane!

---

## Phase 6: Maintenance & Iteration (Ongoing)

### 6.1 Weekly Label Hygiene

**Every Monday:**
1. Review unlabeled issues (should be none)
2. Check for mislabeled issues
3. Ensure all PRs have appropriate labels
4. Update label usage metrics

**Script:** `scripts/label-health-check.sh`

```bash
#!/bin/bash
# Check for issues without lane labels

echo "Issues without lane labels:"
gh issue list --label "!lane/security,!lane/runtime,!lane/inference,!lane/ux,!lane/ecosystem,!lane/distribution" --limit 50

echo "\nIssues with multiple lane labels (should investigate):"
# TODO: Script to detect multiple lane/* labels

echo "\nRecently closed issues (verify labels were useful):"
gh issue list --state closed --limit 10 --json number,title,labels
```

### 6.2 Monthly Label Review

**First Monday of each month:**
1. Generate label usage report
2. Identify unused labels (consider removing)
3. Identify missing labels (consider adding)
4. Update .github/LABELS.md if needed

### 6.3 Metrics to Track

**Label Usage:**
- Issues per lane (should be relatively balanced)
- good-first-issue claim rate
- Time from issue creation to first label
- Average labels per issue

**Contributor Engagement:**
- First-time contributors using good-first-issue labels
- Contributors working across multiple lanes
- Lane lead recruitment success

**Example Dashboard:**
```
Lane          | Open Issues | Contributors | Avg Time to Close
--------------|-------------|--------------|------------------
Security      | 12          | 3            | 14 days
Runtime       | 8           | 2            | 21 days
Inference     | 15          | 4            | 10 days
UX            | 6           | 1            | 30 days
Ecosystem     | 10          | 2            | 18 days
Distribution  | 5           | 1            | 45 days
```

---

## Success Metrics

### Week 1
- [ ] All 55+ labels created
- [ ] 50%+ of existing issues labeled
- [ ] 10 first-time issues created with labels
- [ ] Announcement published

### Month 1
- [ ] 100% of issues have lane label
- [ ] 5+ good-first-issue claimed
- [ ] 1+ lane lead recruited
- [ ] Label usage documented in wiki

### Quarter 1
- [ ] Balanced distribution across lanes
- [ ] Lane leads for 3+ lanes
- [ ] 90%+ label accuracy (verified in reviews)
- [ ] Community using labels to find issues

---

## Rollback Plan

If label system causes confusion:

1. **Pause new labels** - Stop creating new labels
2. **Gather feedback** - Survey contributors about issues
3. **Simplify** - Reduce to essential labels only
4. **Document clearly** - Improve label documentation
5. **Re-launch** - Communicate changes to community

---

## Resources

**GitHub CLI Commands:**
```bash
# Create label
gh label create "label-name" --description "Description" --color "hexcolor"

# List labels
gh label list --limit 100

# Add label to issue
gh issue edit ISSUE_NUMBER --add-label "label-name"

# Remove label from issue
gh issue edit ISSUE_NUMBER --remove-label "label-name"

# Search issues by label
gh issue list --label "label-name"

# Search issues by multiple labels (AND)
gh issue list --label "label1" --label "label2"
```

**Useful Label Queries:**

```
# All security lane good first issues
label:lane/security label:good-first-issue

# Critical priority issues without lane
label:critical -label:lane/security -label:lane/runtime ...

# All first-time contributor issues
label:first-time-contributor
```

---

## Questions?

Contact: DevRel team via GitHub Discussions

**Last Updated:** January 1, 2025
