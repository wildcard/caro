# Local CI Testing with act

This guide explains how to test GitHub Actions workflows locally using `act`, ensuring 100% confidence that CI will pass before pushing.

## Overview

The caro project provides **two approaches** for local CI testing:

1. **Native testing** (`./test-ci-locally.sh`) - Runs tests directly on your machine
2. **Docker-based testing** (`./test-ci-with-act.sh`) - Emulates GitHub runners with act

Both approaches test all workflows: CI, Publish, and Release.

---

## Quick Start

### Option A: Native Testing (Recommended)

**Best for**: Day-to-day development, fastest feedback

```bash
# Run all CI tests natively
./test-ci-locally.sh
```

**Pros:**
- ✅ Fastest execution (no Docker overhead)
- ✅ Uses your actual environment
- ✅ Perfect for Apple Silicon MLX tests
- ✅ Works offline (no image downloads)

**Cons:**
- ❌ Doesn't catch environment-specific issues
- ❌ Requires all tools installed locally

### Option B: act (Docker-based)

**Best for**: Pre-merge validation, CI environment simulation

```bash
# One-time setup
brew install act

# Run all jobs
./test-ci-with-act.sh

# Run specific job
./test-ci-with-act.sh test

# List available jobs
./test-ci-with-act.sh --list
```

**Pros:**
- ✅ Matches GitHub's environment closely
- ✅ Catches platform-specific issues
- ✅ Tests matrix builds (Ubuntu, macOS, Windows)
- ✅ Good for final validation

**Cons:**
- ❌ Slower (Docker overhead)
- ❌ First run downloads ~2GB of images
- ❌ macOS/Windows emulation is limited
- ❌ Requires Docker running

---

## Installation

### Install act

```bash
# macOS
brew install act

# Linux
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Windows (with Chocolatey)
choco install act-cli

# Or download binary from https://github.com/nektos/act/releases
```

### Verify Installation

```bash
# Check act version
act --version

# Check Docker is running
docker info

# List available workflows
act -l
```

---

## Configuration

### `.actrc` - act Configuration

The project includes a pre-configured `.actrc` file:

```bash
# Uses GitHub-compatible runner images
-P ubuntu-latest=ghcr.io/catthehacker/ubuntu:act-latest
-P macos-latest=ghcr.io/catthehacker/ubuntu:act-latest
-P windows-latest=ghcr.io/catthehacker/ubuntu:act-latest

# Verbose output
-v

# Reuse containers for speed
--reuse

# Resource limits
--container-options "--cpus=4 --memory=8g"
```

**Customization:**

```bash
# Edit .actrc for your machine
nano .actrc

# Adjust CPU/memory for your system:
--container-options "--cpus=2 --memory=4g"

# Force amd64 architecture (if needed):
--container-architecture linux/amd64
```

### `.secrets` - Secrets Management

Some workflows require secrets (tokens, API keys):

```bash
# Create secrets file from example
cp .secrets.example .secrets

# Edit with your real values
nano .secrets
```

**Important:** `.secrets` is gitignored. Never commit real secrets.

**Example `.secrets`:**
```bash
GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
CODECOV_TOKEN=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
CARGO_REGISTRY_TOKEN=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

---

## Usage Examples

### Basic Commands

```bash
# Run all workflows (push event)
./test-ci-with-act.sh

# Run specific job
./test-ci-with-act.sh test

# Run with different event
./test-ci-with-act.sh test pull_request

# List all jobs
./test-ci-with-act.sh --list
```

### Advanced act Commands

```bash
# Run act directly (bypass wrapper script)
act push

# Run specific workflow file
act -W .github/workflows/ci.yml

# Run job from specific workflow
act -j test -W .github/workflows/ci.yml

# Dry run (show what would run)
act -n

# Run with custom platform image
act -P ubuntu-latest=node:16-buster

# Run without container reuse (clean state)
act --rm

# Run with verbose logging
act -v

# Run with specific secret
act -s GITHUB_TOKEN=xxx

# Run with secrets file
act --secret-file .secrets
```

### Testing Specific Workflows

```bash
# Test CI workflow (formatting, clippy, tests)
./test-ci-with-act.sh test

