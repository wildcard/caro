# Release Preparation Reference

This document describes the workflow for preparing the next beta release after fixing issues.

## Overview

After P0 fixes are completed and merged, prepare the next beta release:

1. Version Bumping
2. CHANGELOG Update
3. Git Tagging
4. Beta Testing Instructions
5. Release Artifacts

## Phase 1: Version Bumping

### Update Cargo.toml

```bash
# Edit version
vim Cargo.toml

# Change from:
version = "1.1.0-beta.1"

# To:
version = "1.1.0-beta.2"
```

### Verify Version

```bash
# Check updated version
grep '^version = ' Cargo.toml

# Build to verify Cargo.toml is valid
cargo check
```

## Phase 2: CHANGELOG Update

### Read Existing CHANGELOG

```bash
# Check current structure
head -n 50 CHANGELOG.md
```

### Add New Release Section

Add section ABOVE the previous beta release:

```markdown
## [1.1.0-beta.2] - 2026-01-09

### 🔥 Critical Fixes (P0 Issues from Beta.1 Testing)

This release fixes **5 critical P0 issues** identified during v1.1.0-beta.1 comprehensive beta testing that were blocking GA release.

#### Fixed

- **Issue #402**: Telemetry consent prompt appearing on every command invocation
  - Root cause: Consent result was never persisted to config file
  - Fix: Added config persistence after consent prompt
  - Impact: Eliminates 28-line prompt spam and 2-second overhead on every command

- **Issue #403**: Telemetry cannot be disabled despite config setting
  - Root cause: Same as #402 - first_run flag never updated
  - Fix: Properly updates `telemetry.enabled` and `telemetry.first_run` in config
  - Verification: `caro config set telemetry.enabled false` now persists correctly

[... additional issues ...]

#### Testing

- Added `tests/beta_regression.rs` with 5 regression tests to prevent future breakage
- All tests passing: 5/5 regression tests + 148 library tests
- Verified fixes manually per beta testing protocol

#### Documentation

- Updated beta testing instructions to reflect actual command availability
- Added detailed troubleshooting guidance for telemetry configuration

### 📊 Quality Metrics

- **Command Quality**: File Management category improved from 40% to 100%
- **Telemetry UX**: Eliminated consent prompt spam (appears once, persists correctly)
- **JSON Output**: Now 100% spec-compliant (no pollution from interactive prompts)
- **Test Coverage**: +5 regression tests covering all P0 fixes

## [1.1.0-beta.1] - 2026-01-08

[Previous release content...]
```

### CHANGELOG Format

**Section structure**:
1. Version header with date
2. Brief summary of release focus
3. Fixed/Added/Changed sections with bullet points
4. Testing section
5. Documentation section (if applicable)
6. Metrics section showing improvement

**Each fix entry should include**:
- Issue ID
- Brief description
- Root cause (one line)
- Fix (one line)
- Impact/verification

## Phase 3: Git Tagging

### Create Annotated Tag

```bash
# Create tag
git tag -a v1.1.0-beta.2 -m "$(cat <<'EOF'
Release v1.1.0-beta.2 - Critical P0 Fixes

This beta release fixes 5 critical P0 issues identified during v1.1.0-beta.1
comprehensive beta testing that were blocking GA release.

Critical Fixes:
- Issue #402: Telemetry consent prompt spam eliminated
- Issue #403: Telemetry disable now persists correctly
- Issue #404: JSON output is now spec-compliant
- Issue #405: Documentation accuracy restored
- Issue #406: Command quality improved to 100% (File Management)

Quality Metrics:
- Command Quality: 40% → 100% for File Management
- Test Coverage: +5 regression tests
- All tests passing: 5/5 regression + 148 library

Ready for beta.2 testing cycle.

🤖 Tagged with Claude Code
EOF
)"
```

### Verify Tag

```bash
# List tags
git tag -l "v1.1.0-beta*"

# Show tag details
git tag -n20 v1.1.0-beta.2
```

### Push Tag

```bash
# Push tag to remote
git push origin v1.1.0-beta.2

# Verify on remote
git ls-remote --tags origin | grep beta.2
```

## Phase 4: Beta Testing Instructions

### Create New Instructions Document

**File**: `.claude/releases/BETA-TESTING-INSTRUCTIONS-v1.1.0-beta.2.md`

**Structure**:

1. **Title and Welcome**
   ```markdown
   # Beta Testing Instructions - v1.1.0-beta.2

   **Welcome to Beta.2!** Thank you for helping us test the critical P0 fixes.
   ```

2. **Beta.2 Testing Goals**
   - Shorter cycle (3 days instead of 5)
   - Focused on P0 fix verification
   - Regression testing emphasis

3. **What's New in Beta.2**
   - List all P0 fixes
   - Highlight improvements
   - Set expectations

4. **Installation Instructions**
   - Update URLs to beta.2
   - Update version strings
   - Update checksums

5. **Priority Testing Focus**
   - Test #1: Verify Issue #402 fixed
   - Test #2: Verify Issue #403 fixed
   - etc.

6. **Detailed Verification Procedures**
   For each P0 issue:
   - What was broken
   - How to test the fix
   - Expected behavior
   - Success criteria

7. **Regression Testing Checklist**
   - Ensure fixes don't break other features
   - Spot check various categories
   - Performance validation

8. **Feedback Collection**
   - High priority: P0 fix regressions
   - Medium priority: New bugs
   - Low priority: General feedback

### Example Test Procedure

