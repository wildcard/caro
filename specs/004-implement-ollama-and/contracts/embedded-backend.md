# Contract: EmbeddedModelBackend

**Entity**: E0 - EmbeddedModelBackend
**Location**: `src/backends/embedded/mod.rs`
**Purpose**: Primary command generator using embedded Qwen model with platform-specific inference

## Behavioral Contract

### Must Implement

**Trait**: `CommandGenerator` (existing trait from Feature 002)

```rust
#[async_trait]
pub trait CommandGenerator {
    async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand, GeneratorError>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
    async fn shutdown(&self) -> Result<(), GeneratorError>;
}
```

### Contract Requirements

#### CR-EMB-001: Offline Operation (CRITICAL)
**MUST** work completely offline without any network calls.

**Test**:
```rust
#[tokio::test]
async fn test_embedded_backend_offline() {
    // Disable network completely
    std::env::set_var("NO_NETWORK", "1");

    let backend = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        test_model_path()
    ).unwrap();

    let request = CommandRequest::new("list files", ShellType::Bash);
    let result = backend.generate_command(&request).await;

    assert!(result.is_ok(), "Must work offline");
    assert!(!result.unwrap().command.is_empty());
}
```

#### CR-EMB-002: Always Available
**MUST** return `true` from `is_available()` at all times (embedded model always present).

**Test**:
```rust
#[tokio::test]
async fn test_embedded_backend_always_available() {
    let backend = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        test_model_path()
    ).unwrap();

    // Check availability multiple times
    for _ in 0..10 {
        assert!(backend.is_available().await, "Must always be available");
    }
}
```

#### CR-EMB-003: Performance Targets
**MUST** meet platform-specific performance targets from FR-025, FR-027.

**Test**:
```rust
#[tokio::test]
async fn test_embedded_backend_performance() {
    let backend = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        test_model_path()
    ).unwrap();

    let request = CommandRequest::new("list files", ShellType::Bash);

    // Measure startup time (lazy load on first call)
    let start = Instant::now();
    let _ = backend.generate_command(&request).await.unwrap();
    let first_call_duration = start.elapsed();

    // Check performance targets based on variant
    match backend.model_variant {
        ModelVariant::MLX => {
            // FR-027: <100ms startup, FR-025: <2s inference
            assert!(first_call_duration < Duration::from_secs(2),
                "MLX must complete within 2s (startup + inference)");
        },
        ModelVariant::CPU => {
            // Acceptable: <5s total for CPU fallback
            assert!(first_call_duration < Duration::from_secs(5),
                "CPU must complete within 5s");
        }
    }

    // Subsequent calls should be faster (model already loaded)
    let start = Instant::now();
    let _ = backend.generate_command(&request).await.unwrap();
    let subsequent_duration = start.elapsed();

    assert!(subsequent_duration < first_call_duration,
        "Subsequent calls must be faster (no load time)");
}
```

#### CR-EMB-004: Safety Integration
**MUST** integrate with safety validation module (all commands validated regardless of backend).

**Test**:
```rust
#[tokio::test]
async fn test_embedded_backend_safety_integration() {
    let backend = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        test_model_path()
    ).unwrap();

    let request = CommandRequest::new("delete everything", ShellType::Bash);
    let result = backend.generate_command(&request).await.unwrap();

    // Generated command must pass through safety validator
    // (Safety validator will flag dangerous commands)
    let safety_result = validate_command(&result.command);

    // Either:
    // 1. Backend generates safe alternative, OR
    // 2. Safety validator catches dangerous command
    assert!(safety_result.risk_level != RiskLevel::Safe ||
            result.command.contains("# DANGEROUS"),
        "Dangerous commands must be flagged");
}
```

#### CR-EMB-005: Model Variant Detection
**MUST** correctly detect and use platform-appropriate variant (MLX on Apple Silicon, CPU elsewhere).

**Test**:
```rust
#[test]
fn test_model_variant_detection() {
    let detected = ModelVariant::detect();

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    assert_eq!(detected, ModelVariant::MLX, "Apple Silicon must use MLX");

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    assert_eq!(detected, ModelVariant::CPU, "Other platforms must use CPU");
}
```

#### CR-EMB-006: Lazy Loading
**MUST** implement lazy loading (load model on first inference, not on construction).

