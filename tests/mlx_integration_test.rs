// MLX Integration Test - Validates MLX backend on Apple Silicon
// This test demonstrates the full MLX workflow and auto-adapts based on CMAKE availability

#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use caro::backends::embedded::{EmbeddedConfig, InferenceBackend, MlxBackend, ModelVariant};
use caro::backends::CommandGenerator;
use caro::EmbeddedModelBackend;
use caro::models::{CommandRequest, ShellType};
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

/// Test MLX backend with simulated inference (works with stub implementation)
#[tokio::test]
async fn test_mlx_backend_simulated_inference() {
    let model_path = PathBuf::from("/tmp/test_model.gguf");
    let mut backend = MlxBackend::new(model_path).unwrap();

    // Create a temporary test file to simulate model existence
    std::fs::write("/tmp/test_model.gguf", "fake model data").unwrap();

    // Load model
    let load_result = backend.load().await;
    assert!(load_result.is_ok(), "Model loading should succeed");

    // Run inference
    let config = EmbeddedConfig::default();
    let result = backend.infer("list all files", &config).await;

    assert!(result.is_ok(), "Inference should succeed");

    let response = result.unwrap();
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(
        response.contains("cmd") || response.contains("ls"),
        "Response should contain command"
    );

    println!("✅ MLX Inference: Stub implementation working");
    println!("   Response: {}", response);

    // Cleanup
    backend.unload().await.ok();
    std::fs::remove_file("/tmp/test_model.gguf").ok();
}

/// Test EmbeddedModelBackend with MLX variant
#[tokio::test]
async fn test_embedded_backend_with_mlx() {
    // Create test model file
    let test_model_path = PathBuf::from("/tmp/test_embedded_model.gguf");
    std::fs::write(&test_model_path, "fake model data").unwrap();

    let backend =
        EmbeddedModelBackend::with_variant_and_path(ModelVariant::MLX, test_model_path.clone());

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

    // Cleanup
    std::fs::remove_file(&test_model_path).ok();
}

/// Test full command generation workflow with MLX backend
#[tokio::test]
async fn test_mlx_command_generation_workflow() {
    // Create test model file
    let test_model_path = PathBuf::from("/tmp/test_mlx_workflow.gguf");
    std::fs::write(&test_model_path, "fake model data").unwrap();

    let backend =
        EmbeddedModelBackend::with_variant_and_path(ModelVariant::MLX, test_model_path.clone())
            .unwrap();

    let request = CommandRequest::new("list all files", ShellType::Bash);

    let result = backend.generate_command(&request).await;

    assert!(result.is_ok(), "Command generation should succeed");

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

    // Cleanup
    std::fs::remove_file(&test_model_path).ok();
}

/// Test MLX performance characteristics (with stub)
#[tokio::test]
async fn test_mlx_performance_stub() {
    use std::time::Instant;

    let test_model_path = PathBuf::from("/tmp/test_mlx_perf.gguf");
    std::fs::write(&test_model_path, "fake model data").unwrap();

    let backend =
        EmbeddedModelBackend::with_variant_and_path(ModelVariant::MLX, test_model_path.clone())
            .unwrap();

    let request = CommandRequest::new("list files", ShellType::Bash);

    // Measure first inference (includes lazy loading)
    let start = Instant::now();
    let result = backend.generate_command(&request).await;
    let first_inference = start.elapsed();

    assert!(result.is_ok(), "First inference should succeed");
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

    // Stub should be reasonably fast (allow time for model download if it happens)
    // Note: First inference may download model from Hugging Face if not cached
    assert!(
        first_inference.as_secs() < 60,
        "Stub should complete within 60s (includes potential model download)"
    );
    
    // Second inference should be much faster since model is loaded
    assert!(
        second_inference.as_millis() < 5000,
        "Subsequent calls should be faster"
    );

    // Cleanup
    std::fs::remove_file(&test_model_path).ok();
}

/// Integration test: Full MLX + Model Download + Real Inference
/// This test is ignored by default because it:
/// 1. Requires CMAKE to be installed
/// 2. Downloads ~1.1GB model from Hugging Face
/// 3. Requires actual MLX implementation (not stub)
///
/// To run this test:
/// 1. Install CMAKE: brew install cmake
/// 2. Build with MLX: cargo build --features embedded-mlx
/// 3. Run: cargo test test_mlx_full_integration -- --ignored --nocapture
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
        elapsed.as_secs() < 10,
        "First inference should complete within 10s (includes model load)"
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
        elapsed2.as_secs() < 5,
        "Subsequent inference should complete within 5s"
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
    let cache_dir = dirs::cache_dir()
        .unwrap()
        .join("caro")
        .join("models");
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
