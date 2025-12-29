# Tasks: TDD GREEN Phase - Core Models and Safety System

**Feature**: 002-implement-tdd-green | **Branch**: `002-implement-tdd-green` | **Date**: 2025-10-01
**Input**: Design documents from `/workspaces/caro/specs/002-implement-tdd-green/`
**Prerequisites**: plan.md, research.md, data-model.md, quickstart.md (all available)

## Execution Flow

```
1. Load plan.md from feature directory
   → Loaded: Tech stack (Rust 1.75+, clap 4.4, tokio, async-trait, regex)
   → Extracted: 4 modules (models, safety, backends, CLI)
2. Load optional design documents:
   → data-model.md: 7 entities extracted
   → quickstart.md: Integration workflow identified
   → research.md: 5 technical decisions confirmed
3. Generate tasks by category:
   → Module A: Core Models (T001-T010)
   → Module B: Safety Validation (T011-T020)
   → Module C: Backend Trait (T021-T030)
   → Module D: CLI Interface (T031-T040)
   → Module E: Integration (T041-T050)
4. Apply task rules:
   → [P] marked for parallel tasks (different files, no dependencies)
   → Tests already exist from Feature 001 (TDD RED phase)
   → Implementation tasks make tests pass
5. Dependencies validated:
   → Models → (Safety | Backends) → CLI → Integration
   → Strict TDD: implement only what tests require
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[VERIFY]**: Milestone verification task (run tests, check coverage)
- Absolute file paths included in descriptions

## Test Pass Rate Tracking

| Milestone | Tasks | Tests Passing | % Complete | Notes |
|-----------|-------|---------------|------------|-------|
| Baseline | - | 0/80 | 0% | RED phase complete, all tests failing |
| After T010 | Models | ~0/80 | 0% | Types compile, tests can import |
| After T020 | Safety | ~17/80 | 21% | safety_validator_contract.rs passing |
| After T030 | Backends | ~28/80 | 35% | backend_trait_contract.rs passing |
| After T040 | CLI | ~42/80 | 52% | cli_interface_contract.rs passing |
| After T046 | Integration | 80/80 | 100% | **SUCCESS - All tests green** |

## Module A: Core Models (T001-T010)
**Foundation layer - No external dependencies, enables all other modules**

- [ ] **T001 [P]** Define ShellType enum in `/workspaces/caro/src/models/enums.rs`
  - **Description**: Create ShellType enum with variants (Bash, Zsh, Fish, Sh, PowerShell, Cmd, Unknown)
  - **Files**: Create `src/models/enums.rs`, update `src/models/mod.rs`
  - **Acceptance Criteria**:
    - Enum derives Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
    - Implement FromStr trait with lowercase matching
    - Implement Display trait for string representation
    - Add `detect()` method with platform-specific logic (Windows vs POSIX)
    - Add `is_posix()` and `is_windows()` helper methods
    - Implement Default trait calling `detect()`
  - **Dependencies**: None
  - **Test Verification**: `cargo test --lib models::enums` (should compile, no tests yet)
  - **Commit Message**: `[T001] Define ShellType enum with platform detection`

- [ ] **T002 [P]** Define RiskLevel enum in `/workspaces/caro/src/models/enums.rs`
  - **Description**: Create RiskLevel enum for safety risk categorization
  - **Files**: Update `src/models/enums.rs`, export in `src/models/mod.rs`
  - **Acceptance Criteria**:
    - Enum with variants: Safe, Moderate, High, Critical
    - Derives Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize
    - Ordering: Safe < Moderate < High < Critical (test with comparisons)
    - Implement Display trait with colored output (green, yellow, orange, red)
    - Add `requires_confirmation(SafetyLevel)` method
    - Add `is_blocked(SafetyLevel)` method for blocking logic
  - **Dependencies**: None (can run parallel with T001)
  - **Test Verification**: `cargo test --lib models::enums::risk_level`
  - **Commit Message**: `[T002] Define RiskLevel enum with ordering and confirmation logic`

- [ ] **T003 [P]** Define SafetyLevel enum in `/workspaces/caro/src/models/enums.rs`
  - **Description**: Create SafetyLevel enum for user safety preferences
  - **Files**: Update `src/models/enums.rs`, export in `src/models/mod.rs`
  - **Acceptance Criteria**:
    - Enum with variants: Strict, Moderate, Permissive
    - Derives Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
    - Implement Default trait returning Moderate
    - Document action matrix (Strict blocks High+, Moderate blocks Critical)
  - **Dependencies**: None (can run parallel with T001, T002)
  - **Test Verification**: `cargo test --lib models::enums::safety_level`
  - **Commit Message**: `[T003] Define SafetyLevel enum with default Moderate`

- [ ] **T004 [P]** Define BackendType enum in `/workspaces/caro/src/models/backend.rs`
  - **Description**: Create BackendType enum for backend categorization
  - **Files**: Create `src/models/backend.rs`, export in `src/models/mod.rs`
  - **Acceptance Criteria**:
    - Enum with variants: Mock, Ollama, VLlm, Mlx
    - Derives Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
    - Implement Display trait (lowercase names)
    - Document backend characteristics (features, use cases)
  - **Dependencies**: None (can run parallel with T001-T003)
  - **Test Verification**: `cargo test --lib models::backend::backend_type`
  - **Commit Message**: `[T004] Define BackendType enum for backend selection`

- [ ] **T005** Implement CommandRequest struct in `/workspaces/caro/src/models/request.rs`
  - **Description**: Create CommandRequest struct with builder pattern
  - **Files**: Create `src/models/request.rs`, export in `src/models/mod.rs`
  - **Acceptance Criteria**:
    - Struct fields: input (String), shell_type (ShellType), safety_level (SafetyLevel), context (Option<String>)
    - Derives Debug, Clone, Serialize, Deserialize
    - Implement `new(input, shell_type)` constructor with trim validation
    - Implement `with_safety(SafetyLevel)` builder method
    - Implement `with_context(String)` builder method
    - Validation: non-empty input after trimming
  - **Dependencies**: T001 (ShellType), T003 (SafetyLevel)
  - **Test Verification**: `cargo test --test integration_tests test_command_request_creation`
  - **Commit Message**: `[T005] Implement CommandRequest struct with builder pattern`

- [ ] **T006** Implement GeneratedCommand struct in `/workspaces/caro/src/models/response.rs`
  - **Description**: Create GeneratedCommand struct for command generation results
  - **Files**: Create `src/models/response.rs`, export in `src/models/mod.rs`
  - **Acceptance Criteria**:
    - Struct fields: command, explanation, safety_level (RiskLevel), estimated_impact, alternatives (Vec<String>), backend_used, generation_time_ms (u64), confidence_score (f64)
    - Derives Debug, Clone, Serialize, Deserialize
    - Validation: confidence_score in range [0.0, 1.0]
    - Validation: non-empty command string
    - All fields public for serialization
  - **Dependencies**: T002 (RiskLevel)
  - **Test Verification**: `cargo test --test integration_tests test_generated_command_structure`
  - **Commit Message**: `[T006] Implement GeneratedCommand struct with validation`

- [ ] **T007** Implement BackendInfo struct in `/workspaces/caro/src/models/backend.rs`
  - **Description**: Create BackendInfo struct for backend metadata
  - **Files**: Update `src/models/backend.rs`, export in `src/models/mod.rs`
  - **Acceptance Criteria**:
    - Struct fields: backend_type (BackendType), model_name, supports_streaming (bool), max_tokens (u32), typical_latency_ms (u64), memory_usage_mb (u64), version
    - Derives Debug, Clone, Serialize, Deserialize
    - Validation: positive numbers for metrics
    - Non-empty model_name and version strings
  - **Dependencies**: T004 (BackendType)
  - **Test Verification**: `cargo test --test backend_trait_contract test_backend_info_metadata`
  - **Commit Message**: `[T007] Implement BackendInfo struct for diagnostics`

- [ ] **T008** Add serde serialization support in `/workspaces/caro/src/models/mod.rs`
  - **Description**: Ensure all models serialize correctly to JSON/YAML
  - **Files**: Update `src/models/mod.rs` with re-exports and serde config
  - **Acceptance Criteria**:
    - All enums use `#[serde(rename_all = "lowercase")]`
    - Test JSON serialization of CommandRequest
    - Test JSON serialization of GeneratedCommand
    - Test YAML serialization of both structs
    - Verify round-trip (serialize → deserialize → equals original)
  - **Dependencies**: T001-T007 (all model types)
  - **Test Verification**: `cargo test --lib models::serde_tests`
  - **Commit Message**: `[T008] Add comprehensive serde serialization support`

