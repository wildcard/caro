# Immediate Next Steps - cmdai Production Launch

**Target**: Production-ready v1.0 in 6-8 weeks
**Current Status**: 80% complete, 4 critical items remaining

---

## ⚡ THIS WEEK: Critical Blockers (Week 1)

### 🔴 Priority 1: Fix Contract Test Alignment
**Blocking**: CI/CD pipeline
**Effort**: 4-8 hours
**Owner**: Unassigned

**Quick Start**:
```bash
cd /home/user/cmdai
. "$HOME/.cargo/env"

# Identify compilation errors
cargo test --lib 2>&1 | tee test-errors.log

# Focus files:
# - tests/contract/config_contract.rs
# - tests/contract/logging_contract.rs
# - src/config/mod.rs
# - src/logging/mod.rs
```

**Goal**: All 44+ tests passing without compilation errors

**Acceptance**:
- [ ] `cargo test --all-features` succeeds
- [ ] No API breaking changes
- [ ] Update CHANGELOG.md with fixes

---

### 🔴 Priority 2: Implement HF Model Download
**Blocking**: Offline-first capability
**Effort**: 16-24 hours
**Owner**: Unassigned

**Implementation Plan**:

#### Step 1: Create HF Downloader Module (6-8 hours)
```bash
# Create new module
touch src/cache/hf_download.rs

# Add to src/cache/mod.rs:
# pub mod hf_download;
```

**Core API**:
```rust
// src/cache/hf_download.rs
pub struct HfDownloader {
    client: reqwest::Client,
    cache_dir: PathBuf,
}

impl HfDownloader {
    pub async fn download_model(
        &self,
        repo: &str,
        filename: &str
    ) -> Result<PathBuf> {
        // Implementation with progress bar
    }

    pub async fn resume_download(
        &self,
        partial_path: &PathBuf
    ) -> Result<PathBuf> {
        // Support Range header for resume
    }

    fn verify_checksum(
        &self,
        path: &PathBuf,
        expected: &str
    ) -> Result<bool> {
        // SHA256 validation
    }
}
```

#### Step 2: Integrate with Embedded Backend (4-6 hours)
```rust
// src/backends/embedded/mod.rs
pub async fn ensure_model_available(&self) -> Result<PathBuf> {
    let model_path = self.cache_dir.join("qwen2.5-coder-1.5b-q4km.gguf");

    if !model_path.exists() {
        println!("📦 Downloading Qwen2.5-Coder model (~1.1GB)...");
        let downloader = HfDownloader::new(self.cache_dir.clone());
        downloader.download_model(
            "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF",
            "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"
        ).await?;
    }

    Ok(model_path)
}
```

#### Step 3: Add First-Run UX (2-3 hours)
```rust
// src/cli/mod.rs
pub async fn run(&self) -> Result<()> {
    // Check if first run
    if !self.backend_manager.has_cached_model() {
        self.show_first_run_message();
    }

    // Rest of CLI logic...
}

fn show_first_run_message(&self) {
    println!("👋 Welcome to cmdai!");
    println!();
    println!("📦 First-time setup: Downloading model (~1.1GB)");
    println!("This is a one-time operation.");
    println!("Model will be cached at: {}", cache_dir);
    println!();
}
```

#### Step 4: Testing (4-6 hours)
```bash
# Unit tests with mock HTTP
cargo test hf_download

# Integration test with local server
cargo test hf_download_integration

# E2E test (optional, slow)
CMDAI_TEST_REAL_DOWNLOAD=1 cargo test hf_download_e2e
```

**Acceptance**:
- [ ] Fresh install downloads model automatically
- [ ] Resume works after interruption
- [ ] Checksum validation prevents corruption
- [ ] Progress bar shows accurate ETA
- [ ] Model loads and generates commands

**Test Command**:
```bash
# Clear cache for testing
rm -rf ~/.cache/cmdai/

# Run fresh install
cargo run -- "list files"
# Should see: "📦 Downloading model..." with progress
```

---

