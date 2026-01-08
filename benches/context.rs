use criterion::{criterion_group, criterion_main, Criterion};

// Placeholder for context benchmarks (WP04)
fn benchmark_placeholder(c: &mut Criterion) {
    c.bench_function("context/placeholder", |b| b.iter(|| {}));
}

criterion_group!(benches, benchmark_placeholder);
criterion_main!(benches);
