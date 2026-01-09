# Releasing the Next Beta Iteration

**Purpose**: This document provides step-by-step instructions for releasing subsequent beta iterations (e.g., v1.1.0-beta.2, v1.1.0-beta.3) after receiving feedback from the current beta testing cycle.

**When to Use**: After completing a beta testing cycle and fixing bugs/issues identified by beta testers.

---

## 📋 Prerequisites

Before releasing the next beta iteration, ensure:

- [ ] Current beta testing cycle completed (5 days)
- [ ] Feedback collected from all beta testers
- [ ] Bugs triaged and prioritized (P0/P1/P2/P3)
- [ ] Critical (P0) and high-priority (P1) bugs fixed
- [ ] All tests passing: `cargo test --all-features`
- [ ] QA validation run: Beta test suite passing
- [ ] Changes documented in CHANGELOG.md

---

## 🔄 Beta Iteration Release Process

### Step 1: Triage Beta Testing Feedback

**Review Feedback Sources**:
```bash
# Check GitHub issues with beta-testing label
gh issue list --label "beta-testing" --state open

# Review beta tester survey responses
# Review telemetry exports (if provided)
# Review daily check-in notes
```

**Categorize Issues**:

| Priority | Description | Action |
|----------|-------------|--------|
| **P0** | Critical bugs, crashes, data loss, security issues | MUST fix before next beta |
| **P1** | High-impact bugs, major UX issues, broken features | SHOULD fix before next beta |
| **P2** | Medium bugs, minor UX issues, edge cases | Consider for next beta |
| **P3** | Low-priority, cosmetic, nice-to-haves | Defer to GA or later |

**Decision Gate**: Fix all P0 bugs, majority of P1 bugs before next iteration.

### Step 2: Fix Identified Issues

**Create Fix Branches** (if needed):
```bash
# Stay on release/v1.1.0 branch or create fix branches
git checkout release/v1.1.0

# For each P0/P1 bug, create commits with clear messages
git commit -m "fix(safety): Prevent false positive on safe command X

- Issue: Beta tester reported safe command blocked
- Root cause: Pattern too broad in safety validation
- Fix: Refined pattern to be more specific
- Tests: Added regression test in tests/regression_beta_issue_X.rs

Reported by: @beta-tester-username
Related to: #issue-number
"
```

**Test Each Fix**:
```bash
# Run all tests
cargo test --all-features

# Run specific regression test
cargo test test_beta_issue_X

# Test manually with the original failing case
./target/release/caro "the command that failed"
```

**Track Progress**:
- Create `.claude/beta-testing/beta-2-fixes.md` to document all fixes
- Link each fix to the original bug report
- Note which tester reported each issue

### Step 3: Update Version Number

**Increment Beta Version**:

Edit `Cargo.toml`:
```toml
[package]
name = "caro"
version = "1.1.0-beta.2"  # Increment beta number
# ... rest of config
```

**Update Version in Code** (if needed):
```bash
# Build to regenerate version info
cargo build --release
```

**Verify Version**:
```bash
./target/release/caro --version
# Should show: caro 1.1.0-beta.2 (commit-hash date)
```

### Step 4: Update CHANGELOG.md

Add new beta iteration entry:

```markdown
## [1.1.0-beta.2] - 2026-01-XX

### 🐛 Fixes from Beta.1 Testing

**Critical Fixes (P0)**:
- Fixed crash when using empty query (reported by @tester1)
- Fixed false positive on safe commands with "remove" keyword (reported by @tester2)
- Fixed memory leak in telemetry uploader (reported by @tester3)

**High-Priority Fixes (P1)**:
- Improved command generation for file size queries (reported by @tester1)
- Fixed platform detection on macOS Intel (reported by @tester4)
- Enhanced error messages for invalid queries (reported by @tester2)

**Improvements**:
- Added 5 new static patterns based on common tester queries
- Improved safety pattern specificity (reduced false negative risk)
- Enhanced telemetry PII validation (additional regex patterns)

**Testing**:
- All 146 library tests passing
- Beta test suite: 56/58 passing (96.6% pass rate, up from 93.1%)
- 0% false positive rate maintained
- No regressions detected

**Known Issues Remaining**:
- Minor: Command variations for some queries (P2)
- Minor: MLX build requires cmake (P3, documentation issue)

**Beta Testing Acknowledgments**:
- @tester1 - 12 bug reports, excellent daily check-ins
- @tester2 - Found critical safety validation issue
- @tester3 - Discovered memory leak, provided detailed telemetry
- @tester4 - Extensive platform testing
- @tester5 - UX feedback and documentation improvements

---
```

