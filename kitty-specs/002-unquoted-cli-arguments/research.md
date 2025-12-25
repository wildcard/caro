# Research Decision Log

Document the outcomes of Phase 0 discovery work. Capture every clarification you resolved and the supporting evidence that backs each decision.

## Summary

- **Feature**: 002-unquoted-cli-arguments
- **Date**: 2025-12-25
- **Researchers**: Claude Sonnet 4.5, User
- **Open Questions**: None - all critical decisions resolved during planning

## Decisions & Rationale

| Decision | Rationale | Evidence | Status |
|----------|-----------|----------|--------|
| Use fresh implementation vs fixing PR #68 | PR #68 has test failures across all platforms; starting fresh allows us to avoid inherited issues while using PR #68 as reference | User confirmed "A sounds reasonable" for fresh approach | final |
| Minimal changes using clap's trailing_var_arg | Reduces complexity and risk; keeps changes isolated to main.rs and cli/mod.rs; leverage's clap's built-in capabilities | User confirmed Option A (minimal changes) | final |
| Input prioritization: -p flag > stdin > trailing args | Follows Unix conventions used by tools like cat, grep; explicit flag overrides piped input | POSIX best practices, standard Unix behavior | final |
| Shell operator detection via custom parsing | BSD/GNU conventions for metacharacters (>, |, <, >>, 2>, &, ;); POSIX shell specification defines these | User specified "follow BSD/GNU best practices" | final |
| Empty/whitespace input shows help | Consistent with standard CLI behavior; non-error response for missing input | User confirmed "show help if empty" | final |
| Backward compatibility mandatory | Existing quoted prompt usage must continue working without modification | Spec requirement FR-002, success criteria SC-002 | final |

## Evidence Highlights

### Key Insight 1: PR #68 Test Failures

**Finding**: PR #68 changes 15 files with 800+ lines of modifications and has test failures on Linux, macOS, and Windows.

**Evidence**:
- GitHub PR #68 review showed failing CI checks across all platforms
- Large scope (15 files) increases risk of regressions
- Test failures suggest implementation issues need diagnosis

**Decision Impact**: Confirms fresh implementation approach is lower risk than debugging existing PR.

### Key Insight 2: clap's trailing_var_arg Feature

**Finding**: clap 4.5+ provides `trailing_var_arg = true` and `num_args = 0..` for capturing unquoted trailing arguments.

**Evidence**:
- clap documentation: https://docs.rs/clap/latest/clap/
- Existing usage in tools like ripgrep, fd

**Decision Impact**: Leverages battle-tested library feature instead of custom argument parsing.

### Key Insight 3: Shell Operator Handling Requirements

**Finding**: Unix shell operators (>, |, <, >>, 2>, &, ;) must terminate prompt parsing per POSIX conventions.

**Evidence**:
- POSIX Shell Command Language specification
- BSD/GNU tool behavior (bash, zsh, sh)
- User requirement: "shell operator of all kind must continue to work"

**Decision Impact**: Requires custom detection logic; clap stops at first non-flag argument, so we need post-processing.

### Key Insight 4: Multiple Input Sources

**Finding**: Three input methods need support: -p/--prompt flag, stdin, and trailing arguments.

**Evidence**:
- User requirement: "allow user to pass prompt as stdin or -p --prompt arg"
- Unix conventions: cat, grep prioritize explicit flags > stdin > arguments
- Non-interactive mode needed for automation

**Decision Impact**: Input source detection and prioritization logic required in main.rs.

### Risks / Concerns

1. **Shell operator detection complexity**: May be tricky to handle all edge cases correctly across different shells
2. **Test coverage**: Need comprehensive tests for all input methods and shell operator combinations
3. **Cross-platform compatibility**: Windows PowerShell has different operators than Unix shells
4. **Breaking changes**: Must ensure no regressions for existing quoted prompt users

## Investigation Tasks for Implementation

### Task 1: Analyze PR #68 Changes
**Purpose**: Understand what approach was taken and why tests failed

**Actions**:
- Review PR #68 diff focusing on src/main.rs and src/cli/mod.rs changes
- Identify test failures and root causes
- Extract any good patterns to reuse

**Expected Outcome**: List of issues to avoid and patterns to adopt

### Task 2: Shell Operator Detection Research
**Purpose**: Determine how to reliably detect shell metacharacters

**Actions**:
- Document complete list of shell operators per POSIX spec
- Determine detection strategy (regex, character scanning, etc.)
- Handle edge cases (operators in quotes, escaped operators)

**Expected Outcome**: Shell operator detection algorithm

### Task 3: Test Strategy for Multiple Input Methods
**Purpose**: Ensure comprehensive coverage of all input scenarios

**Actions**:
- Define test cases for -p flag, stdin, trailing args, and combinations
- Cover empty input, whitespace, special characters
- Test cross-platform behavior (Linux, macOS, Windows)

**Expected Outcome**: Test plan covering all FR requirements

## Next Actions

1. **Proceed to Phase 1 (Design)**: Create data-model.md and contracts for argument parsing flow
2. **Generate task breakdown**: Use /spec-kitty.tasks to create implementation work packages
3. **Investigation during implementation**: Analyze PR #68 in detail when starting implementation

## Technical Constraints Confirmed

- **Language**: Rust 1.75+ (project standard)
- **CLI Framework**: clap 4.5+ (existing dependency)
- **Testing Framework**: cargo test with tokio-test (existing)
- **Performance**: Argument parsing must complete in <10ms (startup overhead budget)
- **Platforms**: Linux, macOS (both x86_64 and aarch64), Windows

## References

- **PR #68**: https://github.com/wildcard/caro/pull/68
- **clap documentation**: https://docs.rs/clap/latest/clap/
- **POSIX Shell spec**: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html
- **Project constitution**: .specify/memory/constitution.md (v1.0.0)

> Research phase complete. All critical decisions finalized. Ready for design and task breakdown.
