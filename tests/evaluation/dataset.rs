//! Test dataset loading and validation

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    pub id: String,
    pub prompt: String,
    pub expected_command: String,
    pub category: Category,
    pub safe: bool,
    pub posix_compliant: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Correctness,
    Safety,
    Posix,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestDataset {
    pub version: String,
    pub test_cases: Vec<TestCase>,
}

impl TestDataset {
    pub fn from_toml(path: &std::path::Path) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let dataset: TestDataset = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;

        dataset.validate()?;
        Ok(dataset)
    }

    fn validate(&self) -> Result<(), String> {
        if self.test_cases.is_empty() {
            return Err("Dataset must contain at least one test case".to_string());
        }

        let mut seen_ids = HashSet::new();
        for test_case in &self.test_cases {
            if !seen_ids.insert(&test_case.id) {
                return Err(format!("Duplicate test ID: {}", test_case.id));
            }
            if test_case.id.is_empty() {
                return Err("Test ID cannot be empty".to_string());
            }
            if test_case.prompt.is_empty() {
                return Err(format!("Prompt cannot be empty for test {}", test_case.id));
            }
            if test_case.expected_command.is_empty() {
                return Err(format!("Expected command cannot be empty for test {}", test_case.id));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_dataset() {
        // Create a temporary TOML file
        let toml_content = r#"
version = "1.0.0"

[[test_cases]]
id = "test_01"
prompt = "list files"
expected_command = "ls"
category = "correctness"
safe = true
posix_compliant = true
"#;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_dataset.toml");
        std::fs::write(&temp_file, toml_content).unwrap();

        let dataset = TestDataset::from_toml(&temp_file).unwrap();
        assert_eq!(dataset.version, "1.0.0");
        assert_eq!(dataset.test_cases.len(), 1);
        assert_eq!(dataset.test_cases[0].id, "test_01");
    }

    #[test]
    fn test_duplicate_id_error() {
        let toml_content = r#"
version = "1.0.0"

[[test_cases]]
id = "test_01"
prompt = "list files"
expected_command = "ls"
category = "correctness"
safe = true
posix_compliant = true

[[test_cases]]
id = "test_01"
prompt = "list files again"
expected_command = "ls -la"
category = "correctness"
safe = true
posix_compliant = true
"#;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_duplicate.toml");
        std::fs::write(&temp_file, toml_content).unwrap();

        let result = TestDataset::from_toml(&temp_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate test ID: test_01"));
    }

    #[test]
    fn test_empty_dataset_error() {
        let toml_content = r#"
version = "1.0.0"
test_cases = []
"#;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_empty.toml");
        std::fs::write(&temp_file, toml_content).unwrap();

        let result = TestDataset::from_toml(&temp_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must contain at least one test case"));
    }
}
