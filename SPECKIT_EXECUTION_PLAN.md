# Speckit Command Execution Plan - cmdai Production Roadmap

**Goal**: Implement cmdai production roadmap using spec-driven development
**Methodology**: GitHub spec-kit workflow
**Status**: Ready to Execute
**Created**: 2025-11-01

---

## Overview

This document provides a step-by-step execution plan using spec-kit slash commands to implement the cmdai production roadmap. Follow the commands in order for each phase.

## Spec-Driven Development Workflow

Each feature follows this cycle:
```
/specify → /clarify → /plan → /tasks → /analyze → /implement
```

---

## Phase 1: v1.0 Critical Path (Weeks 1-3)

### Feature 1.1: Contract Test Alignment (Week 1, Day 1-2)

**Priority**: P0 - BLOCKER
**Effort**: 4-8 hours
**Blocks**: CI/CD pipeline

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Fix contract test alignment in cmdai test suite.

Problem: Currently ~35 compilation errors exist in contract tests due to API
signature mismatches between test contracts and implemented APIs.

Affected Files:
- tests/contract/config_contract.rs
- tests/contract/logging_contract.rs
- src/config/mod.rs
- src/logging/mod.rs

Goal: Update test contracts to match the implemented API signatures without
breaking existing production code. All 44+ tests should compile and pass.

Requirements:
1. Analyze API signature mismatches between contracts and implementations
2. Update test contracts to use correct constructor signatures
3. Update test contracts to use correct method signatures
4. Ensure no breaking changes to production APIs
5. All tests must compile: cargo test --all-features
6. Document API decisions in CHANGELOG.md

Success Criteria:
- cargo test --all-features succeeds without compilation errors
- All 44+ tests pass
- No breaking API changes to production code
- CHANGELOG.md updated with fixes
"""

# Step 2: Clarify ambiguities
/clarify

# Step 3: Generate implementation plan
/plan

# Step 4: Generate actionable tasks
/tasks

# Step 5: Validate consistency
/analyze

# Step 6: Execute implementation
/implement
```

**Expected Output**:
- `.speckit/features/001-fix-contract-tests/spec.md`
- `.speckit/features/001-fix-contract-tests/plan.md`
- `.speckit/features/001-fix-contract-tests/tasks.md`
- Updated test files
- CHANGELOG.md entry

---

### Feature 1.2: Hugging Face Model Download (Week 1, Days 3-7 + Week 2, Days 1-2)

**Priority**: P0 - BLOCKER
**Effort**: 16-24 hours
**Enables**: Offline-first capability

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Implement Hugging Face model download functionality for cmdai.

Problem: Embedded backend cannot automatically download models from Hugging
Face Hub. Users must manually download models, breaking the "offline-first"
promise for new users.

Goal: Implement automatic model downloading with resume capability, checksum
validation, and progress tracking.

Components to Implement:

1. HF Downloader Module (src/cache/hf_download.rs):
   - HTTP client with reqwest
   - Range request support for resume
   - SHA256 checksum validation
   - Progress bar with indicatif
   - Retry logic with exponential backoff

2. Integration (src/backends/embedded/mod.rs):
   - ensure_model_available() method
   - First-run detection
   - Model path resolution

3. CLI UX (src/cli/mod.rs):
   - First-run welcome message
   - Download progress display
   - Cache location information

Model Targets:
- Primary: Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF (Q4_K_M, ~1.1GB)
- Fallback: Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF (Q4_K_S, ~400MB)

Requirements:
1. Download models from HF Hub API
2. Resume interrupted downloads (HTTP Range headers)
3. Validate checksums (SHA256)
4. Show progress bar with ETA
5. Handle network errors gracefully (retry 3x)
6. Cache models at ~/.cache/cmdai/models/
7. Update manifest after successful download

Success Criteria:
- Fresh install downloads model without manual intervention
- Download can be resumed after interruption
- Checksum validation prevents corrupted models
- Progress bar shows accurate ETA
- Downloaded model loads and generates commands
- Tests pass: cargo test hf_download

Technical Specifications:
- Use reqwest for HTTP client
- Use indicatif for progress bars
- Use sha2 for checksum validation
- HF API endpoint: https://huggingface.co/{repo}/resolve/{revision}/{filename}
- Cache format follows XDG Base Directory spec
"""

