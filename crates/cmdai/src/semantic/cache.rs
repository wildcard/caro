//! Local Embedding Cache Implementation
//!
//! Provides privacy-preserving embedding storage and retrieval with automatic
//! cleanup and performance optimization for semantic search operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Local embedding cache with disk persistence and cleanup policies
#[derive(Debug)]
pub struct LocalEmbeddingCache {
    /// Directory for cache storage
    pub cache_directory: PathBuf,
    /// In-memory cache for fast access
    pub embeddings: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Cache metadata for tracking
    pub metadata: Arc<RwLock<EmbeddingMetadata>>,
    /// Cleanup policy configuration
    pub cleanup_policy: CacheCleanupPolicy,
}

/// Individual cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Content hash (SHA-256 of original text)
    pub content_hash: String,
    /// Embedding vector
    pub embedding_vector: Vec<f32>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Number of times accessed
    pub access_count: u64,
    /// Last access timestamp
    pub last_accessed: DateTime<Utc>,
    /// Size in bytes
    pub size_bytes: usize,
    /// Original content length for validation
    pub content_length: usize,
}

/// Cache metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    /// Cache format version
    pub version: String,
    /// Embedding model used
    pub model_name: String,
    /// Vector dimensions
    pub dimensions: usize,
    /// Total entries in cache
    pub total_entries: usize,
    /// Total cache size in bytes
    pub total_size_bytes: u64,
    /// Cache creation time
    pub created_at: DateTime<Utc>,
    /// Last cleanup time
    pub last_cleanup: Option<DateTime<Utc>>,
    /// Cache hit rate (access count / lookup count)
    pub hit_rate: f64,
    /// Total lookups performed
    pub total_lookups: u64,
    /// Total cache hits
    pub total_hits: u64,
}

/// Cache cleanup policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheCleanupPolicy {
    /// Maximum age for entries (days)
    pub max_age_days: u32,
    /// Maximum cache size (MB)
    pub max_size_mb: u64,
    /// Preserve frequently accessed entries
    pub preserve_frequent: bool,
    /// Minimum access count to preserve
    pub min_access_count: u32,
    /// Enable automatic cleanup
    pub auto_cleanup: bool,
    /// Cleanup interval (hours)
    pub cleanup_interval_hours: u32,
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub entry_count: usize,
    pub total_size_bytes: u64,
    pub hit_rate: f64,
    pub average_access_count: f64,
    pub oldest_entry_age_days: Option<u32>,
    pub newest_entry_age_days: Option<u32>,
    pub cache_efficiency: f64,
}

