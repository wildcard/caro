# Evaluation Harness Development - Ralph Wiggum Loop

**Goal**: Continuously improve the LLM evaluation harness through iterative TDD cycles, implementing Milestone #11 (WP09-WP23) over 16 weeks.

**Pattern**: Ralph Wiggum Loop (TDD iteration until all tests pass)
**Timeline**: 16 weeks (Jan 17 - May 16, 2026)
**Project Board**: https://github.com/users/wildcard/projects/6
**Milestone**: https://github.com/wildcard/caro/milestone/11

---

## Context

You are continuing development of the **Caro LLM Evaluation Harness**, a comprehensive quality assurance platform for testing command generation quality across multiple backends.

### Foundation Complete (WP01-WP08)

✅ **Already Implemented**:
- Core data types and models (src/evaluation/models.rs)
- 55 test cases in tests/evaluation/test_cases.toml
- Evaluators for correctness, safety, POSIX compliance
- Baseline tracking system (31.0% baseline established)
- GitHub Actions CI/CD workflow
- Custom cargo test harness

### Your Mission (WP09-WP23)

Transform the evaluation harness into a world-class QA platform by implementing 15 work packages across 8 phases. See full roadmap in:
- thoughts/shared/plans/evaluation-harness-maturity-milestone.md
- thoughts/shared/plans/evaluation-harness-foundation-status.md

---

## Ralph Wiggum Loop Protocol

### Phase Structure

Each work package follows this cycle:

```
1. READ SPECIFICATION
   ↓
2. WRITE TESTS (that will fail)
   ↓
3. RUN TESTS (verify they fail for right reasons)
   ↓
4. IMPLEMENT (minimal code to pass tests)
   ↓
5. RUN TESTS (iterate until ALL pass)
   ↓
6. REFACTOR (clean up while keeping tests green)
   ↓
7. VERIFY (final test run + baseline check)
   ↓
8. COMMIT & UPDATE PROJECT BOARD
```

**Loop Invariant**: Never move to next work package until ALL tests pass.

### Iteration Rules

1. **Red → Green → Refactor**: Classic TDD cycle
2. **No Skipping Failures**: If a test fails, fix it before proceeding
3. **Baseline Protection**: Never regress below 31.0% pass rate
4. **Incremental Commits**: Commit after each green cycle
5. **Update Tracking**: Mark issue progress on project board

---

## Work Package Execution Template

For each work package (WP09-WP23):

### Step 1: Read Specification

```bash
# Read the GitHub issue
gh issue view <ISSUE_NUMBER>

# Read related documentation
grep -A 50 "WP<XX>" thoughts/shared/plans/evaluation-harness-maturity-milestone.md
```

### Step 2: Write Tests First

Create test file: tests/evaluation/tests/test_wp<XX>.rs

```rust
#[cfg(test)]
mod wp_tests {
    use super::*;

    #[test]
    fn test_feature_works() {
        // Arrange
        let input = /* test setup */;

        // Act
        let result = /* call new feature */;

        // Assert
        assert_eq!(result, expected);
    }
}
```

### Step 3: Run Tests (Verify Red)

```bash
# Run specific work package tests
cargo test --test evaluation wp<XX>

# Expected: Tests should FAIL (we haven't implemented yet)
```

### Step 4: Implement Minimum Code

Write only enough code to pass tests. Keep it simple.

### Step 5: Run Tests (Iterate Until Green)

```bash
# Continuous test loop
while ! cargo test --test evaluation wp<XX>; do
    echo "Tests failing, implementing..."
    # Make changes to code
    sleep 2
done

echo "✅ All WP<XX> tests passing!"
```

**Ralph Wiggum Rule**: Stay in this loop until ALL tests pass.

### Step 6: Refactor (Keep Tests Green)

```bash
# Refactor for clarity
# After each change, verify tests still pass:
cargo test --test evaluation wp<XX>
```

Refactoring checklist:
- [ ] Remove duplication
- [ ] Improve naming
- [ ] Add documentation
- [ ] Run cargo fmt and cargo clippy

### Step 7: Verify Integration

```bash
# Run ALL evaluation tests
cargo test --test evaluation

# Check baseline hasn't regressed
cargo test --test evaluation | grep "Passed:"
# Must show >= 31.0%

# Run full test suite
cargo test --lib
cargo clippy -- -D warnings
```

### Step 8: Commit & Update Tracking

