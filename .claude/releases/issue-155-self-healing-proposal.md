# Issue #155: Self-Healing Feature - Implementation Proposal

**Date**: 2026-01-08
**Assessor**: Claude (Tech Lead)
**Status**: 🟡 Needs Specification & Scope Decision

## Problem Statement

Issue #155 requests "self-healing capability for error recovery and command correction" but lacks detailed specification of:
- What types of errors should trigger self-healing
- How autonomous corrections should be (auto-apply vs. suggest)
- What the scope is (single retry vs. multi-step recovery)

## Current Error Infrastructure

### Existing Capabilities

**Execution Errors** (`src/execution/executor.rs`):
```rust
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub success: bool,
}

pub enum ExecutorError {
    SpawnError(String),
    WaitError(String),
    Timeout(u64),
    InvalidCommand(String),
}
```

**Safety Validation** (`src/safety/mod.rs`):
```rust
pub struct ValidationResult {
    pub allowed: bool,
    pub risk_level: RiskLevel,
    pub explanation: String,
    pub warnings: Vec<String>,
    pub matched_patterns: Vec<String>,
    pub confidence_score: f32,
}
```

**Platform Detection**:
- Shell type detection (Bash, Zsh, Fish, PowerShell, etc.)
- Platform detection (Linux, macOS, Windows)
- Environment variable access

### What's Missing for Self-Healing

1. **Error Pattern Detection**: Categorizing failure types from stderr/exit codes
2. **Correction Suggestion Engine**: Mapping errors to suggested fixes
3. **Retry Mechanism**: Re-executing with corrected commands
4. **Learning/History**: Storing successful corrections for future use
5. **User Confirmation Workflow**: Asking before applying corrections

## Implementation Options

### Option A: Defer to v1.2.0 (RECOMMENDED)

**Rationale**:
- Issue is too vague for v1.1.0 timeline (5 weeks remaining)
- Requires extensive research and design (spec-kit workflow)
- High complexity: error detection, AI-driven correction, learning system
- Risk of scope creep and delayed release

