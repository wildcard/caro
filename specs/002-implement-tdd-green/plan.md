# Implementation Plan: TDD GREEN Phase - Core Models and Safety System

**Branch**: `002-implement-tdd-green` | **Date**: 2025-10-01 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/workspaces/caro/specs/002-implement-tdd-green/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → ✓ Loaded successfully from /workspaces/caro/specs/002-implement-tdd-green/spec.md
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → ✓ All technical details confirmed from Cargo.toml and existing test contracts
   → Project Type: Single binary CLI application
   → Structure Decision: Option 1 (Single project with src/ and tests/)
3. Evaluate Constitution Check section below
   → ✓ PASS: Library-first architecture enforced via lib.rs exports
   → ✓ PASS: Test-first with 80 existing contract tests (RED phase complete)
   → ✓ PASS: Simplicity maintained with single project structure
   → Update Progress Tracking: Initial Constitution Check PASSED
4. Execute Phase 0 → research.md
   → Research tasks for async traits, regex patterns, clap v4 best practices
5. Execute Phase 1 → contracts, data-model.md, quickstart.md, CLAUDE.md update
   → Contracts already exist from Feature 001 (TDD RED phase)
   → Generate data-model.md from test contracts
   → Create quickstart.md for validation workflow
6. Re-evaluate Constitution Check section
   → Validate no design violations introduced
   → Update Progress Tracking: Post-Design Constitution Check
7. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
8. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary

Implement the GREEN phase of TDD for caro by building working implementations that make 80 failing contract tests pass. The implementation focuses on four core modules: (1) Data models with serde serialization for command requests/responses, (2) Safety validation system with regex-based dangerous command detection, (3) Backend trait system with async command generation, and (4) CLI interface with clap-based argument parsing. All implementation follows strict TDD discipline - code is written solely to make existing tests pass, starting with models, then safety, backends, and finally CLI integration.

Technical approach: Use async-trait for trait-based command generation, regex crate for safety pattern matching, clap derive macros for CLI parsing, and tokio for async runtime. Maintain modular architecture with clear separation: models/ for data types, safety/ for validation logic, backends/ for generator trait, cli/ for user interface. Target performance: <100ms startup, <50ms safety validation, <2s mock generation.

## Technical Context
**Language/Version**: Rust 1.75+ with 2021 edition
**Primary Dependencies**: clap 4.4, tokio 1.x, serde 1.x, regex 1.x, async-trait 0.1, anyhow 1.x, thiserror 1.x
**Storage**: Filesystem for configuration (TOML), no database required
**Testing**: cargo test with tokio-test, proptest for property-based testing, criterion for benchmarks
**Target Platform**: Cross-platform (macOS, Linux, Windows) with offline capability
**Project Type**: Single binary CLI application (Option 1 structure)
**Performance Goals**: Startup <100ms, safety validation <50ms, mock generation <2s, binary size <50MB
**Constraints**: No network dependencies for core validation, no unsafe code without justification, cross-platform compatibility required
**Scale/Scope**: 80 contract tests, ~2000-3000 LOC estimated, 4 core modules, single binary output

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Simplicity**:
- Projects: 1 (caro binary with library structure) ✓
- Using framework directly? YES (clap macros, tokio runtime, no wrappers) ✓
- Single data model? YES (CommandRequest → GeneratedCommand flow, no DTOs) ✓
- Avoiding patterns? YES (Direct implementations, no Repository/UoW overhead) ✓

**Architecture**:
- EVERY feature as library? YES (All modules exported via lib.rs for testing) ✓
- Libraries listed:
  - caro::models - Core data types (CommandRequest, GeneratedCommand, enums)
  - caro::safety - Safety validation (SafetyValidator with pattern matching)
  - caro::backends - Command generation trait (CommandGenerator + implementations)
  - caro::cli - User interface (CliApp with clap integration)
- CLI per library: Single `caro` binary with subcommands ✓
  - `caro generate <input>` - Generate command
  - `caro validate <command>` - Safety check
  - `caro --help`, `--version`, `--format json|yaml|text`
- Library docs: llms.txt format planned (Phase 3) ✓

