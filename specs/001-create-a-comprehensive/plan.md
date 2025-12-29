# Implementation Plan: caro - Natural Language to Shell Command CLI Tool


**Branch**: `001-create-a-comprehensive` | **Date**: 2025-09-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/workspaces/caro/specs/001-create-a-comprehensive/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
4. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
5. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, or `GEMINI.md` for Gemini CLI).
6. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
7. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
8. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
A Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs. Primary focus on safety-first command validation, multi-backend LLM support (MLX for Apple Silicon, vLLM, Ollama), and production-ready performance targets (<50MB binary, <100ms startup, <2s inference). Single binary distribution with offline capability and comprehensive command safety validation.

## Technical Context
**Language/Version**: Rust 1.75+ with 2021 edition for performance and safety  
**Primary Dependencies**: clap v4.4+ (CLI), tokio (async runtime), serde (serialization), reqwest (HTTP), anyhow (error handling)  
**Storage**: Local file system for model caching, configuration files (TOML), command history logs  
**Testing**: cargo test with tokio-test for async, property-based testing for safety validation  
**Target Platform**: Primary: macOS with Apple Silicon (M1/M2/M3/M4), Secondary: Linux (x86_64, ARM64), Tertiary: Windows
**Project Type**: single - CLI application with modular backend system  
**Performance Goals**: <100ms startup time, <2s inference time, >90% command accuracy  
**Constraints**: <50MB binary size, <500MB memory usage, offline-capable, POSIX compliance  
**Scale/Scope**: Single-user CLI tool, support for 3+ LLM backends, comprehensive safety rule database

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Simplicity**:
- Projects: 1 (single CLI binary)
- Using framework directly? YES (clap, tokio, serde directly without wrappers)
- Single data model? YES (unified command/response structures)
- Avoiding patterns? YES (direct implementations, no unnecessary abstractions)

**Architecture**:
- EVERY feature as library? YES (backend traits, safety module, cache management)
- Libraries listed: [caro-core (main logic), caro-backends (LLM integration), caro-safety (validation), caro-cache (model management)]
- CLI per library: Single `caro` command with --help/--version/--backend flags
- Library docs: llms.txt format planned for agent integration

**Testing (NON-NEGOTIABLE)**:
- RED-GREEN-Refactor cycle enforced? YES (TDD mandatory for safety validation)
- Git commits show tests before implementation? YES (test-first development)
- Order: Contract→Integration→E2E→Unit strictly followed? YES
- Real dependencies used? YES (actual LLM backends, real file system)
- Integration tests for: new backends, safety rules, command generation
- FORBIDDEN: Implementation before test, skipping RED phase

**Observability**:
- Structured logging included? YES (tracing crate with configurable levels)
- Frontend logs → backend? N/A (single CLI application)
- Error context sufficient? YES (detailed error messages with suggestions)

**Versioning**:
- Version number assigned? YES (0.1.0 - MAJOR.MINOR.BUILD format)
- BUILD increments on every change? YES
- Breaking changes handled? YES (configuration migration, compatibility checks)

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
# Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure]
```

**Structure Decision**: [DEFAULT to Option 1 unless Technical Context indicates web/mobile app]

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
   - Run `/scripts/bash/update-agent-context.sh claude` for your AI assistant
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, agent-specific file

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
- Load `/templates/tasks-template.md` as base structure
- Generate tasks from Phase 1 artifacts: contracts/, data-model.md, quickstart.md
- Contract tests: backend-trait.rs → backend contract tests [P]
- Contract tests: safety-validator.rs → safety validation tests [P]  
- Contract tests: cli-interface.rs → CLI argument parsing tests [P]
- Data model: Core entities → Rust struct definitions [P]
- Integration tests: quickstart scenarios → end-to-end test cases
- Implementation tasks: Make failing tests pass (TDD cycle)

**Specific Task Categories**:
1. **Contract Test Tasks** [P - Parallel execution]:
   - Backend trait contract tests (MLX, vLLM, Ollama)
   - Safety validator contract tests (dangerous pattern detection)
   - CLI interface contract tests (argument parsing, subcommands)

2. **Core Model Tasks** [P - Parallel execution]:
   - CommandRequest/GeneratedCommand structures
   - SafetyRule/ValidationResult models
   - Configuration and cache management models

3. **Infrastructure Tasks** (Sequential):
   - Project structure and Cargo.toml setup
   - Error handling and logging framework
   - Configuration loading hierarchy

4. **Backend Implementation Tasks** (Sequential, depends on contracts):
   - Backend trait system implementation
   - MLX backend with FFI bindings
   - vLLM HTTP client backend
   - Ollama local API backend

5. **Safety System Tasks** (Sequential, depends on models):
   - Safety rule database and pattern matching
   - Risk assessment and validation pipeline
   - User confirmation and override system

6. **CLI Implementation Tasks** (Sequential, depends on backends):
   - Main CLI application with clap
   - Subcommand handlers (config, models, backends)
   - Output formatting and user interaction

7. **Integration Tasks** (Sequential, depends on all components):
   - End-to-end command generation workflow
   - Model caching and download system
   - Performance optimization and benchmarking

**Ordering Strategy**:
- **Phase A**: Contract tests (all [P] - parallel execution)
- **Phase B**: Core models and infrastructure (foundation first)
- **Phase C**: Backend implementations (can be partially parallel)
- **Phase D**: Safety and CLI systems (depends on backends)
- **Phase E**: Integration and optimization (final assembly)

**TDD Enforcement**:
- Every implementation task preceded by failing test task
- Contract tests must fail before any implementation
- Integration tests validate acceptance criteria from spec
- Performance tests validate benchmarks from quickstart

**Estimated Output**: 
- 35-40 numbered, ordered tasks in tasks.md
- 8-10 contract test tasks [P]
- 12-15 implementation tasks following TDD cycle
- 8-10 integration and validation tasks
- 5-7 optimization and polish tasks

**Dependencies Management**:
- Clear task numbering shows execution order
- [P] markers indicate safe parallel execution
- Dependency chains: Contracts → Models → Backends → CLI → Integration
- Each phase gates on previous completion

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
- [x] Complexity deviations documented (none required)

**Artifacts Generated**:
- [x] research.md - Technical decisions and rationale
- [x] data-model.md - Core entities and relationships
- [x] contracts/backend-trait.rs - Backend interface specification
- [x] contracts/safety-validator.rs - Safety validation interface
- [x] contracts/cli-interface.rs - Command-line interface specification
- [x] quickstart.md - User scenarios and validation tests

---
*Based on Constitution v1.0.0 - See `/workspaces/caro/.specify/memory/constitution.md`*