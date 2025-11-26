# cmdai Production Roadmap

**Status**: 80% Complete | **Goal**: Production-Ready v1.0
**Current Branch**: `claude/project-roadmap-planning-011CUgsthZ8dB5hfkKuKbTFq`

## Executive Summary

cmdai has excellent foundations with all core infrastructure complete. To become fully operational, we need to:
1. **Fix contract test alignment** (4-8 hours) - Unblock CI/CD
2. **Implement model download** (16-24 hours) - Enable offline-first operation
3. **Optimize MLX backend** (8-16 hours) - Deliver Apple Silicon performance promise
4. **Create binary distribution** (8-12 hours) - Enable user installation

**Total Estimated Effort**: 36-60 hours to v1.0 (1-2 sprints)

---

## 🚨 Critical Path to v1.0 (Production Ready)

### Phase 1: Stabilization (Week 1) - **CRITICAL**

#### 1.1 Fix Contract Test API Alignment ⚡ **BLOCKER**
**Priority**: P0 | **Effort**: 4-8 hours | **Status**: 🔴 Blocking CI/CD

**Problem**: ~35 compilation errors in test suite due to API signature mismatches
- Config contract tests expect different constructor signatures
- Logging contract tests have outdated API calls
- Prevents full test suite from passing

**Tasks**:
```rust
// Decision needed: Update tests OR update implementation
// Recommended: Update tests to match implemented APIs

// Files to review:
- tests/contract/config_contract.rs
- tests/contract/logging_contract.rs
- src/config/mod.rs
- src/logging/mod.rs
```

**Acceptance Criteria**:
- [ ] All contract tests compile without errors
- [ ] Full test suite passes (`cargo test --all-features`)
- [ ] CI/CD pipeline runs green
- [ ] No breaking API changes to production code

**Implementation Plan**:
1. Run `cargo test --lib 2>&1 | tee test-errors.log`
2. Categorize errors (constructor signatures, method calls, return types)
3. Update test contracts to match implemented APIs
4. Verify all 44+ tests pass
5. Document API decisions in CHANGELOG.md

---

#### 1.2 Implement Hugging Face Model Download ⚡ **BLOCKER**
**Priority**: P0 | **Effort**: 16-24 hours | **Status**: 🔴 Core Feature Missing

**Problem**: Embedded backend cannot download models from Hugging Face Hub
- Currently expects pre-downloaded models
- Breaks "offline-first" promise for new users
- No automated model acquisition workflow

**Required Components**:
```rust
// src/cache/hf_download.rs (NEW FILE)
pub struct HfDownloader {
    client: reqwest::Client,
    cache_dir: PathBuf,
    progress_tracker: ProgressTracker,
}

impl HfDownloader {
    // Core functionality:
    async fn download_model(&self, repo: &str, filename: &str) -> Result<PathBuf>;
    async fn resume_download(&self, partial_path: &PathBuf) -> Result<PathBuf>;
    fn verify_checksum(&self, path: &PathBuf, expected: &str) -> Result<bool>;
    fn show_progress(&self, bytes: u64, total: u64);
}
```

**Technical Requirements**:
- HTTP client with range request support (resume downloads)
- SHA256 checksum validation against HF Hub manifest
- Progress bar with ETA using `indicatif`
- Bandwidth throttling (optional, respect user network)
- Automatic retry with exponential backoff (3 attempts)

**Model Targets**:
- Primary: `Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF` (Q4_K_M, ~1.1GB)
- Fallback: `Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF` (Q4_K_M, ~400MB)

**HF Hub API Endpoints**:
```bash
# Model file download
GET https://huggingface.co/{repo}/resolve/{revision}/{filename}

# Resume download with Range header
GET https://huggingface.co/{repo}/resolve/{revision}/{filename}
Range: bytes=<start>-<end>

# Get file metadata (for checksum)
HEAD https://huggingface.co/{repo}/resolve/{revision}/{filename}
```

