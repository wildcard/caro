
# Implementation Plan: Embedded Model + Remote Backend Support

**Branch**: `004-implement-ollama-and` | **Date**: 2025-10-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/workspaces/caro/specs/004-implement-ollama-and/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path ✅
   → Loaded from /workspaces/caro/specs/004-implement-ollama-and/spec.md
2. Fill Technical Context ✅
   → All technical decisions resolved via clarifications
3. Fill Constitution Check ✅
   → Based on constitution v1.0.0
4. Evaluate Constitution Check
   → Initial check: PASS (all principles satisfied)
5. Execute Phase 0 → research.md
   → Generate research for 8 technical decision areas
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, AGENTS.md
   → Design embedded model + remote backend architecture
7. Re-evaluate Constitution Check
   → Post-design check: PASS
8. Plan Phase 2 → Describe task generation approach
9. STOP - Ready for /tasks command
```

## Summary

caro will ship with an **embedded Qwen coding model** as the default backend, providing batteries-included, plug-and-play command generation that works offline out-of-box. Two build variants will be provided:

1. **MLX GPU build** (primary for Apple Silicon): Optimized for M1/M2/M3/M4 MacBook Pro using MLX framework
2. **CPU build** (cross-platform fallback): Using Burn or Candle inference runtime

Remote backends (Ollama and vLLM) are **optional enhancements** for power users seeking better quality or alternative models. The system automatically falls back to the embedded model if remote backends are unavailable.

**Core Architecture**: Embedded model (always available) → Ollama (optional local) → vLLM (optional remote)

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)

**Primary Dependencies**:
  - **Inference Runtimes**:
    - `mlx-rs` or FFI via `cxx` for MLX GPU acceleration (Apple Silicon)
    - `burn` or `candle-core` for CPU inference (cross-platform) - test both frameworks
    - `reqwest` 0.11+ (HTTP client for remote backends, already present)
  - **Async Runtime**: `tokio` 1.0+ (already present)
  - **Serialization**: `serde` + `serde_json` (already present)
  - **URL Parsing**: `url` crate (for remote backend URLs)
  - **Model Weights**: Qwen (Qwen2.5-Coder) as default; Phi-3 and StarCoder2 for CI/CD benchmarking

**Storage**:
  - Model weights embedded in binary or downloaded on first run (TBD in research)
  - Configuration: `~/.caro/config.toml` (already implemented)
  - Cache: HuggingFace cache directory for remote model downloads (Feature 003)

**Testing**:
  - `cargo test` with TDD RED-GREEN-REFACTOR methodology
  - Contract tests for backend trait implementations
  - Integration tests for end-to-end workflows
  - Property tests for safety validation with multiple backends
  - Performance benchmarks in GitHub Actions CI/CD

**Target Platform**:
  - **Primary**: macOS with Apple Silicon (M1/M2/M3/M4) using MLX GPU
  - **Secondary**: Linux x86_64, Windows x86_64 using CPU inference
  - Cross-platform binary distribution

**Project Type**: Single project with library-first architecture

**Performance Goals**:
  - **Embedded model (MLX GPU)**: <2s inference time on M1 Mac (FR-025)
  - **Embedded model (CPU)**: <5s inference acceptable for fallback
  - **Startup time**: <100ms including embedded model load (FR-027)
  - **Backend selection**: <500ms (FR-019)
  - **Safety validation**: <50ms P95

**Constraints**:
  - **Binary size**: <50MB excluding model weights (FR-028) - model size tracked separately
  - **Offline operation**: No network required for embedded model (FR-024, FR-031)
  - **Zero-config**: Works immediately after installation with embedded model (FR-012)
  - **Memory**: Efficient use of unified memory architecture on Apple Silicon
  - **Safety-first**: All commands validated regardless of backend source (FR-009)

**Scale/Scope**:
  - 3 backend implementations: EmbeddedModelBackend (MLX + CPU variants), OllamaBackend, VllmBackend
  - 2 build variants: MLX GPU (primary macOS), CPU (cross-platform fallback)
  - 3 model options tested: Qwen (default), Phi-3, StarCoder2
  - ~2,000 LOC new code (embedded backends + remote backends + config integration)
  - 9+ integration test scenarios from quickstart.md

**Embedded Model Implementation Details** (from clarifications):
  - **Default Model**: Qwen (Qwen2.5-Coder) as primary embedded model
  - **Alternative Models**: Phi-3 and StarCoder2 for CI/CD performance benchmarking
  - **Inference Runtime**: Custom Rust inference using Burn or Candle (test both frameworks)
  - **Platform Priority**: Apple Silicon (MLX GPU) optimized for latest MacBook Pros
  - **Build Matrix**: Two release tracks:
    1. **MLX GPU build** (Apple Silicon - primary/default for macOS)
    2. **CPU build** (cross-platform fallback using Burn/Candle)
  - **Testing Strategy**: GitHub Actions pipeline with model performance benchmarking
  - **Future Roadmap**: ONNX and GGUF format support (post-MVP)
  - **Core Vision**: Batteries-included, plug-and-play on macOS with Apple Silicon optimization

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Simplicity ✅
- **Single project structure**: Extending existing `src/backends/` module, no new projects
- **No new organizational patterns**: Reusing existing trait system (`CommandGenerator`)
- **Direct framework usage**: MLX via FFI, Burn/Candle direct integration, reqwest for HTTP
- **Minimal abstractions**: Each backend implements the existing `CommandGenerator` trait
- **YAGNI applied**: Embedded model is essential (batteries-included requirement), remote backends optional

### II. Library-First Architecture ✅
- **All features exported via `src/lib.rs`**:
  - `caro::backends::embedded` - Embedded model with MLX and CPU variants
  - `caro::backends::ollama` - Ollama HTTP API integration
  - `caro::backends::vllm` - vLLM HTTP API integration
  - `caro::backends::connection` - Shared HTTP connection logic
  - `caro::backends::retry` - Retry policy for resilience
- **Self-contained libraries**: Each backend independently testable
- **Clear public APIs**: `CommandGenerator` trait defines contract
- **Binary orchestrates**: `main.rs` selects backend, no business logic

### III. Test-First (NON-NEGOTIABLE) ✅
- **TDD cycle enforced**: Contract tests → Integration tests → Implementation
- **Strict ordering**:
  1. Write contract tests for `EmbeddedModelBackend`, `OllamaBackend`, `VllmBackend` (MUST FAIL)
  2. Write integration tests from quickstart.md scenarios (MUST FAIL)
  3. Implement backends to pass contract tests
  4. Verify integration tests pass
  5. Refactor for performance and safety
- **Real dependencies**: Actual MLX/Burn/Candle inference, real HTTP requests in tests (use `mockito` only for error scenarios)
- **Git commits**: Tests before implementation, granular commits per passing test

### IV. Safety-First Development ✅
- **Universal validation**: All commands validated regardless of backend (embedded, Ollama, vLLM) - FR-009
- **Embedded model safety**: Qwen output passes through same safety pipeline as remote backends
- **No unsafe Rust**: MLX FFI via `cxx` (safe), Burn/Candle safe APIs
- **Risk assessment**: High/Critical commands require confirmation even from embedded model
- **Property testing**: Validate safety across all three backends with random inputs

### V. Observability & Versioning ✅
- **Structured logging**:
  - `info!("Selected backend: {}", backend_type)` for user visibility
  - `debug!("Model inference latency: {}ms", duration)` for performance tracking
  - `warn!("Backend fallback: {} -> {}", failed, fallback)` for reliability
- **Error context**: `anyhow` for binary, `thiserror` for backend library errors
- **Performance instrumentation**: Log startup time, backend selection time, inference duration
- **Semantic versioning**: Feature 004 contributes to minor version bump (0.1.0 → 0.2.0)

**Initial Constitution Check**: ✅ PASS - All principles satisfied

## Project Structure

### Documentation (this feature)
```
specs/004-implement-ollama-and/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command) - EXISTS
├── contracts/           # Phase 1 output (/plan command) - PARTIAL (remote backends only)
│   ├── embedded-backend.md  # NEW: Contract for embedded model backend
│   ├── mlx-backend.md       # NEW: Contract for MLX GPU variant
│   ├── ollama-backend.md    # EXISTS: Update with fallback behavior
│   └── vllm-backend.md      # EXISTS: Update with fallback behavior
└── tasks.md             # Phase 2 output (/tasks command)
```

### Source Code (repository root)
```
src/
├── backends/
│   ├── mod.rs               # CommandGenerator trait (exists)
│   ├── embedded/            # NEW: Embedded model backend
│   │   ├── mod.rs          # Common embedded backend logic
│   │   ├── mlx.rs          # MLX GPU variant (Apple Silicon)
│   │   ├── cpu.rs          # CPU variant (Burn or Candle)
│   │   └── qwen.rs         # Qwen model-specific logic
│   ├── ollama.rs            # NEW: Ollama HTTP API backend
│   ├── vllm.rs              # NEW: vLLM HTTP API backend
│   ├── connection.rs        # NEW: Shared HTTP connection logic
│   └── retry.rs             # NEW: Retry policy
├── models/
│   └── mod.rs               # Extend with backend config types
├── config/
│   └── mod.rs               # Extend with backend selection logic
├── cli/
│   └── mod.rs               # Add --backend flag
└── lib.rs                   # Export new backend modules

