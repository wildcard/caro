# cmdai MVP Roadmap
## Spec-Driven Development Path to Production

**Document Version**: 1.0
**Last Updated**: 2025-11-19
**Project**: cmdai - Natural Language to Shell Command CLI
**Current Status**: Pre-Alpha (65% MVP Complete)
**Target MVP Release**: Q1 2026

---

## Executive Summary

This roadmap defines the structured path from the current pre-alpha state to a production-ready MVP, followed by community-driven feature prioritization for post-MVP releases. The project follows **spec-driven development** methodology using GitHub's spec-kit workflow.

### Current State (as of 2025-11-19)
- ✅ **65% MVP Feature Complete**
- ✅ Core architecture implemented (backends, safety, config, CLI)
- ✅ 44/44 unit tests passing, 9/9 integration tests passing
- ⚠️ Model inference is **simulated** - not connected to real ML models
- ⚠️ Download infrastructure incomplete
- ❌ No binary distribution pipeline
- ❌ Branding and final UX polish needed

### MVP Definition

**Minimum Viable Product Criteria**:
1. Single-binary distribution under 50MB (without embedded model)
2. Real LLM inference on local machine (MLX for Apple Silicon, CPU fallback)
3. Automatic model download on first run with progress indication
4. Production-grade safety validation preventing dangerous commands
5. Startup time < 100ms, first inference < 2s on M1 Mac
6. Cross-platform support (macOS, Linux, Windows)
7. Professional branding and polished UX
8. Complete user documentation and installation guides
9. Binary releases for all major platforms
10. Open source community infrastructure (contributing, CoC, templates)

---

## Phase Breakdown

### Phase 0: Foundation Complete ✅
**Status**: DONE (Current State)
**Completion**: ~65%

#### Implemented Components
- ✅ CLI argument parsing with clap
- ✅ Safety validation (52 dangerous patterns)
- ✅ Configuration management (TOML-based)
- ✅ Backend trait system (CommandGenerator)
- ✅ Remote backend support (Ollama, vLLM)
- ✅ Execution context detection
- ✅ Logging infrastructure
- ✅ Cache directory management
- ✅ Comprehensive test suite

#### Known Issues
- 🐛 Arc<dyn CommandGenerator> trait bound compilation error
- 🐛 2 clippy warnings (unused imports)
- ⚠️ Mock/simulation code mixed with production code

---

## 🎯 Phase 1: Real Model Inference (CRITICAL PATH)
**Target**: 2 weeks
**Priority**: CRITICAL - Blocks all other work
**Assignable**: Yes
**Spec Command**: `/specify Feature 006: Real Model Inference Integration`

### Overview
Connect simulated inference to actual neural network models using mlx-rs (Apple Silicon) and candle-transformers (cross-platform CPU). This is the single most critical blocker to MVP.

### Milestones

#### Milestone 1.1: MLX Backend Real Inference
**Spec**: `specs/006-real-mlx-inference.md`
**Files**: `src/backends/embedded/mlx.rs`
**Dependencies**: mlx-rs crate, MLX framework bindings

**Requirements**:
- [ ] Research mlx-rs API and integration patterns
- [ ] Implement GGUF model loading via mlx-rs
- [ ] Create tokenization pipeline (tiktoken or sentencepiece)
- [ ] Wire up actual inference call replacing simulate_inference()
- [ ] Handle generation parameters (temperature, top_p, max_tokens)
- [ ] Add streaming support for long responses
- [ ] Validate <2s inference time on M1 Mac
- [ ] Add comprehensive error handling for model failures
- [ ] Write integration tests with real model

**Acceptance Criteria**:
- Real Qwen2.5-Coder model generates valid shell commands
- JSON extraction works with actual model output
- Performance meets <2s target on Apple Silicon
- Memory usage stays under 4GB
- Graceful fallback when MLX unavailable

**Technical Debt**:
- Remove simulate_inference() mock code
- Add proper model lifecycle management
- Document MLX-specific requirements

#### Milestone 1.2: CPU Backend Real Inference
**Spec**: `specs/007-real-cpu-inference.md`
**Files**: `src/backends/embedded/cpu.rs`
**Dependencies**: candle-core, candle-transformers, tokenizers

**Requirements**:
- [ ] Integrate candle-transformers for GGUF model loading
- [ ] Implement CPU-optimized inference path
- [ ] Add multi-threading support for CPU inference
- [ ] Create unified tokenizer abstraction (shared with MLX)
- [ ] Handle quantization (Q4_K_M, Q8_0 variants)
- [ ] Optimize for <5s inference on modern CPUs
- [ ] Add SIMD optimizations where possible
- [ ] Test on Linux, macOS x86_64, Windows
- [ ] Write cross-platform integration tests

**Acceptance Criteria**:
- CPU backend generates valid commands on non-Apple hardware
- Performance acceptable on 4-core CPU (<5s)
- Binary size increase <20MB
- Works offline after initial model download
- Consistent output with MLX backend

**Technical Debt**:
- Remove CPU simulate_inference() mock
- Add CPU performance benchmarking
- Document CPU requirements (AVX2, etc.)

#### Milestone 1.3: Prompt Engineering & Output Parsing
**Spec**: `specs/008-prompt-optimization.md`
**Files**: `src/backends/embedded/embedded_backend.rs`, `src/backends/prompt.rs`

**Requirements**:
- [ ] Test current prompts with real Qwen2.5-Coder model
- [ ] Optimize system prompt for reliable JSON output
- [ ] Add few-shot examples to prompt template
- [ ] Implement robust JSON extraction (handle markdown, extra text)
- [ ] Tune generation parameters per model variant
- [ ] Add prompt versioning and A/B testing
- [ ] Create prompt regression test suite
- [ ] Document prompt engineering guidelines

**Acceptance Criteria**:
- >95% JSON parse success rate with real model
- Commands are POSIX-compliant and safe
- Handles edge cases (complex queries, ambiguous requests)
- Fallback strategies work when JSON malformed
- Consistent quality across MLX and CPU backends

#### Milestone 1.4: Model Download & Caching
**Spec**: `specs/009-model-download-system.md`
**Files**: `src/cache/mod.rs`, `src/model_loader.rs`

**Requirements**:
- [ ] Implement actual Hugging Face download (currently returns error)
- [ ] Add progress indicators for download (progress bars)
- [ ] Support resume on network interruption
- [ ] Verify SHA256 checksums after download
- [ ] Add model variant selection (Q4_K_M vs Q8_0)
- [ ] Implement cache eviction when disk space low
- [ ] Add offline mode detection and graceful handling
- [ ] Test on slow/unreliable networks

