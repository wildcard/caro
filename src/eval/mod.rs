//! Evaluation Framework for Command Generation
//!
//! Provides systematic testing and measurement of command generation quality.
//! Tests cover website claims, natural language variants, edge cases, and
//! platform-specific differences.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use colored::Colorize;

/// Test suite containing multiple evaluation cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalSuite {
    pub name: String,
    pub description: String,
    pub test_cases: Vec<EvalCase>,
}

/// Single evaluation test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCase {
    /// Natural language input
    pub input: String,

    /// Expected command outputs (multiple valid variants allowed)
    pub expected_outputs: Vec<String>,

    /// Category for result grouping
    pub category: EvalCategory,

    /// Human-readable description
    pub description: String,
}

/// Category of evaluation test
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvalCategory {
    /// Website-advertised examples (must pass 100%)
    WebsiteClaim,

    /// Natural language variants (target 80%+)
    Variant,

    /// Edge cases (document behavior)
    EdgeCase,

    /// Platform-specific (GNU vs BSD)
    PlatformSpecific,
}

impl std::fmt::Display for EvalCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalCategory::WebsiteClaim => write!(f, "Website Claim"),
            EvalCategory::Variant => write!(f, "Natural Variant"),
            EvalCategory::EdgeCase => write!(f, "Edge Case"),
            EvalCategory::PlatformSpecific => write!(f, "Platform-Specific"),
        }
    }
}

/// Result of running an evaluation suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResults {
    pub suite_name: String,
    pub backend: String,
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub results_by_category: HashMap<String, CategoryResults>,
    pub individual_results: Vec<IndividualResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResults {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualResult {
    pub input: String,
    pub expected: Vec<String>,
    pub actual: Option<String>,
    pub passed: bool,
    pub category: EvalCategory,
    pub error: Option<String>,
}

impl EvalResults {
    /// Calculate pass rate for the entire suite
    pub fn pass_rate(&self) -> f64 {
        if self.total_cases == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total_cases as f64) * 100.0
        }
    }

    /// Print results in a human-readable format
    pub fn print_summary(&self) {
        println!("\n{}", "Evaluation Results".bold());
        println!("{} {}", "Suite:".bold(), self.suite_name);
        println!("{} {}", "Backend:".bold(), self.backend);
        println!();

        // Overall summary
        let pass_rate_color = if self.pass_rate() >= 80.0 {
            "green"
        } else if self.pass_rate() >= 60.0 {
            "yellow"
        } else {
            "red"
        };

        println!(
            "{} {}/{} ({:.1}%)",
            "Overall:".bold(),
            self.passed.to_string().color(pass_rate_color),
            self.total_cases,
            self.pass_rate()
        );
        println!();

        // By category
        println!("{}", "Results by Category:".bold());
        for (category, results) in &self.results_by_category {
            let cat_color = if results.pass_rate >= 100.0 {
                "green"
            } else if results.pass_rate >= 80.0 {
                "yellow"
            } else {
                "red"
            };

            println!(
                "  {}: {}/{} ({:.1}%)",
                category,
                results.passed.to_string().color(cat_color),
                results.total,
                results.pass_rate
            );
        }
        println!();

        // Failed cases
        if self.failed > 0 {
            println!("{}", "Failed Cases:".bold().red());
            for result in &self.individual_results {
                if !result.passed {
                    println!("  ✗ [{}] {}", result.category, result.input);
                    println!("    Expected: {}", result.expected.join(" OR ").dimmed());
                    if let Some(ref actual) = result.actual {
                        println!("    Got: {}", actual.yellow());
                    } else if let Some(ref err) = result.error {
                        println!("    Error: {}", err.red());
                    }
                    println!();
                }
            }
        }
    }
}

impl EvalSuite {
    /// Create the default evaluation suite with website claims
    pub fn default_suite() -> Self {
        Self {
            name: "Default Command Generation Eval".to_string(),
            description: "Tests website claims, variants, and edge cases".to_string(),
            test_cases: Self::default_test_cases(),
        }
    }

