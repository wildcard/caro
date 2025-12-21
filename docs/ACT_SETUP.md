# Setting Up Local CI Testing with act

## Quick Setup (5 minutes)

### 1. Install act

**macOS:**
```bash
brew install act
```

**Linux:**
```bash
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

**Windows (Chocolatey):**
```bash
choco install act-cli
```

**Manual install:** Download from [act releases](https://github.com/nektos/act/releases)

### 2. Verify Docker

```bash
# Check Docker is running
docker info

# If not, start Docker Desktop
open -a Docker  # macOS
```

### 3. Test Installation

```bash
# List available workflows
act -l

# You should see jobs from .github/workflows/ci.yml
```

### 4. Create Secrets File (Optional)

```bash
# Copy example
cp .secrets.example .secrets

# Edit with your tokens (if needed)
nano .secrets
```

Add real values for:
- `GITHUB_TOKEN` - For GitHub API access (generate at https://github.com/settings/tokens)
- `CODECOV_TOKEN` - For code coverage uploads (optional)
- `CARGO_REGISTRY_TOKEN` - For crate publishing (optional)

**Note:** Most tests don't require secrets. Only create `.secrets` if workflows fail with "secret not found".

### 5. Run Your First Test

```bash
# Test the CI workflow
./test-ci-with-act.sh test
```

**First run:** Downloads runner images (~2GB), takes 5-10 minutes.  
**Subsequent runs:** Much faster (images are cached).

---

## Configuration

### `.actrc` - Runner Settings

The project includes pre-configured settings in `.actrc`:

```bash
# GitHub-compatible runner images
-P ubuntu-latest=ghcr.io/catthehacker/ubuntu:act-latest

# Verbose output for debugging
-v

# Reuse containers for speed
--reuse

# Resource limits (adjust for your machine)
--container-options "--cpus=4 --memory=8g"
```

**Customize for your machine:**

```bash
# Edit .actrc
nano .actrc

# Reduce resources for older machines:
--container-options "--cpus=2 --memory=4g"
```

### `.secrets` - Secret Management

**Never commit `.secrets`** - it's gitignored for security.

**Minimal secrets** (most workflows work without secrets):
```bash
# .secrets
GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**Full secrets** (for all workflows):
```bash
# .secrets
GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
CODECOV_TOKEN=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
CARGO_REGISTRY_TOKEN=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

---

## Usage

### Run All Tests

```bash
./test-ci-with-act.sh
```

This runs all jobs in `.github/workflows/ci.yml`:
- ✅ Formatting check (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Test suite (`cargo test`)
- ✅ Security audit (`cargo audit`)
- ✅ Release builds
- ✅ Benchmarks
- ✅ Code coverage

### Run Specific Job

```bash
# Just the test job
./test-ci-with-act.sh test

# Just security audit
./test-ci-with-act.sh security

# Just builds
./test-ci-with-act.sh build
```

### List Available Jobs

```bash
./test-ci-with-act.sh --list
```

Output:
```
Stage  Job ID      Job name        Workflow name  Workflow file  Events
0      test        Test Suite      CI             ci.yml         push,pull_request
0      security    Security Audit  CI             ci.yml         push,pull_request
0      build       Build Release   CI             ci.yml         push,pull_request
0      benchmark   Benchmarks      CI             ci.yml         push,pull_request
0      coverage    Code Coverage   CI             ci.yml         push,pull_request
```

---

## Comparison: Native vs. act

You have **two testing options**:

### Native Testing (Recommended for daily dev)

```bash
./test-ci-locally.sh
```

**Pros:**
- ⚡ Fast (no Docker overhead)
- 🎯 Tests your actual environment
- 🍎 Perfect for Apple Silicon MLX tests
- 📴 Works offline

**Best for:** Daily development, rapid iteration

### act Testing (Recommended pre-push)

```bash
./test-ci-with-act.sh
```

**Pros:**
- 🎭 Emulates GitHub's exact environment
- 🔄 Tests matrix builds (Ubuntu/macOS/Windows)
- 🐛 Catches platform-specific bugs
- ✅ High confidence CI will pass

**Best for:** Pre-merge validation, workflow testing

**Recommendation:** Use native testing during development, then run `act` before pushing.

---

## Apple Silicon (M1/M2/M3) Notes

### Native Testing Preferred

For **MLX backend tests**, use native testing:

```bash
./test-ci-locally.sh
```

This runs real MLX tests on Metal hardware.

### act Limitations

`act` runs in Docker, which can't access Metal APIs:

- ❌ Can't run MLX inference tests
- ✅ Can verify MLX tests **compile** on Ubuntu
- ✅ Can test all other backends (vLLM, Ollama, CPU)

### Architecture Flag

If you get architecture errors:

```bash
# Force amd64 architecture
act --container-architecture linux/amd64