**Acceptance Criteria**:
- First run downloads model automatically with clear progress
- Download resumes correctly after interruption
- Offline usage works after initial download
- Clear error messages for network failures
- Cache integrity validation on startup

**Success Metrics**:
- All inference tests passing with real models
- Performance targets met (MLX <2s, CPU <5s)
- Binary size under target (50MB without model)
- Clean separation of production and test code

---

## 🎨 Phase 2: User Experience & Polish
**Target**: 1.5 weeks
**Priority**: HIGH
**Spec Command**: `/specify Feature 010: Production UX Polish`

### Milestones

#### Milestone 2.1: Interactive First-Run Experience
**Spec**: `specs/010-first-run-ux.md`
**Files**: `src/cli/first_run.rs`, `src/config/setup.rs`

**Requirements**:
- [ ] Detect first run (no config file)
- [ ] Welcome message with ASCII art logo
- [ ] Interactive setup wizard (dialoguer)
  - [ ] Select default backend (auto-detect best)
  - [ ] Choose safety level (strict/moderate/permissive)
  - [ ] Configure model download location
  - [ ] Optional: analytics opt-in
- [ ] Create default config file
- [ ] Trigger model download with progress
- [ ] Run test inference and show sample output
- [ ] Generate welcome documentation

**Acceptance Criteria**:
- First-time users can get started in <3 minutes
- Clear explanation of what's being downloaded and why
- Progress indication doesn't block (can cancel)
- Config file created with sensible defaults
- Help text shows next steps

#### Milestone 2.2: Branding & Visual Identity
**Spec**: `specs/011-branding-identity.md`
**Files**: `assets/`, `README.md`, `src/cli/branding.rs`

**Requirements**:
- [ ] Design cmdai logo (CLI-friendly ASCII + SVG versions)
- [ ] Choose color scheme for terminal output
  - Safe commands: Green
  - Moderate risk: Yellow
  - High risk: Orange
  - Critical risk: Red
- [ ] Create ASCII art banner for --help
- [ ] Design output formatting templates
- [ ] Standardize error message formatting
- [ ] Add emoji support (optional, flag-gated)
- [ ] Create social media graphics (GitHub, Twitter)
- [ ] Write tagline and project description

**Brand Identity**:
- **Tagline**: "Think it. Type it. Trust it." or "Safe shell commands from natural language"
- **Voice**: Professional, safety-conscious, developer-friendly
- **Colors**: Monochrome with semantic safety colors
- **Style**: Minimal, Unix philosophy, terminal-native

**Deliverables**:
- [ ] Logo files (SVG, PNG, ASCII)
- [ ] Brand guidelines document
- [ ] Color palette specification
- [ ] Terminal output style guide

#### Milestone 2.3: Enhanced Error Messages
**Spec**: `specs/012-error-messages.md`
**Files**: `src/error.rs`, `src/cli/error_display.rs`

**Requirements**:
- [ ] Audit all error types in codebase
- [ ] Rewrite error messages for clarity
  - Before: "Backend unavailable"
  - After: "Ollama backend is not responding. Is Ollama running? Try: ollama serve"
- [ ] Add actionable suggestions to each error
- [ ] Include relevant documentation links
- [ ] Add --debug flag for detailed error traces
- [ ] Format errors with colors and structure
- [ ] Test error messages with non-technical users

**Error Message Template**:
```
❌ Error: [One-line summary]

What happened:
  [Detailed explanation]

How to fix:
  1. [Action step 1]
  2. [Action step 2]

Learn more: https://docs.cmdai.dev/errors/[error-code]
```

#### Milestone 2.4: Output Formatting & Presentation
**Spec**: `specs/013-output-formatting.md`
**Files**: `src/cli/output.rs`, `src/cli/formatters/`

**Requirements**:
- [ ] Implement rich command preview display
  ```
  ┌─ Generated Command ─────────────────────────────────
  │ find . -name "*.rs" -type f -exec grep -l "TODO" {} \;
  │
  │ Safety: ✓ Safe  Risk: Low
  │ POSIX: ✓ Compatible
  └─────────────────────────────────────────────────────
  Execute this command? [y/N/e/explain]
  ```
- [ ] Add syntax highlighting for shell commands
- [ ] Create verbose mode with timing information
- [ ] Add JSON/YAML output for scripting
- [ ] Implement quiet mode (command only, no decorations)
- [ ] Add copy-to-clipboard support (where available)
- [ ] Show command history in session

**Output Modes**:
- `--format=pretty` (default): Rich terminal output
- `--format=json`: Machine-readable JSON
- `--format=plain`: Command only, no decoration
- `--format=explain`: Command with explanation

---

## 🚀 Phase 3: Distribution & Packaging
**Target**: 1 week
**Priority**: HIGH
**Spec Command**: `/specify Feature 014: Binary Distribution Pipeline`

### Milestones

#### Milestone 3.1: Cross-Platform Compilation
**Spec**: `specs/014-cross-compilation.md`
**Files**: `.github/workflows/release.yml`, `build/`, `Cargo.toml`

**Requirements**:
- [ ] Setup cross-compilation targets
  - [ ] macOS Apple Silicon (aarch64-apple-darwin)
  - [ ] macOS Intel (x86_64-apple-darwin)
  - [ ] Linux x86_64 (x86_64-unknown-linux-gnu)
  - [ ] Linux ARM (aarch64-unknown-linux-gnu)
  - [ ] Windows x86_64 (x86_64-pc-windows-msvc)
- [ ] Configure feature flags per platform
- [ ] Add code signing for macOS (notarization)
- [ ] Test binaries on each platform
- [ ] Optimize binary size with strip and LTO
- [ ] Create universal macOS binary (lipo)

**Build Targets**:
```
cmdai-v0.1.0-aarch64-apple-darwin.tar.gz      # macOS ARM
cmdai-v0.1.0-x86_64-apple-darwin.tar.gz       # macOS Intel
cmdai-v0.1.0-universal-apple-darwin.tar.gz    # macOS Universal
cmdai-v0.1.0-x86_64-unknown-linux-gnu.tar.gz  # Linux x64
cmdai-v0.1.0-aarch64-unknown-linux-gnu.tar.gz # Linux ARM
cmdai-v0.1.0-x86_64-pc-windows-msvc.zip       # Windows
```

#### Milestone 3.2: Package Manager Integration
**Spec**: `specs/015-package-managers.md`
**Files**: `packaging/homebrew/`, `packaging/apt/`, `packaging/scoop/`

**Requirements**:
- [ ] **Homebrew Formula** (macOS/Linux)
  - Create cmdai.rb formula
  - Submit to homebrew-core or maintain tap
  - Test: `brew install cmdai`

