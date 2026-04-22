---
description: Bump version and update changelog for caro release
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `Cargo.toml`). Never refer to a folder by name alone.

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
```

**If NOT on a release branch** (`release/v*` or `hotfix/v*`):
- REFUSE to proceed with error:
  "ERROR: Must be on a release branch. Current: $CURRENT_BRANCH
  Run /caro.release.prepare first."

---

## Workflow Context

**Before this**: `/caro.release.security` fixed vulnerabilities and updated dependencies

**This command**: Bumps version in Cargo.toml and updates CHANGELOG.md

**After this**: Run `/caro.release.publish` to create PR, merge, and tag

---

## Outline

### 1. Extract Target Version

Parse the version from the current branch name:
```bash
CURRENT_BRANCH=$(git branch --show-current)
VERSION=${CURRENT_BRANCH#release/v}
VERSION=${VERSION#hotfix/v}
echo "Target version: $VERSION"
```

**Validate format**: Must match semantic versioning (X.Y.Z)

If version cannot be extracted or is invalid:
- Display error: "ERROR: Cannot extract valid version from branch: $CURRENT_BRANCH"
- Exit with instructions to use proper branch naming

### 2. Read Current Version

Check current version in `Cargo.toml`:
```bash
grep '^version = ' Cargo.toml | head -1
```

Display: "Current version in Cargo.toml: X.Y.Z"
Display: "Target version from branch: X.Y.Z"

**If versions already match**:
- Ask: "Version is already set to X.Y.Z. Continue anyway? (y/n)"
- If no, exit with message: "Version bump skipped."

### 3. Update Cargo.toml

Update the version field in `Cargo.toml`:
```bash
# Use sed or direct file edit
```

**Verify the change**:
```bash
grep '^version = ' Cargo.toml | head -1
```

Expected output: `version = "X.Y.Z"`

### 4. Update CHANGELOG.md

**Read current CHANGELOG.md** to understand structure.

**Transformation required**:
1. Find the `## [Unreleased]` section
2. Rename it to `## [X.Y.Z] - YYYY-MM-DD` (use today's date)
3. Add a new `## [Unreleased]` section at the top
4. Ensure the new release section has proper content

**New Unreleased section template**:
```markdown
## [Unreleased]

### Added

### Changed

### Fixed

### Security
```

**Example transformation**:
```markdown
# Before:
## [Unreleased]

### Added
- New feature X
- New feature Y

### Fixed
- Bug fix Z

# After:
## [Unreleased]

### Added

### Changed

### Fixed

### Security

## [X.Y.Z] - 2025-01-15

### Added
- New feature X
- New feature Y

### Fixed
- Bug fix Z
```

**If [Unreleased] section is empty or missing**:
- Warn: "WARNING: No unreleased changes found in CHANGELOG.md"
- Ask: "Continue with empty release notes? (y/n)"
- If no, exit with instructions to update CHANGELOG.md first

### 5. Update User-Facing Version References (MANDATORY)

**Per `.claude/rules/release-version-alignment.md`, every release PR must update
all version references in a single commit.** The following files drift silently
if not touched — v1.3.0 shipped with README showing `1.1.1`, homebrew tap
example at `VERSION=1.1.0`, and nuget installer defaulting to `1.1.0`, requiring
a post-release alignment PR to fix.

Update these files in addition to `Cargo.toml` and `CHANGELOG.md`:

1. **`README.md`** — find the line `**Current Version:** <old>` (or similar
   landing-page version banner) and bump to `X.Y.Z`. Search for any other
   version strings in the README header block.

2. **`ROADMAP.md`** — three edits:
   - Bump `**Last Updated**: <date>` near the top to today.
   - In the status / milestone table, either flip the existing `vX.Y.Z` row
     to `✅ **RELEASED**` with the release date, or prepend a new row
     newest-first.
   - Prepend a new `### 🎉 vX.Y.Z - <headline>` section at the top of
     `## Release Milestones` summarising what shipped, with a link to the
     feature PR(s).

3. **`homebrew-tap/README.md`** — in the "Computing SHA256 Checksums" code
   block, bump `VERSION=X.Y.Z` so the example matches this release.

4. **`nuget/tools/install.ps1`** — bump the default parameter value
   `[string]$Version = "X.Y.Z"` on line 3 so fresh NuGet installs pull the
   current release by default.

**Scan for any additional version references** that may have been added since
this checklist was written:
```bash
# Any file outside .claude/releases/ still showing the previous version?
git grep -l "<old-version>" -- ':!.claude/releases/' ':!specs/' ':!kitty-specs/'
```

### 6. Update Version Documentation (if needed)

**Check if `docs/RELEASE_PROCESS.md` needs updates**:
- Read the file
- Look for version-specific references
- If release process changed during this release cycle, update accordingly

**This step is optional** - only update if procedures changed.

### 7. Verify Build

Run `cargo check` to ensure Cargo.toml is valid:
```bash
cargo check
```

**If check fails**:
- Display error output
- REFUSE to proceed
- Instruct user to fix issues and re-run

### 8. Review Changes

Display summary of changes:
```bash
git diff Cargo.toml Cargo.lock CHANGELOG.md README.md ROADMAP.md \
        homebrew-tap/README.md nuget/tools/install.ps1
```

Show clear summary:
```
Version bump summary:
- Cargo.toml:              X.Y.Z-old → X.Y.Z
- Cargo.lock:              regenerated
- CHANGELOG.md:            Added [X.Y.Z] section with N changes
- README.md:               Current Version banner bumped
- ROADMAP.md:              Status table + milestone section updated
- homebrew-tap/README.md:  VERSION= example bumped
- nuget/install.ps1:       Default -Version bumped
- Build verification:      ✓ Passed
```

**Ask for confirmation**:
"Commit these changes? (y/n)"

If no, exit without committing.

### 9. Commit Changes

Create a well-formatted commit staging ALL aligned files:
```bash
git add Cargo.toml Cargo.lock CHANGELOG.md README.md ROADMAP.md \
        homebrew-tap/README.md nuget/tools/install.ps1 docs/
git commit -m "$(cat <<'EOF'
chore: Bump version to X.Y.Z

Updated version in Cargo.toml and aligned all user-facing version
references (README banner, ROADMAP milestone+status, homebrew tap
example, nuget installer default). Moved unreleased changes to
CHANGELOG.md release section.

Release notes:
- N features added
- N bugs fixed
- N security updates

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

### 10. Output Summary

Display next steps:
```
✓ Version bumped to X.Y.Z
✓ CHANGELOG.md updated
✓ Build verification passed
✓ Changes committed

Next step:
Run /caro.release.publish to create PR, merge, and tag the release
```

---

## References

- **Release Process**: `docs/RELEASE_PROCESS.md` (Version Management section)
- **Semantic Versioning**: https://semver.org/
- **Keep a Changelog**: https://keepachangelog.com/
