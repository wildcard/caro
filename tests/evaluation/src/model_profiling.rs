//! Per-model analysis and profiling.
//!
//! This module provides model-specific analysis including:
//! - Failure pattern extraction
//! - Category performance tracking
//! - Strengths/weaknesses identification
//! - Model recommendation system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::test_runner::TestResult;

/// Performance metrics for a specific category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryPerformance {
    pub category: String,
    pub passed: usize,
    pub total: usize,
    pub pass_rate: f64,
}

/// Complete profile for a single model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub model_name: String,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub overall_pass_rate: f64,
    pub category_performance: Vec<CategoryPerformance>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

/// Model profiler for analyzing test results
pub struct ModelProfiler {
    strength_threshold: f64,
    weakness_threshold: f64,
}

impl ModelProfiler {
    /// Create a new profiler with default thresholds
    pub fn new() -> Self {
        Self {
            strength_threshold: 0.75, // >= 75% is a strength
            weakness_threshold: 0.40, // < 40% is a weakness
        }
    }

    /// Build a model profile from test results
    pub fn build_profile(&self, model_name: &str, results: &[TestResult]) -> ModelProfile {
        let model_results: Vec<_> = results.iter().filter(|r| r.backend == model_name).collect();

        let total_tests = model_results.len();
        let passed_tests = model_results.iter().filter(|r| r.passed).count();
        let overall_pass_rate = if total_tests > 0 {
            passed_tests as f64 / total_tests as f64
        } else {
            0.0
        };

        // Calculate per-category performance
        let mut category_stats: HashMap<String, (usize, usize)> = HashMap::new();

        for result in &model_results {
            let entry = category_stats
                .entry(result.category.clone())
                .or_insert((0, 0));
            entry.1 += 1; // total
            if result.passed {
                entry.0 += 1; // passed
            }
        }

        let mut category_performance: Vec<CategoryPerformance> = category_stats
            .into_iter()
            .map(|(category, (passed, total))| CategoryPerformance {
                pass_rate: passed as f64 / total as f64,
                category,
                passed,
                total,
            })
            .collect();

        category_performance.sort_by(|a, b| {
            b.pass_rate
                .partial_cmp(&a.pass_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Identify strengths and weaknesses
        let strengths: Vec<String> = category_performance
            .iter()
            .filter(|cp| cp.pass_rate >= self.strength_threshold)
            .map(|cp| cp.category.clone())
            .collect();

        let weaknesses: Vec<String> = category_performance
            .iter()
            .filter(|cp| cp.pass_rate < self.weakness_threshold)
            .map(|cp| cp.category.clone())
            .collect();

        ModelProfile {
            model_name: model_name.to_string(),
            total_tests,
            passed_tests,
            overall_pass_rate,
            category_performance,
            strengths,
            weaknesses,
        }
    }

    /// Extract common failure patterns from test results
    pub fn extract_failure_patterns(&self, results: &[TestResult]) -> Vec<String> {
        let mut pattern_counts: HashMap<String, usize> = HashMap::new();

        for result in results {
            if !result.passed {
                if let Some(reason) = &result.failure_reason {
                    *pattern_counts.entry(reason.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut patterns: Vec<(String, usize)> = pattern_counts.into_iter().collect();
        patterns.sort_by(|a, b| b.1.cmp(&a.1));

        patterns.into_iter().map(|(pattern, _)| pattern).collect()
    }

    /// Generate a markdown report for a model profile
    pub fn generate_report(&self, profile: &ModelProfile) -> String {
        let mut report = String::new();

        report.push_str(&format!("# {} Profile\n\n", profile.model_name));

        report.push_str(&format!(
            "**Overall Performance**: {:.1}% ({}/{})\n\n",
            profile.overall_pass_rate * 100.0,
            profile.passed_tests,
            profile.total_tests
        ));

        // Strengths section
        report.push_str("## Strengths\n\n");
        if profile.strengths.is_empty() {
            report.push_str("None identified (no category above 75%)\n\n");
        } else {
            for strength in &profile.strengths {
                if let Some(perf) = profile
                    .category_performance
                    .iter()
                    .find(|cp| &cp.category == strength)
                {
                    report.push_str(&format!(
                        "- **{}**: {:.1}% pass rate\n",
                        strength,
                        perf.pass_rate * 100.0
                    ));
                } else {
                    report.push_str(&format!("- **{}**\n", strength));
                }
            }
            report.push('\n');
        }

        // Weaknesses section
        report.push_str("## Weaknesses\n\n");
        if profile.weaknesses.is_empty() {
            report.push_str("None identified (all categories above 40%)\n\n");
        } else {
            for weakness in &profile.weaknesses {
                if let Some(perf) = profile
                    .category_performance
                    .iter()
                    .find(|cp| &cp.category == weakness)
                {
                    report.push_str(&format!(
                        "- **{}**: {:.1}% pass rate\n",
                        weakness,
                        perf.pass_rate * 100.0
                    ));
                } else {
                    report.push_str(&format!("- **{}**\n", weakness));
                }
            }
            report.push('\n');
        }

        // Category breakdown
        report.push_str("## Category Performance\n\n");
        report.push_str("| Category | Pass Rate | Tests |\n");
        report.push_str("|----------|-----------|-------|\n");

        for perf in &profile.category_performance {
            report.push_str(&format!(
                "| {} | {:.1}% | {}/{} |\n",
                perf.category,
                perf.pass_rate * 100.0,
                perf.passed,
                perf.total
            ));
        }

        report
    }

    /// Recommend best model for a given category
    pub fn recommend_model(&self, profiles: &[ModelProfile], category: &str) -> String {
        let mut best_model = "";
        let mut best_rate = 0.0;

        for profile in profiles {
            if let Some(perf) = profile
                .category_performance
                .iter()
                .find(|cp| cp.category == category)
            {
                if perf.pass_rate > best_rate {
                    best_rate = perf.pass_rate;
                    best_model = &profile.model_name;
                }
            }
        }

        best_model.to_string()
    }
}

impl Default for ModelProfiler {
    fn default() -> Self {
        Self::new()
    }
}
