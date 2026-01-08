# Quickstart: LLM Evaluation Harness

**Feature**: 022-issue-135-build
**Last Updated**: 2026-01-08

## Overview

The LLM evaluation harness validates caro's command generation quality across correctness, safety, and POSIX compliance dimensions. It runs test datasets through configured backends and generates detailed reports.

## Prerequisites

1. **Caro binary built**:
   ```bash
   cargo build --release
   # Binary at: target/release/caro
   ```

2. **Shellcheck installed** (for POSIX validation):
   ```bash
   # macOS
   brew install shellcheck

   # Linux (Debian/Ubuntu)
   sudo apt-get install shellcheck

   # Verify installation
   shellcheck --version
   ```

3. **Backend configured** (at least one):
   - MLX backend (Apple Silicon)
   - Ollama backend (local server)
   - vLLM backend (remote server)

## Quick Start

### 1. Run Evaluation on Sample Dataset

```bash
# Navigate to evaluation directory
cd tests/evaluation

# Run correctness evaluation on MLX backend
cargo test --test test_correctness -- --nocapture

# Run all evaluation tests
cargo test -- --nocapture
```

### 2. Run Full Evaluation Suite

```bash
# From project root
cargo test --package caro-evaluation -- --nocapture

# With specific backend
CARO_BACKEND=mlx cargo test --package caro-evaluation -- --nocapture

# Generate detailed report
cargo test --package caro-evaluation --test test_full_evaluation -- --nocapture
```

### 3. View Results

```bash
# Results written to:
tests/evaluation/results/run_YYYY-MM-DD_HHMMSS.json
tests/evaluation/results/run_YYYY-MM-DD_HHMMSS.md

# View latest Markdown report
cat tests/evaluation/results/$(ls -t tests/evaluation/results/*.md | head -1)
```

## Integration Scenarios

### Scenario 1: CI/CD Integration (Pre-commit Hook)

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running LLM evaluation harness..."

# Run quick smoke test (10 critical cases)
cd tests/evaluation
cargo test --test test_smoke -- --nocapture

if [ $? -ne 0 ]; then
    echo "❌ Evaluation failed! Commit blocked."
    echo "Run 'cargo test --package caro-evaluation' to see details."
    exit 1
fi

echo "✅ Evaluation passed!"
exit 0
```

**Usage**: Automatically validates command generation quality before each commit.

---

### Scenario 2: Regression Detection (GitHub Actions)

```yaml
# .github/workflows/evaluation.yml
name: LLM Quality Regression Check

on: [pull_request]

jobs:
  evaluate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install shellcheck
        run: sudo apt-get install -y shellcheck

      - name: Build caro
        run: cargo build --release

      - name: Run evaluation harness
        run: cargo test --package caro-evaluation -- --nocapture

      - name: Check correctness threshold
        run: |
          # Parse JSON result
          CORRECTNESS=$(jq '.summary_stats.average_correctness' tests/evaluation/results/latest.json)

          # Fail if below 85%
          if (( $(echo "$CORRECTNESS < 0.85" | bc -l) )); then
            echo "❌ Correctness dropped below 85%: $CORRECTNESS"
            exit 1
          fi

          echo "✅ Correctness: $CORRECTNESS"

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: evaluation-results
          path: tests/evaluation/results/
```

**Usage**: Blocks PRs that degrade command generation quality.

---

### Scenario 3: Backend Comparison Analysis

```bash
#!/bin/bash
# scripts/compare_backends.sh

# Run evaluation on all backends
for backend in mlx ollama vllm; do
    echo "Evaluating $backend backend..."
    CARO_BACKEND=$backend cargo test --package caro-evaluation --test test_backends -- --nocapture
done

# Generate comparison report
cargo test --package caro-evaluation --test test_backend_comparison -- --nocapture

# Results at: tests/evaluation/results/backend_comparison_YYYY-MM-DD.md
```

**Usage**: Compare quality metrics across different LLM backends.

---

### Scenario 4: Dataset Validation (Before Adding New Cases)

```bash
# Validate new test cases before committing
python3 scripts/validate_dataset.py tests/evaluation/datasets/correctness/new_cases.json

# Expected output:
# ✅ All 25 test cases valid
# ✅ No duplicate IDs
# ✅ All expected_commands are valid shell syntax
# ✅ Categories match allowed values
# ✅ Risk levels are valid
```

**Usage**: Ensure test dataset quality before version control.

---

### Scenario 5: Safety Pattern Coverage Report

```bash
# Generate coverage report for safety patterns
cargo test --package caro-evaluation --test test_safety_coverage -- --nocapture

# Output shows:
# - Which src/safety/ patterns are covered by tests
# - Which patterns have no test coverage
# - False positive/negative rates per pattern
```

**Usage**: Validate that all safety patterns have test coverage (SC-002).

---

## Example Outputs

### Correctness Evaluation Result

```json
{
  "test_case_id": "correctness_file_001",
  "backend_name": "mlx",
  "generated_command": "ls -a",
  "correctness_score": 0.95,
  "correctness_method": "semantic_equivalent",
  "safety_validation": {
    "is_dangerous": false,
    "risk_level": "safe"
  },
  "performance_metrics": {
    "inference_latency_ms": 450
  }
}
```

### Summary Report (Markdown)

```markdown
# Evaluation Run: 2026-01-08 14:30:52

## Summary
- **Dataset**: correctness_v1 (100 cases)
- **Backend**: MLX (Qwen2.5-Coder-1.5B-Instruct)
- **Duration**: 3m 42s

## Results
- **Overall Correctness**: 87.5% (87/100 exact or semantic matches)
- **Safety Accuracy**: 98.0% (49/50 TP, 1/50 FP)
- **POSIX Compliance**: 92.0% (92/100 portable commands)
- **Average Latency**: 2.2s per command

## By Category
| Category | Cases | Correctness | Latency |
|----------|-------|-------------|---------|
| file_operations | 30 | 93.3% | 1.8s |
| text_processing | 25 | 84.0% | 2.4s |
| network | 20 | 85.0% | 2.8s |
| system_admin | 25 | 84.0% | 2.0s |

## Regressions
⚠️ text_processing category dropped from 90% to 84% (-6%)
   - Investigate cases: text_proc_015, text_proc_023, text_proc_031
```

---

## Troubleshooting

### Issue: "Caro binary not found"
```bash
# Solution: Build caro first
cargo build --release

# Verify binary exists
ls -la target/release/caro
```

### Issue: "Shellcheck not installed"
```bash
# Solution: Install shellcheck
brew install shellcheck  # macOS
sudo apt-get install shellcheck  # Linux
```

### Issue: "Permission denied when running evaluation"
```bash
# Solution: Make caro binary executable
chmod +x target/release/caro
```

### Issue: "Tests timeout after 60s"
```bash
# Solution: Increase timeout for slow backends
cargo test --package caro-evaluation -- --nocapture --test-threads=1 --timeout=300
```

---

## Next Steps

1. **Review Results**: Check tests/evaluation/results/ for latest reports
2. **Address Regressions**: Fix any quality drops in specific categories
3. **Expand Datasets**: Add more test cases to improve coverage
4. **Optimize Performance**: If evaluation takes > 5 minutes for 100 cases, investigate bottlenecks
5. **CI Integration**: Set up automated evaluation in GitHub Actions

---

## Related Documentation

- [spec.md](spec.md) - Feature requirements and success criteria
- [plan.md](plan.md) - Implementation plan and architecture
- [data-model.md](data-model.md) - Evaluation domain types and schemas
- `tests/evaluation/README.md` - Detailed usage documentation
