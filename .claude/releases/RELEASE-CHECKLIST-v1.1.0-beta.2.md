# Release Checklist - v1.1.0-beta.2

**Release Date**: 2026-01-09
**Release Type**: Beta (Critical P0 Fixes)
**Target Audience**: Beta testers from beta.1 cycle

---

## ✅ Completed Pre-Release Tasks

- [x] Fixed all 5 P0 issues from beta.1 testing
  - [x] Issue #402: Telemetry consent persistence
  - [x] Issue #403: Telemetry disable functionality
  - [x] Issue #404: JSON output correctness
  - [x] Issue #405: Documentation accuracy
  - [x] Issue #406: Command quality (40% → 100%)
- [x] Added regression tests (`tests/beta_regression.rs`)
- [x] All tests passing (5/5 regression + 148 library)
- [x] Updated `Cargo.toml` version to 1.1.0-beta.2
- [x] Updated `CHANGELOG.md` with comprehensive release notes
- [x] Created git tag `v1.1.0-beta.2`
- [x] Pushed tag and release branch to GitHub
- [x] Created beta.2 testing instructions document

---

## 🚀 Release Deployment Tasks

### 1. Build Release Binaries

#### macOS Apple Silicon (aarch64)
```bash
# On macOS ARM64 machine
cargo build --release --target aarch64-apple-darwin

# Binary location: target/aarch64-apple-darwin/release/caro
# Rename for distribution
cp target/aarch64-apple-darwin/release/caro caro-1.1.0-beta.2-macos-aarch64

# Generate checksum
shasum -a 256 caro-1.1.0-beta.2-macos-aarch64 > caro-1.1.0-beta.2-macos-aarch64.sha256

# Verify
cat caro-1.1.0-beta.2-macos-aarch64.sha256
```

#### macOS Intel (x86_64) - Optional
```bash
# On macOS Intel machine or with cross-compilation
cargo build --release --target x86_64-apple-darwin

cp target/x86_64-apple-darwin/release/caro caro-1.1.0-beta.2-macos-x86_64
shasum -a 256 caro-1.1.0-beta.2-macos-x86_64 > caro-1.1.0-beta.2-macos-x86_64.sha256
```

#### Linux x86_64 - Optional
```bash
# On Linux x86_64 machine
cargo build --release --target x86_64-unknown-linux-gnu

cp target/x86_64-unknown-linux-gnu/release/caro caro-1.1.0-beta.2-linux-x86_64
sha256sum caro-1.1.0-beta.2-linux-x86_64 > caro-1.1.0-beta.2-linux-x86_64.sha256
```

### 2. Create GitHub Release

1. **Navigate to GitHub Releases**:
   - https://github.com/wildcard/caro/releases/new

2. **Tag Selection**:
   - Choose tag: `v1.1.0-beta.2`

3. **Release Title**:
   ```
   v1.1.0-beta.2 - Critical P0 Fixes
   ```

4. **Release Description** (use template below):

```markdown
## 🔥 Critical P0 Fixes - Beta.2

This beta release fixes **5 critical P0 issues** identified during v1.1.0-beta.1 comprehensive beta testing that were blocking GA release.

### What's Fixed

#### Issue #402: Telemetry Consent Prompt Spam ✅
- **Problem**: Consent prompt appeared on every command (28 lines, 2 sec overhead)
- **Fix**: Properly persist consent result to config file
- **Impact**: Prompt now appears once, never again

#### Issue #403: Telemetry Cannot Be Disabled ✅
- **Problem**: `caro config set telemetry.enabled false` didn't persist
- **Fix**: Update and save both `telemetry.enabled` and `telemetry.first_run`
- **Impact**: Telemetry settings now persist correctly

#### Issue #404: Invalid JSON Output ✅
- **Problem**: `--output json` produced invalid JSON (telemetry prompt in stdout)
- **Fix**: Skip interactive prompts for non-human output formats
- **Impact**: JSON output is now spec-compliant

#### Issue #405: Documentation Mismatch ✅
- **Problem**: Docs claimed `caro assess` and `caro telemetry` don't exist
- **Fix**: Updated beta testing instructions
- **Impact**: Eliminates tester confusion

#### Issue #406: Command Quality Below Target ✅
- **Problem**: File Management pass rate was 40% (2/5 tests)
- **Fix**: Added missing static patterns for common queries
- **Impact**: Pass rate improved to 100% (5/5 tests)

### Quality Metrics

- **Command Quality**: 40% → 100% for File Management category
- **Test Coverage**: +5 regression tests (all passing)
- **Build Status**: ✅ All 153 tests passing (5 regression + 148 library)

### Installation

**macOS Apple Silicon** (Recommended):
```bash
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.2/caro-1.1.0-beta.2-macos-aarch64 -o caro
chmod +x caro
sudo mv caro /usr/local/bin/caro
caro --version  # Should show: caro 1.1.0-beta.2
```

**Build from Source** (All Platforms):
```bash
git clone https://github.com/wildcard/caro
cd caro
git checkout v1.1.0-beta.2
cargo build --release
sudo cp target/release/caro /usr/local/bin/caro
```

### Testing Instructions

See: [BETA-TESTING-INSTRUCTIONS-v1.1.0-beta.2.md](https://github.com/wildcard/caro/blob/release/v1.1.0/.claude/releases/BETA-TESTING-INSTRUCTIONS-v1.1.0-beta.2.md)

**Focus**: 3-day regression testing to verify P0 fixes

### For Beta Testers

**Priority 1**: Verify all 5 P0 fixes are resolved (see testing instructions)
**Priority 2**: General stability and regression testing
**Priority 3**: New feedback and suggestions

### Upgrade from Beta.1

Your config and telemetry preferences will be preserved:
```bash
caro --version  # Verify current version
# Install beta.2 using instructions above
caro --version  # Should show: 1.1.0-beta.2
```

### Changelog

Full changelog: [CHANGELOG.md](https://github.com/wildcard/caro/blob/v1.1.0-beta.2/CHANGELOG.md)

### Feedback

- 🐛 **Report bugs**: [GitHub Issues](https://github.com/wildcard/caro/issues) (label: `beta.2`)
- 🐛 **P0 Regressions**: [GitHub Issues](https://github.com/wildcard/caro/issues) (label: `beta.2-regression`)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/wildcard/caro/discussions)
- 📧 **Email**: beta@caro.sh

---

**Thank you to all beta.1 testers** who helped identify these critical issues! 🙏
```

