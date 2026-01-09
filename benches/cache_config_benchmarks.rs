//! Benchmarks for cache, config, execution context, and logging operations
//! Validates performance requirements from Issue #9

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::path::PathBuf;

use caro::cache::{CacheManager, CacheStats};
use caro::config::ConfigManager;
use caro::models::{ExecutionContext, Platform, ShellType, UserConfiguration};
use caro::{ConfigError};

// ====================
// Cache Operations Benchmarks
// ====================

fn bench_cache_get_model_hit(c: &mut Criterion) {
    c.bench_function("cache_get_model_hit", |b| {
        b.iter(|| {
            // Setup: Create cache manager
            let cache = CacheManager::new();

            if let Ok(cache) = cache {
                // Benchmark: Check if model is cached (simulates cache hit check)
                let result = cache.is_cached("test-model-small");
                black_box(result)
            } else {
                black_box(false)
            }
        })
    });
}

fn bench_cache_stats(c: &mut Criterion) {
    c.bench_function("cache_stats", |b| {
        b.iter(|| {
            // Setup: Create cache manager
            let cache = CacheManager::new();

            let stats = if let Ok(cache) = cache {
                // Benchmark: Get cache statistics
                cache.stats()
            } else {
                // Return empty stats on error
                CacheStats {
                    cache_dir: PathBuf::from("/tmp"),
                    total_models: 0,
                    total_size_bytes: 0,
                    models: Vec::new(),
                }
            };
            black_box(stats)
        })
    });
}

// ====================
// Config Operations Benchmarks
// ====================

fn bench_config_load(c: &mut Criterion) {
    c.bench_function("config_load", |b| {
        b.iter(|| {
            // Create temporary config manager
            let config_manager = ConfigManager::new();

            let result = if let Ok(manager) = config_manager {
                // Benchmark: Load configuration (returns defaults if not found)
                manager.load()
            } else {
                // Return error on failure
                Err(ConfigError::DirectoryError("Benchmark error".to_string()))
            };
            black_box(result)
        })
    });
}

fn bench_config_merge_with_cli(c: &mut Criterion) {
    c.bench_function("config_merge_with_cli", |b| {
        b.iter(|| {
            // Setup: Create config manager
            let config_manager = ConfigManager::new();

            let result = if let Ok(manager) = config_manager {
                // Benchmark: Merge CLI args into config
                manager.merge_with_cli(
                    Some("strict"),
                    Some("bash"),
                    Some("debug"),
                )
            } else {
                Err(ConfigError::DirectoryError("Benchmark error".to_string()))
            };
            black_box(result)
        })
    });
}

fn bench_config_merge_with_env(c: &mut Criterion) {
    c.bench_function("config_merge_with_env", |b| {
        b.iter(|| {
            // Setup: Set test environment variables
            std::env::set_var("CARO_SAFETY_LEVEL", "moderate");
            std::env::set_var("CARO_DEFAULT_SHELL", "bash");
            std::env::set_var("CARO_LOG_LEVEL", "info");

            // Create config manager
            let config_manager = ConfigManager::new();

            let result = if let Ok(manager) = config_manager {
                // Benchmark: Merge env vars into config
                let result = manager.merge_with_env();

                // Cleanup
                std::env::remove_var("CARO_SAFETY_LEVEL");
                std::env::remove_var("CARO_DEFAULT_SHELL");
                std::env::remove_var("CARO_LOG_LEVEL");

                result
            } else {
                Err(ConfigError::DirectoryError("Benchmark error".to_string()))
            };
            black_box(result)
        })
    });
}

// ====================
// Execution Context Benchmarks
// ====================

fn bench_execution_context_capture_baseline(c: &mut Criterion) {
    c.bench_function("execution_context_capture_baseline", |b| {
        b.iter(|| {
            // Benchmark: Create execution context
            let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            let result = ExecutionContext::new(
                current_dir,
                ShellType::Bash,
                Platform::MacOS,
            );
            black_box(result)
        })
    });
}

fn bench_execution_context_capture_large_env(c: &mut Criterion) {
    c.bench_function("execution_context_capture_large_env", |b| {
        b.iter(|| {
            // Setup: Add 100+ environment variables
            for i in 0..150 {
                let key = format!("TEST_ENV_VAR_{}", i);
                let value = format!("test_value_{}", i);
                std::env::set_var(&key, &value);
            }

            // Benchmark: Create context with large environment
            let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            let result = ExecutionContext::new(
                current_dir,
                ShellType::Bash,
                Platform::MacOS,
            );

            // Cleanup
            for i in 0..150 {
                let key = format!("TEST_ENV_VAR_{}", i);
                std::env::remove_var(&key);
            }

            black_box(result)
        })
    });
}

// ====================
// Logging Benchmarks
// ====================

fn bench_logging_throughput(c: &mut Criterion) {
    c.bench_function("logging_throughput_msgs_per_sec", |b| {
        b.iter(|| {
            // Benchmark: Log 1000 messages (measure throughput)
            for i in 0..1000 {
                tracing::info!("Throughput test message {}", i);
            }
            black_box(())
        })
    });
}

fn bench_logging_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("logging_latency");

    // Benchmark individual log call latency
    group.bench_function("single_log_latency", |b| {
        b.iter(|| {
            tracing::info!("Latency test message");
            black_box(())
        })
    });

    group.finish();
}

fn bench_logging_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("logging_levels");

    // Benchmark different log levels
    for level in ["debug", "info", "warn", "error"].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(level), level, |b, &level| {
            b.iter(|| {
                match level {
                    "debug" => tracing::debug!("Debug level message"),
                    "info" => tracing::info!("Info level message"),
                    "warn" => tracing::warn!("Warn level message"),
                    "error" => tracing::error!("Error level message"),
                    _ => {}
                }
                black_box(())
            })
        });
    }

    group.finish();
}

// Group all benchmarks
criterion_group!(
    cache_benches,
    bench_cache_get_model_hit,
    bench_cache_stats,
);

criterion_group!(
    config_benches,
    bench_config_load,
    bench_config_merge_with_cli,
    bench_config_merge_with_env,
);

criterion_group!(
    execution_benches,
    bench_execution_context_capture_baseline,
    bench_execution_context_capture_large_env,
);

criterion_group!(
    logging_benches,
    bench_logging_throughput,
    bench_logging_latency,
    bench_logging_levels,
);

criterion_main!(
    cache_benches,
    config_benches,
    execution_benches,
    logging_benches
);
