# Tasks: Implement Embedded Model + Remote Backend Support

**Feature**: 004-implement-ollama-and
**Status**: Ready for Implementation
**Generated**: 2025-10-14
**Prerequisites**: plan.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅, quickstart.md ✅

---

## Task Organization

Tasks are organized by implementation phase. Tasks marked with `[P]` can be executed in parallel with other `[P]` tasks in the same phase.

**Dependency Flow**:
1. Phase 4.0: Embedded Model Setup (research & decisions)
2. Phase 4.1: TDD RED - Contract Tests (MUST FAIL initially)
3. Phase 4.2: Supporting Types (enables both embedded + remote backends)
4. Phase 4.3: TDD GREEN - Embedded Model Implementation
5. Phase 4.4: TDD GREEN - Remote Backends Implementation
6. Phase 4.5: Integration Tests (end-to-end scenarios)
7. Phase 4.6: Config & CLI Integration
8. Phase 4.7: Multi-Platform Build Matrix
9. Phase 4.8: Documentation & PR

---

## Phase 4.0: Embedded Model Setup

### T001: Set Up MLX Rust Dependencies
**File**: `Cargo.toml`

**Tasks**:
1. Add `mlx-rs = "0.11"` dependency with `target.'cfg(all(target_os = "macos", target_arch = "aarch64"))'` platform gate
2. Add `candle-core = "0.7"`, `candle-transformers = "0.7"` for CPU inference
3. Add `tokenizers = "0.15"` for Qwen tokenizer support
4. Add `hf-hub = "0.3"` for Hugging Face model downloading
5. Verify compilation on both macOS (MLX available) and Linux (MLX unavailable)

**Success Criteria**:
- `cargo build` succeeds on macOS aarch64
- `cargo build` succeeds on Linux x86_64 (MLX code skipped via cfg)
- No compilation errors related to platform-specific code

**Dependencies**: None

---

### T002: Create Embedded Model Module Structure [P]
**Files**:
- `src/backends/embedded/mod.rs`
- `src/backends/embedded/mlx.rs`
- `src/backends/embedded/cpu.rs`

**Tasks**:
1. Create `src/backends/embedded/` directory
2. Create `mod.rs` with module exports:
   ```rust
   #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
   pub mod mlx;

   pub mod cpu;

   mod common;
   ```
3. Create placeholder `mlx.rs` with `#[cfg(all(target_os = "macos", target_arch = "aarch64"))]` gate
4. Create placeholder `cpu.rs` for Candle CPU implementation
5. Update `src/backends/mod.rs` to export `embedded` module

**Success Criteria**:
- Module structure compiles on all platforms
- `use crate::backends::embedded::*;` works in lib.rs
- MLX module only compiles on macOS aarch64

**Dependencies**: None

---

### T003: Download and Embed Qwen Model Metadata [P]
**Files**:
- `models/qwen2.5-coder-1.5b/README.md`
- `models/qwen2.5-coder-1.5b/config.json`
- `models/qwen2.5-coder-1.5b/tokenizer.json`

**Tasks**:
1. Create `models/` directory at project root
2. Download Qwen2.5-Coder-1.5B-Instruct Q4_K_M model metadata from Hugging Face
3. Embed `config.json` and `tokenizer.json` in repo (small files)
4. Add `.gitattributes` with `*.gguf filter=lfs` for future LFS support
5. Document model source and license in `models/qwen2.5-coder-1.5b/README.md`

**Success Criteria**:
- `models/qwen2.5-coder-1.5b/tokenizer.json` exists
- `models/qwen2.5-coder-1.5b/config.json` exists
- Model metadata can be loaded with `tokenizers` crate

**Dependencies**: None

---

### T004: Create Model Distribution Strategy
**File**: `src/model_loader.rs`

**Tasks**:
1. Create `ModelLoader` struct with methods:
   - `get_embedded_model_path() -> Result<PathBuf>` - returns bundled model path
   - `download_model_if_missing(variant: ModelVariant) -> Result<PathBuf>` - downloads from HF Hub
   - `detect_platform() -> ModelVariant` - returns `MLX` or `CPU` based on platform
