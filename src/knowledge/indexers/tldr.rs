//! Tldr page indexer
//!
//! Indexes tldr pages (simplified man pages) into the Docs collection.
//! Uses the tldr-pages repository for concise, example-focused documentation.

use super::{IndexStats, Indexer, ProgressCallback};
use crate::knowledge::{
    backends::VectorBackend, collections::CollectionType, index::KnowledgeEntry, schema::EntryType,
    Result,
};
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// Tldr page indexer
///
/// Discovers and indexes tldr pages into the knowledge base.
/// Tldr pages are community-maintained, simplified documentation with practical examples.
pub struct TldrIndexer {
    /// Path to tldr-pages repository (if None, uses default cache)
    pub tldr_path: Option<std::path::PathBuf>,
    /// Platform filter (e.g., "linux", "osx", "common")
    pub platforms: Vec<String>,
}

impl TldrIndexer {
    /// Create a new tldr indexer
    ///
    /// # Arguments
    /// * `tldr_path` - Optional path to tldr-pages repo (None = use system cache)
    /// * `platforms` - Platform filters (empty = all platforms)
    pub fn new(tldr_path: Option<std::path::PathBuf>, platforms: Vec<String>) -> Self {
        Self {
            tldr_path,
            platforms,
        }
    }

    /// Create indexer using system tldr cache
    pub fn from_system_cache() -> Self {
        Self::new(None, vec![])
    }

    /// Create indexer for current platform only
    pub fn current_platform() -> Self {
        let platform = if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "common"
        };

        Self::new(None, vec![platform.to_string(), "common".to_string()])
    }

    /// Find tldr cache directory
    fn find_tldr_cache(&self) -> Result<PathBuf> {
        if let Some(ref path) = self.tldr_path {
            return Ok(path.clone());
        }

        // Try standard locations
        let home = std::env::var("HOME")
            .map_err(|_| crate::knowledge::KnowledgeError::Indexing("HOME not set".to_string()))?;

        let cache_path = PathBuf::from(home).join(".cache/tldr/pages");
        if cache_path.exists() {
            return Ok(cache_path);
        }

        Err(crate::knowledge::KnowledgeError::Indexing(
            "tldr cache not found - install tldr first".to_string(),
        ))
    }

    /// List tldr markdown files
    fn list_tldr_pages(&self, cache_dir: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut pages = Vec::new();

        // If platforms specified, only search those directories
        if !self.platforms.is_empty() {
            for platform in &self.platforms {
                let platform_dir = cache_dir.join(platform);
                if platform_dir.exists() {
                    self.collect_md_files(&platform_dir, &mut pages)?;
                }
            }
        } else {
            // Search all platform directories
            self.collect_md_files(cache_dir, &mut pages)?;
        }

        Ok(pages)
    }

    /// Recursively collect .md files
    fn collect_md_files(&self, dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.collect_md_files(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Parse tldr markdown content
    fn parse_tldr_content(&self, path: &PathBuf) -> Result<String> {
        let content = fs::read_to_string(path)?;

        // tldr pages are already concise markdown - just clean up slightly
        let mut result = String::new();

        for line in content.lines() {
            // Remove markdown # headers but keep the text
            if let Some(text) = line.strip_prefix('#') {
                result.push_str(text.trim());
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        Ok(result.trim().to_string())
    }
}

#[async_trait]
impl Indexer for TldrIndexer {
    fn name(&self) -> &'static str {
        "tldr"
    }

    async fn index_all(
        &self,
        backend: Arc<dyn VectorBackend>,
        progress: Option<ProgressCallback>,
    ) -> Result<IndexStats> {
        let mut stats = IndexStats::new();

        // Find tldr cache
        let cache_dir = match self.find_tldr_cache() {
            Ok(dir) => dir,
            Err(_) => return Ok(stats), // No tldr cache, return empty stats
        };

        // List all pages
        let pages = self.list_tldr_pages(&cache_dir)?;
        let total = pages.len();

        if total == 0 {
            return Ok(stats);
        }

        // Index each page
        for (idx, page_path) in pages.iter().enumerate() {
            if let Some(ref callback) = progress {
                callback(idx, total);
            }

            // Extract command name from filename
            let command_name = page_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            if !self.should_index(&command_name) {
                stats.record_skip();
                continue;
            }

            // Parse content
            match self.parse_tldr_content(page_path) {
                Ok(content) => {
                    let entry = KnowledgeEntry {
                        request: command_name.clone(),
                        command: content,
                        context: Some(format!("tldr:{}", command_name)),
                        similarity: 0.0,
                        timestamp: Utc::now(),
                        entry_type: EntryType::Success,
                        original_command: None,
                        feedback: None,
                        profile: None,
                    };

                    match backend.add_entry(entry, CollectionType::Docs).await {
                        Ok(()) => stats.record_success(),
                        Err(_) => stats.record_failure(),
                    }
                }
                Err(_) => stats.record_failure(),
            }
        }

        if let Some(ref callback) = progress {
            callback(total, total);
        }

        Ok(stats)
    }

    async fn index_one(&self, backend: Arc<dyn VectorBackend>, item: &str) -> Result<bool> {
        // Find tldr cache
        let cache_dir = self.find_tldr_cache()?;

        // Try to find the page in any platform directory
        let mut found_path: Option<PathBuf> = None;

        if !self.platforms.is_empty() {
            for platform in &self.platforms {
                let page_path = cache_dir.join(platform).join(format!("{}.md", item));
                if page_path.exists() {
                    found_path = Some(page_path);
                    break;
                }
            }
        } else {
            // Search all platforms
            for platform in &["common", "linux", "osx", "windows"] {
                let page_path = cache_dir.join(platform).join(format!("{}.md", item));
                if page_path.exists() {
                    found_path = Some(page_path);
                    break;
                }
            }
        }

        let page_path = match found_path {
            Some(path) => path,
            None => return Ok(false), // Page not found
        };

        // Parse and index
        let content = self.parse_tldr_content(&page_path)?;

        let entry = KnowledgeEntry {
            request: item.to_string(),
            command: content,
            context: Some(format!("tldr:{}", item)),
            similarity: 0.0,
            timestamp: Utc::now(),
            entry_type: EntryType::Success,
            original_command: None,
            feedback: None,
            profile: None,
        };

        backend.add_entry(entry, CollectionType::Docs).await?;
        Ok(true)
    }

    fn should_index(&self, _item: &str) -> bool {
        // tldr pages are all useful - no filtering needed
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tldr_indexer_new() {
        let indexer = TldrIndexer::new(None, vec!["linux".to_string()]);
        assert_eq!(indexer.platforms, vec!["linux"]);
        assert_eq!(indexer.name(), "tldr");
    }

    #[test]
    fn test_tldr_indexer_from_system_cache() {
        let indexer = TldrIndexer::from_system_cache();
        assert!(indexer.tldr_path.is_none());
        assert!(indexer.platforms.is_empty());
    }

    #[test]
    fn test_tldr_indexer_current_platform() {
        let indexer = TldrIndexer::current_platform();
        assert!(indexer.platforms.contains(&"common".to_string()));

        // Platform-specific check
        #[cfg(target_os = "linux")]
        assert!(indexer.platforms.contains(&"linux".to_string()));

        #[cfg(target_os = "macos")]
        assert!(indexer.platforms.contains(&"osx".to_string()));
    }
}
