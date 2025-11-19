# Contract: Embedded Backend (Candle Metal/CPU)

**Entity**: E1 - Embedded Model Backend
**Location**: `src/backends/embedded/` (cpu.rs, metal.rs)
**Purpose**: Candle-powered GPU-accelerated inference for Apple Silicon with CPU fallback

## Behavioral Contract

### Must Implement

**Trait**: `InferenceBackend` (internal trait for embedded model variants)

```rust
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError>;
    fn variant(&self) -> ModelVariant;
    async fn load(&mut self) -> Result<(), GeneratorError>;
    async fn unload(&mut self) -> Result<(), GeneratorError>;
}
```

### Contract Requirements

#### CR-METAL-001: Platform Restriction
**MUST** only compile Metal backend on macOS with Apple Silicon (aarch64). CPU backend is cross-platform.

**Test**:
```rust
#[test]
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
fn test_metal_backend_unavailable() {
    // Metal backend should not compile on non-Apple Silicon platforms
    // CPU backend should always be available
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn test_metal_backend_available() {
    let result = CandleBackend::new_with_device(test_model_path(), Device::new_metal(0)?);
    assert!(result.is_ok(), "Metal must be available on Apple Silicon");
}

#[test]
fn test_cpu_backend_available() {
    let result = CandleBackend::new_with_device(test_model_path(), Device::Cpu);
    assert!(result.is_ok(), "CPU backend must be available on all platforms");
}
```

#### CR-METAL-002: Device Auto-Selection
**MUST** automatically select Metal device on Apple Silicon, with CPU fallback if unavailable.

**Test**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_device_auto_selection() {
    let backend = CandleBackend::new(test_model_path()).unwrap();

    // Load model
    backend.load().await.unwrap();

    // Check that Metal device is used
    let device = backend.device();
    assert!(matches!(device, Device::Metal(_)),
        "Should use Metal device on Apple Silicon");
}

#[tokio::test]
async fn test_cpu_fallback() {
    // Force CPU mode
    let backend = CandleBackend::new_with_device(test_model_path(), Device::Cpu).unwrap();
    backend.load().await.unwrap();

    let device = backend.device();
    assert!(matches!(device, Device::Cpu),
        "Should use CPU when explicitly requested");
}
```

#### CR-METAL-003: Fast Initialization
**MUST** initialize within 100ms (FR-027 startup budget).

**Test**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_fast_initialization() {
    let start = Instant::now();

    let mut backend = CandleBackend::new(test_model_path()).unwrap();
    backend.load().await.unwrap();

    let load_time = start.elapsed();

    assert!(load_time < Duration::from_millis(100),
        "Metal initialization must complete within 100ms, got {:?}", load_time);
}
```

#### CR-METAL-004: Inference Performance
**MUST** generate commands within 2s total (FR-025).

**Test**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_inference_performance() {
    let mut backend = CandleBackend::new(test_model_path()).unwrap();
    backend.load().await.unwrap();

    let config = EmbeddedConfig::default();
    let start = Instant::now();

    let output = backend.infer("Generate bash command to list all files", &config).await.unwrap();

    let inference_time = start.elapsed();

    assert!(inference_time < Duration::from_secs(2),
        "Candle Metalinference must complete within 2s, got {:?}", inference_time);
    assert!(!output.is_empty(), "Must return generated text");
}
```

#### CR-METAL-005: First Token Latency
**MUST** achieve <200ms first token latency for responsive UX.

**Test**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_first_token_latency() {
    let mut backend = CandleBackend::new(test_model_path()).unwrap();
    backend.load().await.unwrap();

    let config = EmbeddedConfig::default();
    let start = Instant::now();

    // Use streaming to measure first token (if supported)
    let first_token_time = measure_first_token_time(&mlx, "list files", &config).await;

    assert!(first_token_time < Duration::from_millis(200),
        "First token must arrive within 200ms, got {:?}", first_token_time);
}
```

#### CR-METAL-006: Metal Framework Compatibility
**MUST** handle Metal framework errors gracefully.

**Test**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_metal_error_handling() {
    let backend = CandleBackend::new(test_model_path()).unwrap();

    // Simulate Metal framework error (e.g., device loss)
    simulate_metal_error();

    let result = backend.infer("test", &EmbeddedConfig::default()).await;

    assert!(result.is_err(), "Must handle Metal errors");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Metal") ||
            error.to_string().contains("GPU"),
        "Error must indicate Metal/GPU issue");
}
```

#### CR-METAL-007: Model Format Support
**MUST** support GGUF quantized models (Q4_K_M minimum).

**Test**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn test_metal_gguf_support() {
    // Test with Q4_K_M quantized model
    let q4_model_path = get_qwen_q4_model_path();
    let result = MlxBackend::new(&q4_model_path);

    assert!(result.is_ok(), "Must support GGUF Q4_K_M quantization");

    // Test with Q8_0 quantized model (higher quality)
    let q8_model_path = get_qwen_q8_model_path();
    let result = MlxBackend::new(&q8_model_path);

    assert!(result.is_ok(), "Must support GGUF Q8_0 quantization");
}
```

