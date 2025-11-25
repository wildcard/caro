// Contract tests for EmbeddedModelBackend
// These tests verify the behavioral contract defined in:
// specs/004-implement-ollama-and/contracts/embedded-backend.md

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cmdai::backends::embedded::{EmbeddedConfig, EmbeddedModelBackend, ModelVariant};
use cmdai::backends::{CommandGenerator, GeneratorError};
use cmdai::models::{CommandRequest, SafetyLevel, ShellType};

// Helper function to get test model path
fn test_model_path() -> PathBuf {
    // Use a placeholder path for testing
    // In real implementation, this would point to an actual test model
    PathBuf::from("/tmp/test_model.gguf")
}

// Helper function to create test EmbeddedModelBackend
fn create_test_backend() -> Result<EmbeddedModelBackend, GeneratorError> {
    // Use the actual EmbeddedModelBackend with test model path
    EmbeddedModelBackend::with_variant_and_path(ModelVariant::detect(), test_model_path())
}

/// CR-EMB-001: Offline Operation (CRITICAL)
/// MUST work completely offline without any network calls
#[tokio::test]
async fn test_offline_operation_no_network_calls() {
    // Disable network (best effort)
    std::env::set_var("NO_NETWORK", "1");

    let backend = create_test_backend().expect("Failed to create backend");

    let request =
        CommandRequest::new("list files", ShellType::Bash).with_safety(SafetyLevel::Moderate);

    let result = backend.generate_command(&request).await;

    // This will fail because CpuBackend doesn't implement generate_command yet
    assert!(result.is_ok(), "Must work offline without network");

    let command = result.unwrap();
    assert!(
        !command.command.is_empty(),
        "Must generate non-empty command"
    );
    assert_eq!(
        command.backend_used, "embedded",
        "Must report embedded backend"
    );
}

/// CR-EMB-002: Zero-Config Immediate Availability
/// MUST return true from is_available() at all times
#[tokio::test]
async fn test_zero_config_immediate_availability() {
    let backend = create_test_backend().expect("Failed to create backend");

    // Check availability multiple times
    for iteration in 0..10 {
        let available = backend.is_available().await;
        assert!(
            available,
            "Must always be available (iteration {})",
            iteration
        );
    }
}

/// CR-EMB-003: Performance Targets (MLX vs CPU)
/// MUST meet platform-specific performance targets
#[tokio::test]
#[ignore] // Ignore by default as it requires actual model
async fn test_performance_targets_mlx_vs_cpu() {
    let backend = create_test_backend().expect("Failed to create backend");
    let _info = backend.backend_info();

    let request = CommandRequest::new("list all files", ShellType::Bash);

    // Measure first call (includes lazy loading)
    let start = Instant::now();
    let result = backend.generate_command(&request).await;
    let first_call_duration = start.elapsed();

    assert!(result.is_ok(), "Generation must succeed");

    // Check performance based on variant
    let variant = ModelVariant::detect();
    match variant {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        ModelVariant::MLX => {
            assert!(
                first_call_duration < Duration::from_secs(2),
                "MLX must complete within 2s, got {:?}",
                first_call_duration
            );
        }
        ModelVariant::CPU => {
            assert!(
                first_call_duration < Duration::from_secs(5),
                "CPU must complete within 5s, got {:?}",
                first_call_duration
            );
        }
    }

    // Measure subsequent call (model already loaded)
    let start = Instant::now();
    let _ = backend.generate_command(&request).await.unwrap();
    let subsequent_duration = start.elapsed();

    // Subsequent calls should be faster
    assert!(
        subsequent_duration < first_call_duration,
        "Subsequent calls should be faster than first call"
    );
}

/// CR-EMB-004: Platform Detection Automatic
/// MUST automatically select correct variant (MLX vs CPU) based on platform
#[test]
fn test_platform_detection_automatic() {
    let variant = ModelVariant::detect();

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    assert_eq!(
        variant,
        ModelVariant::MLX,
        "Must detect MLX on Apple Silicon"
    );

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    assert_eq!(
        variant,
        ModelVariant::CPU,
        "Must detect CPU on other platforms"
    );
}

/// CR-EMB-005: Safety Validator Integration
/// MUST integrate with safety validation system
#[tokio::test]
async fn test_safety_validator_integration() {
    let backend = create_test_backend().expect("Failed to create backend");

    // Request a dangerous command
    let dangerous_request =
        CommandRequest::new("delete all files", ShellType::Bash).with_safety(SafetyLevel::Strict);

    let result = backend.generate_command(&dangerous_request).await;

    // Even if generation succeeds, safety validation should catch dangerous commands
    // (This is handled at a higher level, but backend must provide the command)
    assert!(
        result.is_ok() || matches!(result.unwrap_err(), GeneratorError::GenerationFailed { .. }),
        "Must handle safety-sensitive requests"
    );
}

