//! Evaluation result caching for performance optimization.

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Mutex;

/// Cache key for evaluation results
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub test_id: String,
    pub backend: String,
    pub prompt_version: String,
}

/// Cached evaluation result with timestamp
#[derive(Debug, Clone)]
pub struct CachedResult {
    pub result: String,
    pub timestamp: DateTime<Utc>,
}

impl CachedResult {
    pub fn is_fresh(&self) -> bool {
        let age = Utc::now() - self.timestamp;
        age < Duration::hours(24)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f64 / total as f64
    }
}

/// Evaluation result cache
pub struct EvalCache {
    cache: Mutex<HashMap<CacheKey, CachedResult>>,
    stats: Mutex<CacheStats>,
}

impl EvalCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            stats: Mutex::new(CacheStats::default()),
        }
    }

    pub fn put(&self, key: CacheKey, result: String) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(
            key,
            CachedResult {
                result,
                timestamp: Utc::now(),
            },
        );
    }

    pub fn put_with_timestamp(&self, key: CacheKey, cached_result: CachedResult) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, cached_result);
    }

    pub fn get(&self, key: &CacheKey) -> Option<String> {
        let cache = self.cache.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if let Some(cached) = cache.get(key) {
            if cached.is_fresh() {
                stats.hits += 1;
                return Some(cached.result.clone());
            }
        }

        stats.misses += 1;
        None
    }

    pub fn get_stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }
}

impl Default for EvalCache {
    fn default() -> Self {
        Self::new()
    }
}
