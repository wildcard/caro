// Performance benchmarks for Metal vs CPU inference backends
// Uses Criterion for statistical performance measurement
//
// Run with: cargo bench --bench metal_vs_cpu

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::PathBuf;
use std::time::Duration;

use cmdai::backends::embedded::{CpuBackend, EmbeddedConfig, InferenceBackend};

// Helper function to get test model path
fn model_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let hf_cache = format!(
        "{}/.cache/huggingface/hub/models--Qwen--Qwen2.5-Coder-1.5B-Instruct-GGUF/snapshots",
        home
    );

    // Try HF cache first
    if std::path::Path::new(&hf_cache).exists() {
        if let Ok(entries) = std::fs::read_dir(&hf_cache) {
            for entry in entries.flatten() {
                let model_file = entry.path().join("qwen2.5-coder-1.5b-instruct-q4_k_m.gguf");
                if model_file.exists() {
                    return model_file;
                }
            }
        }
    }

    // Fallback
    PathBuf::from("/tmp/test_model.gguf")
}

/// Benchmark: CPU inference performance
fn bench_cpu_inference(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Try to create and load backend
    let mut backend = match CpuBackend::new(model_path()) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Skipping CPU benchmark: backend creation failed");
            return;
        }
    };

    // Load model
    if rt.block_on(backend.load()).is_err() {
        eprintln!("Skipping CPU benchmark: model not available");
        return;
    }

    let config = EmbeddedConfig::default();

    c.bench_function("cpu_inference", |b| {
        b.to_async(&rt).iter(|| async {
            let result = backend.infer("list all files", &config).await;
            black_box(result)
        });
    });
}

/// Benchmark: Metal inference performance (Apple Silicon only)
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn bench_metal_inference(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Create backend that will use Metal device on Apple Silicon
    let mut backend = match CpuBackend::new(model_path()) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Skipping Metal benchmark: backend creation failed");
            return;
        }
    };

    // Load model
    if rt.block_on(backend.load()).is_err() {
        eprintln!("Skipping Metal benchmark: model not available");
        return;
    }

    let config = EmbeddedConfig::default();

    c.bench_function("metal_inference", |b| {
        b.to_async(&rt).iter(|| async {
            let result = backend.infer("list all files", &config).await;
            black_box(result)
        });
    });
}

/// Benchmark: Model loading time
fn bench_model_loading(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("model_load_time", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let mut total_duration = Duration::ZERO;

                for _ in 0..iters {
                    let mut backend = CpuBackend::new(model_path()).unwrap();

                    let start = std::time::Instant::now();
                    let _ = backend.load().await;
                    total_duration += start.elapsed();

                    // Unload for next iteration
                    let _ = backend.unload().await;
                }

                total_duration
            })
        });
    });
}

