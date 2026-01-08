//! Config loading and merging benchmarks
//!
//! Expected Performance (from research.md):
//! - config_load_small: 1-5ms (< 1KB files)
//! - config_load_large: 10-50ms (> 100KB files)
//! - config_merge_with_cli: 1-5ms (overlay CLI args)
//! - config_merge_with_env: 1-5ms (overlay env vars)

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::fs;
use tempfile::TempDir;

use caro::config::ConfigManager;

/// T017: Create small config fixture (<1KB)
fn create_small_config() -> String {
    r#"
[settings]
safety_level = "moderate"
default_shell = "bash"
log_level = "info"
cache_max_size_gb = 10
log_rotation_days = 30
"#
    .to_string()
}

/// T017: Create large config fixture (>100KB)
fn create_large_config() -> String {
    let mut config = String::from("[settings]\n");
    // Add lots of comments to make it >100KB
    for i in 0..5000 {
        config.push_str(&format!("# Comment line {}\n", i));
    }
    config.push_str("safety_level = \"moderate\"\n");
    config.push_str("default_shell = \"bash\"\n");
    config.push_str("log_level = \"info\"\n");
    config
}

/// T015: Benchmark loading small config files (<1KB)
fn bench_config_load_small(c: &mut Criterion) {
    c.bench_function("config/load_small", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let config_path = temp_dir.path().join("config.toml");
                fs::write(&config_path, create_small_config()).unwrap();
                (temp_dir, config_path)
            },
            |(temp_dir, config_path)| {
                let manager = ConfigManager::with_config_path(config_path).unwrap();
                let _config = manager.load().unwrap();
                drop(temp_dir);
            },
            BatchSize::SmallInput,
        );
    });
}

/// T016: Benchmark loading large config files (>100KB)
fn bench_config_load_large(c: &mut Criterion) {
    c.bench_function("config/load_large", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let config_path = temp_dir.path().join("config.toml");
                fs::write(&config_path, create_large_config()).unwrap();
                (temp_dir, config_path)
            },
            |(temp_dir, config_path)| {
                let manager = ConfigManager::with_config_path(config_path).unwrap();
                let _config = manager.load().unwrap();
                drop(temp_dir);
            },
            BatchSize::SmallInput,
        );
    });
}

/// T018: Benchmark merging CLI arguments with config
fn bench_config_merge_with_cli(c: &mut Criterion) {
    c.bench_function("config/merge_with_cli", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let config_path = temp_dir.path().join("config.toml");
                fs::write(&config_path, create_small_config()).unwrap();
                (temp_dir, config_path)
            },
            |(temp_dir, config_path)| {
                let manager = ConfigManager::with_config_path(config_path).unwrap();
                let _config = manager
                    .merge_with_cli(Some("high"), Some("zsh"), Some("debug"))
                    .unwrap();
                drop(temp_dir);
            },
            BatchSize::SmallInput,
        );
    });
}

/// T019: Benchmark merging environment variables with config
fn bench_config_merge_with_env(c: &mut Criterion) {
    c.bench_function("config/merge_with_env", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let config_path = temp_dir.path().join("config.toml");
                fs::write(&config_path, create_small_config()).unwrap();
                (temp_dir, config_path)
            },
            |(temp_dir, config_path)| {
                let manager = ConfigManager::with_config_path(config_path).unwrap();
                let _config = manager.merge_with_env().unwrap();
                drop(temp_dir);
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_config_load_small,
    bench_config_load_large,
    bench_config_merge_with_cli,
    bench_config_merge_with_env
);
criterion_main!(benches);
