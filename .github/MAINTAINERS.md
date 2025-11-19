# Maintainer Guide

Welcome to the cmdai maintainer team! This guide provides workflows, best practices, and processes for maintaining the project and reviewing contributions.

## Table of Contents

- [Maintainer Responsibilities](#maintainer-responsibilities)
- [Reviewing Pull Requests](#reviewing-pull-requests)
- [Documentation PR Reviews](#documentation-pr-reviews)
- [Updating "What's Being Built"](#updating-whats-being-built)
- [Contributor Recognition](#contributor-recognition)
- [Weekly Update Process](#weekly-update-process)
- [Release Management](#release-management)
- [Community Management](#community-management)

## Maintainer Responsibilities

As a cmdai maintainer, you help ensure:

1. **Code Quality** - Review PRs for correctness, style, and test coverage
2. **Documentation** - Keep docs accurate and up-to-date
3. **Community** - Foster a welcoming, inclusive environment
4. **Transparency** - Keep "What's Being Built" current
5. **Recognition** - Celebrate contributor achievements
6. **Security** - Address security issues promptly
7. **Releases** - Manage versioning and releases

## Reviewing Pull Requests

### General Review Process

1. **Initial Triage** (within 24 hours)
   - Label the PR appropriately
   - Check if CI passes
   - Verify PR template is complete
   - Add to project board if applicable

2. **Technical Review**
   - Does the code solve the stated problem?
   - Are tests included and passing?
   - Is the code well-structured and documented?
   - Does it follow project conventions?
   - Are there any security concerns?

3. **Testing**
   - Checkout the branch locally
   - Run tests: `cargo test`
   - Try the feature manually if applicable
   - Check for edge cases

4. **Feedback**
   - Be specific and constructive
   - Explain the "why" behind suggestions
   - Acknowledge good work
   - Use GitHub's suggestion feature for small fixes

5. **Approval and Merge**
   - Ensure all CI checks pass
   - Verify changelog/docs are updated
   - Use "Squash and merge" for feature PRs
   - Use "Rebase and merge" for documentation fixes
   - Update the PR description if needed

### Code Review Checklist

- [ ] PR has a clear description of changes
- [ ] Code follows Rust best practices
- [ ] Tests are included and pass
- [ ] Documentation is updated
- [ ] No new clippy warnings
- [ ] Code is formatted with `cargo fmt`
- [ ] Commit messages follow conventions
- [ ] No merge conflicts
- [ ] CI/CD pipeline passes
- [ ] Security implications considered

### Review Response Times

**Target response times:**
- Initial triage: 24 hours
- First review: 48 hours
- Follow-up reviews: 24 hours
- Final approval: 24 hours after last change

If you can't meet these timelines, communicate with the contributor.

## Documentation PR Reviews

Documentation PRs have a simpler review process:

### Documentation Review Checklist

- [ ] Content is accurate and up-to-date
- [ ] Markdown is properly formatted
- [ ] Links work correctly
- [ ] Code examples are complete and tested
- [ ] Spelling and grammar are correct
- [ ] Images (if any) are optimized
- [ ] Follows documentation style guide
- [ ] Navigation/SUMMARY.md updated if needed

### Testing Documentation Changes

```bash
cd book/
mdbook serve
```

Then verify:
- All pages render correctly
- Navigation works
- Code blocks have syntax highlighting
- Links are not broken
- Images display properly

### Documentation-Only PR Fast Track

For documentation-only PRs from trusted contributors:
1. Quick review (same day if possible)
2. Test rendering locally
3. Merge if no issues
4. Deploy automatically via GitHub Actions

## Updating "What's Being Built"

The [What's Being Built](../book/src/community/active-development.md) page is our transparency dashboard. Keep it current!

### When to Update

Update this page when:
- New feature branch is created
- PR is opened
- PR status changes (review, changes requested, approved)
- Work is completed and merged
- New help wanted items identified
- Weekly contributor highlights

### How to Update

1. **Open the file:**
   ```bash
   vim book/src/community/active-development.md
   ```

2. **Update the relevant section:**
   - **Active Feature Branches** - Add/update feature work
   - **Active Pull Requests** - Update PR status
   - **Documentation Work** - Note doc improvements
   - **Bug Fixes in Progress** - Track bug work
   - **Help Wanted** - Add tasks needing help

3. **Use status emojis:**
   - 🌱 Starting (< 25% complete)
   - 🔨 In Development (25-75% complete)
   - 🧪 Testing (75-95% complete)
   - 📝 Documentation (95-100%, docs pending)
   - ✅ Complete (merged or shipped)

4. **Include contributor names:**
   Always credit contributors by GitHub username.

5. **Add metrics:**
   Update the weekly statistics section.

### Example Entry

```markdown
#### Enhanced Safety Validation System

**Status:** 🔨 In Development (60% complete)
**Contributors:** @username, @contributor2
**Branch:** `feature/enhanced-safety`
**PR:** #123

Expanding the safety validation system with:
- ✅ Pattern-based command analysis
- 🔨 Machine learning risk scoring
- 📝 User-defined safety rules
- 📝 Audit logging

**Help Wanted:** Testing on different shell configurations
```

## Contributor Recognition

Recognition is crucial for community health. Make contributors feel valued!

### When to Recognize Contributors

- **First Contribution** - Welcome them publicly
- **Significant PR** - Highlight in "What's Being Built"
- **Ongoing Work** - Feature in weekly updates
- **Milestone Achievements** - Celebrate in showcase
- **Monthly** - Feature top contributors

### Recognition Workflow

1. **Immediate Recognition (on PR merge)**
   - Thank contributor in merge commit
   - Add to "What's Being Built" highlights
   - Update contributor showcase if needed

2. **Add to Contributor Showcase**

   Edit `book/src/community/contributors.md`:

   ```markdown
   #### @username

   **Role:** Code Contributor
   **Contributions:** Enhanced safety validation system
   **First Contribution:** Nov 2025
   **Notable Work:** Implemented ML-based risk scoring (#123)
   ```

3. **Weekly Highlights**

   In "What's Being Built", add to weekly section:

   ```markdown
   ### Week of Nov 11-17, 2025

   **🌟 Top Contributors:**
   - @username - Enhanced safety validation (#123)
   - @contributor2 - Documentation improvements (#124)
   - @contributor3 - Bug fixes (#125)
   ```

4. **Social Media Recognition**

   Tweet/post about significant contributions (with permission).

### Recognition Categories

Track contributors in these categories:
- **Core Contributors** - Regular significant contributions
- **Code Contributors** - Code improvements
- **Documentation Champions** - Doc contributions
- **Design Contributors** - UI/UX work
- **Testing & QA** - Testing and bug reports
- **Community Leaders** - Helping others

## Weekly Update Process

Every Monday, perform these updates:

### 1. Review Activity (15 minutes)

```bash
# Review last week's merged PRs
gh pr list --state merged --search "merged:>=2025-11-11"

# Review open PRs
gh pr list --state open

# Review new issues
gh issue list --created ">=2025-11-11"
```

### 2. Update "What's Being Built" (20 minutes)

- Update PR statuses
- Add new work
- Mark completed items
- Update help wanted
- Add weekly highlights
- Update statistics

### 3. Update Contributor Showcase (10 minutes)

- Add new contributors
- Update contribution counts
- Recognize monthly achievements

### 4. Triage Issues and PRs (15 minutes)

- Label new issues
- Assign issues to milestones
- Close stale issues/PRs
- Update project boards

### 5. Community Engagement (10 minutes)

- Respond to discussions
- Welcome new community members
- Share updates on social media

### Weekly Update Template

```markdown
# Weekly Update - Week of [DATE]

## Highlights

- [Major achievement 1]
- [Major achievement 2]
- [Major achievement 3]

## Merged This Week

- #123 - [Description] by @contributor
- #124 - [Description] by @contributor

## In Progress

- #125 - [Description] - 60% complete
- #126 - [Description] - 80% complete

## Top Contributors

1. @contributor1 - [Contribution summary]
2. @contributor2 - [Contribution summary]
3. @contributor3 - [Contribution summary]

## Looking Ahead

- [Upcoming feature]
- [Planned improvement]
- [Help wanted areas]
```

## Release Management

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- **Major (1.0.0)** - Breaking changes
- **Minor (0.1.0)** - New features, backward compatible
- **Patch (0.0.1)** - Bug fixes

### Release Process

1. **Prepare Release** (2-3 days before)
   - Update `CHANGELOG.md`
   - Update version in `Cargo.toml`
   - Update documentation
   - Run full test suite
   - Build release binaries

2. **Create Release PR**
   - Title: "Release v0.X.0"
   - Include changelog
   - Tag reviewers

3. **After Merge**
   - Create GitHub release
   - Tag the commit
   - Upload binaries
   - Announce on discussions
   - Share on social media

4. **Post-Release**
   - Monitor for issues
   - Update "What's Being Built"
   - Thank contributors
   - Start next milestone

### Release Checklist

- [ ] All planned features merged
- [ ] All tests passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Release notes written
- [ ] Binaries built and tested
- [ ] GitHub release created
- [ ] Git tag created
- [ ] Announcement posted
- [ ] Social media updated

## Community Management

### Fostering a Welcoming Community

1. **Respond Promptly**
   - Acknowledge new issues/PRs quickly
   - Answer questions within 24-48 hours
   - Be patient and helpful

2. **Be Inclusive**
   - Use inclusive language
   - Welcome all skill levels
   - Encourage diverse perspectives
   - Follow Code of Conduct

3. **Provide Guidance**
   - Point to relevant documentation
   - Suggest good first issues
   - Offer mentorship when possible

4. **Recognize Effort**
   - Thank people for contributions
   - Celebrate milestones
   - Feature contributors

### Handling Difficult Situations

**Unhelpful Issues/PRs:**
- Politely ask for more information
- Provide templates and examples
- Close if no response after 7 days

**Code of Conduct Violations:**
- Address privately first if minor
- Document the violation
- Consult other maintainers
- Follow enforcement guidelines

**Burnout Prevention:**
- Set boundaries
- Share maintainer duties
- Take breaks when needed
- Ask for help

## Tools and Scripts

### Useful GitHub CLI Commands

```bash
# List open PRs needing review
gh pr list --search "review:required"

# List stale issues (no activity in 30 days)
gh issue list --search "updated:<2025-10-11"

# View PR details
gh pr view 123

# Check PR status
gh pr checks 123

# Add label to issue
gh issue edit 123 --add-label "good first issue"
```

### Automation Scripts

Consider creating scripts for:
- Generating weekly reports
- Updating contributor lists
- Checking documentation links
- Running comprehensive tests

## Getting Help as a Maintainer

- **Maintainer Discussions** - Private channel for maintainers
- **Documentation** - Refer to [DOCUMENTATION.md](../DOCUMENTATION.md)
- **Ask Other Maintainers** - Don't hesitate to ask questions
- **Community Input** - Involve the community in decisions

## Resources

- [GitHub Maintainer Guide](https://opensource.guide/best-practices/)
- [Rust Code Review Guidelines](https://rust-lang.github.io/rfcs/)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)

---

**Thank you for being a cmdai maintainer!** Your work keeps the project healthy and the community thriving.

For questions or suggestions about this guide, open a [GitHub Discussion](https://github.com/wildcard/cmdai/discussions).
