# Implementation Plan: Unquoted CLI Arguments

**Branch**: `002-unquoted-cli-arguments` | **Date**: 2025-12-25 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `kitty-specs/002-unquoted-cli-arguments/spec.md`

**Planning Status**: All discovery and planning questions resolved. Technical approach confirmed with user.

## Summary

Enable users to type natural language prompts without quotes (`caro list files`) while maintaining full backward compatibility with quoted prompts (`caro "list files"`). Implementation will use a fresh codebase approach with minimal changes to `src/main.rs` and `src/cli/mod.rs`, leveraging clap's `trailing_var_arg` feature. The feature supports three input methods (-p flag, stdin, trailing args) with Unix-convention prioritization, handles shell operators per POSIX/BSD/GNU standards, and includes comprehensive cross-platform testing.

## Technical Context

**Language/Version**: Rust 1.75+ (project standard)
**Primary Dependencies**: clap 4.5+ (CLI argument parsing with `trailing_var_arg`), tokio (async runtime), serde/serde_json (JSON serialization)
**Storage**: N/A (transient command-line processing only)
**Testing**: cargo test with tokio-test (existing test infrastructure)
**Target Platform**: Linux, macOS (x86_64 + aarch64), Windows
**Project Type**: Single binary CLI application
**Performance Goals**: Argument parsing must complete in <10ms (startup overhead budget)
**Constraints**:
  - Backward compatibility: All existing quoted prompt usage must work unchanged
  - Cross-platform: Tests must pass on Linux, macOS, Windows
  - Safety-first: Shell operator detection must follow POSIX/BSD/GNU best practices
  - Library-first: Core parsing logic should be testable independent of CLI interface
**Scale/Scope**:
  - 2 primary files modified (`src/main.rs`, `src/cli/mod.rs`)
  - 12 functional requirements (FR-001 through FR-012)
  - 7 success criteria with measurable outcomes
  - ~50-100 LOC net addition (minimal changes approach)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Constitution Version**: v1.0.0 (`.specify/memory/constitution.md`)

### Principle I: Simplicity
**Status**: ✅ PASS
- Minimal changes approach: Only 2 files modified (`src/main.rs`, `src/cli/mod.rs`)
- Leverages existing clap feature (`trailing_var_arg`) instead of custom parsing
- No new abstractions or design patterns introduced
- Estimated ~50-100 LOC net addition (focused, targeted changes)

### Principle II: Library-First Architecture
**Status**: ✅ PASS
- Argument parsing logic will be separated from CLI interface
- Prompt source resolution (flag > stdin > args) implemented as pure function
- Shell operator detection will be testable independent of clap
- Follows existing caro pattern: `src/cli/mod.rs` handles arg parsing, `src/main.rs` orchestrates

### Principle III: Test-First Development (TDD)
**Status**: ✅ PASS
- Comprehensive test plan defined in spec.md (5 user stories with acceptance scenarios)
- Will follow RED-GREEN-REFACTOR cycle for each functional requirement
- Existing test infrastructure (`cargo test`, contract tests) will be used
- Cross-platform CI validation required (Linux, macOS, Windows)

### Principle IV: Safety-First Development
**Status**: ✅ PASS
- Shell operator detection follows POSIX/BSD/GNU standards (no custom unsafe parsing)
- No changes to existing safety validation system
- Backward compatibility ensures no regressions in safety checks
- Edge case handling documented (empty input, whitespace, operators)

### Principle V: Observability & Versioning
**Status**: ✅ PASS
- Argument parsing errors will use existing logging infrastructure
- Changes will be backward compatible (no breaking API changes)
- Semantic versioning: This is a minor version feature (new capability, no breaking changes)
- Logging will capture which input source was used (flag/stdin/args) for troubleshooting

**Overall Assessment**: ✅ ALL GATES PASSED - No constitutional violations. Approach aligns with all five core principles.

## Project Structure

### Documentation (this feature)

```
kitty-specs/002-unquoted-cli-arguments/
├── plan.md              # This file (Phase 0-1 output) ✅ COMPLETE
├── research.md          # Phase 0 output ✅ COMPLETE
├── data-model.md        # Phase 0 output ✅ COMPLETE
├── quickstart.md        # Phase 1 output ⏳ PENDING
├── spec.md              # Feature specification ✅ COMPLETE
├── checklists/
│   └── requirements.md  # Spec quality validation ✅ COMPLETE
├── contracts/           # Phase 1 output ⏳ PENDING
└── research/
    ├── evidence-log.csv     # Research findings log ✅ COMPLETE
    └── source-register.csv  # Source reference tracking ✅ COMPLETE
```

