# Development Lifecycle Improvements
## Based on Safety Validation Work (Jan 2026)

---

## 1. Pre-Commit Pattern Validation

### Current State
- Patterns manually reviewed after implementation
- Gaps discovered through testing (reactive)
- No automated validation before commit

### Recommended Improvement

**Add Git Pre-Commit Hook**: `.git/hooks/pre-commit`

```bash
#!/bin/bash
# Pre-commit hook for safety pattern validation

echo "Running safety pattern validation..."

# Check if patterns.rs was modified
if git diff --cached --name-only | grep -q "src/safety/patterns.rs"; then
    echo "Detected changes to safety patterns..."

    # Run pattern analyzer
    cargo run --bin pattern-analyzer --quiet || {
        echo "❌ Pattern validation failed!"
        echo "Please fix pattern issues before committing."
        exit 1
    }

    # Check for common pattern gaps
    ./scripts/check-pattern-gaps.sh || {
        echo "⚠️  Potential pattern gaps detected"
        echo "Review the warnings above before committing."
        # Don't block commit, just warn
    }
fi

echo "✅ Pattern validation passed"
```

**Benefits**:
- Catches pattern compilation errors before commit
- Warns about potential gaps early
- Encourages pattern testing before merge

---

## 2. Test-Driven Pattern Development

### Current State
- Patterns implemented first
- Tests created after (reactive)
- Gaps found through manual testing

### Recommended Improvement

**TDD Workflow for Safety Patterns**:

```
1. Write failing test case first
   - Example: dangerous-command that should be blocked
   - Run test → verify it PASSES (command not blocked)

2. Implement pattern to block it
   - Add pattern to patterns.rs
   - Run test → verify it FAILS (command blocked)

3. Verify no false positives
   - Add safe command tests
   - Run test → verify safe commands still pass
```

**Enforcement**: Update `CONTRIBUTING.md` with TDD requirement for patterns

---

## 3. Automated Pattern Gap Analysis

### Current State
- Manual line-by-line review (time-consuming)
- Gaps identified through human analysis
- No systematic gap detection

### Recommended Tool

**Create**: `scripts/analyze-pattern-gaps.py`

```python
#!/usr/bin/env python3
"""
Automated safety pattern gap analyzer.

Analyzes all safety patterns and identifies:
- Argument order variations (like dd if/of swap)
- Flag order variations (like -Force -Recurse vs -Recurse -Force)
- Path variations (/, ~, ., ..)
- Wildcard variations (*, *.*, .*)
"""

import re
from dataclasses import dataclass
from typing import List, Set

@dataclass
class PatternGap:
    gap_type: str  # "argument_order", "flag_order", "path_variant"
    severity: str  # "critical", "high", "medium"
    pattern_id: int
    missing_variant: str
    example_command: str
    recommendation: str

class PatternGapAnalyzer:
    def __init__(self, patterns_file: str):
        self.patterns = self.load_patterns(patterns_file)
        self.gaps: List[PatternGap] = []

    def analyze(self) -> List[PatternGap]:
        """Run all gap detection algorithms."""
        self.check_argument_order_gaps()
        self.check_flag_order_gaps()
        self.check_path_variant_gaps()
        self.check_wildcard_gaps()
        return self.gaps

    def check_argument_order_gaps(self):
        """Detect commands with fixed argument order (like dd)."""
        # Analyze patterns for if=/of= ordering
        # Flag if only one order is covered
        pass

    def check_path_variant_gaps(self):
        """Detect missing path variations."""
        path_sets = {
            'root': ['/', '//', '///'],
            'home': ['~', '~/', '$HOME'],
            'current': ['.', './', './*'],
            'parent': ['..', '../', '../*'],  # Previously missed!
        }
        # Check if pattern covers all relevant variants
        pass

# Usage:
# python scripts/analyze-pattern-gaps.py src/safety/patterns.rs
```

**Integration**: Run in CI on every PR touching patterns.rs

---

## 4. Comprehensive Pattern Test Matrix

### Current State
- Tests created ad-hoc
- Coverage gaps not systematically identified
- No test matrix for pattern variants

### Recommended Improvement

**Create**: `scripts/generate-pattern-tests.py`

```python
"""
Generate comprehensive test matrix from patterns.

For each dangerous pattern, generates:
- Base case (exact match)
- Argument variations
- Flag order variations
- Path variations
- Safe similar commands (false positive tests)
"""

def generate_test_matrix(pattern):
    tests = []

    # Base case
    tests.append(generate_base_test(pattern))

    # Variations
    if has_arguments(pattern):
        tests.extend(generate_argument_variations(pattern))

    if has_flags(pattern):
        tests.extend(generate_flag_variations(pattern))

    if has_paths(pattern):
        tests.extend(generate_path_variations(pattern))

    # False positive tests (safe commands that look similar)
    tests.extend(generate_safe_variants(pattern))

    return tests

# Auto-generates test-cases.yaml from patterns.rs
```

**Benefits**:
- Systematic test coverage
- Automatically detects missing variants
- Prevents regression

---

## 5. Pattern Contribution Workflow

