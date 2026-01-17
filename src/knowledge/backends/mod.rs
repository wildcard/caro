//! Vector database backend abstraction
//!
//! Provides a unified interface for different vector database implementations.
//! Supports both embedded (LanceDB) and server-based (ChromaDB) backends.

use crate::knowledge::collections::{CollectionType, QueryScope};
use crate::knowledge::{KnowledgeEntry, Result};
use async_trait::async_trait;

pub mod lancedb;

#[cfg(feature = "chromadb")]
pub mod chromadb;

/// Statistics about a vector backend
#[derive(Debug, Clone)]
pub struct BackendStats {
    /// Total number of entries in the backend
    pub total_entries: usize,
    /// Number of success entries
    pub success_count: usize,
    /// Number of correction entries
    pub correction_count: usize,
}

/// Unified interface for vector database backends
///
/// This trait provides a common API for storing and retrieving command knowledge,
/// regardless of the underlying vector database implementation.
#[async_trait]
pub trait VectorBackend: Send + Sync {
    /// Record a successful command execution
    ///
    /// # Arguments
    /// * `request` - The natural language request
    /// * `command` - The executed shell command
    /// * `context` - Optional project/directory context
    async fn record_success(
        &self,
        request: &str,
        command: &str,
        context: Option<&str>,
    ) -> Result<()>;

    /// Record a correction from agentic refinement
    ///
    /// # Arguments
    /// * `request` - The natural language request
    /// * `original` - The original (incorrect) command
    /// * `corrected` - The corrected command
    /// * `feedback` - Optional feedback about why it was wrong
    async fn record_correction(
        &self,
        request: &str,
        original: &str,
        corrected: &str,
        feedback: Option<&str>,
    ) -> Result<()>;

    /// Find similar past commands using semantic search
    ///
    /// # Arguments
    /// * `query` - The search query
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// Vector of knowledge entries, sorted by similarity (descending)
    async fn find_similar(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>>;

    /// Get statistics about the backend
    async fn stats(&self) -> Result<BackendStats>;

    /// Clear all entries from the backend
    async fn clear(&self) -> Result<()>;

    /// Check if the backend is healthy and ready to serve requests
    async fn is_healthy(&self) -> bool;

    /// Add a knowledge entry to a specific collection
    ///
    /// # Arguments
    /// * `entry` - The knowledge entry to add
    /// * `collection` - The target collection type
    async fn add_entry(&self, entry: KnowledgeEntry, collection: CollectionType) -> Result<()>;

    /// Find similar entries within a specific query scope
    ///
    /// # Arguments
    /// * `query` - The search query
    /// * `limit` - Maximum number of results to return
    /// * `scope` - The collection scope to search within
    ///
    /// # Returns
    /// Vector of knowledge entries, sorted by similarity (descending)
    async fn find_similar_in(
        &self,
        query: &str,
        limit: usize,
        scope: QueryScope,
    ) -> Result<Vec<KnowledgeEntry>>;
}
