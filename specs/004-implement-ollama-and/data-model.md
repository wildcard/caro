# Data Model: Embedded Model + Remote Backend Support

**Feature**: 004-implement-ollama-and
**Date**: 2025-10-14 (Updated)
**Purpose**: Define data structures for embedded model (Qwen + MLX + Candle) and remote backends (Ollama, vLLM)

## Overview

This data model extends the existing `caro::models` and `caro::backends` modules with:
1. **Embedded model backends** (E0-E2): Primary backends using Qwen model with MLX GPU or Candle CPU inference
2. **Remote backend types** (E3-E5): HTTP connection management, retry logic, health tracking
3. **Remote backend implementations** (E6-E7): Ollama and vLLM optional enhancements

All entities implement the existing `CommandGenerator` trait and integrate with `CommandRequest`, `GeneratedCommand`, `BackendInfo`.

**Architecture**: Embedded model (always available) → Remote backends (optional enhancements with fallback)

---

## Entity Definitions

### E0: EmbeddedModelBackend

**Purpose**: Primary command generator using embedded Qwen model with platform-specific inference (MLX GPU or Candle CPU).

**Location**: `src/backends/embedded/mod.rs` (new file)

**Fields**:
- `model_variant: ModelVariant` - Enum: `MLX` (Apple Silicon GPU) or `CPU` (Candle cross-platform)
- `model_path: PathBuf` - Path to embedded GGUF model file (~900MB Q4_K_M quantization)
- `model_name: String` - Model identifier (e.g., "qwen2.5-coder-1.5b-q4")
- `config: EmbeddedConfig` - Temperature, context size, stop tokens
- `backend: Box<dyn InferenceBackend>` - Either `MlxBackend` or `CpuBackend` (trait object)
- `is_loaded: AtomicBool` - Lazy loading state (load on first inference)
- `load_time_ms: AtomicU64` - Startup performance tracking

**Validation Rules**:
- `model_path` must exist and be readable GGUF file
- `model_variant` must match platform (MLX only on macOS aarch64)
- `config.temperature` must be 0.0-2.0 (default 0.7)
- `config.context_size` must be 256-32768 (default 2048)

**Methods**:
```rust
impl EmbeddedModelBackend {
    pub fn new(variant: ModelVariant, model_path: PathBuf) -> Result<Self, GeneratorError>;
    pub async fn load_model(&self) -> Result<(), GeneratorError>;  // Lazy load
    pub fn with_config(mut self, config: EmbeddedConfig) -> Self;
}

#[async_trait]
impl CommandGenerator for EmbeddedModelBackend {
    async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand, GeneratorError>;
    async fn is_available(&self) -> bool;  // Always true (embedded)
    fn backend_info(&self) -> BackendInfo;
    async fn shutdown(&self) -> Result<(), GeneratorError>;
}
```

**Example**:
```rust
let backend = EmbeddedModelBackend::new(
    ModelVariant::MLX,
    PathBuf::from("/usr/local/lib/caro/models/qwen2.5-coder-1.5b-q4.gguf")
)?
.with_config(EmbeddedConfig {
    temperature: 0.7,
    context_size: 2048,
    stop_tokens: vec!["\n\n".to_string(), "```".to_string()],
});

let request = CommandRequest::new("list all files", ShellType::Bash);
let command = backend.generate_command(&request).await?;
assert!(!command.command.is_empty());
```

---

### E1: MlxBackend

**Purpose**: MLX GPU-accelerated inference backend for Apple Silicon using unified memory architecture.

**Location**: `src/backends/embedded/mlx.rs` (new file)

**Fields**:
- `model: Arc<Mutex<MlxModel>>` - MLX model wrapper (thread-safe, shared across requests)
- `device: MlxDevice` - Metal GPU device (unified memory)
- `tokenizer: Arc<Tokenizer>` - Shared tokenizer for encoding/decoding
- `config: MlxConfig` - MLX-specific settings (batch size, precision)

**Validation Rules**:
- Platform must be macOS with Apple Silicon (aarch64)
- Metal framework must be available
- Model file must be compatible with MLX (GGUF format)

**Methods**:
```rust
impl MlxBackend {
    pub fn new(model_path: &Path) -> Result<Self, GeneratorError>;
    pub async fn forward(&self, input_ids: &[u32]) -> Result<Vec<u32>, GeneratorError>;
}