#### CR-METAL-008: Concurrent Request Handling
**MUST** safely handle concurrent inference requests (mutex-protected model).

**Test**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_concurrent_requests() {
    let mlx = Arc::new(MlxBackend::new(test_model_path()).unwrap());
    let config = EmbeddedConfig::default();

    // Spawn multiple concurrent requests
    let mut handles = vec![];
    for i in 0..5 {
        let backend_clone = Arc::clone(&backend);
        let config_clone = config.clone();

        handles.push(tokio::spawn(async move {
            backend_clone.infer(&format!("request {}", i), &config_clone).await
        }));
    }

    // All requests should complete successfully
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent requests must all succeed");
    }
}
```

#### CR-METAL-009: Resource Cleanup
**MUST** release GPU resources on unload.

**Test**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_resource_cleanup() {
    let mut backend = CandleBackend::new(test_model_path()).unwrap();

    // Load model to GPU
    backend.load().await.unwrap();

    // Check GPU memory usage
    let gpu_mem_before = get_metal_memory_usage();

    // Unload
    backend.unload().await.unwrap();

    // Wait for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    // GPU memory should be released
    let gpu_mem_after = get_metal_memory_usage();
    assert!(gpu_mem_after < gpu_mem_before,
        "GPU memory must be released after unload");
}
```

#### CR-METAL-010: Temperature Control
**MUST** respect temperature setting for sampling control.

**Test**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_metal_temperature_control() {
    let mut backend = CandleBackend::new(test_model_path()).unwrap();
    backend.load().await.unwrap();

    // Test with low temperature (deterministic)
    let config_low = EmbeddedConfig {
        temperature: 0.1,
        ..Default::default()
    };

    let output1 = backend.infer("list files", &config_low).await.unwrap();
    let output2 = backend.infer("list files", &config_low).await.unwrap();

    // Low temperature should produce similar outputs
    assert_eq!(output1, output2,
        "Low temperature should be deterministic");

    // Test with high temperature (creative)
    let config_high = EmbeddedConfig {
        temperature: 1.5,
        ..Default::default()
    };

    let output3 = backend.infer("list files", &config_high).await.unwrap();
    let output4 = backend.infer("list files", &config_high).await.unwrap();

    // High temperature may produce different outputs
    // (Not guaranteed, but statistically likely)
}
```

## Integration Points

### With EmbeddedModelBackend
- CandleBackend instantiated based on device availability (Metal/CPU)
- Wrapped in `Box<dyn InferenceBackend>` for polymorphism
- Called via `InferenceBackend::infer()` interface

### With Candle Framework
- Use `candle-core` 0.9+ for tensor operations
- Use `candle-transformers` for GGUF model loading
- Conditional compilation: `#[cfg(all(target_os = "macos", target_arch = "aarch64"))]` for Metal
- Device selection: `Device::new_metal(0)?` for GPU, `Device::Cpu` for CPU

### With Tokenizer
- Use `tokenizers` crate for Qwen tokenizer
- Load from `tokenizer.json` in model directory or HuggingFace Hub
- Encode prompt → tokens → Candle forward pass → tokens → decode output

## Performance Requirements

| Metric | Target | Notes |
|--------|--------|-------|
| Initialization | <100ms | Model load to GPU (FR-027) |
| First token | <200ms | Prompt processing + first generation |
| Throughput (Metal) | 15-30 tokens/sec | M4 Max with Metal GPU |
| Throughput (CPU) | 3-8 tokens/sec | M4 Max CPU fallback |
| Total inference | <2s | 20-token command generation (FR-025) |
| Memory usage | ~1.1GB | GGUF Q4_K_M model + runtime |
| GPU utilization | >50% | Efficient Metal usage |

## Error Cases

| Scenario | Expected Behavior |
|----------|-------------------|
| Non-Apple Silicon | Compile-time error (cfg gate) |
| Metal unavailable | `GeneratorError::PlatformUnsupported` |
| Model load failure | `GeneratorError::ModelLoadFailed` with Metal error details |
| OOM (model too large) | `GeneratorError::ResourceExhausted` with memory info |
| Inference timeout | `GeneratorError::Timeout` after 30s |
| Concurrent access | Mutex ensures serial access, no data race |

---

**Test Coverage Target**: 95% for backend-specific logic

**Platform Testing**:
- Metal backend: Requires physical M1/M2/M3/M4 Mac for integration tests
- CPU backend: Cross-platform testing on Linux, macOS, Windows

**Conditional Compilation**: Metal-specific code must be behind `#[cfg(all(target_os = "macos", target_arch = "aarch64"))]`

**Framework**: Powered by HuggingFace Candle with Metal backend support
