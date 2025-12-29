
# Implementation Plan: Core Infrastructure Modules

**Branch**: `003-implement-core-infrastructure` | **Date**: 2025-10-02 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/workspaces/caro/specs/003-implement-core-infrastructure/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from file system structure or context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Fill the Constitution Check section based on the content of the constitution document.
4. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, `GEMINI.md` for Gemini CLI, `QWEN.md` for Qwen Code or `AGENTS.md` for opencode).
7. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
Implement four core infrastructure modules for caro: (1) Hugging Face model caching with offline support and directory management, (2) configuration management with CLI integration and user preferences, (3) execution context with environment capture and shell detection, and (4) structured logging with tracing integration for observability. These modules provide the foundational infrastructure for all other caro components.

## Technical Context
**Language/Version**: Rust 1.75+ (2021 edition)
**Primary Dependencies**: serde/serde_json (serialization), tokio (async runtime), directories (XDG paths), toml (config parsing), tracing/tracing-subscriber (logging)
**Storage**: File-based (cache manifest, TOML config files, JSON logs)
**Testing**: cargo test with tokio-test for async, proptest for property-based validation
**Target Platform**: macOS, Linux (primary), Windows (secondary)
**Project Type**: Single project (library-first architecture via lib.rs exports)
**Performance Goals**: Cache <5s for <1GB models, config load <100ms, context capture <50ms, non-blocking async logging
**Constraints**: XDG-compliant paths, offline-capable, graceful degradation on failures, zero unsafe Rust
**Scale/Scope**: 4 infrastructure modules, ~1000 LOC total, foundational for all caro features

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Simplicity ✅ PASS
- **Single project structure**: All 4 modules within existing `src/` hierarchy
- **No wrapper abstractions**: Using dependencies directly (serde, tokio, directories, toml, tracing)
- **Single data flow**: Configuration → Context → Execution → Logging (linear pipeline)
- **No organizational-only patterns**: No repositories, DTOs, or UoW—just functional modules

### II. Library-First Architecture ✅ PASS
- **All modules exported via lib.rs**: `caro::cache`, `caro::config`, `caro::execution`, `caro::logging`
- **Self-contained libraries**: Each module has clear public API and single responsibility
- **Binary orchestration**: `main.rs` uses libraries, contains no business logic
- **Reusability**: Modules can be used programmatically beyond CLI

### III. Test-First (NON-NEGOTIABLE) ✅ PASS
- **TDD methodology enforced**: Contract tests → Integration tests → Implementation
- **Strict ordering**: Tests written and verified failing before any implementation
- **Real dependencies**: Using actual filesystems (with tempdir), real async runtime
- **Integration tests required**: Cross-module communication (config + context, cache + logging)

### IV. Safety-First Development ✅ PASS
- **No unsafe Rust**: All infrastructure modules use safe Rust
- **Path validation**: XDG directory resolution with permission checking
- **Error handling**: All file operations return Results with user-friendly messages
- **Graceful degradation**: Modules continue with defaults on failures (NFR-009)

### V. Observability & Versioning ✅ PASS
- **Structured logging with tracing**: Logging module uses tracing crate throughout
- **Error context chains**: Using anyhow for binary context, thiserror for library errors
- **Performance instrumentation**: Timing logs for cache operations, config loading, context capture
- **User-facing messages**: Clear error messages distinct from debug logs

**Initial Assessment**: ✅ **CONSTITUTIONAL COMPLIANCE VERIFIED** - No violations detected

## Project Structure

### Documentation (this feature)
```
specs/[###-feature]/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
src/
├── lib.rs                  # Library exports (add cache, config, execution, logging)
├── models/                 # Existing - data types (extend with infrastructure models)
├── cache/                  # NEW - Model caching with HF integration
│   ├── mod.rs             # CacheManager, model download, integrity validation
│   └── manifest.rs        # Cache manifest management
├── config/                 # NEW - Configuration management
│   ├── mod.rs             # ConfigManager, TOML parsing, validation
│   └── schema.rs          # Configuration schema definitions
├── execution/              # NEW - Execution context capture
│   ├── mod.rs             # ExecutionContext, environment detection
│   └── shell.rs           # Shell type detection logic
├── logging/                # NEW - Structured logging
│   ├── mod.rs             # Logger initialization, tracing setup
│   └── redaction.rs       # Sensitive data redaction logic
├── safety/                 # Existing - command validation
├── backends/               # Existing - LLM backends
└── cli/                    # Existing - CLI orchestration (will use new modules)

tests/
├── contract/               # Contract tests for module APIs
│   ├── cache_contract.rs  # NEW - Cache module public API tests
│   ├── config_contract.rs # NEW - Config module public API tests
│   ├── execution_contract.rs # NEW - Execution module public API tests
│   └── logging_contract.rs # NEW - Logging module public API tests
├── integration/            # Cross-module integration tests
│   └── infrastructure_integration.rs # NEW - Config + Context + Cache + Logging
└── unit/                   # Edge case testing (property-based where applicable)
```

**Structure Decision**: Single project structure maintained. All four infrastructure modules added to existing `src/` hierarchy with corresponding contract tests. Library-first architecture enforced through `lib.rs` exports. Each module self-contained with clear public API.

## Phase 0: Outline & Research
1. **Extract unknowns from Technical Context** above:
   - For each NEEDS CLARIFICATION → research task
   - For each dependency → best practices task
   - For each integration → patterns task