**Integration Points**:
1. Update `src/backends/embedded/mod.rs`:
   ```rust
   pub async fn ensure_model_available(&self) -> Result<PathBuf> {
       let model_path = self.cache_dir.join("qwen2.5-coder-1.5b-q4km.gguf");
       if !model_path.exists() {
           let downloader = HfDownloader::new(self.cache_dir.clone());
           downloader.download_model(
               "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF",
               "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"
           ).await?;
       }
       Ok(model_path)
   }
   ```

2. Add first-run UX to CLI:
   ```rust
   // src/cli/mod.rs
   if !self.has_model_cached() {
       println!("📦 First-time setup: Downloading Qwen2.5-Coder model (~1.1GB)");
       println!("This is a one-time operation. Model will be cached for offline use.");
       // Show progress bar during download
   }
   ```

**Testing Strategy**:
- Unit tests with mock HTTP responses
- Integration test with local HTTP server (wiremock)
- Property tests for resume logic
- E2E test with actual HF download (optional, slow)

**Acceptance Criteria**:
- [ ] Fresh install can download model without manual intervention
- [ ] Download can be resumed after interruption
- [ ] Checksum validation prevents corrupted models
- [ ] Progress bar shows accurate ETA
- [ ] Downloaded model works with embedded backend
- [ ] Cached manifest updated after successful download

---

#### 1.3 Create Minimal Documentation & Examples
**Priority**: P1 | **Effort**: 4-6 hours

**Tasks**:
- [ ] Update README.md with installation instructions
- [ ] Add usage examples for all backends
- [ ] Document environment variables and config file format
- [ ] Create QUICKSTART.md for first-time users
- [ ] Add troubleshooting section

**Example Usage Section**:
```markdown
## Quick Start

### Installation
\`\`\`bash
# Download latest release
curl -L https://github.com/wildcard/cmdai/releases/latest/download/cmdai-linux-x64 -o cmdai
chmod +x cmdai
sudo mv cmdai /usr/local/bin/

# Verify installation
cmdai --version
\`\`\`

### First Command
\`\`\`bash
# First run downloads model (~1.1GB, one-time)
cmdai "list all files in current directory sorted by size"

# Generated command (with safety check):
# ls -lhS

# Execute? [y/n]: y
\`\`\`

### Backend Configuration
\`\`\`bash
# Use local Ollama
cmdai --backend ollama "find large files"

# Use remote vLLM
export VLLM_API_KEY="your-key"
cmdai --backend vllm "create backup archive"

# Show current config
cmdai --show-config
\`\`\`
```

---

### Phase 2: Performance & Optimization (Week 2)

#### 2.1 Full MLX Backend Implementation 🚀
**Priority**: P1 | **Effort**: 8-16 hours | **Impact**: Deliver on Apple Silicon promise

**Current State**:
- MLX backend structure exists with cxx FFI
- Simulated inference (500ms placeholder)
- No actual Metal Performance Shaders integration

**Goal**: Achieve <2s first inference on Apple Silicon M1/M2/M3

**Implementation Tasks**:

1. **Create C++ MLX Wrapper** (`src/backends/embedded/mlx_wrapper.cpp`):
```cpp
#include "mlx/mlx.h"
#include "cmdai/src/backends/embedded/mlx_bridge.rs.h"

namespace cmdai {
    class MLXInferenceEngine {
        mlx::core::array weights;
        mlx::core::Stream gpu_stream;

    public:
        void load_model(const std::string& path);
        std::string generate(const std::string& prompt, int max_tokens);
        void warmup();
        ~MLXInferenceEngine();
    };
}
```