### Step 5: Update Documentation

**Update Release Notes Template**:

Edit `.claude/releases/github-release-notes.md` with beta.2 information:
- Add "Changes from beta.1" section
- Update metrics (pass rate, bug counts)
- Add acknowledgments for beta.1 testers
- Update known issues list

**Update Deployment Status**:

Edit `.claude/releases/BETA-DEPLOYMENT-STATUS.md`:
```markdown
**Last Updated**: 2026-01-XX
**Beta Version**: v1.1.0-beta.2
**Status**: 🔄 BETA ITERATION 2 - READY FOR TESTING
```

**Update Beta Testing Instructions** (if needed):
- Add any new features to test
- Add specific regression tests for fixed bugs
- Update known issues list

### Step 6: Build and Test

**Clean Build**:
```bash
# Clean previous builds
cargo clean

# Build release binary
cargo build --release

# Time the build
time cargo build --release
```

**Comprehensive Testing**:
```bash
# All library tests
cargo test --all-features

# Beta test suite
cargo test --test beta_test_suite

# Safety validation tests
cargo test --test safety_validator_contract

# Platform-specific tests
cargo test --test platform_detection_contract

# Regression tests for beta.1 issues
cargo test --test regression_beta_issue_*
```

**Manual Verification**:
```bash
# Test each fixed bug manually
./target/release/caro "command that previously failed"

# Test new features (if any)
./target/release/caro assess
./target/release/caro doctor

# Verify telemetry
./target/release/caro telemetry status
```

### Step 7: Commit Changes

**Commit Version Bump**:
```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(release): Bump version to 1.1.0-beta.2

- Incremented beta version for second iteration
- All P0 and P1 bugs from beta.1 fixed
- Pass rate improved: 93.1% → 96.6%
- 0% false positive rate maintained

Beta.1 testing complete:
- 5 testers, 5 days
- 15 bugs reported (8 P1, 5 P2, 2 P3)
- All P0/P1 bugs fixed

Ready for beta.2 testing cycle.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
"
```

**Commit Documentation Updates**:
```bash
git add CHANGELOG.md .claude/releases/
git commit -m "docs(release): Update documentation for v1.1.0-beta.2

- Updated CHANGELOG with beta.2 fixes
- Updated release notes with beta.1 results
- Updated deployment status
- Acknowledged beta.1 testers

Changes from beta.1:
- 8 P1 bugs fixed
- Pass rate: 93.1% → 96.6%
- Added 5 new static patterns

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
"
```

### Step 8: Create GitHub Release (Beta.2)

**Tag the Release**:
```bash
# Create annotated tag
git tag -a v1.1.0-beta.2 -m "Beta iteration 2 - Bug fixes from beta.1 testing"

# Push tag to GitHub
git push origin v1.1.0-beta.2

# Push branch updates
git push origin release/v1.1.0
```

**Create Pre-Release on GitHub**:
```bash
gh release create v1.1.0-beta.2 \
  --title "v1.1.0-beta.2 - Beta Iteration 2 (Bug Fixes)" \
  --notes-file .claude/releases/github-release-notes.md \
  --prerelease
```

**Prepare and Upload Binary**:
```bash
# Copy and rename binary
cp target/release/caro caro-1.1.0-beta.2-macos-aarch64
chmod +x caro-1.1.0-beta.2-macos-aarch64

# Generate checksum
shasum -a 256 caro-1.1.0-beta.2-macos-aarch64 > caro-1.1.0-beta.2-macos-aarch64.sha256

# Upload to release
gh release upload v1.1.0-beta.2 \
  caro-1.1.0-beta.2-macos-aarch64 \
  caro-1.1.0-beta.2-macos-aarch64.sha256

# Clean up
rm caro-1.1.0-beta.2-macos-aarch64 caro-1.1.0-beta.2-macos-aarch64.sha256
```

