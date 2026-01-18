//! GitHub documentation indexer
//!
//! Fetches and indexes README files and documentation from GitHub repositories.

use super::{IndexStats, Indexer, ProgressCallback};
use crate::knowledge::{backends::VectorBackend, collections::CollectionType, KnowledgeEntry, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

/// GitHub documentation indexer
///
/// Fetches README.md and other documentation files from GitHub repositories
/// and indexes them into the Docs collection for enhanced command suggestions.
pub struct GitHubDocsIndexer {
    /// HTTP client for fetching from GitHub
    client: reqwest::Client,
}

impl GitHubDocsIndexer {
    /// Create a new GitHub docs indexer
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("caro-cli")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| crate::knowledge::KnowledgeError::Indexing(e.to_string()))?;

        Ok(Self { client })
    }

    /// Fetch README from a GitHub repository
    ///
    /// # Arguments
    /// * `repo` - Repository in format "owner/repo" (e.g., "wildcard/caro")
    ///
    /// # Returns
    /// README content as markdown string
    async fn fetch_readme(&self, repo: &str) -> Result<String> {
        let url = format!("https://raw.githubusercontent.com/{}/main/README.md", repo);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::knowledge::KnowledgeError::Indexing(format!("Failed to fetch {}: {}", url, e)))?;

        if !response.status().is_success() {
            // Try master branch if main doesn't exist
            let url = format!("https://raw.githubusercontent.com/{}/master/README.md", repo);
            let response = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| crate::knowledge::KnowledgeError::Indexing(format!("Failed to fetch README: {}", e)))?;

            if !response.status().is_success() {
                return Err(crate::knowledge::KnowledgeError::Indexing(format!(
                    "README not found for repository: {}",
                    repo
                )));
            }

            response
                .text()
                .await
                .map_err(|e| crate::knowledge::KnowledgeError::Indexing(e.to_string()))
        } else {
            response
                .text()
                .await
                .map_err(|e| crate::knowledge::KnowledgeError::Indexing(e.to_string()))
        }
    }

    /// Parse README into indexable chunks
    ///
    /// Extracts meaningful sections from markdown that could be useful for
    /// command suggestions.
    fn parse_readme(&self, readme: &str, repo: &str) -> Vec<(String, String)> {
        let mut chunks = Vec::new();

        // Split by headers
        let lines: Vec<&str> = readme.lines().collect();
        let mut current_section = String::new();
        let mut current_title = format!("{} documentation", repo);

        for line in lines {
            if line.starts_with('#') {
                // Save previous section if non-empty
                if !current_section.trim().is_empty() {
                    chunks.push((current_title.clone(), current_section.trim().to_string()));
                }

                // Start new section
                current_title = line.trim_start_matches('#').trim().to_string();
                current_section = String::new();
            } else {
                current_section.push_str(line);
                current_section.push('\n');
            }
        }

        // Add final section
        if !current_section.trim().is_empty() {
            chunks.push((current_title, current_section.trim().to_string()));
        }

        // Filter out chunks that are too small or unlikely to be useful
        chunks
            .into_iter()
            .filter(|(_, content)| {
                content.len() > 50 && // At least 50 characters
                !content.to_lowercase().contains("license") && // Skip license sections
                !content.to_lowercase().contains("contributor") // Skip contributor sections
            })
            .collect()
    }
}

impl Default for GitHubDocsIndexer {
    fn default() -> Self {
        Self::new().expect("Failed to create GitHubDocsIndexer")
    }
}

#[async_trait]
impl Indexer for GitHubDocsIndexer {
    fn name(&self) -> &'static str {
        "github"
    }

    async fn index_all(
        &self,
        _backend: Arc<dyn VectorBackend>,
        _progress: Option<ProgressCallback>,
    ) -> Result<IndexStats> {
        // index_all doesn't make sense for GitHub - there's no finite list
        // Users should call index_one for specific repositories
        Err(crate::knowledge::KnowledgeError::Indexing(
            "GitHub indexer requires specific repository. Use index_one() with repo name.".to_string(),
        ))
    }

    async fn index_one(&self, backend: Arc<dyn VectorBackend>, repo: &str) -> Result<bool> {
        // Validate repo format (owner/repo)
        if !repo.contains('/') || repo.split('/').count() != 2 {
            return Err(crate::knowledge::KnowledgeError::Indexing(format!(
                "Invalid repository format. Expected 'owner/repo', got: {}",
                repo
            )));
        }

        // Fetch README
        let readme = self.fetch_readme(repo).await?;

        // Parse into chunks
        let chunks = self.parse_readme(&readme, repo);

        if chunks.is_empty() {
            return Ok(false); // No useful content found
        }

        // Index each chunk as a knowledge entry
        for (title, content) in chunks {
            let entry = KnowledgeEntry {
                request: format!("{} - {}", repo, title),
                command: content.clone(),
                context: Some(format!("GitHub: {}", repo)),
                similarity: 0.0, // Not used for indexing
                timestamp: Utc::now(),
                entry_type: crate::knowledge::schema::EntryType::Success,
                original_command: None,
                feedback: None,
                profile: None,
            };

            backend.add_entry(entry, CollectionType::Docs).await?;
        }

        Ok(true)
    }

    fn collection(&self) -> CollectionType {
        CollectionType::Docs
    }

    fn should_index(&self, repo: &str) -> bool {
        // Only index if it looks like a valid repo name
        repo.contains('/') && repo.split('/').count() == 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexer_name() {
        let indexer = GitHubDocsIndexer::new().unwrap();
        assert_eq!(indexer.name(), "github");
    }

    #[test]
    fn test_should_index() {
        let indexer = GitHubDocsIndexer::new().unwrap();
        assert!(indexer.should_index("wildcard/caro"));
        assert!(indexer.should_index("rust-lang/rust"));
        assert!(!indexer.should_index("invalid"));
        assert!(!indexer.should_index(""));
    }

    #[test]
    fn test_parse_readme() {
        let indexer = GitHubDocsIndexer::new().unwrap();
        let readme = r#"# Test Project

This is a test project with enough content to pass the length filter.

## Installation

To install this project, run `cargo install test`. Make sure you have Rust installed on your system before proceeding with the installation.

## Usage

Use it like this:
```bash
test command --flag value
```
This will execute the test command with the specified flag and value. You can also use additional options for more advanced usage scenarios.

## License

MIT License
"#;

        let chunks = indexer.parse_readme(readme, "user/test");

        // Should have parsed sections, but filtered out License
        assert!(!chunks.is_empty());
        assert!(chunks.iter().all(|(title, _)| !title.to_lowercase().contains("license")));
    }
}