# Step 2: Clarify implementation details
/clarify

# Review clarifications, answer questions about:
# - Error handling strategies
# - Progress bar implementation
# - Retry backoff timing
# - Cache eviction policy

# Step 3: Generate implementation plan
/plan

# Step 4: Generate actionable tasks
/tasks

# Step 5: Validate design consistency
/analyze

# Step 6: Execute implementation
/implement
```

**Expected Output**:
- `.speckit/features/002-hf-model-download/spec.md`
- `.speckit/features/002-hf-model-download/plan.md`
- `.speckit/features/002-hf-model-download/tasks.md`
- `src/cache/hf_download.rs` (new file)
- Updated `src/backends/embedded/mod.rs`
- Updated `src/cli/mod.rs`
- Integration tests

---

### Feature 1.3: MLX Backend Optimization (Week 2, Days 3-7 + Week 3, Days 1-2)

**Priority**: P1 - HIGH
**Effort**: 8-16 hours
**Delivers**: Apple Silicon performance promise

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Implement real MLX backend with Metal Performance Shaders for Apple Silicon.

Problem: Current MLX backend only simulates inference with 100ms delay.
No actual Metal GPU acceleration is implemented.

Goal: Achieve <2s first inference on Apple Silicon M1/M2/M3 using real MLX
framework with Metal GPU acceleration.

Components to Implement:

1. C++ MLX Wrapper (src/backends/embedded/mlx_wrapper.cpp):
   - MLX model loading
   - Metal GPU stream initialization
   - Inference execution
   - Memory management
   - Warmup functionality

2. Rust FFI Bridge (src/backends/embedded/mlx_bridge.rs):
   - cxx bridge definition
   - Safe wrapper around C++ calls
   - Error handling
   - Resource cleanup

3. Backend Integration (src/backends/embedded/mlx.rs):
   - Replace simulated inference
   - Real model loading
   - Token generation
   - Response parsing

4. Build System (build.rs):
   - cxx-build configuration
   - Metal framework linking
   - Platform-specific compilation
   - Feature flag handling

Platform: macOS aarch64 only (Apple Silicon)

Requirements:
1. Load GGUF models using MLX
2. Initialize Metal Performance Shaders
3. Execute inference on GPU
4. Manage unified memory architecture
5. Achieve <2s first inference target
6. Graceful fallback to CPU backend on non-Apple hardware
7. Proper resource cleanup (no memory leaks)

Performance Targets:
- Model loading: <1s cold start
- First inference: <2s
- Subsequent inference: <500ms
- Memory usage: <2GB
- GPU utilization: >50%

Success Criteria:
- MLX backend compiles on macOS with Apple Silicon
- Model loading uses Metal GPU acceleration
- Inference meets performance targets
- cargo bench shows 3-5x speedup vs CPU backend
- No memory leaks (validated with instruments)
- Graceful fallback on non-Apple hardware

Technical Specifications:
- Use mlx-rs crate (0.25+)
- Use cxx for FFI (1.0+)
- Link Metal and MetalPerformanceShaders frameworks
- Conditional compilation: #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
- Build flags: -std=c++17, -framework Metal, -framework MetalPerformanceShaders
"""

# Step 2: Clarify technical details
/clarify

# Review clarifications about:
# - MLX API usage
# - Memory management strategy
# - Error handling approach
# - Fallback mechanism

# Step 3: Generate implementation plan
/plan

# Step 4: Generate actionable tasks
/tasks

# Step 5: Validate design
/analyze

# Step 6: Execute implementation
/implement
```

**Expected Output**:
- `.speckit/features/003-mlx-optimization/spec.md`
- `.speckit/features/003-mlx-optimization/plan.md`
- `.speckit/features/003-mlx-optimization/tasks.md`
- `src/backends/embedded/mlx_wrapper.cpp` (new file)
- `src/backends/embedded/mlx_wrapper.hpp` (new file)
- Updated `src/backends/embedded/mlx_bridge.rs`
- Updated `src/backends/embedded/mlx.rs`
- Updated `build.rs`
- Benchmark suite

---

### Feature 1.4: Binary Distribution (Week 3, Days 3-7)

**Priority**: P1 - HIGH
**Effort**: 8-12 hours
**Enables**: User installation

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Create binary distribution system for cmdai with multi-platform support.

