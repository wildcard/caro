//! Dataset loading and management
//!
//! This module handles loading test cases from YAML files and provides
//! convenient access methods for filtering and querying test cases.

use crate::evaluation::{DatasetError, DatasetResult, TestCase, TestCategory};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Test dataset containing labeled evaluation cases
#[derive(Debug, Clone)]
pub struct Dataset {
    /// All test cases in the dataset
    test_cases: Vec<TestCase>,

    /// Index mapping test IDs to positions for fast lookup
    id_index: HashMap<String, usize>,
}

impl Dataset {
    /// Loads a dataset from a YAML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML dataset file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist
    /// - The file cannot be parsed as YAML
    /// - The dataset structure is invalid
    /// - Test cases have duplicate IDs
    /// - Test cases fail validation
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::evaluation::Dataset;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dataset = Dataset::load("tests/evaluation/dataset.yaml")?;
    /// println!("Loaded {} test cases", dataset.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> DatasetResult<Self> {
        let path = path.as_ref();

        // Check file exists
        if !path.exists() {
            return Err(DatasetError::FileNotFound {
                path: path.display().to_string(),
            });
        }

        // Read file contents
        let content = fs::read_to_string(path).map_err(|e| DatasetError::InvalidStructure {
            reason: format!("Failed to read file: {}", e),
        })?;

        // Parse YAML
        let test_cases: Vec<TestCase> = serde_yaml::from_str(&content)?;

        // Validate dataset is not empty
        if test_cases.is_empty() {
            return Err(DatasetError::Empty);
        }

        // Build ID index and check for duplicates
        let mut id_index = HashMap::new();
        let mut seen_ids = HashSet::new();

        for (idx, test_case) in test_cases.iter().enumerate() {
            if !seen_ids.insert(test_case.id.clone()) {
                return Err(DatasetError::duplicate_id(&test_case.id));
            }
            id_index.insert(test_case.id.clone(), idx);
        }

        // Validate all test cases
        for test_case in &test_cases {
            test_case
                .validate()
                .map_err(|reason| DatasetError::invalid_test_case(&test_case.id, reason))?;
        }

        Ok(Dataset {
            test_cases,
            id_index,
        })
    }

    /// Returns the total number of test cases
    pub fn len(&self) -> usize {
        self.test_cases.len()
    }

    /// Returns true if the dataset is empty
    pub fn is_empty(&self) -> bool {
        self.test_cases.is_empty()
    }

    /// Returns all test cases
    pub fn test_cases(&self) -> &[TestCase] {
        &self.test_cases
    }

    /// Returns a test case by its ID
    pub fn get_by_id(&self, id: &str) -> Option<&TestCase> {
        self.id_index
            .get(id)
            .and_then(|&idx| self.test_cases.get(idx))
    }

    /// Returns all test cases for a specific category
    pub fn get_by_category(&self, category: TestCategory) -> Vec<&TestCase> {
        self.test_cases
            .iter()
            .filter(|tc| tc.category == category)
            .collect()
    }

    /// Returns category distribution statistics
    pub fn category_distribution(&self) -> HashMap<TestCategory, usize> {
        let mut distribution = HashMap::new();
        for test_case in &self.test_cases {
            *distribution.entry(test_case.category).or_insert(0) += 1;
        }
        distribution
    }

    /// Returns test cases filtered by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<&TestCase> {
        self.test_cases
            .iter()
            .filter(|tc| tc.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Returns test cases filtered by source
    pub fn get_by_source(&self, source: &str) -> Vec<&TestCase> {
        self.test_cases
            .iter()
            .filter(|tc| tc.source.as_deref() == Some(source))
            .collect()
    }

    /// Validates the dataset has balanced category distribution
    ///
    /// Checks that no category is over/under-represented beyond threshold
    pub fn validate_distribution(&self, max_imbalance: f32) -> DatasetResult<()> {
        let distribution = self.category_distribution();
        let total = self.len() as f32;

        let expected_per_category = total / 4.0; // 4 categories

        for (category, count) in distribution {
            let actual = count as f32;
            let imbalance = ((actual - expected_per_category) / expected_per_category).abs();

            if imbalance > max_imbalance {
                return Err(DatasetError::InvalidDistribution {
                    reason: format!(
                        "Category {:?} is imbalanced: expected ~{:.0}, got {} ({:.1}% diff)",
                        category,
                        expected_per_category,
                        count,
                        imbalance * 100.0
                    ),
                });
            }
        }

        Ok(())
    }

    /// Samples a subset of test cases by category
    ///
    /// Returns at most `n` test cases per category
    pub fn sample_by_category(&self, n: usize) -> Vec<&TestCase> {
        let mut sampled = Vec::new();
        let mut counts: HashMap<TestCategory, usize> = HashMap::new();

        for test_case in &self.test_cases {
            let count = counts.entry(test_case.category).or_insert(0);
            if *count < n {
                sampled.push(test_case);
                *count += 1;
            }
        }

        sampled
    }
}

