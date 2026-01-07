# Hotfix Workflow

Emergency procedure for releasing critical fixes outside the normal release cycle.

## When to Use Hotfix Workflow

Use this workflow ONLY when:
- **P0 (Critical)** bug discovered post-release that blocks primary use case
- **Security vulnerability** requiring immediate patch (CVE, exploit, data leak)
- **Release-breaking regression** that makes the software unusable for most users

**DO NOT use for**:
- P1/P2 bugs (schedule for next regular release)
- Feature requests (wait for next minor version)
- Documentation updates (can be pushed directly)
- Non-urgent improvements

---

## Hotfix Process

### Step 1: Assess Severity

Use the decision matrix:

| Condition | Severity | Action |
|-----------|----------|--------|
| Prevents all users from basic functionality | P0 | HOTFIX NOW |
| Data loss or corruption possible | P0 | HOTFIX NOW |
| Security vulnerability with active exploits | P0 | HOTFIX NOW |
| Breaks common workflow, workaround exists | P1 | Regular release |
| Degrades experience, reasonable workaround | P2 | Regular release |

**Example P0 Issues**:
- CLI crashes on launch for all platforms
- Model download fails with no fallback
- Security vulnerability allowing code execution
- Data corruption in user's config files

**Not P0 (Use Regular Release)**:
- Slow performance on edge case
- Missing feature causing inconvenience
- Cosmetic UI issues
- Platform-specific quirks with workarounds

---

### Step 2: Create Hotfix Branch

```bash
# Branch from the release tag that needs fixing
git checkout -b hotfix/vX.Y.Z+1 vX.Y.Z

# Example: Fixing v1.0.4 → create hotfix/v1.0.5
git checkout -b hotfix/v1.0.5 v1.0.4
```

**Alternative** (if fix is already on main):
```bash
# Cherry-pick the fix commit from main
git checkout -b hotfix/v1.0.5 v1.0.4
git cherry-pick <commit-sha>
```

---

### Step 3: Apply Minimal Fix

**Critical**: Keep changes MINIMAL. Hotfix scope creep is dangerous.

**DO**:
- Fix only the P0 issue
- Include tests for the fix
- Update CHANGELOG.md with hotfix entry
- Keep commit message clear

**DON'T**:
- Add new features "while we're at it"
- Refactor surrounding code
- Fix unrelated P1/P2 issues
- Make "improvements" beyond the fix

**Example Fix Commit**:
```bash
# Apply the minimal fix
git add <fixed-files>
git commit -m "fix(critical): Prevent crash on model path with spaces

Fixes #123

The model loader incorrectly handled paths containing spaces,
causing a panic on initialization. This adds proper quoting to
the path resolution logic.

Tested on: Linux, macOS, Windows"
```

---

### Step 4: Run Fast-Track Testing

**Critical Path Tests Only** (not full test suite):

```bash
# Run core functionality tests
cargo test --test e2e_cli_tests --verbose
cargo test --test system_integration --verbose

# Run the specific test for the fix
cargo test <test-name> --verbose

# Manual smoke test
cargo build --release
./target/release/caro "test command"
```

**Platform Testing**:
- Test on the platform where bug was discovered
- If cross-platform issue, test on all 3 major platforms
- Use pre-built binaries from CI if possible

---

### Step 5: Update Version and Changelog

```bash
# Bump patch version in Cargo.toml
# v1.0.4 → v1.0.5
cargo set-version X.Y.Z+1

# Or manually edit Cargo.toml:
# version = "1.0.5"

# Update Cargo.lock
cargo update -p caro

# Update CHANGELOG.md
cat >> CHANGELOG.md << 'EOF'

## [X.Y.Z+1] - $(date +%Y-%m-%d)

### Fixed
- Critical: [Brief description of fix] (#issue-number)

EOF

# Commit version bump
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: bump version to X.Y.Z+1 for hotfix"
```

---

### Step 6: Merge to Main and Tag

```bash
# Ensure main is up to date
git checkout main
git pull origin main

# Merge hotfix branch
git merge --no-ff hotfix/vX.Y.Z+1

# Push to main
git push origin main

# Create and push tag
git tag -a vX.Y.Z+1 -m "Hotfix vX.Y.Z+1: [Brief description]

Fixes #issue-number

Critical fix for [symptom].

See CHANGELOG.md for details."

git push origin vX.Y.Z+1
```

**Important**: Tag push triggers publish and release workflows

---

### Step 7: Monitor Workflows

```bash
# Watch publish workflow
gh run list --workflow=publish.yml --limit 1
gh run watch <run-id>

# Watch release workflow
gh run list --workflow=release.yml --limit 1
gh run watch <run-id>

# Trigger bundle workflow (optional for hotfixes)
gh workflow run bundle.yml -f version=vX.Y.Z+1
```

**Expected Timeline**:
- Publish: ~7 minutes
- Release: ~15 minutes per platform
- Bundles: ~5 minutes per bundle (optional)

---

### Step 8: Verify Hotfix Deployment

```bash
# Verify crates.io
curl -s https://crates.io/api/v1/crates/caro/X.Y.Z+1 | jq -r '.version.num'

# Verify GitHub release
gh release view vX.Y.Z+1 --json assets -q '.assets[] | .name'

# Test install from crates.io
cargo install caro --version X.Y.Z+1 --force

# Test the fix
caro --version
caro "[command that was broken]"
```

---

### Step 9: Communicate Hotfix

