# Initial GitHub Issues for V1.0 Launch

> **15 ready-to-create issues for cmdai V1.0**
>
> Copy-paste these into GitHub Issues with the specified labels and milestones.

---

## Issue #1: Performance Optimization - Achieve <100ms Startup Time

**Title**: `[v1.0] Optimize CLI startup time to <100ms`

**Labels**: `type: feature`, `priority: P0`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 5 story points

**Description**:

## Goal
Reduce CLI startup time to <100ms (cold start) for release builds to meet V1.0 performance requirements.

## Current Status
- Current startup time: ~200ms (estimated, needs benchmarking)
- Target: <100ms for release builds, <50ms ideal

## Problem
Slow startup impacts user experience, especially for quick one-off commands. Users expect near-instant response when typing `cmdai "command"`.

## Tasks
- [ ] Benchmark current startup performance with `hyperfine`
- [ ] Profile with `cargo flamegraph` to identify hot paths
- [ ] Lazy-load dependencies (defer initialization until first use)
- [ ] Optimize config file parsing (lazy TOML parsing)
- [ ] Cache compiled regexes in safety validator
- [ ] Reduce binary bloat (remove unused dependencies)
- [ ] Re-benchmark and verify <100ms target met
- [ ] Document performance optimizations for future reference

## Acceptance Criteria
- [ ] Release build starts in <100ms on M1 Mac
- [ ] Debug build starts in <200ms (acceptable for dev)
- [ ] All tests still passing
- [ ] No functionality regressions
- [ ] Performance benchmarks added to CI

