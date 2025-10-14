//! Semantic search and embedding cache infrastructure
//! 
//! This module provides local semantic understanding capabilities with:
//! - Embedding cache management for performance
//! - Local embedding models (privacy-preserving)
//! - Similarity search and ranking
//! - Query processing and understanding

use std::path::PathBuf;
use anyhow::Result;
use directories::ProjectDirs;

pub mod cache;
pub mod embedding;
pub mod similarity;
pub mod query;
pub mod search;

/// Initialize semantic search infrastructure with cache directories
pub struct SemanticInit;

impl SemanticInit {
    /// Initialize embedding cache directories and configuration
    pub fn initialize() -> Result<SemanticConfig> {
        let config = SemanticConfig::default()?;
        
        // Ensure cache directory exists
        std::fs::create_dir_all(&config.cache_directory)?;
        
        // Initialize metadata file if not exists
        let metadata_path = config.cache_directory.join("metadata.json");
        if !metadata_path.exists() {
            let metadata = CacheMetadata::default();
            let metadata_json = serde_json::to_string_pretty(&metadata)?;
            std::fs::write(&metadata_path, metadata_json)?;
        }
        
        tracing::info!(
            cache_dir = %config.cache_directory.display(),
            model = %config.embedding_model,
            dimensions = config.embedding_dimensions,
            "Semantic search infrastructure initialized"
        );
        
        Ok(config)
    }
}

/// Configuration for semantic search system
#[derive(Debug, Clone)]
pub struct SemanticConfig {
    /// Directory for storing embedding cache
    pub cache_directory: PathBuf,
    
    /// Local embedding model to use
    pub embedding_model: String,
    
    /// Dimensions of embedding vectors
    pub embedding_dimensions: usize,
    
    /// Maximum cache size in MB
    pub max_cache_size_mb: u64,
    
    /// Similarity threshold for search results
    pub similarity_threshold: f64,
    
    /// Maximum results to return
    pub max_results: usize,
}

impl SemanticConfig {
    pub fn default() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "cmdai", "cmdai")
            .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?;
        
        let cache_dir = project_dirs.cache_dir().join("embeddings");
        
        Ok(Self {
            cache_directory: cache_dir,
            embedding_model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            embedding_dimensions: 384,
            max_cache_size_mb: 100,
            similarity_threshold: 0.7,
            max_results: 20,
        })
    }
    
    pub fn development() -> Result<Self> {
        let mut config = Self::default()?;
        config.embedding_model = "sentence-transformers/all-MiniLM-L6-v2".to_string();
        config.embedding_dimensions = 384;
        config.max_cache_size_mb = 50;
        config.similarity_threshold = 0.6;
        Ok(config)
    }
    
    pub fn production() -> Result<Self> {
        let mut config = Self::default()?;
        config.embedding_model = "sentence-transformers/all-MiniLM-L12-v2".to_string();
        config.embedding_dimensions = 384;
        config.max_cache_size_mb = 200;
        config.similarity_threshold = 0.75;
        config.max_results = 50;
        Ok(config)
    }
}

/// Cache metadata for tracking and cleanup
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheMetadata {
    /// Cache format version
    pub version: String,
    
    /// Model used for embeddings
    pub model: String,
    
    /// Embedding dimensions
    pub dimensions: usize,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last access timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    
    /// Number of cached embeddings
    pub entry_count: usize,
    
    /// Cache size in bytes
    pub size_bytes: u64,
}

impl Default for CacheMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            version: "1.0.0".to_string(),
            model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            dimensions: 384,
            created_at: now,
            last_accessed: now,
            entry_count: 0,
            size_bytes: 0,
        }
    }
}

/// Embedding cache cleanup policy
#[derive(Debug, Clone)]
pub struct CacheCleanupPolicy {
    /// Maximum age for cached embeddings (days)
    pub max_age_days: u32,
    
    /// Maximum cache size before cleanup (MB)
    pub max_size_mb: u64,
    
    /// Preserve frequently accessed embeddings
    pub preserve_frequent: bool,
    
    /// Minimum access count to preserve
    pub min_access_count: u32,
}

impl Default for CacheCleanupPolicy {
    fn default() -> Self {
        Self {
            max_age_days: 30,
            max_size_mb: 100,
            preserve_frequent: true,
            min_access_count: 3,
        }
    }
}

/// Initialize semantic search system on application startup
pub fn init_semantic_infrastructure() -> Result<()> {
    let config = SemanticInit::initialize()?;
    
    // Log initialization with performance requirements
    tracing::info!(
        cache_dir = %config.cache_directory.display(),
        model = %config.embedding_model,
        max_cache_mb = config.max_cache_size_mb,
        "Semantic search system ready for production backend integration"
    );
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_semantic_config_initialization() {
        let config = SemanticConfig::default().unwrap();
        assert!(!config.cache_directory.as_os_str().is_empty());
        assert_eq!(config.embedding_dimensions, 384);
        assert!(config.similarity_threshold > 0.0);
    }
    
    #[test]
    fn test_cache_metadata_serialization() {
        let metadata = CacheMetadata::default();
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: CacheMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.dimensions, deserialized.dimensions);
    }
    
    #[tokio::test]
    async fn test_semantic_init_creates_directories() {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        
        // This would be tested with a custom config pointing to temp_dir
        let result = SemanticInit::initialize();
        assert!(result.is_ok());
    }
}