Goal: Enable one-command installation of cmdai on Linux, macOS, and Windows
with automated binary builds and size optimization.

Components to Implement:

1. Release Workflow (.github/workflows/release.yml):
   - Multi-platform matrix builds
   - Binary stripping and optimization
   - Artifact uploads to GitHub Releases
   - Automated tagging on version bump

2. Installation Script (install.sh):
   - Platform detection (Linux, macOS, Windows)
   - Architecture detection (x64, ARM64)
   - Binary download from GitHub Releases
   - Installation to /usr/local/bin or user directory
   - Verification and version check

3. Package Manager Support:
   - Homebrew formula (cmdai.rb)
   - Cargo.toml publish configuration
   - AUR PKGBUILD (optional)

4. Binary Optimization:
   - Release profile optimization
   - Strip symbols
   - Size verification (<50MB target)

Platforms to Support:
- Linux x86_64 (GNU libc)
- Linux x86_64 (musl, static)
- Linux aarch64 (ARM64)
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

Requirements:
1. Automated builds on git tag push (v*.*.*)
2. Binary size <50MB after strip
3. Static linking where possible (musl)
4. Single-command installation
5. Cross-platform install script
6. Homebrew formula
7. Crates.io publishing

Success Criteria:
- Binaries build for all 6 platforms
- Binary size <50MB for all platforms
- install.sh works on Ubuntu 22.04, macOS 13+, Windows 11
- cmdai --version works immediately after install
- No external dependencies required
- Homebrew installation works: brew install wildcard/tap/cmdai

Distribution Channels:
- GitHub Releases (primary)
- Homebrew tap (macOS/Linux)
- Crates.io (Rust users)
- AUR (Arch Linux, optional)

Technical Specifications:
- Use cross for ARM64 Linux builds
- Release profile: opt-level="z", lto=true, strip=true
- Install location: /usr/local/bin (Unix), %LOCALAPPDATA%\Programs (Windows)
- One-liner install: curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash
"""

# Step 2: Clarify distribution details
/clarify

# Step 3: Generate implementation plan
/plan

# Step 4: Generate actionable tasks
/tasks

# Step 5: Validate design
/analyze

# Step 6: Execute implementation
/implement
```

**Expected Output**:
- `.speckit/features/004-binary-distribution/spec.md`
- `.speckit/features/004-binary-distribution/plan.md`
- `.speckit/features/004-binary-distribution/tasks.md`
- Updated `.github/workflows/release.yml`
- `install.sh` (new file)
- `cmdai.rb` (Homebrew formula)
- Updated `Cargo.toml` (publish config)

---

## Phase 2: v1.1 Real Inference Testing (Weeks 4-7)

### Feature 2.1: Real Candle CPU Backend (Week 4)

**Priority**: P2 - MEDIUM
**Effort**: 16-24 hours
**Enables**: Real inference validation

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Replace simulated CPU inference with real Candle framework integration.

Problem: Current CPU backend uses hardcoded responses with 800ms delay.
No actual LLM inference is performed.

Goal: Implement real model loading and inference using the Candle framework
for cross-platform CPU-based LLM inference.

Components to Implement:

1. Real Candle Integration (src/backends/embedded/cpu.rs):
   - Model loading with candle-core
   - Tokenization with candle-transformers
   - Inference execution
   - Response generation and parsing
   - Memory management

2. Model Loader (src/model_loader.rs):
   - GGUF file loading
   - Model weight initialization
   - Device selection (CPU)
   - Cache integration

3. Test Integration (tests/inference/cpu_inference.rs):
   - Real inference tests with #[ignore] flag
   - Quality validation against fixtures
   - Performance benchmarking
   - Feature flag: slow-tests

Model Format: GGUF (Qwen2.5-Coder-0.5B/1.5B)

Requirements:
1. Load GGUF models using Candle
2. Tokenize prompts with Qwen tokenizer
3. Execute forward pass for inference
4. Decode tokens to text response
5. Parse JSON from LLM output
6. Handle OOM errors gracefully
7. Achieve <5s inference on 2 CPU cores

Performance Targets:
- 0.5B model: 2-3s inference
- 1.5B model: 5-8s inference
- Memory usage: 2-4GB
- Startup time: <10s cold start

Success Criteria:
- Real model loads from GGUF file
- Inference generates valid commands
- Quality tests pass (>90% on basic.yaml)
- Performance within budget
- Tests run: cargo test --features slow-tests -- --ignored

Technical Specifications:
- Use candle-core for tensor operations
- Use candle-transformers for Qwen2 model
- Use tokenizers crate for tokenization
- Device: CPU (no GPU required)
- Precision: F32 or F16 depending on model
- Batch size: 1 (single command generation)
"""

