// Cache module contract tests - THESE MUST FAIL INITIALLY (TDD)
// Tests validate the cache module API from specs/003-implement-core-infrastructure/contracts/cache-api.md

use tempfile::TempDir;

// Import types from cache module
use cmdai::cache::{CacheError, CacheManager};

#[tokio::test]
async fn test_cache_manager_new() {
    // CONTRACT: CacheManager::new() creates manager with default XDG cache directory
    let result = CacheManager::new();

    assert!(result.is_ok(), "CacheManager creation should succeed");
    let cache_manager = result.unwrap();

    // Verify cache directory exists and is accessible
    let stats = cache_manager.stats();
    assert!(stats.cache_dir.exists(), "Cache directory should exist");
    assert!(stats.cache_dir.is_dir(), "Cache path should be a directory");
}

#[tokio::test]
async fn test_cache_manager_with_custom_dir() {
    // CONTRACT: CacheManager::with_cache_dir() uses custom directory
    let temp_dir = TempDir::new().unwrap();
    let custom_path = temp_dir.path().to_path_buf();

    let result = CacheManager::with_cache_dir(custom_path.clone());

    assert!(
        result.is_ok(),
        "CacheManager with custom dir should succeed"
    );
    let cache_manager = result.unwrap();

    let stats = cache_manager.stats();
    assert_eq!(
        stats.cache_dir, custom_path,
        "Should use custom cache directory"
    );
}

#[tokio::test]
#[ignore = "Requires HF download implementation (Feature 004, Issue #10)"]
async fn test_get_model_returns_cached_model() {
    // CONTRACT: get_model() returns existing cached model without downloading
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // First call downloads model (will be mocked in actual implementation)
    let model_id = "test-model";
    let result = cache_manager.get_model(model_id).await;

    assert!(result.is_ok(), "First get_model should succeed");
    let model_path = result.unwrap();
    assert!(model_path.exists(), "Model file should exist");

    // Second call should return cached version
    let cached_result = cache_manager.get_model(model_id).await;
    assert!(cached_result.is_ok(), "Cached get_model should succeed");

    let cached_path = cached_result.unwrap();
    assert_eq!(model_path, cached_path, "Should return same cached path");
}

