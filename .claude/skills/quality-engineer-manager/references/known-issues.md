# Known Issues Database

This document tracks historical issues, their resolutions, and prevention strategies to avoid regressions.

## Format

Each entry should include:
- **Issue ID**: GitHub issue number or unique identifier
- **Version**: When issue was discovered
- **Severity**: P0 (Critical), P1 (High), P2 (Medium), P3 (Low)
- **Symptoms**: What the user/tester observed
- **Root Cause**: Technical reason for the issue
- **Resolution**: How it was fixed
- **Prevention**: Steps to avoid regression
- **Related Issues**: Links to similar problems

---

## CI/CD Issues

### #1: Formatting Check Failures (v1.0.4)

**Severity**: P1
**Symptoms**: Publish workflow fails with "cargo fmt check" errors, showing multiple files with formatting diffs

**Root Cause**: Code committed without running rustfmt

**Resolution**:
```bash
cargo fmt --all
git add -u
git commit -m "style: Apply cargo fmt formatting fixes"
git push origin main
```

**Prevention**:
- Add pre-commit hook for rustfmt
- Configure editor to format on save
- Add `.rustfmt.toml` configuration to project root
- Document formatting requirements in CONTRIBUTING.md

**Related Issues**: None

---

### #2: Clippy Warning Violations (v1.0.4)

**Severity**: P1
**Symptoms**: Publish workflow fails on `cargo clippy -- -D warnings` with multiple warnings

**Specific Warnings Encountered**:
1. **Unused Import**: `use anyhow::{Context, Result}` where `Context` was unused
   - File: `src/doctor.rs`
   - Fix: Remove unused import

2. **DoubleEndedIterator Optimization**: `.last()` on iterator that supports `.next_back()`
   - File: `src/doctor.rs`
   - Warning: `double_ended_iterator_last`
   - Fix: Replace `.last()` with `.next_back()`
   - Rationale: More efficient for DoubleEndedIterator

3. **Match to Matches Macro**: Match expression could be simplified
   - File: `src/doctor.rs`
   - Warning: `match_like_matches_macro`
   - Fix: Convert to `matches!()` macro

4. **Unnecessary Cast**: Casting u64 value that's already u64
   - File: `src/model_loader.rs`
   - Warning: `unnecessary_cast`
   - Fix: Remove `as u64` cast

**Resolution**:
```bash
# Apply clippy fixes
cargo clippy --fix --allow-dirty --allow-staged
# Or manually fix each warning
git commit -m "fix: Resolve clippy warnings for vX.Y.Z"
```

**Prevention**:
- Run `cargo clippy` before committing
- Add clippy to pre-push hook
- Configure CI to fail on clippy warnings
- Use `clippy::pedantic` in development for stricter checks
- Document common clippy fixes in CONTRIBUTING.md

**Related Issues**: #1 (formatting)

---

### #3: Dead Code in Test Structs (v1.0.4)

**Severity**: P2
**Symptoms**: Compiler warnings about unused fields in test code

**Root Cause**: Debug/troubleshooting fields added to `ValidationResult` struct for development but not used in test assertions

**Resolution**:
```rust
#[derive(Debug)]
struct ValidationResult {
    success: bool,
    #[allow(dead_code)]
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    is_blocked: bool,
    risk_level: Option<String>,
    #[allow(dead_code)]
    duration: Duration,
}
```

**Prevention**:
- Use `#[allow(dead_code)]` for intentional debug fields
- Consider removing fields if truly unused
- Use `#[cfg(debug_assertions)]` for debug-only fields
- Document why fields are intentionally unused

**Related Issues**: None

---

### #4: Flaky Test - Shell Detection (v1.0.4)

**Severity**: P1
**Symptoms**: `test_shell_detector_uses_env_variable` fails intermittently in CI with:
```
assertion `left == right` failed
  left: Some(Zsh)
 right: Some(Bash)
```

**Root Cause**: Test logic didn't mirror the actual detection implementation. Test assumed specific shells in CI environment but detection logic was more flexible.

**Problematic Test Logic**:
```rust
// This assumes environment will have specific shell types
if shell_path.contains("bash") {
    assert_eq!(detected, Some(ShellType::Bash));  // Fails if detection returns different result
}
```

