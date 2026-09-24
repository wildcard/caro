# Release Runbook

This document provides step-by-step instructions for executing a caro release.

## Pre-Release Checklist

- [ ] All PRs for milestone merged
- [ ] CHANGELOG.md updated with release notes
- [ ] Version bumped in Cargo.toml (and Cargo.lock updated)
- [ ] All CI checks passing on main branch
- [ ] Known issues documented in known-issues.md
- [ ] No P0/P1 issues blocking release

## Release Steps

### Step 1: Verify Pre-Conditions

```bash
# Ensure you're on latest main
git checkout main
git pull origin main

# Verify version in Cargo.toml
VERSION=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version')
echo "Releasing version: $VERSION"

# Verify no uncommitted changes
git status --porcelain
```

**Expected**: Clean working directory, correct version number

---

### Step 2: Create and Push Tag

```bash
# Create annotated tag (triggers publish workflow)
git tag -a v$VERSION -m "Release v$VERSION"

# Push tag to trigger workflows
git push origin v$VERSION
```

**Expected**: Tag pushed successfully, workflows triggered

---

### Step 3: Monitor Publish Workflow

```bash
# Find the publish workflow run
gh run list --workflow=publish.yml --limit 1

# Watch it in real-time
gh run watch <run-id>
```

**What to Watch For**:
- `cargo fmt --check` passes
- `cargo clippy -- -D warnings` passes
- All tests pass
- `cargo publish` succeeds
- Package appears on crates.io

**If Failures Occur**: See troubleshooting section below

---

### Step 4: Verify crates.io Publication

```bash
# Check crates.io API
curl -s https://crates.io/api/v1/crates/caro/$VERSION | jq -r '.version.num'

# Should return: $VERSION
```

**Expected**: Version is published and downloadable

---

### Step 5: Monitor Release Workflow

The release workflow triggers automatically after successful publish.

```bash
# Find the release workflow run
gh run list --workflow=release.yml --limit 1

# Watch it
gh run watch <run-id>
```

**What to Watch For**:
- Version consistency check passes
- All platform binaries build successfully:
  - linux-amd64
  - linux-arm64
  - macos-intel
  - macos-silicon
  - windows-amd64
- GitHub release created
- All binaries uploaded

**If Failures Occur**: See troubleshooting section below

---

### Step 6: Verify GitHub Release Assets

```bash
# Check GitHub release
gh release view v$VERSION --json assets -q '.assets[] | .name'
```

**Expected Assets** (10 files):
```
caro-$VERSION-linux-amd64
caro-$VERSION-linux-amd64.sha256
caro-$VERSION-linux-arm64
caro-$VERSION-linux-arm64.sha256
caro-$VERSION-macos-intel
caro-$VERSION-macos-intel.sha256
caro-$VERSION-macos-silicon
caro-$VERSION-macos-silicon.sha256
caro-$VERSION-windows-amd64.exe
caro-$VERSION-windows-amd64.exe.sha256
```

---

### Step 7: Trigger Model Bundling (Post-Release)

**IMPORTANT**: Only trigger after QE sign-off (see Step 8)

```bash
# Manual trigger via GitHub UI (recommended until workflow is indexed):
# https://github.com/wildcard/caro/actions/workflows/bundle.yml
# Click "Run workflow", enter version: v$VERSION

# Or via CLI (after workflow is indexed):
gh workflow run bundle.yml -f version=v$VERSION
```

**What to Watch For**:
- 10 bundle jobs execute (5 platforms × 2 models)
- Each bundle creates tar.gz + sha256
- All bundles upload to release

---

### Step 8: QE Sign-Off

**DO NOT SKIP THIS STEP**

Invoke the Quality Engineer Manager skill for systematic validation:

```
/quality-engineer-manager action=sign-off version=v$VERSION
```

This will:
1. Verify all workflows succeeded
2. Validate release artifacts
3. Dispatch beta testers (optional)
4. Generate sign-off report
5. Approve or reject release

**Sign-Off Decision Matrix**:
- ✅ **SHIP IT**: No P0/P1 issues, all critical paths working
- 🔧 **HOTFIX**: P0 issue found, quick fix available
- ❌ **ABORT**: P0 issue found, complex fix needed
- ⚠️ **CONDITIONAL**: Multiple P1 issues, assess workarounds

---

### Step 9: Post-Release Tasks

```bash
# Close the milestone
gh api -X PATCH /repos/wildcard/caro/milestones/<milestone-number> \
  -f state='closed'

# Create next milestone (if needed)
gh api -X POST /repos/wildcard/caro/milestones \
  -f title='v<next-version>' \
  -f description='Next release planning'
```

**Documentation Updates**:
- [ ] Update README.md if installation changed
- [ ] Announce release on social channels
- [ ] Update project website (if applicable)
- [ ] Share release notes with community

