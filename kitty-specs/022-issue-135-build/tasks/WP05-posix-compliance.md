---
work_package_id: "WP05"
subtasks:
  - "T031"
  - "T032"
  - "T033"
  - "T034"
  - "T035"
  - "T036"
  - "T037"
  - "T038"
title: "POSIX Compliance Checker"
phase: "Phase 3 - Extended Validation"
lane: "planned"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP05 – POSIX Compliance Checker

## ⚠️ IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately (right below this notice).
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Mark as acknowledged**: When you understand the feedback and begin addressing it, update `review_status: acknowledged` in the frontmatter.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** – Reviewers add detailed feedback here when work needs changes. Implementation must address every item listed below before returning for re-review.

*[This section is empty initially. Reviewers will populate it if the work is returned from review. If you see feedback here, treat each item as a must-do before completion.]*

---

## Objectives & Success Criteria

**Goal**: Implement POSIX compliance validation (User Story 3) using shellcheck.

**Success Criteria**:
- Run 20-prompt POSIX dataset
- Identify bash-specific syntax violations
- Report POSIX compliance rate
- Suggest portable alternatives where available

**Independent Test**: Load POSIX dataset, run shellcheck on generated commands, identify violations, calculate compliance percentage.

## Context & Constraints

**Prerequisites**: WP02 (dataset loader), WP03 (executor)

**Supporting Documents**:
- **Spec**: `kitty-specs/022-issue-135-build/spec.md` - User Story 3, FR-005, SC-004
- **Plan**: `kitty-specs/022-issue-135-build/plan.md` - Shellcheck external dependency

**Key Design Decisions**:
- Use shellcheck via subprocess for static analysis
- Parse shellcheck JSON output format
- Common violations: `[[ ]]`, arrays, process substitution

**Constraints**:
- Shellcheck must be installed (fail gracefully with error if missing)
- Pin minimum shellcheck version (0.8.0+)

## Subtasks & Detailed Guidance

### T031 – Implement `tests/evaluation/src/posix_checker.rs` module

**Purpose**: Interface for POSIX compliance checking.

**Steps**:
1. Create module with PosixChecker struct
2. Define PosixValidationResult type matching data-model.md
3. Add shellcheck availability check

**Files**: `tests/evaluation/src/posix_checker.rs` (create)

### T032 – Implement shellcheck subprocess invocation and output parsing

**Purpose**: Execute shellcheck and parse JSON results.

**Steps**:
1. Invoke shellcheck with JSON format: `shellcheck --format=json --shell=sh <script>`
2. Parse JSON output into structured data
3. Handle shellcheck not found gracefully

**Files**: `tests/evaluation/src/posix_checker.rs` (modify)

### T033 – Implement bash-specific syntax detection

**Purpose**: Identify non-POSIX features in commands.

**Steps**:
1. Filter shellcheck warnings for POSIX violations
2. Common codes: SC2039 (bash-only features), SC2102 (arrays), SC2076 (regex)
3. Categorize violations by severity

**Files**: `tests/evaluation/src/posix_checker.rs` (modify)

### T034 – Implement POSIX compliance scoring

**Purpose**: Quantify compliance as percentage.

**Steps**:
1. Score: 0 violations = 100%, scale down based on violation count
2. Weight by severity (critical violations reduce score more)

**Files**: `tests/evaluation/src/posix_checker.rs` (modify)

### T035 – Implement portable alternative suggestions

**Purpose**: Suggest POSIX-compliant rewrites when shellcheck provides them.

**Steps**:
1. Extract "fix" suggestions from shellcheck output
2. Format as actionable alternatives

**Files**: `tests/evaluation/src/posix_checker.rs` (modify)

### T036 – Create POSIX test datasets

**Purpose**: Test cases for bash-specific and portable commands.

**Steps**:
1. Create `datasets/posix/bash_specific.json` with non-POSIX examples
2. Create `datasets/posix/portable_commands.json` with POSIX-compliant examples

**Files**:
- `tests/evaluation/datasets/posix/bash_specific.json` (create)
- `tests/evaluation/datasets/posix/portable_commands.json` (create)

**Parallel?**: Yes

### T037 – Add unit tests for shellcheck parsing

**Purpose**: Verify JSON parsing and violation detection.

**Files**: `tests/evaluation/src/posix_checker.rs` (modify)

### T038 – Add integration test `tests/evaluation/tests/test_posix.rs`

**Purpose**: End-to-end POSIX validation flow.

**Files**: `tests/evaluation/tests/test_posix.rs` (create)

## Risks & Mitigations

**Risk**: Shellcheck not installed
**Mitigation**: Fail gracefully with clear error message, document prerequisite

**Risk**: Shellcheck version differences
**Mitigation**: Pin minimum version requirement (0.8.0+), document in README

## Definition of Done Checklist

- [ ] PosixChecker module implemented
- [ ] Shellcheck subprocess invocation working
- [ ] Bash-specific syntax detection implemented
- [ ] POSIX compliance scoring calculated
- [ ] Portable alternative suggestions implemented
- [ ] Two POSIX datasets created
- [ ] Unit tests for shellcheck parsing
- [ ] Integration test runs POSIX validation
- [ ] All tests pass
- [ ] Shellcheck prerequisite documented in README
- [ ] `tasks.md` updated with WP05 completion status

## Activity Log

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created.
