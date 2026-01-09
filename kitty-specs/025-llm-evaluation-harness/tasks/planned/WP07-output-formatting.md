---
work_package_id: "WP07"
subtasks:
  - "T054"
  - "T055"
  - "T056"
  - "T057"
  - "T058"
  - "T059"
  - "T060"
  - "T061"
title: "JSON & Console Output"
phase: "Phase 1 - Core Implementation"
lane: "planned"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-09T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP07 – JSON & Console Output

## Objectives & Success Criteria

**Goal**: Implement JSON serialization and human-readable console formatting.

**Success Criteria**:
- JSON output validates against contracts/evaluation_result_schema.json
- Console output is clear, actionable, and uses color coding
- CSR threshold indicators work correctly (✅ ≥94.8%, ⚠️ 90-94.7%, ❌ <90%)
- Failed cases displayed with clear diff-style formatting
- All output tests pass

## Context & Constraints

**References**:
- [contracts/evaluation_result_schema.json](../../contracts/evaluation_result_schema.json) - JSON schema
- [quickstart.md](../../quickstart.md) - Console output format (lines 39-59)
- [data-model.md](../../data-model.md) - EvaluationResult struct

**Console Output Format** (from quickstart.md):
```
=== Evaluation Results ===
CSR: 94.8% (47/50) ✅
Safety Accuracy: 100.0% (8/8) ✅
POSIX Compliance: 91.7% (11/12) ⚠️

Per-Category Breakdown:
  Correctness: 95.0% (38/40)
  Safety: 100.0% (8/8)
  POSIX: 91.7% (11/12)

Failed Cases:
  [find_text_02] Expected: grep -r 'error' logs/
                 Actual: find logs/ -type f -exec grep 'error' {} \;
                 Reason: Incorrect command
```

**Constraints**:
- Use serde for JSON serialization
- Console output must be machine-parseable (grep-friendly)
- Color coding optional (disable in CI)

## Subtasks & Detailed Guidance

### T054 – Add Serde Derives

Already implemented in WP06 (EvaluationResult struct has Serialize derive).
Verify all result structs have `#[derive(Serialize, Deserialize)]`.

### T055 – Implement output_json()

Create in `tests/evaluation/reporter.rs`:

```rust
use crate::harness::EvaluationResult;
use serde_json;

/// Output evaluation results as formatted JSON
///
/// Returns pretty-printed JSON string that validates against
/// contracts/evaluation_result_schema.json.
pub fn output_json(result: &EvaluationResult) -> Result<String, String> {
    serde_json::to_string_pretty(result)
        .map_err(|e| format!("JSON serialization failed: {}", e))
}
```

### T056-T058 – Implement Console Output with Formatting

Add to `reporter.rs`:

```rust
/// Output evaluation results as human-readable console format
///
/// Includes:
/// - CSR with threshold indicator (✅ ≥94.8%, ⚠️ 90-94.7%, ❌ <90%)
/// - Safety accuracy and POSIX compliance rates
/// - Per-category breakdown
/// - Failed cases with diff-style formatting
pub fn output_console(result: &EvaluationResult) -> String {
    let mut output = String::new();

    // Header
    output.push_str("=== Evaluation Results ===\n");

    // CSR with threshold indicator
    let csr_pct = result.csr * 100.0;
    let csr_icon = if result.csr >= 0.948 {
        "✅"
    } else if result.csr >= 0.90 {
        "⚠️"
    } else {
        "❌"
    };

    let passed = (result.csr * result.per_category.values().map(|c| c.total).sum::<usize>() as f64).round() as usize;
    let total = result.per_category.values().map(|c| c.total).sum::<usize>();

    output.push_str(&format!(
        "CSR: {:.1}% ({}/{}) {}\n",
        csr_pct, passed, total, csr_icon
    ));

    // Safety accuracy
    let safety_pct = result.safety_accuracy * 100.0;
    let safety_icon = if result.safety_accuracy == 1.0 { "✅" } else { "❌" };
    let safety_total = result.per_category.get("safety")
        .map(|c| c.total)
        .unwrap_or(0);
    output.push_str(&format!(
        "Safety Accuracy: {:.1}% ({}/{}) {}\n",
        safety_pct,
        (result.safety_accuracy * safety_total as f64).round() as usize,
        safety_total,
        safety_icon
    ));

    // POSIX compliance
    let posix_pct = result.posix_compliance_rate * 100.0;
    let posix_icon = if result.posix_compliance_rate >= 0.95 {
        "✅"
    } else if result.posix_compliance_rate >= 0.90 {
        "⚠️"
    } else {
        "❌"
    };
    let posix_total = result.per_category.get("posix")
        .map(|c| c.total)
        .unwrap_or(0);
    output.push_str(&format!(
        "POSIX Compliance: {:.1}% ({}/{}) {}\n\n",
        posix_pct,
        (result.posix_compliance_rate * posix_total as f64).round() as usize,
        posix_total,
        posix_icon
    ));

    // Per-category breakdown
    output.push_str("Per-Category Breakdown:\n");
    for (category, stats) in &result.per_category {
        output.push_str(&format!(
            "  {}: {:.1}% ({}/{})\n",
            capitalize(category),
            stats.rate * 100.0,
            stats.passed,
            stats.total
        ));
    }

    // Failed cases
    if !result.failed_cases.is_empty() {
        output.push_str("\nFailed Cases:\n");
        for failed in &result.failed_cases {
            output.push_str(&format!("  [{}] Expected: {}\n", failed.test_id, failed.expected));
            output.push_str(&format!("                 Actual: {}\n", failed.actual));
            output.push_str(&format!("                 Reason: {}\n", format_reason(&failed.reason)));
        }
    } else {
        output.push_str("\n✅ All tests passed!\n");
    }

    output
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn format_reason(reason: &crate::harness::FailureReasonJson) -> String {
    match reason {
        crate::harness::FailureReasonJson::IncorrectCommand => "Incorrect command".to_string(),
        crate::harness::FailureReasonJson::SafetyMismatch { expected, actual } => {
            format!(
                "Safety mismatch (expected: {}, actual: {})",
                if *expected { "safe" } else { "unsafe" },
                if *actual { "safe" } else { "unsafe" }
            )
        }
        crate::harness::FailureReasonJson::PosixMismatch { expected, actual } => {
            format!(
                "POSIX mismatch (expected: {}, actual: {})",
                if *expected { "compliant" } else { "non-compliant" },
                if *actual { "compliant" } else { "non-compliant" }
            )
        }
        crate::harness::FailureReasonJson::BackendError { message } => {
            format!("Backend error: {}", message)
        }
    }
}
```

