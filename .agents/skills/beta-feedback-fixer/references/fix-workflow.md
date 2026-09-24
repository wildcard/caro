# Fix Workflow Reference

Step-by-step process for implementing fixes based on root cause analysis.

## Overview

This document describes the systematic workflow for fixing issues identified in beta testing:

1. Worktree Creation
2. Test-Driven Development
3. Implementation
4. Verification
5. Committing
6. PR Creation

## Phase 1: Worktree Creation

### Why Worktrees?

- **Isolation**: Fix bugs without affecting main branch
- **Clean baseline**: Start with passing tests
- **Easy cleanup**: Delete worktree when done
- **Multiple fixes**: Work on different issues in parallel

### Create Worktree

```bash
# Navigate to main repo
cd /path/to/caro

# Create worktree for beta.X P0 fixes
git worktree add .worktrees/fix-beta-X-p0-issues -b fix/beta-X-p0-issues

# Navigate to worktree
cd .worktrees/fix-beta-X-p0-issues

# Verify clean state
git status
```

### Branch Naming Convention

Pattern: `fix/beta-X-<severity>-issues`

Examples:
- `fix/beta-1-p0-issues` - Critical fixes for beta.1
- `fix/beta-2-p1-issues` - High priority fixes for beta.2
- `fix/beta-3-docs` - Documentation fixes for beta.3

### Verify Clean Baseline

```bash
# Build project
cargo build

# Run all tests
cargo test

# Expected: All tests pass
# If tests fail, investigate before starting fixes
```

**Why this matters**: Need to know which test failures are pre-existing vs introduced by your changes.

## Phase 2: Test-Driven Development

### Write Regression Test First

**Benefits**:
- Confirms you understand the issue
- Provides verification that fix works
- Prevents bug from returning

**Test location**: `tests/beta_regression.rs`

### Test Structure

```rust
/// Issue #{ID} Regression Test: {Brief description}
///
/// Root cause: {One-line explanation}
/// Fix: {One-line explanation}
#[tokio::test]
async fn test_issue_{id}_{brief_name}() {
    // Step 1: Setup
    // Create minimal environment to reproduce

    // Step 2: Execute
    // Run the operation that was broken

    // Step 3: Assert
    // Verify expected behavior

    // Step 4: Cleanup (if needed)
}
```

### Example: Telemetry Persistence Test

```rust
/// Issue #402/#403 Regression Test
///
/// Root cause: Config loaded but never saved after consent
/// Fix: Added config_manager.save() call
#[tokio::test]
async fn test_telemetry_consent_persists() {
    // Setup: Create temp config
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Execute: Simulate first run with consent
    let mut config = Config::default();
    config.telemetry.first_run = true;

    // User accepts telemetry
    config.telemetry.enabled = true;
    config.telemetry.first_run = false;

    // Save config (this is what was missing)
    let manager = ConfigManager::with_path(&config_path);
    manager.save(&config).expect("Failed to save config");

    // Assert: Load config again, verify persistence
    let loaded_config = manager.load().expect("Failed to load config");
    assert_eq!(loaded_config.telemetry.first_run, false);
    assert_eq!(loaded_config.telemetry.enabled, true);
}
```

### Run Test (Should Fail)

```bash
# Run new test
cargo test test_telemetry_consent_persists

# Expected: FAILED
# Confirms the bug exists
```

If test passes before fix: Test is wrong or bug doesn't exist.

## Phase 3: Implementation

### Locate the Code

From root cause analysis, you know:
- File: `src/main.rs`
- Line: Around 627-636
- Problem: Missing `config_manager.save()`

### Read Existing Code

```bash
# Read the problematic section
head -n 650 src/main.rs | tail -n 30
```

### Implement Minimal Fix

**Principle**: Change only what's necessary to fix the root cause.

