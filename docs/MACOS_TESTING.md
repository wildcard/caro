# macOS Apple Silicon Testing Guide

## Overview

This document explains how to use the macOS Apple Silicon GitHub Actions workflow to validate that cmdai builds and runs correctly on Apple Silicon hardware (M1/M2/M3/M4).

## Why This Exists

**Problem**: Contributors report that the embedded backends don't work on their local macOS machines.

**Solution**: This automated workflow runs on **actual Apple Silicon hardware in GitHub's cloud** and provides:

1. ✅ **Source of truth**: Proves whether the project builds correctly on macOS
2. ✅ **Isolation**: Separates project issues from local environment issues
3. ✅ **Reproducibility**: Every run uses a clean macOS environment
4. ✅ **Transparency**: Public logs show exactly what works and what doesn't

## Workflow Status

### Current Build Status

![macOS Apple Silicon](https://github.com/wildcard/cmdai/actions/workflows/macos-apple-silicon.yml/badge.svg)

**How to interpret:**
- 🟢 **Green (Passing)**: Project builds and tests pass on Apple Silicon
  - If green and your local build fails → Issue is with your local environment
- 🔴 **Red (Failing)**: Project has issues on Apple Silicon
  - If red → Issue is in the project code, not your machine
- 🟡 **Yellow (Running)**: Tests are currently executing
- ⚪ **Gray (No status)**: Workflow hasn't run yet

### Where to Check Status

**Option 1: GitHub Actions Tab**
1. Go to https://github.com/wildcard/cmdai/actions
2. Click on "macOS Apple Silicon Testing"
3. View the latest run results

**Option 2: README Badge**
Look for the build status badge at the top of README.md

**Option 3: PR Checks**
Every pull request will show the workflow status in the checks section

## What Gets Tested

### Test Matrix

The workflow tests **three configurations**:

| Configuration | Features | Purpose |
|--------------|----------|---------|
| **CPU Backend** | `embedded-cpu` | Cross-platform baseline using Candle |
| **MLX Backend** | `embedded-mlx` | Apple Silicon GPU acceleration (stub) |
| **All Backends** | `embedded-mlx,embedded-cpu` | Full embedded feature set |

### Test Stages

Each configuration goes through:

1. **Code Quality**
   - ✅ Formatting check (`cargo fmt`)
   - ✅ Linting (`cargo clippy`)

2. **Compilation**
   - ✅ Library compilation
   - ✅ Test compilation
   - ✅ Release binary build for `aarch64-apple-darwin`

3. **Unit Tests**
   - ✅ Library unit tests
   - ✅ Backend-specific tests
   - ✅ MLX contract tests (including `#[ignore]` tests)

4. **Integration Tests**
   - ✅ End-to-end workflows
   - ✅ Embedded backend integration

5. **E2E Tests**
   - ✅ Smoke tests
   - ✅ Critical functionality tests

6. **Binary Validation**
   - ✅ Size check (<50MB requirement)
   - ✅ Execution sanity checks (`--help`, `--version`)

### Additional Validation

- **Metal GPU Support**: Validates Metal framework availability
- **Benchmarks**: Runs performance benchmarks on Apple Silicon
- **Cross-compilation**: Tests building for Intel Macs from Apple Silicon

## How to Use This Workflow

### Automatic Triggers

The workflow runs automatically on:

- ✅ **Pushes** to `main`, `develop`, or any `claude/**` branch
- ✅ **Pull requests** targeting `main` or `develop`

### Manual Trigger

You can manually trigger the workflow:

1. Go to https://github.com/wildcard/cmdai/actions/workflows/macos-apple-silicon.yml
2. Click "Run workflow" button
3. Select branch
4. Click "Run workflow"

**When to manually trigger:**
- Testing a specific branch
- Validating a fix without creating a PR
- Re-running after a transient failure

## Interpreting Results

### Scenario 1: Green Build, Local Failure

**Symptoms:**
- ✅ GitHub Actions shows passing tests
- ❌ Your local `cargo build` or `cargo test` fails

**Diagnosis**: Issue is with your local environment, not the project.

**Solutions:**

1. **Check Rust version**
   ```bash
   rustup update
   rustc --version  # Should be 1.82.0+
   ```

2. **Clean and rebuild**
   ```bash
   cargo clean
   cargo build --features embedded-cpu
   ```

3. **Check dependencies**
   ```bash
   # Ensure Xcode Command Line Tools installed
   xcode-select --install

   # Check for conflicting cargo installations
   which cargo
   ```

4. **Match CI environment**
   ```bash
   # Use same target as CI
   cargo build --target aarch64-apple-darwin --features embedded-cpu
   ```

5. **Check feature flags**
   ```bash
   # Are you using the right features?
   cargo build --features embedded-mlx  # For MLX backend
   cargo build --features embedded-cpu  # For CPU backend
   ```

### Scenario 2: Red Build

**Symptoms:**
- ❌ GitHub Actions shows failing tests
- ❌ Your local build also fails (or you haven't tried locally)

**Diagnosis**: Issue is in the project code.

**Solutions:**

1. **Review the failure logs**
   - Click on the failed job in GitHub Actions
   - Expand the failing step
   - Read the error message

2. **Common failure types**:
   - **Compilation errors**: Syntax or type errors in code
   - **Test failures**: Logic bugs or incorrect assertions
   - **Clippy warnings**: Code quality issues (`-D warnings` flag)
   - **Binary size**: Binary exceeds 50MB limit

3. **Report the issue**
   - If you didn't introduce the failure, open a GitHub issue
   - Include link to the failed workflow run
   - Include error messages from logs

### Scenario 3: Partial Failure (continue-on-error)

**Symptoms:**
- 🟡 Some steps show warnings but overall workflow passes
- Logs show "⚠️ Some tests may fail during TDD phase"

**Diagnosis**: Expected during development.

**Explanation**:
- Some tests are marked `#[ignore]` pending full implementation
- These tests run with `continue-on-error: true`
- The workflow still passes overall if non-ignored tests pass

**What to check**:
- Look for which specific tests are ignored
- Check if the test failure is expected or a regression

## Viewing Detailed Logs

### Accessing Logs

1. Go to https://github.com/wildcard/cmdai/actions/workflows/macos-apple-silicon.yml
2. Click on a workflow run
3. Click on a job (e.g., "Test on Apple Silicon - CPU Backend")
4. Expand individual steps to see output

### Key Information in Logs

**System Information**:
```
=== System Information ===
Model Name: Mac14,6  (M1 processor)
Chip: Apple M1
Total Number of Cores: 8
Memory: 7 GB
```

**Binary Size**:
```
Binary size: 12582912 bytes (12 MB)
✅ PASS: Binary size (12 MB) is within 50MB limit
```

**Test Results**:
```
test backends::embedded::cpu::tests::test_cpu_backend_new ... ok
test backends::embedded::mlx::tests::test_mlx_backend_new ... ok

test result: ok. 45 passed; 0 failed; 6 ignored; 0 measured
```

## Troubleshooting Common Issues

### Issue: "Binary size exceeds 50MB"

**Cause**: Release binary is too large

**Check**:
```bash
# Locally
cargo build --release --target aarch64-apple-darwin
ls -lh target/aarch64-apple-darwin/release/cmdai
```

**Solutions**:
- Review `Cargo.toml` profile settings
- Remove unnecessary dependencies
- Check for debug symbols: `strip target/release/cmdai`

### Issue: "Metal framework not found"

**Cause**: Should not happen on GitHub Actions (has Metal support)

**If occurs locally**:
- Ensure macOS 12.0+
- Install Xcode Command Line Tools
- Check: `ls /System/Library/Frameworks/Metal.framework`

### Issue: "Tests timeout"

**Cause**: Tests taking too long (default 360 minutes max)

**Solutions**:
- Check for infinite loops in test code
- Review async test timeout settings
- Add explicit timeouts to long-running tests

### Issue: "Workflow not triggering"

**Causes**:
1. Branch name doesn't match trigger pattern
2. Workflow file has YAML syntax errors
3. Repository settings disable Actions

**Solutions**:
```bash
# Check YAML syntax
yamllint .github/workflows/macos-apple-silicon.yml

# Manually trigger via UI
# Go to Actions → macOS Apple Silicon Testing → Run workflow
```

## Comparing Local vs CI Environment

| Aspect | GitHub Actions (CI) | Local Mac |
|--------|-------------------|-----------|
| **Hardware** | M1 (macOS-14 runner) | M1/M2/M3/M4 (varies) |
| **OS** | macOS 14 (Sonoma) | User's macOS version |
| **Rust** | Latest stable | User's version |
| **Environment** | Clean on each run | May have cached artifacts |
| **Dependencies** | Freshly installed | May be stale |
| **Network** | GitHub's network | User's network |

**If CI passes but local fails**: Environment difference is the cause.

## Advanced: Reproducing CI Locally

### Match CI Rust Version

```bash
# Update to latest stable
rustup update stable
rustup default stable
```

### Match CI Build Command

```bash
# CPU backend test (as CI does)
cargo test --features embedded-cpu --verbose -- --nocapture

# MLX backend test (as CI does)
cargo test --features embedded-mlx --test mlx_backend_contract --verbose -- --nocapture

# Build release binary (as CI does)
cargo build --release --target aarch64-apple-darwin --features embedded-mlx,embedded-cpu
```

### Clean Build (Like CI)

```bash
# Remove all build artifacts
cargo clean
rm -rf target/

# Remove cargo cache (nuclear option)
rm -rf ~/.cargo/registry
rm -rf ~/.cargo/git

# Rebuild from scratch
cargo build --release --features embedded-cpu
```

### Run Ignored Tests (Like CI)

```bash
# Run ALL tests including #[ignore]
cargo test --features embedded-mlx --test mlx_backend_contract -- --include-ignored --nocapture
```

## Performance Benchmarks

The workflow runs performance benchmarks and uploads results as artifacts.

### Accessing Benchmark Results

1. Go to workflow run
2. Scroll to "Artifacts" section at bottom
3. Download `benchmark-results-apple-silicon`
4. Open `target/criterion/report/index.html` in browser

### What's Benchmarked

- Inference performance (CPU vs MLX)
- Model loading time
- Command generation latency
- Safety validation overhead

## Binary Artifacts

Each successful run uploads binary artifacts:

### Available Artifacts

- `cmdai-macos-silicon-CPU Backend`
- `cmdai-macos-silicon-MLX Backend (Stub)`
- `cmdai-macos-silicon-All Embedded Backends`

### Downloading Artifacts

1. Go to workflow run
2. Scroll to "Artifacts" section
3. Click to download
4. Unzip and test:
   ```bash
   chmod +x cmdai
   ./cmdai --help
   ```

**Use cases**:
- Testing PR builds before merging
- Comparing performance between branches
- Distributing pre-release binaries to testers

## Workflow Configuration

### Customizing the Workflow

The workflow file is at: `.github/workflows/macos-apple-silicon.yml`

**Key configuration points**:

```yaml
# Trigger branches
on:
  push:
    branches: [ main, develop, 'claude/**' ]  # Add more here

# Runner (macOS version)
runs-on: macos-14  # M1 hardware, update for newer macOS

# Feature matrix
matrix:
  include:
    - name: "CPU Backend"
      features: "embedded-cpu"
      # Add more configurations here
```

### Adding New Tests

To add tests to the workflow:

1. Write test in `tests/` directory
2. Test passes → automatically runs in CI
3. Test needs `#[ignore]` → runs with `--include-ignored` flag

No workflow changes needed!

## FAQ

### Q: Why use macOS-14 runners?

**A**: macOS-14 runs on Apple Silicon (M1). Earlier versions (macOS-12, macOS-13) run on Intel, which can't test MLX features.

### Q: Can I test on M2/M3/M4 specifically?

**A**: GitHub Actions currently provides M1 runners. For M2/M3/M4 testing, you need self-hosted runners (see GitHub docs).

### Q: How long do tests take?

**A**: Typically 5-10 minutes for full test suite, depending on runner availability and cache hits.

### Q: What if workflow fails due to GitHub outage?

**A**: Re-run the workflow manually. GitHub shows "runner failure" vs "test failure" in logs.

### Q: Can I run this workflow on forks?

**A**: Yes! Fork the repo and the workflow runs automatically on your fork's Actions tab.

### Q: Why do some tests show "continue-on-error"?

**A**: During TDD development, some tests are expected to fail. These steps allow workflow to continue and show overall project health.

## Getting Help

### If CI Shows Green But You Have Local Issues

1. **Post in Discussions**: https://github.com/wildcard/cmdai/discussions
   - Title: "Local build fails but CI passes"
   - Include: macOS version, Rust version, error logs

2. **Check Similar Issues**: Search closed issues for similar problems

3. **Compare with CI**:
   - Look at "Display system information" step in CI logs
   - Compare with your `system_profiler SPHardwareDataType`
   - Look for differences (macOS version, etc.)

### If CI Shows Red

1. **Open an Issue**: https://github.com/wildcard/cmdai/issues/new
   - Title: "CI failure: [brief description]"
   - Include: Link to failed workflow run
   - Include: Error message from logs

2. **Check Recent Changes**:
   - Look at recent commits that might have broken CI
   - Check if failure is specific to one configuration

## Summary

This workflow provides **automated proof** that cmdai builds and runs on Apple Silicon hardware:

✅ **For Users**: Shows if project is healthy before trying locally
✅ **For Contributors**: Validates PRs before merging
✅ **For Maintainers**: Source of truth for "it works on my machine" issues

**The green checkmark means it works. If your local build fails, the issue is local.**

---

**Workflow File**: `.github/workflows/macos-apple-silicon.yml`
**Status**: https://github.com/wildcard/cmdai/actions/workflows/macos-apple-silicon.yml