- [ ] **Cargo** (Rust users)
  - Publish to crates.io
  - Test: `cargo install cmdai`

- [ ] **APT/DEB** (Debian/Ubuntu)
  - Create .deb package
  - Setup PPA or provide direct download
  - Test: `apt install cmdai`

- [ ] **AUR** (Arch Linux)
  - Create PKGBUILD
  - Submit to AUR
  - Test: `yay -S cmdai`

- [ ] **Scoop** (Windows)
  - Create scoop manifest
  - Submit to scoop bucket
  - Test: `scoop install cmdai`

- [ ] **Nix** (NixOS/Nix users)
  - Create Nix derivation
  - Submit to nixpkgs
  - Test: `nix-env -iA nixpkgs.cmdai`

**Distribution Channels**:
1. GitHub Releases (primary)
2. Homebrew (macOS/Linux primary)
3. crates.io (Rust developers)
4. Package managers (platform-specific)

#### Milestone 3.3: Installation Documentation
**Spec**: `specs/016-installation-docs.md`
**Files**: `docs/installation.md`, `README.md`

**Requirements**:
- [ ] Write installation guide for each platform
- [ ] Add verification steps (checksum, signature)
- [ ] Document system requirements
- [ ] Create troubleshooting section
- [ ] Add uninstallation instructions
- [ ] Write upgrade guide
- [ ] Add video walkthrough (optional)

**Installation Methods**:
```markdown
# macOS
brew install cmdai

# Linux (Homebrew)
brew install cmdai

# Linux (APT - Ubuntu/Debian)
sudo add-apt-repository ppa:cmdai/stable
sudo apt update && sudo apt install cmdai

# Arch Linux
yay -S cmdai

# Windows
scoop install cmdai

# From source
cargo install cmdai

# Direct download
curl -L https://github.com/wildcard/cmdai/releases/latest/download/cmdai-$(uname -s)-$(uname -m).tar.gz | tar xz
```

#### Milestone 3.4: Auto-Update System
**Spec**: `specs/017-auto-updates.md`
**Files**: `src/update/`, `src/cli/update.rs`

**Requirements**:
- [ ] Check GitHub releases API for new versions
- [ ] Compare semantic versions
- [ ] Show update notification on startup
- [ ] Implement `cmdai update` command
- [ ] Download and verify new binary
- [ ] Replace current binary (platform-specific)
- [ ] Add --check-update flag
- [ ] Respect update check frequency (daily)
- [ ] Add opt-out mechanism

**Update Flow**:
```
$ cmdai "list files"
ℹ️  Update available: v0.2.0 → v0.3.0
   Run 'cmdai update' to upgrade

[normal operation continues]

$ cmdai update
Downloading cmdai v0.3.0... ████████████ 100%
Verifying signature...     ✓
Installing update...       ✓
Update complete! Restart cmdai to use v0.3.0.
```

---

## 📚 Phase 4: Documentation & Community
**Target**: 1 week
**Priority**: HIGH
**Spec Command**: `/specify Feature 018: Comprehensive Documentation`

### Milestones

#### Milestone 4.1: User Documentation
**Spec**: `specs/018-user-documentation.md`
**Files**: `docs/`, `README.md`, `USAGE.md`

**Requirements**:
- [ ] **Getting Started Guide**
  - Installation
  - First run walkthrough
  - Basic usage examples
  - Configuration guide

- [ ] **User Guide**
  - Command generation best practices
  - Safety system explanation
  - Backend selection guide
  - Configuration reference
  - Troubleshooting

- [ ] **FAQ**
  - Common questions
  - Comparison with alternatives
  - Privacy and security
  - Offline usage

- [ ] **Cookbook / Examples**
  - File operations
  - Text processing
  - System administration
  - Git workflows
  - Data analysis

**Documentation Structure**:
```
docs/
├── README.md              # Documentation index
├── getting-started.md     # Quick start guide
├── user-guide/
│   ├── installation.md
│   ├── configuration.md
│   ├── safety.md
│   └── backends.md
├── cookbook/
│   ├── file-operations.md
│   ├── text-processing.md
│   └── examples.md
├── faq.md
├── troubleshooting.md
└── api/                   # For library usage
    └── rust-api.md
```

#### Milestone 4.2: Developer Documentation
**Spec**: `specs/019-developer-docs.md`
**Files**: `CONTRIBUTING.md`, `ARCHITECTURE.md`, `docs/development/`

**Requirements**:
- [ ] **Architecture Guide**
  - System overview
  - Backend trait system
  - Safety validation pipeline
  - Configuration management

- [ ] **Contributing Guide**
  - Development setup
  - Code style guidelines
  - Testing requirements
  - PR process
  - Spec-driven development workflow

- [ ] **API Documentation**
  - Rust API docs (cargo doc)
  - Backend integration guide
  - Plugin system (future)

- [ ] **Testing Guide**
  - Unit testing approach
  - Integration testing
  - Safety validation tests
  - Performance benchmarks

#### Milestone 4.3: Community Infrastructure
**Spec**: `specs/020-community-setup.md`
**Files**: `CODE_OF_CONDUCT.md`, `.github/`, `GOVERNANCE.md`

**Requirements**:
- [ ] **Code of Conduct**
  - Adopt Contributor Covenant
  - Define enforcement process
  - Add contact information

- [ ] **GitHub Templates**
  - Issue templates (bug, feature request, question)
  - PR template with checklist
  - Discussion categories

- [ ] **Contributing Guide**
  - How to report bugs
  - Feature request process
  - Development workflow
  - Spec-driven development intro

- [ ] **Governance Document**
  - Decision-making process
  - Maintainer roles
  - Feature prioritization process
  - Voting mechanism for features

- [ ] **Community Channels**
  - GitHub Discussions
  - Discord server (optional)
  - Twitter/Mastodon presence
  - Blog/website (future)

#### Milestone 4.4: Website & Landing Page
**Spec**: `specs/021-website.md`
**Files**: `website/`, `docs-site/`

**Requirements**:
- [ ] Create landing page (GitHub Pages or custom domain)
  - Hero section with demo
  - Feature highlights
  - Installation CTA
  - Safety messaging

- [ ] Setup documentation site (mdBook or Docusaurus)
  - Searchable docs
  - Versioned documentation
  - Code examples with syntax highlighting
  - Mobile-friendly

- [ ] Add demo/playground
  - Interactive command generator
  - Show safety validation in action
  - No actual execution (safety)

- [ ] Analytics (privacy-respecting)
  - Page views
  - Popular docs
  - Download counts

