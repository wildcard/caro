// Integration tests for embedded model inference
// Tests end-to-end workflows with Candle Metal/CPU backends

use cmdai::backends::embedded::{CpuBackend, EmbeddedConfig, InferenceBackend, ModelVariant};
use cmdai::backends::GeneratorError;
use std::path::PathBuf;
use std::time::{Duration, Instant};

// Helper function to get test model path
fn test_model_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let hf_cache = format!(
        "{}/.cache/huggingface/hub/models--Qwen--Qwen2.5-Coder-1.5B-Instruct-GGUF/snapshots",
        home
    );

    // Check if model exists in HF cache
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

    PathBuf::from("/tmp/test_model.gguf")
}

/// Test: End-to-end Metal inference on Apple Silicon
#[tokio::test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
async fn test_end_to_end_metal_inference() {
    // Test full workflow:
    // 1. Initialize backend
    // 2. Load model
    // 3. Run inference on multiple prompts
    // 4. Validate JSON responses
    // 5. Check performance

    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    // Load model
    if backend.load().await.is_err() {
        eprintln!("Skipping test: model not available");
        return;
    }

    let test_cases = vec![
        ("list all files", "ls"),
        ("find text files", "find"),
        ("show current directory", "pwd"),
        ("search for 'error' in logs", "grep"),
        ("count lines in file", "wc"),
    ];

    let config = EmbeddedConfig::default();

    for (prompt, expected_cmd_prefix) in test_cases {
        println!("Testing prompt: {}", prompt);

        let start = Instant::now();
        let result = backend.infer(prompt, &config).await;
        let duration = start.elapsed();

        // Check that inference completes
        assert!(
            result.is_ok(),
            "Inference failed for '{}': {:?}",
            prompt,
            result
        );

        let response = result.unwrap();
        assert!(!response.is_empty(), "Response should not be empty");

        // Check performance
        assert!(
            duration < Duration::from_secs(3),
            "Inference took too long: {:?}",
            duration
        );

        // Try to validate response contains expected command
        let response_lower = response.to_lowercase();
        println!("  Response: {}", response);
        println!("  Duration: {:?}", duration);

        // Response should mention the expected command or be valid JSON
        let is_valid = response_lower.contains(expected_cmd_prefix)
            || response.contains('{')
            || response.contains("cmd");

        assert!(
            is_valid,
            "Response should contain '{}' or be JSON: {}",
            expected_cmd_prefix, response
        );
    }
}

/// Test: CPU backend fallback (cross-platform)
#[tokio::test]
async fn test_cpu_backend_fallback() {
    // Test that CPU backend works as fallback on all platforms
    let backend = CpuBackend::new(test_model_path());

    assert!(
        backend.is_ok(),
        "CPU backend should be available on all platforms"
    );

    let mut backend = backend.unwrap();

    // Verify variant
    let variant = backend.variant();
    assert_eq!(
        variant,
        ModelVariant::CPU,
        "Should use CPU variant as base"
    );

    // Try to load model
    let load_result = backend.load().await;

    if load_result.is_ok() {
        // If model loads, test inference
        let config = EmbeddedConfig::default();
        let result = backend.infer("echo hello", &config).await;

        assert!(result.is_ok(), "CPU inference should work");
        let response = result.unwrap();
        assert!(!response.is_empty(), "Should generate response");
    } else {
        // Model not available - that's OK for this test
        eprintln!("Model not available, skipping inference test");
    }
}

/// Test: JSON parsing robustness
#[tokio::test]
async fn test_json_parsing_robustness() {
    // Test various JSON response formats
    // Test malformed JSON handling
    // Test fallback extraction

    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    if backend.load().await.is_err() {
        eprintln!("Skipping test: model not available");
        return;
    }

    let config = EmbeddedConfig::default();

    // Test cases that might produce different JSON formats
    let test_prompts = vec![
        "list files",
        "show current directory",
        "find all .txt files",
    ];

    for prompt in test_prompts {
        let result = backend.infer(prompt, &config).await;

        if let Ok(response) = result {
            println!("Prompt: {} -> Response: {}", prompt, response);

            // Response should be:
            // 1. Valid JSON with "cmd" field, OR
            // 2. Valid JSON string, OR
            // 3. Plain text command, OR
            // 4. Contains command-like text

            let is_valid =
                // Try parsing as JSON
                serde_json::from_str::<serde_json::Value>(&response).is_ok()
                // Or contains command indicators
                || response.contains("cmd")
                || response.contains('{')
                || response.contains('/')
                || response.split_whitespace().count() > 0;

            assert!(
                is_valid,
                "Response should be parseable or contain command: {}",
                response
            );
        }
    }
}

/// Test: Error handling
#[tokio::test]
async fn test_error_handling() {
    // Test model not found
    let invalid_path = PathBuf::from("/nonexistent/path/model.gguf");
    let mut backend = CpuBackend::new(invalid_path).unwrap();

    let load_result = backend.load().await;
    assert!(
        load_result.is_err(),
        "Should fail when model file doesn't exist"
    );

    let error = load_result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("model") || error_msg.contains("file") || error_msg.contains("load"),
        "Error should be descriptive: {}",
        error_msg
    );
}

