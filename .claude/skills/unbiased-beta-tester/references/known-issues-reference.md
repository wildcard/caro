# Known Issues Reference for Beta Testers

This document provides beta testers with awareness of known issues to help distinguish between:
- **New bugs** (need to be reported)
- **Known issues** (already documented, may have workarounds)
- **Expected behavior** (working as designed)

## How to Use This Reference

**Before Filing a Bug Report:**
1. Check this document for similar symptoms
2. If issue is listed as "Known", note the workaround
3. If issue is listed as "Resolved", verify you're testing the correct version
4. If issue is new, file detailed bug report

## Known Issues by Category

### Installation Issues

#### Model Download on First Run (Known)
**Symptoms**: First `caro` command triggers model download, can take 1-5 minutes depending on connection

**Expected Behavior**: This is normal! Models download automatically on first use.

**Workaround**: None needed - this is expected behavior

**Status**: Working as designed

---

#### Cargo Install Fails on Non-Apple Platforms (Resolved in v1.0.4)
**Symptoms**: `cargo install caro` fails with MLX-related errors on Linux/Windows

**Root Cause**: MLX dependencies not properly feature-gated

**Status**: ✅ RESOLVED in v1.0.4

**Resolution**: Cargo features now properly conditional on platform

**If You See This**: You may be testing an older version. Verify version with `cargo search caro`.

---

### Runtime Issues

#### Model Bundling Not Available (v1.0.4)
**Symptoms**: No pre-bundled model downloads available for some platforms

**Known Limitation**: Model bundling partially fails in CI for macOS/Linux

**Workaround**: Models download automatically on first use

**Impact**: Adds 1-5 minutes to first-run experience

**Status**: Non-critical, scheduled for v1.0.5

**If You Encounter**: This is expected. First run will download model automatically.

---

### CI/CD Issues (For Contributors)

These issues affect developers/contributors, not end users:

#### Clippy Warnings (Resolved in v1.0.4)
**Symptoms**: CI fails on `cargo clippy` step

**Status**: ✅ RESOLVED

**Prevention**: Run `cargo clippy` before committing

---

#### Formatting Check Failures (Resolved in v1.0.4)
**Symptoms**: CI fails on `cargo fmt --check`

**Status**: ✅ RESOLVED

**Prevention**: Run `cargo fmt --all` before committing

---

#### Shell Detection Test Flakiness (Resolved in v1.0.4)
**Symptoms**: `test_shell_detector_uses_env_variable` fails intermittently

**Status**: ✅ RESOLVED

**Fix**: Test now mirrors implementation logic exactly

---

## Expected Behaviors (Not Bugs)

### First-Run Model Download
- **Observation**: First command takes several minutes
- **Why**: Downloading ~1.1GB model from HuggingFace
- **Workaround**: Be patient, or pre-download with specific model selection
- **Future**: Pre-bundled model downloads may be available

### Safety Validation Warnings
- **Observation**: Some commands show safety warnings/confirmations
- **Why**: Safety validation system preventing destructive commands
- **Workaround**: Review command carefully, confirm if safe
- **Not a Bug**: This is the security feature working as designed

### Model Selection
- **Observation**: Different models may give different command quality
- **Why**: Smaller models (135MB) are faster but less accurate than larger models (1.5B)
- **Workaround**: Use `CARO_MODEL` environment variable to select specific model
- **Not a Bug**: Trade-off between speed and quality

## Reporting New Issues

If you encounter an issue NOT listed above, please file a bug report with:

1. **caro Version**: `caro --version`
2. **Rust Version** (if relevant): `rustc --version`
3. **Operating System**: OS name and version
4. **Shell Type**: `echo $SHELL`
5. **Exact Command**: The command that triggered the issue
6. **Expected Behavior**: What you expected to happen
7. **Actual Behavior**: What actually happened
8. **Steps to Reproduce**: Detailed steps for someone else to reproduce
9. **Workaround**: If you found a way around it, document it!

**Use GitHub Issue Template**: It includes all required fields automatically.

## Workarounds Database

### Issue: Model Download Slow
**Workaround**: Use smaller model for faster download:
```bash
export CARO_MODEL=smollm-135m-q4
caro "list files"  # Downloads smaller 135MB model
```

### Issue: Need Offline Installation
**Workaround**: Pre-download model on machine with internet, copy cache:
```bash
# On connected machine
caro "test"  # Triggers model download
tar -czf caro-cache.tar.gz ~/.cache/caro/

# Transfer caro-cache.tar.gz to offline machine
# On offline machine
tar -xzf caro-cache.tar.gz -C ~/
```

### Issue: Corporate Proxy Blocks HuggingFace
**Workaround**: Configure proxy environment variables:
```bash
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080
caro "list files"
```

If that doesn't work, download model manually and place in cache:
```bash
mkdir -p ~/.cache/caro/models/
# Download model from HuggingFace manually
# Place qwen2.5-coder-1.5b-instruct-q4_k_m.gguf in that directory
```

## Version Compatibility

| Issue | Fixed In | Test With |
|-------|----------|-----------|
| Cargo install MLX errors | v1.0.4 | `cargo install caro --version 1.0.4` |
| Clippy warnings | v1.0.4 | CI only (not user-facing) |
| Shell test flakiness | v1.0.4 | CI only (not user-facing) |

## Getting Help

If you're unsure whether an issue is known or new:
1. Check this document first
2. Search GitHub issues: https://github.com/wildcard/caro/issues
3. Ask in discussions before filing bug report
4. When in doubt, file the bug - better to have duplicates than miss issues!

## For Quality Engineers

This is a simplified version for beta testers. For the full technical database including root causes, resolutions, and prevention strategies, see:

**Quality Engineer Manager Skill**: `.claude/skills/quality-engineer-manager/references/known-issues.md`

That document includes:
- Detailed root cause analysis
- Code-level resolutions
- CI/CD prevention strategies
- Regression testing checklists