**Website Structure**:
```
https://cmdai.dev/
├── /                    # Landing page
├── /docs/              # Documentation site
├── /install/           # Installation guide
├── /blog/              # Project updates
└── /community/         # Contributing, CoC
```

---

## 🎯 Phase 5: Testing & Quality Assurance
**Target**: 1 week
**Priority**: HIGH
**Spec Command**: `/specify Feature 022: Production Testing & QA`

### Milestones

#### Milestone 5.1: Integration Testing with Real Models
**Spec**: `specs/022-integration-testing.md`
**Files**: `tests/integration/`, `tests/e2e/`

**Requirements**:
- [ ] End-to-end tests with real model inference
- [ ] Test all backends (MLX, CPU, Ollama, vLLM)
- [ ] Test safety validation with real commands
- [ ] Cross-platform compatibility tests
- [ ] Performance regression tests
- [ ] Memory leak detection
- [ ] Stress testing (rapid successive calls)
- [ ] Offline mode testing

**Test Scenarios**:
1. First run experience (model download)
2. Common file operations
3. Complex multi-step commands
4. Dangerous command detection
5. Edge cases (empty input, very long input)
6. Network failures during download
7. Corrupted model files
8. Low disk space scenarios

#### Milestone 5.2: Performance Benchmarking
**Spec**: `specs/023-performance-benchmarks.md`
**Files**: `benches/`, `scripts/benchmark.sh`

**Requirements**:
- [ ] Create criterion-based benchmarks
- [ ] Measure startup time (target: <100ms)
- [ ] Measure first inference time (MLX: <2s, CPU: <5s)
- [ ] Measure memory usage (target: <4GB)
- [ ] Benchmark safety validation (target: <10ms)
- [ ] Binary size tracking (target: <50MB)
- [ ] Model load time (target: <1s)
- [ ] Compare against baseline
- [ ] Add CI/CD performance gate

**Benchmark Suite**:
```rust
benches/
├── startup.rs          // Binary startup time
├── inference.rs        // Model inference speed
├── safety.rs           // Safety validation performance
├── parsing.rs          // JSON parsing speed
└── end_to_end.rs       // Full command generation flow
```

#### Milestone 5.3: Security Audit
**Spec**: `specs/024-security-audit.md`
**Files**: `SECURITY.md`, `docs/security.md`

**Requirements**:
- [ ] Audit safety validation patterns
- [ ] Test command injection vectors
- [ ] Review dependency vulnerabilities (cargo audit)
- [ ] Test privilege escalation scenarios
- [ ] Review file system operations
- [ ] Audit network security (HTTPS only)
- [ ] Test input sanitization
- [ ] Code signing verification
- [ ] Write security policy document

**Security Checklist**:
- [ ] No command execution without user confirmation
- [ ] All network requests use HTTPS
- [ ] No secrets logged or cached
- [ ] Dependencies audited and up-to-date
- [ ] Binaries signed and verified
- [ ] File operations properly sandboxed
- [ ] Safety patterns cover OWASP top 10

#### Milestone 5.4: Beta Testing Program
**Spec**: `specs/025-beta-program.md`
**Files**: `docs/beta-testing.md`

**Requirements**:
- [ ] Define beta criteria
- [ ] Recruit beta testers (50-100 users)
- [ ] Create feedback collection mechanism
- [ ] Setup telemetry (opt-in)
- [ ] Define success metrics
- [ ] Run 2-week beta period
- [ ] Collect and prioritize feedback
- [ ] Fix critical issues before launch

**Beta Metrics**:
- Installation success rate
- First-run completion rate
- Command generation success rate
- Safety validation effectiveness
- Performance on real hardware
- Bug reports and severity
- User satisfaction (NPS)

---

## 🎉 MVP Release Preparation
**Target**: 1 week
**Priority**: CRITICAL
**Spec Command**: `/specify Feature 026: MVP Launch Preparation`

### Launch Checklist

#### Pre-Launch (T-1 week)
- [ ] All Phase 1-5 milestones complete
- [ ] All tests passing (unit, integration, e2e)
- [ ] Performance benchmarks meet targets
- [ ] Security audit complete
- [ ] Documentation complete
- [ ] Beta testing feedback addressed
- [ ] Release notes written
- [ ] Marketing materials prepared

#### Launch Day (T-0)
- [ ] Tag v1.0.0 release
- [ ] Build and publish binaries
- [ ] Publish to package managers
- [ ] Update documentation site
- [ ] Publish blog post
- [ ] Social media announcement
- [ ] Submit to Show HN, /r/rust, etc.
- [ ] Monitor for critical issues

#### Post-Launch (T+1 week)
- [ ] Monitor crash reports
- [ ] Respond to community feedback
- [ ] Fix critical bugs (hotfix releases)
- [ ] Update documentation based on questions
- [ ] Plan post-MVP features

### Success Metrics
- **Downloads**: 1,000+ in first month
- **GitHub Stars**: 500+ in first month
- **Installation Success**: >95%
- **Crash Rate**: <1%
- **User Retention**: >60% (return after 7 days)
- **Community**: 50+ contributors, 100+ discussions

---

## 🚀 Post-MVP Feature Roadmap
**Community-Driven Prioritization**

The following features are organized into **Themes** for community voting. Each theme contains related features that can be prioritized together.

### Voting Mechanism
- **GitHub Discussions**: Feature voting threads
- **Issue Reactions**: 👍 for priority, 👀 for interest
- **Quarterly Planning**: Top-voted features enter next release cycle
- **Spec-Driven**: All features follow `/specify` → `/plan` → `/tasks` → `/implement` workflow

---

## Theme 1: Advanced Model Support
**Goal**: Expand model ecosystem and customization

### Feature 1.1: Custom Model Support
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/027-custom-models.md`

**Description**:
Allow users to use any GGUF-format model from Hugging Face or local filesystem.

**Requirements**:
- [ ] Model configuration in config.toml
- [ ] Auto-download custom models from HF Hub
- [ ] Support local model paths
- [ ] Model compatibility validation
- [ ] Custom prompt templates per model
- [ ] Performance benchmarking per model

**Use Cases**:
- Use larger models for better quality (Qwen 7B, Mistral)
- Use specialized models (coding-specific, language-specific)
- Use company-internal fine-tuned models
- Experiment with new models without code changes

### Feature 1.2: Model Marketplace
**Priority**: Community Vote
**Effort**: Large (3 weeks)
**Spec**: `specs/028-model-marketplace.md`

**Description**:
Curated marketplace of tested models with ratings and benchmarks.

**Requirements**:
- [ ] Model registry (JSON/TOML manifest)
- [ ] Browse available models in CLI (`cmdai models list`)
- [ ] Download and install models (`cmdai models install qwen-7b`)
- [ ] Community ratings and reviews
- [ ] Performance benchmarks per model
- [ ] Model size and requirements display

**UI**:
```
$ cmdai models list
Available Models:
┌─────────────────────────────────────────────────────────────┐
│ Name            Size   Speed  Quality  Platform   Downloads │
├─────────────────────────────────────────────────────────────┤
│ qwen-1.5b (*)   1.1GB  ⚡⚡⚡   ⭐⭐⭐     All        45.2k     │
│ qwen-7b         5.2GB  ⚡⚡     ⭐⭐⭐⭐⭐   All        12.8k     │
│ codellama-7b    4.8GB  ⚡⚡     ⭐⭐⭐⭐    All        8.3k      │
│ mistral-7b      5.1GB  ⚡⚡     ⭐⭐⭐⭐    All        6.1k      │
└─────────────────────────────────────────────────────────────┘
(*) Currently installed

