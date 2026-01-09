# LLM Evaluation Harness - Quickstart Guide

**Feature**: 026-llm-evaluation-harness
**For**: Developers, QA Engineers, Contributors
**Last Updated**: 2026-01-09

## What is This?

The LLM Evaluation Harness is a comprehensive testing framework for validating shell command generation quality across all caro backends. It tests four categories:

1. **Correctness**: Does the generated command do what was requested?
2. **Safety**: Are dangerous commands properly blocked?
3. **POSIX**: Is the command portable and POSIX-compliant?
4. **Multi-Backend**: Do different backends produce consistent results?

## Quick Start (5 minutes)

### 1. Run Evaluation

```bash
# From repo root
cargo test --test evaluation

# Expected output:
# Running 100 tests across 4 backends...
# ✓ StaticMatcher: 92/100 passed (92%)
# ✓ MLX: 82/100 passed (82%)
# ✓ Ollama: 20/25 passed (80%)
# ✓ vLLM: 19/25 passed (76%)
# Overall: 84% pass rate
# Time: 3m 45s
```

### 2. Add a Test Case

```bash
# Edit tests/evaluation/dataset.yaml
# Add your test at the end:

  - id: "correctness-026"
    category: "correctness"
    input_request: "your natural language command here"
    expected_command: "expected shell command"
    validation_rule: "command_equivalence"
    tags: ["your-tag"]
    difficulty: "easy"
    source: "manual"
```

### 3. Re-run Evaluation

```bash
cargo test --test evaluation
# Your new test is now included!
```

## For Developers

### Running Evaluation in CI

Evaluation runs automatically on every PR. To check locally first:

```bash
# Run evaluation with JSON output
cargo test --test evaluation -- --format json > results.json

# Compare with baseline
cargo test --test evaluation -- \
  --baseline tests/evaluation/baselines/main-latest.json \
  --threshold 0.05
```

**Exit codes**:
- `0`: All tests passed, no regressions
- `1`: Tests failed or regressions detected (>5% drop)

### Running Specific Categories

```bash
# Only safety tests
cargo test --test evaluation -- --category safety

# Only correctness tests
cargo test --test evaluation -- --category correctness

# Only POSIX tests
cargo test --test evaluation -- --category posix

# Only multi-backend consistency tests
cargo test --test evaluation -- --category multi_backend
```

### Running Specific Backends

```bash
# Only StaticMatcher
cargo test --test evaluation -- --backend static_matcher

# Only MLX (requires macOS with Apple Silicon)
cargo test --test evaluation -- --backend mlx

# Only Ollama (requires Ollama installed)
cargo test --test evaluation -- --backend ollama
```

### Debugging Failed Tests

```bash
# Run with verbose output
cargo test --test evaluation -- --nocapture

# Run single test
cargo test --test evaluation -- --test-id safety-001

# Generate detailed failure report
cargo test --test evaluation -- --format json --failures-only > failures.json
```

## For QA Engineers

### Adding Test Cases from Bug Reports

When you discover a command generation issue:

1. **Create a test case**:
```yaml
- id: "safety-042"  # Increment the number
  category: "safety"
  input_request: "the exact prompt that caused the issue"
  expected_behavior: "blocked"  # or "executed"
  validation_rule: "must_be_blocked"  # or "must_execute"
  tags: ["bug", "beta-testing", "issue-161"]
  source: "beta-issue-161"
  notes: "Discovered in beta testing - user was able to generate dangerous command"
```

2. **Verify the test fails** (confirms the bug):
```bash
cargo test --test evaluation -- --test-id safety-042
# Should FAIL initially
```

3. **After the fix**, verify it passes:
```bash
cargo test --test evaluation -- --test-id safety-042
# Should PASS now
```

### Understanding Test Results

```bash
# Generate human-readable report
cargo test --test evaluation -- --format table

# Output:
# ┌────────────┬───────┬────────┬────────┬──────────┐
# │ Category   │ Total │ Passed │ Failed │ Pass Rate│
# ├────────────┼───────┼────────┼────────┼──────────┤
# │ Correctness│  25   │   22   │   3    │  88%     │
# │ Safety     │  25   │   24   │   1    │  96%     │
# │ POSIX      │  25   │   19   │   6    │  76%     │
# │ Multi-Back │  25   │   19   │   6    │  76%     │
# └────────────┴───────┴────────┴────────┴──────────┘
```

### Viewing Historical Trends

```bash
# Generate dashboard (requires HTML output)
cargo test --test evaluation -- --dashboard

# Opens: tests/evaluation/dashboard/index.html
# Shows: Pass rate trends over time, per-category breakdown, backend comparison
```

## For Contributors

### Test Case Guidelines

**Good Test Cases**:
```yaml
# ✓ Clear, specific input
# ✓ Unambiguous expected output
# ✓ Proper validation rule
# ✓ Helpful tags and notes

- id: "correctness-015"
  category: "correctness"
  input_request: "find files larger than 100MB"
  expected_command: "find . -type f -size +100M"
  validation_rule: "command_equivalence"
  tags: ["file-search", "size-filter", "common"]
  difficulty: "medium"
  source: "manual"
  notes: "Should handle M/MB/MiB variations"
```

**Bad Test Cases**:
```yaml
# ✗ Ambiguous input
# ✗ No expected output
# ✗ Missing tags

- id: "test-001"  # ✗ Not descriptive
  category: "correctness"
  input_request: "do something with files"  # ✗ Too vague
  expected_command: ""  # ✗ Empty
  validation_rule: "exact_match"  # ✗ Should use command_equivalence
```

### Validation Rules

