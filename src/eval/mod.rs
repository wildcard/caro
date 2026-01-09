//! Evaluation Framework for Command Generation
//!
//! Provides systematic testing and measurement of command generation quality.
//! Tests cover website claims, natural language variants, edge cases, and
//! platform-specific differences.

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
    /// Test case ID (e.g., "fm_001")
    #[serde(default)]
    pub id: Option<String>,

    /// Natural language input
    pub input: String,

    /// Expected command output (singular form for YAML compatibility)
    #[serde(default, rename = "expected_output")]
    pub expected_output: Option<String>,

    /// Expected command outputs (multiple valid variants allowed)
    #[serde(default, rename = "expected_outputs")]
    pub expected_outputs: Vec<String>,

    /// Category for result grouping
    pub category: EvalCategory,

    /// Human-readable description
    #[serde(default)]
    pub description: String,

    /// Primary tester profile for this test
    #[serde(default)]
    pub primary_profile: Option<String>,

    /// Secondary tester profiles for this test
    #[serde(default)]
    pub secondary_profiles: Option<Vec<String>>,

    /// Source location in website/docs
    #[serde(default)]
    pub source: Option<String>,

    /// Risk level (safe, moderate, dangerous)
    #[serde(default)]
    pub risk_level: Option<String>,
}

/// Category of evaluation test
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalCategory {
    /// Website-advertised examples (must pass 100%)
    WebsiteClaim,

    /// Natural language variants (target 80%+)
    Variant,

    /// Edge cases (document behavior)
    EdgeCase,

    /// Platform-specific (GNU vs BSD)
    PlatformSpecific,

    /// File management operations (find, ls, du)
    FileManagement,

    /// System monitoring (ps, top, lsof)
    SystemMonitoring,

    /// Git and version control
    GitVersionControl,

    /// Log analysis (grep, journalctl, awk)
    LogAnalysis,

    /// Network operations (ping, ss, wget)
    NetworkOperations,

    /// DevOps and Kubernetes
    DevopsKubernetes,

    /// Text processing (sed, awk, grep)
    TextProcessing,

    /// Dangerous commands (safety testing)
    DangerousCommands,
}