2. **Update Rust FFI Bridge** (`src/backends/embedded/mlx_bridge.rs`):
```rust
#[cxx::bridge(namespace = "cmdai")]
mod ffi {
    unsafe extern "C++" {
        include!("cmdai/src/backends/embedded/mlx_wrapper.hpp");

        type MLXInferenceEngine;
        fn new_mlx_engine() -> UniquePtr<MLXInferenceEngine>;
        fn load_model(self: Pin<&mut MLXInferenceEngine>, path: &str) -> Result<()>;
        fn generate(self: Pin<&mut MLXInferenceEngine>,
                   prompt: &str,
                   max_tokens: i32) -> Result<String>;
        fn warmup(self: Pin<&mut MLXInferenceEngine>) -> Result<()>;
    }
}
```

3. **Integrate with EmbeddedModelBackend**:
```rust
// src/backends/embedded/mod.rs
#[cfg(feature = "embedded-mlx")]
async fn generate_with_mlx(&self, prompt: &str) -> Result<String> {
    let engine = self.mlx_engine.lock().await;
    let result = engine.generate(prompt, 100)?;
    Ok(result)
}
```

4. **Build System Updates**:
```toml
# Cargo.toml
[target.'cfg(target_os = "macos")'.dependencies]
mlx-rs = { version = "0.25", optional = true }

[build-dependencies]
cxx-build = "1.0"

# build.rs
fn main() {
    #[cfg(all(target_os = "macos", feature = "embedded-mlx"))]
    {
        cxx_build::bridge("src/backends/embedded/mlx_bridge.rs")
            .file("src/backends/embedded/mlx_wrapper.cpp")
            .flag_if_supported("-std=c++17")
            .flag_if_supported("-framework Metal")
            .flag_if_supported("-framework Foundation")
            .compile("cmdai-mlx");

        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
    }
}
```

**Performance Targets**:
- Model loading: <1s (cold start)
- First inference: <2s
- Subsequent inference: <500ms
- Memory usage: <2GB

**Acceptance Criteria**:
- [ ] MLX backend compiles on macOS with Apple Silicon
- [ ] Model loading uses Metal GPU acceleration
- [ ] Inference meets performance targets
- [ ] Graceful fallback to CPU backend on non-Apple hardware
- [ ] Benchmark suite shows 3-5x speedup vs CPU backend

---

#### 2.2 Performance Benchmarking Suite
**Priority**: P2 | **Effort**: 4-6 hours

**Goal**: Establish baseline performance metrics and detect regressions

**Tasks**:
```rust
// benches/benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_safety_validation(c: &mut Criterion) {
    c.bench_function("safety_check_safe_command", |b| {
        b.iter(|| validator.check(black_box("ls -la")))
    });

    c.bench_function("safety_check_dangerous_command", |b| {
        b.iter(|| validator.check(black_box("rm -rf /")))
    });
}

fn bench_embedded_inference(c: &mut Criterion) {
    c.bench_function("embedded_generate_command_mlx", |b| {
        b.iter(|| runtime.block_on(backend.generate(black_box(&request))))
    });
}

fn bench_cache_operations(c: &mut Criterion) {
    c.bench_function("cache_lru_lookup", |b| {
        b.iter(|| cache.get(black_box("model-key")))
    });
}

criterion_group!(benches,
    bench_safety_validation,
    bench_embedded_inference,
    bench_cache_operations
);
criterion_main!(benches);
```

**Metrics to Track**:
- CLI startup time (cold/warm)
- Safety validation (per pattern type)
- Model inference (MLX vs CPU)
- Cache hit/miss performance
- Configuration loading

**CI Integration**:
```yaml
# .github/workflows/benchmark.yml
- name: Run benchmarks
  run: cargo bench --features full
- name: Store benchmark result
  uses: benchmark-action/github-action-benchmark@v1
```

---

### Phase 3: Distribution & Packaging (Week 3)

#### 3.1 Binary Distribution Setup
**Priority**: P1 | **Effort**: 8-12 hours

**Goal**: Single-command installation for end users