tests/
├── contract/
│   ├── embedded_backend_contract.rs   # NEW: Embedded model contract tests
│   ├── mlx_backend_contract.rs        # NEW: MLX variant contract tests
│   ├── ollama_backend_contract.rs     # NEW: Ollama contract tests
│   └── vllm_backend_contract.rs       # NEW: vLLM contract tests
├── integration/
│   ├── backend_selection_test.rs      # NEW: Test CLI > Config > Auto-detect
│   ├── embedded_model_test.rs         # NEW: Test embedded model inference
│   └── fallback_test.rs               # NEW: Test remote → embedded fallback
└── property/
    └── multi_backend_safety_test.rs   # NEW: Property tests across all backends

.github/workflows/
├── model-benchmark.yml      # NEW: CI/CD model performance benchmarking
└── release.yml              # EXTEND: Multi-platform builds (MLX + CPU)
```

**Structure Decision**: Single project with library-first architecture. New `src/backends/embedded/` module for embedded model with platform-specific variants (MLX GPU, CPU fallback). Existing `src/backends/` module extended with Ollama and vLLM implementations. All backends implement the existing `CommandGenerator` trait for consistency.

## Phase 0: Outline & Research

**Goal**: Resolve 8 technical decision areas for embedded model + remote backend integration.

### Research Areas

**R1: Embedded Model Selection**
- **Research**: Compare Qwen2.5-Coder, Phi-3-mini, StarCoder2 for shell command generation
- **Criteria**: Inference speed, model size, code quality, shell command accuracy
- **Output**: Decision on default model (Qwen), alternatives for benchmarking

**R2: MLX Framework Integration**
- **Research**: MLX Rust bindings (`mlx-rs` vs FFI via `cxx`)
- **Criteria**: API stability, performance, unified memory handling, build complexity
- **Output**: Integration approach for Apple Silicon GPU acceleration

**R3: CPU Inference Runtime Comparison**
- **Research**: Burn vs Candle for cross-platform CPU inference
- **Criteria**: Qwen model support, inference speed, binary size impact, API ergonomics
- **Output**: Selected runtime (or both if testing required)

**R4: Model Distribution Strategy**
- **Research**: Embed weights in binary vs download on first run vs hybrid
- **Criteria**: Binary size (<50MB target), download time, offline operation, update mechanism
- **Output**: Distribution approach with trade-offs documented

**R5: Ollama API Integration**
- **Research**: Ollama `/api/generate` endpoint specification
- **Criteria**: Request/response format, error handling, model availability detection
- **Output**: HTTP client implementation approach (already covered in existing research.md)

**R6: vLLM API Integration**
- **Research**: vLLM OpenAI-compatible `/v1/completions` endpoint
- **Criteria**: Authentication (Bearer token), request format, error codes
- **Output**: HTTP client implementation approach (already covered in existing research.md)

**R7: Backend Selection Logic**
- **Research**: Priority order implementation (CLI > Config > Auto-detect)
- **Criteria**: Configuration schema, fallback behavior, user experience
- **Output**: Backend selection algorithm with configuration management

**R8: CI/CD Model Benchmarking**
- **Research**: GitHub Actions workflow for model performance testing
- **Criteria**: Benchmark metrics (latency, memory, accuracy), test harness design, reporting
- **Output**: CI/CD pipeline design for multi-model comparison

### Research Execution Plan

1. **Sequential Research** (order matters):
   - R1: Model selection (informs R2, R3)
   - R2: MLX integration (Apple Silicon primary platform)
   - R3: CPU runtime (cross-platform fallback)
   - R4: Distribution strategy (affects binary packaging)
   - R7: Backend selection (affects configuration design)
   - R8: CI/CD benchmarking (integrates all above decisions)

2. **Parallel Research** (independent):
   - R5: Ollama API (already partially covered)
   - R6: vLLM API (already partially covered)

3. **Consolidation**: Update existing `research.md` with R1-R4, R7-R8, integrate R5-R6

**Output**: `research.md` with 8 decision areas documented

## Phase 1: Design & Contracts

**Goal**: Define data model, API contracts, and integration test scenarios for embedded + remote backends.

### Step 1: Data Model Design

Generate `/workspaces/caro/specs/004-implement-ollama-and/data-model.md` with:

**New Entities**:
- **E1: EmbeddedModelBackend** (implements `CommandGenerator`)
  - Fields: `model_variant: ModelVariant` (enum: MLX, CPU), `model_name: String`, `config: EmbeddedConfig`
  - Methods: `generate_command`, `is_available`, `backend_info`, `shutdown`

- **E2: MlxBackend** (embedded model variant)
  - Fields: `model_path: PathBuf`, `context_size: usize`, `temperature: f32`
  - Methods: MLX-specific inference via FFI or `mlx-rs`

- **E3: CpuBackend** (embedded model variant)
  - Fields: `model_path: PathBuf`, `runtime: InferenceRuntime` (Burn or Candle)
  - Methods: CPU inference via selected runtime

- **E4: BackendConnection** (for remote backends)
  - Fields: `url: Url`, `client: reqwest::Client`, `backend_type: BackendType`, `health_status: HealthStatus`
  - Methods: `health_check`, `is_healthy`, `record_success`, `record_failure`

- **E5: RetryPolicy**
  - Fields: `max_attempts: u32`, `base_delay_ms: u64`, `backoff_multiplier: f32`
  - Methods: `delay_for_attempt`, `should_retry`

- **E6: OllamaBackend** (implements `CommandGenerator`)
  - Fields: `connection: BackendConnection`, `model: String`, `retry_policy: RetryPolicy`

- **E7: VllmBackend** (implements `CommandGenerator`)
  - Fields: `connection: BackendConnection`, `model: String`, `api_key: Option<String>`, `retry_policy: RetryPolicy`

**Supporting Types**:
- `ModelVariant` enum: `MLX`, `CPU`
- `BackendType` enum: `Embedded`, `Ollama`, `Vllm`
- `HealthStatus` enum: `Healthy`, `Degraded`, `Unavailable`
- `InferenceRuntime` enum: `Burn`, `Candle`
- Request/Response types for Ollama and vLLM APIs

### Step 2: API Contracts

Generate contracts in `/workspaces/caro/specs/004-implement-ollama-and/contracts/`:

**New Contracts**:
1. **embedded-backend.md**: Contract for `EmbeddedModelBackend`
   - MUST implement `CommandGenerator` trait
   - MUST work offline (no network calls)
   - MUST complete inference within 2s on M1 Mac (MLX variant)
   - MUST handle model loading errors gracefully
   - MUST support both MLX and CPU variants

2. **mlx-backend.md**: Contract for MLX GPU variant
   - MUST use unified memory architecture efficiently
   - MUST initialize within 100ms
   - MUST release resources on shutdown
   - MUST handle Metal framework errors

**Updated Contracts** (existing files):
3. **ollama-backend.md**: Update with fallback behavior
   - Add: MUST fallback to embedded model on connection failure
   - Add: MUST respect retry policy before fallback

4. **vllm-backend.md**: Update with fallback behavior
   - Add: MUST fallback to embedded model on connection failure
   - Add: MUST handle authentication failures gracefully

### Step 3: Contract Tests

Generate failing contract tests:
- `tests/contract/embedded_backend_contract.rs`
- `tests/contract/mlx_backend_contract.rs`
- `tests/contract/ollama_backend_contract.rs`
- `tests/contract/vllm_backend_contract.rs`

Each test file asserts:
- Trait implementation correctness
- Error handling behavior
- Performance requirements
- Resource cleanup

### Step 4: Integration Test Scenarios

Update `/workspaces/caro/specs/004-implement-ollama-and/quickstart.md` with new scenarios:

**New Scenarios**:
- **Scenario 1** (UPDATED): First-time user with embedded model (batteries-included experience)
- **Scenario 10**: MLX vs CPU backend performance comparison
- **Scenario 11**: Embedded model → Ollama fallback on user config
- **Scenario 12**: All remote backends unavailable, embedded model always works

**Existing Scenarios** (from previous planning):
- Scenarios 2-9: Ollama/vLLM testing with embedded fallback added

### Step 5: Update AGENTS.md

Run `.specify/scripts/bash/update-agent-context.sh claude` to update project guidance with:
- New MLX/Burn/Candle dependencies
- Embedded model architecture
- Multi-platform build strategy
- Recent changes: Feature 004 embedded model integration

**Output**:
- `data-model.md` (7 entities defined)
- `contracts/` (4 contract files: embedded, mlx, ollama-updated, vllm-updated)
- Failing contract tests (4 test files)
- `quickstart.md` (12 integration scenarios)
- `AGENTS.md` updated

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:

1. **Load base template**: `.specify/templates/tasks-template.md`

2. **Generate tasks from Phase 1 artifacts**:
   - **Embedded Model Setup** (Phase 4.0 - NEW):
     - Research Qwen model variants and quantization (T001)
     - Evaluate MLX Rust bindings vs FFI (T002)
     - Compare Burn vs Candle for CPU inference (T003)
     - Design model distribution strategy (T004)
     - Set up CI/CD model benchmarking pipeline (T005)

   - **TDD RED - Contract Tests** (Phase 4.1):
     - Write embedded backend contract tests (T006) [P]
     - Write MLX backend contract tests (T007) [P]
     - Write Ollama backend contract tests (T008) [P]
     - Write vLLM backend contract tests (T009) [P]
     - All tests MUST FAIL initially

   - **Supporting Types** (Phase 4.2):
     - Implement `ModelVariant`, `BackendType`, `InferenceRuntime` enums (T010) [P]
     - Implement `BackendConnection` struct (T011)
     - Implement `RetryPolicy` struct (T012)
     - Implement Ollama/vLLM request/response types (T013) [P]

   - **TDD GREEN - Embedded Model** (Phase 4.3):
     - Implement `EmbeddedModelBackend` (T014)
     - Implement `MlxBackend` (MLX GPU variant) (T015)
     - Implement `CpuBackend` (CPU variant - Burn or Candle) (T016)
     - Integrate Qwen model weights (T017)
     - Verify embedded model contract tests pass (T018)

   - **TDD GREEN - Remote Backends** (Phase 4.4):
     - Implement `OllamaBackend` (T019)
     - Implement `VllmBackend` (T020)
     - Add HTTP connection logic and retry (T021)
     - Verify remote backend contract tests pass (T022)

   - **Integration Tests** (Phase 4.5):
     - Write Scenario 1: First-time embedded model user (T023) [P]
     - Write Scenario 10: MLX vs CPU performance (T024) [P]
     - Write Scenario 11: Embedded → Ollama fallback (T025) [P]
     - Write Scenario 12: Offline embedded model (T026) [P]
     - Write Scenarios 2-9: Remote backends + fallback (T027-T034) [P]

   - **Config & CLI Integration** (Phase 4.6):
     - Extend UserConfiguration with backend fields (T035)
     - Implement backend selection logic (CLI > Config > Auto) (T036)
     - Add `--backend` CLI flag (T037)
     - Add `caro init` wizard for remote backend setup (T038)
     - Update configuration documentation (T039)

   - **Multi-Platform Build** (Phase 4.7):
     - Set up MLX GPU build (macOS Apple Silicon) (T040)
     - Set up CPU build (Linux/Windows fallback) (T041)
     - Configure GitHub Actions release matrix (T042)
     - Test binary size meets <50MB target (T043)

   - **Documentation & PR** (Phase 4.8):
     - Update README with embedded model architecture (T044)
     - Update CHANGELOG.md for Feature 004 (T045)
     - Create pull request with complete test coverage (T046)

3. **Task Ordering**:
   - **Strict TDD order**: Contract tests (T006-T009) before implementation (T014-T022)
   - **Dependency order**: Types (T010-T013) → Embedded model (T014-T018) → Remote backends (T019-T022)
   - **Parallel execution**: Mark [P] for independent tasks (different files/modules)

4. **Estimated Output**: ~46 numbered tasks in tasks.md

**Complexity Estimation**:
- Embedded model backends: ~800 LOC (MLX + CPU variants)
- Remote backends: ~600 LOC (Ollama + vLLM + connection logic)
- Configuration integration: ~200 LOC (backend selection + CLI flags)
- Tests: ~1,200 LOC (contract + integration + property tests)
- **Total**: ~2,800 LOC

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)
**Phase 4**: Implementation (execute tasks.md following TDD RED-GREEN-REFACTOR)
**Phase 5**: Validation (run tests, execute quickstart.md, performance benchmarks)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

No constitutional violations - complexity is justified:

| Added Complexity | Justification | Constitutional Alignment |
|------------------|---------------|-------------------------|
| Embedded model backend | **FR-001** requires batteries-included experience. Users must be able to use caro without external services. | ✅ Simplicity: Enables zero-config operation |
| Two build variants (MLX + CPU) | **FR-032-034** require optimal performance on Apple Silicon while maintaining cross-platform support. | ✅ Platform optimization justified by user base (macOS developers) |
| Three backend implementations | **FR-002-003** specify optional remote backends. Trait-based design keeps complexity contained. | ✅ Library-First: All backends implement `CommandGenerator` trait |
| CI/CD model benchmarking | **FR-036** requires testing Qwen vs Phi-3 vs StarCoder2 for optimal default model selection. | ✅ Test-First: Validates model performance before release |

**Net Complexity**: Moderate increase justified by batteries-included requirement and platform optimization goals.

## Progress Tracking

**Phase Status**:
- [x] Phase 0: Research complete ✅ (research.md with R0-R8 documented)
- [x] Phase 1: Design complete ✅ (data-model.md, contracts/, quickstart.md, AGENTS.md)
- [x] Phase 2: Task planning complete ✅ (approach described in plan.md)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS ✅
- [x] Post-Design Constitution Check: PASS ✅ (re-evaluated below)
- [x] All NEEDS CLARIFICATION resolved ✅ (via /clarify command)
- [x] Complexity deviations documented ✅ (see Complexity Tracking)
- [x] Research complete ✅ (9 decision areas: R0-R8)
- [x] Design artifacts complete ✅ (7 entities + 3 types, 4 contracts, 12 test scenarios)

**Post-Design Constitution Re-Check**:
- ✅ **Simplicity**: Embedded model adds one new module (`src/backends/embedded/`), maintains single project structure
- ✅ **Library-First**: All backends exported via `src/lib.rs`, `CommandGenerator` trait unifies interface
- ✅ **Test-First**: 4 contract files with comprehensive tests, 12 integration scenarios in quickstart.md
- ✅ **Safety-First**: All backends (embedded + remote) pass through same safety pipeline (FR-009)
- ✅ **Observability**: Structured logging for backend selection, fallback events, performance metrics

---
*Based on Constitution v1.0.0 - See `.specify/memory/constitution.md`*
