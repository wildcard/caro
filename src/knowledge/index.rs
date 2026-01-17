//! Knowledge Index with pluggable backend support
//!
//! Provides persistent storage and semantic search for command knowledge
//! using a configurable vector database backend (LanceDB by default).

use crate::knowledge::{backends::VectorBackend, schema::EntryType, Result};
use chrono::{DateTime, Utc};
use std::path::Path;
use std::sync::Arc;

/// A knowledge entry retrieved from the index
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    /// The original natural language request
    pub request: String,
    /// The shell command
    pub command: String,
    /// Directory/project context (if available)
    pub context: Option<String>,
    /// Similarity score (0.0 to 1.0, higher is more similar)
    pub similarity: f32,
    /// When this entry was created
    pub timestamp: DateTime<Utc>,
    /// Type of entry (success or correction)
    pub entry_type: EntryType,
    /// For corrections: the original command that was wrong
    pub original_command: Option<String>,
    /// For corrections: feedback about why it was wrong
    pub feedback: Option<String>,
}

/// Statistics about the knowledge index
#[derive(Debug, Clone)]
pub struct KnowledgeStats {
    pub total_entries: usize,
    pub success_count: usize,
    pub correction_count: usize,
}

/// Local knowledge index for storing and searching command patterns
///
/// This is a thin wrapper around a VectorBackend that provides a consistent API
/// regardless of the underlying vector database implementation.
pub struct KnowledgeIndex {
    backend: Arc<dyn VectorBackend>,
}

impl KnowledgeIndex {
    /// Open or create a knowledge index at the given path using the default backend (LanceDB)
    ///
    /// # Arguments
    /// * `path` - Directory where the knowledge index will be stored
    ///
    /// # Returns
    /// A configured knowledge index ready for use
    pub async fn open(path: &Path) -> Result<Self> {
        use crate::knowledge::backends::lancedb::LanceDbBackend;
        let backend = LanceDbBackend::new(path).await?;
        Ok(Self {
            backend: Arc::new(backend),
        })
    }

    /// Create a knowledge index with a custom backend
    ///
    /// # Arguments
    /// * `backend` - The vector backend to use
    ///
    /// # Returns
    /// A knowledge index using the provided backend
    pub fn with_backend(backend: Arc<dyn VectorBackend>) -> Self {
        Self { backend }
    }

    /// Create a knowledge index from configuration
    ///
    /// # Arguments
    /// * `config` - Backend configuration (LanceDB or ChromaDB)
    ///
    /// # Returns
    /// A configured knowledge index ready for use
    pub async fn from_config(config: &crate::models::KnowledgeBackendConfig) -> Result<Self> {
        use crate::knowledge::backends::lancedb::LanceDbBackend;

        match config {
            crate::models::KnowledgeBackendConfig::LanceDb { path } => {
                let backend = LanceDbBackend::new(path).await?;
                Ok(Self {
                    backend: Arc::new(backend),
                })
            }
            #[cfg(feature = "chromadb")]
            crate::models::KnowledgeBackendConfig::ChromaDb { url, cache_dir } => {
                use crate::knowledge::backends::chromadb::ChromaDbBackend;
                let backend = ChromaDbBackend::new(url, cache_dir.as_deref()).await?;
                Ok(Self {
                    backend: Arc::new(backend),
                })
            }
            #[cfg(not(feature = "chromadb"))]
            crate::models::KnowledgeBackendConfig::ChromaDb { .. } => {
                Err(crate::knowledge::KnowledgeError::Database(
                    "ChromaDB backend not available. Rebuild with --features chromadb".to_string(),
                ))
            }
        }
    }

    /// Record a successful command execution
    ///
    /// # Arguments
    /// * `request` - The natural language request
    /// * `command` - The executed shell command
    /// * `context` - Optional project/directory context
    pub async fn record_success(
        &self,
        request: &str,
        command: &str,
        context: Option<&str>,
    ) -> Result<()> {
        self.backend.record_success(request, command, context).await
    }

    /// Record a correction from agentic refinement
    ///
    /// # Arguments
    /// * `request` - The natural language request
    /// * `original` - The original (incorrect) command
    /// * `corrected` - The corrected command
    /// * `feedback` - Optional feedback about why it was wrong
    pub async fn record_correction(
        &self,
        request: &str,
        original: &str,
        corrected: &str,
        feedback: Option<&str>,
    ) -> Result<()> {
        self.backend
            .record_correction(request, original, corrected, feedback)
            .await
    }

    /// Find similar past commands
    ///
    /// # Arguments
    /// * `query` - The search query
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// Vector of knowledge entries, sorted by similarity (descending)
    pub async fn find_similar(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>> {
        self.backend.find_similar(query, limit).await
    }

    /// Get statistics about the knowledge index
    pub async fn stats(&self) -> Result<KnowledgeStats> {
        let backend_stats = self.backend.stats().await?;
        Ok(KnowledgeStats {
            total_entries: backend_stats.total_entries,
            success_count: backend_stats.success_count,
            correction_count: backend_stats.correction_count,
        })
    }

    /// Clear all entries from the index
    pub async fn clear(&self) -> Result<()> {
        self.backend.clear().await
    }

    /// Check if the backend is healthy and ready to serve requests
    pub async fn is_healthy(&self) -> bool {
        self.backend.is_healthy().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_index_create() {
        let temp_dir = TempDir::new().unwrap();
        let index = KnowledgeIndex::open(temp_dir.path()).await.unwrap();
        assert!(index.is_healthy().await);
        let stats = index.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_record_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let index = KnowledgeIndex::open(temp_dir.path()).await.unwrap();

        // Record a success
        index
            .record_success("list all files", "ls -la", Some("rust project"))
            .await
            .unwrap();

        // Search for similar
        let results = index.find_similar("show files", 5).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].command, "ls -la");
    }

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_record_correction() {
        let temp_dir = TempDir::new().unwrap();
        let index = KnowledgeIndex::open(temp_dir.path()).await.unwrap();

        index
            .record_correction(
                "show disk usage",
                "ls -lh",
                "du -h -d 1",
                Some("ls shows files not disk usage"),
            )
            .await
            .unwrap();

        let stats = index.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
    }

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_clear() {
        let temp_dir = TempDir::new().unwrap();
        let index = KnowledgeIndex::open(temp_dir.path()).await.unwrap();

        index
            .record_success("test", "echo test", None)
            .await
            .unwrap();

        let stats = index.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);

        index.clear().await.unwrap();

        let stats = index.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }
}
