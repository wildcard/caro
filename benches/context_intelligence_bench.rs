//! Benchmarks for Context Intelligence Engine
//!
//! Run with: cargo bench --bench context_intelligence_bench

use cmdai::intelligence::{ContextGraph, ContextOptions};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::env;

fn benchmark_full_context_build(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let cwd = env::current_dir().unwrap();

    c.bench_function("context_full_build", |b| {
        b.to_async(&runtime).iter(|| async {
            let result = ContextGraph::build(black_box(&cwd)).await;
            result.unwrap()
        });
    });
}

fn benchmark_context_no_history(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let cwd = env::current_dir().unwrap();

    let options = ContextOptions {
        enable_git: true,
        enable_tools: true,
        enable_history: false, // Disable slowest analyzer
        timeout_ms: 300,
    };

    c.bench_function("context_no_history", |b| {
        b.to_async(&runtime).iter(|| async {
            let result = ContextGraph::build_with_options(black_box(&cwd), options.clone()).await;
            result.unwrap()
        });
    });
}

fn benchmark_context_minimal(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let cwd = env::current_dir().unwrap();

    let options = ContextOptions {
        enable_git: false,
        enable_tools: false,
        enable_history: false,
        timeout_ms: 100,
    };

    c.bench_function("context_minimal", |b| {
        b.to_async(&runtime).iter(|| async {
            let result = ContextGraph::build_with_options(black_box(&cwd), options.clone()).await;
            result.unwrap()
        });
    });
}

fn benchmark_llm_context_generation(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let cwd = env::current_dir().unwrap();

    // Pre-build context
    let context = runtime.block_on(async {
        ContextGraph::build(&cwd).await.unwrap()
    });

    c.bench_function("llm_context_generation", |b| {
        b.iter(|| {
            black_box(context.to_llm_context())
        });
    });
}

criterion_group!(
    benches,
    benchmark_full_context_build,
    benchmark_context_no_history,
    benchmark_context_minimal,
    benchmark_llm_context_generation
);
criterion_main!(benches);