```bash
# Commit with clear message
git add .
git commit -m "feat(eval): Implement WP<XX> - <Feature Name>

- Added <test count> tests for <feature>
- Implemented <module> with <functionality>
- All tests passing, baseline maintained
- Closes #<ISSUE_NUMBER>

Work Package: WP<XX>
Phase: <X>"

# Create PR
gh pr create \
  --title "WP<XX>: <Feature Name>" \
  --body "Implements Work Package <XX> from Milestone #11

Closes #<ISSUE_NUMBER>" \
  --milestone 11
```

---

## Phase-Specific Guidance

### Phase 1: Multi-Backend Validation (WP09)

**Issue**: #516
**Goal**: Test all backends (MLX, SmolLM, Qwen, Ollama) in CI

**Tests to Write**:
```rust
#[test]
fn test_mlx_backend_evaluates_correctly() { /* ... */ }

#[test]
fn test_smollm_backend_evaluates_correctly() { /* ... */ }

#[test]
fn test_graceful_degradation_when_backend_unavailable() { /* ... */ }
```

**Implementation**: Update .github/workflows/evaluation.yml with matrix strategy

### Phase 2: Prompt Engineering (WP10-11)

**Issue**: #517
**Goal**: Prompt versioning and A/B testing

**Tests to Write**:
```rust
#[test]
fn test_load_prompt_version() { /* ... */ }

#[test]
fn test_compare_multiple_prompts() { /* ... */ }

#[test]
fn test_statistical_significance_calculation() { /* ... */ }
```

**Implementation**: Create prompts/ directory, implement src/prompts/registry.rs

### Phase 3: Model Intelligence (WP12-13)

**Issue**: #521
**Goal**: Model profiling and capability matrices

### Phase 4: Feedback Loops (WP14-15)

**Issue**: #522
**Goal**: Automated issue creation and pattern extraction

### Phase 5: Fine-Tuning (WP16-17)

**Issue**: #518
**Goal**: Export training data and track effectiveness

### Phase 6: Analytics (WP18-19)

**Issue**: #523
**Goal**: Time-series analysis and dashboards

### Phase 7: Test Quality (WP20-21)

**Issue**: #524
**Goal**: Automated test generation and quality metrics

### Phase 8: Performance (WP22-23)

**Issue**: #525
**Goal**: Token efficiency and parallel execution

---

## Success Criteria (Exit Conditions)

Exit the Ralph Wiggum loop when:

- ✅ All WP tests pass (100% green)
- ✅ Full evaluation suite passes
- ✅ Baseline maintained (≥31.0% pass rate)
- ✅ cargo clippy -- -D warnings clean
- ✅ cargo fmt --check clean
- ✅ Documentation updated
- ✅ PR merged to main
- ✅ Project board updated to "Done"

---

## Daily Workflow

### Start of Day

```bash
# Check project board for current task
gh project view 6 --owner wildcard

# Pull latest changes
git checkout main && git pull origin main

# Pick next work package
gh issue list --milestone 11 --state open

# Create feature branch
git checkout -b wp<XX>-<feature-slug>
```

### During Development

```bash
# Continuous test-driven loop
while ! cargo test --test evaluation wp<XX>; do
    echo "❌ Tests failing, implementing..."
    sleep 2
done

echo "✅ Tests passing! Running full suite..."
cargo test --test evaluation
```

### End of Day

```bash
# Run full validation
cargo test
cargo clippy -- -D warnings

# Commit progress
git add .
git commit -m "wip: WP<XX> progress - <description>"
git push origin wp<XX>-<feature-slug>
```

---

## Ralph Wiggum Mantras

- **"Tests first, code second"** - Write failing test before implementing
- **"Red, green, refactor"** - Classic TDD cycle
- **"It works when tests pass"** - Not when you think it works
- **"One failing test at a time"** - Don't accumulate failures
- **"Commit on green"** - Never commit on red

---

## Start Command

When ready to begin a work package:

```bash
# 1. Check project board for next task
gh project view 6 --owner wildcard

# 2. Review the issue
gh issue view <NUMBER>

# 3. Create feature branch
git checkout -b wp<XX>-<feature-name>

# 4. Begin TDD cycle
echo "Starting WP<XX> - Tests First!"
```

**Your mission**: Implement all 15 work packages using the Ralph Wiggum loop, iterating until all tests pass, never moving forward until green, delivering a world-class evaluation harness by May 16, 2026.

🎯 **"I'm helping! I'm helping!"** - Ralph Wiggum (when tests finally pass)
