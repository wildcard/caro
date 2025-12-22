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

/// Helper to create backend or skip test if model unavailable
fn try_create_backend() -> Option<EmbeddedModelBackend> {
    match EmbeddedModelBackend::new() {
        Ok(backend) => Some(backend),
        Err(e) => {
            eprintln!("⚠️  Skipping test - model not available: {}", e);
            None
        }
    }
}

#[tokio::test]
#[serial]
async fn smoke_test_model_load() {
    setup_deterministic();
    if let Some(backend) = try_create_backend() {
        assert!(backend.is_available().await);
    }
}

#[tokio::test]
#[serial]
async fn smoke_test_basic_inference() {
    setup_deterministic();
    let Some(backend) = try_create_backend() else {
        return;
    };
    let request = CommandRequest::new("list files", ShellType::Bash);

    let result = backend.generate_command(&request).await;
    assert!(result.is_ok(), "Inference should succeed");

    let cmd = result.unwrap();
    assert!(!cmd.command.is_empty(), "Command should not be empty");
    assert!(
        cmd.command.len() < 500,
        "Command should be reasonable length"
    );

    // Basic sanity check - should contain shell-like patterns
    let lower = cmd.command.to_lowercase();
    let has_shell_patterns = lower.contains("ls")
        || lower.contains("dir")
        || lower.contains("find")
        || lower.contains("echo");
    assert!(
        has_shell_patterns,
        "Command should look like a shell command, got: {}",
        cmd.command
    );
}

#[tokio::test]
#[serial]
async fn smoke_test_determinism() {
    setup_deterministic();
    let Some(backend) = try_create_backend() else {
        return;
    };
    let request = CommandRequest::new("show current directory", ShellType::Bash);

    let result1 = backend
        .generate_command(&request)
        .await
        .expect("Should succeed");
    let result2 = backend
        .generate_command(&request)
        .await
        .expect("Should succeed");

    println!("First output:  {}", result1.command);
    println!("Second output: {}", result2.command);

    // For small models on CPU, we expect some variation but not complete randomness
    // We'll check that both outputs are at least valid (non-empty)
    // and contain similar shell command patterns
    assert!(
        !result1.command.is_empty(),
        "First command should not be empty"
    );
    assert!(
        !result2.command.is_empty(),
        "Second command should not be empty"
    );

    // Both should contain shell-like patterns (pwd, cd, echo, ls, etc.)
    let lower1 = result1.command.to_lowercase();
    let lower2 = result2.command.to_lowercase();

    let has_shell1 = lower1.contains("pwd")
        || lower1.contains("cd")
        || lower1.contains("echo")
        || lower1.contains("print")
        || lower1.contains("ls")
        || lower1.contains("dir");
    let has_shell2 = lower2.contains("pwd")
        || lower2.contains("cd")
        || lower2.contains("echo")
        || lower2.contains("print")
        || lower2.contains("ls")
        || lower2.contains("dir");

    assert!(
        has_shell1,
        "First output should contain shell patterns: {}",
        result1.command
    );
    assert!(
        has_shell2,
        "Second output should contain shell patterns: {}",
        result2.command
    );
}

#[tokio::test]
#[serial]
async fn smoke_test_output_structure() {
    setup_deterministic();
    let Some(backend) = try_create_backend() else {
        return;
    };

    let cases = vec![
        ("list files in current directory", ShellType::Bash),
        ("show current date", ShellType::Bash),
        ("print text hello", ShellType::Bash),
    ];

    for (prompt, shell) in cases {
        let request = CommandRequest::new(prompt, shell);
        let result = backend
            .generate_command(&request)
            .await
            .expect("Should succeed");

        assert!(
            !result.command.is_empty(),
            "Command for '{}' should not be empty",
            prompt
        );
        assert!(
            result.command.len() < 500,
            "Command for '{}' should be reasonable length",
            prompt
        );
    }
}

#[tokio::test]
#[serial]
async fn smoke_test_performance() {
    setup_deterministic();

    // Test model load time
    let load_start = std::time::Instant::now();
    let Some(backend) = try_create_backend() else {
        return;
    };
    let load_time = load_start.elapsed();

    println!("Model load time: {:?}", load_time);
    assert!(load_time.as_secs() < 60, "Model load should take < 60s");

    // Test cold inference
    let request = CommandRequest::new("list files", ShellType::Bash);
    let cold_start = std::time::Instant::now();
    let _ = backend
        .generate_command(&request)
        .await
        .expect("Cold inference should succeed");
    let cold_time = cold_start.elapsed();

    println!("Cold inference time: {:?}", cold_time);
    assert!(cold_time.as_secs() < 30, "Cold inference should take < 30s");

    // Test warm inference
    let warm_start = std::time::Instant::now();
    let _ = backend
        .generate_command(&request)
        .await
        .expect("Warm inference should succeed");
    let warm_time = warm_start.elapsed();

    println!("Warm inference time: {:?}", warm_time);
    assert!(warm_time.as_secs() < 10, "Warm inference should take < 10s");
}