## Related
- See [MVP_TO_V1.md#1-performance-optimization](../MVP_TO_V1.md#1-performance-optimization-1-2-weeks)
- Blocked by: None
- Blocks: V1.0 launch

## Technical Notes
Potential optimizations:
- Use `once_cell` for lazy static initialization
- Defer regex compilation until safety validation needed
- Minimize allocations in hot paths
- Consider stripping unused features from dependencies

---

## Issue #2: Binary Size Reduction - Target <50MB

**Title**: `[v1.0] Reduce binary size to <50MB`

**Labels**: `type: feature`, `priority: P0`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 3 story points

**Description**:

## Goal
Reduce release binary size to <50MB to make distribution and installation faster.

## Current Status
- Current binary size: ~70MB (needs reduction)
- Target: <50MB without embedded models

## Problem
Large binaries:
- Slow to download
- Take up unnecessary disk space
- Harder to distribute via package managers
- Poor user experience

## Tasks
- [ ] Analyze binary with `cargo bloat --release --crates`
- [ ] Remove unused dependencies (audit Cargo.toml)
- [ ] Feature-gate large dependencies (MLX, Candle with conditional compilation)
- [ ] Use dynamic linking for system libraries (openssl, libz)
- [ ] Enable aggressive LTO optimization (verify already enabled)
- [ ] Strip debug symbols in release (verify already enabled)
- [ ] Consider UPX compression (optional, test impact)
- [ ] Test all features still work after size reduction

## Acceptance Criteria
- [ ] Release binary <50MB on all platforms
- [ ] All features functional
- [ ] No significant performance regression (<5% slower acceptable)
- [ ] Cross-platform compatibility maintained

## Related
- See [MVP_TO_V1.md#2-binary-size-reduction](../MVP_TO_V1.md#2-binary-size-reduction-1-week)
- Blocks: Package distribution, V1.0 launch

## Technical Notes
```toml
# Cargo.toml optimizations already in place:
[profile.release]
lto = true
strip = true
codegen-units = 1
```

Focus areas:
- Dependency audit (remove unused crates)
- Feature flags for optional backends
- Static vs dynamic linking trade-offs

---

## Issue #3: Homebrew Formula for macOS Installation

**Title**: `[v1.0] Create Homebrew formula for easy macOS installation`

**Labels**: `type: infra`, `priority: P0`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 2 story points

**Description**:

## Goal
Enable one-command installation on macOS via Homebrew.

## Desired Installation Flow
```bash
brew tap wildcard/cmdai
brew install cmdai
cmdai --version
```

## Tasks
- [ ] Create Homebrew tap repository (`homebrew-cmdai`)
- [ ] Write Formula file (`Formula/cmdai.rb`)
- [ ] Test installation on fresh macOS (Intel + Apple Silicon)
- [ ] Add SHA256 hash for release tarball
- [ ] Sign and notarize binary for macOS Gatekeeper
- [ ] Update README.md with installation instructions
- [ ] Test `brew upgrade cmdai` workflow
- [ ] (Optional) Submit to Homebrew core after validation

## Formula Template
```ruby
class Cmdai < Formula
  desc "AI-native shell command generator using local LLMs"
  homepage "https://github.com/wildcard/cmdai"
  url "https://github.com/wildcard/cmdai/releases/download/v1.0.0/cmdai-1.0.0.tar.gz"
  sha256 "..." # Generated from release tarball
  license "MIT"
  version "1.0.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "cmdai 1.0.0", shell_output("#{bin}/cmdai --version")
  end
end
```

## Acceptance Criteria
- [ ] `brew install cmdai` works on macOS (both Intel and ARM)
- [ ] Binary in PATH after installation
- [ ] All dependencies bundled or auto-installed
- [ ] Uninstall works (`brew uninstall cmdai`)
- [ ] Upgrade path works (`brew upgrade cmdai`)
- [ ] Formula passes `brew audit`

## Related
- See [MVP_TO_V1.md#3-package-distribution](../MVP_TO_V1.md#3-package-distribution-1-week)
- Depends on: Binary signing/notarization process
- Blocks: macOS launch announcement

---

## Issue #4: Publish to crates.io

**Title**: `[v1.0] Publish cmdai to crates.io`

**Labels**: `type: infra`, `priority: P0`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 1 story point

**Description**:

## Goal
Enable installation via `cargo install cmdai`

## Tasks
- [ ] Verify Cargo.toml metadata is complete
  - [ ] Description
  - [ ] License (MIT/Apache 2.0)
  - [ ] Repository URL
  - [ ] Keywords
  - [ ] Categories
- [ ] Add README to crates.io package
- [ ] Test local publish with `cargo publish --dry-run`
- [ ] Create crates.io API token
- [ ] Publish v1.0.0 to crates.io
- [ ] Verify listing appears correctly
- [ ] Test installation: `cargo install cmdai`
- [ ] Add crates.io badge to README

## Cargo.toml Checklist
```toml
[package]
name = "cmdai"
version = "1.0.0"
description = "AI-native shell command generator with local LLM support"
license = "MIT OR Apache-2.0"
repository = "https://github.com/wildcard/cmdai"
keywords = ["cli", "ai", "llm", "shell", "command"]
categories = ["command-line-utilities", "development-tools"]
readme = "README.md"
```

## Acceptance Criteria
- [ ] Package published successfully
- [ ] `cargo install cmdai` works
- [ ] Documentation renders correctly on crates.io
- [ ] All links work
- [ ] Correct version displayed

## Related
- See [MVP_TO_V1.md#3-package-distribution](../MVP_TO_V1.md#3-package-distribution-1-week)
- Blocks: V1.0 announcement

---

## Issue #5: End-to-End Integration Testing

**Title**: `[v1.0] Comprehensive E2E integration tests for all user workflows`

**Labels**: `type: test`, `priority: P0`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 5 story points

**Description**:

## Goal
Create comprehensive end-to-end tests covering all critical user workflows.

## Test Scenarios Needed

### High Priority (P0)
- [ ] **First-time user workflow**
  - Install → Run first command → Execute
- [ ] **Backend fallback scenarios**
  - Ollama down → fallback to CPU backend
  - MLX unavailable → fallback to CPU
- [ ] **Safety validation**
  - Dangerous command blocked (`rm -rf /`)
  - User confirmation for moderate risk commands
  - Safe commands execute without prompt
- [ ] **Configuration scenarios**
  - Custom config file loaded
  - Environment variable overrides
  - Multiple backend configurations

### Medium Priority (P1)
- [ ] **Error handling**
  - Invalid prompts
  - Network errors (for remote backends)
  - Model not found errors
- [ ] **Cross-platform compatibility**
  - macOS (Intel + Apple Silicon)
  - Linux (Ubuntu, Fedora)
  - Windows (WSL + native)

## Test Framework Structure
```rust
// tests/e2e/user_workflows.rs
#[test]
fn test_first_time_user_workflow() {
    // 1. Simulate fresh install
    // 2. Run: cmdai "list files"
    // 3. Verify safe command generated
    // 4. Mock execution
    // 5. Verify output
}

#[test]
fn test_backend_fallback() {
    // 1. Configure Ollama backend
    // 2. Simulate Ollama unavailable
    // 3. Verify fallback to CPU
    // 4. Verify command still generated
}
```

## Acceptance Criteria
- [ ] 10+ E2E scenarios covered
- [ ] All tests passing on CI (GitHub Actions)
- [ ] Cross-platform tests run on matrix (macOS, Linux, Windows)
- [ ] Test coverage >80% for critical paths
- [ ] Tests run in <5 minutes total

## Related
- See [MVP_TO_V1.md#4-integration-testing](../MVP_TO_V1.md#4-integration-testing-1-week)
- Blocks: V1.0 quality assurance

## Technical Notes
Use `assert_cmd` for CLI testing:
```rust
use assert_cmd::Command;

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("cmdai").unwrap();
    cmd.arg("--version")
       .assert()
       .success()
       .stdout(predicates::str::contains("cmdai 1.0.0"));
}
```

---

## Issue #6: Documentation Polish for V1.0

**Title**: `[v1.0] Polish and complete documentation for production launch`

**Labels**: `type: docs`, `priority: P1`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 3 story points

**Description**:

## Goal
Ensure all documentation is clear, complete, and ready for new users.

## Documentation Needs

### Critical (P0)
- [ ] **README.md updates**
  - Clear "Quick Start" section (5-minute getting started)
  - Installation instructions (all platforms)
  - Usage examples (10+ common scenarios)
  - Troubleshooting FAQ
  - Badges (version, build status, downloads)

- [ ] **INSTALLATION.md** (new file)
  - Platform-specific instructions
  - Dependency requirements
  - Verification steps
  - Common installation issues

- [ ] **USAGE_GUIDE.md** (new file)
  - Basic usage patterns
  - Advanced features (custom backends, config)
  - Safety configuration and risk levels
  - Backend selection guide

### Important (P1)
- [ ] **API documentation (rustdoc)**
  - All public APIs documented
  - Examples for common use cases
  - Architecture overview
  - Run `cargo doc --no-deps --open` to verify

- [ ] **VIDEO_TUTORIAL.md** (or YouTube video)
  - 2-3 minute demo video
  - Installation → First command → Execution
  - Upload to YouTube, embed in README

### Nice to Have (P2)
- [ ] **CONTRIBUTING.md** updates
  - Development workflow
  - Testing guidelines
  - Code style guide

## Acceptance Criteria
- [ ] New user can install and use cmdai in <5 minutes
- [ ] All common questions answered in docs
- [ ] rustdoc coverage >80% for public APIs
- [ ] Zero broken links
- [ ] Spelling/grammar checked

## Related
- See [MVP_TO_V1.md#5-documentation-polish](../MVP_TO_V1.md#5-documentation-polish-1-week)
- Blocks: V1.0 launch announcement

## Style Guidelines
- Use clear, concise language
- Include code examples for every feature
- Add screenshots/terminal recordings where helpful
- Keep README concise, link to detailed docs

---

## Issue #7: Security Audit and Dependency Hardening

**Title**: `[v1.0] Complete security audit and fix all vulnerabilities`

**Labels**: `type: infra`, `priority: P0`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 3 story points

**Description**:

## Goal
Ensure cmdai has no known security vulnerabilities before V1.0 launch.

## Security Checklist

### Dependency Security
- [ ] Run `cargo audit` → Fix all HIGH/CRITICAL issues
- [ ] Run `cargo deny check` → Fix license incompatibilities
- [ ] Review dependency tree for suspicious packages
- [ ] Update all dependencies to latest secure versions
- [ ] Add `cargo-audit` to CI pipeline

### Code Security
- [ ] Run `cargo clippy -- -W clippy::all -W clippy::pedantic`
- [ ] Static analysis with `cargo semver-checks`
- [ ] Review command execution for injection vulnerabilities
- [ ] Ensure no API keys or secrets logged
- [ ] File permissions hardening (config files should be 0600)

### Optional but Recommended
- [ ] Fuzzing for safety validator
  ```bash
  cargo install cargo-fuzz
  cargo fuzz run safety_validator
  ```
- [ ] Manual security review of critical modules
  - `src/safety/` - Command validation
  - `src/backends/` - LLM interaction
  - `src/config/` - Configuration parsing

## Acceptance Criteria
- [ ] Zero HIGH or CRITICAL vulnerabilities
- [ ] No GPL/AGPL dependencies (check licensing)
- [ ] SECURITY.md file created with disclosure policy
- [ ] Security scanning added to CI/CD
- [ ] All clippy warnings addressed

## Related
- See [MVP_TO_V1.md#6-security-audit](../MVP_TO_V1.md#6-security-audit-1-week)
- Blocks: V1.0 launch

## Security Policy Template
Create `SECURITY.md`:
```markdown
# Security Policy

## Reporting a Vulnerability

Please report security vulnerabilities to security@cmdai.dev
Do NOT open public GitHub issues for security vulnerabilities.

## Supported Versions
| Version | Supported |
|---------|-----------|
| 1.0.x   | ✅        |
| < 1.0   | ❌        |
```

---

## Issue #8: Shell Completion Scripts

**Title**: `[v1.0] Generate shell completions for Bash, Zsh, Fish`

**Labels**: `type: feature`, `priority: P2`, `component: cli`, `phase: v1.0`, `good-first-issue`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 2 story points

**Description**:

## Goal
Provide tab completion for cmdai commands across popular shells.

## Shells to Support
- [ ] Bash
- [ ] Zsh
- [ ] Fish
- [ ] PowerShell (Windows)

## Implementation
Use `clap_complete` to generate completion scripts:

```rust
// build.rs or separate binary
use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

fn main() {
    let mut cmd = Cli::command();
    let outdir = std::path::PathBuf::from("completions/");

    generate_to(Shell::Bash, &mut cmd, "cmdai", &outdir).unwrap();
    generate_to(Shell::Zsh, &mut cmd, "cmdai", &outdir).unwrap();
    generate_to(Shell::Fish, &mut cmd, "cmdai", &outdir).unwrap();
    generate_to(Shell::PowerShell, &mut cmd, "cmdai", &outdir).unwrap();
}
```

## Installation
Document how to install completions:

```bash
# Bash
sudo cp completions/cmdai.bash /usr/share/bash-completion/completions/cmdai

# Zsh
cp completions/_cmdai ~/.zsh/completion/

# Fish
cp completions/cmdai.fish ~/.config/fish/completions/
```

## Acceptance Criteria
- [ ] Completions generated for all 4 shells
- [ ] Tab completion works for:
  - [ ] Subcommands
  - [ ] Flags (`--help`, `--version`, etc.)
  - [ ] Options (`--backend`, `--config`, etc.)
- [ ] Installation instructions in README
- [ ] Completions bundled in release artifacts

## Related
- See [MVP_TO_V1.md#7-shell-completions](../MVP_TO_V1.md#7-shell-completions-3-days)
- Nice to have for V1.0, not blocking

---

## Issue #9: Man Pages for Offline Documentation

**Title**: `[v1.0] Generate man pages for offline documentation`

**Labels**: `type: docs`, `priority: P2`, `component: cli`, `phase: v1.0`, `good-first-issue`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 1 story point

**Description**:

## Goal
Provide offline documentation via `man cmdai`

## Tasks
- [ ] Generate man pages from clap CLI definitions
- [ ] Create `cmdai.1` man page
- [ ] Include in release artifacts
- [ ] Add installation instructions for package managers
- [ ] Test: `man cmdai` displays correctly

## Installation
Man pages should install to:
- Linux: `/usr/local/share/man/man1/cmdai.1`
- macOS: `/usr/local/share/man/man1/cmdai.1`

## Man Page Sections
```
NAME
    cmdai - AI-native shell command generator

SYNOPSIS
    cmdai [OPTIONS] <PROMPT>

DESCRIPTION
    cmdai generates safe shell commands from natural language prompts...

OPTIONS
    -b, --backend <BACKEND>
        LLM backend to use (mlx, cpu, ollama, vllm)

EXAMPLES
    cmdai "list all large files"
    cmdai --backend ollama "compress this directory"

SEE ALSO
    bash(1), zsh(1), fish(1)
```

## Acceptance Criteria
- [ ] Man page generated and formatted correctly
- [ ] All commands and options documented
- [ ] Examples included
- [ ] Installs correctly with package managers

## Related
- See [MVP_TO_V1.md#8-man-pages](../MVP_TO_V1.md#8-man-pages-2-days)
- Nice to have for V1.0

---

## Issue #10: GitHub Release Automation

**Title**: `[v1.0] Automate GitHub releases with precompiled binaries`

**Labels**: `type: infra`, `priority: P1`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 3 story points

**Description**:

## Goal
Automate creation of GitHub releases with precompiled binaries for all platforms.

## Platforms to Support
- [ ] macOS (Apple Silicon - aarch64)
- [ ] macOS (Intel - x86_64)
- [ ] Linux (x86_64, glibc)
- [ ] Linux (x86_64, musl - for Alpine/static builds)
- [ ] Windows (x86_64)

## GitHub Actions Workflow
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v3
```

## Tasks
- [ ] Create `.github/workflows/release.yml`
- [ ] Cross-compilation setup for all targets
- [ ] Binary signing (macOS notarization)
- [ ] Generate SHA256 checksums
- [ ] Create release notes template
- [ ] Auto-publish to GitHub Releases
- [ ] Test full release workflow

## Release Artifacts
Each release should include:
- Precompiled binaries for all platforms
- SHA256 checksums (`checksums.txt`)
- Source tarball
- CHANGELOG for this version

## Acceptance Criteria
- [ ] Pushing a tag triggers automated release
- [ ] All platform binaries built successfully
- [ ] Binaries are signed and notarized (macOS)
- [ ] Release notes auto-generated from commits
- [ ] Download counts tracked

## Related
- Blocks: V1.0 launch
- Enables: Easy distribution

---

## Issue #11: Performance Benchmarking Suite

**Title**: `[v1.0] Create comprehensive performance benchmarks`

**Labels**: `type: test`, `priority: P1`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 3 story points

**Description**:

## Goal
Establish performance benchmarks to track regressions and measure improvements.

## Benchmarks Needed

### Startup Performance
- [ ] Cold start time (first run)
- [ ] Warm start time (cached)
- [ ] Memory usage at startup

### Inference Performance
- [ ] MLX backend inference time
- [ ] CPU backend inference time
- [ ] Ollama backend inference time
- [ ] vLLM backend inference time

### Safety Validation
- [ ] Pattern matching performance
- [ ] POSIX validation speed
- [ ] Risk assessment time

## Implementation
```rust
// benches/startup.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_startup(c: &mut Criterion) {
    c.bench_function("cold_start", |b| {
        b.iter(|| {
            // Measure CLI initialization time
            cmdai::Cli::new()
        });
    });
}

criterion_group!(benches, benchmark_startup);
criterion_main!(benches);
```

## CI Integration
- [ ] Run benchmarks on every PR
- [ ] Track performance over time
- [ ] Alert on >10% regressions
- [ ] Publish benchmark results

## Acceptance Criteria
- [ ] Benchmarks for all critical paths
- [ ] CI runs benchmarks automatically
- [ ] Baseline established for V1.0
- [ ] Performance dashboard (optional)

## Target Metrics (V1.0)
- Startup: <100ms
- MLX inference: <2s
- Safety validation: <50ms

## Related
- Supports: Issue #1 (Performance Optimization)
- Prevents: Performance regressions

---

## Issue #12: Debian Package (.deb) for Ubuntu/Debian

**Title**: `[v1.0] Create .deb package for Ubuntu/Debian installation`

**Labels**: `type: infra`, `priority: P1`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 2 story points

**Description**:

## Goal
Enable easy installation on Debian-based Linux distributions.

## Installation Flow
```bash
sudo dpkg -i cmdai_1.0.0_amd64.deb
# or
sudo apt install ./cmdai_1.0.0_amd64.deb
```

## Tasks
- [ ] Create Debian package structure
  ```
  cmdai_1.0.0_amd64/
  ├── DEBIAN/
  │   ├── control
  │   └── postinst
  └── usr/
      ├── bin/
      │   └── cmdai
      └── share/
          ├── man/man1/
          │   └── cmdai.1.gz
          └── doc/cmdai/
              └── README.md
  ```
- [ ] Write `control` file with metadata
- [ ] Test installation on Ubuntu 22.04, 24.04
- [ ] Test installation on Debian 11, 12
- [ ] Add to release automation
- [ ] Document installation in README

## Debian Control File
```
Package: cmdai
Version: 1.0.0
Section: utils
Priority: optional
Architecture: amd64
Maintainer: cmdai Team <team@cmdai.dev>
Description: AI-native shell command generator
 cmdai generates safe shell commands from natural language
 using local LLMs with comprehensive safety validation.
```

## Acceptance Criteria
- [ ] `.deb` package installs cleanly
- [ ] Binary in PATH after install
- [ ] Man page accessible
- [ ] Uninstall works: `sudo apt remove cmdai`
- [ ] No lintian errors

## Related
- See [MVP_TO_V1.md#3-package-distribution](../MVP_TO_V1.md#3-package-distribution-1-week)
- Complements: Homebrew formula (Issue #3)

---

## Issue #13: Configuration Validation and Error Messages

**Title**: `[v1.0] Improve config validation with helpful error messages`

**Labels**: `type: feature`, `priority: P1`, `component: cli`, `phase: v1.0`, `good-first-issue`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 2 story points

**Description**:

## Goal
Provide clear, actionable error messages when configuration is invalid.

## Current Problem
When config is malformed, errors are cryptic:
```
Error: failed to parse config
```

## Desired Behavior
```
Error: Invalid configuration in ~/.config/cmdai/config.toml

Line 5: backend = "invalid-backend"
        ^
Unknown backend "invalid-backend"

Valid backends: mlx, cpu, ollama, vllm

Hint: See https://github.com/wildcard/cmdai#configuration for examples
```

## Tasks
- [ ] Add validation for all config fields
- [ ] Provide helpful error messages with:
  - [ ] Location (file, line number)
  - [ ] What's wrong
  - [ ] Valid options
  - [ ] Link to documentation
- [ ] Validate on startup before running
- [ ] Add `cmdai config validate` command
- [ ] Add `cmdai config init` to generate default config
- [ ] Update error messages to be beginner-friendly

## Validation Checks
- [ ] Backend exists and is available
- [ ] Paths are valid (config_dir, cache_dir)
- [ ] Boolean values are true/false
- [ ] Enum values match allowed options
- [ ] Required fields are present

## Acceptance Criteria
- [ ] All config errors have helpful messages
- [ ] Error messages include line numbers
- [ ] Suggestions for fixes provided
- [ ] Links to docs included
- [ ] `cmdai config validate` works

## Related
- Improves: User experience for new users
- Good first issue: Clear scope, well-defined

---

## Issue #14: Logging and Verbosity Controls

**Title**: `[v1.0] Add logging levels and verbosity controls`

**Labels**: `type: feature`, `priority: P2`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 2 story points

**Description**:

## Goal
Give users control over logging verbosity for debugging and normal use.

## Desired CLI Flags
```bash
cmdai -v "list files"          # Verbose (INFO level)
cmdai -vv "list files"         # Very verbose (DEBUG level)
cmdai -vvv "list files"        # Extremely verbose (TRACE level)
cmdai -q "list files"          # Quiet (ERROR only)
cmdai --log-file debug.log     # Log to file
```

## Current Behavior
Logging is controlled only by `RUST_LOG` environment variable, which is not user-friendly.

## Tasks
- [ ] Add `-v/--verbose` flag (incremental verbosity)
- [ ] Add `-q/--quiet` flag (suppress non-critical output)
- [ ] Add `--log-file <PATH>` flag
- [ ] Map verbosity to log levels:
  - Default: WARN
  - `-v`: INFO
  - `-vv`: DEBUG
  - `-vvv`: TRACE
  - `-q`: ERROR only
- [ ] Ensure logs don't leak sensitive data (commands, paths OK; API keys NO)
- [ ] Add structured logging (JSON format with `--log-json`)
- [ ] Document in `--help` and README

## Implementation
```rust
use tracing_subscriber::EnvFilter;

let log_level = match verbosity {
    0 => "warn",
    1 => "info",
    2 => "debug",
    _ => "trace",
};

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(log_level))
    .init();
```

## Acceptance Criteria
- [ ] Verbosity flags work as documented
- [ ] Quiet mode suppresses all non-essential output
- [ ] Log files created when `--log-file` specified
- [ ] Sensitive data never logged
- [ ] Documentation updated

## Related
- Helps: Debugging (Issue #5 - E2E testing)
- Improves: User experience for troubleshooting

---

## Issue #15: Launch Checklist and Release Preparation

**Title**: `[v1.0] Complete V1.0 launch checklist`

**Labels**: `type: infra`, `priority: P0`, `component: cli`, `phase: v1.0`

**Milestone**: `v1.0 - Production CLI`

**Effort**: 5 story points

**Description**:

## Goal
Final preparations for V1.0 public launch on Hacker News and Product Hunt.

## Pre-Launch Checklist

### Code & Quality
- [ ] All P0 issues closed
- [ ] All tests passing (unit + integration + E2E)
- [ ] Security audit complete (Issue #7)
- [ ] Performance benchmarks meet targets (Issue #11)
- [ ] Code review for all critical modules
- [ ] CHANGELOG.md updated with V1.0 changes

### Distribution
- [ ] Homebrew formula published (Issue #3)
- [ ] crates.io package published (Issue #4)
- [ ] Debian package available (Issue #12)
- [ ] GitHub release with binaries (Issue #10)
- [ ] All binaries tested on target platforms

### Documentation
- [ ] README.md polished (Issue #6)
- [ ] Installation guides complete
- [ ] Usage examples comprehensive
- [ ] Troubleshooting FAQ added
- [ ] Video tutorial recorded and published
- [ ] API docs (rustdoc) published

### Marketing
- [ ] Launch blog post written
- [ ] Hacker News post prepared
- [ ] Product Hunt launch scheduled
- [ ] Twitter/social media posts prepared
- [ ] Demo video uploaded to YouTube
- [ ] Screenshots and GIFs ready

### Monitoring
- [ ] Analytics set up (download counts, GitHub stars)
- [ ] Error tracking configured (Sentry or similar)
- [ ] GitHub Discussions enabled for community
- [ ] Discord server ready (optional)

### Legal & Compliance
- [ ] License files up to date (MIT/Apache 2.0)
- [ ] SECURITY.md policy published
- [ ] Code of Conduct added
- [ ] Contributing guidelines updated

## Launch Day Tasks
- [ ] Bump version to 1.0.0 in Cargo.toml
- [ ] Tag release: `git tag v1.0.0`
- [ ] Push tag: `git push origin v1.0.0`
- [ ] Verify all releases published
- [ ] Post on Hacker News
- [ ] Launch on Product Hunt
- [ ] Share on social media
- [ ] Monitor feedback and respond to issues

## Post-Launch (First 48 Hours)
- [ ] Triage GitHub issues (respond within 24h)
- [ ] Fix critical bugs (P0 only)
- [ ] Engage with community feedback
- [ ] Track metrics (stars, downloads, installations)

## Acceptance Criteria
- [ ] All pre-launch checklist items complete
- [ ] V1.0 release live on all platforms
- [ ] Launch announcements posted
- [ ] Community engagement active

## Related
- Depends on: ALL other V1.0 issues
- This is the final meta-issue for V1.0 launch

---

## Summary

### Priority Breakdown
- **P0 (Critical)**: Issues #1, #2, #3, #4, #5, #7, #15 (7 issues)
- **P1 (High)**: Issues #6, #10, #11, #12, #13 (5 issues)
- **P2 (Medium)**: Issues #8, #9, #14 (3 issues)

### Effort Estimate
- **Total**: 38 story points
- **Timeline**: 6-8 weeks with 2-3 contributors
- **Target Launch**: December 2025 / January 2026

### Good First Issues
- Issue #8: Shell Completion Scripts (2 points)
- Issue #9: Man Pages (1 point)
- Issue #13: Config Validation (2 points)

---

**Copy these issues into GitHub and start tracking progress toward V1.0!**
