//! Pattern extraction from test failures.
//!
//! This module analyzes test failures to extract common patterns,
//! identify safety gaps, and suggest new test cases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::test_runner::TestResult;

/// A common failure pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub description: String,
    pub count: usize,
    pub examples: Vec<String>,
    pub suggested_fix: String,
}

/// Pattern extractor
pub struct PatternExtractor {
    min_pattern_count: usize,
}

impl PatternExtractor {
    /// Create a new pattern extractor
    pub fn new() -> Self {
        Self {
            min_pattern_count: 1, // Include all patterns (even single occurrences)
        }
    }

    /// Extract common failure patterns from test results
    pub fn extract_patterns(&self, failures: &[TestResult]) -> Vec<FailurePattern> {
        let mut pattern_map: HashMap<String, Vec<String>> = HashMap::new();

        for failure in failures {
            if let Some(reason) = &failure.failure_reason {
                pattern_map
                    .entry(reason.clone())
                    .or_default()
                    .push(failure.actual_output.clone());
            }
        }

        let mut patterns: Vec<FailurePattern> = pattern_map
            .into_iter()
            .filter(|(_, examples)| examples.len() >= self.min_pattern_count)
            .map(|(description, examples)| {
                let suggested_fix = self.suggest_fix(&description);
                FailurePattern {
                    count: examples.len(),
                    description,
                    examples: examples.into_iter().take(3).collect(), // Keep first 3 examples
                    suggested_fix,
                }
            })
            .collect();

        // Sort by count (most common first)
        patterns.sort_by(|a, b| b.count.cmp(&a.count));
        patterns
    }

    /// Suggest a fix based on failure description
    fn suggest_fix(&self, description: &str) -> String {
        if description.contains("-mtime") {
            "Add POSIX-compliant examples to prompt".to_string()
        } else if description.contains("-n") || description.contains("line number") {
            "Include -n flag in grep examples".to_string()
        } else if description.contains("flag") || description.contains("Missing") {
            "Add more examples showing common flags".to_string()
        } else {
            "Review prompt for this command pattern".to_string()
        }
    }

    /// Detect safety pattern gaps from blocked commands
    pub fn detect_safety_gaps(
        &self,
        blocked_commands: &[String],
        existing_patterns: &[&str],
    ) -> Vec<String> {
        let mut gaps = Vec::new();

        for command in blocked_commands {
            let has_pattern = existing_patterns
                .iter()
                .any(|pattern| command.contains(pattern));

            if !has_pattern {
                // Extract the main command word
                if let Some(cmd_word) = command.split_whitespace().next() {
                    if !gaps.contains(&cmd_word.to_string()) {
                        gaps.push(format!(
                            "Missing pattern for '{}' (example: {})",
                            cmd_word, command
                        ));
                    }
                }
            }
        }

        gaps
    }

    /// Suggest new test cases based on failures
    pub fn suggest_test_cases(&self, failures: &[TestResult]) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Group failures by command type
        let mut command_types: HashMap<String, usize> = HashMap::new();

        for failure in failures {
            if let Some(first_word) = failure.expected_output.split_whitespace().next() {
                *command_types.entry(first_word.to_string()).or_insert(0) += 1;
            }
        }

        // Suggest test cases for failing command types
        for (cmd, count) in command_types {
            if count >= 1 {
                suggestions.push(format!(
                    "Add more {} test cases (currently failing {} tests)",
                    cmd, count
                ));
            }
        }

        suggestions
    }

    /// Generate insights report
    pub fn generate_insights_report(
        &self,
        patterns: &[FailurePattern],
        safety_gaps: &[String],
        test_suggestions: &[String],
    ) -> String {
        let mut report = String::new();

        report.push_str("# Evaluation Insights\n\n");

        // Common failure modes
        report.push_str("## Common Failure Modes\n\n");
        if patterns.is_empty() {
            report.push_str("No common patterns detected.\n\n");
        } else {
            for (i, pattern) in patterns.iter().enumerate() {
                report.push_str(&format!(
                    "{}. **{}** ({} failures)\n",
                    i + 1,
                    pattern.description,
                    pattern.count
                ));
                if !pattern.examples.is_empty() {
                    report.push_str(&format!("   - Example: `{}`\n", pattern.examples[0]));
                }
                report.push_str(&format!(
                    "   - Suggested fix: {}\n\n",
                    pattern.suggested_fix
                ));
            }
        }

        // Safety pattern gaps
        report.push_str("## Safety Pattern Gaps\n\n");
        if safety_gaps.is_empty() {
            report.push_str("No safety gaps detected.\n\n");
        } else {
            for gap in safety_gaps {
                report.push_str(&format!("- {}\n", gap));
            }
            report.push('\n');
        }

        // Test suggestions
        report.push_str("## Suggested Test Cases\n\n");
        if test_suggestions.is_empty() {
            report.push_str("No new test cases suggested.\n\n");
        } else {
            for suggestion in test_suggestions {
                report.push_str(&format!("- {}\n", suggestion));
            }
            report.push('\n');
        }

        report
    }

    /// Group failures by category
    pub fn group_by_category(&self, failures: &[TestResult]) -> HashMap<String, Vec<TestResult>> {
        let mut grouped: HashMap<String, Vec<TestResult>> = HashMap::new();

        for failure in failures {
            grouped
                .entry(failure.category.clone())
                .or_default()
                .push(failure.clone());
        }

        grouped
    }
}

impl Default for PatternExtractor {
    fn default() -> Self {
        Self::new()
    }
}
