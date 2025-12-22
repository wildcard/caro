// Smoke tests for CI - fast, deterministic, minimal model downloads

use caro::{
    backends::{embedded::EmbeddedModelBackend, CommandGenerator},
    models::{CommandRequest, ShellType},
};
use serial_test::serial;

/// Setup deterministic environment
fn setup_deterministic() {
    std::env::set_var("OMP_NUM_THREADS", "1");
    std::env::set_var("OPENBLAS_NUM_THREADS", "1");
    std::env::set_var("MKL_NUM_THREADS", "1");
}

#[tokio::test]
#[serial]
async fn smoke_test_model_load() {
    setup_deterministic();
    let backend = EmbeddedModelBackend::new().expect("Should create backend");
    assert!(backend.is_available().await);
}

#[tokio::test]
#[serial]
async fn smoke_test_basic_inference() {
    setup_deterministic();
    let backend = EmbeddedModelBackend::new().expect("Should create backend");
    let request = CommandRequest::new("list files", ShellType::Bash);

    let result = backend.generate_command(&request).await;
    assert!(result.is_ok(), "Inference should succeed");

    let cmd = result.unwrap();
    assert!(!cmd.command.is_empty(), "Command should not be empty");
    assert!(
        cmd.command.len() < 500,
        "Command should be reasonable length"
    );
}

#[tokio::test]
#[serial]
async fn smoke_test_determinism() {
    setup_deterministic();
    let backend = EmbeddedModelBackend::new().expect("Should create backend");
    let request = CommandRequest::new("show current directory", ShellType::Bash);

    let result1 = backend
        .generate_command(&request)
        .await
        .expect("Should succeed");
    let result2 = backend
        .generate_command(&request)
        .await
        .expect("Should succeed");

    // Check token overlap
    let tokens1: Vec<&str> = result1.command.split_whitespace().collect();
    let tokens2: Vec<&str> = result2.command.split_whitespace().collect();

    let common = tokens1.iter().filter(|t| tokens2.contains(t)).count();
    let total = tokens1.len().max(tokens2.len());
    let overlap = if total > 0 {
        (common as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    assert!(
        overlap >= 70.0,
        "Should be reasonably deterministic, got {:.1}% overlap",
        overlap
    );
}

#[tokio::test]
#[serial]
async fn smoke_test_output_structure() {
    setup_deterministic();
    let backend = EmbeddedModelBackend::new().expect("Should create backend");

    let cases = vec![
        ("list files", ShellType::Bash),
        ("show date", ShellType::Bash),
        ("print hello", ShellType::Bash),
    ];

    for (prompt, shell) in cases {
        let request = CommandRequest::new(prompt, shell);
        let result = backend
            .generate_command(&request)
            .await
            .expect("Should succeed");

        assert!(!result.command.is_empty());
    }
}

#[tokio::test]
#[serial]
async fn smoke_test_performance() {
    setup_deterministic();

    let load_start = std::time::Instant::now();
    let backend = EmbeddedModelBackend::new().expect("Should create backend");
    let load_time = load_start.elapsed();

    assert!(
        load_time.as_secs() < 60,
        "Load should take <60s, took {:?}",
        load_time
    );

    let request = CommandRequest::new("list files", ShellType::Bash);

    // Cold inference
    let start = std::time::Instant::now();
    let _result = backend
        .generate_command(&request)
        .await
        .expect("Should succeed");
    let cold_time = start.elapsed();

    assert!(
        cold_time.as_secs() < 30,
        "Cold inference should take <30s, took {:?}",
        cold_time
    );

    // Warm inference
    let start = std::time::Instant::now();
    let _result = backend
        .generate_command(&request)
        .await
        .expect("Should succeed");
    let warm_time = start.elapsed();

    assert!(
        warm_time.as_secs() < 10,
        "Warm inference should take <10s, took {:?}",
        warm_time
    );
}