#[async_trait]
impl InferenceBackend for MlxBackend {
    async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError>;
    fn variant(&self) -> ModelVariant { ModelVariant::MLX }
}
```

**Performance Targets**:
- Model load: <100ms (lazy load on first inference)
- First token latency: <200ms
- Generation throughput: ~8 tokens/sec
- Total inference time: <2s for 20-token command

**Example**:
```rust
let mlx = MlxBackend::new(Path::new("/path/to/qwen-q4.gguf"))?;
let output = mlx.infer("Generate bash command to list files", &config).await?;
assert_eq!(output, "ls -la");
```

---

### E2: CpuBackend

**Purpose**: Cross-platform CPU inference backend using Candle with `candle-transformers` Qwen2 support.

**Location**: `src/backends/embedded/cpu.rs` (new file)

**Fields**:
- `model: Arc<Qwen2Model>` - Candle Qwen2 model (thread-safe)
- `device: Device` - CPU device (Candle abstraction)
- `tokenizer: Arc<Tokenizer>` - Shared tokenizer
- `config: CandleConfig` - Candle-specific settings

**Validation Rules**:
- Model file must be SafeTensors or GGUF format compatible with Candle
- Device must be CPU (fallback when MLX unavailable)

**Methods**:
```rust
impl CpuBackend {
    pub fn new(model_path: &Path) -> Result<Self, GeneratorError>;
    pub async fn forward(&self, input_ids: &Tensor) -> Result<Tensor, GeneratorError>;
}

#[async_trait]
impl InferenceBackend for CpuBackend {
    async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError>;
    fn variant(&self) -> ModelVariant { ModelVariant::CPU }
}
```

**Performance Targets**:
- Model load: <500ms (one-time cost)
- First token latency: <800ms
- Generation throughput: ~5 tokens/sec
- Total inference time: <5s acceptable for CPU fallback

**Example**:
```rust
let cpu = CpuBackend::new(Path::new("/path/to/qwen-q4.gguf"))?;
let output = cpu.infer("find large files", &config).await?;
```

---

### E3: BackendConnection

**Purpose**: Represents an active HTTP connection to a remote LLM backend with health tracking.

**Location**: `src/backends/connection.rs` (new file)

**Fields**:
- `url: Url` - Base URL of the backend service (e.g., `http://localhost:11434`)
- `client: reqwest::Client` - Shared HTTP client with connection pooling
- `backend_type: BackendType` - Enum: `Ollama` or `Vllm`
- `last_health_check: Option<Instant>` - Timestamp of last successful health check
- `health_status: HealthStatus` - Enum: `Healthy`, `Degraded`, `Unavailable`
- `consecutive_failures: u32` - Count of failures since last success (for circuit breaking)
- `average_latency_ms: Option<u64>` - Rolling average response time

**Validation Rules**:
- `url` must be valid HTTP(S) URL
- `consecutive_failures` capped at 10 (triggers circuit breaker)
- `health_status` updated on every request attempt

**Methods**:
```rust
impl BackendConnection {
    pub fn new(url: Url, backend_type: BackendType) -> Result<Self, ConnectionError>;
    pub async fn health_check(&mut self) -> Result<(), ConnectionError>;
    pub fn is_healthy(&self) -> bool;
    pub fn record_success(&mut self, duration: Duration);
    pub fn record_failure(&mut self);
    pub fn should_retry(&self) -> bool;
}
```

**Example**:
```rust
let connection = BackendConnection::new(
    Url::parse("http://localhost:11434")?,
    BackendType::Ollama
)?;
assert!(connection.health_check().await.is_ok());
```

---

### E4: RetryPolicy

**Purpose**: Configuration for exponential backoff retry logic.

**Location**: `src/backends/retry.rs` (new file)

**Fields**:
- `max_attempts: u32` - Maximum retry attempts (default: 3)
- `base_delay_ms: u64` - Initial delay in milliseconds (default: 1000)
- `max_delay_ms: u64` - Maximum delay cap (default: 10000)
- `backoff_multiplier: f32` - Exponential multiplier (default: 2.0)
- `jitter: bool` - Add randomness to prevent thundering herd (default: true)

**Derived Values**:
- Delay for attempt N: `min(base_delay_ms * (backoff_multiplier ^ (N-1)), max_delay_ms)`
- With jitter: Add random ±25% variance

