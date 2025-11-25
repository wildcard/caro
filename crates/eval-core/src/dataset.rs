// Dataset loading and management for command accuracy evaluation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;

use crate::types::*;

/// Enhanced test case with runtime validation support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Unique test case identifier
    pub id: String,

    /// Category (e.g., "file_search", "text_processing")
    pub category: String,

    /// Subcategory (e.g., "size_filtering", "pattern_matching")
    pub subcategory: String,

    /// Target shell type
    pub shell: ShellType,

    /// Difficulty level
    pub difficulty: DifficultyLevel,

    /// Natural language input prompt
    pub input: String,

    /// Expected command outputs (multiple valid answers)
    pub expected_commands: Vec<String>,

    /// Human-readable explanation
    pub explanation: String,

    /// Tags for filtering and categorization
    pub tags: Vec<String>,

    /// Safety level of expected commands
    pub safety_level: SafetyLevel,

    /// Optional sandbox configuration for runtime testing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<SandboxConfig>,

    /// Optional assertion configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertions: Option<AssertionConfig>,
}

/// Collection of test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDataset {
    pub test_cases: Vec<TestCase>,
}

impl TestDataset {
    /// Load a test dataset from a YAML file
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let dataset: TestDataset = serde_yaml::from_str(&content)?;
        Ok(dataset)
    }

    /// Load multiple datasets from a directory
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let mut all_test_cases = Vec::new();

        fn load_yaml_files(dir: &Path, test_cases: &mut Vec<TestCase>) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    load_yaml_files(&path, test_cases)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                    || path.extension().and_then(|s| s.to_str()) == Some("yml")
                {
                    let dataset = TestDataset::load_from_yaml(&path)?;
                    test_cases.extend(dataset.test_cases);
                }
            }
            Ok(())
        }

        load_yaml_files(dir.as_ref(), &mut all_test_cases)?;

        Ok(TestDataset {
            test_cases: all_test_cases,
        })
    }

    /// Filter test cases by category
    pub fn filter_by_category(&self, category: &str) -> Self {
        TestDataset {
            test_cases: self
                .test_cases
                .iter()
                .filter(|tc| tc.category == category)
                .cloned()
                .collect(),
        }
    }

    /// Filter test cases by shell type
    pub fn filter_by_shell(&self, shell: ShellType) -> Self {
        TestDataset {
            test_cases: self
                .test_cases
                .iter()
                .filter(|tc| tc.shell == shell)
                .cloned()
                .collect(),
        }
    }

    /// Filter test cases by difficulty
    pub fn filter_by_difficulty(&self, difficulty: DifficultyLevel) -> Self {
        TestDataset {
            test_cases: self
                .test_cases
                .iter()
                .filter(|tc| std::mem::discriminant(&tc.difficulty) == std::mem::discriminant(&difficulty))
                .cloned()
                .collect(),
        }
    }

    /// Get test cases by tags
    pub fn filter_by_tags(&self, tags: &[&str]) -> Self {
        TestDataset {
            test_cases: self
                .test_cases
                .iter()
                .filter(|tc| tags.iter().any(|tag| tc.tags.contains(&tag.to_string())))
                .cloned()
                .collect(),
        }
    }

    /// Filter test cases that have runtime validation
    pub fn filter_runtime_tests(&self) -> Self {
        TestDataset {
            test_cases: self
                .test_cases
                .iter()
                .filter(|tc| tc.sandbox.is_some() || (tc.assertions.is_some() && tc.assertions.as_ref().unwrap().runtime.is_some()))
                .cloned()
                .collect(),
        }
    }

    /// Filter test cases by sandbox backend
    pub fn filter_by_sandbox(&self, backend: SandboxBackend) -> Self {
        TestDataset {
            test_cases: self
                .test_cases
                .iter()
                .filter(|tc| tc.sandbox.as_ref().map(|s| s.backend == backend).unwrap_or(false))
                .cloned()
                .collect(),
        }
    }

    /// Get statistics about the dataset
    pub fn stats(&self) -> DatasetStats {
        let mut category_counts = HashMap::new();
        let mut shell_counts = HashMap::new();
        let mut difficulty_counts = HashMap::new();
        let mut safety_counts = HashMap::new();
        let mut sandbox_counts = HashMap::new();

        for test_case in &self.test_cases {
            *category_counts.entry(test_case.category.clone()).or_insert(0) += 1;
            *shell_counts.entry(test_case.shell).or_insert(0) += 1;
            *difficulty_counts.entry(format!("{:?}", test_case.difficulty)).or_insert(0) += 1;
            *safety_counts.entry(format!("{:?}", test_case.safety_level)).or_insert(0) += 1;

            if let Some(sandbox) = &test_case.sandbox {
                *sandbox_counts.entry(format!("{:?}", sandbox.backend)).or_insert(0) += 1;
            } else {
                *sandbox_counts.entry("None".to_string()).or_insert(0) += 1;
            }
        }

        DatasetStats {
            total_cases: self.test_cases.len(),
            category_counts,
            shell_counts,
            difficulty_counts,
            safety_counts,
            sandbox_counts,
            runtime_test_count: self.test_cases.iter().filter(|tc| tc.sandbox.is_some()).count(),
        }
    }
}