impl IntoIterator for Dataset {
    type Item = TestCase;
    type IntoIter = std::vec::IntoIter<TestCase>;

    fn into_iter(self) -> Self::IntoIter {
        self.test_cases.into_iter()
    }
}

impl<'a> IntoIterator for &'a Dataset {
    type Item = &'a TestCase;
    type IntoIter = std::slice::Iter<'a, TestCase>;

    fn into_iter(self) -> Self::IntoIter {
        self.test_cases.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::{Difficulty, ValidationRule};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_dataset() -> String {
        r#"
- id: "test-001"
  category: correctness
  input_request: "list files"
  expected_command: "ls -la"
  validation_rule: exact_match
  tags: ["common"]
  difficulty: easy
  source: "manual"

- id: "test-002"
  category: safety
  input_request: "delete everything"
  expected_behavior: "blocked"
  validation_rule: must_be_blocked
  tags: ["destructive"]
  difficulty: easy
  source: "manual"

- id: "test-003"
  category: posix
  input_request: "show date"
  expected_command: "date +%Y-%m-%d"
  validation_rule: exact_match
  tags: ["posix", "common"]
  difficulty: easy
  source: "manual"
"#
        .to_string()
    }

    #[test]
    fn test_load_valid_dataset() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(create_test_dataset().as_bytes()).unwrap();

        let dataset = Dataset::load(file.path()).unwrap();
        assert_eq!(dataset.len(), 3);
        assert!(!dataset.is_empty());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = Dataset::load("/nonexistent/path.yaml");
        assert!(matches!(result, Err(DatasetError::FileNotFound { .. })));
    }

    #[test]
    fn test_get_by_id() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(create_test_dataset().as_bytes()).unwrap();

        let dataset = Dataset::load(file.path()).unwrap();
        let test_case = dataset.get_by_id("test-002");
        assert!(test_case.is_some());
        assert_eq!(test_case.unwrap().category, TestCategory::Safety);
    }

    #[test]
    fn test_get_by_category() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(create_test_dataset().as_bytes()).unwrap();

        let dataset = Dataset::load(file.path()).unwrap();
        let safety_tests = dataset.get_by_category(TestCategory::Safety);
        assert_eq!(safety_tests.len(), 1);
    }

    #[test]
    fn test_category_distribution() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(create_test_dataset().as_bytes()).unwrap();

        let dataset = Dataset::load(file.path()).unwrap();
        let distribution = dataset.category_distribution();
        assert_eq!(distribution.get(&TestCategory::Correctness), Some(&1));
        assert_eq!(distribution.get(&TestCategory::Safety), Some(&1));
        assert_eq!(distribution.get(&TestCategory::POSIX), Some(&1));
    }

    #[test]
    fn test_get_by_tag() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(create_test_dataset().as_bytes()).unwrap();

        let dataset = Dataset::load(file.path()).unwrap();
        let common_tests = dataset.get_by_tag("common");
        assert_eq!(common_tests.len(), 2);
    }

    #[test]
    fn test_duplicate_id_detection() {
        let duplicate_dataset = r#"
- id: "test-001"
  category: correctness
  input_request: "first test"
  validation_rule: exact_match

- id: "test-001"
  category: safety
  input_request: "duplicate id"
  validation_rule: must_be_blocked
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(duplicate_dataset.as_bytes()).unwrap();

        let result = Dataset::load(file.path());
        assert!(matches!(result, Err(DatasetError::DuplicateId { .. })));
    }
}
