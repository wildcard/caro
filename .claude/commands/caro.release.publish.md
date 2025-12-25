---
description: Create PR, merge, tag, and publish caro release
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `.github/workflows/publish.yml`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Branch Pre-flight Check (MANDATORY)

**This command enforces the release branch workflow per docs/RELEASE_PROCESS.md.**

Run these checks:
```bash
CURRENT_BRANCH=$(git branch --show-current)
git status --porcelain
```

**If NOT on a release branch** (`release/v*` or `hotfix/v*`):
- REFUSE to proceed with error:
  "ERROR: Must be on a release branch. Current: $CURRENT_BRANCH
  Run /caro.release.prepare first."

**If working directory is not clean**:
- REFUSE to proceed with error:
  "ERROR: Working directory must be clean. Please commit all changes.
  Run /caro.release.version if you haven't bumped the version yet."

---

## Workflow Context

**Before this**: `/caro.release.version` bumped version and updated changelog

**This command**: Creates PR, merges to main, tags release, monitors CI/CD

**After this**: Run `/caro.release.verify` to test the published release

---

## Outline

### 1. Extract Version from Branch

Parse the version from the current branch name:
```bash
CURRENT_BRANCH=$(git branch --show-current)
VERSION=${CURRENT_BRANCH#release/v}
VERSION=${VERSION#hotfix/v}
echo "Releasing version: v$VERSION"
```

### 2. Verify Version Consistency

**Check that version in Cargo.toml matches**:
```bash
CARGO_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "Cargo.toml version: $CARGO_VERSION"
echo "Branch version: $VERSION"
```

**If versions don't match**:
- Display error: "ERROR: Version mismatch! Cargo.toml has $CARGO_VERSION but branch is release/v$VERSION"
- Exit with instructions: "Run /caro.release.version to fix version consistency"

### 3. Show Release Summary

Display what will be released:
```bash
echo "Release Summary for v$VERSION:"
echo ""
echo "Changes since last tag:"
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
git log --oneline $LAST_TAG..HEAD
echo ""
echo "Files changed:"
git diff --name-only $LAST_TAG..HEAD
```

**Extract changelog entries**:
- Read CHANGELOG.md
- Find the section for this version
- Display the changes

**Ask for confirmation**:
"Proceed with publishing v$VERSION? (y/n)"

If no, exit without proceeding.

### 4. Push Release Branch

Push the release branch to origin:
```bash
git push -u origin $CURRENT_BRANCH
```

**If push fails**:
- Display error
- Check for remote branch conflicts
- Provide resolution instructions

### 5. Create Pull Request

Create PR using GitHub CLI:
```bash
gh pr create \
  --title "Release v$VERSION" \
  --body "$(cat <<'EOF'
## Release v$VERSION

This PR prepares the release of caro v$VERSION.

### Release Checklist

- [x] Security audit completed (`/caro.release.security`)
- [x] Version bumped in Cargo.toml
- [x] CHANGELOG.md updated with release notes
- [x] All tests passing
- [x] Documentation updated

### Changes

[Paste changelog entries here]

### Testing

- All N unit tests pass
- All N integration tests pass
- Security audit clean (or documented warnings)

### Review Requirements

Per docs/RELEASE_PROCESS.md:
- Requires 1+ maintainer approval
- All CI checks must pass
- Branch must be up to date with main

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Release initiated by: [Maintainer Name]
Release branch: $CURRENT_BRANCH
Target version: v$VERSION
EOF
)" \
  --base main \
  --head $CURRENT_BRANCH
```

**Capture PR number**:
```bash
PR_NUMBER=$(gh pr view --json number -q .number)
echo "Created PR #$PR_NUMBER"
```

### 6. Monitor CI Checks

Wait for CI checks to complete:
```bash
echo "Waiting for CI checks to complete..."
gh pr checks --watch
```

**If any checks fail**:
- Display failed check details
- Provide link to workflow run: `gh pr view --web`
- REFUSE to proceed with merge
- Instruct: "Fix the failing checks, commit, and re-run /caro.release.publish"

### 7. Request Manual Review

**If checks pass**:
```
✓ All CI checks passed

Waiting for maintainer review and approval.

PR URL: [display URL]

Next steps:
1. Request review from maintainer (if not automatic)
2. Wait for approval
3. This command will continue once approved
```

**Check PR review status**:
```bash
gh pr view --json reviewDecision -q .reviewDecision
```

**Wait for approval** (poll every 30 seconds):
- Display: "Waiting for PR approval... (check: N)"
- If approved: Continue to merge
- If changes requested: Exit with instructions to address feedback
- User can Ctrl+C to exit and re-run later

