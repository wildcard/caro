//! Multi-source documentation indexers
//!
//! Provides indexers for different documentation sources:
//! - ManPageIndexer: Index man pages into Docs collection
//! - TldrIndexer: Index tldr pages into Docs collection
//! - HelpIndexer: Index --help output into Docs collection
//! - GitHubDocsIndexer: Index GitHub repository README files

use crate::knowledge::{backends::VectorBackend, collections::CollectionType, Result};
use async_trait::async_trait;
use std::sync::Arc;

pub mod github;
pub mod help;
pub mod man;
pub mod tldr;

/// Progress callback for indexing operations
pub type ProgressCallback = Box<dyn Fn(usize, usize) + Send + Sync>;

/// Statistics about an indexing operation
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Total items processed
    pub total_processed: usize,
    /// Successfully indexed items
    pub successful: usize,
    /// Failed items
    pub failed: usize,
    /// Skipped items (already indexed or filtered out)
    pub skipped: usize,
}

impl IndexStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self {
            total_processed: 0,
            successful: 0,
            failed: 0,
            skipped: 0,
        }
    }

    /// Record a successful indexing
    pub fn record_success(&mut self) {
        self.total_processed += 1;
        self.successful += 1;
    }

    /// Record a failed indexing
    pub fn record_failure(&mut self) {
        self.total_processed += 1;
        self.failed += 1;
    }

    /// Record a skipped item
    pub fn record_skip(&mut self) {
        self.total_processed += 1;
        self.skipped += 1;
    }
}

impl Default for IndexStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified interface for documentation indexers
///
/// Each indexer is responsible for:
/// 1. Discovering items to index (man pages, tldr pages, help output)
/// 2. Parsing the content into structured knowledge entries
/// 3. Adding entries to the backend's Docs collection
#[async_trait]
pub trait Indexer: Send + Sync {
    /// Get the name of this indexer (e.g., "man", "tldr", "help")
    fn name(&self) -> &'static str;

    /// Index all available items
    ///
    /// # Arguments
    /// * `backend` - The vector backend to store indexed entries
    /// * `progress` - Optional callback for progress updates (current, total)
    ///
    /// # Returns
    /// Statistics about the indexing operation
    async fn index_all(
        &self,
        backend: Arc<dyn VectorBackend>,
        progress: Option<ProgressCallback>,
    ) -> Result<IndexStats>;

    /// Index a specific item by name/path
    ///
    /// # Arguments
    /// * `backend` - The vector backend to store indexed entries
    /// * `item` - The specific item to index (e.g., "ls", "grep")
    ///
    /// # Returns
    /// True if successfully indexed, false if item not found or failed
    async fn index_one(&self, backend: Arc<dyn VectorBackend>, item: &str) -> Result<bool>;

    /// Get the target collection for this indexer's entries
    fn collection(&self) -> CollectionType {
        CollectionType::Docs
    }

    /// Check if an item should be indexed
    ///
    /// Override this to implement filtering logic (e.g., skip certain commands)
    fn should_index(&self, _item: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_stats_new() {
        let stats = IndexStats::new();
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.successful, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.skipped, 0);
    }

    #[test]
    fn test_index_stats_record() {
        let mut stats = IndexStats::new();

        stats.record_success();
        assert_eq!(stats.total_processed, 1);
        assert_eq!(stats.successful, 1);

        stats.record_failure();
        assert_eq!(stats.total_processed, 2);
        assert_eq!(stats.failed, 1);

        stats.record_skip();
        assert_eq!(stats.total_processed, 3);
        assert_eq!(stats.skipped, 1);
    }
}
