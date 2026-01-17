//! Prompt metadata structures and serialization.

use serde::{Deserialize, Serialize};

/// Metadata for a prompt version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    /// Semantic version (e.g., "1.1")
    pub version: String,

    /// Author or team name
    pub author: String,

    /// Creation date (YYYY-MM-DD)
    pub created: String,

    /// Target models this prompt is optimized for
    pub target_models: Vec<String>,

    /// Changelog describing changes from previous version
    pub changelog: String,

    /// Baseline pass rate (if measured)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_pass_rate: Option<f64>,
}

impl PromptMetadata {
    /// Load metadata from YAML file
    pub fn from_yaml_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let metadata: PromptMetadata = serde_yaml::from_str(&content)?;
        Ok(metadata)
    }

    /// Save metadata to YAML file
    pub fn to_yaml_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
}