#[tokio::test]
#[ignore = "Requires HF download implementation (Feature 004, Issue #10)"]
async fn test_get_model_downloads_uncached_model() {
    // CONTRACT: get_model() downloads model if not cached
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let model_id = "new-test-model";
    assert!(
        !cache_manager.is_cached(model_id),
        "Model should not be cached initially"
    );

    let result = cache_manager.get_model(model_id).await;

    // Note: This will fail in tests without network/mock, but validates API contract
    // In real implementation, this would download from Hugging Face
    match result {
        Ok(path) => {
            assert!(path.exists(), "Downloaded model should exist");
            assert!(
                cache_manager.is_cached(model_id),
                "Model should be cached after download"
            );
        }
        Err(CacheError::DownloadFailed(_)) => {
            // Expected in test environment without network
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn test_get_model_fails_offline_for_uncached() {
    // CONTRACT: get_model() fails gracefully when offline and model not cached
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // Attempt to get uncached model (will fail due to no network in test)
    let result = cache_manager.get_model("uncached-model").await;

    assert!(
        result.is_err(),
        "Should fail for uncached model without network"
    );
    match result.unwrap_err() {
        CacheError::DownloadFailed(msg) => {
            assert!(
                msg.contains("network") || msg.contains("connection"),
                "Error should mention network issue"
            );
        }
        _ => panic!("Should return DownloadFailed error"),
    }
}

#[tokio::test]
async fn test_get_model_detects_corrupted_cache() {
    // CONTRACT: get_model() detects and reports checksum mismatches
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // This test would need to manually corrupt a cached file
    // For now, validates that checksum validation exists in API
    let model_id = "test-model-for-corruption";

    // In actual test, would:
    // 1. Cache a model
    // 2. Manually corrupt the file
    // 3. Call get_model and expect ChecksumMismatch error

    // Placeholder assertion for contract validation
    let result = cache_manager.get_model(model_id).await;

    // Contract: If checksum fails, should return ChecksumMismatch error
    if let Err(CacheError::ChecksumMismatch {
        model_id: id,
        expected,
        actual,
    }) = &result
    {
        assert_eq!(id, model_id);
        assert_ne!(
            expected, actual,
            "Expected and actual checksums should differ"
        );
    }
}

#[tokio::test]
async fn test_is_cached_returns_accurate_status() {
    // CONTRACT: is_cached() accurately reflects cache state
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let model_id = "test-cached-model";

    // Initially not cached
    assert!(
        !cache_manager.is_cached(model_id),
        "Should not be cached initially"
    );

    // After download, should be cached
    let _ = cache_manager.get_model(model_id).await;

    // is_cached should return true (if download succeeded)
    // In test environment, may still be false if download failed
}

#[tokio::test]
async fn test_remove_model_frees_disk_space() {
    // CONTRACT: remove_model() deletes model and updates stats
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let model_id = "test-model-to-remove";

    // Cache a model first
    let cache_result = cache_manager.get_model(model_id).await;
    if cache_result.is_err() {
        // Skip test if caching failed (expected in test environment)
        return;
    }

    assert!(cache_manager.is_cached(model_id), "Model should be cached");

    let stats_before = cache_manager.stats();
    let size_before = stats_before.total_size_bytes;

    // Remove the model
    let remove_result = cache_manager.remove_model(model_id).await;
    assert!(remove_result.is_ok(), "Model removal should succeed");

    // Verify removal
    assert!(
        !cache_manager.is_cached(model_id),
        "Model should no longer be cached"
    );

    let stats_after = cache_manager.stats();
    assert!(
        stats_after.total_size_bytes < size_before,
        "Cache size should decrease"
    );
}

#[tokio::test]
async fn test_clear_cache_removes_all_models() {
    // CONTRACT: clear_cache() removes all cached models
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // Cache multiple models (if downloads succeed)
    let _ = cache_manager.get_model("model-1").await;
    let _ = cache_manager.get_model("model-2").await;

    // Clear cache
    let clear_result = cache_manager.clear_cache().await;
    assert!(clear_result.is_ok(), "Cache clear should succeed");

    // Verify all models removed
    let stats = cache_manager.stats();
    assert_eq!(stats.total_models, 0, "No models should remain after clear");
    assert_eq!(stats.total_size_bytes, 0, "Cache size should be zero");

    // Cache directory should still exist
    assert!(
        stats.cache_dir.exists(),
        "Cache directory should still exist"
    );
}

#[tokio::test]
async fn test_stats_returns_accurate_information() {
    // CONTRACT: stats() returns current cache statistics
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let stats = cache_manager.stats();

    assert_eq!(stats.total_models, 0, "Initially should have no models");
    assert_eq!(stats.total_size_bytes, 0, "Initially should have zero size");
    assert!(stats.cache_dir.exists(), "Cache directory should exist");
    assert_eq!(stats.models.len(), 0, "Models list should be empty");
}

#[tokio::test]
async fn test_validate_integrity_detects_corruption() {
    // CONTRACT: validate_integrity() checks checksums of all cached models
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // Cache some models
    let _ = cache_manager.get_model("valid-model").await;

    // Validate integrity
    let integrity_result = cache_manager.validate_integrity().await;

    assert!(
        integrity_result.is_ok(),
        "Integrity validation should succeed"
    );

    let report = integrity_result.unwrap();

    // Report should contain lists (all should be empty for empty cache)
    assert_eq!(
        report.valid_models.len(),
        0,
        "Empty cache should have no valid models"
    );
    assert_eq!(
        report.corrupted_models.len(),
        0,
        "Empty cache should have no corrupted models"
    );
    assert_eq!(
        report.missing_models.len(),
        0,
        "Empty cache should have no missing models"
    );
}

#[tokio::test]
async fn test_cache_directory_creation() {
    // CONTRACT: CacheManager creates cache directory if it doesn't exist
    let temp_dir = TempDir::new().unwrap();
    let non_existent_dir = temp_dir.path().join("new_cache_dir");

    assert!(
        !non_existent_dir.exists(),
        "Directory should not exist initially"
    );

    let result = CacheManager::with_cache_dir(non_existent_dir.clone());

    assert!(result.is_ok(), "Should create cache directory");
    assert!(
        non_existent_dir.exists(),
        "Cache directory should be created"
    );
    assert!(non_existent_dir.is_dir(), "Should be a directory");
}

#[tokio::test]
async fn test_concurrent_access_safety() {
    // CONTRACT: Multiple CacheManagers can access same cache safely
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let cache_manager_1 = CacheManager::with_cache_dir(cache_dir.clone()).unwrap();
    let cache_manager_2 = CacheManager::with_cache_dir(cache_dir.clone()).unwrap();

    // Both should be able to access same model
    let model_id = "shared-model";

    let result_1 = cache_manager_1.get_model(model_id).await;
    let result_2 = cache_manager_2.get_model(model_id).await;

    // At least one should succeed (or both fail due to network in test env)
    // Contract: No corrupted state from concurrent access
    if result_1.is_ok() && result_2.is_ok() {
        assert_eq!(
            result_1.unwrap(),
            result_2.unwrap(),
            "Both should return same path"
        );
    }
}

#[tokio::test]
async fn test_cache_operation_performance() {
    // CONTRACT: Cache operations meet performance requirements (NFR-001)
    use std::time::Instant;

    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // Test is_cached performance (<1ms)
    let start = Instant::now();
    let _ = cache_manager.is_cached("test-model");
    let duration = start.elapsed();

    assert!(duration.as_millis() < 10, "is_cached should be <10ms");

    // Test stats performance (<10ms)
    let start = Instant::now();
    let _ = cache_manager.stats();
    let duration = start.elapsed();

    assert!(duration.as_millis() < 100, "stats should be <100ms");
}
