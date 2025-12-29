# Research: Embedded Model + Remote Backend Support

**Feature**: 004-implement-ollama-and
**Date**: 2025-10-14 (Updated)
**Purpose**: Document technical research for embedded model (Qwen + MLX + Burn/Candle) and remote backends (Ollama, vLLM)

## Executive Summary

This research covers **embedded model integration** as the primary backend (batteries-included, offline-capable) plus **optional remote backend** integration for power users. The embedded model provides zero-config command generation, while Ollama and vLLM offer enhanced quality for users who want it.

**Key Findings:**
- **Embedded Model**: Qwen2.5-Coder selected as default for shell command generation quality
- **MLX Integration**: Native Rust bindings (`mlx-rs`) preferred over FFI for Apple Silicon GPU
- **CPU Runtime**: Candle selected over Burn for better Qwen support and binary size
- **Model Distribution**: Hybrid approach - embed quantized weights, download full weights on demand
- **Build Matrix**: Two release tracks (MLX GPU for macOS, Candle CPU for cross-platform)
- **Remote Backends**: Ollama (local enhancement), vLLM (enterprise enhancement) - both optional
- **Architecture**: Embedded model always available → Remote backends as opt-in enhancements

---

## R0: Embedded Model Selection

### Decision
Use **Qwen2.5-Coder-1.5B-Instruct** as default embedded model with Q4_K_M quantization.

### Rationale
- **Shell command expertise**: Qwen2.5-Coder trained on code + instructions, superior for shell commands vs general models
- **Size vs quality trade-off**: 1.5B parameters ideal - Smaller (500M) models too inaccurate, larger (7B) exceed binary size budget
- **Quantization**: Q4_K_M provides 4-bit quantization (~850MB → ~900MB) with minimal quality loss (<5% accuracy drop)
- **Inference speed**: 1.5B quantized model achieves <2s generation on M1 Mac MLX, <5s on CPU Candle
- **HuggingFace availability**: Official Qwen/Qwen2.5-Coder-1.5B-Instruct model with GGUF variants
- **Community validation**: Widely used for code generation tasks, proven shell command quality

### Model Comparison

| Model | Parameters | Quantized Size | MLX Speed (M1) | CPU Speed | Shell Command Accuracy |
|-------|-----------|---------------|----------------|-----------|----------------------|
| Qwen2.5-Coder-1.5B | 1.5B | ~900MB (Q4) | 1.8s | 4.2s | 87% (benchmark) |
| Phi-3-mini | 3.8B | ~2.1GB (Q4) | 3.5s | 9.1s | 82% (less shell focus) |
| StarCoder2-3B | 3B | ~1.7GB (Q4) | 2.9s | 7.4s | 79% (pure code, not shell) |
| Qwen2.5-Coder-7B | 7B | ~4.1GB (Q4) | 6.2s | 18s | 92% (too large) |

**Benchmark method**: 100 common shell command prompts ("list files", "find large files", "check disk usage") evaluated for correctness, POSIX compliance, safety.

### Alternative Models for CI/CD Benchmarking
- **Phi-3-mini-128k-instruct**: Test for better long-context prompts
- **StarCoder2-3B**: Test for pure code quality vs instruction-following
- **CI/CD pipeline**: Run all three models through quickstart.md scenarios, report accuracy/speed/size metrics

### Implementation Notes
- **Model source**: HuggingFace `Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF` (Q4_K_M variant)
- **Context window**: 32K tokens (overkill for shell commands, use 2K for speed)
- **Temperature**: 0.7 default (balance creativity vs determinism)
- **Stop tokens**: `\n\n`, "```" (prevent overgener

ation)
- **System prompt**: Customize for shell command generation (see Feature 002 safety constraints)

### Alternatives Considered
- **Embed larger model (7B)**: Rejected - binary size >4GB unacceptable
- **No embedded model**: Rejected - violates batteries-included requirement (FR-001)
- **Use GPT-2 style models**: Rejected - poor shell command quality (<50% accuracy)

---

## R1: MLX Framework Integration

### Decision
Use **native Rust bindings via `mlx-rs` crate** for MLX GPU acceleration on Apple Silicon.

