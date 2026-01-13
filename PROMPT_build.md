# Ralph Building Mode - Caro

You are in **Building Mode**. Your job is to implement ONE task from the implementation plan, following TDD, then exit so the loop can restart with fresh context.

## 0a. Study Operational Guide

Study the operational reference:

```
.claude/AGENTS.md       # Build commands, test commands, patterns
```

Key commands you'll need:
```bash
make check              # Full validation (lint + fmt + test + audit)
make test-verbose       # Detailed test output
make test-contract      # Contract tests only
make lint               # Clippy warnings as errors
make fmt                # Format check
```

## 0b. Study Project Principles

Study the constitution - these are NON-NEGOTIABLE:

```
.specify/memory/constitution.md
```

Critical principles:
1. **TDD is NON-NEGOTIABLE** - Write tests FIRST (red-green-refactor)
2. **Safety-First** - Never weaken safety patterns
3. **Library-First** - Features are testable library functions
4. **Simplicity** - Minimal complexity, direct implementations

## 0c. Study Current Plan

Read `IMPLEMENTATION_PLAN.md` to understand available tasks:

- What tasks are pending?
- What are their priorities?
- What are their dependencies?
- What are their test requirements?

## 0d. Study Relevant Code

Before selecting a task, study the files it will affect.
Use subagents to explore if needed.

**CRITICAL**: Don't assume features are not implemented. Study first.

## 1. Select ONE Task

From `IMPLEMENTATION_PLAN.md`, select the most important task:

### Selection Rules

1. **Priority Order**: Critical > High > Medium > Low
2. **Dependencies**: Skip tasks whose dependencies aren't met
3. **Scope**: Choose tasks you can complete in one iteration
4. **Impact**: Within same priority, prefer higher impact

### Announce Selection

Before starting, clearly state:
```
Selected Task: [task title]
Priority: [level]
Files: [files to modify]
Test Plan: [how you'll test]
```

## 2. Implement with TDD

Follow the strict TDD cycle. Caro's constitution makes this NON-NEGOTIABLE.

### Step 2a: RED - Write Failing Test

Write a test that captures the expected behavior:

```bash
# Add test to appropriate file
# tests/contract_*.rs for contracts
# tests/*_test.rs for integration
# src/**/tests.rs for unit tests

# Verify it fails
make test-verbose
```

The test MUST fail before you write implementation code.

### Step 2b: GREEN - Write Minimum Code

Write the minimum code to make the test pass:

```bash
# Implement just enough
# Run tests to verify
make test-verbose
```

Don't over-engineer. Just make the test pass.

### Step 2c: REFACTOR - Clean Up

With tests green, improve code quality:

```bash
# Refactor while keeping tests green
make check  # Full validation
```

## 3. Validate Thoroughly

Run full validation before committing:

```bash
# Required checks
make check

# If safety patterns were touched
cargo test --test safety_validator_contract

# If backends were touched
cargo test --test backend_trait_contract
```

### Validation Checklist

- [ ] All tests pass (`make test`)
- [ ] No clippy warnings (`make lint`)
- [ ] Code is formatted (`make fmt`)
- [ ] No security issues (`make audit`)
- [ ] Contract tests pass (if applicable)

**DO NOT PROCEED** if any validation fails. Fix issues first.

## 4. Update Implementation Plan

Mark the task as completed in `IMPLEMENTATION_PLAN.md`:

1. Change `### Task X.Y` status indicator
2. Add completion date
3. Move to "Completed" section with commit reference
4. Note any follow-up tasks discovered

Example update:
```markdown
## Completed

### Add --dry-run flag (Completed 2026-01-13)

- **Commit**: abc1234
- **Summary**: Added --dry-run flag to show commands without executing.
  Discovered: need to update shell completion scripts (added as Task 4.3).
```

## 5. Create Atomic Commit

Stage and commit with a clear message:

```bash
# Stage all changes
git add -A

# Commit with conventional commit format
git commit -m "$(cat <<'EOF'
feat(cli): add --dry-run flag for command preview

- Add --dry-run/-n flag to CLI arguments
- Show command without executing when flag is set
- Add tests for dry-run behavior

Task: 2.1 from IMPLEMENTATION_PLAN.md
Tests: tests/cli_test.rs::test_dry_run_*
EOF
)"
```

### Commit Message Format

```
<type>(<scope>): <description>

- <change 1>
- <change 2>

Task: <reference to plan>
Tests: <test file(s) added/modified>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

## 6. Exit Cleanly

After successful commit, exit so the loop restarts with fresh context.

The next iteration will:
1. Read the updated `IMPLEMENTATION_PLAN.md`
2. See your task is completed
3. Select the next priority task

## 999. Guardrails (Higher Number = More Critical)

### 9990. TDD is Non-Negotiable
Write the test FIRST. Verify it FAILS. Then implement. This is caro's constitution.

### 9991. One Task Per Iteration
Complete ONE task fully. Don't batch multiple tasks. Exit after commit.

### 9992. Don't Assume Not Implemented
Study the code before assuming something is missing. Use grep, subagents.

### 9993. Validate Before Commit
`make check` must pass. No exceptions. Fix issues before committing.

### 9994. Keep Plan Updated
Always update `IMPLEMENTATION_PLAN.md` with completion status and notes.

### 9995. Preserve Safety
Never weaken safety patterns. Run safety contract tests if patterns change:
```bash
cargo test --test safety_validator_contract
```

### 9996. Atomic Commits
One commit per task. Don't mix unrelated changes. Keep history clean.

### 9997. Exit After Commit
After successful commit, exit cleanly. Don't start another task.

### 9998. No Partial Work
If you can't complete a task, don't commit partial work. Note the blocker in the plan and exit.

### 9999. Ultrathink Before Complex Changes
For architectural decisions, use extended thinking. Capture the "why" in comments.
