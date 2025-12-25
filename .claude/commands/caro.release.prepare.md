---
description: Create release branch and run all pre-flight checks for caro release
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `docs/RELEASE_PROCESS.md`). Never refer to a folder by name alone.

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

**If NOT on `main` branch**:
- REFUSE to proceed with error:
  "ERROR: Must be on main branch. Current branch: $CURRENT_BRANCH"

**If working directory is not clean**:
- REFUSE to proceed with error:
  "ERROR: Working directory must be clean. Please commit or stash changes."

---

## Workflow Context

**This is the START** of the caro release workflow.

**After this command**:
1. You will be on a new release branch: `release/vX.Y.Z`
2. Run `/caro.release.security` to audit and fix vulnerabilities
3. Run `/caro.release.version` to bump version and update changelog
4. Run `/caro.release.publish` to create PR, merge, and tag
5. Run `/caro.release.verify` to test the published release

---

## Outline

### 1. Verify Prerequisites

Check that all required tools are installed:
```bash
which gh || echo "ERROR: gh CLI not installed"
which cargo || echo "ERROR: cargo not installed"
```

### 2. Pull Latest from Main

```bash
git pull origin main
```

### 3. Determine Release Version

**If user provided version via $ARGUMENTS**, use that.

**Otherwise, prompt the user**:
- Read current version from `Cargo.toml`:
  ```bash
  grep '^version = ' Cargo.toml | head -1
  ```
- Display: "Current version: X.Y.Z. What version are you releasing?"
- Wait for user input
- Validate format matches semantic versioning (vX.Y.Z)

### 4. Create Release Branch

```bash
git checkout -b release/vX.Y.Z
```

### 5. Check for Release Blockers

Use GitHub CLI to check for issues labeled `release-blocker`:
```bash
gh issue list --label release-blocker --state open
```

**If any blockers found**:
- Display them with details
- Ask: "Found N release blockers. Continue anyway? (y/n)"
- If user says no, abort and output: "Resolve blockers and run /caro.release.prepare again"

### 6. Verify CI Status on Main

Check that latest CI run on main passed:
```bash
gh run list --branch main --limit 5 --json conclusion,status,headSha
```

**If latest run failed or is in progress**:
- Warn: "WARNING: Latest CI on main is not green. Recommendation: wait for CI to pass."
- Ask: "Continue anyway? (y/n)"

### 7. List Pending Changes

Show what's changed since last release tag:
```bash
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
echo "Changes since $LAST_TAG:"
git log --oneline $LAST_TAG..HEAD
```

### 8. Output Next Steps

Display summary:
```
✓ Release branch created: release/vX.Y.Z
✓ Pre-flight checks passed

Next steps:
1. Run /caro.release.security to audit dependencies and fix vulnerabilities
2. Run /caro.release.version to bump version and update changelog
3. Run /caro.release.publish to create PR and publish release

For emergency hotfixes, use /caro.release.hotfix instead.
```

---

## References

- **Release Process**: `docs/RELEASE_PROCESS.md`
- **Security Settings**: `docs/SECURITY_SETTINGS.md`
- **Contributing Guide**: `CONTRIBUTING.md`