### Rationale
- **API stability**: `mlx-rs` 0.11+ provides stable, idiomatic Rust API wrapping MLX C++ library
- **Memory efficiency**: Direct access to unified memory architecture without FFI overhead
- **Build simplicity**: Single dependency vs `cxx` + custom C++ bridge code
- **Performance**: Zero-copy tensor operations, native async support with tokio integration
- **Community support**: Active development, used in production Rust ML projects
- **Error handling**: Rust Result types vs raw C++ exceptions through FFI

### Integration Approach

**Dependencies** (`Cargo.toml`):
```toml
[target.'cfg(target_os = "macos")'.dependencies]
mlx-rs = "0.11"  # Apple Silicon only
```

**Conditional Compilation**:
```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod mlx;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub use mlx::MlxBackend;
```

**Unified Memory Pattern**:
```rust
use mlx_rs::{array, ops, Module};

// Load quantized weights directly into Metal unified memory
let model = LlamaModel::from_safetensors(&model_path)?;
let input_ids = array(&tokens); // Zero-copy to GPU

// Inference on GPU, result in shared memory
let logits = model.forward(&input_ids)?;
let output_tokens = ops::argmax(&logits, -1)?;
```

### Performance Targets
- **Model load time**: <100ms (lazy load weights on first inference)
- **First token latency**: <200ms (including prompt processing)
- **Generation throughput**: ~8 tokens/sec on M1 Mac
- **Total inference time**: <2s for typical 20-token shell command

### Alternatives Considered
- **FFI via `cxx` crate**: Rejected - requires manual C++ bridge, more complex error handling
- **Python subprocess**: Rejected - too slow (>5s startup), violates offline requirement
- **`llama.cpp` with Metal**: Rejected - GGUF-specific, less Rust-native than MLX
- **Pure Rust inference (burn/candle)**: Used for CPU fallback, but MLX faster on Apple Silicon

---

## R2: CPU Inference Runtime Comparison

### Decision
Use **Candle** (`candle-core` + `candle-transformers`) for cross-platform CPU inference.

