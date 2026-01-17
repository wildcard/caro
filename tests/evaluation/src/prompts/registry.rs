//! Prompt version registry and loading.

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use super::metadata::PromptMetadata;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Prompt not found: {0}")]
    PromptNotFound(String),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid prompt directory structure")]
    InvalidStructure,
}

/// Prompt registry for managing versioned prompts
pub struct PromptRegistry {
    prompts_dir: PathBuf,
}

impl PromptRegistry {
    /// Create a new prompt registry
    pub fn new<P: AsRef<Path>>(prompts_dir: P) -> Result<Self, RegistryError> {
        let prompts_dir = prompts_dir.as_ref().to_path_buf();

        if !prompts_dir.exists() {
            fs::create_dir_all(&prompts_dir)?;
        }

        Ok(Self { prompts_dir })
    }

    /// Load a specific prompt version
    pub fn load_prompt(&self, name: &str, version: &str) -> Result<String, RegistryError> {
        let prompt_path = self.prompts_dir.join(version).join(format!("{}.md", name));

        if !prompt_path.exists() {
            return Err(RegistryError::PromptNotFound(format!(
                "{} v{}",
                name, version
            )));
        }

        Ok(fs::read_to_string(prompt_path)?)
    }

    /// List all available versions for a prompt
    pub fn list_versions(&self, name: &str) -> Result<Vec<String>, RegistryError> {
        let mut versions = Vec::new();

        for entry in fs::read_dir(&self.prompts_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let version = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Check if this version has the requested prompt
                let prompt_file = path.join(format!("{}.md", name));
                if prompt_file.exists() {
                    versions.push(version);
                }
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Load metadata for a specific version
    pub fn load_metadata(&self, version: &str) -> Result<PromptMetadata, RegistryError> {
        let metadata_path = self.prompts_dir.join(version).join("metadata.yaml");

        if !metadata_path.exists() {
            return Err(RegistryError::VersionNotFound(version.to_string()));
        }

        PromptMetadata::from_yaml_file(&metadata_path).map_err(|_| RegistryError::InvalidStructure)
    }
}
