//! Test case quality metrics and analysis.
//!
//! This module provides difficulty scoring, redundancy detection,
//! coverage gap analysis, and quality reporting.

use crate::dataset::TestCase;
use crate::test_runner::TestResult;
use serde::{Deserialize, Serialize};

/// Test difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    /// Easy: 80%+ models pass
    Easy,
    /// Medium: 40-80% models pass
    Medium,
    /// Hard: <40% models pass
    Hard,
}

/// Difficulty distribution statistics
#[derive(Debug, Clone)]
pub struct DifficultyDistribution {
    pub easy_count: usize,
    pub medium_count: usize,
    pub hard_count: usize,
}

/// Difficulty analyzer
pub struct DifficultyAnalyzer;

impl DifficultyAnalyzer {
    /// Create a new difficulty analyzer
    pub fn new() -> Self {
        Self
    }

    /// Calculate difficulty level for a test based on results
    pub fn calculate_difficulty(&self, results: &[TestResult]) -> DifficultyLevel {
        if results.is_empty() {
            return DifficultyLevel::Medium;
        }

        let pass_count = results.iter().filter(|r| r.passed).count();
        let pass_rate = pass_count as f64 / results.len() as f64;

        if pass_rate > 0.8 {
            DifficultyLevel::Easy
        } else if pass_rate > 0.4 {
            DifficultyLevel::Medium
        } else {
            DifficultyLevel::Hard
        }
    }

    /// Calculate difficulty distribution
    pub fn calculate_distribution(&self, levels: Vec<DifficultyLevel>) -> DifficultyDistribution {
        let easy_count = levels
            .iter()
            .filter(|l| **l == DifficultyLevel::Easy)
            .count();
        let medium_count = levels
            .iter()
            .filter(|l| **l == DifficultyLevel::Medium)
            .count();
        let hard_count = levels
            .iter()
            .filter(|l| **l == DifficultyLevel::Hard)
            .count();

        DifficultyDistribution {
            easy_count,
            medium_count,
            hard_count,
        }
    }
}

impl Default for DifficultyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Redundancy detector using similarity analysis
pub struct RedundancyDetector {
    similarity_threshold: f64,
}

impl RedundancyDetector {
    /// Create a new redundancy detector
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.95,
        }
    }

    /// Find redundant test case pairs
    pub fn find_redundant_pairs(&self, test_cases: &[TestCase]) -> Vec<(String, String)> {
        let mut redundant_pairs = Vec::new();

        for i in 0..test_cases.len() {
            for j in (i + 1)..test_cases.len() {
                let similarity = self.calculate_similarity(&test_cases[i], &test_cases[j]);

                if similarity >= self.similarity_threshold {
                    redundant_pairs.push((test_cases[i].id.clone(), test_cases[j].id.clone()));
                }
            }
        }

        redundant_pairs
    }

    /// Calculate similarity between two test cases
    fn calculate_similarity(&self, a: &TestCase, b: &TestCase) -> f64 {
        // Simplified similarity using Levenshtein-like approach
        // In production, use sentence transformers or embeddings

        // If expected commands are identical, high similarity
        if a.expected_command == b.expected_command {
            return 1.0;
        }

        // Calculate word overlap in prompts
        let words_a: std::collections::HashSet<&str> = a.prompt.split_whitespace().collect();
        let words_b: std::collections::HashSet<&str> = b.prompt.split_whitespace().collect();

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f64 / union as f64
    }
}

impl Default for RedundancyDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Coverage gap information
#[derive(Debug, Clone)]
pub struct CoverageGap {
    /// Command type with gap
    pub command_type: String,
    /// Current number of test cases
    pub current_count: usize,
    /// Recommended minimum
    pub recommended_count: usize,
}

/// Coverage analyzer
pub struct CoverageAnalyzer {
    min_tests_per_command: usize,
}

impl CoverageAnalyzer {
    /// Create a new coverage analyzer
    pub fn new() -> Self {
        Self {
            min_tests_per_command: 3,
        }
    }

    /// Analyze coverage gaps
    pub fn analyze_gaps(&self, test_cases: &[TestCase]) -> Vec<CoverageGap> {
        let mut command_counts = std::collections::HashMap::new();

        // Count test cases per command type
        for test in test_cases {
            let command_type = self.extract_command_type(&test.expected_command);
            *command_counts.entry(command_type).or_insert(0) += 1;
        }

        // Identify gaps
        let mut gaps = Vec::new();
        for (command_type, count) in command_counts {
            if count < self.min_tests_per_command {
                gaps.push(CoverageGap {
                    command_type,
                    current_count: count,
                    recommended_count: self.min_tests_per_command,
                });
            }
        }

        gaps
    }

    /// Extract command type from command string
    fn extract_command_type(&self, command: &str) -> String {
        // Extract first word as command type
        command
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string()
    }
}

impl Default for CoverageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Quality metrics aggregator
pub struct QualityMetrics {
    difficulty_analyzer: DifficultyAnalyzer,
}

impl QualityMetrics {
    /// Create a new quality metrics analyzer
    pub fn new() -> Self {
        Self {
            difficulty_analyzer: DifficultyAnalyzer::new(),
        }
    }

    /// Generate a comprehensive quality report
    pub fn generate_report(&self, test_cases: &[TestCase], results: &[TestResult]) -> String {
        let mut report = String::new();

        report.push_str("# Test Suite Quality Report\n\n");

        // Group results by test ID
        let mut test_results_map = std::collections::HashMap::new();
        for result in results {
            test_results_map
                .entry(result.test_id.clone())
                .or_insert_with(Vec::new)
                .push(result.clone());
        }

        // Calculate difficulty for each test
        let mut difficulties = Vec::new();
        for test in test_cases {
            if let Some(test_results) = test_results_map.get(&test.id) {
                let difficulty = self.difficulty_analyzer.calculate_difficulty(test_results);
                difficulties.push((test.id.clone(), difficulty));
            }
        }

        // Difficulty Distribution section
        report.push_str("## Difficulty Distribution\n\n");
        let levels: Vec<DifficultyLevel> = difficulties.iter().map(|(_, d)| *d).collect();
        let distribution = self.difficulty_analyzer.calculate_distribution(levels);

        report.push_str(&format!("- Easy: {}\n", distribution.easy_count));
        report.push_str(&format!("- Medium: {}\n", distribution.medium_count));
        report.push_str(&format!("- Hard: {}\n\n", distribution.hard_count));

        // Individual test difficulty
        report.push_str("## Test Difficulty Breakdown\n\n");
        for (test_id, difficulty) in &difficulties {
            let difficulty_str = match difficulty {
                DifficultyLevel::Easy => "Easy",
                DifficultyLevel::Medium => "Medium",
                DifficultyLevel::Hard => "Hard",
            };
            report.push_str(&format!("- {}: {}\n", test_id, difficulty_str));
        }

        report
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}
