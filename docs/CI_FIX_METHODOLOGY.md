# CI Fix Methodology

This guide documents battle-tested approaches for fixing CI failures, based on real work from PR #92 and other successful fixes.

## Table of Contents

- [Overview](#overview)
- [When to Use This Guide](#when-to-use-this-guide)
- [Standard Work Plan](#standard-work-plan)
- [Methodology Patterns](#methodology-patterns)
- [Common Failure Categories](#common-failure-categories)
- [Lessons Learned](#lessons-learned)

---

## Overview

CI failures fall into predictable categories. This methodology provides a systematic approach to diagnosing and fixing them efficiently, avoiding common pitfalls that lead to wasted cycles.

**Key Principle**: Fix issues incrementally, verifying each fix before moving to the next. One focused commit per issue category.

---

## When to Use This Guide

Use this methodology when:
- ✅ CI is completely failing (red builds, multiple errors)
- ✅ Release is blocked by test failures
- ✅ You need to fix multiple categories of failures (lint + test + format)
- ✅ Working with environment-dependent tests (MLX, Ollama, model downloads)

Skip this guide when:
- ❌ Single trivial fix (just fix it)
- ❌ CI passing but needs optimization
- ❌ Adding new features (use feature workflow instead)

---

## Standard Work Plan

### Phase 1: Analysis & Triage (15-30 min)

**Goal**: Understand the full scope before touching code.

```bash
# 1. Review all CI logs
gh run view <run-id> --log-failed

# 2. Categorize failures
# - Security: cargo audit failures
# - Correctness: Test failures, logic errors
# - Style: Clippy warnings, formatting
# - Environment: Missing tools, model downloads

# 3. Check recent commits
git log --oneline -10

# 4. Identify dependencies between failures
# Example: Test fails because of clippy error in same file
```

**Output**: Prioritized list by severity (security > correctness > style)

### Phase 2: Linter Fixes (30-60 min)

**Goal**: Clean up all clippy warnings.

```bash
# 1. Run clippy locally
cargo clippy --all-targets --all-features -- -D warnings

# 2. Fix issues by category:
# - Method name conflicts (e.g., trait method vs struct method)
# - Redundant patterns (is_ok().unwrap() → if let Ok)
# - Needless borrows (&x when x: &T already)
# - Unused variables (_var or remove)

# 3. Update call sites
# Use grep to find all usages:
grep -r "old_name" src/ tests/

# 4. Verify
cargo clippy --all-targets --all-features -- -D warnings
```

**Common patterns to fix:**

```rust
// ❌ Antipattern: is_ok() + unwrap()
if result.is_ok() {
    let value = result.unwrap();
    // use value
}

// ✅ Idiomatic: if let
if let Ok(value) = result {
    // use value
}

// ❌ Antipattern: Method name conflict
impl Default for Foo {
    fn default() -> Self { ... }
}
impl Foo {
    fn default() -> Self { ... }  // Conflict!
}

// ✅ Fix: Rename non-trait method
impl Foo {
    fn default_instance() -> Self { ... }
}
```

**Commit**: One commit for all clippy fixes

### Phase 3: Formatting Fixes (5-10 min)

**Goal**: Ensure code style consistency.

```bash
# 1. Auto-format
cargo fmt

# 2. Verify
cargo fmt --check

# 3. Review diff to ensure no accidental changes
git diff

# 4. If formatting changes logic (shouldn't happen), revert and fix manually
```

**Commit**: One commit for formatting (if separate from clippy)

### Phase 4: Test Failures (60-120 min)

**Goal**: Make tests pass or gracefully skip when environment unavailable.

#### Categorize Test Failures

1. **Logic errors**: Actual bugs in code
2. **Environment-dependent**: Missing models, services, hardware
3. **Flaky tests**: Timing, concurrency issues
4. **Broken test setup**: Wrong fixtures, stale data

#### Fix Strategy by Category

**Logic errors:**
```bash
# 1. Reproduce locally
cargo test <test_name> -- --nocapture

# 2. Debug with prints or debugger
# 3. Fix the bug
# 4. Verify all related tests pass
cargo test <module>
```

**Environment-dependent tests:**

Use **graceful degradation pattern**:

```rust
// ❌ Fails in CI when model unavailable
#[test]
fn test_inference() {
    let backend = MLXBackend::new().expect("MLX required");
    // test code
}

// ✅ Gracefully skips when unavailable
#[test]
fn test_inference() {
    let Ok(backend) = MLXBackend::new() else {
        eprintln!("⚠️  Skipping MLX test - Metal not available in CI");
        return;
    };
    // test code
}
```

**Benefits**:
- Tests work both locally (with model) and CI (without model)
- No `#[ignore]` annotations needed
- Clear skip messages for debugging

**Flaky tests:**
```rust
// Add timeouts, retries, or better synchronization
use std::time::Duration;

// Before
assert!(condition);

// After
for _ in 0..3 {
    if condition { break; }
    std::thread::sleep(Duration::from_millis(100));
}
assert!(condition, "Condition failed after retries");
```

**Commit**: One commit per test category (e.g., "fix: Make smoke tests environment-aware")

### Phase 5: Validation (15-30 min)

**Goal**: Ensure all fixes work together.

```bash
# 1. Run full local test suite
cargo test --all-targets --all-features

# 2. Run clippy again (regressions possible)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Check formatting
cargo fmt --check

# 4. Run security audit
cargo audit

# 5. Test release build
cargo build --release

# 6. If act available, run full CI locally
./test-ci-locally.sh
```

**Push commits incrementally** to monitor CI:
```bash
# Push after each phase
git push origin <branch>

# Monitor CI
gh run watch
```

### Phase 6: Documentation & Handoff (15-30 min)

**Goal**: Prevent similar issues, help future contributors.

1. **Create follow-up issues** for technical debt:
   ```bash
   gh issue create --title "TODO: Improve error handling in X" \
                   --body "Context from PR #Y..." \
                   --label "technical-debt"
   ```

2. **Document workarounds** if any:
   ```markdown
   # tests/README.md

   ## Known Issues

   - MLX tests skip in CI (Metal not available)
   - Some integration tests require Ollama running
   ```

3. **Add PR comment** summarizing work:
   ```markdown
   ## Fixed
   - 10 clippy warnings (method conflicts, redundant patterns)
   - 4 test failures (graceful degradation for environment-dependent tests)
   - Formatting issues

   ## Follow-up Issues
   - #123: Add integration test for X
   - #124: Refactor Y for better testability
   ```

---

## Methodology Patterns

### 1. Iterative Problem Solving

**Expect layers of issues** - fixing one often reveals the next:

```
Commit 1: Fix backend creation
  ↓ (reveals)
Commit 2: Fix inference calls
  ↓ (reveals)
Commit 3: Fix similar pattern in other tests
```

**Don't try to fix everything in one commit** - you'll miss edge cases.

### 2. Exact Whitespace Matching

When using `Edit` tool or `sed`:

```bash
# ❌ Wrong: Approximate match
old_string: "fn test() {"

# ✅ Right: Exact match with context
old_string: "    fn test_something() {\n        let backend = ..."

# Tips:
# - Use Read tool to see exact indentation
# - Count spaces/tabs carefully
# - Include 3-5 lines of surrounding context
```

### 3. Graceful Degradation for CI

**Pattern**: Tests should work with AND without external dependencies.

```rust
// Template for environment-dependent tests:
#[test]
fn test_feature() {
    // Attempt to create resource
    let Ok(resource) = fallible_operation() else {
        eprintln!("⚠️  Skipping test - [reason]");
        eprintln!("    To run: [setup instructions]");
        return;
    };

    // Actual test code
    assert!(resource.works());
}
```

**Apply to**:
- Model downloads (HuggingFace, Ollama)
- Hardware requirements (Metal, CUDA, Apple Silicon)
- Service dependencies (databases, APIs)
- Large file operations

### 4. Commit Granularity

**One commit per logical fix category:**

✅ Good:
```
fix: Resolve method name conflicts in ModelCatalog
fix: Replace is_ok().unwrap() with if let Ok pattern
fix: Make smoke tests environment-aware
```

❌ Bad:
```
fix: Fix all CI issues  # (too broad)
fix: Update file.rs line 42  # (too specific)
```

**Benefits**:
- Easier to review
- Easier to bisect if issues arise
- Clear history

---

## Common Failure Categories

### 1. Clippy Warnings

| Warning | Fix | Example |
|---------|-----|---------|
| `clippy::method-name-conflict` | Rename non-trait method | `default()` → `default_instance()` |
| `clippy::unnecessary-unwrap` | Use `if let Ok/Err` | `is_ok().unwrap()` → `if let Ok(x)` |
| `clippy::needless-borrow` | Remove `&` | `foo(&x)` → `foo(x)` when `x: &T` |
| `clippy::unused-variable` | Prefix with `_` or remove | `result` → `_result` |

### 2. Test Failures

| Failure Type | Root Cause | Fix Strategy |
|--------------|------------|--------------|
| Panic in test | Missing resource | Graceful skip pattern |
| Assertion failure | Logic bug | Debug and fix |
| Timeout | Slow operation | Increase timeout or mock |
| Flaky test | Race condition | Add synchronization |

### 3. Formatting Issues

| Issue | Fix |
|-------|-----|
| Mixed indentation | Run `cargo fmt` |
| Line length > 100 | Auto-formatted by rustfmt |
| Inconsistent spacing | Auto-formatted by rustfmt |

---

## Lessons Learned

### From PR #92 (Complete CI Fix)

**Context**: 10+ clippy warnings, 4 test failures, formatting issues
**Duration**: ~2 hours, 9 commits
**Result**: All CI green ✅

#### What Worked

1. **Systematic triage** - Categorized all failures before coding
2. **Incremental commits** - One fix category per commit
3. **Graceful degradation** - Tests work with/without models
4. **Exact whitespace matching** - No failed edits
5. **Follow-up issues** - Documented technical debt (6 issues created)

#### Pitfalls Avoided

- ❌ Trying to fix everything in one commit → Created logical commits instead
- ❌ Assuming `is_ok().unwrap()` is fine → Used idiomatic `if let Ok`
- ❌ Hard-coding CI skips → Used runtime checks instead
- ❌ Ignoring follow-up work → Created 6 issues for technical debt

#### Key Insight

**"Each fix reveals the next layer"** - Don't be surprised when fixing backend creation reveals inference issues, which reveal test setup issues. Budget time for iteration.

---

## Examples from Real Fixes

### Example 1: Method Name Conflict (PR #92, commit 8390fe2)

**Error**:
```
error: method `default` is associated with multiple trait implementations
  --> src/model_catalog.rs:42:5
```

**Root cause**: Both `Default` trait and inherent impl had `default()` method.

**Fix**:
```rust
// Before
impl ModelCatalog {
    pub fn default() -> Self { ... }  // Conflicts with trait
}

// After
impl ModelCatalog {
    pub fn default_model() -> Self { ... }  // Renamed
}

// Update call sites (2 in src/, 1 in tests/)
```

**Lesson**: Trait methods have priority - rename inherent methods.

### Example 2: Environment-Aware Smoke Tests (PR #92, commits 5bbc9ae, 363e5d6, 31547ba)

**Error**:
```
thread 'smoke_embedded_generate' panicked at 'Model download failed'
```

**Root cause**: CI doesn't have models cached, tests expected download to work.

**Fix pattern applied to 5 tests**:
```rust
#[test]
fn test_smoke_embedded() {
    // Before: Panics in CI
    let backend = EmbeddedBackend::new().expect("Backend required");

    // After: Graceful skip
    let Ok(backend) = EmbeddedBackend::new() else {
        eprintln!("⚠️  Skipping embedded test - model download failed");
        eprintln!("    This is expected in CI without model cache");
        return;
    };

    // Test code...
}
```

**Lesson**: Every test with external dependencies needs graceful degradation.

### Example 3: Iterative Discovery (PR #92)

**Iteration sequence**:
1. Fixed `MLXBackend::new()` panic → test passed ✅
2. Test failed at `generate()` call → Fixed inference setup ✅
3. Other smoke tests failed similarly → Applied pattern to 5 tests ✅

**Lesson**: First fix reveals next issue. Commit after each discovery.

---

## Quick Reference Checklists

### Before Starting
- [ ] Review all CI logs (`gh run view --log-failed`)
- [ ] Categorize failures by type
- [ ] Identify dependencies between failures
- [ ] Estimate time for each category

### During Fixes
- [ ] Run tests locally before pushing
- [ ] Verify clippy after each change
- [ ] Use exact whitespace matching for edits
- [ ] Create follow-up issues for technical debt
- [ ] Write clear commit messages

### Before Merge
- [ ] Full local test suite passes
- [ ] Clippy clean (no warnings)
- [ ] Formatting clean (`cargo fmt --check`)
- [ ] Security audit passes (`cargo audit`)
- [ ] CI green on remote
- [ ] Follow-up issues created

---

## Related Documentation

- [Local CI Testing](LOCAL_CI_TESTING.md) - Run CI locally before pushing
- [Smoke Tests](SMOKE_TESTS.md) - Understanding the smoke test suite
- [Contributing](../CONTRIBUTING.md) - General contribution guidelines

---

## Template: CI Fix Plan

Copy this template when starting a CI fix:

```markdown
## CI Fix Plan - [PR Number/Branch]

### Phase 1: Analysis
- [ ] Review CI logs
- [ ] Categorize failures:
  - Security:
  - Correctness:
  - Style:
- [ ] Prioritize by severity

### Phase 2: Linter Fixes
- [ ] Run clippy locally
- [ ] Fix issues:
  - [ ] Method conflicts
  - [ ] Redundant patterns
  - [ ] Needless borrows
- [ ] Verify: `cargo clippy -- -D warnings`
- [ ] Commit: "fix: Resolve clippy warnings"

### Phase 3: Formatting
- [ ] Run `cargo fmt`
- [ ] Verify: `cargo fmt --check`
- [ ] Commit: "style: Format code"

### Phase 4: Tests
- [ ] Reproduce failures locally
- [ ] Fix by category:
  - [ ] Logic errors
  - [ ] Environment-dependent (add graceful skips)
  - [ ] Flaky tests (add retries/timeouts)
- [ ] Verify: `cargo test`
- [ ] Commit per category

### Phase 5: Validation
- [ ] Full test suite passes
- [ ] Clippy clean
- [ ] Formatting clean
- [ ] Security audit passes
- [ ] Push and monitor CI

### Phase 6: Documentation
- [ ] Create follow-up issues
- [ ] Update relevant docs
- [ ] Add PR summary comment
```

---

## Contributing to This Guide

Found a new CI failure pattern? Add it here! This guide improves with battle-tested experience.

**To contribute**:
1. Document the failure pattern you encountered
2. Describe the fix that worked
3. Explain why it worked (not just what)
4. Add to appropriate section above
5. Submit PR with label `documentation`
