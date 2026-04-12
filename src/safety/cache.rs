//! LRU + TTL cache for safety validation decisions
//!
//! # OES Inspiration
//!
//! OpenEndpointSecurity uses a hash-table cache for authorization decisions,
//! keyed by event type and process token, with LRU eviction and per-client
//! TTL. This avoids re-running expensive authorization checks for repeated
//! identical events (e.g. the same process opening the same file many times).
//!
//! Caro's safety validator runs 52+ regex patterns on every call. During
//! the agent loop's 2-iteration refinement cycle, or when a user retries the
//! same prompt, the same command is often validated multiple times. Caching
//! the decision lets us skip redundant pattern matching.
//!
//! # Design
//!
//! - **Key**: `(hash(command), ShellType, SafetyLevel)` -- a 64-bit hash plus
//!   two small enums. Storing the hash instead of the raw command text avoids
//!   keeping sensitive prompts in memory longer than necessary.
//! - **Value**: The cached `ValidationResult` plus creation timestamp and
//!   last-accessed timestamp for LRU ordering.
//! - **Eviction**: On insert when at capacity, evict the entry with the
//!   oldest `last_accessed` timestamp.
//! - **TTL**: Entries older than `ttl` are treated as cache misses and
//!   lazily removed on lookup.
//! - **Thread safety**: Wrapped in `Mutex` for interior mutability since
//!   `SafetyValidator::validate_command` takes `&self`.
//!
//! # Fail-safe
//!
//! The cache is a pure optimization: if caching is disabled or the cache
//! is full and all entries are fresh, validation falls through to the
//! normal pattern-matching path. Cache misses never cause a command to be
//! blocked -- they only cause the slow path to run.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::models::{SafetyLevel, ShellType};
use crate::safety::ValidationResult;

/// Key for the safety decision cache
///
/// Uses a 64-bit hash of the command instead of the raw string to reduce
/// memory pressure and avoid keeping command text resident in the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey {
    command_hash: u64,
    shell: ShellType,
    safety_level: SafetyLevel,
}

impl CacheKey {
    /// Build a key from a command string, shell, and safety level
    pub fn new(command: &str, shell: ShellType, safety_level: SafetyLevel) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        command.hash(&mut hasher);
        Self {
            command_hash: hasher.finish(),
            shell,
            safety_level,
        }
    }
}

/// A cached validation decision with metadata for LRU + TTL management
#[derive(Debug, Clone)]
struct CacheEntry {
    result: ValidationResult,
    created_at: Instant,
    last_accessed: Instant,
}

/// LRU + TTL cache for safety validation decisions
pub struct SafetyDecisionCache {
    entries: Mutex<HashMap<CacheKey, CacheEntry>>,
    capacity: usize,
    ttl: Duration,
}

impl std::fmt::Debug for SafetyDecisionCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SafetyDecisionCache")
            .field("capacity", &self.capacity)
            .field("ttl", &self.ttl)
            .field("size", &self.entries.lock().map(|e| e.len()).unwrap_or(0))
            .finish()
    }
}