**Methods**:
```rust
impl RetryPolicy {
    pub fn default() -> Self;
    pub fn strict() -> Self;  // No retries
    pub fn aggressive() -> Self;  // 5 attempts
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration;
    pub fn should_retry(&self, attempt: u32, error: &reqwest::Error) -> bool;
}
```

**Example**:
```rust
let policy = RetryPolicy::default();
assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(1000));
assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(2000));
```

---

### E5: OllamaBackend

**Purpose**: Implementation of `CommandGenerator` trait for Ollama HTTP API.

**Location**: `src/backends/ollama.rs` (new file)

**Fields**:
- `connection: BackendConnection` - HTTP connection state
- `model: String` - Ollama model name (e.g., "codellama:7b")
- `retry_policy: RetryPolicy` - Retry configuration
- `timeout: Duration` - Generation timeout (default: 30s)
- `temperature: f32` - Sampling temperature (default: 0.7)
- `max_tokens: Option<u32>` - Token limit (default: 100)

**Trait Implementation**: `CommandGenerator`

**Methods** (from trait):
```rust
#[async_trait]
impl CommandGenerator for OllamaBackend {
    async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand, GeneratorError>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
    async fn shutdown(&self) -> Result<(), GeneratorError>;
}
```

**Additional Methods**:
```rust
impl OllamaBackend {
    pub fn new(url: Url, model: String) -> Result<Self, GeneratorError>;
    pub fn with_retry_policy(self, policy: RetryPolicy) -> Self;
    pub fn with_temperature(self, temp: f32) -> Self;
    async fn parse_response(&self, response: OllamaResponse) -> Result<GeneratedCommand, GeneratorError>;
}
```

**Example**:
```rust
let backend = OllamaBackend::new(
    Url::parse("http://localhost:11434")?,
    "codellama:7b".to_string()
)?;

let request = CommandRequest::new("list files", ShellType::Bash);
let command = backend.generate_command(&request).await?;
assert!(!command.command.is_empty());
```

---

### E6: VllmBackend

**Purpose**: Implementation of `CommandGenerator` trait for vLLM OpenAI-compatible API.

**Location**: `src/backends/vllm.rs` (new file)

**Fields**:
- `connection: BackendConnection` - HTTP connection state
- `model: String` - HuggingFace model path (e.g., "meta-llama/Llama-2-7b-hf")
- `retry_policy: RetryPolicy` - Retry configuration
- `timeout: Duration` - Generation timeout (default: 30s)
- `api_key: Option<String>` - Optional Bearer token for authentication
- `temperature: f32` - Sampling temperature (default: 0.7)
- `max_tokens: u32` - Token limit (default: 100)
- `top_p: f32` - Nucleus sampling (default: 0.95)

**Trait Implementation**: `CommandGenerator`

**Methods** (from trait):
```rust
#[async_trait]
impl CommandGenerator for VllmBackend {
    async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand, GeneratorError>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
    async fn shutdown(&self) -> Result<(), GeneratorError>;
}
```

**Additional Methods**:
```rust
impl VllmBackend {
    pub fn new(url: Url, model: String) -> Result<Self, GeneratorError>;
    pub fn with_api_key(self, key: String) -> Self;
    pub fn with_retry_policy(self, policy: RetryPolicy) -> Self;
    pub fn with_temperature(self, temp: f32) -> Self;
    async fn parse_response(&self, response: VllmResponse) -> Result<GeneratedCommand, GeneratorError>;
}
```

**Example**:
```rust
let backend = VllmBackend::new(
    Url::parse("https://vllm.example.com:8000")?,
    "codellama/CodeLlama-7b-hf".to_string()
)?
.with_api_key("sk-...".to_string());

let request = CommandRequest::new("list files", ShellType::Bash);
let command = backend.generate_command(&request).await?;
```

---

## Supporting Types

### T0: ModelVariant

**Purpose**: Enum representing embedded model inference backend variant.

**Location**: `src/backends/embedded/mod.rs`

**Definition**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelVariant {
    /// MLX GPU backend for Apple Silicon (macOS aarch64 only)
    MLX,
    /// Candle CPU backend for cross-platform fallback
    CPU,
}

