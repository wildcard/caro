---
description: Emergency hotfix workflow for critical security patches
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `Cargo.toml`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

**Expected input**: Brief description of the security issue or bug being hotfixed

**Example**: "/caro.release.hotfix RUSTSEC-2025-0001 critical memory safety issue"

---

## Branch Pre-flight Check (EMERGENCY MODE)

**This command can start from ANY branch** due to emergency nature.

**However**, it will:
1. Fetch latest tags
2. Create a hotfix branch from the latest release tag
3. Apply minimal fix
4. Fast-track through release process

**WARNING**: This bypasses normal development workflow. Use ONLY for:
- Critical security vulnerabilities
- Data loss bugs
- System crash bugs
- Actively exploited vulnerabilities

---

## Workflow Context

**Emergency Workflow**: Bypasses normal feature development cycle

**Purpose**: Rapid patch release for critical issues

**After this**: Hotfix must be backported to main and any active release branches

---

## Outline

### 1. Verify Emergency Justification

**Display warning**:
```
⚠️  EMERGENCY HOTFIX WORKFLOW ⚠️

This workflow is for CRITICAL ISSUES ONLY:
  • Security vulnerabilities (RUSTSEC advisories)
  • Data loss or corruption bugs
  • System crashes or panics
  • Actively exploited vulnerabilities

For non-critical bugs, use the normal release workflow:
  /caro.release.prepare

Continue with hotfix? (yes/no - must type 'yes')
```

**Require explicit confirmation**:
- User must type "yes" (not just "y")
- If anything else, exit with instructions to use normal workflow

### 2. Identify Base Version

**Fetch all tags**:
```bash
git fetch --tags
```

**Get latest release tag**:
```bash
LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null)
if [ -z "$LATEST_TAG" ]; then
  echo "ERROR: No release tags found"
  echo "Cannot create hotfix without a base release"
  exit 1
fi

echo "Latest release: $LATEST_TAG"
```

**Parse version**:
```bash
BASE_VERSION=${LATEST_TAG#v}
echo "Base version: $BASE_VERSION"
```

**Prompt for hotfix version**:
```
Current latest: v$BASE_VERSION

Hotfix version will be: v$BASE_VERSION patch bump

Examples:
  v1.0.0 → v1.0.1 (patch bump)
  v1.2.3 → v1.2.4 (patch bump)

This hotfix will be: v[calculated]

Proceed? (y/n)
```

**Calculate hotfix version**:
- Parse major.minor.patch from BASE_VERSION
- Increment patch number
- Store as HOTFIX_VERSION

### 3. Create Hotfix Branch

**Create branch from release tag**:
```bash
git checkout -b "hotfix/v$HOTFIX_VERSION" "$LATEST_TAG"
```

**Verify branch created**:
```bash
CURRENT_BRANCH=$(git branch --show-current)
echo "Created hotfix branch: $CURRENT_BRANCH"
```

**If branch already exists**:
- Ask: "Hotfix branch already exists. Continue on existing branch? (y/n)"
- If yes: Checkout existing branch
- If no: Exit with instructions

### 4. Document the Issue

**Create or update SECURITY.md**:
- If security vulnerability, document in SECURITY.md
- Include CVE number if available
- Include RUSTSEC advisory ID if applicable
- Document impact and severity

**Prepare hotfix description**:
```
Hotfix v$HOTFIX_VERSION

Issue: [User-provided description from $ARGUMENTS]

Severity: [Prompt user: Critical/High/Medium]

Impact:
- [Prompt user for impact description]

Fix:
- [Will be documented after fix is applied]
```

### 5. Apply Minimal Fix

**This is the manual step**:
```
Apply the MINIMAL fix required to address the issue.

Guidelines:
  • Change only what's necessary to fix the issue
  • Avoid refactoring or "improvements"
  • Minimize risk of introducing new bugs
  • Focus on stability over elegance

Files to modify:
  [User identifies which files need changes]

After making changes, commit them:
  git add [files]
  git commit -m "hotfix: [description]"

Type 'done' when fix is committed:
```

**Wait for user confirmation**:
- User types "done" when fix is committed
- Verify commit exists: `git log -1 --oneline`

### 6. Update Version and Changelog

