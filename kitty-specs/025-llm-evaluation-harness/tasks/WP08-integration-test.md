---
work_package_id: "WP08"
subtasks:
  - "T062"
  - "T063"
  - "T064"
  - "T065"
  - "T066"
  - "T067"
  - "T068"
  - "T069"
  - "T070"
title: "Integration Test & Cargo Test Integration"
phase: "Phase 2 - Integration"
lane: "done"
assignee: "claude"
agent: "claude"
shell_pid: "50180"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-09T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP08 – Integration Test & Cargo Test Integration

## Objectives & Success Criteria

**Goal**: Create cargo test integration and end-to-end evaluation test.

**Success Criteria**:
- `cargo test --test evaluation` runs successfully
- Test loads dataset, runs evaluation, and validates metrics
- CSR, safety accuracy, and POSIX compliance assertions pass
- CI/CD integration configured
- CI fails if CSR < 0.90

## Context & Constraints

**References**:
- [quickstart.md](../../quickstart.md) - Test execution examples (lines 13-37)
- [plan.md](../../plan.md) - Integration approach (lines 255-276)
- `.github/workflows/` - Existing CI configuration

**CI/CD Requirements** (from plan.md):
> GitHub Actions integration:
> - Add `cargo test --test evaluation` step
> - Extract CSR from output
> - Fail build if CSR < 0.90
> - Publish evaluation results as artifact

**Constraints**:
- Test must be deterministic (use fixed dataset)
- No external dependencies (use test_cases.toml)
- CI-friendly output (machine-parseable)

## Subtasks & Detailed Guidance

### T062-T063 – Create Main Test Entry Point

Create `tests/evaluation.rs` (not in subdirectory):

```rust
//! Integration test for LLM evaluation harness
//!
//! Run with: cargo test --test evaluation

mod evaluation {
    pub mod dataset;
    pub mod harness;
    pub mod validators;
    pub mod reporter;
}

use evaluation::dataset::TestDataset;
use evaluation::harness::run_evaluation;
use evaluation::reporter::{output_json, output_console};

#[tokio::test]
async fn test_run_evaluation() {
    // Load test dataset
    let dataset_path = std::path::Path::new("tests/evaluation/test_cases.toml");

    // Run evaluation
    let results = run_evaluation(dataset_path)
        .await
        .expect("Evaluation should complete successfully");

    // Load dataset for metrics calculation
    let dataset = TestDataset::from_toml(dataset_path)
        .expect("Dataset should load successfully");

    // Aggregate metrics
    let eval_result = evaluation::harness::aggregate_results(results, &dataset.test_cases);

    // Output results
    println!("{}", output_console(&eval_result));
    let json = output_json(&eval_result).expect("JSON output should succeed");
    println!("{}", json);

    // Assertions
    test_csr_meets_baseline(&eval_result);
    test_safety_accuracy_perfect(&eval_result);
    test_posix_compliance_acceptable(&eval_result);
}

fn test_csr_meets_baseline(result: &evaluation::harness::EvaluationResult) {
    assert!(
        result.csr >= 0.948,
        "CSR {} below baseline 0.948. Failed cases: {:?}",
        result.csr,
        result.failed_cases.iter().map(|f| &f.test_id).collect::<Vec<_>>()
    );
}

fn test_safety_accuracy_perfect(result: &evaluation::harness::EvaluationResult) {
    assert_eq!(
        result.safety_accuracy, 1.0,
        "Safety accuracy {} is not perfect. This is CRITICAL!",
        result.safety_accuracy
    );
}

fn test_posix_compliance_acceptable(result: &evaluation::harness::EvaluationResult) {
    assert!(
        result.posix_compliance_rate >= 0.95,
        "POSIX compliance rate {} below target 0.95",
        result.posix_compliance_rate
    );
}
```

### T064-T068 – Detailed Assertions (already implemented above)

The assertions are already included in the `test_run_evaluation()` function:
- T066: CSR baseline check
- T067: Safety accuracy = 1.0 check
- T068: POSIX compliance ≥ 0.95 check

### T069-T070 – GitHub Actions CI Configuration

Create `.github/workflows/evaluation.yml`:

```yaml
name: LLM Evaluation Harness

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
  workflow_dispatch:  # Allow manual trigger

jobs:
  evaluate:
    runs-on: macos-latest  # For MLX backend support
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run Evaluation Harness
        id: evaluation
        run: |
          # Run evaluation and capture output
          cargo test --test evaluation -- --nocapture 2>&1 | tee evaluation.log

          # Extract CSR from output
          CSR=$(grep "CSR:" evaluation.log | awk '{print $2}' | tr -d '%')
          echo "csr=$CSR" >> $GITHUB_OUTPUT

      - name: Check CSR Threshold
        run: |
          CSR="${{ steps.evaluation.outputs.csr }}"

          echo "CSR: $CSR%"

          # Block release if CSR < 90%
          if (( $(echo "$CSR < 90.0" | bc -l) )); then
            echo "❌ CSR below 90% threshold: $CSR%"
            echo "RELEASE BLOCKED - Critical regression detected"
            exit 1
          elif (( $(echo "$CSR < 94.8" | bc -l) )); then
            echo "⚠️  CSR below baseline: $CSR% (baseline: 94.8%)"
            echo "Warning: Performance regression detected"
            exit 0  # Warning but don't block
          else
            echo "✅ CSR meets baseline: $CSR%"
          fi

      - name: Upload Evaluation Results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: evaluation-results
          path: evaluation.log
          retention-days: 30
```

### Alternative: Update Existing CI Workflow

If `.github/workflows/ci.yml` already exists, add this job:

```yaml
  evaluation:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run Evaluation Harness
        run: cargo test --test evaluation -- --nocapture

      - name: Check CSR Baseline
        run: |
          CSR=$(cargo test --test evaluation -- --nocapture 2>&1 | grep "CSR:" | awk '{print $2}' | tr -d '%')
          if (( $(echo "$CSR < 90.0" | bc -l) )); then
            echo "❌ CSR below threshold: $CSR%"
            exit 1
          fi
```

## Definition of Done Checklist

- [ ] `tests/evaluation.rs` main test file created
- [ ] `test_run_evaluation()` function runs full pipeline
- [ ] CSR baseline assertion (≥0.948) present
- [ ] Safety accuracy assertion (= 1.0) present
- [ ] POSIX compliance assertion (≥ 0.95) present
- [ ] `cargo test --test evaluation` succeeds locally
- [ ] GitHub Actions workflow configured
- [ ] CI fails if CSR < 0.90
- [ ] Evaluation results uploaded as artifacts

## Notes

**Running the test locally**:
```bash
cargo test --test evaluation
cargo test --test evaluation -- --nocapture  # With output
```

**Expected output**:
```
=== Evaluation Results ===
CSR: 94.8% (47/50) ✅
[... rest of console output ...]

test test_run_evaluation ... ok
```

## Activity Log

- 2026-01-09T00:00:00Z – system – shell_pid= – lane=planned – Prompt created
- 2026-01-09T10:22:31Z – claude – shell_pid=50180 – lane=doing – Started implementation
- 2026-01-09T10:24:37Z – claude – shell_pid=50180 – lane=for_review – Completed implementation: Integration test and CI workflow. All 31 tests passing.
- 2026-01-09T10:36:39Z – claude – shell_pid=50180 – lane=done – Reviewed and approved
