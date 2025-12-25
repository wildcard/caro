---
description: Review and merge Dependabot PRs with breaking change analysis
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `Cargo.toml`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

**If user provides specific action** (e.g., "merge all", "investigate #123"):
- Execute that specific action
- Otherwise, perform full review workflow

---

## Pre-flight Check (OPTIONAL)

**This command does NOT enforce branch requirements.**

You can run this command from any branch to review Dependabot updates.

**Requirements:**
- `gh` CLI must be authenticated: `gh auth status`
- If not authenticated, display: "ERROR: GitHub CLI not authenticated. Run: gh auth login"

---

## Workflow Context

**Use this command when:**
- Dependabot PRs are pending review
- After security alerts are resolved
- Before preparing a release

**Workflow integration:**
1. `/caro.deps.review` - Review and merge dependency updates
2. `/caro.release.prepare` - Start release workflow

---

## Outline

### 1. Fetch Open Dependabot PRs

**Get all Dependabot PRs:**
```bash
gh pr list --author app/dependabot --json number,title,headRefName,url --limit 100
```

**If no PRs found:**
- Display: "No open Dependabot PRs found."
- Exit with success

**Otherwise:**
- Display count: "Found X open Dependabot PRs"
- Continue to categorization

### 2. Categorize Updates

**Parse PR titles** to extract:
- Package name
- Version change (from → to)
- Update type (major, minor, patch)

**Categorize PRs into groups:**
1. **Major Updates** - Breaking changes (X.0.0 version bump)
   - Examples: `thiserror 1.0 → 2.0`, `directories 5.0 → 6.0`
   - Requires manual investigation

2. **Minor/Patch Updates** - Safe updates (0.X.0 or 0.0.X bumps)
   - Examples: `clap 4.5.1 → 4.5.20`, `tokio 1.38 → 1.42`
   - Generally safe to merge

3. **GitHub Actions Updates** - Workflow dependencies
   - Examples: `actions/checkout@v4 → @v6`
   - Safe to merge after CI passes

**Display categorized list:**
```
Major Updates (requires investigation):
  #110: Bump thiserror from 1.0.69 to 2.0.16
  #109: Bump sysinfo from 0.29.11 to 0.37.2

Minor/Patch Updates (safe to merge):
  #125: Bump clap from 4.5.1 to 4.5.20
  #124: Bump tokio from 1.38.0 to 1.42.0

GitHub Actions Updates (safe to merge):
  #122: Bump actions/checkout from v4 to v6
```

### 3. Investigate Major Updates

For each **major update**, perform breaking change analysis:

**Step 3.1: Identify affected dependency**
Extract package name from PR title (e.g., "thiserror" from "Bump thiserror from 1.0 to 2.0")

**Step 3.2: Search for usage in codebase**
```bash
# Search for imports
rg "use.*<package_name>" src/

# Search for macro/API usage
rg "<package_name>::" src/

# Search in Cargo.toml
rg "^<package_name>" Cargo.toml
```

**Step 3.3: Analyze results**
- **If no matches found in `src/`**:
  - Package is unused or dev-only dependency
  - Display: "✓ Package '<name>' not used in source code (safe to merge)"
  - Mark as safe

- **If matches found**:
  - Display file paths and line numbers
  - Read relevant files to check API usage
  - Look for patterns that might break:
    - Macro syntax changes (e.g., `#[error(...)]` in thiserror)
    - API signature changes
    - Trait implementations
  - Provide assessment:
    - "✓ No breaking changes detected - safe to merge"
    - "⚠ Potential breaking changes - manual review needed"

**Step 3.4: Provide merge recommendation**
For each major update:
- ✓ Safe to merge (no usage or compatible API)
- ⚠ Requires manual review (detected potential issues)
- ℹ  Recommendation: [specific guidance]

### 4. Execute Merges (Interactive)

**Ask user for confirmation:**
"Ready to merge safe PRs? The following will be merged:
- Minor/Patch updates: X PRs
- GitHub Actions: X PRs
- Major updates (verified safe): X PRs

Proceed with merge? (y/n)"

**If user confirms:**

**Step 4.1: Merge PRs in batches**
```bash
# Merge minor/patch updates
for pr in <safe_pr_numbers>; do
  echo "Merging PR #$pr..."
  gh pr merge "$pr" --squash --auto
done
```

**Step 4.2: Monitor CI status**
For each merged PR:
```bash
# Wait for merge to complete
gh pr view "$pr" --json state

# Check CI status on main
gh run list --branch main --limit 1 --json conclusion
```

**Step 4.3: Handle CI failures**

**If CI fails:**
1. Identify failing workflow:
```bash
gh run list --branch main --limit 1 --json name,conclusion,url
```

2. Check for flaky tests:
   - If Windows test failure: Rerun with `gh run rerun <run_id> --failed`
   - If clippy/lint error: Investigate and fix

3. If legitimate failure:
   - Display error: "CI failed after merging PR #X"
   - Display logs: `gh run view <run_id> --log-failed`
   - Recommend action: Create fix in feature branch

**If CI passes:**
- Display: "✓ All merges successful, CI passing on main"

### 5. Update CHANGELOG (Optional)

**If multiple PRs were merged:**

Ask user: "Update CHANGELOG.md with dependency changes? (y/n)"

**If yes:**
1. Read current `CHANGELOG.md`
2. Find `## [Unreleased]` section
3. Add dependency update entry:
```markdown
### Changed

#### Dependencies
- Updated X dependencies via Dependabot:
  - `thiserror`: 1.0.69 → 2.0.16
  - `clap`: 4.5.1 → 4.5.20
  - See merged PRs: #110, #125
```
4. Save and display changes

**If no:**
- Skip changelog update
- User can update manually later

### 6. Output Summary

Display comprehensive summary:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Dependabot PR Review Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ MERGED (8 PRs)
  • Minor/Patch: 5 PRs (#125, #124, #123, #121, #120)
  • GitHub Actions: 2 PRs (#122, #119)
  • Major (verified safe): 1 PR (#110)

⚠ REQUIRES REVIEW (1 PR)
  • #109: Bump sysinfo 0.29 → 0.37
    Reason: API changes detected in src/execution/

⏭ SKIPPED (0 PRs)

🔍 NEXT STEPS:
  1. Review remaining PRs manually: #109
  2. Run cargo check to verify builds
  3. Consider running full test suite
  4. Ready for release? Run: /caro.release.prepare

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 7. Verify Build After Merges

**Run basic verification:**
```bash
echo "Verifying build after dependency updates..."
cargo check --all-features
```

**If build fails:**
- Display error output
- Recommend: "Build failed after merging dependencies. Investigate with: cargo build --verbose"

**If build succeeds:**
- Display: "✓ Build verification passed"

---

## Troubleshooting

### PR Merge Conflicts

**If merge conflict detected:**
```
ERROR: PR #123 has merge conflicts
Resolution: Rebase the PR branch manually or close and recreate
```

**Steps:**
1. Check conflict details: `gh pr view <number>`
2. If Dependabot PR, close it: `gh pr close <number>`
3. Dependabot will auto-recreate with resolved conflicts

### CI Timeout

**If CI takes too long:**
- Display: "CI running for >5 minutes, stopping watch"
- User can monitor manually: `gh run watch`

### Authentication Failures

**If gh auth fails:**
```
ERROR: GitHub CLI authentication expired
Run: gh auth login
Then retry: /caro.deps.review
```

---

## References

- **Dependabot docs**: https://docs.github.com/en/code-security/dependabot
- **Release process**: `docs/RELEASE_PROCESS.md`
- **Security policy**: `SECURITY.md`