$ cmdai models install qwen-7b
```

### Feature 1.3: Multi-Model Ensemble
**Priority**: Community Vote
**Effort**: Large (4 weeks)
**Spec**: `specs/029-multi-model.md`

**Description**:
Use multiple models simultaneously for consensus or fallback.

**Requirements**:
- [ ] Configure multiple models in config
- [ ] Consensus mode (run 3 models, use majority)
- [ ] Fallback mode (try model 1, then 2, then 3)
- [ ] Quality voting (pick best response)
- [ ] Performance impact analysis

**Use Cases**:
- Higher confidence in generated commands
- Fallback when primary model fails
- Compare model quality
- Research and benchmarking

---

## Theme 2: Command Execution & Workflows
**Goal**: Make cmdai more powerful for complex tasks

### Feature 2.1: Safe Command Execution
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/030-command-execution.md`

**Description**:
Optionally execute generated commands after user confirmation.

**Requirements**:
- [ ] Execute command in subprocess
- [ ] Stream output to terminal
- [ ] Capture exit code
- [ ] Add to command history
- [ ] Rollback support (where possible)
- [ ] Dry-run mode (show what would happen)

**Safety Features**:
- Triple confirmation for high-risk commands
- Sandbox mode (restrict to specific directories)
- Audit log of all executed commands
- Undo/rollback for destructive operations

**UI**:
```
$ cmdai "delete all .log files older than 30 days" --execute
Generated: find . -name "*.log" -mtime +30 -delete

Safety: ⚠️  High Risk - Irreversible file deletion
Affected: ~47 files, 3.2 MB

Confirm execution? [y/N/preview]
> preview

Files to be deleted:
  ./logs/app-2024-10-15.log
  ./logs/app-2024-10-16.log
  ... (45 more)

Execute? [y/N]
```

### Feature 2.2: Multi-Step Workflows
**Priority**: Community Vote
**Effort**: Large (4 weeks)
**Spec**: `specs/031-workflows.md`

**Description**:
Chain multiple commands into workflows with conditional logic.

**Requirements**:
- [ ] Natural language workflow description
- [ ] Generate multi-step plans
- [ ] Conditional execution (if success, then...)
- [ ] Error handling and retry logic
- [ ] Workflow templates and saving
- [ ] Variable passing between steps

**Example**:
```
$ cmdai workflow "analyze project: count lines of code, run tests, generate coverage report"

Generated Workflow:
1. Count lines: find src -name "*.rs" | xargs wc -l
2. Run tests: cargo test --all
3. Coverage: cargo tarpaulin --out Html

Execute workflow? [y/N/edit]
```

### Feature 2.3: Interactive Mode
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/032-interactive-mode.md`

**Description**:
REPL-style interface for iterative command refinement.

**Requirements**:
- [ ] Start interactive session (`cmdai -i`)
- [ ] Multi-turn conversations
- [ ] Command history and editing
- [ ] Context awareness (previous commands)
- [ ] Refine/modify last command
- [ ] Explain command behavior

**UI**:
```
$ cmdai -i
cmdai> find all rust files
> find . -name "*.rs" -type f

cmdai> exclude the target directory
> find . -name "*.rs" -type f -not -path "*/target/*"

cmdai> sort by modification time
> find . -name "*.rs" -type f -not -path "*/target/*" -printf "%T@ %p\n" | sort -n

cmdai> execute
Executing...
```

---

## Theme 3: Integration & Ecosystem
**Goal**: Integrate cmdai with other tools and platforms

### Feature 3.1: Shell Integration
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/033-shell-integration.md`

**Description**:
Deep integration with bash/zsh/fish shells.

**Requirements**:
- [ ] Shell completions (bash, zsh, fish)
- [ ] Shell functions/aliases
- [ ] Keyboard shortcuts (Ctrl+A for "ask cmdai")
- [ ] History integration
- [ ] Inline suggestions (like GitHub Copilot for CLI)

**Installation**:
```bash
# Add to ~/.bashrc
eval "$(cmdai shell-init bash)"

# Then in terminal:
$ <Ctrl+A>
Ask cmdai: find large files
> find . -type f -size +100M
<Ctrl+E to execute or Ctrl+C to cancel>
```

### Feature 3.2: IDE/Editor Plugins
**Priority**: Community Vote
**Effort**: Large (3 weeks per plugin)
**Spec**: `specs/034-editor-plugins.md`

**Description**:
Plugins for VSCode, Vim, Emacs, Sublime Text.

**Requirements**:
- [ ] VSCode extension
- [ ] Vim plugin
- [ ] Emacs package
- [ ] IntelliJ plugin
- [ ] Generate commands from comments
- [ ] Explain existing commands
- [ ] Command palette integration

**VSCode Example**:
```typescript
// Right-click in terminal
// "Ask cmdai: list all TODO comments"
// Inserts: grep -rn "TODO" src/
```

### Feature 3.3: CI/CD Integration
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/035-cicd-integration.md`

**Description**:
Use cmdai in CI/CD pipelines for dynamic command generation.

**Requirements**:
- [ ] GitHub Actions integration
- [ ] GitLab CI integration
- [ ] Jenkins plugin
- [ ] Environment variable configuration
- [ ] Non-interactive mode
- [ ] Audit logging for compliance

**GitHub Action Example**:
```yaml
- name: Generate deployment command
  uses: cmdai/action@v1
  with:
    query: "deploy to production with zero downtime"
    safety: strict
    execute: true