**Update Cargo.toml**:
```bash
# Update version to HOTFIX_VERSION
sed -i '' "s/^version = .*/version = \"$HOTFIX_VERSION\"/" Cargo.toml
```

**Verify version update**:
```bash
grep '^version = ' Cargo.toml | head -1
```

**Update CHANGELOG.md**:
- Add hotfix section at the top (after [Unreleased])
- Document the security fix
- Include CVE/RUSTSEC IDs if applicable

**Example CHANGELOG entry**:
```markdown
## [X.Y.Z] - YYYY-MM-DD [SECURITY HOTFIX]

### Security
- **CRITICAL**: Fixed [vulnerability description]
  - CVE-YYYY-NNNNN / RUSTSEC-YYYY-NNNN
  - Impact: [description]
  - Severity: [Critical/High]
  - Credit: [Reporter name if applicable]

### Fixed
- [Bug description if non-security]

**Upgrade immediately**: This release fixes a critical security vulnerability.
All users should upgrade as soon as possible.
```

**Commit version bump**:
```bash
git add Cargo.toml CHANGELOG.md SECURITY.md
git commit -m "chore: Bump version to $HOTFIX_VERSION for security hotfix"
```

### 7. Run Tests

**Critical**: Tests must pass before hotfix release:
```bash
echo "Running tests..."
cargo test --lib
cargo test --test '*'
```

**If tests fail**:
- Display failures
- **DO NOT proceed**
- Fix test failures
- Re-run tests until all pass

**If tests pass**:
```
✓ All tests passed

Test summary:
  • N unit tests passed
  • N integration tests passed
```

### 8. Quick Security Audit

**Run cargo audit** (abbreviated):
```bash
cargo audit
```

**If new vulnerabilities found**:
- Display them
- Ask: "New vulnerabilities found. These should be fixed if possible. Continue anyway? (y/n)"
- Document in CHANGELOG.md if proceeding

### 9. Fast-Track PR

**Push hotfix branch**:
```bash
git push -u origin "hotfix/v$HOTFIX_VERSION"
```

**Create URGENT PR**:
```bash
gh pr create \
  --title "🚨 SECURITY HOTFIX: v$HOTFIX_VERSION" \
  --body "$(cat <<'EOF'
## 🚨 SECURITY HOTFIX v$HOTFIX_VERSION

### Critical Issue

[User-provided description]

**Severity**: Critical/High
**Impact**: [Description]

### Fix Applied

[Description of fix]

### Security Advisory

- CVE/RUSTSEC: [If applicable]
- Affected versions: < v$HOTFIX_VERSION
- Fixed in: v$HOTFIX_VERSION

### Testing

- [x] All unit tests pass
- [x] All integration tests pass
- [x] Manual verification completed
- [x] Security audit clean (or documented)

### Fast-Track Justification

This hotfix addresses a **critical security vulnerability** and requires
immediate release per docs/RELEASE_PROCESS.md emergency procedures.

**Maintainer approval required before merge.**

---

⚠️  URGENT: Please review and approve ASAP

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Hotfix initiated by: [Maintainer]
Base version: v$BASE_VERSION
Hotfix version: v$HOTFIX_VERSION
EOF
)" \
  --base main \
  --head "hotfix/v$HOTFIX_VERSION" \
  --label security,urgent,hotfix
```

### 10. Request Immediate Review

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  🚨 HOTFIX PR CREATED - IMMEDIATE REVIEW REQUIRED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

PR: [URL]

Next steps:
1. Notify maintainers immediately (Slack, email, etc.)
2. Request fast-track review
3. Wait for approval (monitoring PR status)
4. After approval, this command will merge and publish

Monitoring PR status...
```

**Monitor PR checks**:
```bash
gh pr checks --watch
```

**Wait for approval** (with shorter timeout than normal releases):
- Poll every 15 seconds (instead of 30)
- Display urgent reminders every 5 minutes
- After approval, continue automatically

### 11. Merge and Tag

**After approval**:
```bash
gh pr merge --squash --delete-branch
```

**Switch to main and pull**:
```bash
git checkout main
git pull origin main
```

**Create annotated tag**:
```bash
git tag -a "v$HOTFIX_VERSION" -m "$(cat <<'EOF'
Security Hotfix v$HOTFIX_VERSION