2. **Generate and dispatch research agents**:
   ```
   For each unknown in Technical Context:
     Task: "Research {unknown} for {feature context}"
   For each technology choice:
     Task: "Find best practices for {tech} in {domain}"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with all NEEDS CLARIFICATION resolved

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

1. **Extract entities from feature spec** → `data-model.md`:
   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:
   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Generate contract tests** from contracts:
   - One test file per endpoint
   - Assert request/response schemas
   - Tests must fail (no implementation yet)

4. **Extract test scenarios** from user stories:
   - Each story → integration test scenario
   - Quickstart test = story validation steps

5. **Update agent file incrementally** (O(1) operation):
   - Run `.specify/scripts/bash/update-agent-context.sh claude`
     **IMPORTANT**: Execute it exactly as specified above. Do not add or remove any arguments.
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, agent-specific file

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
1. Load `.specify/templates/tasks-template.md` as base template
2. Generate tasks from Phase 1 artifacts:
   - **From contracts/**: 4 contract test files → 4 contract test tasks [P]
   - **From data-model.md**: 6 entities + 4 enums → 10 model definition tasks
   - **From quickstart.md**: 9 scenarios → 9 integration test tasks
   - **Implementation tasks**: 4 modules × 2 files each = 8 implementation tasks
   - **Cross-module tasks**: Exports, documentation, validation

**Detailed Task Breakdown**:

### TDD RED Phase (Contract Tests) - Tasks 1-4 [P]
- T001: Write cache module contract tests (cache_contract.rs)
- T002: Write config module contract tests (config_contract.rs)
- T003: Write execution module contract tests (execution_contract.rs)
- T004: Write logging module contract tests (logging_contract.rs)
All tests must FAIL initially (no implementation yet)

### Model Definitions - Tasks 5-14 [Some P]
- T005: Define CachedModel struct with validation (models/mod.rs)
- T006: Define CacheManifest struct with operations (models/mod.rs)
- T007: Define UserConfiguration struct with builder (models/mod.rs)
- T008: Define ConfigSchema struct with validators (models/mod.rs)
- T009: Define ExecutionContext struct with capture (models/mod.rs)
- T010: Define LogEntry struct with serialization (models/mod.rs)
- T011: Extend ShellType enum with detection (models/mod.rs)
- T012: Define Platform enum with detection (models/mod.rs)
- T013: Define LogLevel enum with conversions (models/mod.rs)
- T014: Update lib.rs exports for infrastructure models

### TDD GREEN Phase (Implementation) - Tasks 15-22
- T015: Implement cache module (cache/mod.rs, cache/manifest.rs)
- T016: Implement config module (config/mod.rs, config/schema.rs)
- T017: Implement execution module (execution/mod.rs, execution/shell.rs)
- T018: Implement logging module (logging/mod.rs, logging/redaction.rs)
- T019: Update lib.rs exports for all infrastructure modules
- T020: Add new dependencies to Cargo.toml (directories, toml, tracing, etc.)
- T021: Verify all contract tests pass (cargo test --test cache_contract, etc.)
- T022: Run cargo fmt and cargo clippy

### Integration Tests - Tasks 23-31
- T023: Write Scenario 1 test: First-time user experience
- T024: Write Scenario 2 test: Returning user with cache
- T025: Write Scenario 3 test: CLI argument override
- T026: Write Scenario 4 test: Context-aware generation
- T027: Write Scenario 5 test: Structured logging operations
- T028: Write Scenario 6 test: Cache size limit & LRU
- T029: Write Scenario 7 test: Sensitive data redaction
- T030: Write Scenario 8 test: Config validation errors
- T031: Write Scenario 9 test: Full cross-module integration

### Validation & Documentation - Tasks 32-35
- T032: Run full test suite (cargo test)
- T033: Verify performance requirements (cargo test -- --nocapture, check timings)
- T034: Update CHANGELOG.md with Feature 003 additions
- T035: Create PR for Feature 003

**Ordering Strategy**:
1. **Parallel Phase 1** (T001-T004): Contract tests can be written simultaneously
2. **Sequential Phase 2** (T005-T014): Models have some dependencies (enums before structs)
3. **Parallel Phase 3** (T015-T018): Each module implementation is independent
4. **Sequential Phase 4** (T019-T022): Exports and verification must be sequential
5. **Parallel Phase 5** (T023-T031): Integration tests can run in parallel
6. **Sequential Phase 6** (T032-T035): Final validation and documentation

**Dependency Graph**:
```
T001-T004 (contract tests) [P]
    ↓
T005-T014 (models) [Some P: enums first, then structs]
    ↓
T015-T018 (implementations) [P]
    ↓
T019-T022 (integration & verification) [Sequential]
    ↓
T023-T031 (integration tests) [P]
    ↓
T032-T035 (validation & docs) [Sequential]
```

**Estimated Output**: 35 numbered, dependency-ordered tasks in tasks.md

**Performance Validation Strategy**:
- Each task includes acceptance criteria with performance targets
- Integration tests measure and assert performance requirements
- Tasks 32-33 specifically validate NFR-001 through NFR-010

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |


## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented (None - no violations)

---
*Based on Constitution v1.0.0 - See `.specify/memory/constitution.md`*