**Alternative (user preference)**:
- Ask: "Wait for approval automatically, or exit now? (wait/exit)"
- If exit: Display instructions to run command again after approval

### 8. Merge Pull Request

**Once approved**, merge the PR:
```bash
gh pr merge --squash --delete-branch
```

**If merge fails**:
- Check for conflicts
- Display error and resolution steps
- Exit without tagging

### 9. Switch to Main and Pull

Update local main branch:
```bash
git checkout main
git pull origin main
```

**Verify version in main**:
```bash
grep '^version = ' Cargo.toml | head -1
```

Expected: `version = "$VERSION"`

### 10. Create Annotated Tag

Create the release tag:
```bash
git tag -a "v$VERSION" -m "$(cat <<'EOF'
Release v$VERSION

[Paste release summary from CHANGELOG]

Published to crates.io: https://crates.io/crates/caro/$VERSION
GitHub Release: https://github.com/wildcard/caro/releases/tag/v$VERSION

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

### 11. Push Tag

Push the tag to trigger automated workflows:
```bash
git push origin "v$VERSION"
```

**This triggers**:
- `.github/workflows/publish.yml` - Publishes to crates.io
- `.github/workflows/release.yml` - Creates GitHub release

### 12. Monitor Publish Workflow

Monitor the crates.io publish workflow:
```bash
echo "Monitoring publish workflow..."
gh run watch
```

**Display workflow status**:
```bash
gh run list --workflow=publish.yml --limit 1
```

**If publish workflow fails**:
- Display error details
- Provide link to workflow run
- **CRITICAL**: Provide rollback instructions:
  ```
  ERROR: Publish workflow failed!

  Rollback steps:
  1. Delete the tag: git tag -d v$VERSION && git push origin :refs/tags/v$VERSION
  2. Fix the issue in the release branch
  3. Re-run /caro.release.publish

  Workflow URL: [URL]
  ```

### 13. Monitor Release Workflow

Monitor the GitHub release creation:
```bash
echo "Monitoring release workflow..."
gh run watch
```

**Verify GitHub release created**:
```bash
gh release view "v$VERSION"
```

### 14. Verify crates.io Publication

Poll crates.io for the new version:
```bash
echo "Verifying crates.io publication..."
for i in {1..10}; do
  PUBLISHED_VERSION=$(curl -s https://crates.io/api/v1/crates/caro | jq -r .crate.newest_version)
  if [ "$PUBLISHED_VERSION" = "$VERSION" ]; then
    echo "✓ Published to crates.io: $VERSION"
    break
  fi
  echo "Waiting for crates.io index update... (attempt $i/10)"
  sleep 30
done
```

**If not published after 5 minutes**:
- Display warning
- Provide manual verification link: https://crates.io/crates/caro
- Instruct to run `/caro.release.verify` manually

### 15. Output Success Summary

Display completion summary:
```
🎉 Release v$VERSION published successfully!

✓ PR #N merged to main
✓ Tag v$VERSION created and pushed
✓ Published to crates.io
✓ GitHub release created
✓ All workflows completed

Release URLs:
- crates.io: https://crates.io/crates/caro/$VERSION
- GitHub: https://github.com/wildcard/caro/releases/tag/v$VERSION

Next step:
Run /caro.release.verify to test the published release
```

---

## Error Recovery

### If Publish Fails After Tagging

**Scenario**: Tag was pushed but crates.io publish failed

**Recovery**:
1. Do NOT delete the tag (it's immutable per security settings)
2. Investigate publish failure in workflow logs
3. Fix the issue (usually CARGO_REGISTRY_TOKEN or version conflict)
4. Manually trigger publish workflow: `gh workflow run publish.yml`
5. Or use `cargo publish` manually with maintainer credentials

### If PR Merge Fails

**Scenario**: PR cannot be merged due to conflicts

**Recovery**:
1. Fetch latest main: `git checkout main && git pull`
2. Return to release branch: `git checkout $CURRENT_BRANCH`
3. Rebase on main: `git rebase main`
4. Resolve conflicts
5. Force push: `git push --force-with-lease`
6. Re-run `/caro.release.publish`

---

## References

- **Release Process**: `docs/RELEASE_PROCESS.md` (Tagging and Publishing section)
- **Publish Workflow**: `.github/workflows/publish.yml`
- **Release Workflow**: `.github/workflows/release.yml`
- **Security Settings**: `docs/SECURITY_SETTINGS.md` (Tag protection)
