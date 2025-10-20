// Performance benchmarks - THESE MUST FAIL INITIALLY (TDD)
// Benchmarks validate performance requirements across all components

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Import system components for benchmarking
use cmdai::{
    cli::CliApp,
    models::{SafetyLevel, ShellType},
    safety::{SafetyConfig, SafetyValidator},
};

// Benchmark CLI startup time (<100ms requirement)
fn bench_cli_startup(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("cli_startup", |b| {
        b.to_async(&rt).iter(|| async {
            let result = CliApp::new().await;
            // Benchmark the creation time, regardless of success
            black_box(result)
        })
    });
}

// Benchmark safety validation performance (<100ms per command)
fn bench_safety_validation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let test_commands = vec![
        "ls -la",
        "rm -rf /tmp/test",
        "sudo systemctl restart nginx",
        "find . -name '*.log' -delete",
        "docker run --privileged ubuntu",
        "echo hello world",
        "git status",
        "npm install",
        "cargo build --release",
        "python -c \"import os; print(os.getcwd())\"",
    ];

    c.bench_function("safety_validation_single", |b| {
        b.to_async(&rt).iter(|| async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_ok() {
                let v = validator.unwrap();
                for cmd in &test_commands {
                    let result = v.validate_command(cmd, ShellType::Bash).await;
                    black_box(result);
                }
            }
        })
    });
}

// Benchmark batch validation vs individual validation
fn bench_batch_vs_individual(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let commands: Vec<String> = vec![
        "echo test1",
        "echo test2",
        "echo test3",
        "echo test4",
        "echo test5",
        "ls -la",
        "pwd",
        "date",
        "whoami",
        "uptime",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let mut group = c.benchmark_group("validation_comparison");

    // Individual validation
    group.bench_function("individual_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_ok() {
                let v = validator.unwrap();
                for cmd in &commands {
                    let result = v.validate_command(cmd, ShellType::Bash).await;
                    black_box(result);
                }
            }
        })
    });

    // Batch validation
    group.bench_function("batch_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_ok() {
                let v = validator.unwrap();
                let result = v.validate_batch(&commands, ShellType::Bash).await;
                black_box(result);
            }
        })
    });

    group.finish();
}

// Benchmark different safety levels
fn bench_safety_levels(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let test_command = "rm -f *.tmp";
    let safety_levels = vec![
        ("strict", SafetyLevel::Strict),
        ("moderate", SafetyLevel::Moderate),
        ("permissive", SafetyLevel::Permissive),
    ];

    let mut group = c.benchmark_group("safety_levels");

    for (name, level) in safety_levels {
        group.bench_with_input(BenchmarkId::new("validation", name), &level, |b, &level| {
            b.to_async(&rt).iter(|| async {
                let config = match level {
                    SafetyLevel::Strict => SafetyConfig::strict(),
                    SafetyLevel::Moderate => SafetyConfig::moderate(),
                    SafetyLevel::Permissive => SafetyConfig::permissive(),
                };

                let validator = SafetyValidator::new(config);
                if validator.is_ok() {
                    let v = validator.unwrap();
                    let result = v.validate_command(test_command, ShellType::Bash).await;
                    black_box(result);
                }
            })
        });
    }

    group.finish();
}

// Benchmark concurrent validation
fn bench_concurrent_validation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let commands = vec![
        "echo concurrent1",
        "echo concurrent2",
        "echo concurrent3",
        "ls -la",
        "pwd",
        "date",
        "whoami",
        "uptime",
        "df -h",
        "free -m",
    ];

    c.bench_function("concurrent_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_ok() {
                let v = validator.unwrap();

                // Launch concurrent validations
                let handles: Vec<_> = commands
                    .iter()
                    .map(|cmd| {
                        let cmd = cmd.to_string();
                        async move { v.validate_command(&cmd, ShellType::Bash).await }
                    })
                    .collect();

                let results = futures::future::join_all(handles).await;
                black_box(results);
            }
        })
    });
}

// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Test with large command inputs
    let large_commands: Vec<String> = (0..100).map(|i| {
        format!("echo 'This is a large command number {} with lots of text to test memory usage patterns and ensure the system can handle substantial input sizes without degrading performance significantly'", i)
    }).collect();

    c.bench_function("large_command_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_ok() {
                let v = validator.unwrap();
                for cmd in &large_commands {
                    let result = v.validate_command(cmd, ShellType::Bash).await;
                    black_box(result);
                }
            }
        })
    });
}

// Benchmark shell type variations
fn bench_shell_types(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let shells = vec![
        ("bash", ShellType::Bash),
        ("zsh", ShellType::Zsh),
        ("fish", ShellType::Fish),
        ("sh", ShellType::Sh),
        ("powershell", ShellType::PowerShell),
    ];

    let test_command = "echo 'shell performance test'";
    let mut group = c.benchmark_group("shell_types");

    for (name, shell) in shells {
        group.bench_with_input(BenchmarkId::new("validation", name), &shell, |b, &shell| {
            b.to_async(&rt).iter(|| async {
                let validator = SafetyValidator::new(SafetyConfig::moderate());
                if validator.is_ok() {
                    let v = validator.unwrap();
                    let result = v.validate_command(test_command, shell).await;
                    black_box(result);
                }
            })
        });
    }

    group.finish();
}

// Benchmark regex pattern matching performance
fn bench_pattern_matching(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Commands with various complexity levels
    let pattern_test_commands = vec![
        "simple command",
        "rm -rf /path/to/directory/with/many/nested/levels/and/files",
        "find /very/deep/directory/structure -type f -name '*.log' -exec rm {} \\;",
        "curl -X POST https://api.example.com/endpoint -H 'Content-Type: application/json' -d '{\"key\":\"value\"}'",
        "docker run --rm -it -v /host/path:/container/path:ro --network=host --privileged ubuntu:latest bash",
    ];

    c.bench_function("pattern_matching", |b| {
        b.to_async(&rt).iter(|| async {
            let validator = SafetyValidator::new(SafetyConfig::strict());
            if validator.is_ok() {
                let v = validator.unwrap();
                for cmd in &pattern_test_commands {
                    let result = v.validate_command(cmd, ShellType::Bash).await;
                    black_box(result);
                }
            }
        })
    });
}

// Benchmark system resource usage over time
fn bench_sustained_load(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("sustained_validation_load", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let start = std::time::Instant::now();

                let validator = SafetyValidator::new(SafetyConfig::moderate());
                if validator.is_ok() {
                    let v = validator.unwrap();

                    for _ in 0..iters {
                        // Simulate sustained validation load
                        let commands = vec![
                            "ls -la",
                            "pwd",
                            "echo test",
                            "date",
                            "whoami",
                            "rm temp.txt",
                            "mkdir test",
                            "cp file1 file2",
                        ];

                        for cmd in commands {
                            let result = v.validate_command(cmd, ShellType::Bash).await;
                            black_box(result);
                        }
                    }
                }

                start.elapsed()
            })
        })
    });
}

// Group all benchmarks
criterion_group!(
    benches,
    bench_cli_startup,
    bench_safety_validation,
    bench_batch_vs_individual,
    bench_safety_levels,
    bench_concurrent_validation,
    bench_memory_usage,
    bench_shell_types,
    bench_pattern_matching,
    bench_sustained_load
);

criterion_main!(benches);