**Verify Release**:
```bash
gh release view v1.1.0-beta.2
```

### Step 9: Notify Beta Testers

**Send Update Email**:

```
Subject: caro v1.1.0-beta.2 Ready - Thank You for Beta.1 Feedback!

Hi [Tester Name],

Thank you for your excellent feedback during the beta.1 testing cycle! Your bug reports and suggestions have been invaluable.

🎉 BETA.2 IS READY

We've released v1.1.0-beta.2 with fixes for the issues you reported:

✅ Fixes from Beta.1:
- Fixed crash with empty queries (your report!)
- Fixed false positive on safe commands
- Improved command generation accuracy (93.1% → 96.6%)
- [List specific fixes relevant to this tester]

📦 Installation:
Download: https://github.com/wildcard/caro/releases/tag/v1.1.0-beta.2

# macOS Apple Silicon
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.2/caro-1.1.0-beta.2-macos-aarch64 -o caro
chmod +x caro
sudo mv caro /usr/local/bin/caro
caro --version  # Should show: caro 1.1.0-beta.2

🧪 Testing Focus:
- Verify your reported bugs are fixed
- Test for any regressions
- Continue daily workflow testing (3-5 more days)

📋 Specific Requests for You:
- [List specific areas for this tester to focus on based on their beta.1 feedback]

🙏 Thank You:
You're acknowledged in the CHANGELOG! Your contributions:
- [List their specific contributions]

Questions or issues? Reply to this email or file on GitHub.

Thank you for making caro better!

Best,
The caro Team
```

**Update GitHub Issue Comments**:
```bash
# For each fixed issue, add comment:
gh issue comment <issue-number> --body "Fixed in v1.1.0-beta.2! 🎉

This issue has been resolved in the latest beta iteration.

Download: https://github.com/wildcard/caro/releases/tag/v1.1.0-beta.2

Please verify the fix and let us know if you encounter any issues.

Thank you for the excellent bug report!"
```

### Step 10: Monitor Beta.2 Testing

**Setup Monitoring**:
- Monitor GitHub issues for new reports
- Track daily check-ins from testers
- Review telemetry exports (if provided)
- Respond to questions quickly

**Daily Tasks**:
- Check GitHub issues (morning and evening)
- Respond to tester questions within 4 hours
- Triage new bugs as they come in
- Update `.claude/beta-testing/beta-2-progress.md` daily

**Success Criteria for Beta.2**:
- [ ] All beta.1 P0/P1 bugs verified as fixed
- [ ] No NEW P0 bugs discovered
- [ ] <5 NEW P1 bugs discovered
- [ ] Pass rate ≥95% (tester assessment)
- [ ] 0% false positive rate maintained
- [ ] Average satisfaction ≥4.2/5.0

---

## 🔁 Iterating Further (Beta.3, Beta.4, etc.)

**When to Release Beta.3+**:
- If beta.2 discovers NEW critical bugs
- If significant changes are needed
- If testing reveals architectural issues
- If testers request major feature refinements

**When to Stop Beta Iterations**:
- All P0 and P1 bugs fixed
- Pass rate ≥95%
- False positive rate = 0%
- Average satisfaction ≥4.5/5.0
- Testers say "ready for GA" (majority)
- No new critical issues for 3+ days

**Process for Beta.3+**:
Repeat this entire process with incremented version numbers:
- Beta.3: `v1.1.0-beta.3`
- Beta.4: `v1.1.0-beta.4`
- etc.

**Each iteration should**:
- Fix all P0 bugs from previous iteration
- Fix majority of P1 bugs
- Improve quality metrics
- Add tester acknowledgments
- Document changes clearly

---

## 📊 Tracking Beta Iterations

**Create Iteration Tracker**: `.claude/beta-testing/iterations-summary.md`