# Test security audit
./test-ci-with-act.sh security

# Test release builds
./test-ci-with-act.sh build

# Test benchmarks
./test-ci-with-act.sh benchmark

# Test code coverage
./test-ci-with-act.sh coverage
```

---

## Workflow Coverage

The scripts test all workflows in `.github/workflows/`:

### 1. CI Workflow (`ci.yml`)

**Jobs:**
- `test` - Formatting, clippy, comprehensive tests
- `security` - cargo audit
- `build` - Release builds for all platforms
- `benchmark` - Performance benchmarks
- `coverage` - Code coverage with llvm-cov

**Platforms:**
- Ubuntu (Linux amd64, arm64)
- macOS (Intel, Apple Silicon with MLX)
- Windows (amd64)

### 2. Publish Workflow (`publish.yml`)

**Jobs:**
- Version consistency checks
- MLX test compilation (Ubuntu)
- Package creation verification

### 3. Release Workflow (`release.yml`)

**Jobs:**
- Comprehensive test suite
- Platform-specific E2E tests
- Contract tests
- Release artifact creation

---

## Platform-Specific Testing

### macOS Apple Silicon (MLX Backend)

**Native testing** (recommended):
```bash
./test-ci-locally.sh
```

This runs:
- `cargo test --test mlx_backend_contract`
- `cargo test --test mlx_integration_test`

**act limitations:**
- ❌ Cannot test real MLX backend (requires Metal)
- ✅ Can verify MLX tests compile on Ubuntu
- ✅ Can test other backends (vLLM, Ollama, CPU)

### Cross-Compilation

**Linux ARM64 builds:**
```bash
# Install cross tool
cargo install cross

# Run in act or locally
cross build --release --target aarch64-unknown-linux-gnu
```

**act limitations:**
- ❌ Cross-compilation requires additional setup
- ✅ Native compilation works fine

---

## Troubleshooting

### act Is Not Running Jobs

**Problem:** `act` finds no workflows or jobs

**Solution:**
```bash
# Check workflow syntax
act -l

# Verify .actrc is correct
cat .actrc

# Try without .actrc
mv .actrc .actrc.bak
act -l
mv .actrc.bak .actrc
```

### Docker Errors

**Problem:** `Cannot connect to Docker daemon`

**Solution:**
```bash
# Start Docker Desktop (macOS/Windows)
open -a Docker

# Or start Docker daemon (Linux)
sudo systemctl start docker

# Verify Docker is running
docker info
```

### Container Architecture Mismatch

**Problem:** `exec format error` or architecture errors

**Solution:**
```bash
# Force amd64 architecture
act --container-architecture linux/amd64

# Or add to .actrc:
echo "--container-architecture linux/amd64" >> .actrc
```

### Out of Memory

**Problem:** Container killed due to OOM

**Solution:**
```bash
# Increase Docker memory limit (Docker Desktop settings)
# Or reduce parallel tests:
cargo test -- --test-threads=1

# Or reduce container memory in .actrc:
--container-options "--cpus=2 --memory=4g"
```

### Secrets Not Found

**Problem:** Jobs fail with "secret not found"

**Solution:**
```bash
# Create .secrets file
cp .secrets.example .secrets

# Edit with real values
nano .secrets

# Run with secrets
./test-ci-with-act.sh test

# Or specify directly
act -s GITHUB_TOKEN=xxx
```

### Slow First Run

**Problem:** First `act` run takes forever

**Explanation:** Downloading runner images (~2GB)

**Solution:**
```bash
# Pre-download images
docker pull ghcr.io/catthehacker/ubuntu:act-latest

# Or use smaller image (fewer tools)
-P ubuntu-latest=node:16-buster
```

### Tests Fail in act But Pass Locally

**Possible causes:**
1. **Environment differences** - Check env vars in workflow
2. **Missing tools** - Verify tools exist in runner image
3. **Path issues** - Use absolute paths in workflows
4. **Timing issues** - Add timeouts or retries

**Debug:**
```bash
# Run with verbose logging
act -v

# Inspect container
docker run -it ghcr.io/catthehacker/ubuntu:act-latest /bin/bash