impl LocalEmbeddingCache {
    /// Create a new embedding cache with specified directory and policy
    pub fn new(
        cache_directory: PathBuf,
        cleanup_policy: CacheCleanupPolicy,
    ) -> Result<Self, EmbeddingCacheError> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_directory).map_err(|e| {
            EmbeddingCacheError::DirectoryCreationFailed {
                path: cache_directory.clone(),
                error: e.to_string(),
            }
        })?;

        // Initialize metadata
        let metadata = EmbeddingMetadata {
            version: "1.0.0".to_string(),
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            dimensions: 384,
            total_entries: 0,
            total_size_bytes: 0,
            created_at: Utc::now(),
            last_cleanup: None,
            hit_rate: 0.0,
            total_lookups: 0,
            total_hits: 0,
        };

        let cache = Self {
            cache_directory: cache_directory.clone(),
            embeddings: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(metadata)),
            cleanup_policy,
        };

        // Load existing cache entries
        cache.load_from_disk()?;

        info!(
            cache_dir = %cache_directory.display(),
            entries = cache.get_entry_count(),
            "LocalEmbeddingCache initialized"
        );

        Ok(cache)
    }

    /// Store an embedding in the cache
    pub async fn store_embedding(
        &self,
        content: &str,
        embedding: Vec<f32>,
    ) -> Result<String, EmbeddingCacheError> {
        let content_hash = self.generate_content_hash(content);
        let now = Utc::now();

        let embedding_size = std::mem::size_of_val(&embedding);
        let entry = CacheEntry {
            content_hash: content_hash.clone(),
            embedding_vector: embedding,
            created_at: now,
            access_count: 0,
            last_accessed: now,
            size_bytes: embedding_size + content.len(),
            content_length: content.len(),
        };

        // Store in memory cache
        {
            let mut cache = self
                .embeddings
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
            cache.insert(content_hash.clone(), entry.clone());
        }

        // Update metadata
        {
            let mut metadata = self
                .metadata
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
            metadata.total_entries += 1;
            metadata.total_size_bytes += entry.size_bytes as u64;
        }

        // Persist to disk
        self.persist_entry(&entry).await?;

        debug!(
            content_hash = %content_hash,
            content_length = content.len(),
            embedding_dimensions = entry.embedding_vector.len(),
            "Embedding stored in cache"
        );

        Ok(content_hash)
    }

    /// Retrieve an embedding from the cache
    pub async fn get_embedding(
        &self,
        content: &str,
    ) -> Result<Option<Vec<f32>>, EmbeddingCacheError> {
        let content_hash = self.generate_content_hash(content);

        // Update lookup statistics
        {
            let mut metadata = self
                .metadata
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
            metadata.total_lookups += 1;
        }

        let embedding = {
            let mut cache = self
                .embeddings
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;

            if let Some(entry) = cache.get_mut(&content_hash) {
                // Update access statistics
                entry.access_count += 1;
                entry.last_accessed = Utc::now();

                // Update hit statistics
                {
                    let mut metadata = self
                        .metadata
                        .write()
                        .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
                    metadata.total_hits += 1;
                    metadata.hit_rate = metadata.total_hits as f64 / metadata.total_lookups as f64;
                }

                Some(entry.embedding_vector.clone())
            } else {
                None
            }
        };

        if embedding.is_some() {
            debug!(
                content_hash = %content_hash,
                "Cache hit for embedding"
            );
        }

        Ok(embedding)
    }

    /// Check if an embedding exists in the cache
    pub async fn contains(&self, content: &str) -> Result<bool, EmbeddingCacheError> {
        let content_hash = self.generate_content_hash(content);
        let cache = self
            .embeddings
            .read()
            .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
        Ok(cache.contains_key(&content_hash))
    }

    /// Get cache statistics
    pub async fn get_statistics(&self) -> Result<CacheStatistics, EmbeddingCacheError> {
        let cache = self
            .embeddings
            .read()
            .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
        let metadata = self
            .metadata
            .read()
            .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;

        let now = Utc::now();
        let mut oldest_age_days = None;
        let mut newest_age_days = None;
        let mut total_access_count = 0u64;

        for entry in cache.values() {
            let age_days = (now - entry.created_at).num_days() as u32;
            oldest_age_days = Some(oldest_age_days.map_or(age_days, |old: u32| old.max(age_days)));
            newest_age_days = Some(newest_age_days.map_or(age_days, |new: u32| new.min(age_days)));
            total_access_count += entry.access_count;
        }

        let average_access_count = if cache.is_empty() {
            0.0
        } else {
            total_access_count as f64 / cache.len() as f64
        };

        // Calculate cache efficiency (hit rate weighted by access frequency)
        let cache_efficiency = if metadata.total_lookups > 0 {
            metadata.hit_rate * (average_access_count / 10.0).min(1.0)
        } else {
            0.0
        };

        Ok(CacheStatistics {
            entry_count: cache.len(),
            total_size_bytes: metadata.total_size_bytes,
            hit_rate: metadata.hit_rate,
            average_access_count,
            oldest_entry_age_days: oldest_age_days,
            newest_entry_age_days: newest_age_days,
            cache_efficiency,
        })
    }

    /// Perform cache cleanup based on policy
    pub async fn cleanup(&self) -> Result<usize, EmbeddingCacheError> {
        let mut removed_count = 0;
        let now = Utc::now();

        let entries_to_remove: Vec<String> = {
            let cache = self
                .embeddings
                .read()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;

            cache
                .iter()
                .filter_map(|(hash, entry)| {
                    let age_days = (now - entry.created_at).num_days() as u32;

                    // Remove if too old
                    if age_days > self.cleanup_policy.max_age_days {
                        return Some(hash.clone());
                    }

                    // Remove if infrequently accessed (unless preserving frequent)
                    if !self.cleanup_policy.preserve_frequent
                        || entry.access_count < self.cleanup_policy.min_access_count as u64
                    {
                        if age_days > 7 && entry.access_count == 0 {
                            return Some(hash.clone());
                        }
                    }

                    None
                })
                .collect()
        };

        // Remove entries
        for hash in entries_to_remove {
            if let Some(_entry) = self.remove_entry(&hash).await? {
                removed_count += 1;

                // Remove from disk
                let entry_path = self.cache_directory.join(format!("{}.json", hash));
                if entry_path.exists() {
                    std::fs::remove_file(&entry_path)
                        .map_err(|e| EmbeddingCacheError::DiskOperationFailed(e.to_string()))?;
                }
            }
        }

        // Update cleanup timestamp
        {
            let mut metadata = self
                .metadata
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
            metadata.last_cleanup = Some(now);
        }

        if removed_count > 0 {
            info!(removed_entries = removed_count, "Cache cleanup completed");
        }

        Ok(removed_count)
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<(), EmbeddingCacheError> {
        {
            let mut cache = self
                .embeddings
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
            cache.clear();
        }

        {
            let mut metadata = self
                .metadata
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
            metadata.total_entries = 0;
            metadata.total_size_bytes = 0;
            metadata.total_hits = 0;
            metadata.total_lookups = 0;
            metadata.hit_rate = 0.0;
        }

        // Remove all files from cache directory
        if self.cache_directory.exists() {
            for entry in std::fs::read_dir(&self.cache_directory)
                .map_err(|e| EmbeddingCacheError::DiskOperationFailed(e.to_string()))?
            {
                let entry =
                    entry.map_err(|e| EmbeddingCacheError::DiskOperationFailed(e.to_string()))?;
                if entry.path().extension().map_or(false, |ext| ext == "json") {
                    std::fs::remove_file(entry.path())
                        .map_err(|e| EmbeddingCacheError::DiskOperationFailed(e.to_string()))?;
                }
            }
        }

        info!("Cache cleared");
        Ok(())
    }

    /// Get number of entries in cache
    pub fn get_entry_count(&self) -> usize {
        self.embeddings.read().map(|cache| cache.len()).unwrap_or(0)
    }

    /// Get total cache size in bytes
    pub fn get_total_size(&self) -> u64 {
        self.metadata
            .read()
            .map(|m| m.total_size_bytes)
            .unwrap_or(0)
    }

    /// Generate SHA-256 hash of content for cache key
    fn generate_content_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Load cache entries from disk
    fn load_from_disk(&self) -> Result<(), EmbeddingCacheError> {
        if !self.cache_directory.exists() {
            return Ok(());
        }

        let mut loaded_count = 0;
        let mut total_size = 0u64;

        for entry in std::fs::read_dir(&self.cache_directory)
            .map_err(|e| EmbeddingCacheError::DiskOperationFailed(e.to_string()))?
        {
            let entry =
                entry.map_err(|e| EmbeddingCacheError::DiskOperationFailed(e.to_string()))?;

            if entry.path().extension().map_or(false, |ext| ext == "json") {
                if let Ok(cache_entry) = self.load_entry_from_file(&entry.path()) {
                    let hash = cache_entry.content_hash.clone();
                    total_size += cache_entry.size_bytes as u64;

                    let mut cache = self
                        .embeddings
                        .write()
                        .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
                    cache.insert(hash, cache_entry);
                    loaded_count += 1;
                }
            }
        }

        // Update metadata
        {
            let mut metadata = self
                .metadata
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
            metadata.total_entries = loaded_count;
            metadata.total_size_bytes = total_size;
        }

        if loaded_count > 0 {
            info!(
                loaded_entries = loaded_count,
                total_size_mb = total_size / (1024 * 1024),
                "Cache entries loaded from disk"
            );
        }

        Ok(())
    }

    /// Load a single cache entry from file
    fn load_entry_from_file(&self, path: &Path) -> Result<CacheEntry, EmbeddingCacheError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| EmbeddingCacheError::DiskOperationFailed(e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| EmbeddingCacheError::SerializationFailed(e.to_string()))
    }

    /// Persist a cache entry to disk
    async fn persist_entry(&self, entry: &CacheEntry) -> Result<(), EmbeddingCacheError> {
        let file_path = self
            .cache_directory
            .join(format!("{}.json", entry.content_hash));
        let json_content = serde_json::to_string_pretty(entry)
            .map_err(|e| EmbeddingCacheError::SerializationFailed(e.to_string()))?;

        tokio::fs::write(&file_path, json_content)
            .await
            .map_err(|e| EmbeddingCacheError::DiskOperationFailed(e.to_string()))?;

        Ok(())
    }

    /// Remove an entry from cache
    async fn remove_entry(&self, hash: &str) -> Result<Option<CacheEntry>, EmbeddingCacheError> {
        let entry = {
            let mut cache = self
                .embeddings
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
            cache.remove(hash)
        };

        if let Some(ref entry) = entry {
            let mut metadata = self
                .metadata
                .write()
                .map_err(|_| EmbeddingCacheError::LockAcquisitionFailed)?;
            metadata.total_entries = metadata.total_entries.saturating_sub(1);
            metadata.total_size_bytes = metadata
                .total_size_bytes
                .saturating_sub(entry.size_bytes as u64);
        }

        Ok(entry)
    }
}