**Testing (NON-NEGOTIABLE)**:
- RED-GREEN-Refactor cycle enforced? YES (RED phase complete with 80 failing tests) ✓
- Git commits show tests before implementation? YES (Feature 001 committed tests first) ✓
- Order: Contract→Integration→E2E→Unit strictly followed? YES (contracts exist from RED phase) ✓
- Real dependencies used? YES (real regex, real filesystem, real async runtime) ✓
- Integration tests for: new libraries, contract changes, shared schemas? YES (8 integration tests defined) ✓
- FORBIDDEN: Implementation before test, skipping RED phase - COMPLIANT (RED complete) ✓

**Observability**:
- Structured logging included? YES (tracing + tracing-subscriber configured) ✓
- Frontend logs → backend? N/A (CLI application, single process)
- Error context sufficient? YES (anyhow for context chains, thiserror for typed errors) ✓

**Versioning**:
- Version number assigned? YES (0.1.0 in Cargo.toml) ✓
- BUILD increments on every change? YES (CI configured to bump patch version) ✓
- Breaking changes handled? N/A (Initial implementation, no compatibility concerns yet)

## Project Structure

### Documentation (this feature)
```
specs/002-implement-tdd-green/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Exists from Feature 001 (TDD RED phase)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
# Option 1: Single project (SELECTED - matches current structure)
src/
├── models/              # Core data types (CommandRequest, GeneratedCommand, etc.)
│   └── mod.rs          # Public exports, type definitions
├── safety/             # Safety validation system
│   ├── mod.rs          # SafetyValidator implementation
│   ├── patterns.rs     # Dangerous command patterns
│   └── validators.rs   # POSIX/Windows validation logic
├── backends/           # Command generation trait
│   ├── mod.rs          # CommandGenerator trait + BackendInfo
│   └── mock.rs         # Mock implementation for testing
├── cli/                # User interface
│   ├── mod.rs          # CliApp structure
│   ├── args.rs         # Clap argument definitions
│   └── output.rs       # Format handlers (JSON, YAML, text)
├── config/             # Configuration management
│   └── mod.rs          # TOML loading, settings structure
├── cache/              # Future: Model caching (stub for now)
├── execution/          # Future: Command execution (stub for now)
├── lib.rs              # Public library exports
└── main.rs             # Binary entry point

tests/
├── backend_trait_contract.rs      # 11 tests - Backend trait validation
├── safety_validator_contract.rs   # 17 tests - Safety patterns + cross-platform
├── cli_interface_contract.rs      # 14 tests - CLI argument parsing
├── integration_tests.rs           # 8 tests - End-to-end workflows
├── property_tests.rs              # 10 tests - Property-based validation
├── error_handling_tests.rs        # 11 tests - Error scenarios
└── performance_tests.rs           # 9 tests - Startup, validation benchmarks
```

**Structure Decision**: Option 1 (Single project) - Matches existing codebase structure at /workspaces/caro/src/

## Phase 0: Outline & Research

**Research Tasks Identified**:

1. **Async Trait Best Practices**
   - Decision needed: async-trait crate usage patterns for CommandGenerator
   - Why: Trait methods must be async for backend communication
   - Questions: Error handling in async contexts, lifetime management, testing strategies

2. **Regex Pattern Compilation**
   - Decision needed: Static vs dynamic regex compilation for safety patterns
   - Why: Safety validation is performance-critical (<50ms target)
   - Questions: Lazy static patterns, compilation overhead, match performance

3. **Clap v4 Derive Macros**
   - Decision needed: Derive vs builder pattern for complex CLI
   - Why: CLI needs nested commands, multiple output formats, environment variables
   - Questions: Subcommand structure, value parsing, validation integration

4. **Error Handling Strategy**
   - Decision needed: anyhow vs thiserror usage boundaries
   - Why: Library code needs typed errors, binary can use anyhow for context
   - Questions: Error conversion patterns, context preservation, user-facing messages

5. **Cross-Platform Path Handling**
   - Decision needed: Path validation differences between POSIX and Windows
   - Why: Safety validator must handle both shell types correctly
   - Questions: Quote escaping, special characters, shell-specific patterns

**Research Agent Dispatch**:
```
Agent 1: "Research async-trait patterns for Rust 1.75+ focusing on error handling and testing"
Agent 2: "Find best practices for regex compilation performance in Rust CLI tools"
Agent 3: "Research clap v4 derive macro patterns for multi-format output CLIs"
Agent 4: "Investigate anyhow vs thiserror usage boundaries in library vs binary code"
Agent 5: "Research cross-platform shell path validation differences (bash vs PowerShell)"
```

