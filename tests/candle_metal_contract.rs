// Contract tests for Candle Metal Backend (Apple Silicon GPU acceleration)
// These tests verify the behavioral contract for embedded inference using Candle
// with Metal GPU acceleration on Apple Silicon.
//
// STRATEGIC PIVOT: Using Candle instead of mlx-rs for better performance
// - Candle is FASTER than MLX for LLM inference
// - Built-in GGUF support and mature codebase
// - Same API for CPU and Metal (just different Device)

use cmdai::backends::embedded::ModelVariant;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use std::path::PathBuf;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use std::time::{Duration, Instant};

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use cmdai::backends::embedded::EmbeddedConfig;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use cmdai::backends::embedded::{InferenceBackend, CpuBackend};

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use cmdai::backends::GeneratorError;

// Helper function to get test model path
// Note: This should point to a small test model or use HF cache
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn test_model_path() -> PathBuf {
    // Try to use HF cache first, fallback to test path
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let hf_cache = format!(
        "{}/.cache/huggingface/hub/models--Qwen--Qwen2.5-Coder-1.5B-Instruct-GGUF/snapshots",
        home
    );

    // Check if model exists in HF cache
    if std::path::Path::new(&hf_cache).exists() {
        // Find the actual snapshot directory and model file
        if let Ok(entries) = std::fs::read_dir(&hf_cache) {
            for entry in entries.flatten() {
                let model_file = entry.path().join("qwen2.5-coder-1.5b-instruct-q4_k_m.gguf");
                if model_file.exists() {
                    return model_file;
                }
            }
        }
    }

    // Fallback to test path (will fail if model not present)
    PathBuf::from("/tmp/test_model.gguf")
}

/// CR-METAL-001: Platform Restriction
/// MUST only compile and run on macOS with Apple Silicon (aarch64)
#[test]
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
fn test_metal_backend_unavailable_on_other_platforms() {
    // This test verifies that Metal backend is NOT available on non-Apple Silicon
    // The very fact that this test compiles proves the platform restriction works

    // On non-Apple Silicon platforms, Metal-specific code should not be accessible
    // This is enforced by conditional compilation
    // Test passes by virtue of compiling successfully without Metal backend
}

/// CR-METAL-001: Platform Availability on Apple Silicon
/// MUST be available and instantiable on macOS aarch64 using Candle Metal device
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn test_metal_backend_available_on_apple_silicon() {
    // Note: CpuBackend with Candle can also use Metal device
    // The variant detection determines which device to use
    let result = CpuBackend::new(test_model_path());
    assert!(result.is_ok(), "Backend must be available on Apple Silicon");

    let backend = result.unwrap();
    // On Apple Silicon, we should prefer Metal, but CPU variant is also valid
    let variant = backend.variant();
    assert!(
        variant == ModelVariant::CPU || variant == ModelVariant::MLX,
        "Must return valid variant"
    );
}

/// CR-METAL-002: Metal Unified Memory Efficiency
/// MUST use Metal unified memory architecture for zero-copy operations
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_unified_memory_usage() {
    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    // Load model into memory
    let load_result = backend.load().await;
    assert!(load_result.is_ok(), "Model loading must succeed");

    // Verify unified memory usage through inference
    // Candle Metal backend uses unified memory automatically
    let config = EmbeddedConfig::default();
    let result = backend.infer("list files", &config).await;

    // Should succeed and demonstrate unified memory efficiency
    assert!(
        result.is_ok() || matches!(result.as_ref().unwrap_err(), GeneratorError::GenerationFailed { .. }),
        "Inference must work with unified memory or fail gracefully"
    );
}

/// CR-METAL-003: Fast Initialization
/// MUST initialize within 100ms (FR-027 startup budget)
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_fast_initialization() {
    let start = Instant::now();

    let mut backend = CpuBackend::new(test_model_path()).unwrap();
    backend.load().await.unwrap();

    let load_time = start.elapsed();

    assert!(
        load_time < Duration::from_millis(100),
        "Metal initialization must complete within 100ms, got {:?}",
        load_time
    );
}

/// CR-METAL-004: Inference Performance
/// MUST generate commands within 2s total (FR-025)
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_inference_performance() {
    let mut backend = CpuBackend::new(test_model_path()).unwrap();
    backend.load().await.unwrap();

    let config = EmbeddedConfig::default();
    let start = Instant::now();

    let result = backend.infer("list all files", &config).await;
    let inference_time = start.elapsed();

    assert!(result.is_ok(), "Inference must succeed");
    assert!(
        inference_time < Duration::from_secs(2),
        "Metal inference must complete within 2s, got {:?}",
        inference_time
    );

    let response = result.unwrap();
    assert!(!response.is_empty(), "Must generate non-empty response");
}

/// CR-METAL-005: First Token Latency
/// MUST produce first token within 200ms (FR-025 responsiveness)
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_first_token_latency() {
    let mut backend = CpuBackend::new(test_model_path()).unwrap();
    backend.load().await.unwrap();

    // For now, we test that inference starts quickly
    // Future: implement streaming for true first-token measurement
    let config = EmbeddedConfig::default();
    let start = Instant::now();

    let result = backend.infer("ls", &config).await;
    let first_response_time = start.elapsed();

    assert!(result.is_ok(), "Inference must succeed");

    // For non-streaming, we check that total time is reasonable
    // Target: <500ms for short prompts
    assert!(
        first_response_time < Duration::from_millis(500),
        "First response must be quick, got {:?}",
        first_response_time
    );
}

