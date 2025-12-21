# Local CI Testing Setup - Complete ✓

## What Was Installed

### Core Files

1. **`.actrc`** - act configuration with optimal runner images
   - Uses GitHub-compatible Docker images
   - Configures resource limits (4 CPU, 8GB RAM)
   - Enables container reuse for speed
   - Verbose output for debugging

2. **`.secrets.example`** - Template for workflow secrets
   - GitHub token placeholder
   - Codecov token placeholder
   - Cargo registry token placeholder
   - Instructions for generating tokens

3. **`.gitignore`** - Updated to ignore `.secrets` file
   - Prevents accidental secret commits
   - Added `# act secrets (local GitHub Actions runner)` section

### Helper Scripts

1. **`test-ci-with-act.sh`** - Docker-based CI testing
   - Runs GitHub Actions workflows locally with act
   - Supports job selection, event simulation
   - Help text and usage examples
   - Colored output for easy reading
   - Error handling and troubleshooting tips

2. **`test-ci-locally.sh`** - Native CI testing (already existed)
   - Runs tests directly on your machine
   - Faster than Docker approach
   - Perfect for Apple Silicon MLX tests

### Documentation

1. **`docs/LOCAL_CI_TESTING.md`** - Comprehensive guide (5,000+ words)
   - Overview of both testing approaches
   - Installation instructions
   - Configuration details
   - Usage examples
   - Troubleshooting section
   - Platform-specific notes
   - Best practices

2. **`docs/ACT_QUICKREF.md`** - Quick reference (1-page)
   - Essential commands
   - Common tasks
   - Decision matrix
   - Troubleshooting quick fixes

3. **`docs/ACT_SETUP.md`** - Installation guide
   - Step-by-step setup (5 minutes)
   - Configuration details
   - Workflow integration
   - Advanced usage
   - Development workflow examples

## What to Do Next

### 1. Install act (Required)

```bash
brew install act
```

### 2. Verify Setup (Optional)

```bash
# Check Docker is running
docker info

# List available workflows
act -l

# Check configuration
cat .actrc
```

### 3. Run Your First Test

**Option A: Quick native test** (2-3 minutes)
```bash
./test-ci-locally.sh
```

**Option B: Full act test** (first run: 10-15 min, subsequent: 5 min)
```bash
./test-ci-with-act.sh test
```

### 4. Integrate Into Workflow

**Daily development:**
```bash
# Make changes
vim src/main.rs

# Quick test
cargo test

# Full local CI
./test-ci-locally.sh
```

**Before pushing:**
```bash
# Native tests
./test-ci-locally.sh

# act validation (optional but recommended)
./test-ci-with-act.sh

# Push with confidence
git push
```

## Testing Approaches

### Approach 1: Native Testing (Recommended for daily dev)

**Command:** `./test-ci-locally.sh`

**Pros:**
- ⚡ Fast (no Docker overhead)
- 🍎 Perfect for Apple Silicon MLX tests
- 📴 Works offline
- 🎯 Tests your actual environment

**When to use:**
- During active development
- When iterating quickly
- For platform-specific tests (MLX)
- When Docker isn't available

### Approach 2: act Testing (Recommended pre-push)

**Command:** `./test-ci-with-act.sh`

**Pros:**
- 🎭 Emulates GitHub's exact environment
- 🔄 Tests matrix builds (Ubuntu/macOS/Windows)
- 🐛 Catches platform-specific bugs
- ✅ High confidence CI will pass

**When to use:**
- Before pushing to GitHub
- When testing workflow changes
- For final pre-merge validation
- When you want maximum confidence

## Key Commands

```bash
# Native testing (fast)
./test-ci-locally.sh

# Docker testing (high fidelity)
./test-ci-with-act.sh

# Run specific job
./test-ci-with-act.sh test

# List all jobs
./test-ci-with-act.sh --list

# Show help
./test-ci-with-act.sh --help
```

## Configuration Files

