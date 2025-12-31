//! Knowledge Base caching
//!
//! Manages local storage of the knowledge base in `~/.cache/caro/kb/`.

use super::types::KnowledgeBase;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during KB cache operations
#[derive(Error, Debug)]
pub enum KbCacheError {
    #[error("Cache directory not found")]
    NoCacheDir,

    #[error("Knowledge base not cached")]
    NotCached,

    #[error("Failed to read cache: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to deserialize KB: {0}")]
    DeserializeError(#[from] rmp_serde::decode::Error),

    #[error("Failed to serialize KB: {0}")]
    SerializeError(#[from] rmp_serde::encode::Error),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Corrupt cache data")]
    CorruptData,
}

/// Knowledge Base cache manager
pub struct KbCache {
    /// Cache directory path
    cache_dir: PathBuf,
}

impl KbCache {
    /// Create a new KB cache manager
    pub fn new() -> Option<Self> {
        let cache_dir = dirs::cache_dir()?.join("caro").join("kb");
        Some(Self { cache_dir })
    }

    /// Create with a custom cache directory (for testing)
    pub fn with_dir(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Get the path to the KB file
    pub fn kb_path(&self) -> PathBuf {
        self.cache_dir.join(super::KB_FILENAME)
    }

    /// Get the path to the checksum file
    pub fn checksum_path(&self) -> PathBuf {
        self.cache_dir.join(super::KB_CHECKSUM_FILENAME)
    }

    /// Check if the KB is cached
    pub fn is_cached(&self) -> bool {
        self.kb_path().exists()
    }

    /// Get the cached KB version without loading the full KB
    pub fn cached_version(&self) -> Option<String> {
        let kb = self.load().ok()?;
        Some(kb.version)
    }

    /// Ensure the cache directory exists
    pub fn ensure_cache_dir(&self) -> Result<(), KbCacheError> {
        if !self.cache_dir.exists() {
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Load the knowledge base from cache
    pub fn load(&self) -> Result<KnowledgeBase, KbCacheError> {
        if !self.is_cached() {
            return Err(KbCacheError::NotCached);
        }

        let mut file = fs::File::open(self.kb_path())?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // Verify checksum if available
        if let Ok(expected_checksum) = fs::read_to_string(self.checksum_path()) {
            let actual_checksum = self.calculate_checksum(&data);
            if expected_checksum.trim() != actual_checksum {
                return Err(KbCacheError::ChecksumMismatch {
                    expected: expected_checksum.trim().to_string(),
                    actual: actual_checksum,
                });
            }
        }

        let kb = KnowledgeBase::from_msgpack(&data)?;
        Ok(kb)
    }

    /// Save the knowledge base to cache
    pub fn save(&self, kb: &KnowledgeBase) -> Result<(), KbCacheError> {
        self.ensure_cache_dir()?;

        let data = kb.to_msgpack()?;

        // Calculate and save checksum
        let checksum = self.calculate_checksum(&data);
        let mut checksum_file = fs::File::create(self.checksum_path())?;
        checksum_file.write_all(checksum.as_bytes())?;

        // Save KB data
        let mut kb_file = fs::File::create(self.kb_path())?;
        kb_file.write_all(&data)?;

        Ok(())
    }

    /// Clear the cache
    pub fn clear(&self) -> Result<(), KbCacheError> {
        if self.kb_path().exists() {
            fs::remove_file(self.kb_path())?;
        }
        if self.checksum_path().exists() {
            fs::remove_file(self.checksum_path())?;
        }
        Ok(())
    }

    /// Calculate SHA256 checksum of data
    pub fn calculate_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Check if the cached KB is outdated compared to a new version
    pub fn is_outdated(&self, new_version: &str) -> bool {
        match self.cached_version() {
            Some(cached_version) => {
                // Simple semver comparison
                version_compare(&cached_version, new_version) < 0
            }
            None => true, // No cache means it's outdated
        }
    }

    /// Get cache metadata
    pub fn metadata(&self) -> Option<KbCacheMetadata> {
        if !self.is_cached() {
            return None;
        }

        let kb_path = self.kb_path();
        let metadata = fs::metadata(&kb_path).ok()?;
        let modified = metadata.modified().ok()?;

        let kb = self.load().ok()?;

        Some(KbCacheMetadata {
            version: kb.version,
            size_bytes: metadata.len(),
            modified,
            tip_count: kb.tips.len(),
            alias_count: kb.aliases.len(),
        })
    }
}

impl Default for KbCache {
    fn default() -> Self {
        Self::new().expect("Failed to create KB cache")
    }
}

/// Metadata about the cached KB
#[derive(Debug, Clone)]
pub struct KbCacheMetadata {
    /// KB version
    pub version: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// Last modified time
    pub modified: std::time::SystemTime,
    /// Number of tips
    pub tip_count: usize,
    /// Number of aliases
    pub alias_count: usize,
}

/// Simple semver comparison (returns -1, 0, or 1)
fn version_compare(v1: &str, v2: &str) -> i32 {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };

    let v1_parts = parse(v1);
    let v2_parts = parse(v2);

    for (a, b) in v1_parts.iter().zip(v2_parts.iter()) {
        match a.cmp(b) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => continue,
        }
    }

    // If all compared parts are equal, compare by length
    match v1_parts.len().cmp(&v2_parts.len()) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Equal => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_cache() -> (KbCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache = KbCache::with_dir(temp_dir.path().to_path_buf());
        (cache, temp_dir)
    }

    #[test]
    fn test_cache_not_cached() {
        let (cache, _temp) = create_test_cache();
        assert!(!cache.is_cached());
        assert!(matches!(cache.load(), Err(KbCacheError::NotCached)));
    }

    #[test]
    fn test_save_and_load() {
        let (cache, _temp) = create_test_cache();

        let mut kb = KnowledgeBase::with_version("1.0.0");
        kb.add_tip(super::super::types::KbTip::new("test", "cmd", "message"));

        cache.save(&kb).expect("save");
        assert!(cache.is_cached());

        let loaded = cache.load().expect("load");
        assert_eq!(loaded.version, kb.version);
        assert_eq!(loaded.tip_count(), 1);
    }

    #[test]
    fn test_checksum_verification() {
        let (cache, _temp) = create_test_cache();

        let kb = KnowledgeBase::with_version("1.0.0");
        cache.save(&kb).expect("save");

        // Verify load works with correct checksum
        assert!(cache.load().is_ok());

        // Corrupt the checksum file
        fs::write(cache.checksum_path(), "bad_checksum").unwrap();

        // Load should fail with checksum mismatch
        let result = cache.load();
        assert!(matches!(result, Err(KbCacheError::ChecksumMismatch { .. })));
    }

    #[test]
    fn test_clear_cache() {
        let (cache, _temp) = create_test_cache();

        let kb = KnowledgeBase::with_version("1.0.0");
        cache.save(&kb).expect("save");
        assert!(cache.is_cached());

        cache.clear().expect("clear");
        assert!(!cache.is_cached());
    }

    #[test]
    fn test_is_outdated() {
        let (cache, _temp) = create_test_cache();

        // No cache means outdated
        assert!(cache.is_outdated("1.0.0"));

        // Save version 1.0.0
        let kb = KnowledgeBase::with_version("1.0.0");
        cache.save(&kb).expect("save");

        // Same version is not outdated
        assert!(!cache.is_outdated("1.0.0"));

        // Newer version means outdated
        assert!(cache.is_outdated("1.1.0"));
        assert!(cache.is_outdated("2.0.0"));

        // Older version is not outdated
        assert!(!cache.is_outdated("0.9.0"));
    }

    #[test]
    fn test_version_compare() {
        assert_eq!(version_compare("1.0.0", "1.0.0"), 0);
        assert_eq!(version_compare("1.0.0", "1.1.0"), -1);
        assert_eq!(version_compare("1.1.0", "1.0.0"), 1);
        assert_eq!(version_compare("2.0.0", "1.9.9"), 1);
        assert_eq!(version_compare("1.0", "1.0.0"), -1);
    }

    #[test]
    fn test_metadata() {
        let (cache, _temp) = create_test_cache();

        assert!(cache.metadata().is_none());

        let mut kb = KnowledgeBase::with_version("1.2.3");
        kb.add_tip(super::super::types::KbTip::new("t1", "cmd", "msg"));
        kb.add_alias(super::super::types::KbAlias::new("a", "b"));
        cache.save(&kb).expect("save");

        let meta = cache.metadata().expect("metadata");
        assert_eq!(meta.version, "1.2.3");
        assert_eq!(meta.tip_count, 1);
        assert_eq!(meta.alias_count, 1);
        assert!(meta.size_bytes > 0);
    }
}
