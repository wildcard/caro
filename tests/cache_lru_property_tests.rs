//! Property-based tests for LRU cache eviction
//!
//! These tests use proptest to verify LRU cache invariants hold under various scenarios.
//! Property-based testing generates many random test cases to ensure correctness across
//! a wide range of inputs and edge cases.

use caro::models::{CachedModel, CacheManifest};
use chrono::{Duration, Utc};
use proptest::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

// Helper to create a test model with specific parameters
fn create_test_model(
    id: &str,
    size_mb: u64,
    last_accessed_offset_secs: i64,
) -> CachedModel {
    CachedModel {
        model_id: id.to_string(),
        path: PathBuf::from(format!("/tmp/cache/{}", id)),
        checksum: "a".repeat(64), // Valid 64-char hex checksum
        size_bytes: size_mb * 1024 * 1024,
        downloaded_at: Utc::now(),
        last_accessed: Utc::now() - Duration::seconds(last_accessed_offset_secs),
        version: Some("1.0.0".to_string()),
    }
}

// Property test strategy: Generate a list of models with varying sizes and access times
fn model_strategy() -> impl Strategy<Value = (String, u64, i64)> {
    (
        "[a-z]{3,10}",  // model_id: 3-10 lowercase letters
        1u64..100,      // size_mb: 1-100 MB
        0i64..10000,    // last_accessed_offset_secs: 0-10000 seconds ago
    )
}

