# Tasks: caro - Natural Language to Shell Command CLI Tool

**Input**: Design documents from `/workspaces/caro/specs/001-create-a-comprehensive/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → Extract: Rust 1.75+, clap, tokio, serde, anyhow dependencies
   → Structure: Single CLI binary with modular backend system
2. Load design documents:
   → data-model.md: 7 core entities (CommandRequest, GeneratedCommand, etc.)
   → contracts/: 3 contract files (backend-trait, safety-validator, cli-interface)
   → research.md: Multi-backend trait system, FFI bindings, safety pipeline
   → quickstart.md: Installation scenarios, usage examples, integration tests
3. Generate tasks by category:
   → Setup: Rust project, dependencies, linting
   → Tests: 3 contract tests, 5 integration scenarios, property tests
   → Core: 7 models, backend system, safety validation, CLI interface
   → Integration: model caching, configuration, logging
   → Polish: performance optimization, documentation
4. Apply TDD rules: All tests before implementation
5. Number tasks sequentially (T001-T038)
6. Mark parallel tasks [P] for different files
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Phase 3.1: Setup
- [ ] T001 Create Rust project structure with src/, tests/, specs/ directories
- [ ] T002 Initialize Cargo.toml with dependencies: clap 4.4+, tokio, serde, reqwest, anyhow, tracing
- [ ] T003 [P] Configure build optimization in Cargo.toml for binary size (<50MB target)
- [ ] T004 [P] Setup GitHub Actions CI/CD for cross-platform builds (macOS, Linux, Windows)
- [ ] T005 [P] Configure cargo fmt, clippy, and cargo audit for code quality

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

### Contract Tests
- [ ] T006 [P] Backend trait contract tests in tests/contract/test_backend_trait.rs
- [ ] T007 [P] Safety validator contract tests in tests/contract/test_safety_validator.rs  
- [ ] T008 [P] CLI interface contract tests in tests/contract/test_cli_interface.rs

### Integration Tests from Quickstart Scenarios
- [ ] T009 [P] Integration test: Simple file operations in tests/integration/test_file_operations.rs
- [ ] T010 [P] Integration test: Safety feature demos in tests/integration/test_safety_features.rs
- [ ] T011 [P] Integration test: Backend management in tests/integration/test_backend_management.rs
- [ ] T012 [P] Integration test: Model management in tests/integration/test_model_management.rs
- [ ] T013 [P] Integration test: Configuration management in tests/integration/test_configuration.rs

### Property Tests for Safety
- [ ] T014 [P] Property test: Safety rule validation in tests/property/test_safety_properties.rs
- [ ] T015 [P] Property test: Command parsing edge cases in tests/property/test_command_parsing.rs

## Phase 3.3: Core Data Models (ONLY after tests are failing)
- [ ] T016 [P] CommandRequest struct in src/models/command_request.rs
- [ ] T017 [P] GeneratedCommand struct in src/models/generated_command.rs
- [ ] T018 [P] SafetyRule struct in src/models/safety_rule.rs
- [ ] T019 [P] ModelBackendConfig struct in src/models/backend_config.rs
- [ ] T020 [P] UserConfiguration struct in src/models/user_config.rs
- [ ] T021 [P] CommandHistory struct in src/models/command_history.rs
- [ ] T022 [P] ModelCache struct in src/models/model_cache.rs
- [ ] T023 [P] Core enumerations (RiskLevel, SafetyLevel, BackendType, etc.) in src/models/enums.rs

## Phase 3.4: Backend System Implementation
- [ ] T024 CommandGenerator trait definition in src/backends/mod.rs
- [ ] T025 [P] Ollama backend implementation in src/backends/ollama.rs
- [ ] T026 [P] vLLM backend implementation in src/backends/vllm.rs
- [ ] T027 MLX backend with FFI bindings in src/backends/mlx.rs (depends on cxx crate setup)
- [ ] T028 Backend factory and selection logic in src/backends/factory.rs

## Phase 3.5: Safety Validation System
- [ ] T029 SafetyValidator trait implementation in src/safety/mod.rs
- [ ] T030 [P] Dangerous pattern database in src/safety/patterns.rs
- [ ] T031 [P] Risk assessment engine in src/safety/risk_assessment.rs
- [ ] T032 User confirmation system in src/safety/confirmation.rs

## Phase 3.6: CLI Interface Implementation
- [ ] T033 Main CLI application with clap in src/main.rs
- [ ] T034 [P] Configuration subcommands in src/cli/config.rs
- [ ] T035 [P] Model management subcommands in src/cli/models.rs
- [ ] T036 [P] Backend management subcommands in src/cli/backends.rs
- [ ] T037 [P] History management subcommands in src/cli/history.rs

## Phase 3.7: Integration & Configuration
- [ ] T038 Configuration loading hierarchy in src/config/mod.rs
- [ ] T039 Model cache management with Hugging Face integration in src/cache/mod.rs
- [ ] T040 Command execution engine with safety checks in src/execution/mod.rs
- [ ] T041 Structured logging and error handling in src/logging/mod.rs
- [ ] T042 Cross-platform compatibility layer in src/platform/mod.rs

## Phase 3.8: Core Command Generation Pipeline
- [ ] T043 Command generation orchestrator in src/core/generator.rs
- [ ] T044 Prompt engineering and system prompts in src/core/prompts.rs
- [ ] T045 JSON response parsing with fallback strategies in src/core/parser.rs
- [ ] T046 User interaction and confirmation workflows in src/core/interaction.rs

## Phase 3.9: Performance Optimization
- [ ] T047 [P] Startup time optimization (lazy loading, static patterns) in src/perf/startup.rs
- [ ] T048 [P] Memory usage optimization (bounded caches, streaming) in src/perf/memory.rs
- [ ] T049 [P] Binary size optimization (feature flags, dependency pruning) in Cargo.toml
- [ ] T050 Benchmarking framework and performance tests in tests/benchmarks/mod.rs

## Phase 3.10: Polish & Documentation
- [ ] T051 [P] Update CLAUDE.md with development commands and architecture
- [ ] T052 [P] Create comprehensive README.md with installation and usage
- [ ] T053 [P] Unit tests for utility functions in tests/unit/
- [ ] T054 [P] Documentation examples and API docs with cargo doc
- [ ] T055 Manual testing against quickstart scenarios
- [ ] T056 Performance validation against targets (<100ms startup, <2s inference)

## Dependencies
**Critical Path**:
- Setup (T001-T005) before everything
- All tests (T006-T015) before any implementation (T016+)
- Models (T016-T023) before services and backends
- Backend system (T024-T028) before CLI integration
- Safety system (T029-T032) before command execution
- Core pipeline (T043-T046) depends on backends and safety
- Performance optimization (T047-T050) after core functionality
- Polish (T051-T056) after everything else

**Parallel Execution Blocks**:
- Setup tasks T003-T005 can run together
- Contract tests T006-T008 can run together  
- Integration tests T009-T013 can run together
- Model definitions T016-T023 can run together
- Backend implementations T025-T026 can run together
- CLI subcommands T034-T037 can run together

## Parallel Execution Examples

### Phase 3.2: Launch all contract tests together
```bash
# Launch T006-T008 in parallel:
Task: "Backend trait contract tests in tests/contract/test_backend_trait.rs"
Task: "Safety validator contract tests in tests/contract/test_safety_validator.rs"  
Task: "CLI interface contract tests in tests/contract/test_cli_interface.rs"
```

### Phase 3.3: Launch all model definitions together
```bash
# Launch T016-T023 in parallel:
Task: "CommandRequest struct in src/models/command_request.rs"
Task: "GeneratedCommand struct in src/models/generated_command.rs"
Task: "SafetyRule struct in src/models/safety_rule.rs"
Task: "ModelBackendConfig struct in src/models/backend_config.rs"
Task: "UserConfiguration struct in src/models/user_config.rs"
Task: "CommandHistory struct in src/models/command_history.rs"
Task: "ModelCache struct in src/models/model_cache.rs"
Task: "Core enumerations in src/models/enums.rs"
```

### Phase 3.5: Launch backend implementations
```bash
# Launch T025-T026 in parallel (T027 depends on cxx setup):
Task: "Ollama backend implementation in src/backends/ollama.rs"
Task: "vLLM backend implementation in src/backends/vllm.rs"
```

## Task Validation Checklist
*GATE: Checked before execution*

- [x] All contracts (3) have corresponding contract tests (T006-T008)
- [x] All entities (7) have model creation tasks (T016-T023)  
- [x] All tests (T006-T015) come before implementation (T016+)
- [x] Parallel tasks [P] operate on different files
- [x] Each task specifies exact file path
- [x] TDD cycle enforced: failing tests before implementation
- [x] Dependencies properly ordered and documented
- [x] Performance targets addressed in dedicated tasks
- [x] Cross-platform compatibility handled
- [x] Safety-first approach maintained throughout

## Performance Targets Validation
- **T047**: Startup time <100ms validation
- **T048**: Memory usage <500MB validation  
- **T049**: Binary size <50MB validation
- **T050**: Inference time <2s validation
- **T056**: End-to-end performance validation

## Safety Requirements Validation  
- **T007**: Safety validator contract tests (dangerous pattern detection)
- **T010**: Safety feature integration tests (risk levels, confirmations)
- **T014**: Property tests for safety rules edge cases
- **T029-T032**: Complete safety validation system implementation

## Notes
- [P] tasks = different files, no dependencies, safe for parallel execution
- All tests must fail before implementing corresponding functionality
- Commit after each task completion
- MLX backend (T027) requires careful FFI integration with cxx crate
- Performance optimization tasks (T047-T050) are critical for meeting targets
- Safety system implementation is non-negotiable and must be comprehensive