//! Execution context capture benchmarks
//!
//! Expected Performance (from research.md):
//! - context_capture_baseline: <100μs (minimal environment)
//! - context_capture_large_env: <1ms (100+ environment variables)

use criterion::{criterion_group, criterion_main, Criterion};

/// Benchmark context capture with baseline environment
fn bench_context_capture_baseline(c: &mut Criterion) {
    c.bench_function("context/capture_baseline", |b| {
        b.iter(|| {
            // Placeholder: capture current directory, shell type
            let _ = std::env::current_dir();
            let _ = std::env::var("SHELL");
        });
    });
}

/// Benchmark context capture with large environment (100+ vars)
fn bench_context_capture_large_env(c: &mut Criterion) {
    c.bench_function("context/capture_large_env", |b| {
        // Set up large environment
        for i in 0..100 {
            std::env::set_var(format!("TEST_VAR_{}", i), format!("value_{}", i));
        }

        b.iter(|| {
            // Capture all environment variables
            let _ = std::env::vars().collect::<Vec<_>>();
        });

        // Cleanup
        for i in 0..100 {
            std::env::remove_var(format!("TEST_VAR_{}", i));
        }
    });
}

criterion_group!(benches, bench_context_capture_baseline, bench_context_capture_large_env);
criterion_main!(benches);