```

### Feature 3.4: API Server Mode
**Priority**: Community Vote
**Effort**: Large (3 weeks)
**Spec**: `specs/036-api-server.md`

**Description**:
Run cmdai as HTTP API server for integration with web apps.

**Requirements**:
- [ ] HTTP server mode (`cmdai serve`)
- [ ] REST API for command generation
- [ ] Authentication (API keys)
- [ ] Rate limiting
- [ ] Webhook support
- [ ] OpenAPI specification

**API**:
```bash
POST /api/v1/generate
{
  "query": "find all rust files",
  "safety_level": "strict"
}

Response:
{
  "command": "find . -name \"*.rs\" -type f",
  "safety": {
    "risk": "safe",
    "patterns_matched": []
  },
  "explanation": "Searches recursively..."
}
```

---

## Theme 4: AI/LLM Enhancements
**Goal**: Advanced AI features and capabilities

### Feature 4.1: Command Explanation
**Priority**: Community Vote
**Effort**: Small (1 week)
**Spec**: `specs/037-command-explain.md`

**Description**:
Reverse operation: explain what an existing command does.

**Requirements**:
- [ ] Parse complex shell commands
- [ ] Generate natural language explanations
- [ ] Explain each part of pipeline
- [ ] Highlight potential issues
- [ ] Suggest improvements

**UI**:
```
$ cmdai explain 'find . -name "*.rs" | xargs grep -l "TODO"'

This command:
1. find . -name "*.rs"
   → Searches recursively for all Rust files

2. | xargs grep -l "TODO"
   → Passes files to grep, lists files containing "TODO"

⚠️  Note: This may fail on filenames with spaces.
💡 Safer: find . -name "*.rs" -exec grep -l "TODO" {} +
```

### Feature 4.2: Command Optimization
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/038-command-optimize.md`

**Description**:
Analyze and optimize existing commands for performance/safety.

**Requirements**:
- [ ] Identify inefficient patterns
- [ ] Suggest optimizations
- [ ] Benchmark before/after
- [ ] Explain trade-offs
- [ ] Apply optimizations automatically

**Example**:
```
$ cmdai optimize 'cat file.txt | grep "error" | wc -l'

Original: cat file.txt | grep "error" | wc -l
Optimized: grep -c "error" file.txt

Improvements:
  ⚡ 3x faster (avoids useless cat)
  💾 Uses less memory (no pipeline)
  ✓ More readable

Apply optimization? [y/N]
```

### Feature 4.3: Context-Aware Suggestions
**Priority**: Community Vote
**Effort**: Large (4 weeks)
**Spec**: `specs/039-context-aware.md`

**Description**:
Use repository context to generate better commands.

**Requirements**:
- [ ] Analyze git repository structure
- [ ] Detect project type (Rust, Python, Node.js, etc.)
- [ ] Read relevant files (.gitignore, Makefile, etc.)
- [ ] Understand build system
- [ ] Suggest project-specific commands

**Example**:
```
$ cmdai "run tests for changed files"

Detected: Rust project with cargo
Changed files: src/main.rs, src/lib.rs

Generated: cargo test --lib && cargo test --bin cmdai
```

### Feature 4.4: Learning from Feedback
**Priority**: Community Vote
**Effort**: Large (5 weeks)
**Spec**: `specs/040-feedback-learning.md`

**Description**:
Learn from user corrections and preferences.

**Requirements**:
- [ ] Track accepted/rejected commands
- [ ] Store user corrections
- [ ] Build user preference profile
- [ ] Fine-tune prompts based on history
- [ ] Privacy-preserving learning (local only)
- [ ] Export/import preferences

**Workflow**:
```
$ cmdai "list files"
> ls -la

User edits: ls -lh
cmdai learns: User prefers human-readable sizes

$ cmdai "show directory size"
> du -sh *  (automatically includes -h flag)
```

---

## Theme 5: Platform & Language Support
**Goal**: Expand platform and language coverage

### Feature 5.1: Windows PowerShell Support
**Priority**: Community Vote
**Effort**: Medium (3 weeks)
**Spec**: `specs/041-powershell.md`

**Description**:
Generate PowerShell commands for Windows users.

**Requirements**:
- [ ] Detect PowerShell environment
- [ ] PowerShell-specific command generation
- [ ] PowerShell safety patterns
- [ ] cmdlet usage (Get-ChildItem vs ls)
- [ ] Windows-specific operations

**Example**:
```
PS> cmdai "find large files"
> Get-ChildItem -Recurse | Where-Object {$_.Length -gt 100MB}
```

### Feature 5.2: Multi-Language Support
**Priority**: Community Vote
**Effort**: Large (2 weeks per language)
**Spec**: `specs/042-i18n.md`

**Description**:
Support queries in multiple languages.

**Requirements**:
- [ ] Language detection
- [ ] Translate query to English
- [ ] Support major languages (ES, FR, DE, ZH, JA)
- [ ] Localized error messages
- [ ] RTL language support

**Example**:
```
$ cmdai "查找所有Rust文件"  (Chinese)
> find . -name "*.rs" -type f
```

### Feature 5.3: Platform-Specific Optimizations
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/043-platform-opts.md`

**Description**:
Optimize for each platform's unique features.

**Requirements**:
- [ ] macOS: Use mdutil, mdfind, osascript
- [ ] Linux: Use systemd, apt, specific distro tools
- [ ] Windows: Use WSL detection, Windows-native tools
- [ ] Platform-specific patterns in prompt

**Example**:
```
# macOS
$ cmdai "find files modified today"
> mdfind 'kMDItemFSContentChangeDate >= $time.today'

# Linux
$ cmdai "find files modified today"
> find . -type f -mtime 0
```

---

## Theme 6: Enterprise & Security
**Goal**: Features for enterprise and security-conscious users

### Feature 6.1: Audit Logging
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/044-audit-logging.md`

**Description**:
Comprehensive audit trail for compliance.

**Requirements**:
- [ ] Log all command generations
- [ ] Log safety violations
- [ ] Log executed commands
- [ ] Structured log format (JSON)
- [ ] Log rotation and retention
- [ ] Export to SIEM systems
- [ ] Tamper-proof logs (signing)

**Log Format**:
```json
{
  "timestamp": "2025-01-15T10:30:45Z",
  "user": "alice",
  "query": "delete old logs",
  "command": "find /var/log -name '*.log' -mtime +30 -delete",
  "safety_risk": "high",
  "executed": true,
  "exit_code": 0
}
```

### Feature 6.2: Policy Enforcement
**Priority**: Community Vote
**Effort**: Large (3 weeks)
**Spec**: `specs/045-policy-enforcement.md`

**Description**:
Organizational policies for allowed/blocked operations.

**Requirements**:
- [ ] Policy file format (YAML/TOML)
- [ ] Whitelist/blacklist commands
- [ ] Path restrictions (no /etc, /var)
- [ ] User/group-based policies
- [ ] Approval workflows
- [ ] Policy violation reporting

