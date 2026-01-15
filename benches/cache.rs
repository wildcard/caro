//! Cache operation benchmarks
//!
//! Measures performance of model caching operations including:
//! - Cache hit lookup (hash map get)
//! - Model addition (with potential LRU eviction)
//! - Model removal
//! - LRU eviction under memory pressure
//!
//! Expected Performance (from research.md):
//! - cache_get_model_hit: 10-100ns (hash map lookup)
//! - cache_add_model: 1-10μs (hash map insert + possible eviction)
//! - cache_remove_model: 100ns-1μs (hash map remove + file I/O)
//! - cache_lru_eviction_full: <1μs per eviction (iterate + remove)

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;
use tempfile::TempDir;

use caro::cache::ManifestManager;
use caro::models::CachedModel;
use chrono::Utc;

/// Fixture: Create a ManifestManager with pre-populated models
fn setup_cache_with_models(num_models: usize) -> (TempDir, ManifestManager) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut manifest = ManifestManager::new(temp_dir.path().to_path_buf())
        .expect("Failed to create ManifestManager");

    // Add models
    for i in 0..num_models {
        let model = CachedModel {
            model_id: format!("model-{}", i),
            path: temp_dir.path().join(format!("model-{}.bin", i)),
            size_bytes: 1024 * 1024, // 1 MB each
            checksum: format!("checksum-{}", i),
            last_accessed: Utc::now(),
            downloaded_at: Utc::now(),
            version: None,
        };
        manifest
            .add_model(model.model_id.clone(), model)
            .expect("Failed to add model");
    }

    (temp_dir, manifest)
}

/// T007: Benchmark cache hit (has_model lookup)
///
/// Measures the performance of checking if a model exists in the cache.
/// This is a simple hash map lookup operation.
///
/// Expected: 10-100ns (O(1) hash map lookup)
fn bench_cache_get_model_hit(c: &mut Criterion) {
    c.bench_function("cache/get_model_hit", |b| {
        b.iter_batched(
            || setup_cache_with_models(100),
            |(temp_dir, manifest)| {
                // Benchmark the lookup
                let exists = manifest.has_model(black_box("model-50"));
                assert!(exists);
                drop(temp_dir); // Clean up
            },
            BatchSize::SmallInput,
        );
    });
}

/// T008: Benchmark adding a model to cache
///
/// Measures the performance of adding a new model to the cache,
/// including hash map insertion and manifest persistence.
///
/// Expected: 1-10μs (hash map insert + JSON serialization + file write)
fn bench_cache_add_model(c: &mut Criterion) {
    c.bench_function("cache/add_model", |b| {
        b.iter_batched(
            || setup_cache_with_models(50),
            |(temp_dir, mut manifest)| {
                let new_model = CachedModel {
                    model_id: "new-model".to_string(),
                    path: temp_dir.path().join("new-model.bin"),
                    size_bytes: 1024 * 1024,
                    checksum: "new-checksum".to_string(),
                    last_accessed: Utc::now(),
                    downloaded_at: Utc::now(),
                    version: None,
                };

                // Benchmark the addition
                manifest
                    .add_model(black_box(new_model.model_id.clone()), black_box(new_model))
                    .expect("Failed to add model");

                drop(temp_dir); // Clean up
            },
            BatchSize::SmallInput,
        );
    });
}

/// T009: Benchmark removing a model from cache
///
/// Measures the performance of removing a model from the cache,
/// including hash map removal and manifest persistence.
///
/// Expected: 100ns-1μs (hash map remove + JSON serialization + file write)
fn bench_cache_remove_model(c: &mut Criterion) {
    c.bench_function("cache/remove_model", |b| {
        b.iter_batched(
            || setup_cache_with_models(100),
            |(temp_dir, mut manifest)| {
                // Benchmark the removal
                manifest
                    .remove_model(black_box("model-75"))
                    .expect("Failed to remove model");

                drop(temp_dir); // Clean up
            },
            BatchSize::SmallInput,
        );
    });
}

/// T010: Benchmark LRU eviction under memory pressure
///
/// Measures the performance of LRU eviction when the cache exceeds
/// its size limit. This involves finding the least-recently-used model
/// and removing it.
///
/// Expected: <1μs per eviction (linear scan + hash map remove)
fn bench_cache_lru_eviction_full(c: &mut Criterion) {
    c.bench_function("cache/lru_eviction_full", |b| {
        b.iter_batched(
            || {
                // Create a cache with 50 models at 1MB each (50MB total)
                // Set max size to 40MB to trigger eviction
                let temp_dir = TempDir::new().expect("Failed to create temp dir");
                let mut manifest = ManifestManager::new(temp_dir.path().to_path_buf())
                    .expect("Failed to create ManifestManager");

                // Add models to fill the cache
                for i in 0..50 {
                    let model = CachedModel {
                        model_id: format!("model-{}", i),
                        path: temp_dir.path().join(format!("model-{}.bin", i)),
                        size_bytes: 1024 * 1024, // 1 MB
                        checksum: format!("checksum-{}", i),
                        last_accessed: Utc::now(),
                        downloaded_at: Utc::now(),
                        version: None,
                    };
                    manifest
                        .add_model(model.model_id.clone(), model)
                        .expect("Failed to add model");
                }

                (temp_dir, manifest)
            },
            |(temp_dir, mut manifest)| {
                // Add one more model to trigger eviction
                // (manifest.add_model automatically calls cleanup_lru when over limit)
                let overflow_model = CachedModel {
                    model_id: "overflow-model".to_string(),
                    path: temp_dir.path().join("overflow.bin"),
                    size_bytes: 5 * 1024 * 1024, // 5 MB - will trigger eviction
                    checksum: "overflow-checksum".to_string(),
                    last_accessed: Utc::now(),
                    downloaded_at: Utc::now(),
                    version: None,
                };

                // Benchmark the addition which triggers LRU cleanup
                manifest
                    .add_model(overflow_model.model_id.clone(), overflow_model)
                    .expect("Failed to add overflow model");

                drop(temp_dir); // Clean up
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_cache_get_model_hit,
    bench_cache_add_model,
    bench_cache_remove_model,
    bench_cache_lru_eviction_full
);
criterion_main!(benches);