# Step 2: Clarify implementation
/clarify

# Step 3: Generate plan
/plan

# Step 4: Generate tasks
/tasks

# Step 5: Validate design
/analyze

# Step 6: Execute implementation
/implement
```

**Expected Output**:
- `.speckit/features/005-real-candle-cpu/spec.md`
- `.speckit/features/005-real-candle-cpu/plan.md`
- `.speckit/features/005-real-candle-cpu/tasks.md`
- Updated `src/backends/embedded/cpu.rs`
- Updated `src/model_loader.rs`
- `tests/inference/cpu_inference.rs` (new file)
- `tests/inference/quality.rs` (new file)

---

### Feature 2.2: Inference Test Framework (Week 5)

**Priority**: P2 - MEDIUM
**Effort**: 12-16 hours
**Enables**: Quality validation

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Create comprehensive test framework for validating real LLM inference quality.

Goal: Implement test infrastructure that loads YAML fixtures, runs real
inference, validates patterns, and generates quality reports.

Components to Implement:

1. Test Utilities (tests/inference/mod.rs):
   - Fixture loading from YAML
   - Backend setup helpers
   - Pattern matching utilities
   - Quality reporting

2. Quality Validation (tests/inference/quality.rs):
   - Load test cases from fixtures
   - Execute inference for each test
   - Validate expected patterns present
   - Validate forbidden patterns absent
   - Check risk levels
   - Generate pass/fail report

3. Performance Tests (tests/inference/performance.rs):
   - Latency measurement
   - Throughput testing
   - Memory profiling
   - Benchmark integration

4. MLX Tests (tests/inference/mlx_inference.rs):
   - Platform-specific tests (#[cfg(target_os = "macos")])
   - GPU acceleration validation
   - Performance comparison vs CPU

Test Fixtures (Already Created):
- tests/fixtures/prompts/basic.yaml (24 test cases)
- tests/fixtures/prompts/dangerous.yaml (23 test cases)

Requirements:
1. Load and parse YAML test fixtures
2. Setup real backend with downloaded model
3. Execute inference for each test case
4. Validate using regex pattern matching
5. Generate detailed quality report
6. Support multiple backends (CPU, MLX)
7. Feature flags: slow-tests, remote-tests

Success Criteria:
- All fixture tests execute successfully
- Pattern matching works correctly
- Quality report generated in markdown
- Performance tests measure latency accurately
- Tests run: cargo test --features slow-tests -- --ignored
- Pass rate >95% on basic.yaml
- Block rate 100% on should_block tests

Technical Specifications:
- Use serde_yaml for fixture loading
- Use regex crate for pattern matching
- Use tokio-test for async testing
- Feature flag: #[cfg(feature = "slow-tests")]
- Ignore by default: #[ignore]
- Output format: Markdown quality report
"""

# Step 2: Clarify test framework
/clarify

# Step 3: Generate plan
/plan

# Step 4: Generate tasks
/tasks

# Step 5: Validate design
/analyze

# Step 6: Execute implementation
/implement
```

**Expected Output**:
- `.speckit/features/006-inference-test-framework/spec.md`
- `.speckit/features/006-inference-test-framework/plan.md`
- `.speckit/features/006-inference-test-framework/tasks.md`
- `tests/inference/mod.rs` (new file)
- Updated `tests/inference/quality.rs`
- `tests/inference/performance.rs` (new file)
- `tests/inference/mlx_inference.rs` (new file)

---

### Feature 2.3: Remote Backend Integration (Week 6)

**Priority**: P2 - MEDIUM
**Effort**: 16-20 hours
**Enables**: Ollama and vLLM support

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Implement real Ollama and vLLM backend integration with fallback mechanisms.

Problem: Remote backends (Ollama, vLLM) are currently placeholder implementations
with no actual HTTP communication.

