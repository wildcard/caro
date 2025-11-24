# MVP to V1.0 Transition Guide

> **Completing the production-ready CLI before launching cloud features**

This guide helps contributors understand what needs to be done to complete V1.0 (production-ready CLI) before we begin V2.0 (cloud features).

---

## Current Status

**What's Done** ✅:
- Core CLI structure with clap
- Backend trait system
- Embedded models (MLX + CPU)
- Remote backends (Ollama, vLLM)
- Safety validation
- Configuration management
- Contract-based testing
- Feature 004 complete

**What's Remaining** ⚠️:
- Performance optimization
- Binary size reduction
- Package distribution
- End-to-end integration testing
- Documentation polish
- Security audit

---

## V1.0 Definition of Done

V1.0 is ready when:

1. ✅ **All tests passing** (unit + integration + contract)
2. ✅ **Binary size <50MB** (release build)
3. ✅ **Startup time <100ms** (cold start)
4. ✅ **Inference time <2s** (on M1 Mac with MLX)
5. ✅ **Safety validation >90% accuracy** (on benchmark dataset)
6. ✅ **Package managers** (Homebrew formula working)
7. ✅ **Documentation** (README, quickstart, troubleshooting)
8. ✅ **Security audit** (dependencies, no known CVEs)

---

## Remaining Work (Priority Order)

### P0: Critical for Launch

#### 1. Performance Optimization (1-2 weeks)

**Goal**: Meet performance targets

**Tasks**:
- [ ] Benchmark current performance (startup, inference)
- [ ] Profile with `cargo flamegraph`
- [ ] Optimize hot paths identified in profiling
- [ ] Lazy-load dependencies (defer until needed)
- [ ] Reduce allocations in command generation loop
- [ ] Cache compiled regexes in safety validator

**Acceptance Criteria**:
- Startup <100ms (debug), <50ms (release)
- MLX inference <2s
- CPU inference <5s

**Owner**: Performance team
**GitHub Issue**: #[TBD]

---

#### 2. Binary Size Reduction (1 week)

**Goal**: Single binary <50MB

**Current Status**: ~70MB (needs reduction)

**Tasks**:
- [ ] Enable LTO (link-time optimization) ✅ (already in Cargo.toml)
- [ ] Strip symbols ✅ (already in Cargo.toml)
- [ ] Analyze binary with `cargo bloat`
- [ ] Remove unused dependencies
- [ ] Feature-gate large dependencies (MLX, Candle)
- [ ] Use dynamic linking for system libraries (openssl)
- [ ] Consider UPX compression (optional)

**Acceptance Criteria**:
- Release binary <50MB
- All features still working
- No significant performance regression

**Owner**: Build team
**GitHub Issue**: #[TBD]

---

#### 3. Package Distribution (1 week)

**Goal**: Easy installation on macOS, Linux, Windows

**Tasks**:
- [ ] Create Homebrew formula (tap: `wildcard/cmdai`)
  ```bash
  brew tap wildcard/cmdai
  brew install cmdai
  ```
- [ ] Publish to crates.io
  ```bash
  cargo install cmdai
  ```
- [ ] Create Debian package (.deb)
  ```bash
  sudo dpkg -i cmdai_1.0.0_amd64.deb
  ```
- [ ] Create RPM package (.rpm)
- [ ] Windows installer (.msi or Scoop manifest)
- [ ] GitHub Releases with precompiled binaries
- [ ] Auto-update mechanism (optional for V1.0)

**Acceptance Criteria**:
- One-command install on all platforms
- Signed binaries (macOS notarization)
- Installation docs updated

**Owner**: Release team
**GitHub Issue**: #[TBD]

---

### P1: High Priority

#### 4. Integration Testing (1 week)

**Goal**: E2E tests covering all user workflows

**Tasks**:
- [ ] End-to-end test: Install → First command → Execution
- [ ] Cross-platform tests (macOS, Linux, Windows)
- [ ] Backend fallback scenarios (Ollama down → MLX fallback)
- [ ] Safety validation scenarios (dangerous commands blocked)
- [ ] Configuration scenarios (custom config file)
- [ ] Error handling scenarios (invalid prompts, network errors)

**Test Framework**:
```rust
// tests/e2e/user_workflows.rs
#[test]
fn test_first_time_user_workflow() {
    // 1. Install cmdai (simulated)
    // 2. Run first command: cmdai "list files"
    // 3. Verify output
    // 4. Execute command
    // 5. Verify execution
}
```

**Acceptance Criteria**:
- 10+ E2E scenarios covered
- All tests passing on CI
- Cross-platform compatibility verified

**Owner**: QA team
**GitHub Issue**: #[TBD]

---

#### 5. Documentation Polish (1 week)

**Goal**: Clear, complete docs for new users

