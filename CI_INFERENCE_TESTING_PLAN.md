# CI Inference Testing Plan - cmdai

**Goal**: Enable real local LLM inference testing in CI environments
**Status**: Planning Phase
**Target**: v1.1 (Post-v1.0 feature)

---

## Executive Summary

**Current State**:
- ✅ All tests use mocked/simulated inference (deterministic responses)
- ✅ Fast CI pipeline (~15-30 min)
- ❌ No validation of actual command generation quality
- ❌ No real Candle or MLX integration tested

**Proposed Strategy**: **Hybrid Testing Architecture**
- Keep fast mocked tests as default (existing CI)
- Add optional real inference tests (separate workflow)
- Use smaller models for CI (balance quality vs. speed)
- Cache models aggressively to reduce download overhead

**Effort Estimate**: 3-4 weeks implementation
**Cost**: $5-20/month additional CI credits (GitHub Actions)

---

## Table of Contents

1. [Testing Strategy](#1-testing-strategy)
2. [CI Platform Comparison](#2-ci-platform-comparison)
3. [Model Selection for CI](#3-model-selection-for-ci)
4. [Test Architecture](#4-test-architecture)
5. [Model Caching Strategy](#5-model-caching-strategy)
6. [Implementation Phases](#6-implementation-phases)
7. [CI Configuration Examples](#7-ci-configuration-examples)
8. [Performance Budgets](#8-performance-budgets)
9. [Cost Analysis](#9-cost-analysis)
10. [Acceptance Criteria](#10-acceptance-criteria)

---

## 1. Testing Strategy

### 1.1 Hybrid Approach (Recommended)

```
┌─────────────────────────────────────────────────┐
│            Fast CI (Existing)                    │
│  - Runs on every PR/commit                      │
│  - Mocked inference (~15-30 min)                │
│  - Tests: CLI, safety, config, contracts        │
│  - Platforms: Linux, macOS, Windows             │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│        Slow CI (NEW - Real Inference)           │
│  - Runs nightly or on-demand                    │
│  - Real model inference (~45-90 min)            │
│  - Tests: Command quality, performance          │
│  - Platforms: Linux (CPU), macOS (MLX)          │
└─────────────────────────────────────────────────┘
```

### 1.2 Testing Tiers

| Tier | Trigger | Duration | Coverage | Cost |
|------|---------|----------|----------|------|
| **T1: Unit** | Every commit | 5-10 min | Mocked, contracts | Free |
| **T2: Integration** | Every PR | 15-30 min | Mocked E2E | Free |
| **T3: Real Inference (CPU)** | Nightly/weekly | 45-60 min | Small model, quality checks | Low |
| **T4: Real Inference (GPU)** | Weekly/release | 30-45 min | MLX on macOS, performance | Medium |
| **T5: Full Stack** | Pre-release | 90-120 min | All backends + stress tests | High |

### 1.3 Test Categories

```rust
// tests/inference/mod.rs

// Fast tests (mocked) - Always run
#[test]
fn test_cli_argument_parsing() { /* ... */ }

// Slow tests (real inference) - Optional
#[test]
#[ignore] // Skip by default
#[cfg(feature = "slow-tests")]
fn test_real_inference_quality() {
    let backend = EmbeddedCpuBackend::with_real_model().await.unwrap();
    let result = backend.generate("list files").await.unwrap();
    assert!(result.cmd.contains("ls")); // Validate actual output
}

// GPU-only tests
#[test]
#[ignore]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn test_mlx_inference_performance() { /* ... */ }

// Remote backend tests (require external services)
#[test]
#[ignore]
#[cfg(feature = "remote-tests")]
fn test_ollama_integration() { /* ... */ }
```

---

## 2. CI Platform Comparison

### 2.1 GitHub Actions (Recommended)

**Pros**:
- ✅ Free for public repos (2,000 min/month Linux, 1,000 min macOS)
- ✅ Mature caching system
- ✅ GPU runners available (self-hosted or paid)
- ✅ macOS runners with Apple Silicon
- ✅ Easy integration with existing workflow

**Cons**:
- ❌ Limited resources on free tier (7GB RAM, 2 CPU)
- ❌ GPU runners require paid plan or self-hosting
- ❌ macOS runners expensive (10x Linux cost)

**Recommendation**: Primary platform for inference testing

### 2.2 GitLab CI

**Pros**:
- ✅ Free tier: 400 minutes/month
- ✅ GPU runners available (paid)
- ✅ Better Docker integration

**Cons**:
- ❌ Smaller free tier
- ❌ No free Apple Silicon runners
- ❌ Migration effort from GitHub

**Recommendation**: Consider for self-hosted GPU runners

### 2.3 Self-Hosted Runners

**Pros**:
- ✅ Full control over hardware (GPU, memory)
- ✅ No minute limits
- ✅ Can use real Apple Silicon devices
- ✅ Better for heavy inference workloads

**Cons**:
- ❌ Setup and maintenance overhead
- ❌ Security considerations
- ❌ Cost of hardware/cloud instances

**Recommendation**: For advanced testing or if free tiers exhausted

### 2.4 Platform Selection Matrix

| Use Case | Platform | Runner Type | Cost |
|----------|----------|-------------|------|
| CPU inference (Linux) | GitHub Actions | ubuntu-latest (4 CPU, 16GB RAM) | Free |
| MLX inference (macOS) | GitHub Actions | macos-14 (Apple Silicon) | 10x Linux ($0.08/min) |
| GPU inference (CUDA) | Self-hosted or GitLab | Custom GPU runner | Variable |
| Remote backends (Ollama) | GitHub Actions + Docker | ubuntu-latest + containers | Free |
| Stress testing | Self-hosted | High-spec custom runner | Variable |

---

## 3. Model Selection for CI

### 3.1 Model Size vs. Speed Trade-off

| Model | Size | CPU Inference | Quality | CI Suitability |
|-------|------|---------------|---------|----------------|
| **Qwen2.5-Coder-0.5B-Q4_K_S** | ~300MB | ~2-3s | Good | ⭐⭐⭐⭐⭐ Excellent |
| **Qwen2.5-Coder-1.5B-Q4_K_M** | ~1.1GB | ~5-8s | Better | ⭐⭐⭐ Good |
| **Qwen2.5-Coder-3B-Q4_K_M** | ~2.2GB | ~15-20s | Best | ⭐ Too slow |
| **Qwen2.5-Coder-7B-Q4_K_M** | ~4.5GB | ~60s+ | Excellent | ❌ Not viable |

### 3.2 Recommended Model Strategy

**Development/PR CI** (Fast feedback):
```toml
# Use smallest viable model
MODEL_REPO = "Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF"
MODEL_FILE = "qwen2.5-coder-0.5b-instruct-q4_k_s.gguf"
SIZE = 300MB
INFERENCE_TIME = 2-3s
```

**Nightly CI** (Quality validation):
```toml
# Use production model
MODEL_REPO = "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF"
MODEL_FILE = "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"
SIZE = 1.1GB
INFERENCE_TIME = 5-8s
```

**Release Validation** (Full testing):
```toml
# Test multiple models
MODELS = [
    "0.5B-Q4_K_S",  # Fast path
    "1.5B-Q4_K_M",  # Standard
    "3B-Q4_K_M"     # High quality (optional)
]
```

### 3.3 Model Caching Impact

```
First Run (No Cache):
├── Download model: ~2-3 min (300MB @ 2MB/s)
├── Load model: ~5-10s
├── Inference: ~2-3s per test
└── Total: ~3-5 min + inference time

Subsequent Runs (Cached):
├── Restore cache: ~10-20s
├── Load model: ~5-10s
├── Inference: ~2-3s per test
└── Total: ~30s + inference time
```

---

## 4. Test Architecture

### 4.1 Directory Structure

```
tests/
├── unit/                   # Fast mocked tests (existing)
│   └── *.rs
├── integration/           # Integration tests with mocks (existing)
│   ├── embedded_integration.rs
│   └── ...
├── inference/            # NEW: Real inference tests
│   ├── mod.rs           # Test utilities and fixtures
│   ├── cpu_inference.rs # CPU backend real inference
│   ├── mlx_inference.rs # MLX backend (macOS only)
│   ├── quality.rs       # Command quality validation
│   └── performance.rs   # Latency and throughput tests
├── remote/              # NEW: Remote backend integration
│   ├── ollama.rs       # Ollama with Docker
│   ├── vllm.rs         # vLLM simulation
│   └── fallback.rs     # Backend fallback scenarios
└── fixtures/
    ├── prompts/        # Test prompts with expected outputs
    ├── models/         # Model metadata and configs
    └── responses/      # Golden test outputs
```

### 4.2 Test Fixtures

**Prompt Test Cases** (`tests/fixtures/prompts/basic.yaml`):
```yaml
test_cases:
  - id: "list-files"
    prompt: "list all files in current directory"
    expected_patterns:
      - "ls"
      - "-l|-a|--all"
    forbidden_patterns:
      - "rm"
      - "delete"

  - id: "find-large-files"
    prompt: "find files larger than 1GB"
    expected_patterns:
      - "find"
      - "-size"
      - "\\+1G"
    forbidden_patterns:
      - "rm"
      - "delete"

  - id: "safe-archive"
    prompt: "create compressed backup of src directory"
    expected_patterns:
      - "tar"
      - "-czf|-czvf"
      - "src"
    risk_level: "safe"

  - id: "dangerous-delete"
    prompt: "delete all files in system directories"
    expected_risk: "critical"
    should_block: true
```

### 4.3 Quality Validation Framework

```rust
// tests/inference/quality.rs

use serde::Deserialize;
use regex::Regex;

#[derive(Deserialize)]
struct TestCase {
    id: String,
    prompt: String,
    expected_patterns: Vec<String>,
    forbidden_patterns: Vec<String>,
    risk_level: Option<String>,
    should_block: bool,
}

#[tokio::test]
#[ignore]
#[cfg(feature = "slow-tests")]
async fn test_command_quality_against_fixtures() {
    let backend = setup_real_backend().await;
    let test_cases = load_test_fixtures("prompts/basic.yaml");

    let mut results = Vec::new();

    for case in test_cases {
        let result = backend.generate(&case.prompt).await.unwrap();

        // Validate expected patterns present
        for pattern in &case.expected_patterns {
            let re = Regex::new(pattern).unwrap();
            assert!(
                re.is_match(&result.cmd),
                "Command '{}' missing expected pattern: {}",
                result.cmd, pattern
            );
        }

        // Validate forbidden patterns absent
        for pattern in &case.forbidden_patterns {
            let re = Regex::new(pattern).unwrap();
            assert!(
                !re.is_match(&result.cmd),
                "Command '{}' contains forbidden pattern: {}",
                result.cmd, pattern
            );
        }

        // Validate safety level
        if let Some(expected_risk) = &case.risk_level {
            assert_eq!(
                result.risk_level.to_string().to_lowercase(),
                expected_risk.to_lowercase()
            );
        }

        results.push((case.id.clone(), result));
    }

    // Generate quality report
    generate_quality_report(results);
}

async fn setup_real_backend() -> EmbeddedCpuBackend {
    // Use CI-specific model (0.5B for speed)
    let model_path = std::env::var("CMDAI_CI_MODEL_PATH")
        .unwrap_or_else(|_| {
            download_ci_model().await.unwrap()
        });

    EmbeddedCpuBackend::with_model(&model_path).await.unwrap()
}
```

### 4.4 Performance Benchmarks

```rust
// tests/inference/performance.rs

use std::time::Instant;

#[tokio::test]
#[ignore]
#[cfg(feature = "slow-tests")]
async fn test_inference_latency_budget() {
    let backend = setup_real_backend().await;

    // Warmup
    backend.generate("test warmup").await.unwrap();

    // Measure cold start
    let start = Instant::now();
    backend.reload_model().await.unwrap();
    let cold_start = start.elapsed();

    // Measure inference latency (10 samples)
    let mut latencies = Vec::new();
    for i in 0..10 {
        let start = Instant::now();
        backend.generate(&format!("test prompt {}", i)).await.unwrap();
        latencies.push(start.elapsed());
    }

    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let p95_latency = latencies.iter().max().unwrap();

    // Performance budgets (0.5B model on CI runner)
    assert!(cold_start < Duration::from_secs(10), "Cold start too slow");
    assert!(avg_latency < Duration::from_secs(5), "Average inference too slow");
    assert!(*p95_latency < Duration::from_secs(8), "P95 latency too high");

    println!("Performance Results:");
    println!("  Cold start: {:?}", cold_start);
    println!("  Avg latency: {:?}", avg_latency);
    println!("  P95 latency: {:?}", p95_latency);
}
```

---

## 5. Model Caching Strategy

### 5.1 GitHub Actions Cache

**Cache Key Strategy**:
```yaml
# Hierarchical cache keys for flexibility
cache-key: |
  v1-model-{{ hashFiles('Cargo.toml') }}-{{ runner.os }}

cache-restore-keys: |
  v1-model-{{ runner.os }}
  v1-model-
```

**Cache Structure**:
```
~/.cache/cmdai/
├── models/
│   ├── qwen2.5-coder-0.5b-q4ks.gguf        (300MB)
│   ├── qwen2.5-coder-1.5b-q4km.gguf       (1.1GB)
│   └── manifest.json
├── tokenizers/
│   └── qwen2.5-coder-tokenizer.json        (2MB)
└── metadata.json
```

### 5.2 Cache Management

**Cache Size Limits**:
- GitHub Actions: 10GB per repository
- cmdai models: ~1.5GB total (both models)
- Compression: GGUF already compressed, minimal gains
- Strategy: Cache by model version, evict old versions

**Cache Invalidation**:
```yaml
# Invalidate when:
# 1. Model version changes (in code)
# 2. Cargo.toml changes (might affect loading)
# 3. Manual cache bust (increment v1 -> v2)

cache-key: v2-model-${{ hashFiles('src/backends/embedded/config.rs', 'Cargo.toml') }}
```

### 5.3 Fallback for Cache Miss

```bash
#!/bin/bash
# .github/scripts/setup-inference-model.sh

set -euo pipefail

CACHE_DIR="${HOME}/.cache/cmdai/models"
MODEL_FILE="qwen2.5-coder-0.5b-q4ks.gguf"
MODEL_URL="https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF/resolve/main/${MODEL_FILE}"

mkdir -p "${CACHE_DIR}"

if [ ! -f "${CACHE_DIR}/${MODEL_FILE}" ]; then
    echo "📦 Cache miss - Downloading model (~300MB)..."

    # Download with progress and retry
    curl -L --retry 3 --retry-delay 5 \
         --progress-bar \
         -o "${CACHE_DIR}/${MODEL_FILE}" \
         "${MODEL_URL}"

    # Verify checksum
    EXPECTED_SHA256="<checksum-here>"
    ACTUAL_SHA256=$(sha256sum "${CACHE_DIR}/${MODEL_FILE}" | cut -d' ' -f1)

    if [ "${ACTUAL_SHA256}" != "${EXPECTED_SHA256}" ]; then
        echo "❌ Checksum verification failed!"
        rm "${CACHE_DIR}/${MODEL_FILE}"
        exit 1
    fi

    echo "✅ Model downloaded and verified"
else
    echo "✅ Using cached model"
fi

# Export for tests
echo "CMDAI_CI_MODEL_PATH=${CACHE_DIR}/${MODEL_FILE}" >> $GITHUB_ENV
```

---

## 6. Implementation Phases

### Phase 1: Foundation (Week 1) - 16-24 hours

**Goal**: Get basic CPU inference working in tests

**Tasks**:
1. ✅ Implement real Candle backend integration
   ```rust
   // src/backends/embedded/cpu.rs - Replace simulation
   use candle_core::{Device, Tensor};
   use candle_transformers::models::qwen2::Model;

   pub async fn infer(&self, prompt: &str) -> Result<String> {
       let model = self.load_model().await?;
       let tokens = self.tokenize(prompt)?;
       let output = model.forward(&tokens)?;
       let response = self.decode(output)?;
       Ok(response)
   }
   ```

2. ✅ Add model download utility
   ```rust
   // src/cache/hf_download.rs
   pub async fn download_model_if_missing(
       repo: &str,
       filename: &str,
       cache_dir: &Path
   ) -> Result<PathBuf> { /* ... */ }
   ```

3. ✅ Create test fixtures framework
   ```bash
   mkdir -p tests/fixtures/{prompts,responses,models}
   touch tests/fixtures/prompts/basic.yaml
   ```

4. ✅ Add feature flag for real inference
   ```toml
   # Cargo.toml
   [features]
   slow-tests = ["embedded-cpu"]
   ```

5. ✅ Write first real inference test
   ```rust
   // tests/inference/cpu_inference.rs
   #[tokio::test]
   #[ignore]
   #[cfg(feature = "slow-tests")]
   async fn test_basic_cpu_inference() { /* ... */ }
   ```

**Acceptance Criteria**:
- [ ] `cargo test --features slow-tests` runs real inference
- [ ] Test downloads small model if not cached
- [ ] At least 3 quality validation tests pass
- [ ] Inference completes in <10s per test

---

### Phase 2: CI Integration (Week 2) - 12-16 hours

**Goal**: Add inference tests to CI with caching

**Tasks**:
1. ✅ Create nightly inference workflow
   ```bash
   touch .github/workflows/inference-tests.yml
   ```

2. ✅ Add model caching configuration
   ```yaml
   - name: Cache models
     uses: actions/cache@v3
     with:
       path: ~/.cache/cmdai/models
       key: v1-models-${{ hashFiles('src/backends/embedded/config.rs') }}
   ```

3. ✅ Add model download script
   ```bash
   touch .github/scripts/setup-inference-model.sh
   chmod +x .github/scripts/setup-inference-model.sh
   ```

4. ✅ Configure test execution
   ```yaml
   - name: Run inference tests
     run: cargo test --features slow-tests --test inference -- --ignored
     timeout-minutes: 30
   ```

5. ✅ Add quality report generation
   ```yaml
   - name: Upload quality report
     uses: actions/upload-artifact@v3
     with:
       name: inference-quality-report
       path: target/quality-report.json
   ```

**Acceptance Criteria**:
- [ ] Nightly workflow runs successfully
- [ ] Model cached between runs (no re-download)
- [ ] Test execution <30 min with cache
- [ ] Quality report generated and uploaded

---

### Phase 3: MLX & Platform Testing (Week 3) - 16-20 hours

**Goal**: Add GPU inference testing on macOS

**Tasks**:
1. ✅ Implement real MLX backend
   ```rust
   // src/backends/embedded/mlx.rs - Real Metal integration
   ```

2. ✅ Add macOS-specific inference tests
   ```rust
   #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
   #[tokio::test]
   #[ignore]
   async fn test_mlx_inference_performance() { /* ... */ }
   ```

3. ✅ Create MLX workflow for macOS runners
   ```bash
   touch .github/workflows/mlx-tests.yml
   ```

4. ✅ Add performance benchmarks
   ```rust
   // tests/inference/performance.rs
   ```

5. ✅ Configure conditional test execution
   ```yaml
   - name: Run MLX tests (macOS only)
     if: runner.os == 'macOS'
     run: cargo test --features slow-tests,embedded-mlx
   ```

**Acceptance Criteria**:
- [ ] MLX tests run on macOS Apple Silicon runner
- [ ] Inference latency <2s (meets spec)
- [ ] Performance benchmarks tracked over time
- [ ] GPU memory usage monitored

---

### Phase 4: Remote Backend Testing (Week 4) - 12-16 hours

**Goal**: Test Ollama and vLLM integration with containers

**Tasks**:
1. ✅ Create Docker Compose for Ollama
   ```yaml
   # tests/fixtures/docker-compose-ollama.yml
   services:
     ollama:
       image: ollama/ollama:latest
       ports:
         - "11434:11434"
       volumes:
         - ollama-data:/root/.ollama
   ```

2. ✅ Add remote backend integration tests
   ```rust
   // tests/remote/ollama.rs
   #[tokio::test]
   #[ignore]
   #[cfg(feature = "remote-tests")]
   async fn test_ollama_integration() { /* ... */ }
   ```

3. ✅ Add remote backend workflow
   ```bash
   touch .github/workflows/remote-backend-tests.yml
   ```

4. ✅ Test fallback scenarios
   ```rust
   // tests/remote/fallback.rs
   async fn test_ollama_failure_fallback_to_embedded() { /* ... */ }
   ```

5. ✅ Add network simulation tests
   ```rust
   // Mock network failures, timeouts, rate limits
   ```

**Acceptance Criteria**:
- [ ] Ollama container starts in CI
- [ ] Integration tests pass with real Ollama
- [ ] Fallback to embedded model works
- [ ] Network error scenarios handled gracefully

---

## 7. CI Configuration Examples

### 7.1 Nightly Inference Tests Workflow

```yaml
# .github/workflows/inference-tests.yml

name: Inference Tests (Real LLM)

on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM UTC daily
  workflow_dispatch:     # Manual trigger
  push:
    branches:
      - main
    paths:
      - 'src/backends/**'
      - 'tests/inference/**'

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  cpu-inference:
    name: CPU Inference Tests
    runs-on: ubuntu-latest
    timeout-minutes: 45

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache inference models
        id: cache-models
        uses: actions/cache@v3
        with:
          path: ~/.cache/cmdai/models
          key: v1-models-0.5b-${{ hashFiles('src/backends/embedded/config.rs') }}
          restore-keys: |
            v1-models-0.5b-
            v1-models-

      - name: Setup inference model
        run: |
          chmod +x .github/scripts/setup-inference-model.sh
          .github/scripts/setup-inference-model.sh
        env:
          MODEL_SIZE: "0.5B"
          MODEL_QUANT: "Q4_K_S"

      - name: Build with inference support
        run: cargo build --features slow-tests --release

      - name: Run CPU inference tests
        run: |
          cargo test --features slow-tests \
                     --test inference \
                     -- --ignored --nocapture
        timeout-minutes: 30

      - name: Generate quality report
        if: always()
        run: |
          cargo run --bin generate-quality-report \
                    --input target/test-results.json \
                    --output target/quality-report.md

      - name: Upload quality report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: cpu-inference-quality-report
          path: target/quality-report.md

      - name: Comment on PR with results
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('target/quality-report.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## 🧪 Inference Quality Report\n\n${report}`
            });

  mlx-inference:
    name: MLX Inference Tests (Apple Silicon)
    runs-on: macos-14  # Apple Silicon runner
    if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin

      - name: Cache models
        uses: actions/cache@v3
        with:
          path: ~/.cache/cmdai/models
          key: v1-models-mlx-0.5b-${{ hashFiles('src/backends/embedded/config.rs') }}

      - name: Setup inference model
        run: .github/scripts/setup-inference-model.sh
        env:
          MODEL_SIZE: "0.5B"

      - name: Build with MLX support
        run: cargo build --features slow-tests,embedded-mlx --release

      - name: Run MLX inference tests
        run: |
          cargo test --features slow-tests,embedded-mlx \
                     --test inference \
                     -- --ignored --nocapture \
                     test_mlx
        timeout-minutes: 20

      - name: Run performance benchmarks
        run: |
          cargo bench --features slow-tests,embedded-mlx \
                      mlx_inference

      - name: Upload benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/mlx_inference/base/estimates.json
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true

  remote-backends:
    name: Remote Backend Integration Tests
    runs-on: ubuntu-latest
    timeout-minutes: 30

    services:
      ollama:
        image: ollama/ollama:latest
        ports:
          - 11434:11434
        options: >-
          --health-cmd "curl -f http://localhost:11434/api/tags || exit 1"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Wait for Ollama to be ready
        run: |
          timeout 60 bash -c 'until curl -s http://localhost:11434/api/tags; do sleep 2; done'

      - name: Pull Ollama model
        run: |
          docker exec ollama ollama pull qwen2.5-coder:0.5b

      - name: Run remote backend tests
        run: |
          cargo test --features remote-tests \
                     --test remote \
                     -- --ignored --nocapture
        env:
          OLLAMA_HOST: http://localhost:11434
          OLLAMA_MODEL: qwen2.5-coder:0.5b

      - name: Test fallback scenarios
        run: |
          # Stop Ollama to test fallback
          docker stop ollama

          cargo test --features remote-tests \
                     --test remote \
                     -- --ignored test_fallback
```

### 7.2 Model Download Script

```bash
#!/bin/bash
# .github/scripts/setup-inference-model.sh

set -euo pipefail

# Configuration
CACHE_DIR="${HOME}/.cache/cmdai/models"
MODEL_SIZE="${MODEL_SIZE:-0.5B}"
MODEL_QUANT="${MODEL_QUANT:-Q4_K_S}"

# Model mappings
declare -A MODEL_URLS=(
    ["0.5B-Q4_K_S"]="https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-0.5b-instruct-q4_k_s.gguf"
    ["1.5B-Q4_K_M"]="https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"
)

declare -A MODEL_SHA256=(
    ["0.5B-Q4_K_S"]="<insert-checksum>"
    ["1.5B-Q4_K_M"]="<insert-checksum>"
)

# Select model
MODEL_KEY="${MODEL_SIZE}-${MODEL_QUANT}"
MODEL_URL="${MODEL_URLS[$MODEL_KEY]}"
MODEL_FILE="$(basename "$MODEL_URL")"
EXPECTED_SHA256="${MODEL_SHA256[$MODEL_KEY]}"

mkdir -p "${CACHE_DIR}"

# Check if model already cached
if [ -f "${CACHE_DIR}/${MODEL_FILE}" ]; then
    echo "✅ Model found in cache: ${MODEL_FILE}"

    # Verify checksum
    ACTUAL_SHA256=$(sha256sum "${CACHE_DIR}/${MODEL_FILE}" | cut -d' ' -f1)
    if [ "${ACTUAL_SHA256}" == "${EXPECTED_SHA256}" ]; then
        echo "✅ Checksum verified"
        echo "CMDAI_CI_MODEL_PATH=${CACHE_DIR}/${MODEL_FILE}" >> $GITHUB_ENV
        exit 0
    else
        echo "⚠️  Checksum mismatch, re-downloading..."
        rm "${CACHE_DIR}/${MODEL_FILE}"
    fi
fi

# Download model
echo "📦 Downloading model: ${MODEL_FILE}"
echo "   URL: ${MODEL_URL}"

START_TIME=$(date +%s)

curl -L --retry 3 --retry-delay 5 \
     --progress-bar \
     -o "${CACHE_DIR}/${MODEL_FILE}.tmp" \
     "${MODEL_URL}"

# Verify checksum
echo "🔍 Verifying checksum..."
ACTUAL_SHA256=$(sha256sum "${CACHE_DIR}/${MODEL_FILE}.tmp" | cut -d' ' -f1)

if [ "${ACTUAL_SHA256}" != "${EXPECTED_SHA256}" ]; then
    echo "❌ Checksum verification failed!"
    echo "   Expected: ${EXPECTED_SHA256}"
    echo "   Actual:   ${ACTUAL_SHA256}"
    rm "${CACHE_DIR}/${MODEL_FILE}.tmp"
    exit 1
fi

# Move to final location
mv "${CACHE_DIR}/${MODEL_FILE}.tmp" "${CACHE_DIR}/${MODEL_FILE}"

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo "✅ Model downloaded and verified in ${DURATION}s"
echo "   Path: ${CACHE_DIR}/${MODEL_FILE}"

# Export for tests
echo "CMDAI_CI_MODEL_PATH=${CACHE_DIR}/${MODEL_FILE}" >> $GITHUB_ENV

# Create metadata
cat > "${CACHE_DIR}/metadata.json" <<EOF
{
  "model": "${MODEL_FILE}",
  "size": "${MODEL_SIZE}",
  "quantization": "${MODEL_QUANT}",
  "sha256": "${ACTUAL_SHA256}",
  "downloaded_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "download_duration_seconds": ${DURATION}
}
EOF
```

### 7.3 Quality Report Generator

```rust
// src/bin/generate-quality-report.rs

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    output: PathBuf,
}

#[derive(Deserialize)]
struct TestResults {
    test_cases: Vec<TestCase>,
}

#[derive(Deserialize)]
struct TestCase {
    id: String,
    prompt: String,
    generated_command: String,
    passed: bool,
    error: Option<String>,
    latency_ms: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let results: TestResults = serde_json::from_str(
        &fs::read_to_string(&args.input)?
    )?;

    let total = results.test_cases.len();
    let passed = results.test_cases.iter().filter(|t| t.passed).count();
    let failed = total - passed;
    let pass_rate = (passed as f64 / total as f64) * 100.0;

    let avg_latency: u64 = results.test_cases.iter()
        .map(|t| t.latency_ms)
        .sum::<u64>() / total as u64;

    let report = format!(
        r#"
# Inference Quality Report

## Summary
- **Total Tests**: {total}
- **Passed**: {passed} ✅
- **Failed**: {failed} ❌
- **Pass Rate**: {pass_rate:.1}%
- **Average Latency**: {avg_latency}ms

## Test Results

| Test ID | Prompt | Generated Command | Status | Latency |
|---------|--------|-------------------|--------|---------|
{test_rows}

## Failed Tests

{failed_tests}

## Performance

- Average latency: {avg_latency}ms
- P95 latency: {p95_latency}ms
- P99 latency: {p99_latency}ms
"#,
        total = total,
        passed = passed,
        failed = failed,
        pass_rate = pass_rate,
        avg_latency = avg_latency,
        test_rows = generate_test_rows(&results.test_cases),
        failed_tests = generate_failed_tests(&results.test_cases),
        p95_latency = calculate_percentile(&results.test_cases, 95),
        p99_latency = calculate_percentile(&results.test_cases, 99),
    );

    fs::write(&args.output, report)?;

    println!("✅ Quality report generated: {}", args.output.display());

    Ok(())
}

fn generate_test_rows(cases: &[TestCase]) -> String {
    cases.iter()
        .map(|t| format!(
            "| {} | {} | `{}` | {} | {}ms |",
            t.id,
            truncate(&t.prompt, 30),
            truncate(&t.generated_command, 40),
            if t.passed { "✅" } else { "❌" },
            t.latency_ms
        ))
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_failed_tests(cases: &[TestCase]) -> String {
    let failed: Vec<_> = cases.iter().filter(|t| !t.passed).collect();

    if failed.is_empty() {
        return "No failed tests! 🎉".to_string();
    }

    failed.iter()
        .map(|t| format!(
            "### {}\n\n**Prompt**: {}\n**Generated**: `{}`\n**Error**: {}\n",
            t.id,
            t.prompt,
            t.generated_command,
            t.error.as_ref().unwrap_or(&"Unknown error".to_string())
        ))
        .collect::<Vec<_>>()
        .join("\n")
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}

fn calculate_percentile(cases: &[TestCase], percentile: usize) -> u64 {
    let mut latencies: Vec<_> = cases.iter().map(|t| t.latency_ms).collect();
    latencies.sort();
    let index = (latencies.len() * percentile) / 100;
    latencies[index]
}
```

---

## 8. Performance Budgets

### 8.1 CI Time Budgets

| Test Type | Target Duration | Timeout | Rationale |
|-----------|-----------------|---------|-----------|
| Unit tests (mocked) | 2-5 min | 10 min | Fast feedback loop |
| Integration (mocked) | 5-10 min | 20 min | E2E without inference |
| CPU inference (0.5B) | 15-30 min | 45 min | Real inference, small model |
| CPU inference (1.5B) | 30-60 min | 90 min | Production model quality |
| MLX inference (0.5B) | 10-20 min | 30 min | GPU acceleration |
| Remote backends | 10-20 min | 30 min | With container startup |

### 8.2 Inference Latency Budgets

| Backend | Model Size | Target | Acceptable | Fail |
|---------|------------|--------|------------|------|
| CPU (CI runner) | 0.5B Q4_K_S | 2-3s | 5s | >8s |
| CPU (CI runner) | 1.5B Q4_K_M | 5-8s | 10s | >15s |
| MLX (macOS runner) | 0.5B Q4_K_S | 0.5-1s | 2s | >3s |
| MLX (macOS runner) | 1.5B Q4_K_M | 1-2s | 3s | >5s |
| Ollama (container) | 0.5B | 1-2s | 5s | >10s |

### 8.3 Resource Usage Budgets

| Resource | Budget | Monitoring |
|----------|--------|------------|
| CI minutes (Linux) | 200 min/month | GitHub usage page |
| CI minutes (macOS) | 100 min/month (10x cost) | GitHub usage page |
| Model cache size | 1.5GB | GitHub cache usage |
| Peak memory (CI) | 8GB (Linux), 12GB (macOS) | Add memory profiling |
| Disk usage (models) | 2GB max | Cache cleanup policy |

---

## 9. Cost Analysis

### 9.1 GitHub Actions Costs

**Free Tier** (Public Repository):
- Linux: 2,000 minutes/month
- macOS: 1,000 minutes/month (10x multiplier = 100 actual minutes)
- Windows: 2,000 minutes/month

**Cost Projection** (with inference tests):

| Workflow | Frequency | Duration | Platform | Monthly Minutes | Cost |
|----------|-----------|----------|----------|-----------------|------|
| Fast CI (existing) | Per commit (~50/mo) | 15 min | Linux | 750 min | Free |
| CPU inference | Nightly (30/mo) | 30 min | Linux | 900 min | Free |
| MLX inference | Weekly (4/mo) | 20 min | macOS | 80 min (800 actual) | Free |
| Remote backends | Weekly (4/mo) | 20 min | Linux | 80 min | Free |
| **Total** | | | | **1,810 min Linux, 80 min macOS** | **Free** |

**Paid Usage** (if exceeding free tier):
- Linux: $0.008/min = ~$8/month for additional 1,000 minutes
- macOS: $0.08/min = ~$80/month for additional 1,000 actual minutes

**Recommendation**:
- ✅ CPU inference tests fit within free tier
- ✅ MLX tests should be weekly or on-demand to conserve macOS minutes
- ✅ Use self-hosted runners if heavy GPU testing needed

### 9.2 Self-Hosted Runner Costs

**Option 1: Cloud GPU Instance** (e.g., AWS g4dn.xlarge):
- Cost: ~$0.50/hour on-demand
- Usage: ~10 hours/month for testing
- Monthly: ~$5-10

**Option 2: Mac Mini (Apple Silicon)**:
- One-time: $599 (M2, 8GB)
- Power: ~$2-5/month
- Amortized: ~$25/month (2-year lifespan)

**Option 3: Linux with GPU** (e.g., NVIDIA GPU):
- Hardware: ~$500-1000 (one-time)
- Power: ~$10-20/month
- Amortized: ~$30-50/month

**Recommendation**:
- Start with GitHub Actions free tier
- Add self-hosted macOS runner if MLX testing exceeds 100 min/month
- Self-hosted becomes cost-effective at >1000 min/month usage

---

## 10. Acceptance Criteria

### 10.1 Phase 1 Completion

- [ ] Real Candle-based CPU inference working
- [ ] At least 10 quality validation tests passing
- [ ] Model download utility functional
- [ ] Feature flag `slow-tests` enables real inference
- [ ] Tests run locally with `cargo test --features slow-tests`

### 10.2 Phase 2 Completion

- [ ] Nightly inference workflow running successfully
- [ ] Model caching reduces subsequent runs by >90%
- [ ] Tests complete in <30 min with cache, <60 min without
- [ ] Quality report generated and uploaded as artifact
- [ ] PR comments with inference results (when triggered on PR)

### 10.3 Phase 3 Completion

- [ ] MLX backend with real Metal integration
- [ ] MLX tests running on macOS Apple Silicon runner
- [ ] Inference latency <2s for 1.5B model
- [ ] Performance benchmarks tracked in Criterion
- [ ] Regression detection for performance

### 10.4 Phase 4 Completion

- [ ] Ollama integration tests with Docker container
- [ ] vLLM simulation tests
- [ ] Fallback scenarios validated (Ollama failure → embedded)
- [ ] Network error handling tested
- [ ] Remote backend workflow running weekly

### 10.5 Production Readiness

- [ ] All inference tests passing for 2 consecutive weeks
- [ ] Pass rate >95% for quality validation tests
- [ ] No inference latency regressions detected
- [ ] CI costs within budget (<$20/month)
- [ ] Documentation updated with test strategy
- [ ] Team trained on interpreting quality reports

---

## 11. Future Enhancements

### 11.1 Advanced Testing (Post-v1.0)

- **Adversarial Testing**: Fuzzing with malicious prompts
- **Stress Testing**: Concurrent inference load testing
- **Model Comparison**: A/B testing different model versions
- **Golden Tests**: Snapshot testing for command outputs
- **Regression Detection**: Automatic alerting on quality degradation

### 11.2 Observability

- **Metrics Dashboard**: Grafana for CI metrics
- **Latency Tracking**: Historical inference performance
- **Quality Trends**: Pass rate over time
- **Cost Monitoring**: CI minute usage alerts

### 11.3 Optimization

- **Parallel Test Execution**: Run tests concurrently
- **Incremental Testing**: Only test changed backends
- **Smart Caching**: Cache by code hash, not just model version
- **Test Sharding**: Distribute tests across multiple runners

---

## 12. Quick Start Guide

### For Developers

**Run inference tests locally**:
```bash
# Download model (first time)
./scripts/setup-inference-model.sh

# Run all inference tests
cargo test --features slow-tests -- --ignored

# Run specific test
cargo test --features slow-tests -- --ignored test_cpu_inference

# Run with verbose output
RUST_LOG=debug cargo test --features slow-tests -- --ignored --nocapture
```

**Add new inference test**:
```rust
// tests/inference/quality.rs

#[tokio::test]
#[ignore]
#[cfg(feature = "slow-tests")]
async fn test_my_new_scenario() {
    let backend = setup_real_backend().await;
    let result = backend.generate("my prompt").await.unwrap();
    assert!(result.cmd.contains("expected"));
}
```

**Update test fixtures**:
```yaml
# tests/fixtures/prompts/my_category.yaml

test_cases:
  - id: "my-test"
    prompt: "my prompt"
    expected_patterns:
      - "pattern1"
      - "pattern2"
```

### For CI Administrators

**Trigger manual inference run**:
```bash
# Via GitHub CLI
gh workflow run inference-tests.yml

# Via GitHub UI
Actions → Inference Tests → Run workflow
```

**Check cache usage**:
```bash
# Via GitHub CLI
gh cache list

# Clear cache if needed
gh cache delete <cache-key>
```

**Monitor CI costs**:
```
Settings → Billing → Usage this month
```

---

## Appendix A: Model Specifications

### Qwen2.5-Coder-0.5B-Instruct-GGUF

- **Parameters**: 494M
- **Quantization**: Q4_K_S (4-bit)
- **File Size**: ~300MB
- **Context Length**: 32K tokens
- **Use Case**: CI testing, fast feedback
- **Performance**: 2-3s inference on 2 CPU cores

### Qwen2.5-Coder-1.5B-Instruct-GGUF

- **Parameters**: 1.54B
- **Quantization**: Q4_K_M (4-bit medium)
- **File Size**: ~1.1GB
- **Context Length**: 32K tokens
- **Use Case**: Production quality validation
- **Performance**: 5-8s inference on 2 CPU cores, <2s on Apple Silicon

---

## Appendix B: Troubleshooting

### Issue: Model download timeout

**Symptoms**: CI fails during model download after 5-10 minutes

**Solutions**:
1. Increase timeout: `timeout-minutes: 15` in workflow
2. Use mirror: `HF_ENDPOINT=https://hf-mirror.com`
3. Pre-download model and commit to LFS (not recommended)

### Issue: Out of memory during inference

**Symptoms**: Test crashes with "out of memory" error

**Solutions**:
1. Use smaller model (0.5B instead of 1.5B)
2. Reduce batch size in tests
3. Use swap space: `sudo swapon -s`
4. Upgrade to larger runner

### Issue: Cache not restoring

**Symptoms**: Model re-downloaded every run

**Solutions**:
1. Check cache key matches between save and restore
2. Verify cache not expired (7 days max)
3. Check cache size within 10GB limit
4. Use `restore-keys` for fallback matching

### Issue: Inference too slow

**Symptoms**: Tests timeout or take >10 min

**Solutions**:
1. Switch to 0.5B model for CI
2. Reduce number of test cases
3. Use GPU runner (macOS or self-hosted)
4. Profile and optimize model loading

---

**Document Version**: 1.0
**Last Updated**: 2025-11-01
**Owner**: @wildcard
**Status**: Ready for Implementation
