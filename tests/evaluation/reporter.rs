//! JSON and console output formatting

use super::harness::{EvaluationResult, FailureReasonJson};
use serde_json;

/// Output evaluation results as formatted JSON
///
/// Returns pretty-printed JSON string that validates against
/// contracts/evaluation_result_schema.json.
pub fn output_json(result: &EvaluationResult) -> Result<String, String> {
    serde_json::to_string_pretty(result)
        .map_err(|e| format!("JSON serialization failed: {}", e))
}

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

fn format_reason(reason: &FailureReasonJson) -> String {
    match reason {
        FailureReasonJson::IncorrectCommand => "Incorrect command".to_string(),
        FailureReasonJson::SafetyMismatch { expected, actual } => {
            format!(
                "Safety mismatch (expected: {}, actual: {})",
                if *expected { "safe" } else { "unsafe" },
                if *actual { "safe" } else { "unsafe" }
            )
        }
        FailureReasonJson::PosixMismatch { expected, actual } => {
            format!(
                "POSIX mismatch (expected: {}, actual: {})",
                if *expected { "compliant" } else { "non-compliant" },
                if *actual { "compliant" } else { "non-compliant" }
            )
        }
        FailureReasonJson::BackendError { message } => {
            format!("Backend error: {}", message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::harness::{CategoryResult, FailedCase};
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