**Note**: `tasks.md` will be generated later by `/spec-kitty.tasks` command (Phase 2).

### Source Code (repository root)

**Structure**: Single binary CLI application (Rust project)

```
src/
├── main.rs              # CLI entry point - WILL MODIFY (prompt parsing integration)
├── lib.rs               # Library root
├── cli/
│   └── mod.rs          # CLI argument parsing - WILL MODIFY (clap config changes)
├── agent/              # Agent module (unchanged)
├── backends/           # LLM backends (unchanged)
├── cache/              # Model caching (unchanged)
├── config/             # Configuration (unchanged)
├── context/            # Context management (unchanged)
├── execution/          # Command execution (unchanged)
├── logging/            # Logging infrastructure (unchanged)
├── models/             # Model definitions (unchanged)
├── platform/           # Platform detection (unchanged)
└── safety/             # Safety validation (unchanged)

tests/
├── backend_trait_contract.rs      # Backend contract tests (unchanged)
├── cache_contract.rs              # Cache contract tests (unchanged)
├── cli_interface_contract.rs      # CLI contract tests - MAY UPDATE (new argument patterns)
├── config_contract.rs             # Config contract tests (unchanged)
├── e2e_cli_tests.rs               # E2E CLI tests - WILL UPDATE (new test cases for unquoted prompts)
├── e2e_interactive_execution.rs   # Interactive execution tests - MAY UPDATE
├── execution_prompt_behavior.rs   # Prompt behavior tests - WILL UPDATE (test all input methods)
├── integration_tests.rs           # Integration tests - MAY UPDATE
└── [other contract tests...]      # (unchanged)
```

**Structure Decision**: This is a single binary Rust CLI application. Changes will be isolated to argument parsing layer (`src/cli/mod.rs` for clap configuration) and orchestration layer (`src/main.rs` for prompt source resolution). All other modules remain unchanged, maintaining separation of concerns per Library-First Architecture principle.

## Complexity Tracking

*Not applicable - no constitutional violations identified.*

## Implementation Approach (from research.md)

### Decision: Fresh Implementation vs Fixing PR #68

**Selected**: Fresh implementation (Option A)
**Rationale**: PR #68 has test failures across all platforms (Linux, macOS, Windows) and changes 15 files with 800+ lines. Starting fresh allows us to:
- Avoid inherited test issues
- Keep changes minimal (2 files vs 15 files)
- Use PR #68 as reference while avoiding its problems
- Follow TDD from the start with clean slate

### Decision: Minimal Changes Approach

**Selected**: Minimal changes using clap's `trailing_var_arg` (Option A)
**Rationale**:
- Reduces complexity and implementation risk
- Keeps changes isolated to `main.rs` and `cli/mod.rs`
- Leverages clap's built-in capabilities (battle-tested)
- Aligns with Constitution Principle I (Simplicity)

### Key Technical Decisions

1. **Input Prioritization**: Flag (-p) > Stdin > Trailing Args
   - Follows Unix conventions (cat, grep, etc.)
   - Explicit flag takes highest priority
   - Stdin allows piping workflows
   - Trailing args as default/fallback

2. **Shell Operator Detection**: Custom parsing for POSIX metacharacters
   - Operators: `>`, `|`, `<`, `>>`, `2>`, `&`, `;`
   - Follow BSD/GNU conventions
   - Terminates prompt parsing when detected
   - Allows shell redirection to work: `caro list files > output.txt`

3. **Empty Input Handling**: Show help (not error)
   - Non-punitive user experience
   - Consistent with standard CLI behavior
   - Empty string and whitespace-only both trigger help

4. **Backward Compatibility**: Mandatory preservation
   - All existing `caro "quoted prompt"` usage must work unchanged
   - No breaking changes to API or CLI interface
   - Existing tests must continue passing

## Phase 1 Design Artifacts

### Still Required:
- **quickstart.md** - Developer quick-start guide for testing the feature
- **contracts/** - API contracts defining expected behavior for each input method

These will be created next as part of Phase 1 completion.
