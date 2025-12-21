//! Preference caching module
//!
//! This module provides caching for detected user preferences to avoid
//! re-analyzing the same project repeatedly. Cached preferences have
//! a TTL (time-to-live) and are stored in the XDG cache directory.

use super::UserPreferences;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, trace, warn};

/// Default TTL for cached preferences (1 hour)
const DEFAULT_TTL_HOURS: i64 = 1;

/// Maximum number of cached entries
const MAX_CACHE_ENTRIES: usize = 100;

/// Cache entry with TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cached preferences
    pub preferences: UserPreferences,

    /// When this entry was cached
    pub cached_at: DateTime<Utc>,

    /// Time-to-live for this entry
    pub ttl_hours: i64,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(preferences: UserPreferences) -> Self {
        Self {
            preferences,
            cached_at: Utc::now(),
            ttl_hours: DEFAULT_TTL_HOURS,
        }
    }

    /// Create a new cache entry with custom TTL
    pub fn with_ttl(preferences: UserPreferences, ttl_hours: i64) -> Self {
        Self {
            preferences,
            cached_at: Utc::now(),
            ttl_hours,
        }
    }

    /// Check if the cache entry is still valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        let expiry = self.cached_at + Duration::hours(self.ttl_hours);
        now < expiry
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl_secs(&self) -> i64 {
        let now = Utc::now();
        let expiry = self.cached_at + Duration::hours(self.ttl_hours);
        (expiry - now).num_seconds().max(0)
    }
}

/// Preference cache manager
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreferenceCache {
    /// Cache entries by key (typically project path)
    entries: HashMap<String, CacheEntry>,

    /// Maximum number of entries
    max_entries: usize,

    /// Default TTL for new entries
    default_ttl_hours: i64,
}

impl PreferenceCache {
    /// Create a new cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: MAX_CACHE_ENTRIES,
            default_ttl_hours: DEFAULT_TTL_HOURS,
        }
    }

    /// Get the cache file path
    fn cache_path() -> PathBuf {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("caro");

        cache_dir.join("preferences_cache.json")
    }

    /// Load preferences from cache
    pub async fn load(key: &str) -> Option<CacheEntry> {
        let cache_path = Self::cache_path();

        if !cache_path.exists() {
            trace!("Cache file doesn't exist");
            return None;
        }

        let cache = Self::load_from_file(&cache_path)?;
        cache.entries.get(key).cloned()
    }

    /// Save preferences to cache
    pub async fn save(key: &str, preferences: &UserPreferences) -> Result<(), std::io::Error> {
        let cache_path = Self::cache_path();

        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load existing cache or create new
        let mut cache = Self::load_from_file(&cache_path).unwrap_or_else(Self::new);

        // Add new entry
        let entry = CacheEntry::new(preferences.clone());
        cache.entries.insert(key.to_string(), entry);

        // Cleanup expired entries and enforce size limit
        cache.cleanup();

        // Save to file
        let json = serde_json::to_string_pretty(&cache).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        fs::write(&cache_path, json)?;
        debug!("Saved preferences to cache: {}", key);

        Ok(())
    }

    /// Load cache from file
    fn load_from_file(path: &PathBuf) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Cleanup expired entries and enforce size limit
    fn cleanup(&mut self) {
        // Remove expired entries
        self.entries.retain(|_, entry| entry.is_valid());

        // If still over limit, remove oldest entries
        while self.entries.len() > self.max_entries {
            // Find oldest entry
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.cached_at)
                .map(|(k, _)| k.clone());

            if let Some(key) = oldest {
                self.entries.remove(&key);
            } else {
                break;
            }
        }
    }

    /// Clear the entire cache
    pub fn clear() -> Result<(), std::io::Error> {
        let cache_path = Self::cache_path();
        if cache_path.exists() {
            fs::remove_file(cache_path)?;
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let valid_count = self.entries.values().filter(|e| e.is_valid()).count();
        let expired_count = self.entries.len() - valid_count;

        CacheStats {
            total_entries: self.entries.len(),
            valid_entries: valid_count,
            expired_entries: expired_count,
            max_entries: self.max_entries,
        }
    }

    /// Invalidate a specific cache entry
    pub async fn invalidate(key: &str) -> Result<(), std::io::Error> {
        let cache_path = Self::cache_path();

        if !cache_path.exists() {
            return Ok(());
        }

        if let Some(mut cache) = Self::load_from_file(&cache_path) {
            cache.entries.remove(key);

            let json = serde_json::to_string_pretty(&cache).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?;

            fs::write(&cache_path, json)?;
            debug!("Invalidated cache for: {}", key);
        }

        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: usize,

    /// Number of valid (non-expired) entries
    pub valid_entries: usize,

    /// Number of expired entries
    pub expired_entries: usize,

    /// Maximum allowed entries
    pub max_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ShellType;
    use crate::preferences::{ProjectContext, ShellProfile};
    use std::path::PathBuf;

    fn create_test_preferences() -> UserPreferences {
        UserPreferences {
            project: ProjectContext::default(),
            shell: ShellProfile::empty(ShellType::Bash),
            detected_at: Utc::now(),
            cache_key: "/test/project".to_string(),
        }
    }

    #[test]
    fn test_cache_entry_validity() {
        let prefs = create_test_preferences();
        let entry = CacheEntry::new(prefs);

        assert!(entry.is_valid());
        assert!(entry.remaining_ttl_secs() > 0);
    }

    #[test]
    fn test_cache_entry_expired() {
        let prefs = create_test_preferences();
        let mut entry = CacheEntry::new(prefs);

        // Set cached_at to 2 hours ago
        entry.cached_at = Utc::now() - Duration::hours(2);
        entry.ttl_hours = 1;

        assert!(!entry.is_valid());
        assert_eq!(entry.remaining_ttl_secs(), 0);
    }

    #[test]
    fn test_cache_cleanup() {
        let mut cache = PreferenceCache::new();
        cache.max_entries = 2;

        // Add 3 entries with valid (non-expired) TTLs but different ages
        for i in 0..3 {
            let mut prefs = create_test_preferences();
            prefs.cache_key = format!("/test/{}", i);
            let mut entry = CacheEntry::new(prefs);
            // Make older entries have earlier timestamps but still valid
            // All entries are less than 1 hour old (default TTL)
            entry.cached_at = Utc::now() - Duration::minutes((30 - i * 10) as i64);
            cache.entries.insert(format!("/test/{}", i), entry);
        }

        cache.cleanup();

        // Should only have 2 entries (the newest ones)
        assert_eq!(cache.entries.len(), 2);
        assert!(!cache.entries.contains_key("/test/0")); // Oldest removed
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = PreferenceCache::new();

        // Add valid entry
        let prefs = create_test_preferences();
        cache
            .entries
            .insert("/valid".to_string(), CacheEntry::new(prefs.clone()));

        // Add expired entry
        let mut expired = CacheEntry::new(prefs);
        expired.cached_at = Utc::now() - Duration::hours(2);
        expired.ttl_hours = 1;
        cache.entries.insert("/expired".to_string(), expired);

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.valid_entries, 1);
        assert_eq!(stats.expired_entries, 1);
    }
}