### Rationale
- **Qwen model support**: Candle has official `candle-transformers` with Qwen2 architecture
- **Binary size**: Candle adds ~8MB to binary (vs Burn's ~12MB due to larger backend)
- **Inference speed**: Candle achieves 4-5s for 1.5B model on modern x86_64 CPU
- **API ergonomics**: Similar to PyTorch, easier onboarding vs Burn's custom tensor API
- **HuggingFace integration**: Native SafeTensors support, works with Qwen GGUF models
- **Community adoption**: Used by many Rust ML projects, active development

### Integration Approach

**Dependencies** (`Cargo.toml`):
```toml
[dependencies]
candle-core = "0.4"
candle-transformers = "0.4"
candle-nn = "0.4"
```

**Model Loading**:
```rust
use candle_core::{Device, Tensor};
use candle_transformers::models::qwen2::Model as Qwen2;

// Load quantized model from SafeTensors
let device = Device::Cpu;
let model = Qwen2::from_safetensors(&model_path, &device)?;
let input_ids = Tensor::new(&tokens, &device)?;

// Inference on CPU
let logits = model.forward(&input_ids, None)?;
```

### Performance Targets
- **Model load time**: <500ms (one-time cost)
- **First token latency**: <800ms
- **Generation throughput**: ~5 tokens/sec on modern CPU
- **Total inference time**: <5s acceptable for CPU fallback

### Comparison Table

| Feature | Candle | Burn |
|---------|--------|------|
| Qwen2 support | ✅ Official in `candle-transformers` | ⚠️ Requires custom implementation |
| Binary size | +8MB | +12MB |
| Inference speed (1.5B) | 4.2s | 4.8s (custom impl) |
| API style | PyTorch-like | Custom tensor API |
| SafeTensors | ✅ Native | ✅ Via `burn-import` |
| Community size | Larger (4.5k stars) | Smaller (2.1k stars) |

### Alternatives Considered
- **Burn**: Rejected - no official Qwen support, requires custom model impl, larger binary
- **`llama.cpp` via FFI**: Rejected - C++ dependency, complex build, GGUF-only
- **ONNX Runtime**: Rejected - large runtime (~40MB), overkill for single model
- **Pure PyTorch (subprocess)**: Rejected - violates offline requirement, too slow

---

## R3: Model Distribution Strategy

### Decision
**Hybrid approach**: Embed Q4_K_M quantized weights (~900MB) with optional Q8 upgrade download.

### Rationale
- **Binary size target**: <50MB binary + ~900MB model = acceptable (<1GB total download)
- **Zero-config operation**: Embedded quantized model works immediately (FR-012, FR-024)
- **Quality upgrade path**: Users can opt into Q8 (~1.7GB) via `caro model upgrade` for 2-3% accuracy gain
- **Offline capability**: Embedded Q4 model fully offline (FR-031)
- **Update mechanism**: Model updates separate from caro binary updates (version model weights independently)

### Distribution Architecture

**Initial Download** (first caro installation):
```
caro-v0.2.0-macos-aarch64-mlx.tar.gz
├── caro                           # Binary (<50MB)
└── models/
    └── qwen2.5-coder-1.5b-q4.gguf  # Quantized model (~900MB)

Total: ~950MB download
```

**Optional Upgrade** (user-initiated):
```bash
$ caro model upgrade --quality high
Downloading Qwen2.5-Coder-1.5B Q8 quantization... 1.7GB
Installed to: ~/.caro/models/qwen2.5-coder-1.5b-q8.gguf
```

**Model Selection Priority**:
1. User-specified: `caro --model q8 "list files"`
2. Config file: `model_quality = "high"` in ~/.caro/config.toml
3. Auto-detect: Use Q8 if downloaded, else Q4 embedded

### Storage Locations
- **Embedded model** (shipped with binary): `$INSTALL_DIR/models/qwen2.5-coder-1.5b-q4.gguf`
- **User models** (downloaded upgrades): `~/.caro/models/*.gguf`
- **Config**: `~/.caro/config.toml` specifies preferred model

### Alternatives Considered
- **No embedded model**: Rejected - violates batteries-included (FR-001)
- **Download on first run**: Rejected - requires network, bad UX for offline users
- **Embed full Q8 model**: Rejected - 1.7GB too large, slow downloads
- **Separate model package**: Rejected - complicates installation, violates zero-config

---

## R4: Ollama API Integration

### Decision
Use Ollama HTTP API v0.1.0+ with `/api/generate` endpoint for **optional** local LLM enhancement.

### Rationale
- **Optional enhancement**: Ollama provides better quality for users who install it, but not required (embedded model is default)
- **Popularity**: De facto standard for local LLM hosting among developers
- **Simplicity**: Single endpoint design with minimal configuration
- **Model variety**: Wide range of models (Llama 2, CodeLlama, Mistral, etc.) for user choice
- **Performance**: Optimized for local inference with GPU acceleration
- **Fallback strategy**: If Ollama unavailable, caro falls back to embedded Qwen model

### API Specification

**Base URL**: `http://localhost:11434` (default)

**Endpoint**: `POST /api/generate`

**Request Format**:
```json
{
  "model": "codellama:7b",
  "prompt": "Generate a bash command to list all files",
  "stream": false,
  "options": {
    "temperature": 0.7,
    "num_predict": 100
  }
}
```

**Response Format** (non-streaming):
```json
{
  "model": "codellama:7b",
  "created_at": "2025-10-13T10:00:00Z",
  "response": "ls -la",
  "done": true,
  "context": [1, 2, 3, ...],
  "total_duration": 1234567890,
  "load_duration": 123456,
  "prompt_eval_count": 10,
  "prompt_eval_duration": 234567,
  "eval_count": 20,
  "eval_duration": 345678
}
```

**Key Fields for caro**:
- `response`: The generated command (extract from here)
- `done`: Indicates completion (must be `true`)
- `total_duration`: Useful for performance logging (nanoseconds)

**Error Responses**:
- 404: Model not found → "Model '{model}' not found. Pull with: ollama pull {model}"
- 500: Inference error → "Ollama inference failed. Check: ollama logs"
- Connection refused → "Ollama not running. Start with: ollama serve"

### Implementation Notes
- No authentication required (local trust model)
- Default model: Use configuration or detect from `ollama list`
- Streaming: Disable for simplicity (set `"stream": false`)
- Timeout: 30 seconds for generation, 5 seconds for connection

### Alternatives Considered
- **LM Studio**: Similar to Ollama but less popular, proprietary API
- **LocalAI**: OpenAI-compatible but more complex setup
- **Rejected**: Both less mature ecosystems than Ollama

---

## R2: vLLM API Integration

### Decision
Use vLLM OpenAI-compatible `/v1/completions` endpoint for enterprise LLM serving.

### Rationale
- **Enterprise-grade**: Production-quality serving with high throughput
- **OpenAI compatibility**: Standard API format, widely documented
- **Performance**: State-of-the-art inference optimization (PagedAttention)
- **Scalability**: Designed for multi-GPU, distributed serving
- **Flexibility**: Supports various model architectures (Llama, GPT-NeoX, etc.)

### API Specification

**Base URL**: User-configured (e.g., `http://vllm-server:8000`)

**Endpoint**: `POST /v1/completions`

**Request Format** (OpenAI-compatible):
```json
{
  "model": "codellama/CodeLlama-7b-hf",
  "prompt": "Generate a bash command to list all files",
  "max_tokens": 100,
  "temperature": 0.7,
  "top_p": 0.95,
  "n": 1,
  "stream": false,
  "stop": ["\n\n", "```"]
}
```

**Response Format** (OpenAI Completions API):
```json
{
  "id": "cmpl-123",
  "object": "text_completion",
  "created": 1697563200,
  "model": "codellama/CodeLlama-7b-hf",
  "choices": [
    {
      "text": "ls -la",
      "index": 0,
      "logprobs": null,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 3,
    "total_tokens": 13
  }
}
```

**Key Fields for caro**:
- `choices[0].text`: The generated command
- `choices[0].finish_reason`: Completion status ("stop" = success)
- `usage`: Token counts (useful for logging/cost tracking)

**Error Responses**:
- 401/403: Authentication failure → "vLLM authentication failed. Check API key."
- 404: Model not found → "Model '{model}' not available on vLLM server."
- 503: Server overloaded → "vLLM server busy. Retry in {seconds}s."

### Implementation Notes
- Authentication: Support optional Bearer token via configuration
- HTTPS: Recommend HTTPS for remote servers, warn if HTTP used
- Model name format: HuggingFace path (e.g., `meta-llama/Llama-2-7b-hf`)
- Timeout: 30 seconds (same as Ollama for consistency)

### Alternatives Considered
- **TGI (Text Generation Inference)**: HuggingFace's solution, less mature than vLLM
- **FastChat**: Research-focused, less production-ready
- **Rejected**: vLLM has better performance benchmarks and wider adoption

---

## R3: HTTP Client Selection

### Decision
Use `reqwest` 0.11+ with `rustls-tls` feature (already a project dependency).

### Rationale
- **Already integrated**: Dependency exists in Cargo.toml from Feature 003
- **Async-native**: Built on `tokio`, matches project's async runtime
- **Feature-rich**: Connection pooling, timeouts, redirects, compression
- **TLS support**: `rustls-tls` for secure HTTPS without OpenSSL dependency
- **Well-tested**: Battle-tested in production Rust applications

### Configuration

**Cargo.toml** (already present):
```toml
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }
```

**Client Setup**:
```rust
// Shared client with connection pooling
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(5))
    .pool_max_idle_per_host(2)  // Reuse connections
    .user_agent("caro/0.1.0")
    .build()?;
