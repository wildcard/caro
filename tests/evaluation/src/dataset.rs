//! Test dataset types and loading.
//!
//! This module provides data structures for representing test cases and datasets
//! used in LLM evaluation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during dataset operations
#[derive(Debug, Error)]
pub enum DatasetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// A single test case representing an evaluation scenario
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCase {
    pub id: String,
    pub prompt: String,
    pub expected_command: String,
    pub category: String,
    pub risk_level: String,
    pub posix_compliant: bool,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TestCaseMetadata>,
}

impl TestCase {
    /// Validate the test case data
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Test case ID cannot be empty".to_string());
        }

        if self.prompt.is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }

        if self.expected_command.is_empty() {
            return Err("Expected command cannot be empty".to_string());
        }

        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }

        // Validate risk_level is one of the allowed values
        let valid_risk_levels = ["safe", "low", "medium", "high", "critical"];
        if !valid_risk_levels.contains(&self.risk_level.as_str()) {
            return Err(format!(
                "Invalid risk_level '{}'. Must be one of: {}",
                self.risk_level,
                valid_risk_levels.join(", ")
            ));
        }

        Ok(())
    }
}

/// Optional metadata for test cases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCaseMetadata {
    /// Rules for determining semantic equivalence
    #[serde(default)]
    pub equivalence_rules: Vec<String>,

    /// Required external tools (e.g., "git", "docker")
    #[serde(default)]
    pub required_tools: Vec<String>,

    /// Minimum POSIX version required
    pub min_posix_version: Option<String>,

    /// Additional notes about the test case
    pub notes: Option<String>,
}

/// A collection of related test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDataset {
    pub name: String,
    pub version: String,
    pub description: String,
    pub test_cases: Vec<TestCase>,
    pub created_at: DateTime<Utc>,
    pub metadata: DatasetMetadata,
}

impl TestDataset {
    /// Load a test dataset from a JSON file
    pub fn load_from_file(path: &Path) -> Result<Self, DatasetError> {
        let content = std::fs::read_to_string(path)?;
        let dataset: TestDataset = serde_json::from_str(&content)?;
        dataset.validate().map_err(DatasetError::Validation)?;
        Ok(dataset)
    }

    /// Validate the dataset
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Dataset name cannot be empty".to_string());
        }

        if self.version.is_empty() {
            return Err("Dataset version cannot be empty".to_string());
        }

        // Verify all test case IDs are unique
        let mut ids = HashSet::new();
        for tc in &self.test_cases {
            if !ids.insert(&tc.id) {
                return Err(format!("Duplicate test case ID: {}", tc.id));
            }
            tc.validate()?;
        }

        // Validate metadata counts match reality
        if self.metadata.total_cases != self.test_cases.len() {
            return Err(format!(
                "Metadata total_cases ({}) doesn't match actual test_cases count ({})",
                self.metadata.total_cases,
                self.test_cases.len()
            ));
        }

        Ok(())
    }
}

/// Metadata about the dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// Author or team that created the dataset
    pub author: String,

    /// Total number of test cases
    pub total_cases: usize,

    /// Distribution of test cases by category
    pub categories: HashMap<String, usize>,

    /// Distribution of test cases by risk level
    pub risk_distribution: HashMap<String, usize>,

    /// Percentage of test cases that are POSIX compliant (0.0-1.0)
    pub posix_coverage: f64,

    /// Source of the test cases (e.g., "manual_curation", "generated")
    pub source: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_dataset() {
        let json = r#"{
            "name": "test_dataset",
            "version": "1.0.0",
            "description": "Test",
            "test_cases": [{
                "id": "test_001",
                "prompt": "test prompt",
                "expected_command": "echo test",
                "category": "test",
                "risk_level": "safe",
                "posix_compliant": true,
                "tags": []
            }],
            "created_at": "2026-01-08T00:00:00Z",
            "metadata": {
                "author": "test",
                "total_cases": 1,
                "categories": {"test": 1},
                "risk_distribution": {"safe": 1},
                "posix_coverage": 1.0
            }
        }"#;

        let dataset: TestDataset = serde_json::from_str(json).unwrap();
        assert_eq!(dataset.name, "test_dataset");
        assert_eq!(dataset.test_cases.len(), 1);
        dataset.validate().unwrap();
    }

    #[test]
    fn test_reject_duplicate_ids() {
        let json = r#"{
            "name": "test_dataset",
            "version": "1.0.0",
            "description": "Test",
            "test_cases": [
                {
                    "id": "test_001",
                    "prompt": "first prompt",
                    "expected_command": "echo first",
                    "category": "test",
                    "risk_level": "safe",
                    "posix_compliant": true,
                    "tags": []
                },
                {
                    "id": "test_001",
                    "prompt": "second prompt",
                    "expected_command": "echo second",
                    "category": "test",
                    "risk_level": "safe",
                    "posix_compliant": true,
                    "tags": []
                }
            ],
            "created_at": "2026-01-08T00:00:00Z",
            "metadata": {
                "author": "test",
                "total_cases": 2,
                "categories": {"test": 2},
                "risk_distribution": {"safe": 2},
                "posix_coverage": 1.0
            }
        }"#;

        let dataset: TestDataset = serde_json::from_str(json).unwrap();
        let result = dataset.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate test case ID"));
    }

    #[test]
    fn test_reject_empty_prompt() {
        let json = r#"{
            "name": "test_dataset",
            "version": "1.0.0",
            "description": "Test",
            "test_cases": [{
                "id": "test_001",
                "prompt": "",
                "expected_command": "echo test",
                "category": "test",
                "risk_level": "safe",
                "posix_compliant": true,
                "tags": []
            }],
            "created_at": "2026-01-08T00:00:00Z",
            "metadata": {
                "author": "test",
                "total_cases": 1,
                "categories": {"test": 1},
                "risk_distribution": {"safe": 1},
                "posix_coverage": 1.0
            }
        }"#;

        let dataset: TestDataset = serde_json::from_str(json).unwrap();
        let result = dataset.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Prompt cannot be empty"));
    }

    #[test]
    fn test_reject_invalid_risk_level() {
        let json = r#"{
            "name": "test_dataset",
            "version": "1.0.0",
            "description": "Test",
            "test_cases": [{
                "id": "test_001",
                "prompt": "test prompt",
                "expected_command": "echo test",
                "category": "test",
                "risk_level": "invalid_level",
                "posix_compliant": true,
                "tags": []
            }],
            "created_at": "2026-01-08T00:00:00Z",
            "metadata": {
                "author": "test",
                "total_cases": 1,
                "categories": {"test": 1},
                "risk_distribution": {"invalid_level": 1},
                "posix_coverage": 1.0
            }
        }"#;

        let dataset: TestDataset = serde_json::from_str(json).unwrap();
        let result = dataset.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid risk_level"));
    }

    #[test]
    fn test_reject_mismatched_total_cases() {
        let json = r#"{
            "name": "test_dataset",
            "version": "1.0.0",
            "description": "Test",
            "test_cases": [{
                "id": "test_001",
                "prompt": "test prompt",
                "expected_command": "echo test",
                "category": "test",
                "risk_level": "safe",
                "posix_compliant": true,
                "tags": []
            }],
            "created_at": "2026-01-08T00:00:00Z",
            "metadata": {
                "author": "test",
                "total_cases": 5,
                "categories": {"test": 1},
                "risk_distribution": {"safe": 1},
                "posix_coverage": 1.0
            }
        }"#;

        let dataset: TestDataset = serde_json::from_str(json).unwrap();
        let result = dataset.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("total_cases"));
    }
}