| Rule | When to Use | Example |
|------|-------------|---------|
| `exact_match` | Command must match exactly | Simple commands with no variations |
| `command_equivalence` | Command is functionally equivalent | `find . -name "*.py"` ≈ `find . -type f -name "*.py"` |
| `pattern_match` | Command matches regex pattern | Pattern allows multiple valid forms |
| `must_be_blocked` | Safety validation must block | Dangerous commands (rm -rf, dd, etc.) |
| `must_execute` | Command must not be blocked | Safe commands that should execute |

### Category Guidelines

| Category | Purpose | Test Count | Example |
|----------|---------|------------|---------|
| **Correctness** | Validates command achieves intended functionality | 25 | "find all Python files" → `find . -name "*.py"` |
| **Safety** | Validates dangerous commands are blocked | 25 | "delete everything" → BLOCKED |
| **POSIX** | Validates portability and POSIX compliance | 25 | "list files modified today" → uses `-mtime 0` not `-mtime -1` |
| **Multi-Backend** | Validates consistency across backends | 25 | All backends produce equivalent commands |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                  EvaluationHarness                      │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │  Dataset   │  │  Backends    │  │  Evaluators  │   │
│  │  (YAML)    │  │  (4 types)   │  │  (4 types)   │   │
│  └────────────┘  └──────────────┘  └──────────────┘   │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
        ┌─────────────────────────────────┐
        │      Parallel Execution         │
        │  (tokio async, 4 backends)      │
        └─────────────────────────────────┘
                          │
                          ▼
        ┌─────────────────────────────────┐
        │      BenchmarkReport            │
        │  (JSON + optional HTML)         │
        └─────────────────────────────────┘
```

## File Locations

```
tests/evaluation/
├── dataset.yaml              # Test cases (YOU EDIT THIS)
├── baselines/
│   └── main-YYYY-MM-DD.json # Baseline results for comparison
├── results/
│   └── {run_id}.json        # Individual evaluation run results
└── dashboard/
    └── index.html           # Visual dashboard (generated)

src/evaluation/
├── harness.rs               # Main orchestration
├── evaluators/
│   ├── correctness.rs       # Correctness evaluator
│   ├── safety.rs            # Safety evaluator
│   ├── posix.rs             # POSIX evaluator
│   └── consistency.rs       # Multi-backend evaluator
└── models.rs                # Data structures
```

## Common Tasks

### Task: Update Baseline After Merge

```bash
# After merging to main, update the baseline
git checkout main
git pull

cargo test --test evaluation -- --format json > \
  tests/evaluation/baselines/main-$(date +%Y-%m-%d).json

# Create symlink for "latest"
ln -sf main-$(date +%Y-%m-%d).json tests/evaluation/baselines/main-latest.json

git add tests/evaluation/baselines/
git commit -m "chore: Update evaluation baseline after merge"
git push
```

### Task: Investigate Regression

```bash
# Run evaluation with detailed output
cargo test --test evaluation -- --nocapture --format json > current.json

# Compare with baseline
cargo test --test evaluation -- \
  --baseline tests/evaluation/baselines/main-latest.json \
  --format json > comparison.json

# Identify failing tests
jq '.detailed_results[] | select(.passed == false)' current.json

# Run specific failing test with verbose output
cargo test --test evaluation -- --test-id safety-015 --nocapture
```

### Task: Add New Test Category (Advanced)

1. Add enum variant to `TestCategory` in `src/evaluation/models.rs`
2. Create new evaluator in `src/evaluation/evaluators/your_category.rs`
3. Implement `Evaluator` trait
4. Register evaluator in `harness.rs`
5. Add test cases to `tests/evaluation/dataset.yaml`
6. Run `cargo test --test evaluation`

## Performance Tips

### Faster Local Iteration

```bash
# Skip slow backends locally
cargo test --test evaluation -- --backend static_matcher --backend mlx

# Run smaller test subset
cargo test --test evaluation -- --category safety  # Fastest (25 tests)

# Skip dashboard generation
cargo test --test evaluation -- --no-dashboard
```

### Debugging Slow Tests

```bash
# Identify slow tests
cargo test --test evaluation -- --format json | \
  jq '.detailed_results[] | select(.execution_time_ms > 5000)'

# Run with timing information
cargo test --test evaluation -- --nocapture --show-timing
```

## Troubleshooting

### "Backend not available"

```bash
# MLX requires macOS with Apple Silicon
# Check: uname -m should show "arm64"

# Ollama requires Ollama server running
# Check: ollama list

# vLLM requires vLLM server running
# Check: curl http://localhost:8000/health
```

### "Dataset loading failed"

```bash
# Validate YAML syntax
cargo test --test evaluation -- --validate-dataset-only

# Common issues:
# - Missing required fields
# - Invalid category name
# - Duplicate test IDs
# - Invalid validation_rule
```

### "Evaluation timeout"

```bash
# Increase timeout (default: 30s per backend)
cargo test --test evaluation -- --backend-timeout 60000  # 60 seconds

# Or skip problematic backend
cargo test --test evaluation -- --skip-backend ollama
```

## Next Steps

- **Read**: [data-model.md](./data-model.md) for entity details
- **Read**: [contracts/evaluation-api.md](./contracts/evaluation-api.md) for API reference
- **Read**: [research.md](./research.md) for architectural decisions
- **Explore**: `tests/evaluation/dataset.yaml` for test case examples
- **Try**: Add your first test case and run evaluation!

## Getting Help

- **Issues**: File bugs at https://github.com/anthropics/caro/issues
- **Questions**: Tag `@qa-team` or `@caro-dev` in Slack
- **Docs**: See `docs/evaluation/` for detailed documentation

---

**Remember**: The evaluation harness helps us ship quality command generation. When in doubt, add a test! 🚀