**Multi-Platform Builds**:
```yaml
# .github/workflows/release.yml
name: Release Binaries

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: cmdai-linux-x64

          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            artifact: cmdai-linux-x64-static

          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: cmdai-macos-x64

          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: cmdai-macos-arm64

          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: cmdai-windows-x64.exe

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          target: ${{ matrix.target }}

      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }} --features full

      - name: Strip binary (Unix)
        if: runner.os != 'Windows'
        run: strip target/${{ matrix.target }}/release/cmdai

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.artifact }}
          path: target/${{ matrix.target }}/release/cmdai*
```

**Installation Scripts**:
```bash
# install.sh
#!/bin/bash
set -e

REPO="wildcard/cmdai"
INSTALL_DIR="/usr/local/bin"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)   PLATFORM="linux" ;;
    Darwin*)  PLATFORM="macos" ;;
    *)        echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64)   ARCH_SUFFIX="x64" ;;
    aarch64|arm64) ARCH_SUFFIX="arm64" ;;
    *)        echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

BINARY="cmdai-${PLATFORM}-${ARCH_SUFFIX}"
URL="https://github.com/${REPO}/releases/latest/download/${BINARY}"

echo "Downloading cmdai..."
curl -L "$URL" -o cmdai
chmod +x cmdai

echo "Installing to $INSTALL_DIR..."
sudo mv cmdai "$INSTALL_DIR/cmdai"

echo "✓ cmdai installed successfully!"
cmdai --version
```

**One-Liner Installation**:
```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash
```

**Acceptance Criteria**:
- [ ] Binaries build for 5 platforms (Linux x64, Linux musl, macOS x64, macOS ARM64, Windows x64)
- [ ] Binary size <50MB (after strip)
- [ ] Installation script works on fresh Ubuntu 22.04, macOS 13+, Windows 11
- [ ] `cmdai --version` works immediately after install
- [ ] No external dependencies required (static linking)

---

#### 3.2 Package Manager Support
**Priority**: P2 | **Effort**: 6-10 hours

**Homebrew Formula** (`cmdai.rb`):
```ruby
class Cmdai < Formula
  desc "Convert natural language to safe POSIX shell commands using local LLMs"
  homepage "https://github.com/wildcard/cmdai"
  version "1.0.0"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/wildcard/cmdai/releases/download/v1.0.0/cmdai-macos-arm64"
      sha256 "..." # Generated during release
    else
      url "https://github.com/wildcard/cmdai/releases/download/v1.0.0/cmdai-macos-x64"
      sha256 "..."
    end
  elsif OS.linux?
    url "https://github.com/wildcard/cmdai/releases/download/v1.0.0/cmdai-linux-x64"
    sha256 "..."
  end

  def install
    bin.install "cmdai-macos-arm64" => "cmdai" if OS.mac? && Hardware::CPU.arm?
    bin.install "cmdai-macos-x64" => "cmdai" if OS.mac? && Hardware::CPU.intel?
    bin.install "cmdai-linux-x64" => "cmdai" if OS.linux?
  end

  test do
    assert_match "cmdai", shell_output("#{bin}/cmdai --version")
  end
end
```

**Cargo Install Support**:
```bash
# Already works, but optimize for size
cargo install cmdai --locked --features embedded-cpu
```

**AUR Package** (Arch Linux):
```bash
# PKGBUILD
pkgname=cmdai
pkgver=1.0.0
pkgrel=1
pkgdesc="Convert natural language to safe POSIX shell commands"
arch=('x86_64')
url="https://github.com/wildcard/cmdai"
license=('MIT' 'Apache-2.0')
depends=()
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked --features embedded-cpu
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 target/release/cmdai "$pkgdir/usr/bin/cmdai"
}
```

**Distribution Channels**:
- [ ] Homebrew tap: `brew install wildcard/tap/cmdai`
- [ ] Crates.io: `cargo install cmdai`
- [ ] AUR (Arch): `yay -S cmdai`
- [ ] Snap (Ubuntu): `snap install cmdai`
- [ ] Scoop (Windows): `scoop install cmdai`

---

### Phase 4: Advanced Features (Weeks 4-6) - **POST-v1.0**

#### 4.1 Enhanced Context Awareness
**Priority**: P3 | **Effort**: 12-20 hours

