//! Prompt A/B testing and statistical comparison.

use serde::{Deserialize, Serialize};

/// Result of comparing a single prompt version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Prompt version identifier
    pub version: String,

    /// Pass rate (0.0 to 1.0)
    pub pass_rate: f64,

    /// Total number of tests
    pub total_tests: usize,

    /// Number of tests that passed
    pub passed_tests: usize,

    /// P-value from statistical test (if compared against baseline)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p_value: Option<f64>,

    /// Whether this is the winning version
    pub is_winner: bool,
}

/// Prompt comparison and A/B testing engine
pub struct PromptComparison;

impl PromptComparison {
    /// Create a new prompt comparison engine
    pub fn new() -> Self {
        Self
    }

    /// Find the winner from a set of prompt comparison results
    pub fn find_winner<'a>(&self, results: &[(&'a str, f64)]) -> &'a str {
        results
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(version, _)| *version)
            .unwrap_or("")
    }

    /// Calculate chi-square test p-value for two prompt versions
    ///
    /// Returns p-value indicating statistical significance
    /// (lower p-value = more significant difference)
    pub fn chi_square_test(&self, v1: (usize, usize), v2: (usize, usize)) -> f64 {
        let (v1_passed, v1_total) = v1;
        let (v2_passed, v2_total) = v2;

        let v1_failed = v1_total - v1_passed;
        let v2_failed = v2_total - v2_passed;

        // Contingency table:
        // | Version | Passed | Failed | Total |
        // |---------|--------|--------|-------|
        // | v1      | a      | b      | n1    |
        // | v2      | c      | d      | n2    |

        let a = v1_passed as f64;
        let b = v1_failed as f64;
        let c = v2_passed as f64;
        let d = v2_failed as f64;

        let n1 = v1_total as f64;
        let n2 = v2_total as f64;
        let n = n1 + n2;

        // Expected frequencies
        let e_a = (a + c) * n1 / n;
        let e_b = (b + d) * n1 / n;
        let e_c = (a + c) * n2 / n;
        let e_d = (b + d) * n2 / n;

        // Chi-square statistic
        let chi_square = ((a - e_a).powi(2) / e_a)
            + ((b - e_b).powi(2) / e_b)
            + ((c - e_c).powi(2) / e_c)
            + ((d - e_d).powi(2) / e_d);

        // Degrees of freedom = 1 for 2x2 table
        // Approximate p-value using chi-square distribution
        // For chi-square with df=1, common critical values:
        // χ² = 2.706 → p = 0.10
        // χ² = 3.841 → p = 0.05
        // χ² = 5.024 → p = 0.025
        // χ² = 6.635 → p = 0.01
        // χ² = 10.828 → p = 0.001

        if chi_square > 10.828 {
            0.001
        } else if chi_square > 6.635 {
            0.01
        } else if chi_square > 5.024 {
            0.025
        } else if chi_square > 3.841 {
            0.05
        } else if chi_square > 2.706 {
            0.10
        } else {
            0.20 // Not significant
        }
    }

    /// Determine if a new prompt should be rolled back
    ///
    /// Returns true if the regression exceeds the threshold percentage
    pub fn should_rollback(
        &self,
        current_champion: f64,
        new_candidate: f64,
        threshold_pct: f64,
    ) -> bool {
        let regression_pct = ((new_candidate - current_champion) / current_champion) * 100.0;
        regression_pct < -threshold_pct
    }

    /// Generate a comparison report
    pub fn generate_report(&self, results: &[ComparisonResult]) -> String {
        let mut report = String::from("Prompt Comparison Results\n");
        report.push_str("========================\n\n");

        for result in results {
            let pass_rate_pct = result.pass_rate * 100.0;
            let winner_marker = if result.is_winner { " ✅ WINNER" } else { "" };

            report.push_str(&format!(
                "{}: {:.1}% pass rate ({}/{})",
                result.version, pass_rate_pct, result.passed_tests, result.total_tests
            ));

            if let Some(p_value) = result.p_value {
                report.push_str(&format!(" (p={:.3})", p_value));
            }

            report.push_str(&format!("{}\n", winner_marker));
        }

        report
    }
}

impl Default for PromptComparison {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_winner_simple() {
        let comparison = PromptComparison::new();
        let results = vec![("v1.0", 30.0), ("v1.1", 45.0), ("v1.2", 28.0)];

        assert_eq!(comparison.find_winner(&results), "v1.1");
    }

    #[test]
    fn test_chi_square_significant_difference() {
        let comparison = PromptComparison::new();

        // 17/55 vs 35/55 - should be significant (larger difference)
        let p_value = comparison.chi_square_test((17, 55), (35, 55));

        assert!(p_value < 0.05, "p-value should indicate significance");
    }

    #[test]
    fn test_rollback_decision() {
        let comparison = PromptComparison::new();

        // Current: 45%, New: 30% = ~33% regression
        assert!(comparison.should_rollback(45.0, 30.0, 10.0));

        // Current: 45%, New: 42% = ~7% regression (below 10% threshold)
        assert!(!comparison.should_rollback(45.0, 42.0, 10.0));
    }
}
