//! Knowledge Index for Machine Context
//!
//! This module provides a local vector database for indexing:
//! - Successful command executions
//! - Agentic loop corrections
//! - Project-specific patterns
//!
//! Supports multiple vector backends:
//! - LanceDB (embedded, default) - Zero-config local-first storage
//! - ChromaDB (optional) - Server-based for team sharing and cloud deployments

pub mod backends;
mod embedder;
mod index;
mod schema;

pub use embedder::Embedder;
pub use index::{KnowledgeEntry, KnowledgeIndex, KnowledgeStats};
pub use schema::EntryType;

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when working with the knowledge index
#[derive(Error, Debug)]
pub enum KnowledgeError {
    #[error("Failed to initialize embedder: {0}")]
    EmbedderInit(String),

    #[error("Failed to generate embedding: {0}")]
    EmbeddingFailed(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Index not found at {0}")]
    NotFound(PathBuf),
}

/// Result type for knowledge operations
pub type Result<T> = std::result::Result<T, KnowledgeError>;

/// Default path for knowledge index storage
pub fn default_knowledge_path() -> PathBuf {
    directories::ProjectDirs::from("sh", "caro", "caro")
        .map(|dirs| dirs.data_dir().join("knowledge"))
        .unwrap_or_else(|| PathBuf::from("~/.config/caro/knowledge"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_knowledge_path() {
        let path = default_knowledge_path();
        assert!(path.to_string_lossy().contains("caro"));
    }
}