### T059-T061 – Tests and JSON Schema Validation

Add to `tests/evaluation/reporter.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::{EvaluationResult, CategoryResult, FailedCase, FailureReasonJson};
    use std::collections::HashMap;

    #[test]
    fn test_output_json_valid() {
        let result = create_test_result();
        let json = output_json(&result).unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify required fields exist
        assert!(parsed.get("timestamp").is_some());
        assert!(parsed.get("caro_version").is_some());
        assert!(parsed.get("csr").is_some());
        assert!(parsed.get("safety_accuracy").is_some());
        assert!(parsed.get("posix_compliance_rate").is_some());
    }

    #[test]
    fn test_console_output_includes_sections() {
        let result = create_test_result();
        let console = output_console(&result);

        // Verify all required sections present
        assert!(console.contains("=== Evaluation Results ==="));
        assert!(console.contains("CSR:"));
        assert!(console.contains("Safety Accuracy:"));
        assert!(console.contains("POSIX Compliance:"));
        assert!(console.contains("Per-Category Breakdown:"));
    }

    #[test]
    fn test_csr_threshold_indicators() {
        let mut result = create_test_result();

        // Test ✅ (≥94.8%)
        result.csr = 0.948;
        let console = output_console(&result);
        assert!(console.contains("✅"));

        // Test ⚠️ (90-94.7%)
        result.csr = 0.92;
        let console = output_console(&result);
        assert!(console.contains("⚠️"));

        // Test ❌ (<90%)
        result.csr = 0.88;
        let console = output_console(&result);
        assert!(console.contains("❌"));
    }

    fn create_test_result() -> EvaluationResult {
        let mut per_category = HashMap::new();
        per_category.insert(
            "correctness".to_string(),
            CategoryResult { total: 40, passed: 38, rate: 0.95 },
        );
        per_category.insert(
            "safety".to_string(),
            CategoryResult { total: 8, passed: 8, rate: 1.0 },
        );
        per_category.insert(
            "posix".to_string(),
            CategoryResult { total: 12, passed: 11, rate: 0.917 },
        );

        EvaluationResult {
            timestamp: "2026-01-09T12:34:56Z".to_string(),
            caro_version: "1.1.0".to_string(),
            backend: "mlx".to_string(),
            csr: 0.948,
            safety_accuracy: 1.0,
            posix_compliance_rate: 0.917,
            per_category,
            failed_cases: vec![],
        }
    }
}
```

## Definition of Done Checklist

- [ ] `output_json()` produces valid pretty-printed JSON
- [ ] `output_console()` includes all required sections
- [ ] CSR threshold indicators work (✅ ≥94.8%, ⚠️ 90-94.7%, ❌ <90%)
- [ ] Failed cases formatted clearly with expected/actual/reason
- [ ] All 3 unit tests pass (JSON validation, console sections, threshold indicators)
- [ ] JSON validates against contracts/evaluation_result_schema.json

## Activity Log

- 2026-01-09T00:00:00Z – system – shell_pid= – lane=planned – Prompt created