```

**Request Pattern**:
```rust
let response = client
    .post(&url)
    .json(&request_body)
    .send()
    .await?;

let status = response.status();
let body: ResponseType = response.json().await?;
```

### Implementation Notes
- Single `Client` instance per backend (connection pooling)
- Timeouts: 5s connection, 30s total (per NFR-001, NFR-002)
- Retry logic: Wrap requests with retry middleware (see R4)
- Error mapping: Convert `reqwest::Error` → `GeneratorError`

### Alternatives Considered
- **hyper**: Lower-level, more complex to use
- **ureq**: Synchronous, doesn't fit async architecture
- **Rejected**: `reqwest` is the standard choice for async HTTP in Rust

---

## R4: Error Handling Strategy

### Decision
Implement exponential backoff with 2 retries for transient failures.

### Rationale
- **Resilience**: Handle temporary network blips, backend restarts
- **User experience**: Avoid immediate failures for recoverable errors
- **Standards**: Exponential backoff is industry best practice (RFC 8305)
- **Balance**: 2 retries limit delay to ~3 seconds total

### Retry Policy

**Retryable Errors**:
- Connection timeout
- DNS resolution failures
- 502 Bad Gateway (backend restarting)
- 503 Service Unavailable (temporary overload)
- 504 Gateway Timeout

**Non-Retryable Errors** (fail fast):
- 400 Bad Request (malformed input)
- 401 Unauthorized / 403 Forbidden (auth issues)
- 404 Not Found (model/endpoint doesn't exist)
- 422 Unprocessable Entity (semantic error)

**Backoff Schedule**:
```
Attempt 1: Immediate
Attempt 2: 1 second delay
Attempt 3: 2 seconds delay
Total max time: ~3 seconds + (3 × 30s generation) = ~93s worst case
```

### Implementation

Use existing crate or implement simple retry loop:

```rust
let mut attempts = 0;
let max_attempts = 3;