impl ModelVariant {
    /// Auto-detect platform and return appropriate variant
    pub fn detect() -> Self {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Self::MLX;

        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        return Self::CPU;
    }
}
```

---

### T1: InferenceBackend (Trait)

**Purpose**: Trait for platform-specific inference implementations (MLX GPU or Candle CPU).

**Location**: `src/backends/embedded/mod.rs`

**Definition**:
```rust
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    /// Run inference on prompt, return generated text
    async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError>;

    /// Get backend variant (MLX or CPU)
    fn variant(&self) -> ModelVariant;

    /// Load model weights (lazy initialization)
    async fn load(&mut self) -> Result<(), GeneratorError>;

    /// Unload model and free resources
    async fn unload(&mut self) -> Result<(), GeneratorError>;
}
```

---

### T2: EmbeddedConfig

**Purpose**: Configuration for embedded model inference (temperature, context, stop tokens).

**Location**: `src/backends/embedded/mod.rs`

**Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedConfig {
    /// Sampling temperature (0.0 = deterministic, 2.0 = creative)
    pub temperature: f32,

    /// Context window size in tokens (256-32768)
    pub context_size: usize,

    /// Stop sequences to halt generation
    pub stop_tokens: Vec<String>,

    /// Maximum tokens to generate (default: 100)
    pub max_tokens: usize,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            context_size: 2048,
            stop_tokens: vec!["\n\n".to_string(), "```".to_string()],
            max_tokens: 100,
        }
    }
}
```

---

### T3: OllamaRequest

**Purpose**: Serializable request body for Ollama API.

**Location**: `src/backends/ollama.rs`

**Definition**:
```rust
#[derive(Debug, Clone, Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub stream: bool,  // Always false for caro
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
}
```

---

### T2: OllamaResponse

**Purpose**: Deserializable response from Ollama API.

**Location**: `src/backends/ollama.rs`

**Definition**:
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,  // THE GENERATED COMMAND
    pub done: bool,
    #[serde(default)]
    pub context: Vec<i32>,
    #[serde(default)]
    pub total_duration: u64,  // Nanoseconds
}
```

---

### T3: VllmRequest

**Purpose**: Serializable request body for vLLM OpenAI API.

**Location**: `src/backends/vllm.rs`

**Definition**:
```rust
#[derive(Debug, Clone, Serialize)]
pub struct VllmRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    pub n: u32,  // Number of completions (always 1)
    pub stream: bool,  // Always false
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop: Vec<String>,
}
```

---

### T4: VllmResponse

**Purpose**: Deserializable response from vLLM OpenAI API.

**Location**: `src/backends/vllm.rs`

**Definition**:
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct VllmResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<VllmChoice>,
    pub usage: VllmUsage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VllmChoice {
    pub text: String,  // THE GENERATED COMMAND
    pub index: u32,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VllmUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

---

### T5: HealthStatus

**Purpose**: Enum representing backend connection health.

**Location**: `src/backends/connection.rs`

**Definition**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Backend responding normally
    Healthy,
    /// Backend slow or intermittent failures
    Degraded,
    /// Backend unreachable or consistently failing
    Unavailable,
}
```

---

### T6: ConnectionError

**Purpose**: Error type for backend connection issues.

**Location**: `src/backends/connection.rs`

**Definition**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Backend unreachable: {url}")]
    Unreachable { url: String },

    #[error("Connection timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Invalid URL: {details}")]
    InvalidUrl { details: String },

    #[error("Authentication failed: {details}")]
    AuthenticationFailed { details: String },

    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },
}
```

---

## Configuration Extensions

### Extend UserConfiguration

**Location**: `src/models/mod.rs` (extend existing struct)

**New Fields**:
```rust
pub struct UserConfiguration {
    // ... existing fields ...

    /// Preferred backend ("ollama", "vllm", or "auto")
    pub preferred_backend: String,

    /// Ollama-specific configuration
    pub ollama_url: Option<String>,
    pub ollama_model: Option<String>,

    /// vLLM-specific configuration
    pub vllm_url: Option<String>,
    pub vllm_model: Option<String>,
    pub vllm_api_key: Option<String>,
}

impl Default for UserConfiguration {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            preferred_backend: "auto".to_string(),
            ollama_url: Some("http://localhost:11434".to_string()),
            ollama_model: Some("codellama:7b".to_string()),
            vllm_url: None,
            vllm_model: None,
            vllm_api_key: None,
        }
    }
}
```

---

## Entity Relationships