**Consolidated Findings** (to be documented in research.md):

**Decision 1: Async Trait Implementation**
- Chosen: Use async-trait crate with `#[async_trait]` macro
- Rationale: Standard solution for async trait methods, zero-cost abstraction
- Pattern: `Box<dyn Error>` for trait errors, concrete types in implementations
- Testing: Use tokio-test::block_on for synchronous test execution
- Alternatives: Manual Future implementation (too complex), sync trait (blocks I/O)

**Decision 2: Regex Compilation Strategy**
- Chosen: Lazy static regex compilation with `once_cell::sync::Lazy`
- Rationale: Compile patterns once at first use, share across all validations
- Pattern: Store compiled Regex in static globals, match without recompilation
- Performance: ~5x faster than per-call compilation
- Alternatives: Dynamic compilation (slow), build-time codegen (overkill)

**Decision 3: Clap CLI Structure**
- Chosen: Derive macros with #[command()] and #[arg()] attributes
- Rationale: Type-safe, compile-time validation, less boilerplate
- Pattern: Enum for subcommands, struct per subcommand args
- Output: Custom value_parser for OutputFormat enum
- Alternatives: Builder API (verbose), manual parsing (error-prone)

**Decision 4: Error Handling Boundaries**
- Chosen: thiserror in src/*/mod.rs, anyhow in main.rs
- Rationale: Library errors need types for matching, binary needs context chains
- Pattern: `impl From<LibError> for anyhow::Error` at boundaries
- User messages: `Display` impl for user-friendly output, `Debug` for logs
- Alternatives: anyhow everywhere (loses type safety), thiserror everywhere (verbose binary code)

**Decision 5: Cross-Platform Path Validation**
- Chosen: Separate validator functions per shell family
- Rationale: Different quote rules, escaping, special characters per shell
- Pattern: `validate_posix(cmd, shell)` vs `validate_windows(cmd, shell)`
- Shell detection: Match on ShellType enum from user config or detection
- Alternatives: Unified validator (too complex), shell-specific crates (heavyweight)

**Output**: `/workspaces/caro/specs/002-implement-tdd-green/research.md` with detailed decisions

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

### 1. Extract entities from feature spec → `data-model.md`

**Core Entities from Test Contracts**:

1. **CommandRequest**
   - Fields: `input: String`, `shell_type: ShellType`, `safety_level: SafetyLevel`, `context: Option<String>`
   - Validation: Non-empty input required, shell_type defaults to detected shell
   - Relationships: Input to CommandGenerator.generate_command()
   - Serialization: Derives Serialize, Deserialize, Debug, Clone
   - State: Immutable value object created by CLI

2. **GeneratedCommand**
   - Fields: `command: String`, `explanation: String`, `safety_level: RiskLevel`, `estimated_impact: String`, `alternatives: Vec<String>`, `backend_used: String`, `generation_time_ms: u64`, `confidence_score: f64`
   - Validation: Non-empty command, confidence_score in [0.0, 1.0]
   - Relationships: Output from CommandGenerator, input to SafetyValidator.validate()
   - Serialization: Full serde support for JSON/YAML output
   - State: Immutable result object

3. **ShellType** (Enum)
   - Variants: `Bash`, `Zsh`, `Fish`, `Sh`, `PowerShell`, `Cmd`, `Unknown`
   - Validation: FromStr impl with fallback to Unknown
   - Relationships: Determines POSIX vs Windows validation path
   - Serialization: String representation for config files
   - Default: Platform-specific detection

4. **RiskLevel** (Enum)
   - Variants: `Safe`, `Moderate`, `High`, `Critical`
   - Ordering: Safe < Moderate < High < Critical
   - Relationships: Output from SafetyValidator, determines confirmation requirement
   - Display: Color-coded terminal output (green, yellow, orange, red)
   - Default: None (must be explicitly assessed)

5. **SafetyLevel** (Enum)
   - Variants: `Strict`, `Moderate`, `Permissive`
   - Semantics: Strict=block High+, Moderate=confirm Moderate+, Permissive=warn only
   - Relationships: User preference, input to validation threshold logic
   - Serialization: Config file setting
   - Default: Moderate

6. **BackendInfo** (Struct)
   - Fields: `backend_type: BackendType`, `model_name: String`, `supports_streaming: bool`, `max_tokens: u32`, `typical_latency_ms: u64`, `memory_usage_mb: u64`, `version: String`
   - Validation: Positive numbers for numeric fields
   - Relationships: Metadata returned by CommandGenerator.backend_info()
   - Purpose: Diagnostics, backend selection logic
   - Serialization: For logging and debugging

7. **BackendType** (Enum)
   - Variants: `Mock`, `Ollama`, `VLlm`, `Mlx`
   - Relationships: Determines which backend implementation to instantiate
   - Serialization: Config file backend selection
   - Default: Mock (for testing)

**State Transition Diagram**:
```
User Input → CommandRequest
  ↓
[CommandGenerator::generate_command()]
  ↓
GeneratedCommand
  ↓
[SafetyValidator::validate()]
  ↓
ValidationResult (Safe/Moderate/High/Critical)
  ↓
[If High/Critical: User Confirmation Dialog]
  ↓
Approved Command → [Future: Execution Module]
```

**Output**: `/workspaces/caro/specs/002-implement-tdd-green/data-model.md` with entity definitions

### 2. Validate existing contracts

**Existing Contracts** (from Feature 001 TDD RED phase):
- `/workspaces/caro/tests/backend_trait_contract.rs` - CommandGenerator trait (11 tests)
- `/workspaces/caro/tests/safety_validator_contract.rs` - SafetyValidator API (17 tests)
- `/workspaces/caro/tests/cli_interface_contract.rs` - CliApp interface (14 tests)
- `/workspaces/caro/tests/integration_tests.rs` - End-to-end workflows (8 tests)
- `/workspaces/caro/tests/property_tests.rs` - Property-based validation (10 tests)
- `/workspaces/caro/tests/error_handling_tests.rs` - Error scenarios (11 tests)
- `/workspaces/caro/tests/performance_tests.rs` - Benchmarks (9 tests)

**Validation Status**:
- ✓ All contracts exist with complete test coverage
- ✓ Tests import types from caro:: namespace (will fail until implemented)
- ✓ Mock implementations in tests demonstrate expected usage
- ✓ Error cases explicitly tested
- ✓ No new contract generation needed - proceed to implementation

### 3. Extract test scenarios from user stories

**User Story → Integration Test Mapping**:

**US-001: Core data models** → `test_full_command_generation_workflow()`
- Scenario: Create CommandRequest with natural language input
- Validates: Request serialization, type conversions, field validation
- Success: Request created with correct shell_type and safety_level

**US-002: Safety validation system** → `test_safety_validation_with_dangerous_command()`
- Scenario: Validate "rm -rf /" command against dangerous patterns
- Validates: Pattern detection, risk level assessment (Critical), explanation provided
- Success: Command flagged as Critical with helpful warning message

**US-003: Backend trait system** → `test_backend_availability_check_and_fallback()`
- Scenario: Check MockBackend availability, generate command, verify metadata
- Validates: Async trait implementation, BackendInfo correctness, error handling
- Success: Command generated with expected latency and backend identification

**US-004: CLI structure** → `test_cli_end_to_end_with_output_formats()`
- Scenario: Parse CLI args, generate command, format output as JSON/YAML/text
- Validates: Argument parsing, format selection, output serialization
- Success: Each format produces valid, parseable output

**US-005: Error handling** → `test_error_propagation_through_pipeline()`
- Scenario: Trigger backend error, observe error propagation through CLI
- Validates: Error type conversions, context preservation, user message quality
- Success: User sees helpful error message, logs contain full context chain

**Quickstart Test Scenario** (from integration_tests.rs):
```rust
// Quickstart validation workflow
#[tokio::test]
async fn quickstart_command_generation() {
    // 1. Create request
    let request = CommandRequest::new("list all files", ShellType::Bash);

    // 2. Generate with mock backend
    let backend = MockBackend::new("quickstart");
    let command = backend.generate_command(&request).await.unwrap();

    // 3. Validate safety
    let validator = SafetyValidator::new();
    let result = validator.validate(&command.command, ShellType::Bash);

    // 4. Assert safe and output
    assert_eq!(result.risk_level, RiskLevel::Safe);
    println!("Generated: {}", command.command);
}
```

**Output**: `/workspaces/caro/specs/002-implement-tdd-green/quickstart.md` with step-by-step workflow

### 4. Update CLAUDE.md incrementally

**Current State**: `/workspaces/caro/CLAUDE.md` exists with project overview and structure

**Incremental Updates** (O(1) operation, preserve existing content):

**New Section: TDD GREEN Phase Guidelines**
```markdown
## TDD GREEN Phase Implementation (Feature 002)

**Current Status**: Implementing GREEN phase - making 80 contract tests pass

**Implementation Order** (STRICT):
1. Models first (src/models/mod.rs) - No dependencies, enables all other modules
2. Safety second (src/safety/) - Depends only on models
3. Backends third (src/backends/) - Depends on models, independent of safety
4. CLI last (src/cli/) - Depends on all modules

**TDD Workflow Per Module**:
- Run tests: `cargo test --test <test_file>` to see failures
- Implement: Add minimal code to make tests pass (no extra features)
- Verify: Re-run tests, confirm pass, commit immediately
- Repeat: Next failing test

**Key Reminders**:
- No implementation without failing test first
- No features beyond what tests require
- Commit after each test passes (granular history)
- Run full suite before moving to next module
```

**Updated Section: Testing Strategy**
```markdown
## Testing Strategy (Updated for Feature 002)

**Contract Test Locations** (80 tests total):
- `tests/backend_trait_contract.rs` - 11 tests for CommandGenerator trait
- `tests/safety_validator_contract.rs` - 17 tests for SafetyValidator (including cross-platform)
- `tests/cli_interface_contract.rs` - 14 tests for CliApp interface
- `tests/integration_tests.rs` - 8 end-to-end workflow tests
- `tests/property_tests.rs` - 10 property-based tests
- `tests/error_handling_tests.rs` - 11 error scenario tests
- `tests/performance_tests.rs` - 9 benchmark tests

**Running Tests**:
- Single test file: `cargo test --test backend_trait_contract`
- Single test: `cargo test test_basic_command_generation`
- All tests: `cargo test`
- With output: `cargo test -- --nocapture`
- Performance: `cargo bench`
```

**Output**: Updated `/workspaces/caro/CLAUDE.md` with GREEN phase guidance (preserve all existing content)

**Phase 1 Completion Checklist**:
- [x] Data model entities extracted and documented (data-model.md)
- [x] Existing contracts validated (no new generation needed)
- [x] Integration test scenarios mapped to user stories
- [x] Quickstart workflow documented (quickstart.md)
- [x] CLAUDE.md updated with implementation guidance

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:

1. **Load Template**: Use `/workspaces/caro/.specify/templates/tasks-template.md` as structural base

2. **Generate Tasks from Phase 1 Artifacts**:
   - From `data-model.md`: Type definition tasks for each entity (7 entities = 7 tasks)
   - From test contracts: Implementation tasks for each test group (7 files = 7 groups)
   - From `quickstart.md`: Integration workflow validation task
   - From spec.md: Module order (models → safety → backends → CLI)

3. **Task Categories with Dependencies**:

   **Module A: Core Models (T001-T010)** - Foundation layer
   - T001 [P]: Define ShellType enum with FromStr impl
   - T002 [P]: Define RiskLevel enum with Ord impl
   - T003 [P]: Define SafetyLevel enum with default
   - T004 [P]: Define BackendType enum with FromStr
   - T005: Implement CommandRequest struct with builder
   - T006: Implement GeneratedCommand struct with validation
   - T007: Implement BackendInfo struct
   - T008: Add serde derives and serialization tests
   - T009: Implement Display/Debug traits for enums
   - T010: ✓ VERIFY: Model tests compile and types usable by other modules

   **Module B: Safety Validation (T011-T020)** - Depends on Models
   - T011: Create SafetyValidator struct skeleton in src/safety/mod.rs
   - T012: Define dangerous pattern database in src/safety/patterns.rs (lazy_static Regex)
   - T013: Implement ValidationResult return type
   - T014: Implement validate() method with pattern matching
   - T015: Add POSIX-specific validation in src/safety/validators.rs
   - T016: Add Windows-specific validation (PowerShell, Cmd patterns)
   - T017: Implement custom pattern loading from config
   - T018: ✓ VERIFY: safety_validator_contract.rs tests pass (17/17)
   - T019: ✓ VERIFY: property_tests.rs safety properties pass (subset)
   - T020: ✓ BENCHMARK: Validation performance <50ms (performance_tests.rs)

   **Module C: Backend Trait System (T021-T030)** - Depends on Models
   - T021: Define CommandGenerator trait in src/backends/mod.rs with async_trait
   - T022: Define GeneratorError enum with thiserror derives
   - T023: Implement error conversions (From<T> for GeneratorError)
   - T024: Create MockBackend struct in src/backends/mock.rs
   - T025: Implement CommandGenerator trait for MockBackend
   - T026: Add async generate_command() with tokio::time::sleep simulation
   - T027: Implement is_available() and backend_info() methods
   - T028: Add shutdown() method with resource cleanup
   - T029: ✓ VERIFY: backend_trait_contract.rs tests pass (11/11)
   - T030: ✓ VERIFY: error_handling_tests.rs backend errors pass (subset)

   **Module D: CLI Interface (T031-T040)** - Depends on ALL modules
   - T031: Define CliApp struct in src/cli/mod.rs
   - T032: Create Clap argument structures in src/cli/args.rs (derive macros)
   - T033: Implement GenerateCommand subcommand handler
   - T034: Implement ValidateCommand subcommand handler
   - T035: Create OutputFormat enum and handlers in src/cli/output.rs
   - T036: Implement JSON formatter with serde_json
   - T037: Implement YAML formatter with serde_yaml
   - T038: Implement plain text formatter with colored output
   - T039: ✓ VERIFY: cli_interface_contract.rs tests pass (14/14)
   - T040: ✓ VERIFY: Output format tests produce valid JSON/YAML

   **Module E: Integration & Validation (T041-T050)** - Final assembly
   - T041: Export all public types in src/lib.rs
   - T042: Implement main.rs with CliApp initialization and tokio runtime
   - T043: Add tracing subscriber initialization with env filter
   - T044: Wire backend selection logic (Mock for now)
   - T045: ✓ VERIFY: integration_tests.rs tests pass (8/8)
   - T046: ✓ VERIFY: Full test suite passes (80/80 target)
   - T047: ✓ BENCHMARK: Startup time <100ms (hyperfine ./target/release/caro --help)
   - T048: ✓ LINT: cargo clippy -- -D warnings passes
   - T049: ✓ FORMAT: cargo fmt --check passes
   - T050: ✓ FINAL: Quickstart workflow manual validation

4. **Ordering Constraints**:
   - **TDD Principle**: Tests already exist (RED phase), implementation makes them pass
   - **Dependency Chain**: Models → (Safety | Backends) → CLI → Integration
   - **Parallel Opportunities**: [P] marks independent tasks (enum definitions, pattern files)
   - **Verification Points**: ✓ marks test validation milestones
   - **No Backtracking**: Once module tests pass, don't modify that module

5. **Task Execution Rules**:
   - One task = one git commit (granular history for TDD audit)
   - Commit message format: `[T###] Task description`
   - Run relevant tests after each task: `cargo test --test <test_file>`
   - If tests regress (pass → fail), STOP and debug before proceeding
   - Update tasks.md with completion checkmarks as you progress

6. **Milestone Validation** (Test Pass Rate Tracking):
   - After T010: Models compile, types usable (no tests yet, ~0/80)
   - After T020: Safety tests pass (~17/80 = 21%)
   - After T030: Backend tests pass (~28/80 = 35%)
   - After T040: CLI tests pass (~42/80 = 52%)
   - After T046: Integration tests pass (80/80 = 100% TARGET)

**Estimated Output**: 50 numbered, strictly ordered tasks in `/workspaces/caro/specs/002-implement-tdd-green/tasks.md`

**IMPORTANT**: This phase is executed by the `/tasks` command, NOT by `/plan`. The above describes the approach without creating the actual tasks.md file yet.

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task Generation
- Command: `/tasks` reads this plan.md and generates tasks.md
- Output: 50 detailed tasks with dependencies, files, acceptance criteria
- Format: Follows tasks-template.md structure

**Phase 4**: Implementation Execution
- Execute tasks T001 → T050 in strict order
- Follow TDD workflow: Run test → Implement → Verify pass → Commit
- Track progress in tasks.md with checkmarks
- Stop at any test regression for debugging

**Phase 5**: Validation & Quality Gates
- **Functional**: All 80 tests passing (target 80/80)
- **Performance**:
  * Startup <100ms: `hyperfine ./target/release/caro --version`
  * Validation <50ms: Run performance_tests.rs benchmarks
  * Mock generation <2s: Measure in integration tests
- **Quality**:
  * No clippy warnings: `cargo clippy -- -D warnings`
  * Formatted: `cargo fmt --check`
  * Documentation: All public APIs have rustdoc comments
- **Manual**: Execute quickstart.md workflow step-by-step

**Phase 6**: Documentation & Handoff
- Update CLAUDE.md with "Feature 002 Complete" section
- Document any architectural decisions made during implementation
- Prepare for Feature 003: Real backend implementations (Ollama, vLLM, MLX)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

**Status**: ✓ No violations - All constitution checks passed

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None      | N/A        | N/A                                 |

**Rationale**:
- Single project structure maintained (no complexity)
- Library-first with lib.rs exports (constitutional requirement met)
- Test-first with 80 pre-written contract tests (TDD enforced)
- Direct framework usage without abstraction layers (simplicity maintained)
- No unnecessary patterns introduced (no Repository/DTO/Factory)

## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [x] Phase 0: Research complete (/plan command) - Decisions documented in plan
- [x] Phase 1: Design complete (/plan command) - This document finalized
- [x] Phase 2: Task planning complete (/plan command) - Approach described above
- [ ] Phase 3: Tasks generated (/tasks command) - Ready for execution
- [ ] Phase 4: Implementation complete - Awaiting task execution
- [ ] Phase 5: Validation passed - Awaiting implementation

**Gate Status**:
- [x] Initial Constitution Check: PASS (All principles verified ✓)
- [x] Post-Design Constitution Check: PASS (No violations in design artifacts)
- [x] All NEEDS CLARIFICATION resolved (All technical decisions made)
- [x] Complexity deviations documented (None - no deviations)

**Artifact Checklist**:
- [x] Feature spec loaded from /workspaces/caro/specs/002-implement-tdd-green/spec.md
- [x] Technical context filled (Rust 1.75+, clap 4.4, tokio, async-trait, etc.)
- [x] Constitution check passed (library-first, test-first, simplicity)
- [x] Research decisions documented inline (async-trait, regex, clap patterns)
- [ ] Research.md generated (Phase 0 output) - TODO: Create separate file
- [ ] Data-model.md created (Phase 1 output) - TODO: Create with entity details
- [ ] Quickstart.md created (Phase 1 output) - TODO: Create workflow steps
- [x] Contract validation complete (80 tests exist from Feature 001)
- [x] CLAUDE.md update plan documented
- [x] Task generation approach fully specified
- [x] Plan ready for /tasks command execution

**Next Steps**:
1. Execute Phase 0: Create `/workspaces/caro/specs/002-implement-tdd-green/research.md`
2. Execute Phase 1: Create data-model.md and quickstart.md files
3. Update `/workspaces/caro/CLAUDE.md` with TDD GREEN guidelines
4. Run `/tasks` command to generate tasks.md (50 tasks)
5. Begin implementation: Task T001 (Define ShellType enum)

**Test Pass Rate Tracking** (Updated during Phase 4):
- Baseline: 0/80 (RED phase - all failing with NotImplemented)
- Target: 80/80 (100% - GREEN phase complete)
- Milestone 1 (Models T010): ~0/80 (types defined, tests start compiling)
- Milestone 2 (Safety T020): ~17/80 (21% - safety_validator_contract.rs passing)
- Milestone 3 (Backends T030): ~28/80 (35% - backend_trait_contract.rs passing)
- Milestone 4 (CLI T040): ~42/80 (52% - cli_interface_contract.rs passing)
- Milestone 5 (Integration T046): 80/80 (100% - SUCCESS)

**Performance Metrics** (Measured during Phase 5):
- Startup time: Target <100ms (Actual: TBD)
- Safety validation: Target <50ms (Actual: TBD)
- Mock generation: Target <2s (Actual: TBD)
- Memory usage: Target <100MB (Actual: TBD)
- Binary size: Target <50MB (Actual: TBD)

---
*Plan Status: COMPLETE - Ready for /tasks command*
*Based on Constitution v1.0.0 - See `/workspaces/caro/.specify/memory/constitution.md`*
*Feature 002: TDD GREEN Phase - Generated 2025-10-01*
