//! Automated test case generation.
//!
//! This module generates test cases from telemetry data, fuzzes edge cases,
//! and augments the dataset automatically.

use crate::dataset::TestCase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Telemetry query from user interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryQuery {
    /// User's natural language query
    pub query: String,
    /// Generated command
    pub generated_command: String,
    /// Whether user executed the command
    pub user_executed: bool,
    /// When this query was made
    pub timestamp: DateTime<Utc>,
}

/// Test case generator from telemetry
pub struct TestGenerator;

impl TestGenerator {
    /// Create a new test generator
    pub fn new() -> Self {
        Self
    }

    /// Generate a test case from telemetry
    pub fn generate_from_telemetry(&self, telemetry: &TelemetryQuery) -> Result<TestCase, String> {
        // Hash the query to create stable test ID
        let hash = self.hash_query(&telemetry.query);
        let category = self.categorize_query(&telemetry.query);

        Ok(TestCase {
            id: format!("telemetry_{}", hash),
            prompt: telemetry.query.clone(),
            expected_command: telemetry.generated_command.clone(),
            category,
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        })
    }

    /// Categorize a query into test category
    pub fn categorize_query(&self, query: &str) -> String {
        let query_lower = query.to_lowercase();

        // Safety keywords
        if query_lower.contains("delete")
            || query_lower.contains("remove")
            || query_lower.contains("rm ")
            || query_lower.contains("format")
        {
            return "safety".to_string();
        }

        // Default to correctness
        "correctness".to_string()
    }

    /// Hash a query string for stable IDs
    fn hash_query(&self, query: &str) -> String {
        // Simple hash for demo - in production use proper hash function
        let hash: u32 = query
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        format!("{:08x}", hash)
    }

    /// Generate multiple test cases from a batch
    pub fn batch_generate(&self, telemetry_batch: &[TelemetryQuery]) -> Vec<TestCase> {
        telemetry_batch
            .iter()
            .filter_map(|t| self.generate_from_telemetry(t).ok())
            .collect()
    }

    /// Generate test cases with deduplication
    pub fn batch_generate_deduplicated(&self, telemetry_batch: &[TelemetryQuery]) -> Vec<TestCase> {
        let mut seen_queries = HashSet::new();
        let mut test_cases = Vec::new();

        for telemetry in telemetry_batch {
            let normalized = telemetry.query.to_lowercase().trim().to_string();

            if !seen_queries.contains(&normalized) {
                if let Ok(test_case) = self.generate_from_telemetry(telemetry) {
                    test_cases.push(test_case);
                    seen_queries.insert(normalized);
                }
            }
        }

        test_cases
    }
}

impl Default for TestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Edge case fuzzer for generating challenging test scenarios
pub struct EdgeCaseFuzzer {
    templates: Vec<&'static str>,
}

impl EdgeCaseFuzzer {
    /// Create a new edge case fuzzer
    pub fn new() -> Self {
        Self {
            templates: vec![
                "find files with spaces in name",
                "search for pattern with special regex characters like $ ^ * [ ]",
                "list files in directory with unicode name 日本語",
                "grep for string with escaped quotes and backslashes",
                "find files with glob patterns containing * and ?",
                "search for literal dot . in filename",
                "list hidden files starting with dot",
                "find files modified in the last hour",
                "search for case-insensitive pattern ABC",
                "list files larger than 1GB",
            ],
        }
    }

    /// Generate N edge case test scenarios
    pub fn generate_edge_cases(&self, count: usize) -> Vec<TestCase> {
        self.templates
            .iter()
            .take(count)
            .enumerate()
            .map(|(i, template)| TestCase {
                id: format!("edge_{:03}", i + 1),
                prompt: template.to_string(),
                expected_command: self.generate_edge_command(template),
                category: "correctness".to_string(),
                risk_level: "low".to_string(),
                posix_compliant: true,
                tags: vec!["edge-case".to_string()],
                metadata: None,
            })
            .collect()
    }

    /// Generate an expected command for an edge case
    fn generate_edge_command(&self, template: &str) -> String {
        // Simplified - in production this would use the actual backend
        if template.contains("spaces") {
            "find . -name '* *'".to_string()
        } else if template.contains("unicode") {
            "ls 日本語/".to_string()
        } else if template.contains("hidden") {
            "ls -d .*".to_string()
        } else if template.contains("hour") {
            "find . -mmin -60".to_string()
        } else if template.contains("1GB") {
            "find . -size +1G".to_string()
        } else {
            "echo 'edge case'".to_string()
        }
    }
}

impl Default for EdgeCaseFuzzer {
    fn default() -> Self {
        Self::new()
    }
}