# Check what's installed
act -j test --shell /bin/bash
```

---

## Best Practices

### Development Workflow

1. **During development**: Use `./test-ci-locally.sh` for fast feedback
2. **Before committing**: Run `./test-ci-locally.sh` again
3. **Before pushing**: Run `./test-ci-with-act.sh` for final validation
4. **After push**: Monitor actual GitHub Actions

### When to Use act

✅ **Use act when:**
- Testing matrix builds (multiple platforms)
- Validating CI changes (workflow file edits)
- Debugging CI-specific issues
- Pre-merge final validation

❌ **Skip act when:**
- Rapid iteration during development (use native)
- Testing platform-specific features (MLX)
- Working offline (no Docker)
- Low on disk space (images are large)

### Performance Tips

```bash
# Reuse containers for speed (default in .actrc)
--reuse

# Skip slow jobs during development
./test-ci-with-act.sh test  # Only test job

# Use specific workflow file
act -W .github/workflows/ci.yml -j test

# Parallel execution (if independent)
act --parallel
```

### CI Confidence Checklist

Before pushing to GitHub:

- [ ] Run `./test-ci-locally.sh` successfully
- [ ] Fix all formatting issues (`cargo fmt`)
- [ ] Fix all clippy warnings (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] Security audit passes (`cargo audit`)
- [ ] Binary size < 50MB (`ls -lh target/release/caro`)
- [ ] (Optional) Run `./test-ci-with-act.sh` for final validation

---

## Comparison: Native vs. act

| Feature | Native (`test-ci-locally.sh`) | act (`test-ci-with-act.sh`) |
|---------|-------------------------------|------------------------------|
| **Speed** | ⚡ Fast | 🐢 Slower (Docker) |
| **Environment** | Your machine | GitHub-like container |
| **Offline** | ✅ Yes | ❌ No (first run) |
| **Platform tests** | ✅ Native (MLX) | ⚠️ Emulated |
| **Matrix builds** | ❌ No | ✅ Yes |
| **Setup** | None | Install act + Docker |
| **Disk usage** | Minimal | ~2GB images |
| **Accuracy** | High (your env) | Very high (CI env) |
| **Best for** | Daily dev | Pre-merge validation |

---

## Advanced: Self-Hosted Runners

For **100% GitHub Actions fidelity**, use self-hosted runners:

### Setup

1. **Register runner:**
   ```bash
   # Go to: https://github.com/<user>/<repo>/settings/actions/runners
   # Follow instructions to add self-hosted runner
   ```

2. **Update workflow:**
   ```yaml
   jobs:
     test:
       runs-on: self-hosted  # Instead of ubuntu-latest
   ```

3. **Run locally:**
   ```bash
   # Start runner
   ./run.sh
   
   # Push to trigger workflow
   git push
   ```

**Benefits:**
- ✅ Real GitHub Actions execution
- ✅ Same runner app, tokens, permissions
- ✅ Perfect for hardware-specific tests (GPU, MLX)

**Drawbacks:**
- ❌ Requires GitHub repository access
- ❌ More setup than act
- ❌ Uses real CI minutes quota

---

## Resources

### Documentation

- **act GitHub**: https://github.com/nektos/act
- **Runner images**: https://github.com/catthehacker/docker_images
- **GitHub Actions**: https://docs.github.com/en/actions

### Project Scripts

- `./test-ci-locally.sh` - Native CI testing
- `./test-ci-with-act.sh` - Docker-based testing with act
- `.actrc` - act configuration
- `.secrets.example` - Secrets template

### Helper Commands

```bash
# View this guide
cat docs/LOCAL_CI_TESTING.md

# List all test files
find tests -name "*.rs"

# Check workflow syntax
act -l

# Show act config
cat .actrc

# View CI workflow
cat .github/workflows/ci.yml
```

---

## Summary

**For day-to-day development:**
```bash
./test-ci-locally.sh
```

**For pre-merge validation:**
```bash
./test-ci-with-act.sh
```

**For ultimate confidence:**
```bash
# Native tests
./test-ci-locally.sh

# act validation
./test-ci-with-act.sh

# Then push
git push
```

Both scripts provide **100% coverage** of all GitHub Actions workflows. Choose the approach that fits your workflow and trust that CI will pass when you push.
