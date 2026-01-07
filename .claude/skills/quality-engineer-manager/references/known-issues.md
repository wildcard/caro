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

### #7: YAML Heredoc Syntax Incompatibility in GitHub Actions (v1.0.4 Bundle)

**Severity**: P1
**Symptoms**: Workflow fails with YAML parsing error at heredoc line, such as:
```
Line 140: Mapping values are not allowed in this context
error: Invalid workflow file
```

**Root Cause**: GitHub Actions `run: |` blocks use YAML multi-line string syntax which conflicts with shell heredoc (`<<EOF`) syntax. The YAML parser attempts to interpret heredoc markers and content as YAML mapping keys, causing parse failures.

**Problematic Pattern**:
```yaml
- name: Create file with heredoc
  run: |
    cat <<EOF > file.txt
    Line 1: Some content
    Line 2: More content
    EOF
```

The YAML parser sees "Line 1: Some content" and interprets "Line 1" as a YAML key, failing because it's inside a scalar string block.

**Resolution**: Replace heredocs with `printf` command:
```yaml
- name: Create file with printf
  run: |
    printf '%s\n' \
      "Line 1: Some content" \
      "Line 2: More content" \
      > file.txt
```

**Commit**: `689e037` - "fix(ci): Replace heredoc with printf for THIRD_PARTY_NOTICES"

**Location**: `.github/workflows/bundle.yml` lines 149-167

**Prevention**:
- **Never use heredocs** (`<<EOF`, `<<'EOF'`, `<<-EOF`) in GitHub Actions `run: |` blocks
- Use `printf '%s\n'` with escaped strings for multi-line file creation
- Use simple `echo` for single lines
- Consider external template files for complex multi-line content
- Test workflow YAML locally with `act` or YAML validators before pushing
- Add comment in workflow explaining why printf is used instead of heredoc

**Related Issues**: #8, #9 (bundle workflow issues discovered in same session)

---

### #8: GitHub CLI Missing in Alpine Containers (v1.0.4 Bundle)

**Severity**: P1
**Symptoms**: Workflow fails with exit code 127 and error:
```
gh: command not found
/bin/sh: gh: not found
```

**Root Cause**: Alpine Linux minimal images do not include GitHub CLI (`gh`), and it's not available in Alpine's `apk` package manager. This affects workflows using `container: alpine:latest` that need to interact with GitHub releases or repositories.

**Impact**: Any workflow job that runs in Alpine container and uses `gh release upload`, `gh pr create`, etc. will fail unless `gh` is explicitly installed.

**Resolution**: Manually install `gh` CLI from GitHub releases:
```yaml
- name: Install dependencies
  run: |
    apk add --no-cache curl tar gzip bash

    # Install gh CLI for Alpine Linux (amd64)
    curl -fsSL https://github.com/cli/cli/releases/download/v2.63.2/gh_2.63.2_linux_amd64.tar.gz -o gh.tar.gz
    tar -xzf gh.tar.gz
    mv gh_2.63.2_linux_amd64/bin/gh /usr/local/bin/
    rm -rf gh.tar.gz gh_2.63.2_linux_amd64
    gh --version
```

**Commit**: `0ed6e99` - "fix(ci): Install gh CLI in Alpine container for bundle uploads"

**Location**: `.github/workflows/bundle.yml` lines 75-79

**Prevention**:
- Always verify tool availability in container images before use
- Document required tools at the top of workflow files with installation steps
- Consider using `ubuntu-latest` for workflows needing many standard tools
- Pin `gh` CLI version to avoid unexpected breaking changes
- Test container-based workflows locally with `docker run alpine:latest`
- **ARM64 Note**: ARM64 Alpine runners would need `linux_arm64` binary URL

**Alternative Solutions**:
- Use `ubuntu-latest` instead of Alpine (gh CLI pre-installed)
- Use official gh CLI Docker image: `ghcr.io/cli/cli:latest`
- Create custom Alpine image with gh pre-installed

**Related Issues**: #7, #9 (bundle workflow issues discovered in same session)

---

### #9: GitHub Release Upload Permission Denied (v1.0.4 Bundle)

**Severity**: P1
**Symptoms**: `gh release upload` fails with HTTP 403 or permission denied error:
```
HTTP 403: Resource not accessible by integration (https://api.github.com/repos/...)
gh: Resource not accessible by integration
```

**Root Cause**: GitHub Actions workflows default to read-only permissions for `GITHUB_TOKEN`. Uploading files to releases requires explicit `contents: write` permission. This is a GitHub security feature that prevents accidental modifications.

**Problematic Pattern**:
```yaml
name: Bundle Models

on:
  workflow_dispatch:
    # ...

# Missing permissions block!

jobs:
  bundle:
    runs-on: ubuntu-latest
    steps:
      - run: gh release upload ...  # Fails with 403
```