Goal: Implement real HTTP API clients for Ollama and vLLM with automatic
fallback to embedded backend on failure.

Components to Implement:

1. Ollama Backend (src/backends/remote/ollama.rs):
   - HTTP client using reqwest
   - /api/generate endpoint integration
   - Model selection
   - Streaming support (optional)
   - Error handling and retry
   - Fallback to embedded backend

2. vLLM Backend (src/backends/remote/vllm.rs):
   - OpenAI-compatible API client
   - /v1/chat/completions endpoint
   - Bearer token authentication
   - Temperature and max_tokens configuration
   - Error handling
   - Fallback to embedded backend

3. Fallback Logic (src/backends/mod.rs):
   - Backend priority chain
   - Availability checking
   - Automatic fallback on failure
   - Timeout handling

4. Integration Tests (tests/remote/):
   - Ollama container integration
   - vLLM mock server tests
   - Fallback scenario validation
   - Network error simulation

API Specifications:

Ollama API:
- Endpoint: http://localhost:11434/api/generate
- Method: POST
- Headers: Content-Type: application/json
- Body: {"model": "qwen2.5-coder:0.5b", "prompt": "...", "stream": false}

vLLM API (OpenAI-compatible):
- Endpoint: http://localhost:8000/v1/chat/completions
- Method: POST
- Headers: Authorization: Bearer {api_key}, Content-Type: application/json
- Body: {"model": "...", "messages": [...]}

Requirements:
1. Implement HTTP clients with reqwest
2. Parse JSON responses
3. Handle network errors gracefully
4. Automatic retry (3 attempts with backoff)
5. Timeout configuration (30s default)
6. Fallback to embedded on failure
7. Connection pooling for efficiency
8. Tests with Docker containers (Ollama)

Success Criteria:
- Ollama backend connects to local instance
- vLLM backend connects to API endpoint
- Commands generate successfully via remote backends
- Fallback works when remote unavailable
- Network errors handled without panic
- Tests pass: cargo test --features remote-tests -- --ignored
- Docker integration works in CI

Technical Specifications:
- Use reqwest with tokio runtime
- Use serde_json for parsing
- Timeout: 30s per request
- Retry: 3 attempts with exponential backoff (1s, 2s, 4s)
- Feature flag: remote-backends
- Test feature flag: remote-tests
"""

# Step 2: Clarify integration details
/clarify

# Step 3: Generate plan
/plan

# Step 4: Generate tasks
/tasks

# Step 5: Validate design
/analyze

# Step 6: Execute implementation
/implement
```

**Expected Output**:
- `.speckit/features/007-remote-backend-integration/spec.md`
- `.speckit/features/007-remote-backend-integration/plan.md`
- `.speckit/features/007-remote-backend-integration/tasks.md`
- Updated `src/backends/remote/ollama.rs`
- Updated `src/backends/remote/vllm.rs`
- `tests/remote/ollama.rs` (new file)
- `tests/remote/vllm.rs` (new file)
- `tests/remote/fallback.rs` (new file)

---

### Feature 2.4: Quality Report Generator (Week 7)

**Priority**: P3 - LOW
**Effort**: 8-12 hours
**Enables**: CI quality tracking

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Create binary tool to generate quality reports from inference test results.

Goal: Parse test result JSON and generate markdown quality reports with
pass rates, latency metrics, and failed test details.

Component to Implement:

Binary: src/bin/generate-quality-report.rs

Features:
1. Parse test result JSON from stdin or file
2. Calculate aggregate metrics (pass rate, latency stats)
3. Generate markdown report with tables
4. Include failed test details
5. Support multiple output formats (markdown, JSON, HTML)
6. CLI argument parsing with clap

Input Format (JSON):
{
  "test_cases": [
    {
      "id": "test-id",
      "prompt": "prompt text",
      "generated_command": "command",
      "passed": true,
      "error": null,
      "latency_ms": 1234
    }
  ]
}

Output Format (Markdown):
# Inference Quality Report
## Summary
- Total Tests: 24
- Passed: 23 ✅
- Failed: 1 ❌
- Pass Rate: 95.8%
- Average Latency: 2,340ms

## Test Results
| Test ID | Prompt | Command | Status | Latency |
|---------|--------|---------|--------|---------|
| ... | ... | ... | ✅ | 2,340ms |

## Failed Tests
### test-id
**Prompt**: prompt text
**Generated**: `command`
**Error**: error message

Requirements:
1. Parse JSON test results
2. Calculate pass/fail counts and percentages
3. Calculate latency statistics (avg, p95, p99)
4. Generate markdown tables
5. List failed tests with details
6. Support --input and --output flags
7. Validate input JSON schema

Success Criteria:
- Parses valid test result JSON
- Generates well-formatted markdown
- Calculates correct statistics
- Handles malformed input gracefully
- CLI works: ./generate-quality-report --input results.json --output report.md
- Used in CI workflow successfully

Technical Specifications:
- Use clap for CLI parsing
- Use serde_json for JSON parsing
- Use anyhow for error handling
- Binary location: src/bin/generate-quality-report.rs
- Build: cargo build --bin generate-quality-report
"""