```markdown
### Test #1: Issue #402 - Telemetry Prompt Spam (CRITICAL)

**What was broken**: Telemetry consent prompt appeared on EVERY command invocation.

**How to test**:
1. **Fresh install** (or delete `~/.config/caro/config.toml`):
   ```bash
   rm -f ~/.config/caro/config.toml
   ```

2. **First run** - You should see the telemetry consent prompt ONCE:
   ```bash
   caro "list files"
   # Expected: Telemetry consent prompt appears
   ```

3. **Second run** - Prompt should NOT appear:
   ```bash
   caro "show disk usage"
   # Expected: No telemetry prompt, just command output
   ```

4. **Third run** - Still no prompt:
   ```bash
   caro "find large files"
   # Expected: No telemetry prompt
   ```

**Success criteria**:
- [ ] Telemetry prompt appears ONCE on first run
- [ ] Prompt NEVER appears on subsequent runs
- [ ] No 2-second overhead after first run

**If this test fails**: Report immediately as P0 regression!
```

## Phase 5: Commit and Push

### Commit Version Bump

```bash
# Stage files
git add Cargo.toml CHANGELOG.md

# Commit
git commit -m "$(cat <<'EOF'
chore: Bump version to 1.1.0-beta.2

Updated version in Cargo.toml and documented all P0 fixes from beta.1
comprehensive testing in CHANGELOG.md.

Release notes:
- 5 critical P0 issues fixed (telemetry UX, JSON output, command quality)
- Command quality improved from 40% to 100% for File Management
- Telemetry consent now persists correctly (eliminates prompt spam)
- JSON output is now spec-compliant (no pollution from interactive prompts)
- +5 regression tests to prevent future breakage

All tests passing (5/5 regression + 148 library tests).

🤖 Generated with Claude Code

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

### Commit Beta Testing Instructions

```bash
# Stage new file
git add .claude/releases/BETA-TESTING-INSTRUCTIONS-v1.1.0-beta.2.md

# Commit
git commit -m "docs(beta.2): Add comprehensive beta.2 testing instructions

Created focused 3-day testing protocol for v1.1.0-beta.2 that:
- Prioritizes verification of 5 P0 fixes from beta.1
- Provides detailed test procedures for each critical issue
- Includes regression testing checklist
- Updates installation URLs to beta.2
- Focuses on fix validation vs exploratory testing

Key changes from beta.1 instructions:
- Reduced from 5-day to 3-day focused cycle
- Priority 1: P0 fix verification (Issues #402-406)
- Clear success criteria for each test
- Immediate reporting triggers for regressions

🤖 Generated with Claude Code

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

### Push All Changes

```bash
# Push commits and tag
git push origin release/v1.1.0
git push origin v1.1.0-beta.2
```

## Phase 6: GitHub Release (Optional)

### Create Release on GitHub

1. Navigate to: `https://github.com/USER/REPO/releases/new`

2. Select tag: `v1.1.0-beta.2`

3. Release title: `v1.1.0-beta.2 - Critical P0 Fixes`

4. Description:
```markdown
## 🔥 Critical P0 Fixes - Beta.2

This beta release fixes **5 critical P0 issues** identified during v1.1.0-beta.1 comprehensive beta testing that were blocking GA release.

### What's Fixed

#### Issue #402: Telemetry Consent Prompt Spam ✅
- **Problem**: Consent prompt appeared on every command (28 lines, 2 sec overhead)
- **Fix**: Properly persist consent result to config file
- **Impact**: Prompt now appears once, never again

[... other issues ...]

### Quality Metrics

- **Command Quality**: 40% → 100% for File Management category
- **Test Coverage**: +5 regression tests (all passing)
- **Build Status**: ✅ All 153 tests passing (5 regression + 148 library)

### Installation

**macOS Apple Silicon** (Recommended):
```bash
curl -L https://github.com/USER/REPO/releases/download/v1.1.0-beta.2/caro-1.1.0-beta.2-macos-aarch64 -o caro
chmod +x caro
sudo mv caro /usr/local/bin/caro
caro --version  # Should show: caro 1.1.0-beta.2
```

### Testing Instructions

See: [BETA-TESTING-INSTRUCTIONS-v1.1.0-beta.2.md]

**Focus**: 3-day regression testing to verify P0 fixes

### Feedback

- 🐛 **Report bugs**: [GitHub Issues] (label: `beta.2`)
- 🐛 **P0 Regressions**: [GitHub Issues] (label: `beta.2-regression`)
```

5. Check: "This is a pre-release"

6. Attach binaries (if built)

7. Publish release

## Verification Checklist

Before announcing beta.2 release:

- [ ] Version bumped in Cargo.toml
- [ ] CHANGELOG.md updated with all fixes
- [ ] Git tag created and pushed
- [ ] Beta testing instructions created
- [ ] All changes committed and pushed
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] GitHub release created (if applicable)
- [ ] Installation URLs updated in docs

## Summary Timeline

From "fixes complete" to "beta release ready":

1. **Version bump** (5 min)
   - Edit Cargo.toml
   - Verify with cargo check

2. **CHANGELOG update** (15 min)
   - Add new release section
   - Document all fixes with root causes
   - Include metrics

3. **Git tagging** (5 min)
   - Create annotated tag
   - Push to remote

4. **Testing instructions** (30 min)
   - Create beta.2 instructions
   - Focus on P0 verification
   - Include detailed test procedures

5. **Commit and push** (5 min)
   - Commit version bump
   - Commit testing instructions
   - Push all changes

**Total time**: ~1 hour

## Resources

- Version bumping guide: This document
- CHANGELOG format: This document (Phase 2)
- Testing instructions template: `.claude/releases/BETA-TESTING-INSTRUCTIONS-v1.1.0-beta.2.md`
- Tag creation: This document (Phase 3)