**Features**:
- Working directory analysis (detect git repo, package.json, Cargo.toml)
- Shell history integration (avoid repeating failed commands)
- Environment variable context (PATH, EDITOR, SHELL)
- Recent file activity (files modified in last hour)

**Example Context Enrichment**:
```rust
// src/execution/context_enrichment.rs
pub struct RichContext {
    pub base: ExecutionContext,
    pub detected_projects: Vec<ProjectType>, // Git, Node, Rust, Python
    pub recent_commands: Vec<String>,        // Last 10 shell commands
    pub recent_files: Vec<PathBuf>,          // Modified in last hour
    pub available_tools: Vec<String>,        // From PATH
}

impl RichContext {
    pub fn to_prompt_context(&self) -> String {
        format!(
            "Working directory: {}\n\
             Detected project: {:?}\n\
             Available tools: {}\n\
             Recent activity: {}",
            self.base.current_dir.display(),
            self.detected_projects,
            self.available_tools.join(", "),
            self.recent_files.len()
        )
    }
}
```

**Updated System Prompt**:
```rust
const SYSTEM_PROMPT_V2: &str = r#"
You are an expert shell command generator. Generate POSIX-compliant commands.

CONTEXT:
- Working directory: {working_dir}
- Detected project: {project_type}
- Available tools: {tools}
- Shell: {shell}

CONSTRAINTS:
- Use ONLY installed tools from available_tools list
- Prefer project-specific commands (e.g., npm for Node projects)
- Avoid commands that conflict with recent history
- Generate single, safe, executable command

OUTPUT FORMAT: {"cmd": "command here"}
"#;
```

---

#### 4.2 Multi-Step Goal Completion
**Priority**: P3 | **Effort**: 16-24 hours

**Use Case**: "Set up a new Rust project with GitHub CI"

**Current Behavior**: Single command generation
```bash
$ cmdai "set up new rust project"
# Output: cargo new my-project
```

**Enhanced Behavior**: Multi-step plan with approval
```bash
$ cmdai --multi-step "set up new rust project with github ci"

📋 Generated Plan (4 steps):
1. cargo new my-project --bin
2. cd my-project && git init
3. mkdir -p .github/workflows
4. cat > .github/workflows/ci.yml <<EOF
   [GitHub Actions workflow content]
   EOF

Execute all? [y/n/step]: step

Step 1/4: cargo new my-project --bin
Execute? [y/n/quit]: y
✓ Completed

Step 2/4: cd my-project && git init
Execute? [y/n/quit]: y
✓ Completed
...
```

**Implementation**:
```rust
// src/models/multi_step.rs
pub struct ExecutionPlan {
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub estimated_time: Duration,
}

pub struct PlanStep {
    pub command: String,
    pub description: String,
    pub safety_level: RiskLevel,
    pub can_undo: bool,
}

// Enhanced system prompt for planning
const PLANNING_PROMPT: &str = r#"
Generate a multi-step execution plan as JSON array.

INPUT: "{user_goal}"

OUTPUT FORMAT:
{
  "steps": [
    {"cmd": "step1", "desc": "What this does", "risk": "safe"},
    {"cmd": "step2", "desc": "What this does", "risk": "moderate"}
  ]
}
"#;
```

---

#### 4.3 Shell Script Generation Mode
**Priority**: P3 | **Effort**: 8-12 hours

**Use Case**: Save generated commands as executable scripts

```bash
$ cmdai --script backup.sh "create compressed backup of src/ to ~/backups/"

Generated script saved to: backup.sh

#!/bin/bash
set -euo pipefail

# Generated by cmdai on 2025-11-01
# Goal: create compressed backup of src/ to ~/backups/

BACKUP_DIR="$HOME/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"
tar -czf "$BACKUP_DIR/src_backup_$TIMESTAMP.tar.gz" src/

echo "✓ Backup created: $BACKUP_DIR/src_backup_$TIMESTAMP.tar.gz"

$ chmod +x backup.sh
$ ./backup.sh
```