- [ ] **T009** Implement Display/Debug traits in `/workspaces/caro/src/models/enums.rs`
  - **Description**: Add user-friendly string representations for enums
  - **Files**: Update `src/models/enums.rs` with Display implementations
  - **Acceptance Criteria**:
    - ShellType::Display shows lowercase name (bash, zsh, powershell)
    - RiskLevel::Display shows colored uppercase (SAFE in green, CRITICAL in red)
    - SafetyLevel::Display shows capitalized name (Strict, Moderate, Permissive)
    - BackendType::Display shows lowercase name (mock, ollama, vllm, mlx)
    - All Debug impls use derived implementations
  - **Dependencies**: T001-T004 (enum definitions)
  - **Test Verification**: `cargo test --lib models::display_tests`
  - **Commit Message**: `[T009] Implement Display traits with colored output`

- [ ] **T010 ✓ VERIFY** Milestone: Models compile and types are usable
  - **Description**: Verify all model types compile and export correctly
  - **Files**: Verify `src/models/mod.rs` exports, update `src/lib.rs`
  - **Acceptance Criteria**:
    - `cargo build` succeeds without warnings
    - All types importable from `caro::models`
    - Test imports: `use caro::models::{CommandRequest, GeneratedCommand, ShellType, RiskLevel, SafetyLevel, BackendInfo, BackendType};`
    - No clippy warnings: `cargo clippy --lib -- -D warnings`
    - Code formatted: `cargo fmt --check`
    - Update `src/lib.rs` to export `pub mod models;`
  - **Dependencies**: T001-T009 (all model tasks)
  - **Test Verification**: `cargo test --lib models` (should compile even if tests don't exist)
  - **Test Pass Rate**: ~0/80 (types compile, tests can import, but no tests passing yet)
  - **Commit Message**: `[T010] ✓ VERIFY: Models module complete and usable`

## Module B: Safety Validation (T011-T020)
**Depends on Models - Implements dangerous command detection and risk assessment**

- [ ] **T011** Create SafetyValidator struct skeleton in `/workspaces/caro/src/safety/mod.rs`
  - **Description**: Define SafetyValidator struct and ValidationResult type
  - **Files**: Create `src/safety/mod.rs`, update `src/lib.rs`
  - **Acceptance Criteria**:
    - SafetyValidator struct with `new()` constructor
    - ValidationResult struct: risk_level (RiskLevel), explanation (String), patterns_matched (Vec<String>)
    - ValidationResult derives Debug, Clone, Serialize
    - Add helper methods: `safe()`, `moderate(msg)`, `high(msg)`, `critical(msg)`
    - Export from `src/lib.rs`: `pub mod safety;`
  - **Dependencies**: T010 (models complete, RiskLevel available)
  - **Test Verification**: `cargo test --lib safety::validator`
  - **Commit Message**: `[T011] Create SafetyValidator struct skeleton`

- [ ] **T012 [P]** Define dangerous pattern database in `/workspaces/caro/src/safety/patterns.rs`
  - **Description**: Create lazy static regex patterns for dangerous commands
  - **Files**: Create `src/safety/patterns.rs`, update `src/safety/mod.rs`
  - **Acceptance Criteria**:
    - Use `once_cell::sync::Lazy` for static regex compilation
    - Define DANGEROUS_PATTERNS as `Vec<(Regex, &str, RiskLevel)>`
    - Include patterns:
      * `rm -rf /` → Critical (root filesystem deletion)
      * `rm -rf ~` → Critical (home directory deletion)
      * Fork bomb `:(){ :|:& };:` → Critical
      * `dd if=/dev/zero of=/dev/sda` → Critical (disk wipe)
      * `mkfs` → Critical (format filesystem)
      * `chmod 777 /` → High (insecure permissions)
      * Operations on `/bin`, `/usr`, `/etc` → High
      * `sudo su` → Moderate (privilege escalation)
      * Unquoted paths with spaces → Moderate
    - Total: 15+ patterns covering spec requirements
  - **Dependencies**: T010 (RiskLevel type), independent of T011 (can run parallel)
  - **Test Verification**: `cargo test --lib safety::patterns::compilation`
  - **Commit Message**: `[T012] Define dangerous command pattern database`

- [ ] **T013** Implement ValidationResult type in `/workspaces/caro/src/safety/mod.rs`
  - **Description**: Complete ValidationResult with helper constructors
  - **Files**: Update `src/safety/mod.rs`
  - **Acceptance Criteria**:
    - ValidationResult::safe() returns Safe with empty explanation
    - ValidationResult::moderate(msg) returns Moderate with message
    - ValidationResult::high(msg) returns High with message
    - ValidationResult::critical(msg) returns Critical with message
    - Add `is_safe()`, `is_dangerous()` helper methods
    - Implement Display for user-friendly output
  - **Dependencies**: T011 (ValidationResult struct defined)
  - **Test Verification**: `cargo test --lib safety::validation_result`
  - **Commit Message**: `[T013] Implement ValidationResult helper methods`

- [ ] **T014** Implement validate() method in `/workspaces/caro/src/safety/mod.rs`
  - **Description**: Core validation logic with pattern matching
  - **Files**: Update `src/safety/mod.rs`, use `src/safety/patterns.rs`
  - **Acceptance Criteria**:
    - Method signature: `pub fn validate(&self, cmd: &str, shell: ShellType) -> ValidationResult`
    - Iterate through DANGEROUS_PATTERNS, match against command
    - Return first Critical match immediately
    - Track all High/Moderate matches, return highest risk
    - Delegate to platform-specific validators (posix vs windows)
    - Empty commands return ValidationResult with moderate risk
  - **Dependencies**: T012 (patterns), T013 (ValidationResult)
  - **Test Verification**: `cargo test --test safety_validator_contract test_basic_dangerous_patterns`
  - **Commit Message**: `[T014] Implement core validate() method with pattern matching`

- [ ] **T015** Add POSIX-specific validation in `/workspaces/caro/src/safety/validators.rs`
  - **Description**: Implement validate_posix() for bash/zsh/fish/sh shells
  - **Files**: Create `src/safety/validators.rs`, update `src/safety/mod.rs`
  - **Acceptance Criteria**:
    - Function: `pub fn validate_posix(cmd: &str, shell: ShellType) -> ValidationResult`
    - Check unquoted paths with spaces (moderate risk)
    - Detect command substitution `$(...)` and backticks (high risk)
    - Check variable expansion edge cases
    - Validate POSIX compliance (no bash-specific features for `sh`)
    - Proper quote escaping validation
  - **Dependencies**: T014 (validate method structure)
  - **Test Verification**: `cargo test --test safety_validator_contract test_posix_validation`
  - **Commit Message**: `[T015] Add POSIX-specific command validation`

- [ ] **T016** Add Windows-specific validation in `/workspaces/caro/src/safety/validators.rs`
  - **Description**: Implement validate_windows() for PowerShell and cmd.exe
  - **Files**: Update `src/safety/validators.rs`
  - **Acceptance Criteria**:
    - Function: `pub fn validate_windows(cmd: &str, shell: ShellType) -> ValidationResult`
    - Windows path validation (C:\, unquoted paths)
    - PowerShell execution policy bypass detection (high risk)
    - cmd.exe batch script detection
    - Remove-Item -Recurse dangerous patterns
    - Format-Volume and diskpart detection (critical)
  - **Dependencies**: T015 (validators.rs exists)
  - **Test Verification**: `cargo test --test safety_validator_contract test_windows_validation`
  - **Commit Message**: `[T016] Add Windows-specific command validation`

- [ ] **T017** Implement custom pattern loading in `/workspaces/caro/src/safety/mod.rs`
  - **Description**: Allow users to add custom dangerous patterns
  - **Files**: Update `src/safety/mod.rs`
  - **Acceptance Criteria**:
    - Method: `pub fn add_pattern(&mut self, pattern: &str, description: &str, risk: RiskLevel) -> Result<()>`
    - Compile regex and store in instance-specific pattern list
    - Validate regex syntax, return error on invalid patterns
    - Merge custom patterns with static DANGEROUS_PATTERNS during validation
    - Document pattern syntax (regex rules, examples)
  - **Dependencies**: T014 (validate method)
  - **Test Verification**: `cargo test --test safety_validator_contract test_custom_patterns`
  - **Commit Message**: `[T017] Implement custom safety pattern loading`

- [ ] **T018 ✓ VERIFY** Safety contract tests pass (17/17)
  - **Description**: Verify all safety_validator_contract.rs tests pass
  - **Files**: Run tests in `/workspaces/caro/tests/safety_validator_contract.rs`
  - **Acceptance Criteria**:
    - All 17 tests in safety_validator_contract.rs pass
    - Test dangerous pattern detection (rm -rf /, fork bombs)
    - Test POSIX and Windows validation
    - Test custom pattern loading
    - Test risk level assessment correctness
    - No test failures or panics
  - **Dependencies**: T011-T017 (all safety implementation)
  - **Test Verification**: `cargo test --test safety_validator_contract`
  - **Test Pass Rate**: ~17/80 (21% - safety tests passing)
  - **Commit Message**: `[T018] ✓ VERIFY: Safety validator contract tests pass (17/17)`

- [ ] **T019 ✓ VERIFY** Property tests for safety validation
  - **Description**: Verify property-based tests for safety invariants
  - **Files**: Run relevant tests in `/workspaces/caro/tests/property_tests.rs`
  - **Acceptance Criteria**:
    - Property: Safe commands always return RiskLevel::Safe
    - Property: Commands with "rm -rf" always return High or Critical
    - Property: Empty commands never panic
    - Property: All commands validate in <50ms (performance requirement)
    - Run 1000+ randomized inputs per property
  - **Dependencies**: T018 (safety tests pass)
  - **Test Verification**: `cargo test --test property_tests safety_properties`
  - **Commit Message**: `[T019] ✓ VERIFY: Property tests validate safety invariants`

- [ ] **T020 ✓ BENCHMARK** Validation performance <50ms
  - **Description**: Verify safety validation meets performance targets
  - **Files**: Run benchmarks in `/workspaces/caro/tests/performance_tests.rs`
  - **Acceptance Criteria**:
    - Single validation call <50ms (after first call)
    - First validation <100ms (includes regex compilation)
    - 1000 validation calls <5 seconds (5ms average)
    - Memory usage <100MB for validator instance
    - Benchmark with complex commands (long strings, many patterns)
  - **Dependencies**: T019 (property tests pass)
  - **Test Verification**: `cargo test --test performance_tests safety_validation_performance`
  - **Test Pass Rate**: Still ~17/80 (benchmarks don't add to pass count)
  - **Commit Message**: `[T020] ✓ BENCHMARK: Safety validation performance verified`

## Module C: Backend Trait System (T021-T030)
**Depends on Models - Implements async command generation trait**

- [ ] **T021** Define CommandGenerator trait in `/workspaces/caro/src/backends/mod.rs`
  - **Description**: Create async trait for command generation
  - **Files**: Create `src/backends/mod.rs`, update `src/lib.rs`
  - **Acceptance Criteria**:
    - Use `#[async_trait]` macro from async-trait crate
    - Trait bounds: Send + Sync (for async runtime)
    - Method: `async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand, Box<dyn Error>>`
    - Method: `async fn is_available(&self) -> bool`
    - Method: `fn backend_info(&self) -> BackendInfo` (sync, returns metadata)
    - Method: `async fn shutdown(&self) -> Result<(), Box<dyn Error>>` (cleanup)
    - Document trait usage and implementation guide
  - **Dependencies**: T010 (models available)
  - **Test Verification**: `cargo build --lib` (trait compiles)
  - **Commit Message**: `[T021] Define CommandGenerator async trait`

- [ ] **T022** Define GeneratorError enum in `/workspaces/caro/src/backends/error.rs`
  - **Description**: Create typed errors for backend failures
  - **Files**: Create `src/backends/error.rs`, export in `src/backends/mod.rs`
  - **Acceptance Criteria**:
    - Use thiserror for error derivation
    - Error variants:
      * InvalidRequest(String) - bad input
      * BackendUnavailable - backend not responding
      * GenerationFailed(String) - model error
      * TimeoutError - exceeded latency limit
      * ConfigError(#[from] std::io::Error) - configuration issues
    - Implement Display with user-friendly messages
    - Implement From<GeneratorError> for anyhow::Error
  - **Dependencies**: T021 (backends module exists)
  - **Test Verification**: `cargo test --lib backends::error`
  - **Commit Message**: `[T022] Define GeneratorError enum with thiserror`

- [ ] **T023** Implement error conversions in `/workspaces/caro/src/backends/error.rs`
  - **Description**: Add From trait implementations for error types
  - **Files**: Update `src/backends/error.rs`
  - **Acceptance Criteria**:
    - `impl From<std::io::Error> for GeneratorError`
    - `impl From<serde_json::Error> for GeneratorError`
    - `impl From<GeneratorError> for Box<dyn Error>`
    - Test error conversions with `?` operator
    - Preserve error context through conversion chain
  - **Dependencies**: T022 (GeneratorError defined)
  - **Test Verification**: `cargo test --lib backends::error::conversions`
  - **Commit Message**: `[T023] Implement error conversions for GeneratorError`

- [ ] **T024** Create MockBackend struct in `/workspaces/caro/src/backends/mock.rs`
  - **Description**: Implement mock backend for testing
  - **Files**: Create `src/backends/mock.rs`, export in `src/backends/mod.rs`
  - **Acceptance Criteria**:
    - Struct: `pub struct MockBackend { instance_id: String }`
    - Constructor: `pub fn new(instance_id: impl Into<String>) -> Self`
    - Store instance_id for backend_info identification
    - No external dependencies (pure in-memory mock)
    - Thread-safe (no mutable state)
  - **Dependencies**: T021 (CommandGenerator trait)
  - **Test Verification**: `cargo test --lib backends::mock::construction`
  - **Commit Message**: `[T024] Create MockBackend struct skeleton`

- [ ] **T025** Implement CommandGenerator trait for MockBackend
  - **Description**: Implement trait methods for MockBackend
  - **Files**: Update `src/backends/mock.rs`
  - **Acceptance Criteria**:
    - All CommandGenerator methods implemented
    - No actual LLM calls (return deterministic mock responses)
    - Trait bounds satisfied (Send + Sync)
    - Async methods use `async { ... }` blocks
    - Return realistic GeneratedCommand instances
  - **Dependencies**: T024 (MockBackend struct)
  - **Test Verification**: `cargo build --lib` (trait implementation compiles)
  - **Commit Message**: `[T025] Implement CommandGenerator trait for MockBackend`

- [ ] **T026** Implement async generate_command() in `/workspaces/caro/src/backends/mock.rs`
  - **Description**: Core command generation logic for mock
  - **Files**: Update `src/backends/mock.rs`
  - **Acceptance Criteria**:
    - Parse request.input to determine mock response
    - "list" → "ls -la", "find" → "find . -name", "delete" → dangerous command
    - Simulate latency with `tokio::time::sleep(150ms)` for realism
    - Return GeneratedCommand with mock explanation, alternatives
    - Set confidence_score based on input clarity (0.8-0.95)
    - Track generation_time_ms accurately
    - Handle empty input with InvalidRequest error
  - **Dependencies**: T025 (trait implemented)
  - **Test Verification**: `cargo test --test backend_trait_contract test_basic_command_generation`
  - **Commit Message**: `[T026] Implement async generate_command() with mock logic`

- [ ] **T027** Implement is_available() and backend_info() in `/workspaces/caro/src/backends/mock.rs`
  - **Description**: Metadata and availability methods
  - **Files**: Update `src/backends/mock.rs`
  - **Acceptance Criteria**:
    - is_available() always returns true (mock always available)
    - backend_info() returns BackendInfo with:
      * backend_type: BackendType::Mock
      * model_name: format!("mock-backend-v1-{}", instance_id)
      * supports_streaming: false
      * max_tokens: 512
      * typical_latency_ms: 150
      * memory_usage_mb: 50
      * version: "1.0.0"
  - **Dependencies**: T026 (generate_command implemented)
  - **Test Verification**: `cargo test --test backend_trait_contract test_backend_info_metadata`
  - **Commit Message**: `[T027] Implement is_available() and backend_info() methods`

- [ ] **T028** Implement shutdown() method in `/workspaces/caro/src/backends/mock.rs`
  - **Description**: Cleanup method for graceful shutdown
  - **Files**: Update `src/backends/mock.rs`
  - **Acceptance Criteria**:
    - shutdown() returns Ok(()) immediately (no resources to clean)
    - Add logging: `tracing::info!("MockBackend shutdown: {}", instance_id)`
    - Async method (matches trait signature)
    - No panics or errors
  - **Dependencies**: T027 (other methods complete)
  - **Test Verification**: `cargo test --test backend_trait_contract test_backend_shutdown`
  - **Commit Message**: `[T028] Implement shutdown() method for cleanup`

- [ ] **T029 ✓ VERIFY** Backend contract tests pass (11/11)
  - **Description**: Verify all backend_trait_contract.rs tests pass
  - **Files**: Run tests in `/workspaces/caro/tests/backend_trait_contract.rs`
  - **Acceptance Criteria**:
    - All 11 tests in backend_trait_contract.rs pass
    - Test async command generation
    - Test backend availability checking
    - Test backend_info metadata correctness
    - Test error handling (empty input, invalid requests)
    - Test shutdown process
  - **Dependencies**: T021-T028 (all backend implementation)
  - **Test Verification**: `cargo test --test backend_trait_contract`
  - **Test Pass Rate**: ~28/80 (35% - backend + safety tests passing)
  - **Commit Message**: `[T029] ✓ VERIFY: Backend trait contract tests pass (11/11)`

- [ ] **T030 ✓ VERIFY** Error handling tests for backends
  - **Description**: Verify error scenarios handled correctly
  - **Files**: Run relevant tests in `/workspaces/caro/tests/error_handling_tests.rs`
  - **Acceptance Criteria**:
    - Test empty input returns InvalidRequest error
    - Test error message clarity (user-friendly Display)
    - Test error serialization (for logging)
    - Test error conversion from io::Error
    - Test error propagation through async chain
  - **Dependencies**: T029 (backend tests pass)
  - **Test Verification**: `cargo test --test error_handling_tests backend_errors`
  - **Commit Message**: `[T030] ✓ VERIFY: Backend error handling tests pass`

## Module D: CLI Interface (T031-T040)
**Depends on ALL modules - Integrates everything into user interface**

- [ ] **T031** Define CliApp struct in `/workspaces/caro/src/cli/mod.rs`
  - **Description**: Create main CLI application structure
  - **Files**: Create `src/cli/mod.rs`, update `src/lib.rs`
  - **Acceptance Criteria**:
    - Use clap derive macros: `#[derive(Parser, Debug)]`
    - Struct fields: command (Commands enum), format (Option<OutputFormat>), log_level (Option<String>)
    - Add metadata: name, about, version attributes
    - Global flags: --format, --log-level
    - Export from `src/lib.rs`: `pub mod cli;`
  - **Dependencies**: T010, T020, T030 (all modules available)
  - **Test Verification**: `cargo build --lib` (CLI struct compiles)
  - **Commit Message**: `[T031] Define CliApp struct with clap derive`

- [ ] **T032** Create Clap argument structures in `/workspaces/caro/src/cli/args.rs`
  - **Description**: Define subcommands and output format enum
  - **Files**: Create `src/cli/args.rs`, update `src/cli/mod.rs`
  - **Acceptance Criteria**:
    - Commands enum with variants:
      * Generate { input: String, shell: Option<ShellType>, safety: Option<SafetyLevel> }
      * Validate { command: String }
    - OutputFormat enum: Json, Yaml, Text (derives ValueEnum)
    - Implement FromStr for custom value parsing
    - Add help documentation for each argument
    - Environment variable support: CARO_LOG_LEVEL
  - **Dependencies**: T031 (CLI struct)
  - **Test Verification**: `cargo test --lib cli::args::parsing`
  - **Commit Message**: `[T032] Create Clap argument structures with subcommands`

- [ ] **T033** Implement GenerateCommand handler in `/workspaces/caro/src/cli/handlers.rs`
  - **Description**: Handle "caro generate" subcommand
  - **Files**: Create `src/cli/handlers.rs`, update `src/cli/mod.rs`
  - **Acceptance Criteria**:
    - Async function: `pub async fn handle_generate(...) -> Result<GeneratedCommand>`
    - Create CommandRequest from input, shell, safety arguments
    - Initialize MockBackend (for now, later: backend selection)
    - Call backend.generate_command(request)
    - Validate result with SafetyValidator
    - Return GeneratedCommand for formatting
  - **Dependencies**: T032 (Commands enum)
  - **Test Verification**: `cargo test --lib cli::handlers::generate`
  - **Commit Message**: `[T033] Implement GenerateCommand handler logic`

- [ ] **T034** Implement ValidateCommand handler in `/workspaces/caro/src/cli/handlers.rs`
  - **Description**: Handle "caro validate" subcommand
  - **Files**: Update `src/cli/handlers.rs`
  - **Acceptance Criteria**:
    - Function: `pub fn handle_validate(command: &str) -> Result<ValidationResult>`
    - Detect shell type with ShellType::detect()
    - Initialize SafetyValidator
    - Call validator.validate(command, shell)
    - Return ValidationResult for formatting
  - **Dependencies**: T033 (handlers.rs exists)
  - **Test Verification**: `cargo test --test cli_interface_contract test_validate_command`
  - **Commit Message**: `[T034] Implement ValidateCommand handler logic`

- [ ] **T035** Create OutputFormat enum in `/workspaces/caro/src/cli/output.rs`
  - **Description**: Define output format handlers
  - **Files**: Create `src/cli/output.rs`, update `src/cli/mod.rs`
  - **Acceptance Criteria**:
    - OutputFormat enum: Json, Yaml, Text
    - Trait: `pub trait Formatter { fn format(&self, data: &impl Serialize) -> Result<String>; }`
    - Implement Formatter for each OutputFormat variant
    - Default format: Text (human-readable)
  - **Dependencies**: T032 (OutputFormat in args)
  - **Test Verification**: `cargo test --lib cli::output::format_enum`
  - **Commit Message**: `[T035] Create OutputFormat enum with Formatter trait`

- [ ] **T036** Implement JSON formatter in `/workspaces/caro/src/cli/output.rs`
  - **Description**: JSON output formatting with serde_json
  - **Files**: Update `src/cli/output.rs`
  - **Acceptance Criteria**:
    - Function: `pub fn format_json<T: Serialize>(data: &T) -> Result<String>`
    - Use serde_json::to_string_pretty for readable output
    - Handle serialization errors gracefully
    - Validate JSON output is parseable
    - Test with GeneratedCommand and ValidationResult
  - **Dependencies**: T035 (output.rs exists)
  - **Test Verification**: `cargo test --test cli_interface_contract test_json_output_format`
  - **Commit Message**: `[T036] Implement JSON formatter with serde_json`

- [ ] **T037** Implement YAML formatter in `/workspaces/caro/src/cli/output.rs`
  - **Description**: YAML output formatting with serde_yaml
  - **Files**: Update `src/cli/output.rs`
  - **Acceptance Criteria**:
    - Function: `pub fn format_yaml<T: Serialize>(data: &T) -> Result<String>`
    - Use serde_yaml::to_string for output
    - Handle serialization errors gracefully
    - Validate YAML output is parseable
    - Test with GeneratedCommand and ValidationResult
  - **Dependencies**: T036 (formatters pattern established)
  - **Test Verification**: `cargo test --test cli_interface_contract test_yaml_output_format`
  - **Commit Message**: `[T037] Implement YAML formatter with serde_yaml`

- [ ] **T038** Implement plain text formatter in `/workspaces/caro/src/cli/output.rs`
  - **Description**: Human-readable text output with colored formatting
  - **Files**: Update `src/cli/output.rs`
  - **Acceptance Criteria**:
    - Function: `pub fn format_text_command(cmd: &GeneratedCommand) -> String`
    - Function: `pub fn format_text_validation(result: &ValidationResult) -> String`
    - Use colored crate for RiskLevel highlighting
    - Format: "Generated: <command>\nSafety: <level>\nExplanation: <text>"
    - Include alternatives and metadata in verbose mode
    - Clean, readable layout matching quickstart.md examples
  - **Dependencies**: T037 (formatters)
  - **Test Verification**: `cargo test --test cli_interface_contract test_text_output_format`
  - **Commit Message**: `[T038] Implement plain text formatter with colors`

- [ ] **T039 ✓ VERIFY** CLI interface contract tests pass (14/14)
  - **Description**: Verify all cli_interface_contract.rs tests pass
  - **Files**: Run tests in `/workspaces/caro/tests/cli_interface_contract.rs`
  - **Acceptance Criteria**:
    - All 14 tests in cli_interface_contract.rs pass
    - Test argument parsing (generate, validate subcommands)
    - Test output format selection (JSON, YAML, text)
    - Test shell type and safety level parsing
    - Test handler integration with backends and validators
    - Test error messages for invalid arguments
  - **Dependencies**: T031-T038 (all CLI implementation)
  - **Test Verification**: `cargo test --test cli_interface_contract`
  - **Test Pass Rate**: ~42/80 (52% - models, safety, backends, CLI passing)
  - **Commit Message**: `[T039] ✓ VERIFY: CLI interface contract tests pass (14/14)`

- [ ] **T040 ✓ VERIFY** Output format validation
  - **Description**: Verify all output formats produce valid, parseable output
  - **Files**: Run format validation tests
  - **Acceptance Criteria**:
    - JSON output parses with `serde_json::from_str`
    - YAML output parses with `serde_yaml::from_str`
    - Text output contains all required fields
    - Round-trip test: struct → JSON → struct (equality)
    - Round-trip test: struct → YAML → struct (equality)
  - **Dependencies**: T039 (CLI tests pass)
  - **Test Verification**: `cargo test --test cli_interface_contract output_roundtrip`
  - **Commit Message**: `[T040] ✓ VERIFY: Output format validation complete`

## Module E: Integration & Validation (T041-T050)
**Final assembly - Wire everything together and validate full system**

- [ ] **T041** Export all public types in `/workspaces/caro/src/lib.rs`
  - **Description**: Configure public API exports for library users
  - **Files**: Update `src/lib.rs` with all module exports
  - **Acceptance Criteria**:
    - Export: `pub mod models;`
    - Export: `pub mod safety;`
    - Export: `pub mod backends;`
    - Export: `pub mod cli;`
    - Add crate-level documentation with examples
    - Document feature flags (if any)
    - Add module-level re-exports for common types
  - **Dependencies**: T010, T020, T030, T040 (all modules complete)
  - **Test Verification**: `cargo doc --lib --no-deps` (documentation builds)
  - **Commit Message**: `[T041] Export all public types in lib.rs`

- [ ] **T042** Implement main.rs with CLI initialization
  - **Description**: Binary entry point with tokio runtime
  - **Files**: Update `/workspaces/caro/src/main.rs`
  - **Acceptance Criteria**:
    - Use `#[tokio::main]` for async runtime
    - Parse CLI arguments with `CliApp::parse()`
    - Initialize tracing subscriber for logging
    - Match on Commands enum, call appropriate handlers
    - Format output based on --format flag
    - Handle errors with anyhow context chains
    - Return appropriate exit codes (0 success, 1 error)
  - **Dependencies**: T041 (lib exports ready)
  - **Test Verification**: `cargo build --release` (binary builds)
  - **Commit Message**: `[T042] Implement main.rs with CLI initialization`

- [ ] **T043** Add tracing subscriber initialization in `/workspaces/caro/src/main.rs`
  - **Description**: Configure structured logging with env filter
  - **Files**: Update `src/main.rs`
  - **Acceptance Criteria**:
    - Use tracing-subscriber with env filter
    - Default log level: WARN (quiet by default)
    - Support CARO_LOG_LEVEL env variable (DEBUG, INFO, WARN, ERROR)
    - Support --log-level CLI flag (overrides env)
    - Log format: timestamp, level, target, message
    - Initialize before any other operations
  - **Dependencies**: T042 (main.rs structure)
  - **Test Verification**: `CARO_LOG_LEVEL=debug cargo run -- --help` (logs visible)
  - **Commit Message**: `[T043] Add tracing subscriber initialization`

- [ ] **T044** Wire backend selection logic in `/workspaces/caro/src/cli/handlers.rs`
  - **Description**: Add backend selection (Mock only for now)
  - **Files**: Update `src/cli/handlers.rs`
  - **Acceptance Criteria**:
    - Function: `fn select_backend() -> Box<dyn CommandGenerator>`
    - For now: always return MockBackend::new("cli")
    - Add TODO comment for future backend selection (config file)
    - Document backend selection strategy in comments
    - Log selected backend: `tracing::info!("Using backend: {}", backend.backend_info().model_name)`
  - **Dependencies**: T043 (tracing configured)
  - **Test Verification**: `cargo test --lib cli::handlers::backend_selection`
  - **Commit Message**: `[T044] Wire backend selection logic (Mock only)`

- [ ] **T045 ✓ VERIFY** Integration tests pass (8/8)
  - **Description**: Verify all integration_tests.rs tests pass
  - **Files**: Run tests in `/workspaces/caro/tests/integration_tests.rs`
  - **Acceptance Criteria**:
    - All 8 integration tests pass
    - Test full command generation workflow (quickstart)
    - Test safety validation with dangerous commands
    - Test cross-platform validation (POSIX vs Windows)
    - Test error propagation through pipeline
    - Test output formatting end-to-end
  - **Dependencies**: T044 (all integration complete)
  - **Test Verification**: `cargo test --test integration_tests`
  - **Test Pass Rate**: ~72/80 (90% - integration tests passing, some property/performance may still fail)
  - **Commit Message**: `[T045] ✓ VERIFY: Integration tests pass (8/8)`

- [ ] **T046 ✓ VERIFY** Full test suite passes (80/80 target)
  - **Description**: Verify ALL contract tests pass
  - **Files**: Run complete test suite
  - **Acceptance Criteria**:
    - `cargo test` shows 80/80 tests passing (100%)
    - No test failures or panics
    - No ignored tests
    - Test output clean (no warnings in test code)
    - Coverage includes:
      * backend_trait_contract.rs (11 tests)
      * safety_validator_contract.rs (17 tests)
      * cli_interface_contract.rs (14 tests)
      * integration_tests.rs (8 tests)
      * property_tests.rs (10 tests)
      * error_handling_tests.rs (11 tests)
      * performance_tests.rs (9 tests)
  - **Dependencies**: T045 (integration tests pass)
  - **Test Verification**: `cargo test --all-targets`
  - **Test Pass Rate**: **80/80 (100% - SUCCESS!)**
  - **Commit Message**: `[T046] ✓ VERIFY: Full test suite passes (80/80 - GREEN phase complete!)`

- [ ] **T047 ✓ BENCHMARK** Startup time <100ms
  - **Description**: Verify CLI startup performance target
  - **Files**: Benchmark binary with hyperfine
  - **Acceptance Criteria**:
    - Build release binary: `cargo build --release`
    - Benchmark: `hyperfine './target/release/caro --version'`
    - Target: Mean time <100ms
    - Cold start <200ms (includes dynamic linking)
    - Warm start <50ms (cached)
    - Document results in performance log
  - **Dependencies**: T046 (all tests pass, binary stable)
  - **Test Verification**: `hyperfine --warmup 3 './target/release/caro --help'`
  - **Commit Message**: `[T047] ✓ BENCHMARK: Startup time verified <100ms`

- [ ] **T048 ✓ LINT** Clippy warnings resolution
  - **Description**: Ensure zero clippy warnings
  - **Files**: Run clippy on all targets
  - **Acceptance Criteria**:
    - `cargo clippy --all-targets -- -D warnings` succeeds
    - No warnings in lib, binary, tests
    - Common warnings fixed:
      * Unused imports
      * Dead code
      * Unnecessary clones
      * Missing Debug/Display derives
    - Document any intentional clippy allows with rationale
  - **Dependencies**: T046 (tests pass)
  - **Test Verification**: `cargo clippy --all-targets --all-features -- -D warnings`
  - **Commit Message**: `[T048] ✓ LINT: Clippy warnings resolved (zero warnings)`

- [ ] **T049 ✓ FORMAT** Code formatting verification
  - **Description**: Ensure all code is properly formatted
  - **Files**: Run rustfmt on entire project
  - **Acceptance Criteria**:
    - `cargo fmt --check` succeeds (no changes needed)
    - Consistent formatting across all modules
    - Line length <100 characters (rustfmt default)
    - Proper indentation (4 spaces, no tabs)
    - If formatting needed: run `cargo fmt` first
  - **Dependencies**: T048 (clippy clean)
  - **Test Verification**: `cargo fmt --check`
  - **Commit Message**: `[T049] ✓ FORMAT: Code formatting verified`

- [ ] **T050 ✓ FINAL** Quickstart workflow manual validation
  - **Description**: Manually execute quickstart.md workflow
  - **Files**: Follow steps in `/workspaces/caro/specs/002-implement-tdd-green/quickstart.md`
  - **Acceptance Criteria**:
    - Step 1: Create CommandRequest - compiles, runs correctly
    - Step 2: Generate command with MockBackend - async works, returns valid response
    - Step 3: Validate safety - detects dangerous patterns, returns correct risk level
    - Step 4: Display results - all output formats work (JSON, YAML, text)
    - CLI commands work: `caro generate "list files"`, `caro validate "ls"`
    - Performance targets met: <100ms startup, <50ms validation, <2s generation
    - All documentation examples executable and accurate
  - **Dependencies**: T049 (code quality verified)
  - **Test Verification**: Manual execution of quickstart.md examples
  - **Commit Message**: `[T050] ✓ FINAL: Quickstart workflow validated - Feature 002 complete!`

## Dependencies Graph

```
Models (T001-T010)
  ├─→ Safety (T011-T020)
  └─→ Backends (T021-T030)
       └─→ CLI (T031-T040)
            └─→ Integration (T041-T050)

Parallel Opportunities:
- T001, T002, T003, T004 (enum definitions)
- T015, T016 (POSIX vs Windows validators)
- T036, T037 (JSON vs YAML formatters)
```

## Parallel Execution Examples

### Phase 1: Model Enums (Launch T001-T004 together)
```bash
# All enum definitions can run in parallel (different sections of same file)
Task T001: ShellType enum
Task T002: RiskLevel enum
Task T003: SafetyLevel enum
Task T004: BackendType enum
```

### Phase 2: Platform Validators (Launch T015-T016 together)
```bash
# POSIX and Windows validators are independent
Task T015: POSIX validation (bash, zsh, fish, sh)
Task T016: Windows validation (PowerShell, cmd)
```

### Phase 3: Output Formatters (Launch T036-T037 together)
```bash
# JSON and YAML formatters are independent
Task T036: JSON formatter
Task T037: YAML formatter
```

## Task Execution Rules

1. **One Task = One Commit**
   - Commit message format: `[T###] Description`
   - Example: `[T014] Implement core validate() method with pattern matching`

2. **Run Tests After Each Task**
   - If implementing T014: `cargo test --test safety_validator_contract test_basic_dangerous_patterns`
   - If tests regress (pass → fail): STOP and debug immediately

3. **TDD Discipline**
   - Tests already exist (RED phase complete)
   - Implementation makes tests pass (GREEN phase)
   - No implementation before test
   - No features beyond what tests require

4. **Verification Points**
   - T010: Models compile
   - T020: Safety complete (17 tests)
   - T030: Backends complete (28 tests)
   - T040: CLI complete (42 tests)
   - T046: Integration complete (80 tests - GOAL)

## Validation Checklist
*GATE: Must be satisfied before feature completion*

- [ ] All 80 contract tests pass (`cargo test`)
- [ ] All 7 entities from data-model.md implemented
- [ ] Quickstart workflow executes successfully
- [ ] Performance targets met (startup <100ms, validation <50ms)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] Documentation complete (all public APIs have rustdoc)
- [ ] Parallel tasks are truly independent (verified by reviewing files modified)
- [ ] Each task specifies absolute file paths
- [ ] No task modifies same file as another [P] task

## Success Metrics

| Metric | Target | Verification |
|--------|--------|--------------|
| Test Pass Rate | 80/80 (100%) | `cargo test` |
| Code Coverage | >80% | `cargo tarpaulin` |
| Startup Time | <100ms | `hyperfine ./target/release/caro --version` |
| Validation Time | <50ms | Performance tests |
| Mock Generation | <2s | Integration tests |
| Binary Size | <50MB | `ls -lh target/release/caro` |
| Clippy Warnings | 0 | `cargo clippy -- -D warnings` |

## Notes

- **TDD RED phase complete**: All 80 tests exist and currently fail with `NotImplemented` errors
- **Implementation strategy**: Make tests pass in order (models → safety → backends → CLI → integration)
- **No backtracking**: Once module tests pass, don't modify that module (preserve GREEN state)
- **Commit granularity**: One task = one commit for TDD audit trail
- **Parallel execution**: [P] tasks can run simultaneously (different files, no shared state)

## Module F: Quickstart Integration Tests (T051-T055)
**Extension - End-to-End Quickstart Scenarios from README**

- [x] **T051** Test "list all PDF files in Downloads" quickstart scenario
  - **Description**: Implement integration test for PDF file discovery workflow
  - **Files**: Create `tests/quickstart_scenarios.rs`, add test_list_pdf_files_in_downloads()
  - **Acceptance Criteria**:
    - Request: "list all PDF files in my Downloads folder"
    - Expected command contains: `find ~/Downloads -name "*.pdf"` OR `ls ~/Downloads/*.pdf`
    - Safety validation: RiskLevel::Safe
    - Output format compatibility: JSON/YAML/plain
    - **Current Status**: ❌ FAILING (Mock backend generates generic "ls -la")
  - **Dependencies**: T040 (CLI complete)
  - **Test Verification**: `cargo test --test quickstart_scenarios test_list_pdf_files_in_downloads`
  - **Commit Message**: `[T051] Add quickstart test: list PDF files in Downloads`

- [x] **T052** Test "compress all images" with auto-execution
  - **Description**: Test image compression with automatic execution workflow
  - **Files**: Update `tests/quickstart_scenarios.rs`, add test_compress_images_auto_execute()
  - **Acceptance Criteria**:
    - Request: "compress all images in current directory"
    - Safety level: Permissive (auto-execute mode)
    - Command should contain: tar/zip/gzip/compress keyword
    - No user confirmation required (confirm flag set)
    - **Current Status**: ❌ FAILING (Mock generates "pwd", execution module needed)
  - **Dependencies**: T051
  - **Test Verification**: `cargo test --test quickstart_scenarios test_compress_images_auto_execute`
  - **Commit Message**: `[T052] Add quickstart test: compress images with auto-exec`

- [x] **T053** Test "find large files" with custom model selection
  - **Description**: Test file size search with backend metadata verification
  - **Files**: Update `tests/quickstart_scenarios.rs`, add test_find_large_files_custom_backend()
  - **Acceptance Criteria**:
    - Request: "find all files larger than 100MB"
    - Command should contain: `find . -type f -size +100M`
    - Verbose mode provides backend info
    - Generation details mention backend used
    - **Current Status**: ✅ PASSING (with Mock backend, needs real backend switching)
  - **Dependencies**: T052
  - **Test Verification**: `cargo test --test quickstart_scenarios test_find_large_files_custom_backend`
  - **Commit Message**: `[T053] Add quickstart test: find large files with backend info`

- [x] **T054** Test safety blocking of dangerous commands
  - **Description**: Comprehensive safety system testing across all safety levels
  - **Files**: Update `tests/quickstart_scenarios.rs`, add test_dangerous_command_safety_blocking()
  - **Acceptance Criteria**:
    - Test Strict mode: blocks High/Critical commands
    - Test Moderate mode: confirms High, blocks Critical
    - Test Permissive mode: warns Critical, allows others
    - Verify blocked_reason messages are meaningful
    - Verify confirmation prompts are clear
    - **Current Status**: ❌ FAILING (Execution control needs refinement)
  - **Dependencies**: T053
  - **Test Verification**: `cargo test --test quickstart_scenarios test_dangerous_command_safety_blocking`
  - **Commit Message**: `[T054] Add quickstart test: safety blocking across modes`

- [x] **T055** Test multi-backend switching (MLX, vLLM, Ollama)
  - **Description**: Test backend availability detection and fallback chain
  - **Files**: Update `tests/quickstart_scenarios.rs`, add test_multi_backend_availability_and_fallback()
  - **Acceptance Criteria**:
    - Mock backend availability checks
    - Fallback chain: MLX → Ollama → vLLM → Mock
    - Backend metadata verification (backend_info())
    - Graceful degradation on unavailable backends
    - **Current Status**: ✅ PASSING (Mock only, needs real backend implementations)
  - **Dependencies**: T054
  - **Test Verification**: `cargo test --test quickstart_scenarios test_multi_backend_availability_and_fallback`
  - **Commit Message**: `[T055] Add quickstart test: multi-backend switching and fallback`

## Integration Test Summary

| Test Suite | Tests Total | Passing | Status | Notes |
|------------|-------------|---------|--------|-------|
| safety::patterns | 5 | 5/5 | ✅ | Pattern database tests |
| backend_trait_contract | 11 | 11/11 | ✅ | Async trait implementation |
| cli_interface_contract | 14 | 14/14 | ✅ | CLI argument parsing |
| error_handling_tests | 10 | 10/10 | ✅ | Error scenarios |
| integration_tests | 8 | 6/8 | 🟡 | 2 edge case failures |
| quickstart_scenarios | 6 | 3/6 | 🟡 | **NEW** - TDD RED phase |
| **TOTAL** | **54** | **49/54** | **91%** | 5 failing tests define future work |

## Future Implementation Needs (Based on Failing Tests)

**From quickstart_scenarios.rs failing tests:**

1. **Enhanced Mock Backend** (for test_list_pdf_files_in_downloads, test_compress_images_auto_execute)
   - Prompt-aware command generation
   - Keyword detection: "pdf" → find *.pdf, "compress" → tar/zip
   - Context-aware responses (Downloads directory, current directory)

2. **Execution Module** (for test_compress_images_auto_execute)
   - Command execution tracking
   - Auto-execute mode implementation
   - Execution result capture
   - Safety integration with execution control

3. **Real Backend Implementations** (for test_find_large_files_custom_backend, test_multi_backend_availability_and_fallback)
   - Ollama backend (HTTP API client)
   - vLLM backend (OpenAI-compatible API)
   - MLX backend (Apple Silicon FFI)
   - Backend availability detection
   - Fallback chain configuration

4. **Safety Execution Control** (for test_dangerous_command_safety_blocking)
   - Execution blocking based on RiskLevel + SafetyLevel matrix
   - Proper separation: validation (current) vs execution control (needed)
   - User confirmation workflow integration

---
*Tasks generated: 55 | Integration tests: 54 | Current pass rate: 49/54 (91%)*
*Feature 002: TDD GREEN Phase - Core complete, extension tests define future work*
*Based on plan.md, data-model.md, quickstart.md, research.md*