impl Default for CacheCleanupPolicy {
    fn default() -> Self {
        Self {
            max_age_days: 30,
            max_size_mb: 100,
            preserve_frequent: true,
            min_access_count: 3,
            auto_cleanup: true,
            cleanup_interval_hours: 24,
        }
    }
}

/// Errors that can occur during cache operations
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingCacheError {
    #[error("Failed to create cache directory {path}: {error}")]
    DirectoryCreationFailed { path: PathBuf, error: String },

    #[error("Failed to acquire lock on cache data structure")]
    LockAcquisitionFailed,

    #[error("Disk operation failed: {0}")]
    DiskOperationFailed(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Cache entry not found: {hash}")]
    EntryNotFound { hash: String },

    #[error("Invalid embedding dimensions: expected {expected}, got {actual}")]
    InvalidDimensions { expected: usize, actual: usize },

    #[error("Cache size limit exceeded: {current_mb}MB > {limit_mb}MB")]
    SizeLimitExceeded { current_mb: u64, limit_mb: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cache_creation() {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().join("embeddings");
        let policy = CacheCleanupPolicy::default();

        let cache = LocalEmbeddingCache::new(cache_dir, policy).unwrap();
        assert_eq!(cache.get_entry_count(), 0);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_embedding() {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().join("embeddings");
        let policy = CacheCleanupPolicy::default();

        let cache = LocalEmbeddingCache::new(cache_dir, policy).unwrap();

        let content = "list all files";
        let embedding = vec![0.1, 0.2, 0.3, 0.4];

        let hash = cache
            .store_embedding(content, embedding.clone())
            .await
            .unwrap();
        assert!(!hash.is_empty());

        let retrieved = cache.get_embedding(content).await.unwrap();
        assert_eq!(retrieved, Some(embedding));
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().join("embeddings");
        let policy = CacheCleanupPolicy::default();

        let cache = LocalEmbeddingCache::new(cache_dir, policy).unwrap();

        // Store some embeddings
        cache
            .store_embedding("test1", vec![0.1, 0.2])
            .await
            .unwrap();
        cache
            .store_embedding("test2", vec![0.3, 0.4])
            .await
            .unwrap();

        let stats = cache.get_statistics().await.unwrap();
        assert_eq!(stats.entry_count, 2);
        assert!(stats.total_size_bytes > 0);
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().join("embeddings");
        let mut policy = CacheCleanupPolicy::default();
        policy.max_age_days = 0; // Clean up everything older than 0 days
        policy.preserve_frequent = false; // Don't preserve frequent items

        let cache = LocalEmbeddingCache::new(cache_dir, policy).unwrap();

        // Store an embedding
        cache.store_embedding("test", vec![0.1, 0.2]).await.unwrap();
        assert_eq!(cache.get_entry_count(), 1);

        // Simulate time passing by manually updating the entry's creation time
        {
            let mut cache_map = cache.embeddings.write().unwrap();
            if let Some(entry) = cache_map.get_mut(&cache.generate_content_hash("test")) {
                entry.created_at = Utc::now() - chrono::Duration::days(1);
            }
        }

        // Cleanup should remove the entry
        let removed = cache.cleanup().await.unwrap();
        assert_eq!(removed, 1);
        assert_eq!(cache.get_entry_count(), 0);
    }

    #[tokio::test]
    async fn test_cache_persistence() {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().join("embeddings");
        let policy = CacheCleanupPolicy::default();

        // Create cache and store embedding
        {
            let cache = LocalEmbeddingCache::new(cache_dir.clone(), policy.clone()).unwrap();
            cache
                .store_embedding("persistent_test", vec![0.5, 0.6])
                .await
                .unwrap();
        }

        // Create new cache instance - should load from disk
        let cache2 = LocalEmbeddingCache::new(cache_dir, policy).unwrap();
        let retrieved = cache2.get_embedding("persistent_test").await.unwrap();
        assert_eq!(retrieved, Some(vec![0.5, 0.6]));
    }

    #[test]
    fn test_content_hash_generation() {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().join("embeddings");
        let policy = CacheCleanupPolicy::default();
        let cache = LocalEmbeddingCache::new(cache_dir, policy).unwrap();

        let hash1 = cache.generate_content_hash("test content");
        let hash2 = cache.generate_content_hash("test content");
        let hash3 = cache.generate_content_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64-character hex string
    }
}