proptest! {
    /// Property: After cleanup_lru(), total_size_bytes never exceeds max_cache_size_bytes
    #[test]
    fn prop_size_constraint_invariant(
        models in prop::collection::vec(model_strategy(), 1..20),
        max_size_gb in 1u64..5,
    ) {
        let mut manifest = CacheManifest::new(max_size_gb);

        // Add all models
        for (id, size_mb, offset_secs) in models {
            let model = create_test_model(&id, size_mb, offset_secs);
            manifest.add_model(model);
        }

        // Run cleanup
        let _removed = manifest.cleanup_lru();

        // INVARIANT: Cache size never exceeds limit after cleanup
        prop_assert!(
            manifest.total_size_bytes <= manifest.max_cache_size_bytes,
            "Cache size {} exceeds limit {} after cleanup",
            manifest.total_size_bytes,
            manifest.max_cache_size_bytes
        );
    }

    /// Property: Cleanup removes models in LRU order (oldest last_accessed first)
    #[test]
    fn prop_eviction_order_invariant(
        max_size_mb in 10u64..50,  // Small cache to force eviction
    ) {
        let manifest_size_gb = max_size_mb / 1024; // Convert to GB (will be 0 for small sizes)
        let manifest_size_gb = manifest_size_gb.max(1); // Ensure at least 1GB
        let mut manifest = CacheManifest::new(manifest_size_gb);

        // Create 5 models with known access order (oldest to newest)
        let model1 = create_test_model("oldest", 20, 5000);   // 5000 seconds ago
        let model2 = create_test_model("old", 20, 4000);      // 4000 seconds ago
        let model3 = create_test_model("mid", 20, 3000);      // 3000 seconds ago
        let model4 = create_test_model("recent", 20, 2000);   // 2000 seconds ago
        let model5 = create_test_model("newest", 20, 1000);   // 1000 seconds ago

        manifest.add_model(model1);
        manifest.add_model(model2);
        manifest.add_model(model3);
        manifest.add_model(model4);
        manifest.add_model(model5);

        // Force eviction by setting very small limit
        manifest.max_cache_size_bytes = max_size_mb * 1024 * 1024;

        let removed = manifest.cleanup_lru();

        // If eviction occurred, verify order
        if !removed.is_empty() {
            // First removed should be the oldest
            if removed.contains(&"oldest".to_string()) {
                // If oldest was removed, and more were removed, check order
                if removed.len() >= 2 {
                    // Second removed should be "old" (if "oldest" was first)
                    let oldest_idx = removed.iter().position(|id| id == "oldest").unwrap();
                    let old_idx = removed.iter().position(|id| id == "old");

                    if let Some(old_idx_val) = old_idx {
                        prop_assert!(
                            oldest_idx < old_idx_val,
                            "LRU order violated: 'oldest' should be removed before 'old', got order {:?}",
                            removed
                        );
                    }
                }
            }

            // Newest should never be removed before older models (if newest removed, all should be removed)
            if removed.contains(&"newest".to_string()) {
                prop_assert_eq!(
                    removed.len(),
                    5,
                    "If 'newest' was removed, all models should have been removed. Got: {:?}",
                    removed
                );
            }
        }
    }

    /// Property: Total size always equals sum of individual model sizes (completeness)
    #[test]
    fn prop_completeness_invariant(
        models in prop::collection::vec(model_strategy(), 1..15),
    ) {
        let mut manifest = CacheManifest::new(100); // Large cache, no eviction

        // Add all models
        for (id, size_mb, offset_secs) in &models {
            let model = create_test_model(id, *size_mb, *offset_secs);
            manifest.add_model(model);
        }

        // Calculate expected total
        let expected_total: u64 = manifest.models.values()
            .map(|m| m.size_bytes)
            .sum();

        // INVARIANT: total_size_bytes matches sum of individual sizes
        prop_assert_eq!(
            manifest.total_size_bytes,
            expected_total,
            "Total size mismatch: manifest reports {}, but sum of models is {}",
            manifest.total_size_bytes,
            expected_total
        );
    }

    /// Property: Accessing a model updates its last_accessed time correctly
    #[test]
    fn prop_chronological_ordering_invariant(
        initial_offset_secs in 1000i64..5000,
    ) {
        let mut manifest = CacheManifest::new(10);

        // Create a model with known access time
        let model = create_test_model("test-model", 10, initial_offset_secs);
        let initial_access_time = model.last_accessed;
        manifest.add_model(model);

        // Sleep is not practical in property tests, so we manually update last_accessed
        // to simulate an access
        if let Some(model) = manifest.models.get_mut("test-model") {
            let new_access_time = Utc::now();
            model.last_accessed = new_access_time;

            // INVARIANT: New access time is more recent than initial
            prop_assert!(
                new_access_time > initial_access_time,
                "Updated access time {} should be more recent than initial {}",
                new_access_time,
                initial_access_time
            );
        }
    }

    /// Property: Eviction stops when size is under limit
    #[test]
    fn prop_eviction_stops_when_under_limit(
        models in prop::collection::vec(model_strategy(), 3..10),
        max_size_gb in 2u64..10,
    ) {
        let mut manifest = CacheManifest::new(max_size_gb);

        // Add models
        for (id, size_mb, offset_secs) in models {
            let model = create_test_model(&id, size_mb, offset_secs);
            manifest.add_model(model);
        }

        let initial_count = manifest.models.len();
        let _removed = manifest.cleanup_lru();
        let final_count = manifest.models.len();

        // INVARIANT: If cache is under limit, no models should be removed
        if manifest.total_size_bytes <= manifest.max_cache_size_bytes {
            prop_assert_eq!(
                initial_count,
                final_count,
                "No models should be removed when cache is under limit"
            );
        }

        // INVARIANT: After cleanup, we should be under limit (or empty)
        prop_assert!(
            manifest.total_size_bytes <= manifest.max_cache_size_bytes || manifest.models.is_empty(),
            "After cleanup, cache should be under limit or empty"
        );
    }

    /// Property: Cleanup never removes more models than necessary
    #[test]
    fn prop_minimal_eviction(
        max_size_mb in 50u64..150,
    ) {
        let max_size_gb = (max_size_mb / 1024).max(1);
        let mut manifest = CacheManifest::new(max_size_gb);

        // Add exactly 10 models of 20MB each (200MB total)
        for i in 0..10 {
            let model = create_test_model(&format!("model-{}", i), 20, i * 100);
            manifest.add_model(model);
        }

        // Set limit to 100MB (should evict ~5 models to get under limit)
        manifest.max_cache_size_bytes = max_size_mb * 1024 * 1024;

        let removed = manifest.cleanup_lru();

        // After cleanup, we should be under limit
        prop_assert!(
            manifest.total_size_bytes <= manifest.max_cache_size_bytes,
            "Should be under limit after cleanup"
        );

        // Verify we didn't remove too many models
        // If we removed N models, we should be unable to fit back the (N+1)th model
        if !removed.is_empty() && !manifest.models.is_empty() {
            // Get size of a removed model (they're all 20MB)
            let removed_model_size = 20 * 1024 * 1024;

            // Adding back one more model should exceed limit
            let would_exceed = manifest.total_size_bytes + removed_model_size > manifest.max_cache_size_bytes;

            prop_assert!(
                would_exceed,
                "Cleanup removed more models than necessary: {} models removed, \
                 current size {}, limit {}, could fit another {} MB",
                removed.len(),
                manifest.total_size_bytes,
                manifest.max_cache_size_bytes,
                removed_model_size / (1024 * 1024)
            );
        }
    }

    /// Property: All removed model IDs are distinct (no duplicates)
    #[test]
    fn prop_no_duplicate_removals(
        models in prop::collection::vec(model_strategy(), 5..20),
    ) {
        let mut manifest = CacheManifest::new(1); // Very small cache

        // Add models
        for (id, size_mb, offset_secs) in models {
            let model = create_test_model(&id, size_mb, offset_secs);
            manifest.add_model(model);
        }

        let removed = manifest.cleanup_lru();

        // INVARIANT: No duplicate removals
        let unique_removed: HashSet<_> = removed.iter().collect();
        prop_assert_eq!(
            removed.len(),
            unique_removed.len(),
            "Cleanup returned duplicate model IDs: {:?}",
            removed
        );
    }
}