```
┌─────────────────┐
│ CommandRequest  │ (existing)
└────────┬────────┘
         │
         │ passed to
         ▼
┌─────────────────────┐
│  OllamaBackend      │◄──────────┐
│  implements:        │           │
│  CommandGenerator   │           │
└─────────┬───────────┘           │
          │                       │
          │ uses                  │ or
          ▼                       │
┌─────────────────────┐           │
│ BackendConnection   │           │
│  - url              │           │
│  - client           │           │
│  - health_status    │           │
└─────────┬───────────┘           │
          │                       │
          │ uses                  │
          ▼                       │
┌─────────────────────┐           │
│   RetryPolicy       │           │
│  - max_attempts     │           │
│  - backoff config   │           │
└─────────────────────┘           │
                                  │
┌─────────────────────┐           │
│   VllmBackend       │◄──────────┘
│   implements:       │
│   CommandGenerator  │
└─────────┬───────────┘
          │
          │ generates
          ▼
┌─────────────────────┐
│ GeneratedCommand    │ (existing)
│  - command          │
│  - explanation      │
│  - alternatives     │
└─────────────────────┘
```

---

## Validation Rules Summary

| Entity | Rule | Enforcement |
|--------|------|-------------|
| BackendConnection | URL must be valid HTTP(S) | Constructor |
| BackendConnection | consecutive_failures ≤ 10 | record_failure() |
| RetryPolicy | max_attempts > 0 | Constructor |
| RetryPolicy | base_delay_ms > 0 | Constructor |
| OllamaBackend | model name not empty | Constructor |
| OllamaBackend | temperature 0.0-2.0 | with_temperature() |
| VllmBackend | model path valid HF format | Constructor (warn only) |
| VllmBackend | max_tokens > 0 | Constructor |

---

## State Transitions

### HealthStatus State Machine

```
       ┌──────────┐
       │          │
       │ Healthy  │◄────────┐
       │          │         │
       └────┬─────┘         │
            │               │
            │ 2 failures    │ success
            ▼               │
       ┌──────────┐         │
       │          │         │
       │ Degraded │─────────┤
       │          │         │
       └────┬─────┘         │
            │               │
            │ 5+ failures   │
            ▼               │
       ┌──────────┐         │
       │          │         │
       │Unavailable│────────┘
       │          │ 3 successes
       └──────────┘
```

---

## Usage Examples

### Example 1: Create Ollama Backend
```rust
use caro::backends::OllamaBackend;
use url::Url;

let backend = OllamaBackend::new(
    Url::parse("http://localhost:11434")?,
    "codellama:7b".to_string()
)?
.with_temperature(0.7);

assert!(backend.is_available().await);
```

### Example 2: Generate Command with vLLM
```rust
use caro::backends::VllmBackend;
use caro::models::{CommandRequest, ShellType};

let backend = VllmBackend::new(
    Url::parse("https://vllm.company.com")?,
    "meta-llama/Llama-2-7b-hf".to_string()
)?
.with_api_key(std::env::var("VLLM_API_KEY")?);

let request = CommandRequest {
    input: "find large files".to_string(),
    shell: ShellType::Bash,
    // ... other fields ...
};

let command = backend.generate_command(&request).await?;
println!("Generated: {}", command.command);
```

### Example 3: Handle Connection Failures
```rust
use caro::backends::{OllamaBackend, RetryPolicy};

let backend = OllamaBackend::new(
    Url::parse("http://localhost:11434")?,
    "codellama:7b".to_string()
)?
.with_retry_policy(RetryPolicy::aggressive());

match backend.generate_command(&request).await {
    Ok(command) => println!("Success: {}", command.command),
    Err(e) => eprintln!("Failed after retries: {}", e),
}
```

---

## Testing Strategy

### Unit Tests
- `BackendConnection`: Health check state transitions
- `RetryPolicy`: Delay calculations with jitter
- `OllamaBackend`/`VllmBackend`: Request serialization
- Response parsers: Valid, malformed, missing fields

### Integration Tests
- Mock HTTP server responses (using `mockito` crate)
- End-to-end flows with test Ollama/vLLM instances
- Timeout and retry behavior
- Concurrent request handling

### Contract Tests
- Verify `CommandGenerator` trait implementation
- Ensure `GeneratedCommand` output format
- Validate error types match trait expectations

---

**Next Phase**: Generate API contracts in `contracts/` directory.