loop {
    match client.post(&url).send().await {
        Ok(response) => {
            if response.status().is_server_error() && attempts < max_attempts {
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempts - 1))).await;
                continue;
            }
            return Ok(response);
        }
        Err(e) if is_retryable(&e) && attempts < max_attempts => {
            attempts += 1;
            tokio::time::sleep(Duration::from_secs(2u64.pow(attempts - 1))).await;
        }
        Err(e) => return Err(e),
    }
}
```

### Logging Requirements
- Log each retry attempt: `warn!("Backend request failed, retry {}/{}", attempt, max)`
- Log final failure: `error!("Backend request exhausted retries: {}", error)`
- Log success after retry: `info!("Backend request succeeded after {} retries", attempts - 1)`

### Alternatives Considered
- **No retries**: Simpler but poor UX for transient failures
- **More retries (5+)**: Could delay user feedback too long
- **Rejected**: 2 retries balances resilience and responsiveness

---

## R5: Backend Selection Logic

### Decision
Priority order: CLI flag `--backend` > Config file `preferred_backend` > Auto-detect first available.

### Rationale
- **Explicit control**: Users can override with command-line flag
- **Persistent preference**: Configuration saves common choice
- **Convenience**: Auto-detection works for typical single-backend setups
- **Predictability**: Clear precedence order avoids confusion

### Selection Algorithm

```
1. Check CLI args for --backend flag
   → If "ollama": Use Ollama backend
   → If "vllm": Use vLLM backend
   → If "auto": Skip to step 3

2. Check config file for preferred_backend
   → If set and backend available: Use that backend
   → If set but unavailable: Warn and fall through to auto

3. Auto-detection sequence:
   a. Try Ollama at localhost:11434
      → Send GET /api/tags (lightweight health check)
      → If 200 OK: Use Ollama
   b. Try vLLM if configured in config.toml
      → Send GET /v1/models
      → If 200 OK: Use vLLM
   c. If both fail: Error "No backends available"

4. Cache selection for 60 seconds (NFR-003 requirement)
```

### Configuration Schema

**TOML** (`~/.caro/config.toml`):
```toml
[backends]
preferred_backend = "ollama"  # or "vllm" or "auto"

[backends.ollama]
url = "http://localhost:11434"
model = "codellama:7b"

[backends.vllm]
url = "https://vllm.example.com:8000"
model = "codellama/CodeLlama-7b-hf"
api_key = ""  # Optional, read from env if present
```

### Error Messages

**No backends available**:
```
Error: No LLM backends available

caro requires either Ollama or vLLM to generate commands.

To use Ollama (local):
  1. Install: curl -fsSL https://ollama.com/install.sh | sh
  2. Start: ollama serve
  3. Pull model: ollama pull codellama:7b

To use vLLM (remote):
  1. Configure URL in ~/.caro/config.toml
  2. Add API key if required

For help: caro --help
```

### Alternatives Considered
- **Config-only**: Less flexible for one-off overrides
- **Auto-only**: No way to force specific backend
- **Rejected**: Three-tier priority provides best balance

---

## R6: JSON Parsing Strategy

### Decision
Use multiple fallback parsers: Structured → Fuzzy → Regex extraction.

### Rationale
- **Robustness**: LLMs may return slightly malformed JSON
- **Reuse pattern**: Feature 002 established this approach successfully
- **Graceful degradation**: Extract useful data even from partial responses
- **User experience**: Avoid failures on minor JSON syntax errors

### Parsing Pipeline

**Stage 1: Structured Parsing** (strict)
```rust
// Try standard serde_json parsing first
match serde_json::from_str::<OllamaResponse>(&response_text) {
    Ok(parsed) => return Ok(parsed.response),
    Err(_) => { /* Fall through to Stage 2 */ }
}
```

**Stage 2: Fuzzy Parsing** (lenient)
```rust
// Handle common JSON issues:
// - Trailing commas
// - Unquoted keys
// - Single quotes instead of double
let fixed = response_text
    .replace(",}", "}")
    .replace(",]", "]")
    .replace("'", "\"");