**Tasks**:
- [ ] Update README.md
  - Clear "Quick Start" section
  - Installation instructions
  - Usage examples
  - Troubleshooting
- [ ] Create INSTALLATION.md
  - Platform-specific instructions
  - Dependency requirements
  - Verification steps
- [ ] Create USAGE_GUIDE.md
  - Basic usage
  - Advanced features (custom backends, config)
  - Safety configuration
  - Troubleshooting common issues
- [ ] Create VIDEO_TUTORIAL.md (or YouTube video)
  - 2-minute demo
  - Installation → First command → Execution
- [ ] API documentation (rustdoc)
  - All public APIs documented
  - Examples for common use cases

**Acceptance Criteria**:
- New user can install and use cmdai in <5 minutes
- All common questions answered in docs
- rustdoc coverage >80%

**Owner**: Documentation team
**GitHub Issue**: #[TBD]

---

#### 6. Security Audit (1 week)

**Goal**: No known security vulnerabilities

**Tasks**:
- [ ] Run `cargo audit` → Fix all issues
- [ ] Run `cargo deny check` → Fix license issues
- [ ] Review dependency tree for suspicious packages
- [ ] Static analysis with `cargo clippy -- -W clippy::all`
- [ ] Fuzzing for safety validator (optional but recommended)
  ```bash
  cargo fuzz run safety_validator
  ```
- [ ] Review command execution for injection vulnerabilities
- [ ] Secrets management (ensure no API keys logged)
- [ ] File permissions hardening (config files should be 0600)

**Acceptance Criteria**:
- Zero high/critical vulnerabilities
- No GPL/AGPL dependencies (incompatible with MIT)
- Security policy documented (SECURITY.md)

**Owner**: Security team
**GitHub Issue**: #[TBD]

---

### P2: Nice to Have (Optional for V1.0)

#### 7. Shell Completions (3 days)

**Goal**: Tab completion for all shells

**Tasks**:
- [ ] Bash completion
- [ ] Zsh completion
- [ ] Fish completion
- [ ] PowerShell completion (Windows)

**Generated with clap**:
```rust
use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

fn main() {
    let mut cmd = Cli::command();
    generate_to(Shell::Bash, &mut cmd, "cmdai", "completions/");
    generate_to(Shell::Zsh, &mut cmd, "cmdai", "completions/");
    generate_to(Shell::Fish, &mut cmd, "cmdai", "completions/");
}
```

**Owner**: CLI team
**GitHub Issue**: #[TBD]

---

#### 8. Man Pages (2 days)

**Goal**: Offline documentation via `man cmdai`

**Tasks**:
- [ ] Generate man pages from clap CLI
- [ ] Package with binary distribution
- [ ] Install to `/usr/local/share/man/man1/cmdai.1`

**Owner**: Documentation team
**GitHub Issue**: #[TBD]

---

## Timeline

**Total estimated time**: 6-8 weeks (with 2-3 contributors)

```
Week 1-2: Performance Optimization
Week 2-3: Binary Size Reduction + Package Distribution
Week 4-5: Integration Testing
Week 5-6: Documentation Polish
Week 6-7: Security Audit
Week 7-8: Final testing + launch prep
```

**Target Launch**: December 2025 / January 2026

---

## Launch Checklist

When everything above is done:

- [ ] All P0 and P1 tasks complete
- [ ] Version bumped to 1.0.0 in Cargo.toml
- [ ] CHANGELOG.md updated with V1.0 features
- [ ] GitHub Release created (with binaries)
- [ ] Homebrew formula published
- [ ] crates.io package published
- [ ] README.md badges updated (version, build status)
- [ ] Launch blog post written
- [ ] Hacker News post prepared
- [ ] Product Hunt launch scheduled
- [ ] Twitter/social media posts prepared

---

## Post-Launch (V1.0 → V2.0 Transition)

After V1.0 launches successfully:

1. **Monitor metrics** (GitHub stars, downloads, issues)
2. **Collect feedback** (GitHub issues, discussions, social media)
3. **Fix critical bugs** (P0 fixes only, no new features)
4. **Plan V2.0** (cloud backend)
   - Start with architecture design (ARCHITECTURE.md)
   - Create V2.0 milestone in GitHub
   - Begin prototyping cloud API

**No V2.0 work until V1.0 is stable** (at least 2 weeks post-launch)

---

## How to Contribute

1. **Pick a task** from the list above
2. **Comment on the GitHub issue** (will be created soon)
3. **Follow TDD workflow** (see TDD-WORKFLOW.md)
4. **Submit PR** when done
5. **Help review others' PRs**

---

## Questions?

- **General**: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- **Bugs**: [GitHub Issues](https://github.com/wildcard/cmdai/issues)
- **V1.0 progress**: Check [V1.0 Milestone](https://github.com/wildcard/cmdai/milestone/1)

---

**Let's ship V1.0! 🚀**