# Or add to .actrc:
echo "--container-architecture linux/amd64" >> .actrc
```

---

## Troubleshooting

### Error: `act: command not found`

**Solution:** Install act (see step 1 above)

### Error: `Cannot connect to Docker daemon`

**Solution:**
```bash
# Start Docker Desktop
open -a Docker  # macOS
sudo systemctl start docker  # Linux

# Verify it's running
docker info
```

### Error: `Segmentation fault` or `exec format error`

**Cause:** Architecture mismatch (Apple Silicon running x86_64 images)

**Solution:**
```bash
# Force amd64 architecture
act --container-architecture linux/amd64
```

### Error: Container killed (OOM)

**Cause:** Not enough memory allocated to Docker

**Solution 1:** Increase Docker memory (Docker Desktop → Settings → Resources)

**Solution 2:** Reduce memory in `.actrc`:
```bash
--container-options "--cpus=2 --memory=4g"
```

**Solution 3:** Run single-threaded tests:
```bash
# In workflow, use:
cargo test -- --test-threads=1
```

### Slow First Run

**Cause:** Downloading runner images (~2GB)

**Solution:** Be patient - first run takes 5-10 minutes. Subsequent runs are much faster due to caching.

**Pre-download images:**
```bash
docker pull ghcr.io/catthehacker/ubuntu:act-latest
```

### Jobs Fail in act But Pass Locally

**Possible causes:**

1. **Environment differences**
   - Check env vars in workflow
   - Verify tools are installed in runner image

2. **Missing secrets**
   - Create `.secrets` file
   - Add required tokens

3. **Platform-specific code**
   - Some tests require native platform (MLX)
   - Use native testing for these

**Debug:**
```bash
# Run with verbose logging
act -v -j test

# Inspect runner container
docker run -it ghcr.io/catthehacker/ubuntu:act-latest bash

# Check installed tools
which cargo
rustc --version
```

---

## Development Workflow

### Daily Development Loop

```bash
# 1. Make changes
vim src/main.rs

# 2. Quick test (native)
cargo test

# 3. Full local CI check
./test-ci-locally.sh

# 4. Fix any issues, repeat
```

### Pre-Push Validation

```bash
# 1. Run native tests
./test-ci-locally.sh

# 2. Run act validation
./test-ci-with-act.sh

# 3. Push with confidence
git push
```

### Pre-Merge Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] `cargo audit` passes
- [ ] Binary size < 50MB
- [ ] `./test-ci-locally.sh` passes
- [ ] (Optional) `./test-ci-with-act.sh` passes
- [ ] All tests green on GitHub Actions

---

## Advanced Usage

### Run Specific Workflow File

```bash
# Run ci.yml only
act -W .github/workflows/ci.yml

# Run publish.yml
act -W .github/workflows/publish.yml
```

### Test Different Events

```bash
# Test push event (default)
./test-ci-with-act.sh test push

# Test pull_request event
./test-ci-with-act.sh test pull_request

# Test release event
act release
```

### Dry Run (Show What Would Run)

```bash
# Show jobs without running
act -n

# Show specific job
act -n -j test
```

### Custom Runner Images

```bash
# Use different image (e.g., smaller)
act -P ubuntu-latest=node:16-buster

# Or use official GitHub images (larger)
act -P ubuntu-latest=ghcr.io/actions/runner:latest
```

### Debug Mode

```bash
# Verbose logging
act -v

# Even more verbose
act -v -v

# Debug specific step
act --debug -j test
```

---

## Resources

### Documentation

- **Full guide:** [`docs/LOCAL_CI_TESTING.md`](./LOCAL_CI_TESTING.md)
- **Quick reference:** [`docs/ACT_QUICKREF.md`](./ACT_QUICKREF.md)
- **act repository:** https://github.com/nektos/act
- **Runner images:** https://github.com/catthehacker/docker_images

### Scripts

- **`./test-ci-with-act.sh`** - Run workflows with act
- **`./test-ci-locally.sh`** - Run tests natively
- **`.actrc`** - act configuration
- **`.secrets.example`** - Secrets template

### Workflows

- **`.github/workflows/ci.yml`** - Main CI workflow
- **`.github/workflows/publish.yml`** - Package publish workflow
- **`.github/workflows/release.yml`** - Release workflow

---

## Next Steps

1. **Install act:** `brew install act`
2. **Test it works:** `./test-ci-with-act.sh --list`
3. **Run a job:** `./test-ci-with-act.sh test`
4. **Integrate into workflow:** Run before every push

**Questions?** Check [`docs/LOCAL_CI_TESTING.md`](./LOCAL_CI_TESTING.md) for detailed troubleshooting.

---

**Happy testing!** 🚀