---

### Step 10: Monitor Post-Release

**First 24-48 Hours**:
- Monitor GitHub issues for new bug reports
- Track crates.io download counts
- Watch for installation problems
- Respond to user feedback promptly

**Metrics to Track**:
```bash
# Check download stats
curl -s https://crates.io/api/v1/crates/caro | jq '.crate.downloads'

# Check GitHub release downloads
gh release view v$VERSION --json assets \
  -q '.assets[] | "\(.name): \(.download_count) downloads"'

# Check for new issues
gh issue list --state open --label "bug" --created ">$(date -d '24 hours ago' -Iseconds)"
```

---

## Troubleshooting

### Publish Workflow Failures

#### Formatting Check Failed
```bash
cargo fmt --all
git add -u
git commit -m "style: Apply cargo fmt fixes"
git push origin main

# Re-tag and push
git tag -d v$VERSION
git push origin :refs/tags/v$VERSION
git tag -a v$VERSION -m "Release v$VERSION"
git push origin v$VERSION
```

#### Clippy Warnings
```bash
cargo clippy --fix --allow-dirty --allow-staged
git commit -am "fix: Resolve clippy warnings"
git push origin main

# Re-tag (same as above)
```

#### Tests Failed
1. Identify failing test
2. Fix the test or code
3. Push fix to main
4. Re-tag and push

#### cargo publish Failed
- Check crates.io status: https://status.crates.io
- Verify Cargo.toml metadata is correct
- Check CARGO_REGISTRY_TOKEN secret in GitHub

---

### Release Workflow Failures

#### Binary Build Failed
1. Check build logs: `gh run view <run-id> --log-failed`
2. Identify platform-specific issue
3. Fix and push to main
4. Manually re-run workflow: `gh run rerun <run-id>`

#### Release Asset Upload Failed
- Verify `contents: write` permission in workflow
- Check GitHub API status
- Manually re-run workflow

---

### Bundle Workflow Failures

See known issues:
- #7: YAML heredoc syntax (use `printf`)
- #8: Alpine gh CLI missing (manual install)
- #9: Permission denied (add `contents: write`)

**Common Fixes**:
```bash
# Re-trigger bundle workflow
gh workflow run bundle.yml -f version=v$VERSION

# Check bundle job logs
gh run list --workflow=bundle.yml --limit 1
gh run view <run-id> --log-failed
```

---

## Rollback Procedure

**When to Rollback**: P0 issue discovered post-release, no quick fix available

### Step 1: Assess Impact

- How many users affected?
- Is there a workaround?
- How quickly can we fix it?

### Step 2: Delete Tag (Prevents New Installs)

```bash
# Delete local tag
git tag -d v$VERSION

# Delete remote tag
git push origin :refs/tags/v$VERSION
```

### Step 3: Yank from crates.io (Last Resort)

```bash
# Yank the version (makes it un-installable)
cargo yank --version $VERSION

# To un-yank (if fixing quickly):
cargo yank --version $VERSION --undo
```

**Note**: Yanking doesn't remove already-installed versions

### Step 4: Communicate

- Update GitHub release with "YANKED" notice
- File GitHub issue explaining the problem
- Post on social channels if widely used
- Document in known-issues.md

### Step 5: Hotfix

Follow hotfix-workflow.md for emergency patch release

---

## Release Cadence

**Minor Versions** (X.Y.0):
- New features, major improvements
- Every 4-6 weeks
- Requires full QE sign-off with beta testing

**Patch Versions** (X.Y.Z):
- Bug fixes, minor improvements
- As needed (typically 1-2 weeks after minor)
- Streamlined QE process

**Hotfix Versions** (X.Y.Z+1):
- Critical security or P0 bugs only
- Immediate (within 24-48 hours)
- Fast-track QE with critical path testing only

---

## References

- **Known Issues Database**: `known-issues.md`
- **Hotfix Workflow**: `hotfix-workflow.md`
- **Beta Testing Playbook**: `beta-testing-playbook.md`
- **QE Manager Skill**: `../ SKILL.md`
- **GitHub Actions Docs**: https://docs.github.com/en/actions
- **crates.io API**: https://crates.io/data-access

---

## Quick Reference Commands

```bash
# Check version
cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version'

# Create tag
git tag -a vX.Y.Z -m "Release vX.Y.Z" && git push origin vX.Y.Z

# Watch workflow
gh run watch <run-id>

# Verify crates.io
curl -s https://crates.io/api/v1/crates/caro/X.Y.Z | jq -r '.version.num'

# Trigger bundle
gh workflow run bundle.yml -f version=vX.Y.Z

# Check release assets
gh release view vX.Y.Z --json assets -q '.assets[] | .name'

# Rollback
git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z
cargo yank --version X.Y.Z
```