impl SafetyDecisionCache {
    /// Create a new cache with the given capacity and TTL
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            entries: Mutex::new(HashMap::with_capacity(capacity)),
            capacity,
            ttl,
        }
    }

    /// Look up a cached decision
    ///
    /// Returns `Some(result)` on cache hit, `None` on miss or expired entry.
    /// Updates the entry's `last_accessed` timestamp on hit (for LRU ordering).
    pub fn get(&self, key: &CacheKey) -> Option<ValidationResult> {
        let mut entries = self.entries.lock().ok()?;
        let entry = entries.get_mut(key)?;
        let now = Instant::now();
        if now.duration_since(entry.created_at) > self.ttl {
            // Expired: remove lazily
            entries.remove(key);
            return None;
        }
        entry.last_accessed = now;
        Some(entry.result.clone())
    }

    /// Insert a decision into the cache, evicting the LRU entry if at capacity
    pub fn insert(&self, key: CacheKey, result: ValidationResult) {
        let Ok(mut entries) = self.entries.lock() else {
            return;
        };
        if entries.len() >= self.capacity && !entries.contains_key(&key) {
            // Evict LRU: find the entry with the oldest last_accessed.
            // For small capacities (~256), linear scan is fine.
            if let Some(lru_key) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(k, _)| *k)
            {
                entries.remove(&lru_key);
            }
        }
        let now = Instant::now();
        entries.insert(
            key,
            CacheEntry {
                result,
                created_at: now,
                last_accessed: now,
            },
        );
    }

    /// Current number of entries (mainly for testing and metrics)
    pub fn len(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    /// Whether the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all entries
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RiskLevel;

    fn make_result(allowed: bool) -> ValidationResult {
        ValidationResult {
            allowed,
            risk_level: if allowed {
                RiskLevel::Safe
            } else {
                RiskLevel::High
            },
            explanation: "test".to_string(),
            warnings: vec![],
            matched_patterns: vec![],
            confidence_score: 1.0,
        }
    }

    #[test]
    fn cache_hit_returns_stored_result() {
        let cache = SafetyDecisionCache::new(16, Duration::from_secs(60));
        let key = CacheKey::new("ls -la", ShellType::Bash, SafetyLevel::Moderate);
        cache.insert(key, make_result(true));
        let r = cache.get(&key).unwrap();
        assert!(r.allowed);
    }

    #[test]
    fn cache_miss_returns_none() {
        let cache = SafetyDecisionCache::new(16, Duration::from_secs(60));
        let key = CacheKey::new("ls", ShellType::Bash, SafetyLevel::Moderate);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn different_commands_are_different_keys() {
        let cache = SafetyDecisionCache::new(16, Duration::from_secs(60));
        let k1 = CacheKey::new("ls", ShellType::Bash, SafetyLevel::Moderate);
        let k2 = CacheKey::new("pwd", ShellType::Bash, SafetyLevel::Moderate);
        cache.insert(k1, make_result(true));
        assert!(cache.get(&k2).is_none());
    }

    #[test]
    fn different_shells_are_different_keys() {
        let cache = SafetyDecisionCache::new(16, Duration::from_secs(60));
        let k1 = CacheKey::new("ls", ShellType::Bash, SafetyLevel::Moderate);
        let k2 = CacheKey::new("ls", ShellType::Zsh, SafetyLevel::Moderate);
        cache.insert(k1, make_result(true));
        assert!(cache.get(&k2).is_none());
    }

    #[test]
    fn different_safety_levels_are_different_keys() {
        let cache = SafetyDecisionCache::new(16, Duration::from_secs(60));
        let k1 = CacheKey::new("ls", ShellType::Bash, SafetyLevel::Moderate);
        let k2 = CacheKey::new("ls", ShellType::Bash, SafetyLevel::Strict);
        cache.insert(k1, make_result(true));
        assert!(cache.get(&k2).is_none());
    }

    #[test]
    fn ttl_expiry_removes_entry() {
        let cache = SafetyDecisionCache::new(16, Duration::from_millis(10));
        let key = CacheKey::new("ls", ShellType::Bash, SafetyLevel::Moderate);
        cache.insert(key, make_result(true));
        std::thread::sleep(Duration::from_millis(20));
        assert!(
            cache.get(&key).is_none(),
            "expired entry should be treated as miss"
        );
    }

    #[test]
    fn lru_eviction_removes_oldest() {
        let cache = SafetyDecisionCache::new(2, Duration::from_secs(60));
        let k1 = CacheKey::new("a", ShellType::Bash, SafetyLevel::Moderate);
        let k2 = CacheKey::new("b", ShellType::Bash, SafetyLevel::Moderate);
        let k3 = CacheKey::new("c", ShellType::Bash, SafetyLevel::Moderate);

        cache.insert(k1, make_result(true));
        std::thread::sleep(Duration::from_millis(2));
        cache.insert(k2, make_result(true));
        std::thread::sleep(Duration::from_millis(2));
        // Touch k2 to make k1 the LRU
        let _ = cache.get(&k2);
        std::thread::sleep(Duration::from_millis(2));
        // Insert k3 should evict k1 (oldest last_accessed)
        cache.insert(k3, make_result(true));

        assert!(cache.get(&k1).is_none(), "k1 should have been evicted");
        assert!(cache.get(&k2).is_some(), "k2 should still be present");
        assert!(cache.get(&k3).is_some(), "k3 should be present");
    }

    #[test]
    fn len_tracks_entries() {
        let cache = SafetyDecisionCache::new(16, Duration::from_secs(60));
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        cache.insert(
            CacheKey::new("ls", ShellType::Bash, SafetyLevel::Moderate),
            make_result(true),
        );
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
    }

    #[test]
    fn clear_empties_cache() {
        let cache = SafetyDecisionCache::new(16, Duration::from_secs(60));
        cache.insert(
            CacheKey::new("ls", ShellType::Bash, SafetyLevel::Moderate),
            make_result(true),
        );
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn reinsert_same_key_updates_entry() {
        let cache = SafetyDecisionCache::new(2, Duration::from_secs(60));
        let key = CacheKey::new("ls", ShellType::Bash, SafetyLevel::Moderate);
        cache.insert(key, make_result(true));
        cache.insert(key, make_result(false));
        let r = cache.get(&key).unwrap();
        assert!(!r.allowed, "re-insert should overwrite value");
        assert_eq!(cache.len(), 1, "re-insert should not grow cache");
    }
}