**Features**:
- Shebang selection (bash, zsh, sh)
- Error handling (`set -euo pipefail`)
- Timestamped operations
- Logging and confirmation messages
- Variable extraction (avoid hardcoded paths)

---

### Phase 5: Community & Ecosystem (Ongoing)

#### 5.1 Plugin System (Future)
**Priority**: P4 | **Effort**: 24-40 hours

**Vision**: User-extensible command generation
```rust
// ~/.config/cmdai/plugins/git_expert.wasm
// Custom plugin for advanced git workflows

pub trait CmdaiPlugin {
    fn can_handle(&self, request: &str) -> bool;
    fn generate(&self, request: &str) -> Result<String>;
    fn safety_patterns(&self) -> Vec<SafetyPattern>;
}
```

**Use Cases**:
- Git workflow plugins (semantic commit messages)
- Docker compose generation
- Kubernetes manifest generation
- Project-specific command templates

---

## 📊 Success Metrics for v1.0

### Performance
- ✅ CLI startup: <100ms (release build)
- ✅ Model loading: <2s (MLX), <5s (CPU)
- ✅ Command generation: <3s end-to-end
- ✅ Binary size: <50MB (stripped)

### Safety
- ✅ Zero false negatives (dangerous commands always caught)
- ✅ <5% false positives (safe commands incorrectly flagged)
- ✅ 100% pattern coverage for OWASP top shell risks

### Usability
- ✅ One-command installation on major platforms
- ✅ Works offline after first-time setup
- ✅ No configuration required for basic usage
- ✅ Clear error messages with actionable suggestions

### Quality
- ✅ 90%+ test coverage
- ✅ All clippy lints pass (deny warnings)
- ✅ Documented public API (rustdoc)
- ✅ CHANGELOG with all features

---

## 🎯 Release Checklist

### Pre-Release (v0.9.0 - Beta)
- [ ] Fix contract test alignment
- [ ] Implement HF model download
- [ ] Basic MLX backend (simulated OK)
- [ ] Linux + macOS binaries
- [ ] Installation script tested
- [ ] Beta announcement to early adopters

### v1.0.0 Release
- [ ] Full MLX backend with Metal
- [ ] All platforms (Linux, macOS, Windows)
- [ ] Package manager support (Homebrew + Cargo)
- [ ] Comprehensive documentation
- [ ] Performance benchmarks published
- [ ] Security audit completed
- [ ] LICENSE and CONTRIBUTING.md finalized
- [ ] v1.0.0 tag and GitHub release
- [ ] Announcement (Hacker News, Reddit r/rust, Twitter)

### Post-v1.0 (v1.1+)
- [ ] Enhanced context awareness
- [ ] Multi-step goal completion
- [ ] Shell script generation
- [ ] Plugin system (v2.0 target)

---

## 🚀 Quick Start for Contributors

### Setup Development Environment
```bash
# Clone repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Load Rust environment
. "$HOME/.cargo/env"

# Install dependencies
cargo build --features full

# Run tests
cargo test --lib

# Run with debug logging
RUST_LOG=debug cargo run -- "list files"
```

### Priority Areas for Contribution
1. **Contract Test Fixes** (Good first issue) - 4-8 hours
2. **HF Model Download** (Core feature) - 16-24 hours
3. **Documentation Examples** (Good first issue) - 2-4 hours
4. **Platform Testing** (Help wanted) - Test on your OS/architecture

---

## 📞 Getting Help

- **Issues**: https://github.com/wildcard/cmdai/issues
- **Discussions**: https://github.com/wildcard/cmdai/discussions
- **Technical Debt**: See TECH_DEBT.md for known issues
- **Contributing**: See CONTRIBUTING.md for guidelines

---

**Last Updated**: 2025-11-01
**Roadmap Owner**: @wildcard
**Status**: Active Development → Production Ready (6-8 weeks estimated)