**Test**:
```rust
#[tokio::test]
async fn test_lazy_loading() {
    let start = Instant::now();
    let backend = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        test_model_path()
    ).unwrap();
    let construction_time = start.elapsed();

    // Construction must be fast (<10ms)
    assert!(construction_time < Duration::from_millis(10),
        "Construction must not load model");

    // First inference triggers load
    let start = Instant::now();
    let _ = backend.generate_command(&CommandRequest::new("test", ShellType::Bash)).await.unwrap();
    let first_inference = start.elapsed();

    // FR-027: Total time including load must meet targets
    assert!(first_inference < Duration::from_secs(2) ||
            backend.model_variant == ModelVariant::CPU,
        "First inference includes load time");
}
```

#### CR-EMB-007: Error Handling
**MUST** handle model loading errors gracefully with actionable error messages.

**Test**:
```rust
#[tokio::test]
async fn test_missing_model_error() {
    let result = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        PathBuf::from("/nonexistent/model.gguf")
    );

    assert!(result.is_err(), "Must error on missing model");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("model"),
        "Error must mention model file");
}

#[tokio::test]
async fn test_corrupt_model_error() {
    // Create corrupt GGUF file
    let temp_path = create_corrupt_gguf_file();

    let backend = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        temp_path
    ).unwrap();

    let result = backend.generate_command(&CommandRequest::new("test", ShellType::Bash)).await;

    assert!(result.is_err(), "Must error on corrupt model");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("corrupt") ||
            error.to_string().contains("invalid"),
        "Error must indicate corruption");
}
```

#### CR-EMB-008: Resource Cleanup
**MUST** release model resources on shutdown.

**Test**:
```rust
#[tokio::test]
async fn test_resource_cleanup() {
    let backend = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        test_model_path()
    ).unwrap();

    // Load model
    let _ = backend.generate_command(&CommandRequest::new("test", ShellType::Bash)).await.unwrap();

    // Measure memory before shutdown
    let mem_before = get_process_memory();

    // Shutdown
    backend.shutdown().await.unwrap();

    // Wait for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Memory should decrease (model unloaded)
    let mem_after = get_process_memory();
    assert!(mem_after < mem_before,
        "Memory must be released after shutdown");
}
```

#### CR-EMB-009: Backend Info
**MUST** return accurate backend information for observability.

**Test**:
```rust
#[test]
fn test_backend_info() {
    let backend = EmbeddedModelBackend::new(
        ModelVariant::MLX,
        test_model_path()
    ).unwrap();

    let info = backend.backend_info();

    assert_eq!(info.backend_type, "embedded");
    assert!(info.model_name.contains("qwen"));
    assert!(info.version.starts_with("2.5"));
    assert_eq!(info.supports_streaming, false);
}
```

## Integration Points

### With Safety Validator
- Embedded backend generates command → Safety validator checks → User confirmation if needed
- No special handling; same pipeline as remote backends

### With Configuration
- Model path read from `~/.caro/models/` or installation directory
- Variant auto-detected or overridden via config `backend.embedded.variant`
- Temperature/context settings from `EmbeddedConfig`

### With CLI
- Default backend when no `--backend` flag specified
- Fallback backend when remote backends fail
- Always available for offline operation

## Performance Requirements

| Metric | MLX (macOS) | CPU (Other) |
|--------|-------------|-------------|
| Construction | <10ms | <10ms |
| First load | <100ms | <500ms |
| First token | <200ms | <800ms |
| Total inference | <2s (FR-025) | <5s (acceptable) |
| Memory usage | ~1.2GB (model + runtime) | ~1.5GB |

## Error Cases

| Scenario | Expected Behavior |
|----------|-------------------|
| Model file missing | `GeneratorError::ModelNotFound` with path |
| Model file corrupt | `GeneratorError::ModelLoadFailed` with details |
| Out of memory | `GeneratorError::ResourceExhausted` with suggestion |
| Invalid prompt | `GeneratorError::InvalidInput` with validation message |
| Platform mismatch | Auto-fallback to CPU variant (no error) |

---

**Test Coverage Target**: 100% for critical paths (offline operation, performance, safety integration)

**Property Testing**: Fuzz test with random prompts to ensure no panics or hangs