5. **Mark as Pre-release**:
   - ✅ Check "This is a pre-release"

6. **Upload Binaries**:
   - Upload: `caro-1.1.0-beta.2-macos-aarch64`
   - Upload: `caro-1.1.0-beta.2-macos-aarch64.sha256`
   - (Optional) Other platform binaries

7. **Publish Release**:
   - Click "Publish release"

### 3. Notify Beta Testers

#### Email Template

**Subject**: Caro v1.1.0-beta.2 Released - Critical P0 Fixes

```
Hi Beta Tester,

Thank you for your invaluable feedback on v1.1.0-beta.1! We've released beta.2 with fixes for all 5 critical P0 issues you helped identify.

## What's New in Beta.2

✅ Telemetry consent prompt no longer spams (Issue #402)
✅ Telemetry disable now persists correctly (Issue #403)
✅ JSON output is spec-compliant (Issue #404)
✅ Documentation accuracy restored (Issue #405)
✅ Command quality improved to 100% (Issue #406)

## Your Mission (3 Days)

**Priority 1**: Verify all 5 P0 fixes are resolved
**Priority 2**: General stability testing
**Priority 3**: New feedback

## Installation

Download: https://github.com/wildcard/caro/releases/tag/v1.1.0-beta.2

Testing Guide: [link to beta.2 instructions]

## Questions?

Reply to this email or post in GitHub Discussions.

Let's get to GA! 🚀

---
Caro Team
```

#### GitHub Announcement

Post in: https://github.com/wildcard/caro/discussions

**Title**: v1.1.0-beta.2 Released - All P0 Issues Fixed

```markdown
## 🎉 Beta.2 is Live!

We've just released v1.1.0-beta.2 with fixes for all 5 critical P0 issues from beta.1 testing.

### Quick Links

- **Release**: https://github.com/wildcard/caro/releases/tag/v1.1.0-beta.2
- **Testing Instructions**: [Link]
- **Changelog**: [Link]

### What's Fixed

1. ✅ Telemetry prompt spam eliminated
2. ✅ Telemetry disable works correctly
3. ✅ JSON output is valid
4. ✅ Documentation is accurate
5. ✅ Command quality at 100%

### Help Us Test

We need **focused 3-day testing** to verify these fixes before GA.

**Priority**: Verify P0 fixes don't regress

Download and test: [Release link]

### Thank You

Huge thanks to beta.1 testers for identifying these issues! 🙏

Questions? Ask here or email beta@caro.sh
```

---

## 📋 Post-Release Verification

### 1. Installation Testing

Test installation on clean machines:
- [ ] macOS Apple Silicon: Binary download works
- [ ] macOS Intel: Build from source works
- [ ] Linux: Build from source works

### 2. Quick Smoke Tests

```bash
# Version check
caro --version  # Should show: 1.1.0-beta.2

# P0 quick tests
rm -f ~/.config/caro/config.toml
caro "list files"  # Prompt appears once
caro "list files"  # No prompt second time
caro --output json "list files" | jq '.'  # Valid JSON
caro "show disk space by directory"  # Correct command
```

### 3. Monitor Feedback

- [ ] Watch GitHub Issues for beta.2 reports
- [ ] Monitor beta@caro.sh email
- [ ] Check GitHub Discussions
- [ ] Respond to tester questions within 24 hours

---

## 🎯 Success Criteria

Beta.2 is successful if:
- [ ] All 5 P0 tests pass for all testers
- [ ] No P0 regressions reported
- [ ] <2 new P1 bugs found
- [ ] Average tester satisfaction ≥4.0/5.0
- [ ] Ready to proceed to GA release

---

## 📊 Timeline

| Date | Milestone |
|------|-----------|
| 2026-01-09 | Beta.2 released |
| 2026-01-09 | Testers notified |
| 2026-01-09-11 | Testing period (3 days) |
| 2026-01-12 | Collect feedback |
| 2026-01-13 | Go/No-Go decision for GA |

---

## 🚨 Rollback Plan

If critical issues found:
1. Document the issue in GitHub
2. Decide: Fix in beta.3 or revert to beta.1
3. Communicate decision to testers
4. If beta.3 needed, repeat this checklist

---

**Document Version**: 1.0
**Created**: 2026-01-09
**Status**: Ready for deployment