**GitHub Release Notes** (auto-generated, then enhance):
```markdown
## Hotfix: vX.Y.Z+1

**Critical fix for [symptom]**

### What Was Fixed
- [Detailed explanation of the bug]
- [Impact on users]
- [What the fix does]

### Who Should Upgrade
All users on vX.Y.Z should upgrade immediately.

### How to Upgrade

**Via Cargo**:
```bash
cargo install caro --version X.Y.Z+1 --force
```

**Via Pre-Built Binaries**:
Download from: [link to release page]

### Verification
After upgrade, verify with:
```bash
caro --version  # Should show X.Y.Z+1
```

### Related Issues
Fixes #issue-number

### Rollback (If Needed)
To rollback to previous version:
```bash
cargo install caro --version X.Y.Z --force
```
```

**Social Channels**:
- Post on GitHub Discussions
- Update project README if install docs changed
- Notify in community Slack/Discord if applicable

**GitHub Issue**:
- Close the bug report with comment linking to hotfix release
- Thank reporter for finding the issue

---

### Step 10: Close Hotfix Branch

```bash
# Delete local branch
git branch -d hotfix/vX.Y.Z+1

# Delete remote branch (if pushed)
git push origin --delete hotfix/vX.Y.Z+1
```

**Document in Known Issues**:
- Add entry to `known-issues.md`
- Include symptoms, root cause, fix, and prevention
- Reference hotfix version

---

## Hotfix Checklist

Use this checklist for every hotfix:

### Pre-Hotfix
- [ ] Issue confirmed as P0/Critical
- [ ] No reasonable workaround exists
- [ ] Impact assessed (how many users affected)
- [ ] Fix approach identified

### During Hotfix
- [ ] Hotfix branch created from release tag
- [ ] Minimal fix applied (no scope creep)
- [ ] Tests added/updated for the fix
- [ ] Critical path tests pass
- [ ] Manually tested on affected platform(s)
- [ ] Version bumped (patch increment)
- [ ] CHANGELOG.md updated
- [ ] Merged to main
- [ ] Tag created and pushed

### Post-Hotfix
- [ ] Publish workflow succeeded
- [ ] Release workflow succeeded
- [ ] Verified on crates.io
- [ ] Verified on GitHub release
- [ ] Tested fresh install
- [ ] Fix verified to work
- [ ] Release notes updated
- [ ] Community notified
- [ ] GitHub issue closed
- [ ] Known issue documented
- [ ] Hotfix branch deleted

---

## Timeline Expectations

| Phase | Target Time | Maximum Time |
|-------|-------------|--------------|
| Assess & Branch | 15 minutes | 30 minutes |
| Apply Fix | 30 minutes | 2 hours |
| Test | 15 minutes | 1 hour |
| Version Bump | 5 minutes | 15 minutes |
| Merge & Tag | 10 minutes | 20 minutes |
| CI/CD | 20 minutes | 45 minutes |
| Verification | 10 minutes | 30 minutes |
| Communication | 15 minutes | 45 minutes |
| **TOTAL** | **2 hours** | **6 hours** |

**If exceeding maximum time**: Consider if this is truly P0, or if it should be a regular release.

---

## Examples of Past Hotfixes

### Example: v1.0.5 Hotfix (Hypothetical)

**Issue**: Model loader crashes on Windows when path contains spaces

**Severity**: P0 - Affects all Windows users with spaces in username

**Timeline**:
- 09:00 - Issue reported #234
- 09:15 - Confirmed P0, created hotfix/v1.0.5 branch
- 09:45 - Fix applied (added proper path quoting)
- 10:00 - Tests pass, version bumped
- 10:15 - Merged and tagged
- 10:40 - Workflows complete
- 11:00 - Verified and announced
- **Total**: 2 hours from report to release

**Fix**:
```rust
// Before (broken):
let model_path = PathBuf::from(path_str);

// After (fixed):
let model_path = shellexpand::tilde(path_str).into_owned();
let model_path = PathBuf::from(model_path);
```

---

## When NOT to Hotfix

**Use Regular Release Instead**:
- Bug affects <10% of users
- Workaround exists (even if inconvenient)
- Fix requires significant refactoring
- Not time-sensitive (can wait 1-2 weeks)
- Issue is P1 or lower

**Wait for Next Major/Minor**:
- Breaking API changes required
- Architectural redesign needed
- Multiple related issues to fix together
- Feature additions mixed with bug fix

---

## Rollback Procedure (If Hotfix Fails)

If hotfix introduces new critical issues:

```bash
# Delete the hotfix tag
git tag -d vX.Y.Z+1
git push origin :refs/tags/vX.Y.Z+1

# Yank from crates.io
cargo yank --version X.Y.Z+1

# Create GitHub issue documenting the failure
gh issue create --title "[HOTFIX FAILED] vX.Y.Z+1 introduced regression" \
  --body "Description of new issue..."

# Communicate rollback
# - Update release notes with YANKED notice
# - Post on social channels
# - Advise users to stay on vX.Y.Z
```

Then:
1. Fix the fix
2. Create new hotfix vX.Y.Z+2
3. Follow standard hotfix workflow

---

## References

- **Release Runbook**: `release-runbook.md`
- **Known Issues**: `known-issues.md`
- **QE Manager**: `../SKILL.md`
- **GitHub Workflow Troubleshooting**: `known-issues.md` #7, #8, #9

---

## Emergency Contacts

**For Security Hotfixes**:
- Review security policy: `SECURITY.md` (if exists)
- Follow responsible disclosure timeline
- Coordinate with security team
- Consider private security advisory

**For Critical Infrastructure Failures**:
- Check crates.io status: https://status.crates.io
- Check GitHub Actions status: https://www.githubstatus.com
- May need to wait for service restoration
