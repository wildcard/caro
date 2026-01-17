//! Man page indexer
//!
//! Indexes Unix/Linux man pages into the Docs collection.
//! Parses man page content and creates knowledge entries for command documentation.

use super::{IndexStats, Indexer, ProgressCallback};
use crate::knowledge::{backends::VectorBackend, Result};
use async_trait::async_trait;
use std::sync::Arc;

/// Man page indexer
///
/// Discovers and indexes system man pages into the knowledge base.
/// Supports filtering by section and command name.
pub struct ManPageIndexer {
    /// Man page sections to index (1-8, or empty for all)
    pub sections: Vec<u8>,
}

impl ManPageIndexer {
    /// Create a new man page indexer
    ///
    /// # Arguments
    /// * `sections` - Man page sections to index (e.g., vec![1, 8] for user commands and admin)
    ///                If empty, indexes all sections.
    pub fn new(sections: Vec<u8>) -> Self {
        Self { sections }
    }

    /// Create indexer for all sections
    pub fn all_sections() -> Self {
        Self::new(vec![])
    }

    /// Create indexer for user commands only (section 1)
    pub fn user_commands() -> Self {
        Self::new(vec![1])
    }
}

#[async_trait]
impl Indexer for ManPageIndexer {
    fn name(&self) -> &'static str {
        "man"
    }

    async fn index_all(
        &self,
        _backend: Arc<dyn VectorBackend>,
        _progress: Option<ProgressCallback>,
    ) -> Result<IndexStats> {
        // TODO: Implement man page discovery and indexing
        // 1. Use `man -k .` to list all available man pages
        // 2. Filter by section if specified
        // 3. For each page:
        //    - Parse with `man -P cat <page>` to get raw content
        //    - Extract synopsis, description, examples
        //    - Create KnowledgeEntry with command name as request
        //    - Add to Docs collection via backend.add_entry()
        // 4. Report progress via callback

        Ok(IndexStats::new())
    }

    async fn index_one(
        &self,
        _backend: Arc<dyn VectorBackend>,
        _item: &str,
    ) -> Result<bool> {
        // TODO: Index a specific man page
        // 1. Check if man page exists with `man -w <item>`
        // 2. Parse content
        // 3. Add to Docs collection

        Ok(false)
    }

    fn should_index(&self, _item: &str) -> bool {
        // TODO: Implement filtering logic
        // - Skip internal/private commands
        // - Skip deprecated commands
        // - Skip non-shell commands if filtering enabled

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_man_page_indexer_new() {
        let indexer = ManPageIndexer::new(vec![1, 8]);
        assert_eq!(indexer.sections, vec![1, 8]);
        assert_eq!(indexer.name(), "man");
    }

    #[test]
    fn test_man_page_indexer_user_commands() {
        let indexer = ManPageIndexer::user_commands();
        assert_eq!(indexer.sections, vec![1]);
    }

    #[test]
    fn test_man_page_indexer_all_sections() {
        let indexer = ManPageIndexer::all_sections();
        assert!(indexer.sections.is_empty());
    }
}
