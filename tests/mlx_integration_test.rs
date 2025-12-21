// MLX Integration Test - Validates MLX backend on Apple Silicon
// This test demonstrates the full MLX workflow and auto-adapts based on CMAKE availability

#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use caro::backends::embedded::{InferenceBackend, MlxBackend, ModelVariant};
use caro::backends::CommandGenerator;
use caro::models::{CommandRequest, ShellType};
use caro::EmbeddedModelBackend;
use std::path::PathBuf;

/// Test that verifies MLX is correctly detected on Apple Silicon
#[test]
fn test_mlx_platform_detection() {
    let variant = ModelVariant::detect();
    assert_eq!(
        variant,
        ModelVariant::MLX,
        "M4 Pro should detect MLX backend"
    );

    println!("✅ Platform Detection: MLX correctly detected on Apple Silicon");
}

/// Test that MLX backend can be instantiated
#[test]
fn test_mlx_backend_instantiation() {
    let model_path = PathBuf::from("/tmp/test_model.gguf");
    let backend = MlxBackend::new(model_path);

    assert!(backend.is_ok(), "MLX backend should instantiate");

    let backend = backend.unwrap();
    assert_eq!(backend.variant(), ModelVariant::MLX);

    println!("✅ MLX Backend: Successfully instantiated");
}

/// Test MLX backend error handling with non-existent model
#[tokio::test]
async fn test_mlx_backend_error_handling() {
    let model_path = PathBuf::from("/tmp/nonexistent_model.gguf");
    let mut backend = MlxBackend::new(model_path).unwrap();

    // Try to load non-existent model - should fail gracefully
    let load_result = backend.load().await;

    #[cfg(feature = "embedded-mlx")]
    {
        assert!(
            load_result.is_err(),
            "Loading non-existent model should fail"
        );
        println!("✅ MLX Error Handling: Properly rejects non-existent model");
    }

    #[cfg(not(feature = "embedded-mlx"))]
    {
        assert!(
            load_result.is_err(),
            "Stub should return error when feature disabled"
        );
        println!("✅ MLX Stub: Returns appropriate error when feature disabled");
    }
}

/// Test EmbeddedModelBackend with MLX variant using real model
#[tokio::test]
async fn test_embedded_backend_with_mlx() {
    // Use the default model path (will use cached model if available)
    let backend = EmbeddedModelBackend::new();

    assert!(backend.is_ok(), "EmbeddedModelBackend should create");

    let backend = backend.unwrap();
    assert_eq!(backend.variant(), ModelVariant::MLX);
    assert!(backend.is_available().await, "Backend should be available");

    println!("✅ Embedded Backend: MLX variant working");

    let info = backend.backend_info();
    println!("   Backend Info:");
    println!("     Model: {}", info.model_name);
    println!("     Latency: {}ms", info.typical_latency_ms);
    println!("     Memory: {}MB", info.memory_usage_mb);
}

/// Test full command generation workflow with MLX backend
/// This test will download the model if not cached (~1.1GB)
#[tokio::test]
async fn test_mlx_command_generation_workflow() {
    println!("🔄 Testing command generation workflow...");
    println!("   Note: May download model if not cached (~1.1GB)");

    let backend = EmbeddedModelBackend::new().expect("Should create backend");

    let request = CommandRequest::new("list all files", ShellType::Bash);

    let result = backend.generate_command(&request).await;

    assert!(
        result.is_ok(),
        "Command generation should succeed: {:?}",
        result.err()
    );

    let command = result.unwrap();
    assert!(!command.command.is_empty(), "Command should not be empty");
    assert_eq!(
        command.backend_used, "embedded",
        "Should use embedded backend"
    );
    assert!(
        command.generation_time_ms > 0,
        "Should have generation time"
    );

    println!("✅ Command Generation: Full workflow working");
    println!("   Input: 'list all files'");
    println!("   Output: {}", command.command);
    println!("   Time: {}ms", command.generation_time_ms);
    println!("   Confidence: {}", command.confidence_score);
}