    /// Build default test cases covering all categories
    fn default_test_cases() -> Vec<EvalCase> {
        let mut cases = vec![];

        // Website Claims (P0 - must pass 100%)
        cases.extend(vec![
            EvalCase {
                input: "list all files modified today".to_string(),
                expected_outputs: vec!["find . -type f -mtime 0".to_string()],
                category: EvalCategory::WebsiteClaim,
                description: "Website example #1".to_string(),
            },
            EvalCase {
                input: "find large files over 100MB".to_string(),
                expected_outputs: vec!["find . -type f -size +100M".to_string()],
                category: EvalCategory::WebsiteClaim,
                description: "Website example #2".to_string(),
            },
            EvalCase {
                input: "show disk usage by folder".to_string(),
                expected_outputs: vec!["du -sh */ | sort -rh | head -10".to_string()],
                category: EvalCategory::WebsiteClaim,
                description: "Website example #3".to_string(),
            },
            EvalCase {
                input: "find python files modified last week".to_string(),
                expected_outputs: vec!["find . -name \"*.py\" -type f -mtime -7".to_string()],
                category: EvalCategory::WebsiteClaim,
                description: "Website example #4".to_string(),
            },
        ]);

        // Natural Variants (P1 - target 80%+)
        cases.extend(vec![
            EvalCase {
                input: "files changed today".to_string(),
                expected_outputs: vec!["find . -type f -mtime 0".to_string()],
                category: EvalCategory::Variant,
                description: "Shorter phrasing of 'files modified today'".to_string(),
            },
            EvalCase {
                input: "show me all files that were modified today".to_string(),
                expected_outputs: vec!["find . -type f -mtime 0".to_string()],
                category: EvalCategory::Variant,
                description: "Verbose phrasing of 'files modified today'".to_string(),
            },
            EvalCase {
                input: "list files bigger than 100 megabytes".to_string(),
                expected_outputs: vec!["find . -type f -size +100M".to_string()],
                category: EvalCategory::Variant,
                description: "Different wording for 'large files over 100MB'".to_string(),
            },
            EvalCase {
                input: "disk space used by each folder".to_string(),
                expected_outputs: vec!["du -sh */ | sort -rh | head -10".to_string()],
                category: EvalCategory::Variant,
                description: "Different phrasing for 'disk usage by folder'".to_string(),
            },
            EvalCase {
                input: "python files from the last 7 days".to_string(),
                expected_outputs: vec!["find . -name \"*.py\" -type f -mtime -7".to_string()],
                category: EvalCategory::Variant,
                description: "Different phrasing for 'python files modified last week'".to_string(),
            },
        ]);

        // Edge Cases (P2 - document behavior)
        cases.extend(vec![
            EvalCase {
                input: "files modified yesterday".to_string(),
                expected_outputs: vec!["find . -type f -mtime 1".to_string()],
                category: EvalCategory::EdgeCase,
                description: "Similar to 'today' but different time offset".to_string(),
            },
            EvalCase {
                input: "large javascript files over 50MB".to_string(),
                expected_outputs: vec!["find . -name \"*.js\" -type f -size +50M".to_string()],
                category: EvalCategory::EdgeCase,
                description: "Combination: file type + size filter".to_string(),
            },
        ]);

        cases
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_suite_creation() {
        let suite = EvalSuite::default_suite();
        assert!(!suite.test_cases.is_empty());

        // Should have website claims
        let website_claims: Vec<_> = suite
            .test_cases
            .iter()
            .filter(|c| matches!(c.category, EvalCategory::WebsiteClaim))
            .collect();
        assert_eq!(website_claims.len(), 4);
    }

    #[test]
    fn test_category_grouping() {
        let suite = EvalSuite::default_suite();

        let categories: Vec<_> = suite
            .test_cases
            .iter()
            .map(|c| c.category)
            .collect();

        assert!(categories.contains(&EvalCategory::WebsiteClaim));
        assert!(categories.contains(&EvalCategory::Variant));
        assert!(categories.contains(&EvalCategory::EdgeCase));
    }
}