| File | Purpose | Required? |
|------|---------|-----------|
| `.actrc` | act settings | ✅ Auto-loaded by act |
| `.secrets` | Workflow secrets | ⚠️ Optional (most tests work without) |
| `.secrets.example` | Secrets template | ℹ️ Reference only |

## Documentation Structure

```
docs/
├── LOCAL_CI_TESTING.md    # Full guide (read for deep dive)
├── ACT_QUICKREF.md        # Quick reference (1-page cheat sheet)
└── ACT_SETUP.md           # Setup guide (5-minute install)
```

**Start here:** `docs/ACT_SETUP.md` for installation  
**Daily reference:** `docs/ACT_QUICKREF.md` for commands  
**Deep dive:** `docs/LOCAL_CI_TESTING.md` for everything else

## Workflow Coverage

Both scripts test **all GitHub Actions workflows**:

### CI Workflow (`.github/workflows/ci.yml`)
- ✅ Formatting check (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Test suite (`cargo test`)
- ✅ Security audit (`cargo audit`)
- ✅ Release builds
- ✅ Benchmarks
- ✅ Code coverage

### Publish Workflow (`.github/workflows/publish.yml`)
- ✅ Version consistency
- ✅ MLX test compilation
- ✅ Package creation

### Release Workflow (`.github/workflows/release.yml`)
- ✅ Platform E2E tests
- ✅ Contract tests
- ✅ Artifact creation

## Apple Silicon Notes

For **M1/M2/M3 Macs**, native testing is recommended for MLX backend tests:

```bash
./test-ci-locally.sh
```

This runs real MLX inference tests on Metal hardware. `act` can't access Metal APIs inside Docker, so it verifies MLX tests **compile** but can't run them.

## First Run Expectations

### Native Testing
- **First run:** 3-5 minutes (cargo build + test)
- **Subsequent:** 1-2 minutes (cached builds)

### act Testing
- **First run:** 10-15 minutes (downloads ~2GB Docker images)
- **Subsequent:** 5-7 minutes (images cached)

## Secrets (Optional)

Most workflows work without secrets. Only create `.secrets` if you see "secret not found" errors:

```bash
# Create from template
cp .secrets.example .secrets

# Edit with your tokens
nano .secrets
```

**Required tokens:**
- `GITHUB_TOKEN` - For GitHub API access (generate at https://github.com/settings/tokens)
- `CODECOV_TOKEN` - For coverage uploads (optional)
- `CARGO_REGISTRY_TOKEN` - For crate publishing (optional)

## Troubleshooting Quick Reference

| Error | Solution |
|-------|----------|
| `act: command not found` | `brew install act` |
| `Cannot connect to Docker` | `open -a Docker` (start Docker Desktop) |
| `exec format error` | Add `--container-architecture linux/amd64` |
| Container killed (OOM) | Increase Docker memory or reduce in `.actrc` |
| Tests fail in act only | Check env vars, verify tools in runner image |
| Slow first run | Normal - downloading images, subsequent runs faster |

**Full troubleshooting:** See `docs/LOCAL_CI_TESTING.md`

## Summary

You now have **two powerful ways** to test CI locally:

1. **Native testing** (`./test-ci-locally.sh`) - Fast, accurate for your platform
2. **act testing** (`./test-ci-with-act.sh`) - High fidelity, matches GitHub's environment

Use native testing for rapid iteration, then run `act` before pushing for maximum confidence that CI will pass.

**Next step:** Install act with `brew install act` and run your first test!

## Resources

- **act repository:** https://github.com/nektos/act
- **Runner images:** https://github.com/catthehacker/docker_images
- **Setup guide:** `docs/ACT_SETUP.md`
- **Quick reference:** `docs/ACT_QUICKREF.md`
- **Full documentation:** `docs/LOCAL_CI_TESTING.md`

---

**Setup complete!** Install act and start testing: `brew install act && ./test-ci-with-act.sh`