# Step 2: Clarify report format
/clarify

# Step 3: Generate plan
/plan

# Step 4: Generate tasks
/tasks

# Step 5: Validate design
/analyze

# Step 6: Execute implementation
/implement
```

**Expected Output**:
- `.speckit/features/008-quality-report-generator/spec.md`
- `.speckit/features/008-quality-report-generator/plan.md`
- `.speckit/features/008-quality-report-generator/tasks.md`
- `src/bin/generate-quality-report.rs` (new file)

---

## Phase 3: v1.0 Release Preparation (Week 8)

### Feature 3.1: Documentation Update

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Update all project documentation for v1.0 release.

Goal: Ensure README.md, QUICKSTART.md, and all documentation accurately
reflects the implemented features and provides clear installation and usage
instructions.

Files to Update:
1. README.md - Add installation, usage examples, features
2. QUICKSTART.md (new) - First-time user guide
3. TROUBLESHOOTING.md (new) - Common issues and solutions
4. CHANGELOG.md - Add v1.0 release notes
5. CONTRIBUTING.md - Update with contribution guidelines

Requirements:
1. Installation section with one-liner
2. Usage examples for all backends
3. Configuration documentation
4. Safety features explanation
5. Performance benchmarks
6. Troubleshooting guide
7. Release notes with all features

Success Criteria:
- README has clear installation instructions
- QUICKSTART covers first-time setup
- TROUBLESHOOTING addresses common issues
- CHANGELOG has complete v1.0 entry
- All examples tested and working
"""

# Step 2: Clarify documentation scope
/clarify

# Step 3: Generate plan
/plan

# Step 4: Generate tasks
/tasks

# Step 5: Validate documentation
/analyze

# Step 6: Execute updates
/implement
```

---

### Feature 3.2: Release Process

#### Command Sequence:

```bash
# Step 1: Create specification
/specify

# Provide natural language description:
"""
Execute v1.0 release process with proper tagging, binary publishing, and
announcement.

Steps:
1. Version bump in Cargo.toml
2. CHANGELOG.md finalization
3. Tag creation (v1.0.0)
4. GitHub Release creation
5. Binary artifact publishing
6. Crates.io publishing
7. Homebrew formula update
8. Announcement preparation

Requirements:
1. All tests passing
2. Binary size <50MB
3. Cross-platform builds successful
4. Documentation complete
5. CHANGELOG accurate
6. Security audit complete

Success Criteria:
- v1.0.0 tag created
- GitHub Release published with binaries
- Crates.io package published
- Homebrew formula available
- Announcement drafted
"""

# Remaining steps...
/clarify
/plan
/tasks
/analyze
/implement
```

---

## Command Execution Guidelines

### General Workflow

For each feature, execute commands in this order:

1. **`/specify`** - Create detailed specification
   - Provide comprehensive natural language description
   - Include all requirements, success criteria, technical specs
   - Reference related files and components

2. **`/clarify`** - Identify ambiguities
   - Review generated questions
   - Answer all clarifications
   - Encode answers back into spec

3. **`/plan`** - Generate implementation plan
   - Review generated design artifacts
   - Validate architecture decisions
   - Ensure alignment with constitution

4. **`/tasks`** - Generate actionable tasks
   - Review dependency-ordered task list
   - Validate task breakdown
   - Check time estimates

5. **`/analyze`** - Validate consistency
   - Cross-artifact analysis
   - Identify inconsistencies
   - Resolve conflicts