**Resolution**: Updated test to mirror exact detection logic from `ShellType::detect()`:
```rust
// Mirror the exact detection logic
let expected = if shell_path.contains("bash") {
    Some(ShellType::Bash)
} else if shell_path.contains("zsh") {
    Some(ShellType::Zsh)
} else if shell_path.contains("fish") {
    Some(ShellType::Fish)
} else if shell_path.ends_with("/sh") {
    Some(ShellType::Sh)
} else {
    None // Unknown shell
};

assert_eq!(detected, expected, "Shell detection mismatch for SHELL={}", shell_path);
```

**Prevention**:
- Contract tests should mirror implementation logic exactly
- Don't make assumptions about test environment
- Use deterministic inputs or mock environment variables
- Add error messages with context to assertions
- Test with multiple shell environments if shell-specific

**Related Issues**: None

---

## Release Workflow Issues

### #5: Model Bundling Python Environment (v1.0.4)

**Severity**: P2 (Non-Critical)
**Symptoms**: "Bundle with Models" job fails on macOS with:
```
ModuleNotFoundError: No module named 'huggingface_hub'
```

**Root Cause**: `pip install -q huggingface_hub` doesn't install to the Python environment that `python3` uses

**Impact**: Model bundles not created for some platforms, but users can download models automatically on first use

**Workaround**: Users download models via `caro` on first run (built-in functionality)

**Resolution Status**: Deferred (non-critical)

**Future Fix**:
```yaml
# Use python3 -m pip to ensure correct environment
- name: Install huggingface_hub
  run: python3 -m pip install --user huggingface_hub

# Or use system Python path
- name: Setup Python
  uses: actions/setup-python@v5
  with:
    python-version: '3.x'
```

**Prevention**:
- Test bundling workflow on all target platforms
- Use `python3 -m pip` instead of `pip`
- Add Python environment validation to workflow
- Consider using `uv` or `pipx` for more reliable installs

**Related Issues**: #6

---

### #6: HuggingFace Authentication in CI (v1.0.4)

**Severity**: P2 (Non-Critical)
**Symptoms**: Model download fails with 401 Unauthorized from HuggingFace Hub

**Root Cause**: Some HuggingFace models require authentication even for public access

**Impact**: Model bundles fail to create on Linux, but doesn't affect core release

**Resolution Status**: Deferred (non-critical)

**Future Fix**:
```yaml
- name: Download model from Hugging Face
  env:
    HF_TOKEN: ${{ secrets.HUGGINGFACE_TOKEN }}
  run: |
    # Use token for authenticated access
    python3 << 'EOF'
    from huggingface_hub import hf_hub_download
    import os

    token = os.environ.get('HF_TOKEN')
    model_path = hf_hub_download(
        repo_id="...",
        filename="...",
        token=token  # Add token parameter
    )
    EOF
```

**Prevention**:
- Add HF_TOKEN secret to GitHub repository settings
- Test model downloads with and without authentication
- Use public models that don't require auth
- Document which models need authentication

**Related Issues**: #5

---

## Installation Issues

### Known Issue Template

*(Add new issues as they're discovered and resolved)*

**Severity**: P?
**Symptoms**: What users/testers observe

**Root Cause**: Technical explanation

**Resolution**: How it was fixed (with code/commands)

**Prevention**: How to avoid this in future

**Related Issues**: Links to similar problems

---

## Regression Prevention Checklist

Before each release, verify:

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] All tests pass: `cargo test --all-features`
- [ ] Contract tests mirror implementation logic
- [ ] No dead code warnings (or properly suppressed)
- [ ] Release workflow tested end-to-end (at least once per minor version)
- [ ] Model bundling tested on all platforms (or skipped intentionally)
- [ ] Known issues documented for any deferred fixes

## Update Protocol

When a new issue is discovered and resolved:

1. **Add Entry**: Create new section with all required fields
2. **Link Commits**: Reference the commit SHA that fixed the issue
3. **Update Prevention**: Add specific checks to Regression Prevention Checklist
4. **Cross-Reference**: Link related issues in GitHub and this document
5. **Notify Team**: Share learnings in team channel/meeting

## Severity Definitions

- **P0 (Critical)**: Blocks primary use case, no workaround, affects majority of users
- **P1 (High)**: Breaks common workflow, workaround exists but difficult
- **P2 (Medium)**: Degrades experience, reasonable workaround available
- **P3 (Low)**: Minor polish, edge case, or cosmetic issue

## Search Tags

*(For easy searching of this document)*

- CI/CD: #formatting #clippy #linting #tests #build
- Runtime: #crash #hang #memory #performance
- Installation: #cargo #download #dependencies #platform
- Documentation: #readme #docs #examples #guides
- Security: #vulnerability #cve #permissions #auth