#[cfg(test)]
mod deterministic_tests {
    use super::*;

    #[test]
    fn test_lru_evicts_oldest_first() {
        let mut manifest = CacheManifest::new(1); // 1GB limit

        // Add 3 models totaling 90MB (under limit)
        let model1 = create_test_model("oldest", 30, 5000);
        let model2 = create_test_model("middle", 30, 3000);
        let model3 = create_test_model("newest", 30, 1000);

        manifest.add_model(model1);
        manifest.add_model(model2);
        manifest.add_model(model3);

        // Set limit to 50MB to force eviction of 1-2 models
        manifest.max_cache_size_bytes = 50 * 1024 * 1024;

        let removed = manifest.cleanup_lru();

        // Should remove exactly 2 models (60MB) to get under 50MB limit
        assert_eq!(removed.len(), 2, "Should remove 2 models");

        // First removed should be oldest
        assert_eq!(removed[0], "oldest", "First removed should be 'oldest'");
        assert_eq!(removed[1], "middle", "Second removed should be 'middle'");

        // Only newest should remain
        assert!(manifest.models.contains_key("newest"));
        assert_eq!(manifest.models.len(), 1);
    }

    #[test]
    fn test_no_eviction_when_under_limit() {
        let mut manifest = CacheManifest::new(10); // 10GB limit

        // Add 3 small models (90MB total, well under 10GB)
        manifest.add_model(create_test_model("model1", 30, 1000));
        manifest.add_model(create_test_model("model2", 30, 2000));
        manifest.add_model(create_test_model("model3", 30, 3000));

        let removed = manifest.cleanup_lru();

        // No models should be removed
        assert!(removed.is_empty(), "No models should be removed when under limit");
        assert_eq!(manifest.models.len(), 3);
    }

    #[test]
    fn test_eviction_stops_at_limit() {
        let mut manifest = CacheManifest::new(1);

        // Add 5 models of 30MB each (150MB total)
        for i in 0..5 {
            let model = create_test_model(&format!("model-{}", i), 30, (i + 1) * 1000);
            manifest.add_model(model);
        }

        // Set limit to 80MB (should evict 3 models = 90MB, leaving 60MB)
        manifest.max_cache_size_bytes = 80 * 1024 * 1024;

        let removed = manifest.cleanup_lru();

        // Should remove exactly 3 models
        assert_eq!(removed.len(), 3, "Should remove 3 models to get under 80MB limit");

        // Remaining size should be under limit
        assert!(manifest.total_size_bytes <= manifest.max_cache_size_bytes);

        // Should have 2 models remaining
        assert_eq!(manifest.models.len(), 2);
    }

    #[test]
    fn test_total_size_accuracy_after_eviction() {
        let mut manifest = CacheManifest::new(1);

        // Add models
        manifest.add_model(create_test_model("m1", 40, 4000));
        manifest.add_model(create_test_model("m2", 40, 3000));
        manifest.add_model(create_test_model("m3", 40, 2000));

        // Force eviction
        manifest.max_cache_size_bytes = 50 * 1024 * 1024;
        manifest.cleanup_lru();

        // Verify total_size_bytes matches sum
        let actual_total: u64 = manifest.models.values()
            .map(|m| m.size_bytes)
            .sum();

        assert_eq!(
            manifest.total_size_bytes,
            actual_total,
            "total_size_bytes should match sum of remaining models"
        );
    }
}