6. **`/implement`** - Execute implementation
   - Process tasks in dependency order
   - Write code following TDD when appropriate
   - Update documentation

### Between Features

After completing each feature:

```bash
# Commit changes
git add .
git commit -m "feat: implement [feature-name]"

# Run tests
cargo test --all-features

# Update CHANGELOG
# Document what was implemented

# Push changes
git push origin [branch-name]
```

### Quality Gates

Before moving to next feature:

- ✅ All tests passing
- ✅ Code reviewed (if team)
- ✅ Documentation updated
- ✅ CHANGELOG entry added
- ✅ No clippy warnings
- ✅ Formatted with cargo fmt

---

## Quick Reference

### Command Cheat Sheet

```bash
# Specification
/specify
# → Creates spec.md with detailed requirements

# Clarification
/clarify
# → Generates clarifying questions, encodes answers

# Planning
/plan
# → Creates plan.md with architecture and design

# Tasks
/tasks
# → Creates tasks.md with dependency-ordered actions

# Analysis
/analyze
# → Cross-artifact consistency check

# Implementation
/implement
# → Executes tasks, writes code

# Constitution (one-time)
/constitution
# → Define project principles and values
```

### Feature Numbering

Use sequential numbering:
- Feature 001: Contract Test Alignment
- Feature 002: HF Model Download
- Feature 003: MLX Optimization
- Feature 004: Binary Distribution
- Feature 005: Real Candle CPU
- Feature 006: Inference Test Framework
- Feature 007: Remote Backend Integration
- Feature 008: Quality Report Generator

### Directory Structure

```
.speckit/
├── constitution.md
└── features/
    ├── 001-fix-contract-tests/
    │   ├── spec.md
    │   ├── plan.md
    │   └── tasks.md
    ├── 002-hf-model-download/
    │   ├── spec.md
    │   ├── plan.md
    │   └── tasks.md
    └── ...
```

---

## Execution Timeline

### Week 1
- Feature 001: Contract Test Alignment (Days 1-2)
- Feature 002: HF Model Download (Days 3-7)

### Week 2
- Feature 002: HF Model Download (Days 1-2, completion)
- Feature 003: MLX Optimization (Days 3-7)

### Week 3
- Feature 003: MLX Optimization (Days 1-2, completion)
- Feature 004: Binary Distribution (Days 3-7)

**v1.0 Milestone** - End of Week 3

### Week 4
- Feature 005: Real Candle CPU Backend

### Week 5
- Feature 006: Inference Test Framework

### Week 6
- Feature 007: Remote Backend Integration

### Week 7
- Feature 008: Quality Report Generator

**v1.1 Milestone** - End of Week 7

### Week 8
- Documentation finalization
- Release preparation
- Security audit
- v1.0 release

---

## Success Metrics

### Per Feature
- [ ] Spec complete and reviewed
- [ ] All clarifications resolved
- [ ] Plan approved
- [ ] Tasks completed
- [ ] Consistency analysis passed
- [ ] Tests passing
- [ ] Documentation updated

### Per Phase
- [ ] All features implemented
- [ ] Integration tests passing
- [ ] Performance targets met
- [ ] Documentation complete

### Overall
- [ ] v1.0 released with all critical features
- [ ] v1.1 with real inference testing
- [ ] CI pipeline operational
- [ ] User adoption growing

---

## Next Steps

1. **Start with constitution** (if not exists):
   ```bash
   /constitution
   ```

2. **Begin Feature 001**:
   ```bash
   /specify
   # Provide contract test alignment description
   ```

3. **Follow workflow** for each feature

4. **Monitor progress** using tasks.md

5. **Ship v1.0** in 6-8 weeks

---

**Created**: 2025-11-01
**Status**: Ready to Execute
**Owner**: Development Team
**Expected Completion**: 8 weeks for v1.1

---

## Notes

- Each `/specify` command requires detailed natural language input
- `/clarify` is interactive - answer all questions thoroughly
- `/analyze` may identify issues - resolve before `/implement`
- `/implement` executes tasks - monitor progress
- Commit frequently between features
- Run tests after each feature
- Update CHANGELOG as you go

This plan provides a complete roadmap from specification to implementation using spec-driven development methodology. Execute commands in order for systematic, well-documented feature delivery.