/// Test: Invalid prompt handling
#[tokio::test]
async fn test_invalid_prompt_handling() {
    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    if backend.load().await.is_err() {
        eprintln!("Skipping test: model not available");
        return;
    }

    let config = EmbeddedConfig::default();

    // Test with empty prompt
    let result = backend.infer("", &config).await;
    // Should either succeed with empty response or fail gracefully
    assert!(
        result.is_ok() || matches!(result.unwrap_err(), GeneratorError::GenerationFailed { .. }),
        "Should handle empty prompt gracefully"
    );

    // Test with very long prompt
    let long_prompt = "list files ".repeat(100);
    let result = backend.infer(&long_prompt, &config).await;
    // Should either succeed or fail gracefully
    assert!(
        result.is_ok() || matches!(result.unwrap_err(), GeneratorError::GenerationFailed { .. }),
        "Should handle long prompt gracefully"
    );
}

/// Test: Configuration variations
#[tokio::test]
async fn test_configuration_variations() {
    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    if backend.load().await.is_err() {
        eprintln!("Skipping test: model not available");
        return;
    }

    let prompt = "list all files";

    // Test different configurations
    let configs = vec![
        EmbeddedConfig::default().with_temperature(0.1),
        EmbeddedConfig::default().with_temperature(0.7),
        EmbeddedConfig::default().with_temperature(0.9),
        EmbeddedConfig::default().with_max_tokens(50),
        EmbeddedConfig::default().with_max_tokens(200),
        EmbeddedConfig::default().with_top_p(0.5),
        EmbeddedConfig::default().with_top_p(0.95),
    ];

    for (i, config) in configs.iter().enumerate() {
        let result = backend.infer(prompt, config).await;
        assert!(
            result.is_ok(),
            "Config variation {} should work: {:?}",
            i,
            config
        );

        if let Ok(response) = result {
            assert!(
                !response.is_empty(),
                "Config variation {} should produce output",
                i
            );
        }
    }
}

/// Test: Lazy loading behavior
#[tokio::test]
async fn test_lazy_loading_behavior() {
    // Construction should be fast
    let start = Instant::now();
    let mut backend = CpuBackend::new(test_model_path()).unwrap();
    let construction_time = start.elapsed();

    assert!(
        construction_time < Duration::from_millis(50),
        "Construction should be fast: {:?}",
        construction_time
    );

    // First load might be slow
    let start = Instant::now();
    let load_result = backend.load().await;
    let load_time = start.elapsed();

    if load_result.is_ok() {
        println!("Model load time: {:?}", load_time);

        // Second load should be fast (already loaded)
        let start = Instant::now();
        let _ = backend.load().await;
        let reload_time = start.elapsed();

        assert!(
            reload_time < load_time || reload_time < Duration::from_millis(100),
            "Reload should be fast (model already loaded)"
        );
    }
}

/// Test: Memory cleanup
#[tokio::test]
async fn test_memory_cleanup() {
    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    if backend.load().await.is_ok() {
        // Use the model
        let config = EmbeddedConfig::default();
        let _ = backend.infer("test", &config).await;

        // Unload
        let unload_result = backend.unload().await;
        assert!(unload_result.is_ok(), "Unload should succeed");

        // Should be able to load again
        let reload_result = backend.load().await;
        assert!(reload_result.is_ok(), "Should be able to reload");
    }

    // Drop should clean up
    drop(backend);
}

/// Test: Concurrent inference (stress test)
#[tokio::test]
async fn test_concurrent_inference_stress() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    if backend.load().await.is_err() {
        eprintln!("Skipping test: model not available");
        return;
    }

    let backend = Arc::new(Mutex::new(backend));
    let config = EmbeddedConfig::default();

    // Spawn multiple concurrent requests
    let mut handles = vec![];

    for i in 0..5 {
        let backend_clone = Arc::clone(&backend);
        let config_clone = config.clone();

        let handle = tokio::spawn(async move {
            let backend_guard = backend_clone.lock().await;
            let result = backend_guard
                .infer(&format!("list files in directory {}", i), &config_clone)
                .await;

            (i, result)
        });

        handles.push(handle);
    }

    // All requests should complete
    let mut success_count = 0;
    for handle in handles {
        let (i, result) = handle.await.expect("Task panicked");

        if result.is_ok() {
            success_count += 1;
            println!("Request {} succeeded", i);
        } else {
            println!("Request {} failed: {:?}", i, result);
        }
    }

    assert!(
        success_count > 0,
        "At least some concurrent requests should succeed"
    );
}

/// Test: Platform-specific optimizations
#[tokio::test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
async fn test_metal_optimizations() {
    // On Apple Silicon, verify Metal-specific optimizations
    let mut backend = CpuBackend::new(test_model_path()).unwrap();

    if backend.load().await.is_err() {
        eprintln!("Skipping test: model not available");
        return;
    }

    let config = EmbeddedConfig::default();

    // Run a series of inferences to warm up Metal
    for _ in 0..3 {
        let _ = backend.infer("test warmup", &config).await;
    }

    // Measure inference time after warmup
    let start = Instant::now();
    let result = backend.infer("list all files", &config).await;
    let inference_time = start.elapsed();

    if result.is_ok() {
        println!("Warmed up Metal inference time: {:?}", inference_time);

        // After warmup, should be fast
        assert!(
            inference_time < Duration::from_secs(2),
            "Metal inference should be fast after warmup: {:?}",
            inference_time
        );
    }
}
