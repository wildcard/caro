//! Indexing system for various documentation sources

use super::client::KnowledgeBaseClient;
use super::collections::{
    CollectionType, CommandDocMetadata, DocumentMetadata, ProjectContextMetadata,
};
use super::KnowledgeBaseError;
use chrono::Utc;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Progress indicator for indexing operations
#[derive(Debug, Clone)]
pub struct IndexingProgress {
    pub total_items: usize,
    pub indexed_items: usize,
    pub failed_items: usize,
    pub current_item: Option<String>,
}

impl IndexingProgress {
    pub fn new(total: usize) -> Self {
        Self {
            total_items: total,
            indexed_items: 0,
            failed_items: 0,
            current_item: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.indexed_items + self.failed_items >= self.total_items
    }
}

/// Document indexer for various sources
pub struct DocumentIndexer {
    client: KnowledgeBaseClient,
}

impl DocumentIndexer {
    pub fn new(client: KnowledgeBaseClient) -> Self {
        Self { client }
    }

    /// Index man pages for common commands
    pub async fn index_man_pages(
        &self,
        commands: Vec<String>,
    ) -> Result<IndexingProgress, KnowledgeBaseError> {
        info!("Indexing {} man pages", commands.len());
        let mut progress = IndexingProgress::new(commands.len());

        let collection = self
            .client
            .get_or_create_collection(CollectionType::CommandDocs.collection_name())
            .await?;

        for command in commands {
            progress.current_item = Some(command.clone());

            match self.fetch_man_page(&command).await {
                Ok(content) => {
                    let metadata = CommandDocMetadata {
                        command_name: command.clone(),
                        source_type: "man".to_string(),
                        shell_type: None,
                        last_updated: Utc::now().to_rfc3339(),
                        platform: std::env::consts::OS.to_string(),
                    };

                    let id = Uuid::new_v4().to_string();

                    match self
                        .client
                        .add_documents(
                            &collection,
                            vec![id],
                            None, // Let ChromaDB generate embeddings
                            Some(vec![metadata.to_metadata()]),
                            Some(vec![content]),
                        )
                        .await
                    {
                        Ok(_) => {
                            debug!("Indexed man page for {}", command);
                            progress.indexed_items += 1;
                        }
                        Err(e) => {
                            warn!("Failed to index man page for {}: {}", command, e);
                            progress.failed_items += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch man page for {}: {}", command, e);
                    progress.failed_items += 1;
                }
            }
        }

        Ok(progress)
    }

    /// Fetch man page content for a command
    async fn fetch_man_page(&self, command: &str) -> Result<String, KnowledgeBaseError> {
        let output = Command::new("man")
            .arg(command)
            .output()
            .map_err(|e| KnowledgeBaseError::IndexingError {
                message: format!("Failed to run man command: {}", e),
            })?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(content)
        } else {
            Err(KnowledgeBaseError::IndexingError {
                message: format!("man page not found for {}", command),
            })
        }
    }

    /// Index tldr pages for common commands
    /// Uses tealdeer-like approach for accessing tldr cache
    pub async fn index_tldr_pages(
        &self,
        commands: Vec<String>,
    ) -> Result<IndexingProgress, KnowledgeBaseError> {
        info!("Indexing {} tldr pages", commands.len());
        let mut progress = IndexingProgress::new(commands.len());

        let collection = self
            .client
            .get_or_create_collection(CollectionType::CommandDocs.collection_name())
            .await?;

        // Get tldr cache directory (following tealdeer convention)
        let cache_dir = self.get_tldr_cache_dir();

        for command in commands {
            progress.current_item = Some(command.clone());

            match self.fetch_tldr_page(&cache_dir, &command).await {
                Ok(content) => {
                    let metadata = CommandDocMetadata {
                        command_name: command.clone(),
                        source_type: "tldr".to_string(),
                        shell_type: None,
                        last_updated: Utc::now().to_rfc3339(),
                        platform: std::env::consts::OS.to_string(),
                    };

                    let id = Uuid::new_v4().to_string();

                    match self
                        .client
                        .add_documents(
                            &collection,
                            vec![id],
                            None,
                            Some(vec![metadata.to_metadata()]),
                            Some(vec![content]),
                        )
                        .await
                    {
                        Ok(_) => {
                            debug!("Indexed tldr page for {}", command);
                            progress.indexed_items += 1;
                        }
                        Err(e) => {
                            warn!("Failed to index tldr page for {}: {}", command, e);
                            progress.failed_items += 1;
                        }
                    }
                }
                Err(e) => {
                    debug!("tldr page not available for {}: {}", command, e);
                    progress.failed_items += 1;
                }
            }
        }

        Ok(progress)
    }

    /// Get tldr cache directory (following tealdeer convention)
    fn get_tldr_cache_dir(&self) -> PathBuf {
        // Try XDG_CACHE_HOME first, then fall back to ~/.cache
        std::env::var("XDG_CACHE_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("tealdeer")
            .join("tldr-pages")
    }

    /// Fetch tldr page content
    async fn fetch_tldr_page(
        &self,
        cache_dir: &Path,
        command: &str,
    ) -> Result<String, KnowledgeBaseError> {
        // Try platform-specific first, then common
        let platforms = vec!["common", std::env::consts::OS];

        for platform in platforms {
            let page_path = cache_dir
                .join("pages")
                .join(platform)
                .join(format!("{}.md", command));

            if page_path.exists() {
                let content = fs::read_to_string(&page_path)
                    .await
                    .map_err(|e| KnowledgeBaseError::IndexingError {
                        message: format!("Failed to read tldr page: {}", e),
                    })?;

                return Ok(content);
            }
        }

        Err(KnowledgeBaseError::IndexingError {
            message: format!("tldr page not found for {}", command),
        })
    }

    /// Index help output for commands
    pub async fn index_help_output(
        &self,
        commands: Vec<String>,
    ) -> Result<IndexingProgress, KnowledgeBaseError> {
        info!("Indexing help output for {} commands", commands.len());
        let mut progress = IndexingProgress::new(commands.len());

        let collection = self
            .client
            .get_or_create_collection(CollectionType::CommandDocs.collection_name())
            .await?;

        for command in commands {
            progress.current_item = Some(command.clone());

            // Try both --help and -h
            if let Ok(content) = self.fetch_help_output(&command).await {
                let metadata = CommandDocMetadata {
                    command_name: command.clone(),
                    source_type: "help".to_string(),
                    shell_type: None,
                    last_updated: Utc::now().to_rfc3339(),
                    platform: std::env::consts::OS.to_string(),
                };

                let id = Uuid::new_v4().to_string();

                match self
                    .client
                    .add_documents(
                        &collection,
                        vec![id],
                        None,
                        Some(vec![metadata.to_metadata()]),
                        Some(vec![content]),
                    )
                    .await
                {
                    Ok(_) => {
                        debug!("Indexed help output for {}", command);
                        progress.indexed_items += 1;
                    }
                    Err(e) => {
                        warn!("Failed to index help output for {}: {}", command, e);
                        progress.failed_items += 1;
                    }
                }
            } else {
                debug!("No help output available for {}", command);
                progress.failed_items += 1;
            }
        }

        Ok(progress)
    }

    /// Fetch help output for a command
    async fn fetch_help_output(&self, command: &str) -> Result<String, KnowledgeBaseError> {
        // Try --help first
        let mut output = Command::new(command)
            .arg("--help")
            .output()
            .or_else(|_| Command::new(command).arg("-h").output())
            .map_err(|e| KnowledgeBaseError::IndexingError {
                message: format!("Failed to run help command: {}", e),
            })?;

        // Some commands output to stderr
        let content = if !output.stdout.is_empty() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        if content.is_empty() {
            return Err(KnowledgeBaseError::IndexingError {
                message: "Help output is empty".to_string(),
            });
        }

        Ok(content)
    }

    /// Index GitHub/GitLab repository documentation
    /// Supports: README.md, wiki pages, GitHub Pages
    pub async fn index_git_repository(
        &self,
        repo_url: &str,
        local_path: Option<PathBuf>,
    ) -> Result<IndexingProgress, KnowledgeBaseError> {
        info!("Indexing Git repository: {}", repo_url);

        let collection = self
            .client
            .get_or_create_collection(CollectionType::ProjectContext.collection_name())
            .await?;

        // Clone or use existing local path
        let repo_path = if let Some(path) = local_path {
            path
        } else {
            self.clone_repository(repo_url).await?
        };

        // Find all markdown files
        let md_files = self.find_markdown_files(&repo_path).await?;
        let mut progress = IndexingProgress::new(md_files.len());

        for md_file in md_files {
            progress.current_item = Some(md_file.display().to_string());

            match fs::read_to_string(&md_file).await {
                Ok(content) => {
                    let file_type = md_file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Higher relevance for README, CONTRIBUTING, docs/
                    let relevance_score = self.calculate_relevance(&md_file);

                    let metadata = ProjectContextMetadata {
                        project_path: repo_url.to_string(),
                        file_type,
                        relevance_score,
                        last_indexed: Utc::now().to_rfc3339(),
                    };

                    let id = Uuid::new_v4().to_string();

                    match self
                        .client
                        .add_documents(
                            &collection,
                            vec![id],
                            None,
                            Some(vec![metadata.to_metadata()]),
                            Some(vec![content]),
                        )
                        .await
                    {
                        Ok(_) => {
                            debug!("Indexed {:?}", md_file);
                            progress.indexed_items += 1;
                        }
                        Err(e) => {
                            warn!("Failed to index {:?}: {}", md_file, e);
                            progress.failed_items += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read {:?}: {}", md_file, e);
                    progress.failed_items += 1;
                }
            }
        }

        Ok(progress)
    }

    /// Clone a Git repository
    async fn clone_repository(&self, repo_url: &str) -> Result<PathBuf, KnowledgeBaseError> {
        let temp_dir = std::env::temp_dir().join(format!("cmdai_repo_{}", Uuid::new_v4()));

        let output = Command::new("git")
            .args(&["clone", "--depth", "1", repo_url, temp_dir.to_str().unwrap()])
            .output()
            .map_err(|e| KnowledgeBaseError::IndexingError {
                message: format!("Failed to clone repository: {}", e),
            })?;

        if !output.status.success() {
            return Err(KnowledgeBaseError::IndexingError {
                message: format!(
                    "Git clone failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(temp_dir)
    }

    /// Find all markdown files in a directory
    async fn find_markdown_files(&self, dir: &Path) -> Result<Vec<PathBuf>, KnowledgeBaseError> {
        let mut md_files = Vec::new();

        let mut stack = vec![dir.to_path_buf()];

        while let Some(current_dir) = stack.pop() {
            if let Ok(mut entries) = fs::read_dir(&current_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();

                    // Skip .git and node_modules
                    if path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n == ".git" || n == "node_modules")
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    if path.is_dir() {
                        stack.push(path);
                    } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        md_files.push(path);
                    }
                }
            }
        }

        Ok(md_files)
    }

    /// Calculate relevance score for a file
    fn calculate_relevance(&self, path: &Path) -> f64 {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        let path_str = path.to_str().unwrap_or("").to_lowercase();

        // High relevance files
        if filename.starts_with("readme") {
            return 1.0;
        }

        if filename.contains("contributing") || filename.contains("install") {
            return 0.9;
        }

        if path_str.contains("/docs/") || path_str.contains("/documentation/") {
            return 0.8;
        }

        if path_str.contains("/wiki/") {
            return 0.85;
        }

        if filename.contains("guide") || filename.contains("tutorial") {
            return 0.75;
        }

        // Default relevance
        0.5
    }

    /// Index GitHub Pages for a repository
    pub async fn index_github_pages(
        &self,
        pages_url: &str,
    ) -> Result<IndexingProgress, KnowledgeBaseError> {
        info!("Indexing GitHub Pages: {}", pages_url);

        // Fetch the main page
        let response = reqwest::get(pages_url)
            .await
            .map_err(|e| KnowledgeBaseError::IndexingError {
                message: format!("Failed to fetch GitHub Pages: {}", e),
            })?;

        let html = response.text().await.map_err(|e| KnowledgeBaseError::IndexingError {
            message: format!("Failed to read response: {}", e),
        })?;

        // Extract text content (basic HTML parsing)
        let text_content = self.extract_text_from_html(&html);

        let collection = self
            .client
            .get_or_create_collection(CollectionType::ProjectContext.collection_name())
            .await?;

        let metadata = ProjectContextMetadata {
            project_path: pages_url.to_string(),
            file_type: "github_pages".to_string(),
            relevance_score: 0.9,
            last_indexed: Utc::now().to_rfc3339(),
        };

        let id = Uuid::new_v4().to_string();

        self.client
            .add_documents(
                &collection,
                vec![id],
                None,
                Some(vec![metadata.to_metadata()]),
                Some(vec![text_content]),
            )
            .await?;

        let mut progress = IndexingProgress::new(1);
        progress.indexed_items = 1;

        Ok(progress)
    }

    /// Extract text content from HTML (basic implementation)
    fn extract_text_from_html(&self, html: &str) -> String {
        // Remove script and style tags
        let script_regex = Regex::new(r"<script[^>]*>[\s\S]*?</script>").unwrap();
        let style_regex = Regex::new(r"<style[^>]*>[\s\S]*?</style>").unwrap();
        let html_clean = script_regex.replace_all(html, "");
        let html_clean = style_regex.replace_all(&html_clean, "");

        // Remove HTML tags
        let tag_regex = Regex::new(r"<[^>]+>").unwrap();
        let text = tag_regex.replace_all(&html_clean, " ");

        // Clean up whitespace
        let ws_regex = Regex::new(r"\s+").unwrap();
        ws_regex.replace_all(&text, " ").trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_relevance() {
        let indexer = DocumentIndexer {
            client: KnowledgeBaseClient::new(Default::default())
                .await
                .expect("Failed to create client"),
        };

        assert_eq!(indexer.calculate_relevance(Path::new("README.md")), 1.0);
        assert_eq!(indexer.calculate_relevance(Path::new("docs/guide.md")), 0.8);
        assert_eq!(indexer.calculate_relevance(Path::new("CONTRIBUTING.md")), 0.9);
    }
}