/// Dataset statistics
#[derive(Debug)]
pub struct DatasetStats {
    pub total_cases: usize,
    pub category_counts: HashMap<String, usize>,
    pub shell_counts: HashMap<ShellType, usize>,
    pub difficulty_counts: HashMap<String, usize>,
    pub safety_counts: HashMap<String, usize>,
    pub sandbox_counts: HashMap<String, usize>,
    pub runtime_test_count: usize,
}

impl std::fmt::Display for DatasetStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Dataset Statistics:")?;
        writeln!(f, "Total test cases: {}", self.total_cases)?;
        writeln!(f, "Runtime tests: {}", self.runtime_test_count)?;

        writeln!(f, "\nBy Category:")?;
        for (category, count) in &self.category_counts {
            writeln!(f, "  {}: {}", category, count)?;
        }

        writeln!(f, "\nBy Shell:")?;
        for (shell, count) in &self.shell_counts {
            writeln!(f, "  {:?}: {}", shell, count)?;
        }

        writeln!(f, "\nBy Difficulty:")?;
        for (difficulty, count) in &self.difficulty_counts {
            writeln!(f, "  {}: {}", difficulty, count)?;
        }

        writeln!(f, "\nBy Safety Level:")?;
        for (safety, count) in &self.safety_counts {
            writeln!(f, "  {}: {}", safety, count)?;
        }

        writeln!(f, "\nBy Sandbox Backend:")?;
        for (backend, count) in &self.sandbox_counts {
            writeln!(f, "  {}: {}", backend, count)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testcase_serialization() {
        let test_case = TestCase {
            id: "test_001".to_string(),
            category: "file_search".to_string(),
            subcategory: "basic".to_string(),
            shell: ShellType::Bash,
            difficulty: DifficultyLevel::Basic,
            input: "list all files".to_string(),
            expected_commands: vec!["ls -la".to_string()],
            explanation: "Simple file listing".to_string(),
            tags: vec!["basic".to_string()],
            safety_level: SafetyLevel::Safe,
            sandbox: None,
            assertions: None,
        };

        let yaml = serde_yaml::to_string(&test_case).unwrap();
        let deserialized: TestCase = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(test_case.id, deserialized.id);
    }

    #[test]
    fn test_testcase_with_sandbox() {
        let test_case = TestCase {
            id: "test_002".to_string(),
            category: "file_search".to_string(),
            subcategory: "basic".to_string(),
            shell: ShellType::Bash,
            difficulty: DifficultyLevel::Intermediate,
            input: "list all files".to_string(),
            expected_commands: vec!["ls -la".to_string()],
            explanation: "File listing with runtime validation".to_string(),
            tags: vec!["runtime".to_string()],
            safety_level: SafetyLevel::Safe,
            sandbox: Some(SandboxConfig::default()),
            assertions: Some(AssertionConfig {
                runtime: Some(RuntimeAssertions {
                    allowed_exit_codes: vec![0],
                    ..Default::default()
                }),
                ..Default::default()
            }),
        };

        let yaml = serde_yaml::to_string(&test_case).unwrap();
        let deserialized: TestCase = serde_yaml::from_str(&yaml).unwrap();
        assert!(deserialized.sandbox.is_some());
        assert!(deserialized.assertions.is_some());
    }
}