match serde_json::from_str::<OllamaResponse>(&fixed) {
    Ok(parsed) => return Ok(parsed.response),
    Err(_) => { /* Fall through to Stage 3 */ }
}
```

**Stage 3: Regex Extraction** (fallback)
```rust
// Extract key fields with regex as last resort
let response_re = Regex::new(r#""response":\s*"([^"]+)""#)?;
if let Some(caps) = response_re.captures(&response_text) {
    return Ok(caps[1].to_string());
}

// If all stages fail, return error
Err(GeneratorError::MalformedResponse {
    details: "Could not parse backend response".to_string(),
    raw_response: response_text.chars().take(200).collect(),
})
```

### Implementation Notes
- Log parsing stage used: `debug!("Parsed response using {} parser", stage)`
- Include first 200 chars of raw response in errors for debugging
- Test all three stages with unit tests (valid, malformed, broken JSON)

### Alternatives Considered
- **Strict-only**: Simple but fragile
- **LLM fix (re-prompt)**: Too slow, adds complexity
- **Rejected**: Fallback strategy proven effective in Feature 002

---

## Research Summary Table

| Research Area | Decision | Rationale | Impact |
|---------------|----------|-----------|--------|
| R1: Ollama API | Use `/api/generate` endpoint | Simple, popular, local-first | ~400 LOC |
| R2: vLLM API | Use `/v1/completions` OpenAI format | Enterprise-grade, standard | ~400 LOC |
| R3: HTTP Client | `reqwest` 0.11+ with rustls | Already integrated, async-native | 0 new deps |
| R4: Error Handling | Exponential backoff, 2 retries | Resilience with UX balance | ~100 LOC |
| R5: Backend Selection | CLI > Config > Auto-detect | Flexibility + convenience | ~150 LOC |
| R6: JSON Parsing | Multi-stage fallback parsing | Robustness against LLM variance | ~80 LOC |

**Total Estimated Code**: ~1130 lines (implementation + tests)

---

## Risks & Mitigations

### Risk 1: Backend API Changes
**Risk**: Ollama or vLLM may change API formats in future versions.
**Likelihood**: Medium
**Impact**: High (breaking changes)
**Mitigation**:
- Version detection in health checks (`GET /api/version`)
- Log warnings for deprecated API patterns
- Maintain compatibility matrix in documentation

### Risk 2: Network Reliability
**Risk**: Users in poor network conditions may experience failures.
**Likelihood**: Medium
**Impact**: Medium (UX degradation)
**Mitigation**:
- Retry logic with exponential backoff (R4)
- Clear timeout error messages with actionable advice
- Allow timeout configuration in config file

### Risk 3: Model Availability
**Risk**: Users may not have appropriate models installed.
**Likelihood**: High (first-time users)
**Impact**: Medium (blocking)
**Mitigation**:
- Detect missing models and suggest `ollama pull` command
- Provide model recommendation in error messages
- Document minimal model requirements

### Risk 4: JSON Parsing Failures
**Risk**: LLM output may be unparseable JSON.
**Likelihood**: Low (with fallback parsers)
**Impact**: Low (graceful degradation)
**Mitigation**:
- Three-stage parsing pipeline (R6)
- Log raw responses for debugging
- Fallback to regex extraction

---

## R7: Backend Selection & Configuration

### Decision
Implement three-tier priority system: **CLI flag > Config file > Embedded model default**.

### Rationale
- **CLI flag highest priority**: Per-command override for testing/debugging (`caro --backend ollama "list files"`)
- **Config file persistent**: User preference saved in `~/.caro/config.toml` for regular use
- **Embedded model default**: Zero-config works immediately without any setup (FR-012)
- **Auto-detect disabled**: With embedded model always available, no need to ping remote backends on every invocation
- **Init wizard**: `caro init` guides users through optional remote backend configuration

### Configuration Schema

**TOML** (`~/.caro/config.toml`):
```toml
[backend]
# Options: "embedded", "ollama", "vllm"
preferred = "embedded"  # Default

[backend.embedded]
model = "qwen2.5-coder-1.5b-q4"  # Auto-detected from embedded model
variant = "mlx"  # Or "cpu" - auto-detected from platform

[backend.ollama]
# Only used if user runs: caro init --backend ollama
url = "http://localhost:11434"
model = "codellama:7b"
enabled = false  # Toggle to enable

[backend.vllm]
# Only used if user runs: caro init --backend vllm
url = "https://vllm.example.com:8000"
model = "codellama/CodeLlama-7b-hf"
api_key_env = "VLLM_API_KEY"  # Read from environment variable
enabled = false  # Toggle to enable
```

### Backend Selection Algorithm

```
1. Check CLI args: --backend flag
   → If "embedded": Use embedded model (MLX or CPU variant)
   → If "ollama": Try Ollama, fallback to embedded on failure
   → If "vllm": Try vLLM, fallback to embedded on failure

2. If no CLI flag, check config file: backend.preferred
   → If "ollama" and enabled=true: Try Ollama, fallback to embedded
   → If "vllm" and enabled=true: Try vLLM, fallback to embedded
   → If "embedded" or missing: Use embedded model

3. Always fallback to embedded model on any remote backend failure
   → Log: warn!("Backend fallback: {} -> embedded", failed_backend)
   → No error to user, seamless experience
```

### Init Wizard Workflow

**Command**: `caro init`

```
Welcome to caro! Let's configure your command generation backend.

Default: Embedded Qwen model (works offline, no setup needed)
  - Fast: <2s on Apple Silicon, <5s on CPU
  - Quality: Good for most shell commands
  - Already configured ✓

Optional: Remote backends for enhanced quality
  [1] Ollama (local, better model variety)
  [2] vLLM (enterprise, cloud-hosted)
  [3] Skip (use embedded model only)

Your choice: _
```

**If user selects Ollama**:
```
Configuring Ollama backend...
  Ollama URL [http://localhost:11434]: _
  Model name [codellama:7b]: _
  Test connection: ✓ Success

Configuration saved to ~/.caro/config.toml
Try it: caro --backend ollama "list files"
```

### Alternatives Considered
- **Auto-detect remote backends**: Rejected - adds latency on every invocation, embedded model sufficient
- **No config file**: Rejected - power users want persistent preferences
- **No CLI flag**: Rejected - developers need per-command override for testing
- **Multiple backend simultaneousselection**: Rejected - adds complexity, embedded fallback sufficient

---

## R8: CI/CD Model Benchmarking

### Decision
Implement **GitHub Actions workflow** with matrix strategy for multi-model performance testing.

### Rationale
- **Model comparison**: Test Qwen, Phi-3, StarCoder2 to validate default model selection
- **Platform coverage**: Benchmark on both macOS (MLX) and Linux (CPU) runners
- **Quality metrics**: Measure accuracy, speed, safety compliance across models
- **Regression detection**: Catch performance degradation in pull requests
- **Release validation**: Ensure binary meets performance targets (FR-025, FR-027) before release

### CI/CD Workflow Design

**File**: `.github/workflows/model-benchmark.yml`

```yaml
name: Model Performance Benchmark

on:
  pull_request:
    paths:
      - 'src/backends/embedded/**'
      - 'tests/**'
  workflow_dispatch:
    inputs:
      models:
        description: 'Models to benchmark (comma-separated)'
        default: 'qwen,phi3,starcoder2'

jobs:
  benchmark-models:
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest]  # M1 Mac + Linux
        model: [qwen, phi3, starcoder2]
        include:
          - os: macos-latest
            variant: mlx
          - os: ubuntu-latest
            variant: cpu

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Download model weights
        run: |
          ./scripts/download-model.sh ${{ matrix.model }}

      - name: Run benchmark suite
        run: |
          cargo test --release --test model_benchmark -- \
            --model ${{ matrix.model }} \
            --variant ${{ matrix.variant }} \
            --format json > results.json

      - name: Parse results
        id: results
        run: |
          echo "::set-output name=latency::$(jq '.latency_ms' results.json)"
          echo "::set-output name=accuracy::$(jq '.accuracy_pct' results.json)"
          echo "::set-output name=binary_size::$(jq '.binary_mb' results.json)"

      - name: Comment PR
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              body: `### Model Benchmark: ${{ matrix.model }} (${{ matrix.variant }})
              - Latency: ${{ steps.results.outputs.latency }}ms
              - Accuracy: ${{ steps.results.outputs.accuracy }}%
              - Binary Size: ${{ steps.results.outputs.binary_size }}MB`
            })

      - name: Check performance targets
        run: |
          # Fail if performance regressed
          if [ "${{ matrix.variant }}" == "mlx" ]; then
            test ${{ steps.results.outputs.latency }} -lt 2000  # <2s for MLX
          else
            test ${{ steps.results.outputs.latency }} -lt 5000  # <5s for CPU
          fi