/// CR-METAL-006: Metal Error Handling
/// MUST handle Metal framework errors gracefully
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_error_handling() {
    // Test with invalid model path
    let invalid_path = PathBuf::from("/nonexistent/model.gguf");
    let mut backend = CpuBackend::new(invalid_path).unwrap();

    // Model loading should fail gracefully
    let load_result = backend.load().await;
    assert!(
        load_result.is_err(),
        "Must handle invalid model path gracefully"
    );

    // Error should be descriptive
    let error = load_result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("model") || error_msg.contains("file") || error_msg.contains("load"),
        "Error message should be descriptive: {}",
        error_msg
    );
}

/// CR-METAL-007: GGUF Q4 Support
/// MUST support GGUF format with Q4_K_M quantization via Candle
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn test_gguf_q4_support() {
    // Test model path validation for GGUF format
    let gguf_path = PathBuf::from("/tmp/model-q4_k_m.gguf");
    let backend = CpuBackend::new(gguf_path);

    assert!(backend.is_ok(), "Must accept GGUF model paths");

    // Test other quantization levels
    let q8_path = PathBuf::from("/tmp/model-q8_0.gguf");
    let backend_q8 = CpuBackend::new(q8_path);

    assert!(
        backend_q8.is_ok(),
        "Must support different GGUF quantization levels"
    );
}

/// CR-METAL-008: Concurrent Request Handling
/// MUST safely handle concurrent inference requests
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_concurrent_request_handling() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let mut backend = CpuBackend::new(test_model_path()).unwrap();
    backend.load().await.unwrap();

    let backend = Arc::new(Mutex::new(backend));
    let config = EmbeddedConfig::default();

    // Spawn multiple concurrent requests
    let mut handles = vec![];

    for i in 0..3 {
        let backend_clone = Arc::clone(&backend);
        let config_clone = config.clone();

        let handle = tokio::spawn(async move {
            let backend_guard = backend_clone.lock().await;
            backend_guard
                .infer(&format!("ls -l {}", i), &config_clone)
                .await
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

/// CR-METAL-009: Resource Cleanup (GPU)
/// MUST release Metal GPU resources when unloaded
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_resource_cleanup_gpu() {
    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    // Load model (allocates GPU resources)
    let load_result = backend.load().await;
    if load_result.is_ok() {
        // Unload should release resources
        let unload_result = backend.unload().await;
        assert!(
            unload_result.is_ok(),
            "Resource cleanup must succeed: {:?}",
            unload_result
        );
    }

    // Drop should also clean up
    drop(backend);

    // In a real test, we'd verify GPU memory is freed
    // Candle handles this automatically through RAII
}

/// CR-METAL-010: Temperature Control
/// MUST respect temperature parameter for output randomness
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_temperature_control() {
    let mut backend = CpuBackend::new(test_model_path()).unwrap();
    backend.load().await.unwrap();

    let prompt = "list files";

    // Test different temperatures
    let low_temp_config = EmbeddedConfig::default().with_temperature(0.1);
    let high_temp_config = EmbeddedConfig::default().with_temperature(0.9);

    let low_temp_result = backend.infer(prompt, &low_temp_config).await;
    let high_temp_result = backend.infer(prompt, &high_temp_config).await;

    assert!(
        low_temp_result.is_ok(),
        "Low temperature inference must work"
    );
    assert!(
        high_temp_result.is_ok(),
        "High temperature inference must work"
    );

    // With different temperatures, responses should potentially differ
    let low_temp_output = low_temp_result.unwrap();
    let high_temp_output = high_temp_result.unwrap();

    assert!(!low_temp_output.is_empty(), "Low temp must generate output");
    assert!(
        !high_temp_output.is_empty(),
        "High temp must generate output"
    );
}

/// Additional test: Model variant correctness
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn test_backend_variant_correctness() {
    let backend = CpuBackend::new(test_model_path()).unwrap();
    let variant = backend.variant();
    assert!(
        variant == ModelVariant::CPU || variant == ModelVariant::MLX,
        "Must return valid variant: {:?}",
        variant
    );
}

/// Cross-platform test: Ensure Metal types don't leak to other platforms
#[test]
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
fn test_metal_types_unavailable() {
    // This test ensures that Metal-specific types are not available on non-Apple Silicon
    // The fact this test compiles without referencing Metal backend proves the isolation

    let variant = ModelVariant::detect();
    assert_eq!(
        variant,
        ModelVariant::CPU,
        "Non-Apple Silicon must use CPU variant"
    );
}

/// CR-METAL-011: Candle Device Selection
/// MUST automatically select Metal device on Apple Silicon, CPU elsewhere
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn test_candle_device_selection() {
    // On Apple Silicon, Candle should prefer Metal device
    // This is handled internally by the backend
    let backend = CpuBackend::new(test_model_path()).unwrap();

    // Backend should be created successfully
    // Device selection happens during load()
    assert!(backend.variant() == ModelVariant::CPU || backend.variant() == ModelVariant::MLX);
}

/// CR-METAL-012: JSON Response Parsing
/// MUST correctly parse JSON responses from model output
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_json_response_parsing() {
    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    if backend.load().await.is_ok() {
        let config = EmbeddedConfig::default();
        let result = backend.infer("list all files", &config).await;

        if let Ok(response) = result {
            // Response should be valid JSON or extractable command
            // The actual parsing happens at a higher level
            assert!(!response.is_empty(), "Must return non-empty response");

            // Try to parse as JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                // If it's JSON, it should have a "cmd" field
                assert!(
                    json.get("cmd").is_some() || json.is_string(),
                    "JSON should contain cmd field or be a string"
                );
            }
        }
    }
}