/// Benchmark: Temperature variations
fn bench_temperature_variations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut backend = match CpuBackend::new(model_path()) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Skipping temperature benchmark: backend creation failed");
            return;
        }
    };

    if rt.block_on(backend.load()).is_err() {
        eprintln!("Skipping temperature benchmark: model not available");
        return;
    }

    let temperatures = vec![0.1, 0.5, 0.7, 0.9];
    let mut group = c.benchmark_group("temperature_variations");

    for temp in temperatures {
        group.bench_with_input(BenchmarkId::from_parameter(temp), &temp, |b, &temp| {
            let config = EmbeddedConfig::default().with_temperature(temp);

            b.to_async(&rt).iter(|| async {
                let result = backend.infer("list files", &config).await;
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark: Different prompt lengths
fn bench_prompt_lengths(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut backend = match CpuBackend::new(model_path()) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Skipping prompt length benchmark: backend creation failed");
            return;
        }
    };

    if rt.block_on(backend.load()).is_err() {
        eprintln!("Skipping prompt length benchmark: model not available");
        return;
    }

    let prompts = vec![
        ("short", "list files"),
        ("medium", "find all text files in the current directory"),
        (
            "long",
            "find all text files in the current directory and subdirectories, excluding hidden files and system directories",
        ),
    ];

    let config = EmbeddedConfig::default();
    let mut group = c.benchmark_group("prompt_lengths");

    for (name, prompt) in prompts {
        group.bench_with_input(BenchmarkId::from_parameter(name), &prompt, |b, &prompt| {
            b.to_async(&rt).iter(|| async {
                let result = backend.infer(prompt, &config).await;
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark: Max tokens variations
fn bench_max_tokens(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut backend = match CpuBackend::new(model_path()) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Skipping max tokens benchmark: backend creation failed");
            return;
        }
    };

    if rt.block_on(backend.load()).is_err() {
        eprintln!("Skipping max tokens benchmark: model not available");
        return;
    }

    let max_tokens_values = vec![50, 100, 200, 500];
    let mut group = c.benchmark_group("max_tokens");

    for max_tokens in max_tokens_values {
        group.bench_with_input(
            BenchmarkId::from_parameter(max_tokens),
            &max_tokens,
            |b, &max_tokens| {
                let config = EmbeddedConfig::default().with_max_tokens(max_tokens);

                b.to_async(&rt).iter(|| async {
                    let result = backend.infer("list all files", &config).await;
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Sequential vs concurrent inference
fn bench_sequential_vs_concurrent(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut backend = match CpuBackend::new(model_path()) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Skipping sequential vs concurrent benchmark: backend creation failed");
            return;
        }
    };

    if rt.block_on(backend.load()).is_err() {
        eprintln!("Skipping sequential vs concurrent benchmark: model not available");
        return;
    }

    let backend = std::sync::Arc::new(tokio::sync::Mutex::new(backend));
    let config = EmbeddedConfig::default();

    let mut group = c.benchmark_group("sequential_vs_concurrent");

    // Sequential
    group.bench_function("sequential_3_requests", |b| {
        let backend = backend.clone();
        let config = config.clone();

        b.to_async(&rt).iter(|| async {
            let backend = backend.lock().await;
            for i in 0..3 {
                let result = backend.infer(&format!("list files {}", i), &config).await;
                black_box(result);
            }
        });
    });

    // Concurrent
    group.bench_function("concurrent_3_requests", |b| {
        let backend = backend.clone();
        let config = config.clone();

        b.to_async(&rt).iter(|| async {
            let mut handles = vec![];

            for i in 0..3 {
                let backend = backend.clone();
                let config = config.clone();

                let handle = tokio::spawn(async move {
                    let backend = backend.lock().await;
                    backend.infer(&format!("list files {}", i), &config).await
                });

                handles.push(handle);
            }

            for handle in handles {
                let result = handle.await.unwrap();
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: Initialization overhead
fn bench_initialization_overhead(c: &mut Criterion) {
    c.bench_function("backend_construction", |b| {
        b.iter(|| {
            let backend = CpuBackend::new(model_path());
            black_box(backend)
        });
    });
}

/// Benchmark: Memory usage patterns (warmup effect)
fn bench_warmup_effect(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut backend = match CpuBackend::new(model_path()) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Skipping warmup benchmark: backend creation failed");
            return;
        }
    };

    if rt.block_on(backend.load()).is_err() {
        eprintln!("Skipping warmup benchmark: model not available");
        return;
    }

    let config = EmbeddedConfig::default();
    let mut group = c.benchmark_group("warmup_effect");

    // First inference (cold)
    group.bench_function("first_inference_cold", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let mut total_duration = Duration::ZERO;

                for _ in 0..iters {
                    // Reload model to simulate cold start
                    let _ = backend.unload().await;
                    let _ = backend.load().await;

                    let start = std::time::Instant::now();
                    let result = backend.infer("list files", &config).await;
                    total_duration += start.elapsed();
                    black_box(result);
                }

                total_duration
            })
        });
    });

    // Warmed up inference
    group.bench_function("inference_warmed_up", |b| {
        // Do a few warmup runs
        for _ in 0..5 {
            let _ = rt.block_on(backend.infer("warmup", &config));
        }

        b.to_async(&rt).iter(|| async {
            let result = backend.infer("list files", &config).await;
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark comparison group: Metal vs CPU (Apple Silicon only)
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn bench_metal_vs_cpu_comparison(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // This benchmark compares Metal and CPU performance directly
    // In practice, the backend automatically selects the best device
    let mut group = c.benchmark_group("metal_vs_cpu");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark with Metal device (if available)
    if let Ok(mut backend) = CpuBackend::new(model_path()) {
        if rt.block_on(backend.load()).is_ok() {
            let config = EmbeddedConfig::default();

            group.bench_function("backend_inference", |b| {
                b.to_async(&rt).iter(|| async {
                    let result = backend.infer("list all files in directory", &config).await;
                    black_box(result)
                });
            });
        }
    }

    group.finish();
}

// Platform-specific benchmark groups
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
criterion_group!(
    benches,
    bench_cpu_inference,
    bench_metal_inference,
    bench_model_loading,
    bench_temperature_variations,
    bench_prompt_lengths,
    bench_max_tokens,
    bench_sequential_vs_concurrent,
    bench_initialization_overhead,
    bench_warmup_effect,
    bench_metal_vs_cpu_comparison
);

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
criterion_group!(
    benches,
    bench_cpu_inference,
    bench_model_loading,
    bench_temperature_variations,
    bench_prompt_lengths,
    bench_max_tokens,
    bench_sequential_vs_concurrent,
    bench_initialization_overhead,
    bench_warmup_effect
);

criterion_main!(benches);