### 🟡 Priority 3: Update Documentation
**Effort**: 4-6 hours
**Owner**: Unassigned

**Files to Update**:
1. **README.md** - Add installation and usage examples
2. **QUICKSTART.md** (NEW) - First-time user guide
3. **TROUBLESHOOTING.md** (NEW) - Common issues and fixes

**Key Sections**:
```markdown
## README.md Updates

### Installation
- One-liner install script
- Manual download instructions
- Cargo install alternative

### Usage Examples
- Basic command generation
- Backend selection
- Configuration file
- Safety levels

### Configuration
- Environment variables
- Config file format
- Backend configuration
```

---

## 🚀 NEXT WEEK: Performance & Distribution (Week 2)

### MLX Backend Optimization (8-16 hours)
- Create C++ MLX wrapper with Metal integration
- Update Rust FFI bridge
- Achieve <2s first inference on Apple Silicon
- Add benchmarks for performance validation

**Test Command**:
```bash
# Build with MLX on macOS
cargo build --release --features embedded-mlx

# Benchmark
cargo bench mlx_inference
```

### Binary Distribution Setup (8-12 hours)
- GitHub Actions workflow for multi-platform builds
- Installation script (install.sh)
- Binary size optimization (<50MB target)

**Platforms**:
- Linux x64 (GNU + musl)
- macOS x64 + ARM64
- Windows x64

---

## 📋 Week 3: Package Managers & Release

### Package Manager Support (6-10 hours)
- Homebrew formula
- Crates.io publishing
- AUR package (Arch Linux)
- Scoop manifest (Windows)

### v1.0.0 Release Preparation
- Security audit
- Performance benchmarks
- Documentation review
- Release announcement

---

## 🎯 Quick Command Reference

### Development Workflow
```bash
# Setup
cd /home/user/cmdai
. "$HOME/.cargo/env"

# Build & Test
cargo build --release
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check

# Run with logging
RUST_LOG=debug cargo run -- "list files"

# Benchmarks
cargo bench

# Check binary size
ls -lh target/release/cmdai
```

### Git Workflow
```bash
# Current branch
git status
# Should show: claude/project-roadmap-planning-011CUgsthZ8dB5hfkKuKbTFq

# Commit changes
git add .
git commit -m "feat: Add production roadmap and next steps guide"

# Push to feature branch
git push -u origin claude/project-roadmap-planning-011CUgsthZ8dB5hfkKuKbTFq
```

---

## 🤝 Getting Help

### Known Issues
See **TECH_DEBT.md** for:
- Contract test alignment (#4)
- HF model download (multiple TODOs)
- File permissions hardening (#6)
- Configuration error messages

### Resources
- **ROADMAP.md** - Detailed production plan
- **CLAUDE.md** - Project architecture and guidelines
- **TDD-WORKFLOW.md** - Test-driven development process
- **CONTRIBUTING.md** - Contribution guidelines

### Questions?
- GitHub Issues: https://github.com/wildcard/cmdai/issues
- GitHub Discussions: https://github.com/wildcard/cmdai/discussions

---

## 📊 Progress Tracking

### Week 1 Goals
- [ ] Contract tests fixed (4-8 hours)
- [ ] HF download implemented (16-24 hours)
- [ ] Documentation updated (4-6 hours)
- [ ] Tests passing: `cargo test --all-features`

**Total Effort**: ~24-38 hours

### Week 2 Goals
- [ ] MLX backend optimized (8-16 hours)
- [ ] Binary builds automated (8-12 hours)
- [ ] Benchmarks established (4-6 hours)

**Total Effort**: ~20-34 hours

### Week 3 Goals
- [ ] Package managers setup (6-10 hours)
- [ ] Security audit (4-6 hours)
- [ ] v1.0.0 release (2-4 hours)

**Total Effort**: ~12-20 hours

**Grand Total**: 56-92 hours → **6-8 weeks** for v1.0

---

**Last Updated**: 2025-11-01
**Next Review**: After Week 1 completion
**Status**: Ready to start → Fix contract tests first