2. Implement Hugging Face Hub API integration using `hf-hub` crate
3. Add progress bar for model downloads (optional enhancement)
4. Store downloaded models in `~/.cache/caro/models/`

**Success Criteria**:
- `ModelLoader::detect_platform()` returns correct variant
- Model download works when model file missing
- Offline operation works when model already cached

**Dependencies**: T001, T003

---

### T005: Set Up CI/CD Model Benchmarking Pipeline
**File**: `.github/workflows/model-benchmark.yml`

**Tasks**:
1. Create GitHub Actions workflow triggered on:
   - Push to `main` (performance regression detection)
   - Manual dispatch (on-demand benchmarking)
2. Run benchmarks for:
   - Qwen2.5-Coder-1.5B (Q4_K_M) - baseline
   - Phi-3-mini (Q4_K_M) - comparison
   - StarCoder2-3B (Q4_K_M) - comparison
3. Measure metrics:
   - Inference time (5 sample prompts)
   - Memory usage (RSS)
   - Binary size impact
   - Shell command accuracy (small test set)
4. Post results as PR comment or workflow summary

**Success Criteria**:
- Workflow runs successfully on push to main
- Benchmark results show Qwen2.5-Coder as fastest
- Results stored as GitHub Actions artifacts

**Dependencies**: T001, T002, T004

---

## Phase 4.1: TDD RED - Contract Tests (MUST FAIL)

### T006: Write Embedded Backend Contract Tests [P]
**File**: `tests/contracts/embedded_backend_contract.rs`

**Tasks**:
1. Create contract test file based on `specs/004-implement-ollama-and/contracts/embedded-backend.md`
2. Implement tests for all 9 contract requirements:
   - `test_offline_operation_no_network_calls` (CR-EMB-001)
   - `test_zero_config_immediate_availability` (CR-EMB-002)
   - `test_performance_targets_mlx_vs_cpu` (CR-EMB-003)
   - `test_platform_detection_automatic` (CR-EMB-004)
   - `test_safety_validator_integration` (CR-EMB-005)
   - `test_lazy_loading_on_first_inference` (CR-EMB-006)
   - `test_error_handling_model_load_failure` (CR-EMB-007)
   - `test_resource_cleanup_on_drop` (CR-EMB-008)
   - `test_thread_safe_concurrent_requests` (CR-EMB-009)
3. All tests MUST call non-existent methods (will fail compilation initially)

