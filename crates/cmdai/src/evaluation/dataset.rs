// Dataset loading and management for command accuracy evaluation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;

use crate::models::ShellType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub category: String,
    pub subcategory: String,
    pub shell: ShellType,
    pub difficulty: DifficultyLevel,
    pub input: String,
    pub expected_commands: Vec<String>,
    pub explanation: String,
    pub tags: Vec<String>,
    pub safety_level: SafetyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyLevel {
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "intermediate")]
    Intermediate,
    #[serde(rename = "advanced")]
    Advanced,
    #[serde(rename = "expert")]
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyLevel {
    #[serde(rename = "safe")]
    Safe,
    #[serde(rename = "moderate")]
    Moderate,
    #[serde(rename = "dangerous")]
    Dangerous,
}

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
                       || path.extension().and_then(|s| s.to_str()) == Some("yml") {
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
            test_cases: self.test_cases
                .iter()
                .filter(|tc| tc.category == category)
                .cloned()
                .collect(),
        }
    }
    
    /// Filter test cases by shell type
    pub fn filter_by_shell(&self, shell: ShellType) -> Self {
        TestDataset {
            test_cases: self.test_cases
                .iter()
                .filter(|tc| tc.shell == shell)
                .cloned()
                .collect(),
        }
    }
    
    /// Filter test cases by difficulty
    pub fn filter_by_difficulty(&self, difficulty: DifficultyLevel) -> Self {
        TestDataset {
            test_cases: self.test_cases
                .iter()
                .filter(|tc| std::mem::discriminant(&tc.difficulty) == std::mem::discriminant(&difficulty))
                .cloned()
                .collect(),
        }
    }
    
    /// Get test cases by tags
    pub fn filter_by_tags(&self, tags: &[&str]) -> Self {
        TestDataset {
            test_cases: self.test_cases
                .iter()
                .filter(|tc| tags.iter().any(|tag| tc.tags.contains(&tag.to_string())))
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
        
        for test_case in &self.test_cases {
            *category_counts.entry(test_case.category.clone()).or_insert(0) += 1;
            *shell_counts.entry(test_case.shell).or_insert(0) += 1;
            *difficulty_counts.entry(format!("{:?}", test_case.difficulty)).or_insert(0) += 1;
            *safety_counts.entry(format!("{:?}", test_case.safety_level)).or_insert(0) += 1;
        }
        
        DatasetStats {
            total_cases: self.test_cases.len(),
            category_counts,
            shell_counts,
            difficulty_counts,
            safety_counts,
        }
    }
}

#[derive(Debug)]
pub struct DatasetStats {
    pub total_cases: usize,
    pub category_counts: HashMap<String, usize>,
    pub shell_counts: HashMap<ShellType, usize>,
    pub difficulty_counts: HashMap<String, usize>,
    pub safety_counts: HashMap<String, usize>,
}

impl std::fmt::Display for DatasetStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Dataset Statistics:")?;
        writeln!(f, "Total test cases: {}", self.total_cases)?;
        
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
        
        Ok(())
    }
}