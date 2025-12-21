# act Quick Reference

One-page guide for running GitHub Actions locally with `act`.

## Installation

```bash
brew install act  # macOS
```

## Quick Commands

```bash
# Run all workflows
./test-ci-with-act.sh

# Run specific job
./test-ci-with-act.sh test

# List available jobs
./test-ci-with-act.sh --list

# Native testing (faster)
./test-ci-locally.sh
```

## First Time Setup

```bash
# 1. Install act
brew install act

# 2. Verify Docker is running
docker info

# 3. (Optional) Create secrets file
cp .secrets.example .secrets
# Edit .secrets with real values

# 4. Run tests
./test-ci-with-act.sh
```

## What Gets Tested

All GitHub Actions workflows:

- ✅ **CI** - Formatting, clippy, tests, security, builds, benchmarks
- ✅ **Publish** - Package verification, MLX compilation
- ✅ **Release** - E2E tests, contract tests, platform builds

## Common Tasks

### Test Before Pushing

```bash
# Quick validation (2-5 min)
./test-ci-locally.sh

# Full CI simulation (10-15 min)
./test-ci-with-act.sh
```

### Debug Failing Job

```bash
# Run specific job with verbose output
act push -j test -v

# Inspect runner container
docker run -it ghcr.io/catthehacker/ubuntu:act-latest bash
```

### Test Workflow Changes

```bash
# After editing .github/workflows/ci.yml
./test-ci-with-act.sh test

# Test specific workflow file
act -W .github/workflows/ci.yml
```

## Configuration Files

- **`.actrc`** - act settings (runner images, resources)
- **`.secrets`** - Secrets for workflows (gitignored)
- **`.secrets.example`** - Template for secrets

## Troubleshooting

### act not found
```bash
brew install act
```

### Docker not running
```bash
open -a Docker  # Start Docker Desktop
```

### Out of memory
```bash
# Edit .actrc, reduce memory:
--container-options "--cpus=2 --memory=4g"
```

### Slow first run
```bash
# Normal - downloading ~2GB of runner images
# Subsequent runs are much faster
```

## Resources

- **Full guide**: `docs/LOCAL_CI_TESTING.md`
- **act docs**: https://github.com/nektos/act
- **Runner images**: https://github.com/catthehacker/docker_images

## Decision Matrix

| Scenario | Use |
|----------|-----|
| Daily development | `./test-ci-locally.sh` |
| Pre-commit check | `./test-ci-locally.sh` |
| Pre-push validation | `./test-ci-with-act.sh` |
| Workflow debugging | `act -j <job> -v` |
| Platform-specific (MLX) | `./test-ci-locally.sh` |
| Matrix builds | `./test-ci-with-act.sh` |

## Apple Silicon Notes

```bash
# For M1/M2/M3 Macs, use native testing for MLX:
./test-ci-locally.sh

# act can't test MLX (requires Metal)
# But can verify tests compile:
act -j test -v
```

---

**Quick start:** Just run `./test-ci-with-act.sh` to test everything!