**Policy Example**:
```yaml
policies:
  - name: no-system-paths
    deny:
      - paths: ["/etc", "/usr", "/var"]

  - name: require-approval
    high_risk: true
    approvers: ["admin@company.com"]

  - name: allowed-operations
    allow_only:
      - file_operations
      - git_operations
    deny:
      - network_operations
      - system_modifications
```

### Feature 6.3: SSO/LDAP Integration
**Priority**: Community Vote
**Effort**: Large (4 weeks)
**Spec**: `specs/046-sso-integration.md`

**Description**:
Enterprise authentication integration.

**Requirements**:
- [ ] LDAP/Active Directory integration
- [ ] SSO support (SAML, OAuth)
- [ ] Role-based access control
- [ ] Group-based policies
- [ ] Centralized user management

### Feature 6.4: Air-Gapped Deployment
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/047-airgapped.md`

**Description**:
Run completely offline in secure environments.

**Requirements**:
- [ ] Bundle model with binary
- [ ] No network calls
- [ ] Local-only operation
- [ ] Offline license verification
- [ ] Secure update mechanism (manual)

---

## Theme 7: Developer Experience
**Goal**: Improve developer productivity and debugging

### Feature 7.1: Plugin System
**Priority**: Community Vote
**Effort**: Large (4 weeks)
**Spec**: `specs/048-plugin-system.md`

**Description**:
Allow community plugins for custom functionality.

**Requirements**:
- [ ] Plugin API (Rust traits)
- [ ] WASM plugin support
- [ ] Plugin discovery and installation
- [ ] Plugin sandboxing
- [ ] Plugin marketplace
- [ ] Hot reloading during development

**Plugin Types**:
- Custom backends
- Safety validators
- Output formatters
- Command transformers
- Context providers

### Feature 7.2: Testing Framework
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/049-testing-framework.md`

**Description**:
Test command generation without execution.

**Requirements**:
- [ ] Test DSL for command assertions
- [ ] Mock environment setup
- [ ] Snapshot testing
- [ ] Property-based testing
- [ ] Regression test suite

**Example**:
```rust
#[test]
fn test_find_files() {
    assert_command!(
        query: "find rust files",
        generates: "find . -name \"*.rs\" -type f",
        safety: Safe,
        posix: true
    );
}
```

### Feature 7.3: Performance Profiling
**Priority**: Community Vote
**Effort**: Small (1 week)
**Spec**: `specs/050-profiling.md`

**Description**:
Built-in performance profiling and diagnostics.

**Requirements**:
- [ ] `--profile` flag for detailed timing
- [ ] Flamegraph generation
- [ ] Memory profiling
- [ ] Trace export (Chrome tracing format)
- [ ] Performance comparison mode

**Output**:
```
$ cmdai "find files" --profile

Performance Profile:
  Startup:           45ms
  Model load:        823ms
  Tokenization:      12ms
  Inference:         1,234ms
  JSON parsing:      3ms
  Safety validation: 8ms
  Total:             2,125ms

Memory:
  Peak RSS:          1.2GB
  Model:             0.9GB
  Application:       0.3GB
```

---

## Theme 8: Education & Onboarding
**Goal**: Help users learn shell commands

### Feature 8.1: Learning Mode
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/051-learning-mode.md`

**Description**:
Educational mode that explains shell concepts.

**Requirements**:
- [ ] Enable with `--learn` flag
- [ ] Explain each part of command
- [ ] Show alternatives
- [ ] Link to man pages
- [ ] Suggest related commands
- [ ] Quiz mode to test understanding

**UI**:
```
$ cmdai "find files" --learn

Generated: find . -name "*.rs" -type f

Let's break this down:

1. `find` - Searches for files and directories
   📖 Learn more: man find

2. `.` - Current directory (starting point)
   💡 Tip: Use /path/to/dir for other locations

3. `-name "*.rs"` - Match filename pattern
   ⚠️  Note: Quotes prevent shell expansion
   🔗 Related: -iname (case-insensitive)

4. `-type f` - Only files (not directories)
   🔗 Related: -type d (directories only)

Want to see more examples? [Y/n]
```

### Feature 8.2: Command History & Analytics
**Priority**: Community Vote
**Effort**: Small (1 week)
**Spec**: `specs/052-history-analytics.md`

**Description**:
Track and analyze command usage patterns.

**Requirements**:
- [ ] Store command history locally
- [ ] `cmdai history` to view past queries
- [ ] `cmdai stats` for usage analytics
- [ ] Most common commands
- [ ] Learning progress tracking
- [ ] Privacy controls

**Analytics**:
```
$ cmdai stats

Your cmdai Usage:
  Total queries:     347
  Most common:       file operations (42%)
  Safety blocks:     3 (0.8%)
  Avg response time: 1.8s

Top 5 Commands:
  1. find files (78 times)
  2. grep text (45 times)
  3. git operations (34 times)
  4. disk usage (23 times)
  5. process management (19 times)
