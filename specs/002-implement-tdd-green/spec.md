# Feature Specification: Implement TDD GREEN Phase - Core Models and Safety System

**Feature ID**: 002
**Status**: Planning
**Priority**: High
**Created**: 2025-10-01

## Overview

Implement the GREEN phase of Test-Driven Development for caro, making the 80 comprehensive contract tests pass by building the core data models, safety validation system, and backend trait implementations. This phase transforms the RED phase foundation into working functionality while maintaining strict TDD discipline.

## Background

The TDD RED phase (feature 001) established:
- 80 comprehensive contract tests across all components
- Backend trait contracts (11 tests)
- Safety validator contracts (17 tests - including new cross-platform tests)
- CLI interface contracts (14 tests)
- Integration tests (8 tests)
- Property-based tests (10 tests)
- Error handling tests (11 tests)
- Performance benchmarks (9 tests)

All tests are currently failing with `NotImplemented` errors, providing clear contracts for implementation.

## User Stories

### US-001: As a developer, I want core data models implemented
**Acceptance Criteria**:
- `CommandRequest`, `GeneratedCommand`, `ShellType`, `RiskLevel`, `SafetyLevel` types exist
- `BackendInfo`, `BackendType` enums defined
- All types are serializable/deserializable with serde
- Types implement required traits (Debug, Clone, PartialEq where appropriate)
- Model tests pass

### US-002: As a developer, I want the safety validation system working
**Acceptance Criteria**:
- `SafetyValidator` can detect dangerous command patterns
- Risk level assessment works (Safe, Moderate, High, Critical)
- POSIX and Windows path validation implemented
- Cross-platform dangerous pattern detection
- Custom safety patterns configurable
- Network-independent validation (works offline)
- All safety validator contract tests pass

### US-003: As a developer, I want backend trait system operational
**Acceptance Criteria**:
- `CommandGenerator` trait fully defined
- Mock backend implementation for testing
- Backend availability checking works
- Async command generation functional
- Error types properly defined
- All backend contract tests pass

### US-004: As a developer, I want basic CLI structure working
**Acceptance Criteria**:
- `CliApp` struct with initialization
- Command-line argument parsing with clap
- Configuration file loading
- Output formatting (JSON, YAML, plain text)
- Verbose mode logging
- CLI contract tests pass

### US-005: As a system, I want error handling comprehensive
**Acceptance Criteria**:
- Custom error types for all failure modes
- Error serialization for logging/debugging
- Graceful error recovery
- User-friendly error messages
- All error handling tests pass

## Functional Requirements

### FR-001: Core Data Models
- Define all model types in `src/models/mod.rs`
- Implement serde serialization/deserialization
- Add validation logic where appropriate
- Ensure types are Send + Sync for async usage

### FR-002: Safety Validation System
- Implement `SafetyValidator` in `src/safety/mod.rs`
- Add dangerous command pattern database
- Implement risk level assessment algorithm
- Add POSIX compliance checking
- Support custom safety patterns via configuration
- Ensure <50ms validation performance

### FR-003: Backend Trait System
- Define `CommandGenerator` trait in `src/backends/mod.rs`
- Implement mock backend for testing
- Add error types for backend failures
- Implement availability checking
- Support async trait methods

### FR-004: CLI Interface
- Implement `CliApp` in `src/cli/mod.rs`
- Add clap-based argument parsing
- Implement configuration loading from TOML
- Add output format handlers
- Integrate with logging system

### FR-005: Error Handling
- Define error types in each module
- Implement `thiserror` for error derivation
- Add error serialization support
- Ensure errors provide actionable context

## Non-Functional Requirements

### NFR-001: Performance
- CLI startup time < 100ms
- Safety validation < 50ms per command
- Command generation < 2s (with mock backend)
- Memory usage < 100MB during normal operation

### NFR-002: Code Quality
- All public APIs documented with rustdoc
- Comprehensive error messages
- No panics in production code
- Follow Rust idioms and best practices

### NFR-003: Testing
- Maintain TDD discipline (implement to make tests pass)
- All 80 contract tests must pass
- No implementation before test
- Property-based tests must validate invariants

### NFR-004: Maintainability
- Clear module organization
- Separation of concerns
- Minimal coupling between modules
- Extensible design for future backends

## Technical Constraints

### TC-001: Dependencies
- Use only dependencies already defined in Cargo.toml
- Prefer standard library when possible
- Minimize feature bloat

### TC-002: Compatibility
- Rust 1.75+ with 2021 edition
- Cross-platform (macOS, Linux, Windows)
- Offline-capable (no network required for validation)

### TC-003: Safety
- No unsafe code without documentation
- All file I/O properly error-handled
- Command execution validation mandatory

## Out of Scope

- LLM backend implementations (MLX, vLLM, Ollama) - Phase 2
- Command execution functionality - Phase 3
- Model downloading from Hugging Face - Phase 3
- Performance optimization beyond basic requirements - Phase 4
- Advanced CLI features (history, aliases) - Future

## Success Criteria

1. **All 80 contract tests pass**
2. **Zero test failures** in CI pipeline
3. **Code coverage** > 80% for implemented modules
4. **Performance targets met** (startup, validation times)
5. **No clippy warnings** with default settings
6. **Documentation complete** for all public APIs
7. **TDD discipline maintained** (git history shows tests before implementation)

## Dependencies

- Feature 001 (TDD RED Phase) - **COMPLETED**
- Cargo.toml with all required dependencies - **AVAILABLE**
- CI/CD pipeline - **CONFIGURED**

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Tests reveal design flaws | High | Refactor models iteratively, maintain backward compatibility |
| Performance targets not met | Medium | Profile code, optimize hot paths, consider lazy loading |
| Cross-platform issues | Medium | Test on all platforms, use platform-agnostic code |
| Scope creep | Low | Strict adherence to "make tests pass" philosophy |

## Implementation Phases

### Phase 0: Research (if needed)
- Review Rust best practices for async traits
- Research regex patterns for command validation
- Understand clap v4 advanced features

### Phase 1: Design
- Define complete data model with all types
- Design safety validation architecture
- Plan error handling strategy
- Create integration test scenarios

### Phase 2: Implementation (TDD GREEN)
- Implement models module (T001-T010)
- Build safety validator (T011-T025)
- Create backend trait system (T026-T035)
- Implement CLI interface (T036-T045)
- Add error handling (T046-T055)

### Phase 3: Integration
- Wire modules together
- Run integration tests
- Fix integration issues
- Performance validation

### Phase 4: Polish
- Documentation pass
- Error message refinement
- Code cleanup and refactoring
- Final test verification

## Metrics

- **Test Pass Rate**: Target 100% (80/80 tests)
- **Code Coverage**: Target >80%
- **Startup Time**: Target <100ms
- **Validation Time**: Target <50ms per command
- **Lines of Code**: Estimate ~2000-3000 LOC
- **Documentation Coverage**: Target 100% of public APIs

## References

- Feature 001: TDD RED Phase implementation
- CLAUDE.md: Project guidance document
- Cargo.toml: Dependency specification
- tests/*: Contract test specifications