**Success Criteria**:
- File compiles after stubs are added in Phase 4.2
- All tests FAIL when run (implementation doesn't exist yet)
- Test structure follows contract specification exactly

**Dependencies**: None (can write tests before implementation exists)

---

### T007: Write MLX Backend Contract Tests [P]
**File**: `tests/contracts/mlx_backend_contract.rs`

**Tasks**:
1. Create contract test file based on `specs/004-implement-ollama-and/contracts/mlx-backend.md`
2. Implement tests for all 10 contract requirements:
   - `test_platform_restriction_macos_aarch64` (CR-MLX-001)
   - `test_unified_memory_usage` (CR-MLX-002)
   - `test_fast_initialization_under_100ms` (CR-MLX-003)
   - `test_inference_performance_under_2s` (CR-MLX-004)
   - `test_first_token_latency_under_200ms` (CR-MLX-005)
   - `test_metal_error_handling` (CR-MLX-006)
   - `test_gguf_q4_support` (CR-MLX-007)
   - `test_concurrent_request_handling` (CR-MLX-008)
   - `test_resource_cleanup_gpu` (CR-MLX-009)
   - `test_temperature_control` (CR-MLX-010)
3. Use `#[cfg(all(target_os = "macos", target_arch = "aarch64"))]` for platform-specific tests
4. All tests MUST FAIL initially (implementation doesn't exist)

**Success Criteria**:
- Tests only compile on macOS aarch64
- All tests FAIL when run (no implementation yet)
- Performance benchmarks configured correctly

**Dependencies**: None

---

### T008: Write Ollama Backend Contract Tests with Fallback [P]
**File**: `tests/contracts/ollama_backend_contract.rs`

**Tasks**:
1. Update existing Ollama contract tests (if they exist) or create new file
2. Add tests for new fallback behavior from `specs/004-implement-ollama-and/contracts/ollama-backend.md`:
   - `test_fallback_to_embedded_on_connection_failure` (FR-NEW-001)
   - `test_retry_before_fallback` (FR-NEW-002)
   - `test_optional_backend_status_non_blocking` (FR-NEW-003)
3. Keep existing Ollama functionality tests
4. All new fallback tests MUST FAIL initially

**Success Criteria**:
- Fallback tests fail (no fallback logic implemented yet)
- Tests verify seamless embedded model fallback
- Retry policy respected before fallback

**Dependencies**: None

---

### T009: Write vLLM Backend Contract Tests with Fallback [P]
**File**: `tests/contracts/vllm_backend_contract.rs`

**Tasks**:
1. Update existing vLLM contract tests or create new file
2. Add tests for new fallback behavior from `specs/004-implement-ollama-and/contracts/vllm-backend.md`:
   - `test_fallback_to_embedded_on_connection_failure` (FR-NEW-001)
   - `test_auth_failure_fallback_no_retry` (FR-NEW-002)
   - `test_https_warning_for_http_urls` (FR-NEW-005)
3. Keep existing vLLM functionality tests
4. All new fallback tests MUST FAIL initially

**Success Criteria**:
- Fallback tests fail (no fallback logic implemented yet)
- HTTPS enforcement tests configured
- Auth failure handling tests added

**Dependencies**: None

---

## Phase 4.2: Supporting Types

### T010: Implement ModelVariant Enum [P]
**File**: `src/backends/embedded/common.rs`

**Tasks**:
1. Create `ModelVariant` enum from data-model.md (T0):
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   pub enum ModelVariant {
       MLX,  // Apple Silicon GPU
       CPU,  // Cross-platform Candle
   }
   ```
2. Add `detect()` method that returns `MLX` on macOS aarch64, `CPU` otherwise
3. Add `Display` trait implementation for logging
4. Add tests for platform detection logic

**Success Criteria**:
- Enum compiles on all platforms
- `ModelVariant::detect()` returns correct variant
- Tests pass for platform detection

**Dependencies**: T002

---

### T011: Implement InferenceBackend Trait [P]
**File**: `src/backends/embedded/common.rs`

**Tasks**:
1. Create `InferenceBackend` trait from data-model.md (T1):
   ```rust
   #[async_trait]
   pub trait InferenceBackend: Send + Sync {
       async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError>;
       fn variant(&self) -> ModelVariant;
       async fn load(&mut self) -> Result<(), GeneratorError>;
       async fn unload(&mut self) -> Result<(), GeneratorError>;
   }
   ```
2. Document trait requirements in rustdoc comments
3. No implementation yet (implementations in Phase 4.3)

**Success Criteria**:
- Trait compiles
- Trait is `Send + Sync` for async usage
- Clear documentation for implementers

**Dependencies**: T010

---

### T012: Implement EmbeddedConfig Struct [P]
**File**: `src/backends/embedded/common.rs`

**Tasks**:
1. Create `EmbeddedConfig` struct from data-model.md (T2):
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct EmbeddedConfig {
       pub temperature: f32,
       pub max_tokens: usize,
       pub top_p: f32,
       pub stop_tokens: Vec<String>,
   }
   ```
2. Add `Default` implementation with reasonable values:
   - temperature: 0.7
   - max_tokens: 100
   - top_p: 0.9
   - stop_tokens: `["\n\n", "```"]`
3. Add builder methods for fluent API

**Success Criteria**:
- Struct compiles and serializes correctly
- Default values match FR-029 requirements
- Builder pattern works

**Dependencies**: None

---

### T013: Update CommandGenerator Trait for Embedded Model [P]
**File**: `src/backends/mod.rs`

**Tasks**:
1. Verify `CommandGenerator` trait supports embedded model:
   - `async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand, GeneratorError>`
   - `async fn is_available(&self) -> bool`
   - `fn backend_info(&self) -> BackendInfo`
   - `async fn shutdown(&self) -> Result<(), GeneratorError>`
2. Add `BackendType::Embedded` enum variant to `BackendInfo`
3. Update trait documentation with embedded model examples

**Success Criteria**:
- Trait signature unchanged (backwards compatible)
- `BackendType::Embedded` variant added
- Documentation reflects embedded model as primary backend

**Dependencies**: None

---

## Phase 4.3: TDD GREEN - Embedded Model Implementation

### T014: Implement EmbeddedModelBackend Struct
**File**: `src/backends/embedded/mod.rs`

**Tasks**:
1. Implement `EmbeddedModelBackend` struct from data-model.md (E0):
   ```rust
   pub struct EmbeddedModelBackend {
       model_variant: ModelVariant,
       model_path: PathBuf,
       backend: Arc<Mutex<Box<dyn InferenceBackend>>>,
       config: EmbeddedConfig,
   }
   ```
2. Implement constructor:
   ```rust
   pub fn new(variant: ModelVariant, model_path: PathBuf) -> Result<Self, GeneratorError>
   ```
3. Implement lazy loading: Model loaded on first `generate_command()` call, not in constructor
4. Add platform detection logic: Auto-select MLX on Apple Silicon, CPU elsewhere

**Success Criteria**:
- Constructor doesn't load model (lazy loading)
- Platform detection returns correct variant
- Struct compiles on all platforms

**Dependencies**: T004, T010, T011, T012

---

### T015: Implement CommandGenerator Trait for EmbeddedModelBackend
**File**: `src/backends/embedded/mod.rs`

**Tasks**:
1. Implement `CommandGenerator` trait for `EmbeddedModelBackend`:
   - `generate_command()`:
     - Lazy load model on first call
     - Call `backend.infer()` with system prompt + user input
     - Parse JSON response to extract command
     - Return `GeneratedCommand` with `backend_used: "embedded"`
   - `is_available()`: Always return `true` (offline operation)
   - `backend_info()`: Return `BackendInfo` with embedded model metadata
   - `shutdown()`: Call `backend.unload()` to release resources
2. Add error handling for model loading failures
3. Add logging with `tracing::debug!()` for inference timing

**Success Criteria**:
- Contract tests from T006 now PASS
- `generate_command()` completes in <2s on MLX, <5s on CPU
- `is_available()` returns `true` immediately (no I/O)

**Dependencies**: T014

---

### T016: Implement MlxBackend (Apple Silicon)
**File**: `src/backends/embedded/mlx.rs`

**Tasks**:
1. Implement `MlxBackend` struct from data-model.md (E1):
   ```rust
   #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
   pub struct MlxBackend {
       model: Option<Arc<MlxModel>>,
       tokenizer: Arc<Tokenizer>,
       model_path: PathBuf,
   }
   ```
2. Implement `InferenceBackend` trait:
   - `load()`: Load GGUF model via `mlx-rs`, initialize on Metal GPU
   - `infer()`: Tokenize → MLX forward pass → Decode tokens → Return string
   - `unload()`: Release GPU resources
   - `variant()`: Return `ModelVariant::MLX`
3. Use Metal unified memory architecture (zero-copy)
4. Add `#[cfg(all(target_os = "macos", target_arch = "aarch64"))]` guards

**Success Criteria**:
- MLX contract tests from T007 now PASS
- Initialization completes in <100ms (CR-MLX-003)
- Inference completes in <2s (CR-MLX-004)
- First token latency <200ms (CR-MLX-005)

**Dependencies**: T001, T011

---

### T017: Implement CpuBackend (Cross-Platform Fallback)
**File**: `src/backends/embedded/cpu.rs`

**Tasks**:
1. Implement `CpuBackend` struct from data-model.md (E2):
   ```rust
   pub struct CpuBackend {
       model: Option<Arc<CandleModel>>,
       tokenizer: Arc<Tokenizer>,
       model_path: PathBuf,
   }
   ```
2. Implement `InferenceBackend` trait using Candle:
   - `load()`: Load GGUF model via `candle-core` with `candle-transformers` Qwen2 support
   - `infer()`: Tokenize → Candle forward pass → Decode tokens → Return string
   - `unload()`: Release CPU resources
   - `variant()`: Return `ModelVariant::CPU`
3. Optimize for CPU inference (quantization awareness)
4. Works on all platforms (Linux, macOS, Windows)

**Success Criteria**:
- Embedded contract tests from T006 now PASS
- Inference completes in <5s on CPU (CR-EMB-003)
- Works on Linux x86_64 CI runners

**Dependencies**: T001, T011

---

### T018: Implement Embedded Model Integration Tests
**File**: `tests/integration/embedded_model_tests.rs`

**Tasks**:
1. Create integration tests for end-to-end embedded model workflows:
   - Test MLX backend on macOS (if available)
   - Test CPU backend on all platforms
   - Test lazy loading behavior
   - Test concurrent requests (thread safety)
   - Test offline operation (no network calls)
2. Use real Qwen model for testing (download via `ModelLoader`)
3. Measure and assert performance targets

**Success Criteria**:
- All integration tests PASS
- Tests run on CI for both macOS and Linux
- Performance assertions met

**Dependencies**: T014, T015, T016, T017

---

## Phase 4.4: TDD GREEN - Remote Backends Implementation

### T019: Implement Ollama Backend with Embedded Fallback [P]
**File**: `src/backends/ollama.rs`

**Tasks**:
1. Implement `OllamaBackend` struct from data-model.md (E5)
2. Add fallback logic to `generate_command()`:
   ```rust
   match self.http_request().await {
       Ok(response) => Ok(response),
       Err(e) => {
           warn!("Ollama backend failed: {}, falling back to embedded model", e);
           self.embedded_fallback.generate_command(request).await
       }
   }
   ```
3. Add `embedded_fallback: Arc<EmbeddedModelBackend>` field
4. Implement retry policy before fallback (up to 2 retries)
5. Update `is_available()` to return `false` when Ollama unreachable (non-blocking)

**Success Criteria**:
- Ollama contract tests from T008 now PASS
- Fallback to embedded model is seamless
- Retry policy respected before fallback

**Dependencies**: T015, T013

---

### T020: Implement vLLM Backend with Embedded Fallback [P]
**File**: `src/backends/vllm.rs`

**Tasks**:
1. Implement `VllmBackend` struct from data-model.md (E6)
2. Add fallback logic similar to Ollama (FR-NEW-001)
3. Add authentication failure handling (FR-NEW-002):
   - On 401/403, fallback immediately without retry
4. Add HTTPS enforcement warning (FR-NEW-005):
   - `warn!("Using HTTP for vLLM is insecure, use HTTPS in production")` if URL is `http://`
5. Add `embedded_fallback: Arc<EmbeddedModelBackend>` field

**Success Criteria**:
- vLLM contract tests from T009 now PASS
- Auth failures fallback without retry
- HTTPS warning logged for HTTP URLs

**Dependencies**: T015, T013

---

### T021: Implement Backend Selection Logic [P]
**File**: `src/backends/selector.rs`

**Tasks**:
1. Create `BackendSelector` struct with selection logic from research.md (R7):
   - Priority 1: CLI flag (`--backend ollama` or `--backend vllm`)
   - Priority 2: Config file (`~/.config/caro/config.toml`)
   - Priority 3: Embedded model (default fallback)
2. Implement `select_backend()` method:
   ```rust
   pub fn select_backend(
       cli_backend: Option<BackendType>,
       config: &Config,
   ) -> Result<Arc<dyn CommandGenerator>, GeneratorError>
   ```
3. Return embedded model wrapped with remote backend fallback if configured
4. Add health check for remote backends before selection

**Success Criteria**:
- CLI flag overrides config file
- Config file overrides default
- Embedded model always available as fallback
- Health checks prevent selecting unavailable backends

**Dependencies**: T019, T020

---

### T022: Implement Remote Backend Integration Tests [P]
**File**: `tests/integration/remote_backend_tests.rs`

**Tasks**:
1. Create integration tests for remote backends with fallback:
   - Test Ollama connection failure → embedded fallback
   - Test vLLM auth failure → embedded fallback
   - Test backend selection priority (CLI > Config > Embedded)
   - Test health check before backend selection
2. Use mock servers for Ollama/vLLM (no real services required)
3. Verify fallback seamlessly transitions to embedded model

**Success Criteria**:
- All integration tests PASS
- Mock servers simulate failures correctly
- Fallback behavior verified

**Dependencies**: T019, T020, T021

---

## Phase 4.5: Integration Tests (End-to-End Scenarios)

### T023: Implement Scenario 1 - First-Time User with Embedded Model
**File**: `tests/integration/test_scenario_01_first_time_user.rs`

**Tasks**:
1. Implement test from quickstart.md Scenario 1
2. Verify:
   - Fresh installation (no config, no remote backends)
   - First command generation uses embedded model
   - Works completely offline (no network calls)
   - Completes in <2s on MLX, <5s on CPU
3. Assert `backend_used == "embedded"` in result

**Success Criteria**:
- Test PASSES on both macOS (MLX) and Linux (CPU)
- No network calls made (verify with network mocking)
- Performance targets met

**Dependencies**: T015, T021

---

### T024-T034: Implement Scenarios 2-12 [P]
**Files**: `tests/integration/test_scenario_*.rs`

**Tasks** (for each scenario):
- T024: Scenario 2 - Ollama Integration
- T025: Scenario 3 - vLLM Integration
- T026: Scenario 4 - Safety Validation
- T027: Scenario 5 - Command Execution
- T028: Scenario 6 - Backend Switching
- T029: Scenario 7 - Offline Operation
- T030: Scenario 8 - Multi-Shell Support
- T031: Scenario 9 - Error Handling
- T032: Scenario 10 - MLX vs CPU Performance
- T033: Scenario 11 - Embedded Fallback
- T034: Scenario 12 - Complete Offline Operation

**Success Criteria (for each)**:
- Test implements corresponding scenario from quickstart.md
- All assertions pass
- Can run in parallel with other scenarios

**Dependencies**: T015, T019, T020, T021

---

## Phase 4.6: Config & CLI Integration

### T035: Update Config Schema for Embedded Model
**File**: `src/config/mod.rs`

**Tasks**:
1. Update `Config` struct to include:
   ```rust
   pub struct Config {
       pub default_backend: BackendType,  // "embedded", "ollama", "vllm"
       pub embedded: Option<EmbeddedBackendConfig>,
       pub ollama: Option<OllamaBackendConfig>,
       pub vllm: Option<VllmBackendConfig>,
       // ... existing fields
   }
   ```
2. Add `EmbeddedBackendConfig`:
   ```rust
   pub struct EmbeddedBackendConfig {
       pub variant: ModelVariant,  // Auto-detected by default
       pub model_path: Option<PathBuf>,  // Override default path
       pub temperature: f32,
       pub max_tokens: usize,
   }
   ```
3. Update config serialization/deserialization
4. Add migration logic for existing configs

**Success Criteria**:
- Config schema includes embedded model settings
- Backwards compatible with existing configs
- Default values match FR-029

**Dependencies**: T012

---

### T036: Implement `caro init` Command Updates
**File**: `src/cli/init.rs`

**Tasks**:
1. Update `caro init` to configure backends:
   - `caro init` - Set up with embedded model (default)
   - `caro init --backend ollama` - Configure Ollama + embedded fallback
   - `caro init --backend vllm` - Configure vLLM + embedded fallback
2. Interactive prompts for backend URLs, API keys
3. Update config file atomically
4. Verify backend availability during setup

**Success Criteria**:
- `caro init` creates valid config
- Interactive prompts work
- Backend validation during init

**Dependencies**: T035

---

### T037: Implement Backend CLI Flags
**File**: `src/cli/mod.rs`

**Tasks**:
1. Add `--backend` CLI flag to all commands:
   ```bash
   caro "list files" --backend embedded
   caro "list files" --backend ollama
   caro "list files" --backend vllm
   ```
2. Flag overrides config file setting (highest priority)
3. Add `--list-backends` command to show available backends:
   ```bash
   caro --list-backends
   # Output:
   # ✓ embedded (default, MLX GPU)
   # ✓ ollama (http://localhost:11434, healthy)
   # ✗ vllm (https://api.example.com, unreachable)
   ```
4. Update help text with backend options

**Success Criteria**:
- CLI flag works for backend selection
- `--list-backends` shows accurate status
- Help text clear and comprehensive

**Dependencies**: T021, T035

---

### T038: Update Main CLI Entry Point
**File**: `src/main.rs`

**Tasks**:
1. Update `main()` to initialize embedded model backend as default
2. Add backend selection logic using `BackendSelector`
3. Add graceful shutdown for all backends
4. Update error handling for embedded model failures

**Success Criteria**:
- Default backend is embedded model
- Backend selection works correctly
- Graceful shutdown releases resources

**Dependencies**: T015, T021, T037

---

### T039: Add Backend Status Command
**File**: `src/cli/status.rs`

**Tasks**:
1. Create `caro status` command showing:
   - Current backend (from config)
   - Embedded model variant (MLX or CPU)
   - Model path and size
   - Remote backend health (if configured)
   - Last successful inference time
2. Color-coded output (green = healthy, yellow = warning, red = error)
3. Performance statistics (average inference time)

**Success Criteria**:
- Status command shows accurate backend info
- Color-coded output works
- Useful for debugging

**Dependencies**: T021, T037

---

## Phase 4.7: Multi-Platform Build Matrix

### T040: Create MLX GPU Build Configuration
**File**: `.github/workflows/release-mlx.yml`

**Tasks**:
1. Create GitHub Actions workflow for MLX GPU builds:
   - Runs on: `macos-14` (M1 runner)
   - Target: `aarch64-apple-darwin`
   - Features: `mlx,embedded`
   - Binary name: `caro-mlx-aarch64-apple-darwin`
2. Bundle Qwen Q4_K_M model with binary (separate download)
3. Create `.dmg` installer for macOS
4. Upload artifacts to GitHub Releases

**Success Criteria**:
- MLX build succeeds on macOS M1 runner
- Binary size <50MB (excluding model)
- `.dmg` installer works

**Dependencies**: T016, T038

---

### T041: Create CPU Build Configuration
**File**: `.github/workflows/release-cpu.yml`

**Tasks**:
1. Create GitHub Actions workflow for CPU builds:
   - Matrix:
     - Linux: `x86_64-unknown-linux-gnu`
     - macOS: `x86_64-apple-darwin`, `aarch64-apple-darwin` (CPU fallback)
     - Windows: `x86_64-pc-windows-msvc`
   - Features: `candle,embedded`
   - Binary names: `caro-cpu-{target}`
2. Bundle Qwen Q4_K_M model (separate download)
3. Create installers:
   - `.tar.gz` for Linux
   - `.dmg` for macOS
   - `.msi` for Windows
4. Upload artifacts to GitHub Releases

**Success Criteria**:
- CPU builds succeed on all platforms
- Binary size <50MB (excluding model)
- Cross-platform installers work

**Dependencies**: T017, T038

---

### T042: Create Model Download Script
**File**: `scripts/download-model.sh`

**Tasks**:
1. Create shell script to download Qwen model post-install:
   ```bash
   #!/bin/bash
   # Download Qwen2.5-Coder-1.5B-Instruct Q4_K_M from Hugging Face
   # Save to ~/.cache/caro/models/
   ```
2. Show progress bar during download
3. Verify model checksum (SHA256)
4. Handle resume if download interrupted
5. Cross-platform support (bash for Unix, PowerShell for Windows)

**Success Criteria**:
- Model download works on all platforms
- Progress bar functional
- Checksum verification prevents corruption

**Dependencies**: T004

---

### T043: Update Release Documentation
**File**: `docs/RELEASE.md`

**Tasks**:
1. Document two release tracks:
   - **MLX GPU** (recommended for Apple Silicon)
   - **CPU** (cross-platform fallback)
2. Add installation instructions:
   - Homebrew formula (macOS)
   - Cargo install (all platforms)
   - GitHub Releases (manual download)
3. Add model download instructions
4. Add upgrade path documentation

**Success Criteria**:
- Release documentation complete
- Installation instructions tested on all platforms
- Clear guidance for users

**Dependencies**: T040, T041, T042

---

## Phase 4.8: Documentation & PR

### T044: Update README.md with Embedded Model
**File**: `README.md`

**Tasks**:
1. Update README with embedded model as primary feature:
   - "Zero-config, batteries-included shell command generation"
   - "Works completely offline with embedded Qwen model"
   - "Optional remote backends (Ollama, vLLM) for enhanced performance"
2. Add quick start section:
   ```bash
   # Install (includes embedded model)
   brew install caro  # or cargo install caro

   # First command (works immediately, no setup)
   caro "list all files larger than 100MB"
   ```
3. Update architecture diagram showing embedded model as core
4. Add performance comparison table (MLX vs CPU vs Ollama vs vLLM)

**Success Criteria**:
- README accurately reflects embedded model architecture
- Quick start section clear and concise
- Architecture diagram updated

**Dependencies**: T038

---

### T045: Create User Guide Documentation
**File**: `docs/USER_GUIDE.md`

**Tasks**:
1. Create comprehensive user guide:
   - Installation (all platforms)
   - First command generation
   - Backend selection (embedded vs remote)
   - Configuration (`caro init`)
   - Safety features
   - Troubleshooting
2. Add FAQs:
   - "Why is MLX faster than CPU?"
   - "How do I switch from embedded to Ollama?"
   - "Does caro work offline?"
3. Add screenshots and examples

**Success Criteria**:
- User guide covers all common scenarios
- FAQs answer typical questions
- Clear and beginner-friendly

**Dependencies**: T038

---

### T046: Create Pull Request with Feature 004
**File**: N/A (GitHub PR)

**Tasks**:
1. Create feature branch: `feature/004-embedded-model-backends`
2. Commit all changes with conventional commit messages:
   - `feat(embedded): Add Qwen embedded model support`
   - `feat(mlx): Add MLX GPU backend for Apple Silicon`
   - `feat(candle): Add Candle CPU backend for cross-platform`
   - `feat(backends): Add embedded model fallback for Ollama/vLLM`
   - `build: Add multi-platform build matrix (MLX + CPU)`
   - `docs: Update README with embedded model architecture`
3. Create PR with description:
   - Link to spec.md
   - Link to contracts/
   - Performance benchmarks
   - Test coverage report
4. Request reviews from maintainers

**Success Criteria**:
- All CI checks pass (tests, linting, formatting)
- Test coverage >90%
- PR description comprehensive
- Code review feedback addressed

**Dependencies**: All previous tasks

---

## Summary

**Total Tasks**: 46

**Parallel Execution Opportunities**:
- Phase 4.0: T002, T003 can run in parallel
- Phase 4.1: T006, T007, T008, T009 can run in parallel (RED tests)
- Phase 4.2: T010, T011, T012, T013 can run in parallel (supporting types)
- Phase 4.4: T019, T020, T021, T022 can run in parallel (remote backends)
- Phase 4.5: T024-T034 can run in parallel (integration tests)

**Critical Path**:
1. T001 (dependencies) → T004 (model loader)
2. T010-T013 (supporting types) → T014-T015 (embedded model)
3. T016 (MLX) + T017 (CPU) → T018 (integration tests)
4. T019-T020 (remote backends) → T021 (backend selector)
5. T035-T039 (config/CLI) → T040-T043 (build matrix)
6. T044-T046 (documentation + PR)

**Estimated Timeline**: 8-10 days with parallel execution

---

**Next Step**: Execute tasks in order, starting with Phase 4.0 (Embedded Model Setup)