```

### Feature 8.3: Guided Tutorials
**Priority**: Community Vote
**Effort**: Medium (2 weeks)
**Spec**: `specs/053-tutorials.md`

**Description**:
Interactive tutorials for common tasks.

**Requirements**:
- [ ] `cmdai tutorial list`
- [ ] Step-by-step guides
- [ ] Progress tracking
- [ ] Hands-on exercises
- [ ] Achievement system

**Tutorials**:
- File operations basics
- Text processing with grep/sed/awk
- Git workflows
- System administration
- Data analysis with CLI tools

---

## Feature Prioritization Framework

### Voting System

**GitHub Discussions Structure**:
```
discussions/
├── Feature Requests
│   ├── Theme 1: Advanced Model Support
│   ├── Theme 2: Command Execution & Workflows
│   ├── Theme 3: Integration & Ecosystem
│   ├── Theme 4: AI/LLM Enhancements
│   ├── Theme 5: Platform & Language Support
│   ├── Theme 6: Enterprise & Security
│   ├── Theme 7: Developer Experience
│   └── Theme 8: Education & Onboarding
├── Roadmap Planning
└── Release Discussions
```

**Voting Mechanism**:
1. Each feature has a Discussion thread
2. Use reactions: 👍 (want this), 👎 (don't want), 👀 (interested)
3. Comments for use cases and requirements
4. Quarterly voting rounds
5. Top 3-5 features enter next release cycle

### Prioritization Criteria

**Impact Score** (1-10):
- User value: How many users benefit?
- Use case coverage: Does it unlock new workflows?
- Competitive advantage: Does it differentiate cmdai?

**Effort Score** (1-10):
- Development time: Weeks of work
- Complexity: Technical difficulty
- Risk: Potential for bugs/regressions

**Priority Formula**:
```
Priority = (Impact Score) / (Effort Score)
```

**Examples**:
- Command Explanation: 8 impact / 2 effort = **4.0 priority** ⭐⭐⭐⭐
- Multi-Model Ensemble: 6 impact / 8 effort = **0.75 priority** ⭐
- Shell Integration: 9 impact / 4 effort = **2.25 priority** ⭐⭐⭐

### Release Cadence

**Versioning**: Semantic Versioning (semver)
- Major (1.0, 2.0): Breaking changes
- Minor (1.1, 1.2): New features
- Patch (1.1.1): Bug fixes

**Release Schedule**:
- **Patch releases**: As needed (hotfixes)
- **Minor releases**: Every 6-8 weeks
- **Major releases**: Yearly

**Feature Freeze**: 2 weeks before release
**Beta Period**: 1 week before release

---

## Success Metrics & KPIs

### Adoption Metrics
- **Downloads**: Track via GitHub releases, package managers
- **Active Users**: Telemetry (opt-in)
- **Retention**: 7-day, 30-day return rate

### Quality Metrics
- **Crash Rate**: <1% of sessions
- **Command Success Rate**: >90% (user accepts command)
- **Safety Accuracy**: >99% (no dangerous commands accepted)
- **Performance**: Meet SLO targets (startup, inference)

### Community Metrics
- **Contributors**: Number of unique contributors
- **PRs/Issues**: Activity level
- **Response Time**: Time to first response on issues
- **Documentation**: Page views, search success rate

### Business Metrics (if applicable)
- **Sponsorships**: GitHub sponsors, Patreon
- **Enterprise Adoption**: Companies using cmdai
- **Ecosystem**: Plugins, integrations, third-party tools

---

## Appendix A: Spec-Driven Development Workflow

### Overview
All features follow the spec-driven development process defined in `.claude/commands/`.

### Workflow Steps

1. **Feature Specification** (`/specify`)
   ```bash
   /specify Feature 027: Custom Model Support
   ```
   - Creates `specs/027-custom-models.md`
   - Defines requirements, acceptance criteria, success metrics
   - Reviews with stakeholders

2. **Implementation Planning** (`/plan`)
   ```bash
   /plan specs/027-custom-models.md
   ```
   - Creates `plan.md` with technical design
   - Architecture decisions
   - API design
   - Testing strategy

3. **Task Generation** (`/tasks`)
   ```bash
   /tasks specs/027-custom-models.md
   ```
   - Creates `tasks.md` with ordered implementation tasks
   - Dependencies and prerequisites
   - Estimated effort per task

4. **Implementation** (`/implement`)
   ```bash
   /implement tasks.md
   ```
   - Execute tasks in order
   - TDD approach (tests first)
   - Continuous integration

5. **Validation** (`/analyze`)
   ```bash
   /analyze
   ```
   - Cross-artifact consistency check
   - Quality assurance
   - Documentation review

### Clarification Process
Use `/clarify` to identify underspecified areas:
```bash
/clarify specs/027-custom-models.md
```

### Constitution Alignment
Use `/constitution` to ensure alignment with project principles:
```bash
/constitution
```

---

## Appendix B: Community Contribution Guide

### How to Contribute Features

1. **Propose**: Create a Discussion in "Feature Requests"
2. **Vote**: Community votes with reactions
3. **Spec**: If prioritized, create spec with `/specify`
4. **Review**: Maintainers review and approve spec
5. **Implement**: Follow spec-driven development workflow
6. **Submit**: Create PR with implementation
7. **Release**: Included in next minor release

### Feature Proposal Template
```markdown
## Feature: [Name]

**Theme**: [Which theme from roadmap]
**Priority**: [Your assessment: Low/Medium/High]
**Effort**: [Your estimate: Small/Medium/Large]

### Problem
[What problem does this solve?]

### Solution
[Proposed solution]

### Use Cases
[Who benefits and how?]

### Alternatives Considered
[What else did you consider?]

### Open Questions
[What needs clarification?]
```

---

## Appendix C: Release Checklist Template

### Pre-Release (T-2 weeks)
- [ ] Feature freeze
- [ ] All tests passing
- [ ] Performance benchmarks meet SLOs
- [ ] Security audit complete
- [ ] Documentation updated
- [ ] Changelog prepared
- [ ] Migration guide (if breaking changes)

### Beta Release (T-1 week)
- [ ] Tag beta version (v1.1.0-beta.1)
- [ ] Publish beta binaries
- [ ] Announce in Discussions
- [ ] Monitor for critical issues
- [ ] Fix P0 bugs

### Release Day (T-0)
- [ ] Tag final version (v1.1.0)
- [ ] Build and sign binaries
- [ ] Publish to GitHub releases
- [ ] Update package managers
- [ ] Deploy documentation site
- [ ] Publish blog post
- [ ] Social media announcement
- [ ] Update roadmap

### Post-Release (T+1 week)
- [ ] Monitor crash reports
- [ ] Respond to issues
- [ ] Gather feedback
- [ ] Plan hotfixes if needed
- [ ] Start next release planning

---

## Appendix D: Technical Debt Register

### Current Technical Debt
1. **Mock inference code** - Priority: CRITICAL
   - Remove simulation code in MLX/CPU backends
   - Effort: Covered in Phase 1

2. **Arc<dyn CommandGenerator> trait bound** - Priority: HIGH
   - Fix compilation error
   - Effort: 1 day

3. **Clippy warnings** - Priority: MEDIUM
   - Fix unused imports
   - Effort: 30 minutes

4. **Download implementation** - Priority: HIGH
   - Implement actual HF Hub downloads
   - Effort: Covered in Phase 1

### Future Technical Debt
- Plugin system architecture (before v2.0)
- Deprecate old config format (after 6 months)
- Refactor backend trait (breaking change, v2.0)

---

## Conclusion

This roadmap provides a clear, community-driven path from the current pre-alpha state to a production-ready MVP and beyond. The spec-driven development methodology ensures all features are well-defined, properly planned, and thoroughly tested before release.

**Next Steps**:
1. Complete Phase 1-5 for MVP (est. 6-7 weeks)
2. Launch MVP v1.0.0
3. Gather community feedback
4. Hold first quarterly feature voting round
5. Begin post-MVP feature development

**Get Involved**:
- **Vote**: React to feature proposals in Discussions
- **Contribute**: Pick a feature and follow the spec-driven workflow
- **Feedback**: Share your use cases and requirements
- **Spread**: Star the repo, share with your network

Let's build the future of command-line interfaces together! 🚀