**Benefits**:
- ✅ Allows proper research and design phase
- ✅ Can leverage ChromaDB (#166) for learning system
- ✅ Time to gather real-world error patterns from users
- ✅ Can coordinate with #280 (alias suggestions) for pattern learning

**Timeline**:
- v1.1.0: Close as deferred
- v1.2.0: Full self-healing feature with research phase

### Option B: Minimal Viable Implementation for v1.1.0

**Scope**: Basic error detection with simple corrections only

**Features**:
1. **Permission Error Detection**
   - Detect "Permission denied" in stderr
   - Suggest: "Try adding sudo: `sudo [command]`"

2. **Command Not Found**
   - Detect "command not found" in stderr
   - Suggest: "Install via: `brew install [cmd]` or `apt install [cmd]`"

3. **File/Directory Not Found**
   - Detect "No such file or directory"
   - Suggest: "Create directory: `mkdir -p [path]`"

4. **Safety Validation Failure**
   - When ValidationResult.allowed == false
   - Display ValidationResult.explanation with safer alternatives

**Architecture** (Minimal):
```rust
pub struct SelfHealingEngine {
    platform: Platform,
    shell_type: ShellType,
}

impl SelfHealingEngine {
    pub fn analyze_failure(&self, result: &ExecutionResult) -> Option<Vec<Suggestion>> {
        // Pattern match on stderr and exit codes
        // Return suggested corrections
    }
}

pub struct Suggestion {
    pub description: String,
    pub corrected_command: Option<String>,
    pub explanation: String,
    pub requires_confirmation: bool,
}
```

**Effort Estimate**: 3-5 days
**Risk**: Low (simple pattern matching, no AI required)

**Acceptance Criteria**:
- [ ] Detect and suggest fixes for 4 common error types
- [ ] User confirmation workflow for suggestions
- [ ] Unit tests for each error pattern
- [ ] Integration tests with real commands
- [ ] Documentation in CONTRIBUTING.md

### Option C: Break Into Sub-Issues

**Create focused issues for specific scenarios**:

1. **#XXX: Suggest sudo for permission errors**
   - Effort: S (< 3 days)
   - Simple stderr pattern matching

2. **#XXX: Suggest installation for command not found**
   - Effort: S (< 3 days)
   - Platform-specific package manager detection

3. **#XXX: Self-healing for safety validation failures**
   - Effort: M (3-7 days)
   - Leverage existing ValidationResult
   - Suggest safer alternatives based on matched patterns

4. **#XXX: Learning system for successful corrections** (v1.2.0)
   - Effort: L (> 7 days)
   - Requires ChromaDB integration (#166)
   - Stores correction history for pattern learning

**Benefits**:
- ✅ Incremental progress
- ✅ Can merge simple fixes into v1.1.0
- ✅ Defer complex learning to v1.2.0
- ✅ Clear acceptance criteria per sub-issue

## Analysis: Option Comparison

| Criteria | Option A (Defer) | Option B (MVP) | Option C (Sub-issues) |
|----------|------------------|----------------|----------------------|
| v1.1.0 Impact | None | Low-Medium | Low (simple fixes only) |
| Complexity | N/A (deferred) | Low | Low-Medium |
| Time Required | N/A | 3-5 days | 1-3 days per sub-issue |
| Risk to v1.1.0 | None | Low | Very Low |
| User Value | None | Medium | Medium (incremental) |
| Scope Clarity | N/A | Clear | Very Clear |
| Future Extensibility | High | Medium | High |

## Recommendation

**Primary Recommendation**: **Option C (Break Into Sub-Issues)** with v1.1.0 scope limited to:
- Sub-issue #1: Suggest sudo for permission errors (S - < 3 days)
- Sub-issue #2: Suggest installation for command not found (S - < 3 days)

**Defer to v1.2.0**:
- Sub-issue #3: Safety validation self-healing (M - 3-7 days)
- Sub-issue #4: Learning system (L - > 7 days, requires #166)

**Rationale**:
1. ✅ Delivers immediate user value (permission and not-found errors are common)
2. ✅ Low risk to v1.1.0 timeline (simple pattern matching)
3. ✅ Clear scope and acceptance criteria
4. ✅ Incremental progress toward full self-healing
5. ✅ Allows deferring complex features without blocking simple ones

**Secondary Recommendation**: **Option A (Defer Entirely)** if Week 2 timeline is tight and we need to focus on #132 (Performance analysis).

## Proposed Sub-Issues for v1.1.0

### Sub-Issue #1: Self-Healing for Permission Errors

**Description**:
When a command fails with "Permission denied", suggest adding sudo.

**Scope**:
- Detect permission errors from stderr pattern
- Suggest: `sudo [original command]`
- Require user confirmation before re-executing
- Unit tests for detection logic
- Integration test with actual permission-restricted command

**Acceptance Criteria**:
- [ ] Detects "Permission denied" in stderr
- [ ] Suggests sudo variant with explanation
- [ ] Displays clear confirmation prompt
- [ ] Re-executes with sudo after confirmation
- [ ] Tests pass on Linux and macOS

**Effort**: S (2-3 days)
**Files to Modify**:
- Create `src/healing/mod.rs`
- Create `src/healing/permission.rs`
- Modify `src/main.rs` to integrate healing on failure
- Add tests in `tests/healing_permission_tests.rs`

### Sub-Issue #2: Self-Healing for Command Not Found

**Description**:
When a command fails with "command not found", suggest installation.

**Scope**:
- Detect command not found from stderr pattern
- Determine package manager (brew, apt, yum, pacman, choco)
- Suggest: `[package manager] install [command]`
- Display installation instructions (no auto-install)

**Acceptance Criteria**:
- [ ] Detects "command not found" or "not recognized" in stderr
- [ ] Suggests platform-appropriate package manager
- [ ] Displays installation instructions
- [ ] Tests cover macOS, Linux, Windows patterns

**Effort**: S (2-3 days)
**Files to Modify**:
- Create `src/healing/not_found.rs`
- Modify `src/main.rs` to integrate healing on failure
- Add tests in `tests/healing_not_found_tests.rs`

## Deferred Sub-Issues for v1.2.0

### Sub-Issue #3: Self-Healing for Safety Validation Failures

**Description**:
When SafetyValidator blocks a command, suggest safer alternatives.

**Scope**:
- Leverage existing ValidationResult.matched_patterns
- Generate safer alternative commands based on intent
- Require LLM integration for intent understanding
- Store successful corrections for learning

**Effort**: M (5-7 days)
**Dependencies**: Requires ChromaDB (#166) for pattern storage

### Sub-Issue #4: Learning System for Successful Corrections

**Description**:
Store successful corrections and suggest them for similar errors.

**Scope**:
- ChromaDB integration for correction history
- Semantic search for similar error patterns
- Ranking corrections by success rate
- Privacy-conscious data collection

**Effort**: L (> 7 days)
**Dependencies**: Requires ChromaDB (#166)

## Next Steps

1. **Post this proposal** to Issue #155 as a comment
2. **Ask user/maintainer** to choose between:
   - Option A: Defer entirely to v1.2.0
   - Option B: Implement MVP for v1.1.0
   - Option C: Create sub-issues (recommended)
3. **If Option C approved**:
   - Create sub-issue #1 and #2 for v1.1.0
   - Create sub-issue #3 and #4 for v1.2.0 milestone
   - Close Issue #155 as replaced by sub-issues
4. **If Option A or B approved**:
   - Update Issue #155 with chosen scope
   - Create implementation plan

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Scope creep into complex AI-driven healing | High | High | Use Option C with clear v1.1.0 boundaries |
| Timeline impact on v1.1.0 release | Medium (if Option B) | Medium | Defer to v1.2.0 or use Option C |
| User expectations exceeding MVP | Medium | Low | Clear documentation of scope in release notes |
| Integration complexity with existing error handling | Low | Medium | Reuse ExecutionResult and ValidationResult |

---

**Proposal Date**: 2026-01-08
**Status**: Awaiting maintainer decision
**Recommended Option**: C (Sub-issues with v1.1.0 simple fixes, v1.2.0 complex features)