### Current State
- No formal process for adding patterns
- No validation checklist
- No automated review

### Recommended Improvement

**Create**: `.claude/workflows/add-safety-pattern.md`

```markdown
# Safety Pattern Contribution Workflow

## Step 1: Identify Dangerous Command
- Document the dangerous command
- Explain why it's dangerous
- Determine risk level (Critical/High/Moderate)

## Step 2: Write Test Cases FIRST (TDD)
```yaml
# .claude/beta-testing/new-pattern-test.yaml
- id: "new_001"
  input: "dangerous command description"
  dangerous_pattern: "actual dangerous command"
  expected_behavior: "BLOCKED"
```

## Step 3: Run Test (Should PASS - command not blocked yet)
```bash
caro test --suite .claude/beta-testing/new-pattern-test.yaml
# Verify: Test PASSES (command allowed) ← Expected failure
```

## Step 4: Implement Pattern
```rust
// src/safety/patterns.rs
DangerPattern {
    pattern: r"your_regex_here",
    risk_level: RiskLevel::Critical,
    description: "Clear description",
}
```

## Step 5: Run Test (Should FAIL - command now blocked)
```bash
caro test --suite .claude/beta-testing/new-pattern-test.yaml
# Verify: Test FAILS (command blocked) ← Pattern working!
```

## Step 6: Add False Positive Tests
```yaml
# Test safe commands that look similar
- id: "safe_001"
  input: "similar but safe command"
  expected_output: "safe command result"
```

## Step 7: Run Full Suite (No regressions)
```bash
caro test --backend static --suite .claude/beta-testing/test-cases.yaml
# Verify: No new failures
```

## Step 8: Run Pattern Gap Analyzer
```bash
./scripts/analyze-pattern-gaps.py
# Check for suggested variations
```

## Step 9: Document Pattern
- Add comment explaining edge cases
- Document any platform-specific behavior
- Note related patterns

## Step 10: Commit with Checklist
```
feat(safety): Add pattern for [dangerous command]

Pattern Details:
- Risk Level: Critical/High/Moderate
- Platforms: Bash/PowerShell/All
- Variants covered: [list]

Testing:
- [x] Test case written first (TDD)
- [x] Pattern blocks dangerous command
- [x] False positive tests pass
- [x] Full suite passes (no regressions)
- [x] Gap analyzer run
- [x] Documentation updated

Related to #[issue]
```
```

---

## 6. CI/CD Pipeline Enhancements

### Current State
- Manual testing of safety changes
- No automated pattern validation in CI

### Recommended Improvement

**Update**: `.github/workflows/safety-validation.yml`

```yaml
name: Safety Pattern Validation

on:
  pull_request:
    paths:
      - 'src/safety/**'
      - '.claude/beta-testing/**'

jobs:
  validate-patterns:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Compile pattern regex
        run: cargo build --lib --quiet

      - name: Run pattern gap analyzer
        run: ./scripts/analyze-pattern-gaps.py src/safety/patterns.rs

      - name: Run safety test suite
        run: |
          cargo build --release
          ./target/release/caro test --backend static --suite .claude/beta-testing/test-cases.yaml
          ./target/release/caro test --backend embedded --suite .claude/beta-testing/dangerous-commands-test.yaml

      - name: Check for regressions
        run: |
          # Compare pass rate with baseline
          ./scripts/check-safety-regressions.sh

      - name: Comment on PR
        if: failure()
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '⚠️ Safety pattern validation failed. Please review the CI logs.'
            })
```

---

## 7. Monitoring and Alerting

### Current State
- No production monitoring of safety blocking
- No visibility into blocked commands

### Recommended Improvement

**Add**: Safety audit logging

```rust
// src/safety/validator.rs
pub async fn validate_command(&self, command: &str, shell: ShellType)
    -> Result<SafetyResult> {

    let result = self.run_validation(command, shell).await?;

    // Audit logging for blocked commands
    if !result.allowed {
        self.log_blocked_command(&BlockedCommandEvent {
            command: command.to_string(),
            shell,
            risk_level: result.risk_level,
            patterns_matched: result.patterns.clone(),
            timestamp: Utc::now(),
        }).await;
    }

    Ok(result)
}
```

**Benefits**:
- Track which patterns are most effective
- Identify false positives in production
- Inform future pattern improvements

---

## Summary: Lifecycle Improvements

| Improvement | Impact | Effort | Priority |
|-------------|--------|--------|----------|
| Pre-commit pattern validation | High | Low | P0 |
| TDD for patterns | High | Medium | P0 |
| Automated gap analyzer | High | High | P1 |
| Comprehensive test matrix | Medium | Medium | P1 |
| Pattern contribution workflow | Medium | Low | P1 |
| CI/CD safety validation | High | Medium | P0 |
| Safety audit logging | Medium | Medium | P2 |

**Quick Wins** (implement first):
1. Pre-commit hook (1 hour)
2. TDD workflow docs (30 mins)
3. CI/CD pipeline (2 hours)

**High Impact** (implement next):
1. Automated gap analyzer (4 hours)
2. Pattern contribution workflow (1 hour)
3. Comprehensive test matrix generator (3 hours)
