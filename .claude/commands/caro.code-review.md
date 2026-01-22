---
description: Pre-commit code review for quality and bug detection
---

## Code Review: Pre-commit Validation

This command performs a comprehensive code review on recently modified files, focusing on quality, security, and adherence to project standards.

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/caro.code-review` | Review all uncommitted changes |
| `/caro.code-review staged` | Review only staged changes |
| `/caro.code-review <file>` | Review specific file |
| `/caro.code-review --fix` | Attempt automatic fixes |

---

## What This Command Does

`/caro.code-review` evaluates modified code across five dimensions:
1. **Logic Errors** - Bugs, off-by-one errors, race conditions
2. **Security Issues** - Injection, unsafe code, credential exposure
3. **Performance Problems** - Inefficiencies, memory issues, N+1 patterns
4. **Code Quality** - DRY violations, complexity, naming
5. **Standard Adherence** - Project conventions, Rust idioms

---

## Outline

### 1. Parse Arguments

```
ARGUMENTS check:
- Empty → UNCOMMITTED_MODE (all changes)
- "staged" → STAGED_MODE (staged only)
- File path → FILE_MODE (specific file)
- "--fix" → Enable auto-fix suggestions
```

### 2. Gather Context

**Read project standards:**
```bash
# Load project conventions
cat CLAUDE.md | grep -A 50 "Development Standards"
cat .claude/reference/rust-cli-best-practices.md | head -100
```

**Identify changed files:**
```bash
# For UNCOMMITTED_MODE
git status --porcelain
git diff HEAD --name-only

# For STAGED_MODE
git diff --cached --name-only

# Get file statistics
git diff --stat HEAD
```

### 3. Analyze Each File

For each modified file, perform deep analysis:

**Read the file:**
```bash
git diff HEAD -- <file>    # Show changes
cat <file>                  # Full context
```

**Check five dimensions:**

#### 3.1 Logic Errors
```
Look for:
- Off-by-one errors in loops and indexing
- Incorrect boolean logic or conditions
- Missing error handling paths
- Race conditions in async code
- Null/None handling issues
- Unreachable code paths
```

#### 3.2 Security Issues
```
Look for:
- Unsafe Rust blocks without justification
- Command injection vulnerabilities
- Hardcoded credentials or secrets
- Unvalidated user input
- Insecure defaults
- Missing input sanitization
```

#### 3.3 Performance Problems
```
Look for:
- Unnecessary allocations (String vs &str)
- Clone where reference would work
- N+1 patterns in loops
- Blocking operations in async context
- Unbounded collections
- Missing lazy initialization
```

#### 3.4 Code Quality
```
Look for:
- DRY violations (duplicated logic)
- Functions > 50 lines
- Deep nesting (> 4 levels)
- Poor naming (single letters, ambiguous)
- Missing documentation on public items
- Complex conditionals
```

#### 3.5 Standard Adherence
```
Check against:
- Project error handling patterns (thiserror)
- Logging conventions (tracing)
- Test naming conventions
- Module organization
- Clippy compliance
```

### 4. Run Automated Checks

```bash
# Format check
cargo fmt -- --check

# Linter
cargo clippy -- -D warnings 2>&1 | head -50

# Type check
cargo check 2>&1 | head -30

# Run tests on modified code
cargo test --no-run 2>&1 | head -30
```

### 5. Generate Review Report

Create structured findings:

```markdown
## Code Review Report

**Date**: [timestamp]
**Branch**: [current branch]
**Files Reviewed**: [count]
**Changes**: +[additions] -[deletions]

---

### Summary

| Category | Issues | Severity |
|----------|--------|----------|
| Logic Errors | 2 | High |
| Security | 0 | - |
| Performance | 1 | Medium |
| Code Quality | 3 | Low |
| Standards | 1 | Low |

**Overall**: [PASS / NEEDS ATTENTION / BLOCKING]

---

### Findings

#### [CRITICAL] Logic Error in src/safety/mod.rs:45