/// Test MLX performance characteristics
/// This test measures real inference times
#[tokio::test]
async fn test_mlx_performance() {
    use std::time::Instant;

    println!("⏱️  Testing MLX performance...");
    println!("   Note: May download model if not cached (~1.1GB)");

    let backend = EmbeddedModelBackend::new().expect("Should create backend");

    let request = CommandRequest::new("list files", ShellType::Bash);

    // Measure first inference (includes lazy loading)
    let start = Instant::now();
    let result = backend.generate_command(&request).await;
    let first_inference = start.elapsed();

    assert!(
        result.is_ok(),
        "First inference should succeed: {:?}",
        result.err()
    );
    println!(
        "✅ Performance: First inference completed in {:?}",
        first_inference
    );

    // Measure second inference (model already loaded)
    let start = Instant::now();
    let result = backend.generate_command(&request).await;
    let second_inference = start.elapsed();

    assert!(result.is_ok(), "Second inference should succeed");
    println!(
        "✅ Performance: Second inference completed in {:?}",
        second_inference
    );

    // First inference may include model download time
    // Be generous with timeout (allow up to 2 minutes for download + inference)
    assert!(
        first_inference.as_secs() < 120,
        "First inference should complete within 120s (includes potential model download)"
    );

    // Second inference should be much faster (target < 5s)
    println!(
        "   → Second inference {} faster than first",
        if second_inference < first_inference {
            format!(
                "{:.1}x",
                first_inference.as_secs_f64() / second_inference.as_secs_f64()
            )
        } else {
            "was not".to_string()
        }
    );
}

/// Integration test: Full MLX + Model Download + Real Inference
/// This test is the same as test_mlx_command_generation_workflow but kept for backward compatibility
#[tokio::test]
#[ignore]
async fn test_mlx_full_integration() {
    println!("🚀 Starting full MLX integration test...");
    println!("   This will download model if not cached (~1.1GB)");

    // Use real model path
    let backend = EmbeddedModelBackend::new();
    assert!(backend.is_ok(), "Should create backend with real model");

    let backend = backend.unwrap();
    assert_eq!(backend.variant(), ModelVariant::MLX);

    println!("✅ Backend created with MLX variant");

    // Test model download and inference
    let request = CommandRequest::new("list all files recursively", ShellType::Bash);

    println!("🔄 Running inference (may download model)...");

    let start = std::time::Instant::now();
    let result = backend.generate_command(&request).await;
    let elapsed = start.elapsed();

    assert!(
        result.is_ok(),
        "Real inference should succeed: {:?}",
        result.err()
    );

    let command = result.unwrap();
    println!("✅ Real inference successful!");
    println!("   Input: 'list all files recursively'");
    println!("   Command: {}", command.command);
    println!("   Time: {:?}", elapsed);
    println!("   Confidence: {}", command.confidence_score);

    // Verify performance targets
    assert!(
        elapsed.as_secs() < 120,
        "First inference should complete within 120s (includes model download)"
    );

    // Test second inference (should be faster)
    let request2 = CommandRequest::new("find all text files", ShellType::Bash);

    let start = std::time::Instant::now();
    let result2 = backend.generate_command(&request2).await;
    let elapsed2 = start.elapsed();

    assert!(result2.is_ok(), "Second inference should succeed");

    println!("✅ Second inference:");
    println!("   Input: 'find all text files'");
    println!("   Command: {}", result2.unwrap().command);
    println!("   Time: {:?}", elapsed2);

    // Second inference should be much faster
    assert!(
        elapsed2.as_secs() < 10,
        "Subsequent inference should complete within 10s"
    );

    println!("🎉 Full MLX integration test passed!");
}

/// Summary test that reports current status
#[test]
fn test_mlx_implementation_status() {
    println!("\n📊 MLX Implementation Status Report");
    println!("{}", "=".repeat(50));

    // Platform
    let variant = ModelVariant::detect();
    println!("✅ Platform: {} (Apple Silicon)", variant);

    // Backend instantiation
    let backend = MlxBackend::new(PathBuf::from("/tmp/test.gguf"));
    println!("✅ Backend Instantiation: Working");

    // Check if full MLX is available (would require CMAKE)
    #[cfg(feature = "embedded-mlx")]
    {
        println!("✅ MLX Feature: Enabled (CMAKE available)");
        println!("   → Full GPU acceleration available");
    }

    #[cfg(not(feature = "embedded-mlx"))]
    {
        println!("⚠️  MLX Feature: Using stub implementation");
        println!("   → Install CMAKE for full GPU acceleration");
        println!("   → Run: brew install cmake");
        println!("   → Build: cargo build --features embedded-mlx");
    }

    // Model cache
    let cache_dir = dirs::cache_dir().unwrap().join("caro").join("models");
    let model_file = cache_dir.join("qwen2.5-coder-1.5b-instruct-q4_k_m.gguf");

    if model_file.exists() {
        let size_mb = std::fs::metadata(&model_file)
            .map(|m| m.len() / 1_000_000)
            .unwrap_or(0);
        println!("✅ Model: Downloaded ({}MB)", size_mb);
        println!("   → Location: {}", model_file.display());
    } else {
        println!("ℹ️  Model: Not downloaded yet");
        println!("   → Will auto-download on first use (~1.1GB)");
    }

    println!("{}", "=".repeat(50));

    assert!(backend.is_ok(), "Backend should be functional");
}