```markdown
# Beta Iterations Summary

## Beta.1 (v1.1.0-beta.1)
- **Released**: 2026-01-09
- **Duration**: 5 days
- **Testers**: 5
- **Bugs Found**: 15 (0 P0, 8 P1, 5 P2, 2 P3)
- **Pass Rate**: 93.1%
- **Satisfaction**: 4.0/5.0
- **Status**: COMPLETED - Proceeded to Beta.2

**Key Learnings**:
- Safety patterns too broad (caused false positives)
- Platform detection needed refinement
- Telemetry privacy validation excellent (0 PII found)

## Beta.2 (v1.1.0-beta.2)
- **Released**: 2026-01-XX
- **Duration**: 3-5 days (ongoing)
- **Testers**: Same 5 testers
- **Bugs Found**: TBD
- **Pass Rate**: 96.6% (expected)
- **Satisfaction**: TBD
- **Status**: IN PROGRESS

**Focus Areas**:
- Verify beta.1 fixes
- Regression testing
- Edge cases
```

---

## ✅ Pre-Release Checklist (Beta.2+)

Before releasing any beta iteration, verify:

### Code Quality
- [ ] All P0 bugs from previous iteration fixed
- [ ] Majority (≥80%) of P1 bugs fixed
- [ ] All tests passing: `cargo test --all-features`
- [ ] Beta test suite passing: ≥95%
- [ ] No regressions detected
- [ ] Code reviewed (if major changes)

### Documentation
- [ ] CHANGELOG.md updated with iteration changes
- [ ] Release notes updated
- [ ] Deployment status updated
- [ ] Beta testing instructions updated (if needed)
- [ ] Previous testers acknowledged

### Version Management
- [ ] Cargo.toml version incremented (beta.X)
- [ ] Binary shows correct version: `caro --version`
- [ ] Cargo.lock updated
- [ ] Git tag created: `v1.1.0-beta.X`

### Release Assets
- [ ] Binary built successfully
- [ ] Binary tested manually
- [ ] SHA256 checksum generated
- [ ] Release notes ready
- [ ] GitHub pre-release created
- [ ] Assets uploaded

### Communication
- [ ] Beta testers notified
- [ ] GitHub issues updated
- [ ] Testing focus communicated
- [ ] Timeline shared (3-5 days)
- [ ] Monitoring plan established

---

## 🎯 Success Metrics per Iteration

Track these metrics across iterations to measure progress:

| Metric | Beta.1 | Beta.2 | Beta.3 | GA Target |
|--------|--------|--------|--------|-----------|
| Pass Rate | 93.1% | 96.6% | ? | ≥95% |
| P0 Bugs | 0 | ? | ? | 0 |
| P1 Bugs | 8 | ? | ? | <3 |
| False Positive Rate | 0% | ? | ? | 0% |
| Satisfaction | 4.0/5 | ? | ? | ≥4.5/5 |
| Would Use Daily | 80% | ? | ? | ≥90% |

**Goal**: Improve all metrics with each iteration.

---

## 🚀 Transition to GA Release

**When All Criteria Met**:
- Pass rate ≥95%
- 0 P0 bugs, <3 P1 bugs
- 0% false positive rate
- Satisfaction ≥4.5/5.0
- ≥90% would use daily
- User explicitly approves GA release

**Then**: Follow GA release process (separate document)

**IMPORTANT**: GA release ONLY happens if user explicitly requests it. Beta iterations can continue indefinitely until quality targets met.

---

## 📚 Related Documents

- **Beta Testing Instructions**: `.claude/releases/BETA-TESTING-INSTRUCTIONS.md`
- **Deployment Status**: `.claude/releases/BETA-DEPLOYMENT-STATUS.md`
- **Release Notes Template**: `.claude/releases/github-release-notes.md`
- **Go/No-Go Checklist**: `.claude/releases/v1.1.0-beta-go-nogo-checklist.md`

---

**Document Version**: 1.0
**Last Updated**: 2026-01-09
**Applies To**: Beta iterations (beta.2, beta.3, beta.4, etc.)
