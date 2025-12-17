//! Vector store module for ChromaDB/Qdrant integration
//!
//! This module provides functionality to:
//! - Initialize and manage Qdrant vector database
//! - Store indexed man page documents with embeddings
//! - Query for relevant command documentation using RAG
//! - Handle graceful degradation when vector store unavailable

pub mod client;
pub mod embeddings;
pub mod query;

pub use client::VectorStoreClient;
pub use query::{CommandDoc, CommandDocQuery};

use crate::indexing::{ManPageDocument, IndexStatistics};
use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    /// Path to Qdrant database directory
    pub db_path: PathBuf,

    /// Collection name for this OS/distribution
    pub collection_name: String,

    /// Embedding model name
    pub embedding_model: String,

    /// Vector dimension (384 for all-MiniLM-L6-v2)
    pub vector_dim: usize,

    /// Whether to use in-memory mode (for testing)
    pub in_memory: bool,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("~/.cache/cmdai/qdrant"),
            collection_name: "cmdai_default".to_string(),
            embedding_model: "all-MiniLM-L6-v2".to_string(),
            vector_dim: 384,
            in_memory: false,
        }
    }
}

/// Query result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// The matched document
    pub document: ManPageDocument,

    /// Similarity score (0.0 - 1.0)
    pub score: f32,

    /// Rank in results (0 = best match)
    pub rank: usize,
}

/// Vector store statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreStats {
    /// Total number of vectors stored
    pub total_vectors: usize,

    /// Collection name
    pub collection: String,

    /// Database size in bytes
    pub db_size_bytes: u64,

    /// Whether the vector store is healthy
    pub healthy: bool,

    /// Index statistics
    pub index_stats: Option<IndexStatistics>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_store_config_default() {
        let config = VectorStoreConfig::default();
        assert_eq!(config.embedding_model, "all-MiniLM-L6-v2");
        assert_eq!(config.vector_dim, 384);
        assert!(!config.in_memory);
    }
}
