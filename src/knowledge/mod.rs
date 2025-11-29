//! Knowledge base and RAG (Retrieval Augmented Generation) module
//!
//! This module provides ChromaDB integration for:
//! - Indexing man pages, tldr pages, and help output
//! - Tracking execution history and learning from it
//! - User profile management for personalized command generation
//! - Project-specific context and documentation
//! - Learning from mistakes and user preferences

pub mod client;
pub mod collections;
pub mod indexers;
pub mod profiles;
pub mod retrieval;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use client::KnowledgeBaseClient;
pub use collections::{CollectionType, DocumentMetadata};
pub use indexers::{DocumentIndexer, IndexingProgress};
pub use profiles::{UserProfile, UserProfileManager};
pub use retrieval::{RAGRetriever, RetrievalContext, RetrievalResult};

/// Configuration for the knowledge base system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseConfig {
    /// ChromaDB server URL
    pub chroma_url: String,

    /// Enable ChromaDB integration
    pub enabled: bool,

    /// Directory for storing user profiles
    pub profiles_dir: PathBuf,

    /// Maximum number of documents to retrieve for RAG
    pub max_retrieval_docs: usize,

    /// Minimum similarity score for retrieval (0.0 to 1.0)
    pub min_similarity_score: f32,

    /// Enable execution history tracking
    pub track_history: bool,

    /// Enable automatic indexing of man pages
    pub auto_index_man_pages: bool,

    /// Enable project-specific context
    pub project_context_enabled: bool,
}

impl Default for KnowledgeBaseConfig {
    fn default() -> Self {
        Self {
            chroma_url: "http://localhost:8000".to_string(),
            enabled: true,
            profiles_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("cmdai")
                .join("profiles"),
            max_retrieval_docs: 5,
            min_similarity_score: 0.7,
            track_history: true,
            auto_index_man_pages: false,
            project_context_enabled: true,
        }
    }
}

impl KnowledgeBaseConfig {
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled && self.chroma_url.is_empty() {
            return Err("ChromaDB URL cannot be empty when enabled".to_string());
        }

        if self.max_retrieval_docs == 0 {
            return Err("max_retrieval_docs must be positive".to_string());
        }

        if !(0.0..=1.0).contains(&self.min_similarity_score) {
            return Err(format!(
                "min_similarity_score must be between 0.0 and 1.0, got {}",
                self.min_similarity_score
            ));
        }

        Ok(())
    }
}

/// Error types for knowledge base operations
#[derive(Debug, thiserror::Error)]
pub enum KnowledgeBaseError {
    #[error("ChromaDB connection failed: {message}")]
    ConnectionError { message: String },

    #[error("Collection operation failed: {message}")]
    CollectionError { message: String },

    #[error("Indexing failed: {message}")]
    IndexingError { message: String },

    #[error("Retrieval failed: {message}")]
    RetrievalError { message: String },

    #[error("Profile operation failed: {message}")]
    ProfileError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}