```rust
// BEFORE (broken)
if user_config.telemetry.first_run {
    if caro::telemetry::consent::prompt_consent() {
        // User accepted telemetry
        // TODO: This would require updating the config file...
    } else {
        // User declined telemetry - update config to disable it
    }
}

// AFTER (fixed)
if user_config.telemetry.first_run {
    let consent = caro::telemetry::consent::prompt_consent();

    // Update config
    user_config.telemetry.first_run = false;
    user_config.telemetry.enabled = consent;

    // Persist to disk (this was missing)
    if let Some(ref cm) = config_manager {
        if let Err(e) = cm.save(&user_config) {
            tracing::warn!("Failed to save telemetry preferences: {}", e);
        }
    }

    // Show confirmation
    if consent {
        caro::telemetry::consent::show_enabled_message();
    } else {
        caro::telemetry::consent::show_disabled_message();
    }
}
```

### Add Explanatory Comments

```rust
// Persist config to disk
// Previously, consent was captured but never saved,
// causing the prompt to appear on every run (Issue #402)
if let Some(ref cm) = config_manager {
    if let Err(e) = cm.save(&user_config) {
        tracing::warn!("Failed to save telemetry preferences: {}", e);
    }
}
```

### Check for Related Issues

Issue #403 (can't disable telemetry) has same root cause → same fix resolves both.

Issue #404 (JSON pollution) has different root cause → needs separate fix.

## Phase 4: Verification

### Run Regression Test

```bash
# Test should now pass
cargo test test_telemetry_consent_persists

# Expected: ok
```

### Run Full Test Suite

```bash
# All tests should still pass
cargo test

# Check for regressions
# Expected: 148 tests passing (or whatever your baseline was)
```

### Manual Verification

Follow original reproduction steps:

```bash
# Clean config
rm -f ~/.config/caro/config.toml

# First run - should show prompt
caro "list files"
# → Telemetry prompt appears
# → User chooses option

# Second run - should NOT show prompt
caro "list files"
# → No prompt
# → Command executes

# Verify config persisted
cat ~/.config/caro/config.toml
# Should show:
# [telemetry]
# enabled = true/false
# first_run = false
```

### Edge Case Testing

```bash
# Test with different choices
rm ~/.config/caro/config.toml
caro "list files"  # Accept
cat ~/.config/caro/config.toml  # Should show enabled=true

rm ~/.config/caro/config.toml
caro "list files"  # Decline
cat ~/.config/caro/config.toml  # Should show enabled=false

# Test that choice persists
caro "show disk"  # No prompt
caro "find files"  # Still no prompt
```

## Phase 5: Committing

### Atomic Commits

**One logical fix per commit**, even if it fixes multiple related issues.

**Examples of atomic commits**:
- ✅ "Fix telemetry persistence (Issues #402, #403)"
- ✅ "Fix JSON output pollution (Issue #404)"
- ❌ "Fix all P0 issues" (too broad)

### Commit Message Format

```
{type}({scope}): {brief description}

{Detailed explanation of root cause}
{How the fix works}

Fixes: #{issue_id}

{Verification steps}
{Additional context}

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

### Example Commit Message

```
fix(telemetry): Persist consent choice to config

Root cause: src/main.rs:627-636 had empty TODO branches where
consent was captured but config_manager.save() was never called.
This caused first_run to remain true, showing the prompt on every
invocation.

Fix: Added config update and save call after consent prompt.
- Set first_run = false
- Set enabled = consent_result
- Call config_manager.save()

Fixes: #402, #403

Verification:
1. Delete ~/.config/caro/config.toml
2. Run `caro "list files"` → prompt appears
3. Run `caro "list files"` → no prompt
4. Check config file → first_run=false, enabled=<choice>

Regression test: test_telemetry_consent_persists

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

### Commit Types

- `fix`: Bug fix
- `feat`: New feature
- `docs`: Documentation only
- `test`: Adding tests
- `refactor`: Code restructuring without behavior change
- `perf`: Performance improvement
- `style`: Formatting, whitespace
- `chore`: Build, dependencies, tooling

### Commit Scopes

- `telemetry`: Telemetry system
- `config`: Configuration management
- `backend`: Backend/inference
- `safety`: Safety validation
- `cli`: Command-line interface
- `docs`: Documentation

### Stage and Commit