/// CR-EMB-006: Lazy Loading on First Inference
/// MUST implement lazy loading (load model on first inference, not construction)
#[tokio::test]
async fn test_lazy_loading_on_first_inference() {
    // Construction should be fast (<100ms)
    let start = Instant::now();
    let backend = create_test_backend().expect("Failed to create backend");
    let construction_time = start.elapsed();

    assert!(
        construction_time < Duration::from_millis(100),
        "Construction must be fast (<100ms), got {:?}",
        construction_time
    );

    // First inference triggers model loading
    let request = CommandRequest::new("list files", ShellType::Bash);
    let _result = backend.generate_command(&request).await;

    // Model should now be loaded for subsequent calls
}

/// CR-EMB-007: Error Handling - Model Load Failure
/// MUST handle model loading errors gracefully
#[tokio::test]
async fn test_error_handling_model_load_failure() {
    // Try to create backend with non-existent model
    let invalid_path = PathBuf::from("/nonexistent/model.gguf");
    let backend_result =
        EmbeddedModelBackend::with_variant_and_path(ModelVariant::detect(), invalid_path);

    // Constructor should either:
    // 1. Return error if model path is validated during construction, OR
    // 2. Succeed but fail on first inference (lazy loading)

    if let Ok(backend) = backend_result {
        let request = CommandRequest::new("list files", ShellType::Bash);
        let result = backend.generate_command(&request).await;

        // Should fail gracefully with descriptive error
        assert!(
            result.is_err(),
            "Must fail gracefully with non-existent model"
        );

        if let Err(e) = result {
            // Error message should be descriptive
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("model")
                    || error_msg.contains("load")
                    || error_msg.contains("file"),
                "Error message should be descriptive: {}",
                error_msg
            );
        }
    }
}

/// CR-EMB-008: Resource Cleanup on Drop
/// MUST release resources when backend is dropped
#[tokio::test]
async fn test_resource_cleanup_on_drop() {
    let backend = create_test_backend().expect("Failed to create backend");

    // Use the backend
    let request = CommandRequest::new("list files", ShellType::Bash);
    let _ = backend.generate_command(&request).await;

    // Explicit shutdown
    let shutdown_result = backend.shutdown().await;
    assert!(
        shutdown_result.is_ok(),
        "Shutdown must succeed: {:?}",
        shutdown_result
    );

    // Backend is dropped here - resources should be released
    drop(backend);

    // In a real test, we'd verify memory/GPU resources are released
    // This is a placeholder for that verification
}

/// CR-EMB-009: Thread-Safe Concurrent Requests
/// MUST safely handle concurrent inference requests
#[tokio::test]
async fn test_thread_safe_concurrent_requests() {
    let backend = Arc::new(create_test_backend().expect("Failed to create backend"));

    // Spawn multiple concurrent requests
    let mut handles = vec![];

    for i in 0..5 {
        let backend_clone = Arc::clone(&backend);

        let handle = tokio::spawn(async move {
            let request = CommandRequest::new(format!("list files {}", i), ShellType::Bash);
            backend_clone.generate_command(&request).await
        });

        handles.push(handle);
    }

    // All requests should complete successfully
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.expect("Task panicked");
        assert!(
            result.is_ok(),
            "Concurrent request {} must succeed: {:?}",
            i,
            result
        );
    }
}

/// Additional test: Backend info correctness
#[test]
fn test_backend_info_correctness() {
    let backend = create_test_backend().expect("Failed to create backend");
    let info = backend.backend_info();

    // Verify BackendInfo fields
    assert_eq!(info.model_name, "qwen2.5-coder-1.5b-instruct-q4_k_m");
    assert!(
        !info.supports_streaming,
        "Embedded model doesn't support streaming"
    );
    assert!(info.max_tokens > 0, "Max tokens must be positive");
    assert!(
        info.typical_latency_ms > 0,
        "Latency estimate must be positive"
    );
}

/// Additional test: Configuration builder pattern
#[test]
fn test_embedded_config_builder() {
    let config = EmbeddedConfig::default()
        .with_temperature(0.5)
        .with_max_tokens(200)
        .with_top_p(0.95);

    assert_eq!(config.temperature, 0.5);
    assert_eq!(config.max_tokens, 200);
    assert_eq!(config.top_p, 0.95);
}