**Issue**: Off-by-one error in pattern matching loop
**Line**: 45
**Code**:
```rust
for i in 0..patterns.len() - 1 {  // Should be patterns.len()
```
**Fix**:
```rust
for i in 0..patterns.len() {
```
**Rationale**: Last pattern is never checked, could miss dangerous commands

---

#### [MEDIUM] Performance in src/backends/embedded.rs:123

**Issue**: Unnecessary clone of large string
**Line**: 123
**Code**:
```rust
let response = self.model.generate(prompt.clone())?;
```
**Fix**:
```rust
let response = self.model.generate(&prompt)?;
```
**Rationale**: Avoid allocation when reference suffices

---

### Automated Check Results

**cargo fmt**: PASS
**cargo clippy**: 2 warnings
  - warning: unused variable `temp`
  - warning: this could be simplified

**cargo check**: PASS
**cargo test --no-run**: PASS

---

### Recommendations

1. Fix the critical logic error before committing
2. Address clippy warnings
3. Consider the performance suggestion

**Ready to commit?** Address CRITICAL issues first.
```

### 6. Optional: Attempt Fixes

If `--fix` flag is set:

```bash
# Auto-format
cargo fmt

# Apply clippy suggestions
cargo clippy --fix --allow-dirty

# Report what was fixed
git diff
```

---

## Severity Levels

| Level | Description | Action |
|-------|-------------|--------|
| CRITICAL | Bugs, security holes, data loss | Must fix before commit |
| HIGH | Logic errors, major issues | Should fix before commit |
| MEDIUM | Performance, moderate quality | Fix soon |
| LOW | Style, minor improvements | Consider fixing |

---

## Examples

### Review All Changes
```
User: /caro.code-review
Claude:
  Analyzing uncommitted changes...

  Files modified: 3
  - src/safety/mod.rs (+15, -3)
  - src/backends/embedded.rs (+42, -10)
  - tests/safety_test.rs (+25, -0)

  Running automated checks...
  ✓ cargo fmt: PASS
  ✓ cargo clippy: PASS
  ✓ cargo check: PASS

  Reviewing code...

  ## Code Review Report

  ### Summary
  | Category | Issues |
  |----------|--------|
  | Logic | 0 |
  | Security | 0 |
  | Performance | 1 |
  | Quality | 0 |
  | Standards | 0 |

  **Overall**: PASS (1 minor suggestion)

  ### Findings

  #### [LOW] Performance in src/backends/embedded.rs:89
  Consider using `&str` instead of `String::from()` for static strings.

  ---

  Ready to commit. Consider the minor performance suggestion.
```

### Review Specific File
```
User: /caro.code-review src/safety/mod.rs
Claude:
  Reviewing src/safety/mod.rs...

  [Full analysis of single file]
```

### Review with Auto-fix
```
User: /caro.code-review --fix
Claude:
  Analyzing changes...
  Applying automatic fixes...

  Fixed:
  - Formatted 2 files with cargo fmt
  - Applied 1 clippy suggestion

  Remaining issues to address manually:
  - [HIGH] Logic error in src/parser.rs:45

  Review the changes with `git diff`.
```

---

## Integration with Workflow

### Pre-commit Usage
```bash
# Run review before committing
/caro.code-review

# If passing, commit
/caro.commit
```

### CI Integration
The same checks run in CI via GitHub Actions:
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`

---

## Checklist for Reviewers

Before approving changes, verify:

- [ ] No CRITICAL or HIGH severity issues
- [ ] All clippy warnings addressed
- [ ] Tests pass
- [ ] Code is formatted
- [ ] Error handling uses project patterns
- [ ] New public APIs are documented
- [ ] Safety patterns are tested

---

## Notes

- Run before every commit for best results
- Address CRITICAL issues before committing
- Use `--fix` for automatic formatting and simple fixes
- Complex issues require manual intervention
- Reference `.claude/reference/` for best practices