```bash
# Stage the changes
git add src/main.rs tests/beta_regression.rs

# Commit with message
git commit -m "$(cat <<'EOF'
fix(telemetry): Persist consent choice to config

Root cause: Config loaded but never saved after consent.
Fix: Added config_manager.save() call.

Fixes: #402, #403

Regression test: test_telemetry_consent_persists

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"

# Verify commit
git log -1 --stat
```

## Phase 6: PR Creation

### Push Branch

```bash
# Push branch to remote
git push origin fix/beta-1-p0-issues

# If creating PR from this commit
git push -u origin HEAD
```

### Create PR

Use GitHub CLI:
```bash
gh pr create \
  --base release/v1.1.0 \
  --head fix/beta-1-p0-issues \
  --title "fix(beta): Fix all P0 issues from v1.1.0-beta.1 testing" \
  --body "$(cat <<'EOF'
## Summary

Fixes all 5 critical P0 issues identified in comprehensive beta testing.

**Fixes**: #402, #403, #404, #405, #406

## Issues Fixed

### Issue #402 & #403: Telemetry consent persistence
- ✅ FIXED: Consent now persists, prompt shows once
- Root cause: Config loaded but never saved
- Fix: Added config_manager.save() call

### Issue #404: JSON output pollution
- ✅ FIXED: JSON output is valid
- Root cause: Interactive prompt in non-interactive mode
- Fix: Skip prompt when --output json/yaml

[... details for other issues ...]

## Test Coverage

### Regression Tests
- ✅ test_issue_402_telemetry_persistence
- ✅ test_issue_404_json_output

All 5 regression tests pass ✅

### Full Test Suite
- ✅ 148 library tests pass
- ✅ No regressions

## Manual Verification

[Commands to manually verify fixes]

🤖 Generated with Claude Code
EOF
)"
```

### PR Description Template

See the SKILL.md main file for the full PR template.

**Key sections**:
1. Summary (one-sentence overview)
2. Issues fixed (one section per issue with root cause)
3. Test coverage (regression + full suite)
4. Impact metrics (before/after)
5. Manual verification steps

## Best Practices

### DO

- ✅ Write regression test before fix
- ✅ Make minimal changes (only fix root cause)
- ✅ Test edge cases manually
- ✅ Commit with descriptive messages
- ✅ Reference issue IDs in commits
- ✅ Run full test suite before pushing

### DON'T

- ❌ Fix without regression test
- ❌ Refactor unrelated code while fixing
- ❌ Commit broken/failing tests
- ❌ Use vague commit messages ("fix bug")
- ❌ Batch multiple unrelated fixes in one commit
- ❌ Skip manual verification

## Troubleshooting

### Test Still Fails After Fix

**Possible causes**:
1. Test is wrong (doesn't test what you think)
2. Fix is incomplete
3. Fix is in wrong place
4. Need to rebuild: `cargo clean && cargo build`

**Debug**:
```bash
# Run with output
cargo test test_name -- --nocapture

# Run with logging
RUST_LOG=debug cargo test test_name
```

### Full Test Suite Fails

**Possible causes**:
1. Fix broke something else (regression)
2. Test is flaky
3. Test depends on system state

**Debug**:
```bash
# Run failing test individually
cargo test failing_test_name -- --nocapture

# Check git diff
git diff

# Revert and try again
git checkout -- src/file.rs
```

### Manual Verification Fails

**Possible causes**:
1. Cached config interfering
2. Wrong environment
3. Test setup different from reality

**Debug**:
```bash
# Clean all caches
rm -rf ~/.config/caro
rm -rf ~/.cache/caro

# Run in temp directory
cd /tmp/test-caro
caro "command"
```

## Summary Checklist

Before creating PR:

- [ ] Regression test written and passing
- [ ] Full test suite passing
- [ ] Manual verification completed
- [ ] Code changes are minimal
- [ ] Comments explain the fix
- [ ] Commit message is descriptive
- [ ] Related issues are referenced
- [ ] Branch pushed to remote
- [ ] PR description is comprehensive

## Resources

- Root cause analysis: `issue-analysis.md`
- PR template: See SKILL.md
- Commit message examples: This document
- Regression test examples: `../../tests/beta_regression.rs`