CRITICAL SECURITY FIX

[Summary from CHANGELOG]

Published to crates.io: https://crates.io/crates/caro/$HOTFIX_VERSION

⚠️  All users should upgrade immediately.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

**Push tag**:
```bash
git push origin "v$HOTFIX_VERSION"
```

### 12. Monitor Publish

**Monitor workflows** (same as normal release):
```bash
gh run watch
```

**Verify publication**:
```bash
# Poll crates.io
for i in {1..10}; do
  PUBLISHED=$(curl -s https://crates.io/api/v1/crates/caro | jq -r .crate.newest_version)
  if [ "$PUBLISHED" = "$HOTFIX_VERSION" ]; then
    echo "✓ Published to crates.io"
    break
  fi
  sleep 20
done
```

### 13. Publish Security Advisory

**If security vulnerability**:
```
Create GitHub Security Advisory:

  gh api repos/wildcard/caro/security-advisories \
    -X POST \
    -f summary="[Vulnerability summary]" \
    -f description="[Full description]" \
    -f severity="critical" \
    -f cve_id="CVE-YYYY-NNNNN"

Or manually:
  https://github.com/wildcard/caro/security/advisories/new
```

### 14. Announce Hotfix

**Prepare announcement**:
```
🚨 SECURITY HOTFIX: caro v$HOTFIX_VERSION

A critical security vulnerability has been fixed in caro.

**Affected versions**: All versions < v$HOTFIX_VERSION
**Fixed in**: v$HOTFIX_VERSION

**Vulnerability**: [Description]
**Severity**: Critical/High
**CVE/RUSTSEC**: [ID if applicable]

**Immediate action required**: Upgrade to v$HOTFIX_VERSION

  cargo install caro --version $HOTFIX_VERSION --force

**Details**: https://github.com/wildcard/caro/security/advisories/[ID]

Thank you to [Reporter] for responsible disclosure.
```

**Channels to notify**:
- [ ] GitHub Security Advisory
- [ ] GitHub Discussions (announcement)
- [ ] crates.io release notes
- [ ] Social media (if applicable)
- [ ] Email to known users (if applicable)

### 15. Backport to Development

**Important**: Hotfix must be merged back to any active development:
```bash
# If there are active release branches, cherry-pick the fix
git checkout main
git cherry-pick "hotfix/v$HOTFIX_VERSION"

# Or merge if no conflicts
git merge "hotfix/v$HOTFIX_VERSION"
```

### 16. Output Summary

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  🚨 SECURITY HOTFIX v$HOTFIX_VERSION PUBLISHED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ Hotfix applied and tested
✓ Version bumped to v$HOTFIX_VERSION
✓ PR merged to main
✓ Tag created and pushed
✓ Published to crates.io
✓ GitHub release created

⚠️  NEXT STEPS REQUIRED:

1. Publish GitHub Security Advisory
2. Announce on GitHub Discussions
3. Update SECURITY.md if needed
4. Notify users through appropriate channels
5. Monitor for issues

Release URLs:
  • crates.io: https://crates.io/crates/caro/$HOTFIX_VERSION
  • GitHub: https://github.com/wildcard/caro/releases/tag/v$HOTFIX_VERSION
  • Advisory: [Create manually if not automated]

Run /caro.release.verify to verify the hotfix installation.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Emergency Rollback

### If Hotfix Introduces Regression

**CRITICAL**: If hotfix causes worse problems than the original issue:

1. **Yank the version from crates.io**:
   ```bash
   cargo yank --version $HOTFIX_VERSION
   ```

2. **Delete the GitHub release** (keep tag for audit):
   ```bash
   gh release delete "v$HOTFIX_VERSION"
   ```

3. **Publish rollback advisory**:
   - Explain the regression
   - Recommend downgrade to previous version
   - Provide timeline for fixed hotfix

4. **Create new hotfix**:
   - Increment patch again (v1.0.2 → v1.0.3)
   - Fix both original issue AND regression
   - Follow this workflow again

---

## References

- **Release Process**: `docs/RELEASE_PROCESS.md` (Emergency Procedures section)
- **Security Policy**: `SECURITY.md`
- **RustSec Database**: https://rustsec.org/
- **GitHub Security Advisories**: https://docs.github.com/en/code-security/security-advisories