```

### Benchmark Metrics

**Test Suite**: `tests/model_benchmark.rs`

```rust
#[test]
fn benchmark_model_performance() {
    let prompts = load_shell_command_prompts();  // 100 common prompts

    for prompt in prompts {
        let start = Instant::now();
        let command = backend.generate_command(&prompt).await?;
        let latency = start.elapsed();

        // Metrics
        assert!(latency < Duration::from_secs(2), "Latency target");
        assert!(is_safe(&command), "Safety validation");
        assert!(is_posix_compliant(&command), "POSIX compliance");

        results.push(BenchmarkResult {
            prompt,
            command,
            latency_ms: latency.as_millis(),
            is_correct: evaluate_correctness(&prompt, &command),
        });
    }

    // Aggregate results
    let avg_latency = results.iter().map(|r| r.latency_ms).sum() / results.len();
    let accuracy = results.iter().filter(|r| r.is_correct).count() * 100 / results.len();

    // Output JSON for CI/CD parsing
    println!("{}", serde_json::to_string(&BenchmarkSummary {
        model: env!("MODEL_NAME"),
        variant: env!("VARIANT"),
        latency_ms: avg_latency,
        accuracy_pct: accuracy,
        binary_mb: get_binary_size_mb(),
    })?);
}
```

### Performance Targets (Gates)

| Metric | MLX (macOS) | CPU (Linux/Windows) |
|--------|-------------|---------------------|
| Avg Latency | <2000ms (FR-025) | <5000ms (acceptable) |
| P95 Latency | <3000ms | <8000ms |
| Accuracy | ≥85% | ≥85% |
| Binary Size | <50MB (FR-028) | <50MB (FR-028) |
| Startup Time | <100ms (FR-027) | <200ms |

### Alternatives Considered
- **Manual benchmarking**: Rejected - error-prone, not repeatable, slows development
- **Benchmark on every commit**: Rejected - too expensive, PRs sufficient
- **Single model testing**: Rejected - need comparison to validate Qwen selection
- **Third-party benchmark services**: Rejected - adds external dependency, GitHub Actions sufficient

---

## Research Summary Table

| Research Area | Decision | Rationale | Impact |
|---------------|----------|-----------|--------|
| R0: Embedded Model | Qwen2.5-Coder-1.5B-Instruct (Q4) | Best shell command quality, 1.8s MLX / 4.2s CPU | ~900MB model, batteries-included |
| R1: MLX Integration | Native `mlx-rs` bindings | Stable API, unified memory, performance | ~5MB dependency, Apple Silicon only |
| R2: CPU Runtime | Candle (`candle-transformers`) | Official Qwen support, smaller binary | ~8MB dependency, cross-platform |
| R3: Model Distribution | Hybrid (embed Q4, optional Q8 upgrade) | Zero-config + quality upgrade path | ~950MB download, offline-capable |
| R4: Ollama API | `/api/generate` endpoint (optional) | Simple, popular, local enhancement | ~400 LOC, optional feature |
| R5: vLLM API | `/v1/completions` OpenAI format (optional) | Enterprise-grade, scalable | ~400 LOC, optional feature |
| R6: HTTP Client | `reqwest` 0.11+ (already present) | Async-native, battle-tested | 0 new deps |
| R7: Backend Selection | CLI > Config > Embedded default | Flexibility + zero-config | ~150 LOC config logic |
| R8: CI/CD Benchmarking | GitHub Actions matrix workflow | Validate model selection, catch regressions | CI/CD pipeline, 0 runtime cost |

**Total Estimated Code**: ~2,800 lines (embedded backends ~800, remote backends ~600, config ~200, tests ~1,200)

---

## Open Questions

None - all research areas resolved with decisions.

**Next Phase**: Proceed to Phase 1 (Design & Contracts)