impl std::fmt::Display for EvalCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalCategory::WebsiteClaim => write!(f, "Website Claim"),
            EvalCategory::Variant => write!(f, "Natural Variant"),
            EvalCategory::EdgeCase => write!(f, "Edge Case"),
            EvalCategory::PlatformSpecific => write!(f, "Platform-Specific"),
            EvalCategory::FileManagement => write!(f, "File Management"),
            EvalCategory::SystemMonitoring => write!(f, "System Monitoring"),
            EvalCategory::GitVersionControl => write!(f, "Git Version Control"),
            EvalCategory::LogAnalysis => write!(f, "Log Analysis"),
            EvalCategory::NetworkOperations => write!(f, "Network Operations"),
            EvalCategory::DevopsKubernetes => write!(f, "DevOps/Kubernetes"),
            EvalCategory::TextProcessing => write!(f, "Text Processing"),
            EvalCategory::DangerousCommands => write!(f, "Dangerous Commands"),
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
                id: Some("wc_001".to_string()),
                input: "list all files modified today".to_string(),
                expected_output: Some("find . -type f -mtime 0".to_string()),
                expected_outputs: vec!["find . -type f -mtime 0".to_string()],
                category: EvalCategory::WebsiteClaim,
                description: "Website example #1".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
            EvalCase {
                id: Some("wc_002".to_string()),
                input: "find large files over 100MB".to_string(),
                expected_output: Some("find . -type f -size +100M".to_string()),
                expected_outputs: vec!["find . -type f -size +100M".to_string()],
                category: EvalCategory::WebsiteClaim,
                description: "Website example #2".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
            EvalCase {
                id: Some("wc_003".to_string()),
                input: "show disk usage by folder".to_string(),
                expected_output: Some("du -sh */ | sort -rh | head -10".to_string()),
                expected_outputs: vec!["du -sh */ | sort -rh | head -10".to_string()],
                category: EvalCategory::WebsiteClaim,
                description: "Website example #3".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
            EvalCase {
                id: Some("wc_004".to_string()),
                input: "find python files modified last week".to_string(),
                expected_output: Some("find . -name \"*.py\" -type f -mtime -7".to_string()),
                expected_outputs: vec!["find . -name \"*.py\" -type f -mtime -7".to_string()],
                category: EvalCategory::WebsiteClaim,
                description: "Website example #4".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
        ]);

        // Natural Variants (P1 - target 80%+)
        cases.extend(vec![
            EvalCase {
                id: Some("var_001".to_string()),
                input: "files changed today".to_string(),
                expected_output: Some("find . -type f -mtime 0".to_string()),
                expected_outputs: vec!["find . -type f -mtime 0".to_string()],
                category: EvalCategory::Variant,
                description: "Shorter phrasing of 'files modified today'".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
            EvalCase {
                id: Some("var_002".to_string()),
                input: "show me all files that were modified today".to_string(),
                expected_output: Some("find . -type f -mtime 0".to_string()),
                expected_outputs: vec!["find . -type f -mtime 0".to_string()],
                category: EvalCategory::Variant,
                description: "Verbose phrasing of 'files modified today'".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
            EvalCase {
                id: Some("var_003".to_string()),
                input: "list files bigger than 100 megabytes".to_string(),
                expected_output: Some("find . -type f -size +100M".to_string()),
                expected_outputs: vec!["find . -type f -size +100M".to_string()],
                category: EvalCategory::Variant,
                description: "Different wording for 'large files over 100MB'".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
            EvalCase {
                id: Some("var_004".to_string()),
                input: "disk space used by each folder".to_string(),
                expected_output: Some("du -sh */ | sort -rh | head -10".to_string()),
                expected_outputs: vec!["du -sh */ | sort -rh | head -10".to_string()],
                category: EvalCategory::Variant,
                description: "Different phrasing for 'disk usage by folder'".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
            EvalCase {
                id: Some("var_005".to_string()),
                input: "python files from the last 7 days".to_string(),
                expected_output: Some("find . -name \"*.py\" -type f -mtime -7".to_string()),
                expected_outputs: vec!["find . -name \"*.py\" -type f -mtime -7".to_string()],
                category: EvalCategory::Variant,
                description: "Different phrasing for 'python files modified last week'".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
        ]);

        // Edge Cases (P2 - document behavior)
        cases.extend(vec![
            EvalCase {
                id: Some("edge_001".to_string()),
                input: "files modified yesterday".to_string(),
                expected_output: Some("find . -type f -mtime 1".to_string()),
                expected_outputs: vec!["find . -type f -mtime 1".to_string()],
                category: EvalCategory::EdgeCase,
                description: "Similar to 'today' but different time offset".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
            EvalCase {
                id: Some("edge_002".to_string()),
                input: "large javascript files over 50MB".to_string(),
                expected_output: Some("find . -name \"*.js\" -type f -size +50M".to_string()),
                expected_outputs: vec!["find . -name \"*.js\" -type f -size +50M".to_string()],
                category: EvalCategory::EdgeCase,
                description: "Combination: file type + size filter".to_string(),
                primary_profile: None,
                secondary_profiles: None,
                source: None,
                risk_level: Some("safe".to_string()),
            },
        ]);

        cases
    }

    /// Load test suite from YAML file
    pub fn from_yaml(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

        // Extract metadata
        let metadata = yaml
            .get("metadata")
            .ok_or("Missing 'metadata' field in YAML")?;
        let name = metadata
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("YAML Test Suite")
            .to_string();
        let description = metadata
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("Test cases loaded from YAML")
            .to_string();

        // Extract test cases
        let test_cases_yaml = yaml
            .get("test_cases")
            .ok_or("Missing 'test_cases' field in YAML")?;

        let mut test_cases: Vec<EvalCase> = serde_yaml::from_value(test_cases_yaml.clone())?;

        // Post-process: if expected_output is set but expected_outputs is empty,
        // copy expected_output to expected_outputs
        for case in &mut test_cases {
            if case.expected_outputs.is_empty() {
                if let Some(ref output) = case.expected_output {
                    case.expected_outputs = vec![output.clone()];
                }
            }
        }

        Ok(Self {
            name,
            description,
            test_cases,
        })
    }

    /// Filter test cases by profile ID
    pub fn filter_by_profile(mut self, profile_id: &str) -> Self {
        self.test_cases = self
            .test_cases
            .into_iter()
            .filter(|case| {
                // Match primary profile
                if let Some(ref primary) = case.primary_profile {
                    if primary == profile_id {
                        return true;
                    }
                }

                // Match secondary profiles
                if let Some(ref secondaries) = case.secondary_profiles {
                    if secondaries.iter().any(|p| p == profile_id) {
                        return true;
                    }
                }

                false
            })
            .collect();

        self
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

        let categories: Vec<_> = suite.test_cases.iter().map(|c| c.category).collect();

        assert!(categories.contains(&EvalCategory::WebsiteClaim));
        assert!(categories.contains(&EvalCategory::Variant));
        assert!(categories.contains(&EvalCategory::EdgeCase));
    }
}