**Resolution**: Add `contents: write` permission at workflow level:
```yaml
name: Bundle Models

on:
  workflow_dispatch:
    # ...

permissions:
  contents: write  # Required for gh release upload

jobs:
  bundle:
    runs-on: ubuntu-latest
    steps:
      - run: gh release upload ...  # Now works
```

**Commit**: `dcb11bc` - "fix(ci): Add contents: write permission for release uploads in bundle workflow"

**Location**: `.github/workflows/bundle.yml` lines 17-18

**Prevention**:
- Always audit required permissions before adding new workflow functionality
- Add permissions block explicitly (don't rely on repository defaults)
- Use least-privilege principle - only add permissions that are needed
- **Common permission requirements**:
  - `contents: write` - modify files, releases, tags
  - `contents: read` - read repository files (default for most workflows)
  - `pull-requests: write` - comment on or modify PRs
  - `issues: write` - comment on or modify issues
  - `actions: read` - read workflow artifacts
  - `packages: write` - publish packages to GitHub Packages
- Test workflows with `GITHUB_TOKEN` debugging enabled
- Document why specific permissions are required in workflow comments

**GitHub Permissions Reference**:
- Documentation: https://docs.github.com/en/actions/security-guides/automatic-token-authentication#permissions-for-the-github_token
- Permission scopes: https://docs.github.com/en/rest/overview/permissions-required-for-github-apps

**Related Issues**: #7, #8 (bundle workflow issues discovered in same session)

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

---

## Bundle-Specific Issues

### #10: Fish Shell Completions Not Included in Bundles (v1.0.4)

**Severity**: P2 (Medium)
**Symptoms**: Fish shell users don't get tab completions after bundle installation

**Root Cause**: Bundle workflow doesn't generate fish-specific completion files, only includes the binary and models

**Impact**: Fish shell users (estimated 5-10% of CLI users) must manually generate completions

**Workaround**:
```fish
# Generate completions manually
caro completion fish > ~/.config/fish/completions/caro.fish

# Reload fish
source ~/.config/fish/config.fish
```

**Resolution**: Add fish completion generation to bundle workflow

**Future Fix**:
```yaml
# In bundle.yml workflow
- name: Generate shell completions
  run: |
    ./caro completion fish > completions/caro.fish
    ./caro completion bash > completions/caro.bash
    ./caro completion zsh > completions/_caro
    # Include in bundle tar
```

**Prevention**:
- Test with non-bash shells during bundle validation
- Add fish shell user to standard beta testing profiles
- Document completion generation in bundle README

**Related Issues**: None

**Discovered During**: v1.0.4 bundle validation with bt_008 (Fish Shell User)

---

### #11: Bundle README Missing (v1.0.4)

**Severity**: P3 (Low)
**Symptoms**: Users unsure about bundle structure and offline usage after extraction

**Root Cause**: Bundle workflow doesn't create README explaining contents and usage

**Impact**: Minimal - documentation available online, but first-time offline users may be confused

**Resolution**: Add README.txt to bundle

**Future Fix**:
Create `README.txt` in bundle with:
- Directory structure explanation
- How to use bundled model offline
- License information location
- Version information
- Support links

**Prevention**:
- Add README generation to bundle workflow
- Include README template in bundle.yml

**Related Issues**: None

**Discovered During**: v1.0.4 bundle validation with bt_006 (Data Scientist)

---

### #12: License Files Lack Heading Structure (v1.0.4)

**Severity**: P3 (Low)
**Symptoms**: Screen reader users find THIRD_PARTY_NOTICES.txt harder to navigate

**Root Cause**: THIRD_PARTY_NOTICES.txt is plain text without markdown headings or structure markers

**Impact**: Minimal - content still readable, just less navigable for screen readers

**Workaround**: Screen reader users can still read linearly, just can't jump between sections

**Resolution**: Convert THIRD_PARTY_NOTICES.txt to use markdown headings

**Future Fix**:
```markdown
# Third-Party Notices

## Qwen2.5-0.5B-Instruct

### Copyright

Apache License 2.0
Copyright (c) Alibaba Cloud

### Source

https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct
...
```

**Prevention**:
- Use markdown format for license files
- Test with screen reader during accessibility validation
- Include accessibility user in standard beta testing

**Related Issues**: None

**Discovered During**: v1.0.4 bundle validation with bt_009 (Accessibility User)

---

## Search Tags

*(For easy searching of this document)*

- CI/CD: #formatting #clippy #linting #tests #build #yaml #heredoc #alpine #github-cli #permissions
- Runtime: #crash #hang #memory #performance
- Installation: #cargo #download #dependencies #platform
- Documentation: #readme #docs #examples #guides
- Security: #vulnerability #cve #permissions #auth #github-token
- Bundles: #fish-completions #readme #accessibility #screen-reader #offline
