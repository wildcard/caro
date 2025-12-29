# Contract: vLLM Backend

**Component**: `VllmBackend`
**Trait**: `CommandGenerator`
**Module**: `caro::backends::vllm`
**Purpose**: Define behavioral contract for vLLM OpenAI-compatible API backend implementation

---

## Contract Overview

The `VllmBackend` struct implements the `CommandGenerator` trait to provide command generation via vLLM's enterprise LLM serving platform using OpenAI-compatible API. This contract specifies the expected behavior, authentication, error handling, and performance characteristics.

---

## Trait Methods

### Method 1: `generate_command`

**Signature**:
```rust
async fn generate_command(
    &self,
    request: &CommandRequest
) -> Result<GeneratedCommand, GeneratorError>
```

**Purpose**: Generate a shell command from natural language input using vLLM.

**Pre-conditions**:
- `request.input` is not empty
- `request.shell` is a valid `ShellType`
- Backend is available (health check passed within last 60s)
- API key valid (if authentication required)

**Post-conditions**:
- Returns `GeneratedCommand` with non-empty `command` field
- Returns `GeneratedCommand.backend_used` == "vllm"
- Returns `GeneratedCommand.generation_time_ms` > 0

**Behavior**:

1. **HTTP Request Construction**:
   - Endpoint: `POST {self.connection.url}/v1/completions`
   - Headers:
     - `Content-Type: application/json`
     - `Authorization: Bearer {api_key}` (if configured)
   - Body: `VllmRequest` with:
     - `model`: from `self.model` (HuggingFace path)
     - `prompt`: system prompt + `request.input`
     - `max_tokens`: from `self.max_tokens` (default 100)
     - `temperature`: from `self.temperature` (default 0.7)
     - `top_p`: from `self.top_p` (default 0.95)
     - `n`: 1 (single completion)
     - `stream`: false
     - `stop`: ["\n\n", "```"] (command terminators)

2. **Request Execution**:
   - Timeout: `self.timeout` (default 30s)
   - Retry: According to `self.retry_policy`
   - Log: `debug!("vLLM request: model={}, prompt_len={}", model, prompt.len())`

3. **Response Handling**:
   - Parse JSON response to `VllmResponse`
   - Extract `choices[0].text` field (the generated command)
   - Check `choices[0].finish_reason` == "stop" (successful completion)
   - Fallback parsers if JSON malformed (fuzzy → regex)
   - Log: `info!("vLLM response: tokens={}, duration={}ms", usage.total_tokens, duration)`

4. **Command Construction**:
   - Create `GeneratedCommand` with:
     - `command`: extracted from `choices[0].text`
     - `explanation`: generated from prompt echo or empty
     - `alternatives`: empty (vLLM single completion)
     - `backend_used`: "vllm"
     - `generation_time_ms`: measured client-side
     - `confidence_score`: 0.90 (higher than Ollama for enterprise quality)

**Error Cases**:

| Condition | Error Type | Error Message | Recovery |
|-----------|------------|---------------|----------|
| vLLM unreachable | `GeneratorError::BackendUnavailable` | "vLLM server unreachable at {url}" | User action required |
| Authentication failure (401) | `GeneratorError::AuthenticationFailed` | "vLLM authentication failed. Check API key." | User action required |
| Model not found (404) | `GeneratorError::ModelNotFound` | "Model '{model}' not available on vLLM server" | User action required |
| Rate limit (429) | `GeneratorError::RateLimited` | "vLLM rate limit exceeded. Retry in {seconds}s." | Automatic retry |
| Server overload (503) | `GeneratorError::ServerOverloaded` | "vLLM server busy. Retrying..." | Automatic retry |
| Connection timeout | `GeneratorError::Timeout` | "vLLM request timed out after {timeout}s" | Retry exhausted |
| Malformed response | `GeneratorError::MalformedResponse` | "Could not parse vLLM response: {details}" | Log raw response |

**Performance Requirements**:
- P50 latency: < 10 seconds (network + remote inference)
- P95 latency: < 20 seconds
- Timeout: 30 seconds (configurable)
- Retry overhead: < 3 seconds total (2 retries with backoff)

**Example Test**:
```rust
#[tokio::test]
async fn test_vllm_generate_command_success() {
    let backend = VllmBackend::new(
        Url::parse("https://vllm.example.com").unwrap(),
        "meta-llama/Llama-2-7b-hf".to_string()
    ).unwrap()
    .with_api_key("test-key".to_string());

    let request = CommandRequest {
        input: "list all files".to_string(),
        shell: ShellType::Bash,
        context: None,
        safety_level: SafetyLevel::Moderate,
        backend_preference: None,
    };

    let result = backend.generate_command(&request).await;
    assert!(result.is_ok());

    let command = result.unwrap();
    assert!(!command.command.is_empty());
    assert_eq!(command.backend_used, "vllm");
    assert!(command.generation_time_ms > 0);
}
```

---

### Method 2: `is_available`

**Signature**:
```rust
async fn is_available(&self) -> bool
```

**Purpose**: Check if vLLM backend is reachable and responsive.

**Behavior**:

1. **Health Check Request**:
   - Endpoint: `GET {self.connection.url}/v1/models`
   - Headers: `Authorization: Bearer {api_key}` (if configured)
   - Timeout: 5 seconds
   - No retries (fast fail)

2. **Success Criteria**:
   - HTTP 200 OK response
   - Valid JSON body (list of available models)
   - Response time < 5 seconds
   - Configured model exists in model list

3. **Return Value**:
   - `true`: Backend healthy, models available, authentication valid
   - `false`: Connection failed, timeout, auth failure, or invalid response

4. **Side Effects**:
   - Update `self.connection.health_status`
   - Update `self.connection.last_health_check` timestamp
   - Log: `debug!("vLLM health check: status={}, models={}", status, model_count)`

**Performance Requirements**:
- Latency: < 1 second typical (remote call)
- Timeout: 5 seconds
- Cache: Result valid for 60 seconds

**Example Test**:
```rust
#[tokio::test]
async fn test_vllm_is_available_healthy() {
    let backend = VllmBackend::new(
        Url::parse("https://vllm.example.com").unwrap(),
        "meta-llama/Llama-2-7b-hf".to_string()
    ).unwrap()
    .with_api_key("valid-key".to_string());

    assert!(backend.is_available().await);
}

#[tokio::test]
async fn test_vllm_is_available_auth_failure() {
    let backend = VllmBackend::new(
        Url::parse("https://vllm.example.com").unwrap(),
        "meta-llama/Llama-2-7b-hf".to_string()
    ).unwrap()
    .with_api_key("invalid-key".to_string());

    assert!(!backend.is_available().await);
}
```

---

### Method 3: `backend_info`

**Signature**:
```rust
fn backend_info(&self) -> BackendInfo
```

**Purpose**: Provide metadata about this backend instance.

**Return Value**:
```rust
BackendInfo {
    backend_type: BackendType::Vllm,
    model_name: self.model.clone(),
    supports_streaming: false,  // Future enhancement
    max_tokens: self.max_tokens,
    typical_latency_ms: 8000,  // Remote inference estimate
    memory_usage_mb: 0,  // Remote backend, local memory minimal
    version: "0.1.0".to_string(),  // vLLM API compatibility version
}
```

**Behavior**:
- Pure function, no I/O
- Returns current configuration
- No side effects

**Example Test**:
```rust
#[test]
fn test_vllm_backend_info() {
    let backend = VllmBackend::new(
        Url::parse("https://vllm.example.com").unwrap(),
        "meta-llama/Llama-2-7b-hf".to_string()
    ).unwrap();

    let info = backend.backend_info();
    assert_eq!(info.backend_type, BackendType::Vllm);
    assert_eq!(info.model_name, "meta-llama/Llama-2-7b-hf");
    assert!(!info.supports_streaming);
}
```

---

### Method 4: `shutdown`

**Signature**:
```rust
async fn shutdown(&self) -> Result<(), GeneratorError>
```

**Purpose**: Gracefully cleanup backend resources.

**Behavior**:
- Close HTTP connection pool
- Flush any pending logs
- No-op for vLLM (stateless client)

**Return Value**:
- Always returns `Ok(())`

**Example Test**:
```rust
#[tokio::test]
async fn test_vllm_shutdown() {
    let backend = VllmBackend::new(
        Url::parse("https://vllm.example.com").unwrap(),
        "meta-llama/Llama-2-7b-hf".to_string()
    ).unwrap();

    assert!(backend.shutdown().await.is_ok());
}
```

---

## Constructor Contract

**Signature**:
```rust
pub fn new(url: Url, model: String) -> Result<Self, GeneratorError>
```

**Pre-conditions**:
- `url` is valid HTTP(S) URL
- `model` is non-empty string (HuggingFace path format)

**Post-conditions**:
- Returns `VllmBackend` with default configuration
- `retry_policy` set to default (3 attempts, exponential backoff)
- `timeout` set to 30 seconds
- `temperature` set to 0.7
- `api_key` set to None (optional authentication)

**Error Cases**:
- `GeneratorError::InvalidConfiguration`: If URL invalid or model empty

**Example Test**:
```rust
#[test]
fn test_vllm_new_valid() {
    let result = VllmBackend::new(
        Url::parse("https://vllm.example.com").unwrap(),
        "meta-llama/Llama-2-7b-hf".to_string()
    );
    assert!(result.is_ok());
}

#[test]
fn test_vllm_new_empty_model() {
    let result = VllmBackend::new(
        Url::parse("https://vllm.example.com").unwrap(),
        "".to_string()
    );
    assert!(result.is_err());
}
```

---

## Builder Methods

### `with_api_key`
```rust
pub fn with_api_key(mut self, key: String) -> Self
```
- Sets API key for Bearer token authentication
- Chainable builder pattern
- Validates key is non-empty (warn if empty)

### `with_retry_policy`
```rust
pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self
```
- Sets custom retry configuration
- Chainable builder pattern

### `with_temperature`
```rust
pub fn with_temperature(mut self, temp: f32) -> Self
```
- Sets sampling temperature (0.0-2.0)
- Clamps to valid range
- Chainable builder pattern

### `with_timeout`
```rust
pub fn with_timeout(mut self, timeout: Duration) -> Self
```
- Sets generation timeout
- Minimum: 5 seconds
- Chainable builder pattern

### `with_top_p`
```rust
pub fn with_top_p(mut self, top_p: f32) -> Self
```
- Sets nucleus sampling parameter (0.0-1.0)
- Default: 0.95
- Chainable builder pattern

---

## Authentication Contract

### API Key Handling

**Configuration Sources** (priority order):
1. Builder method: `.with_api_key(key)`
2. Environment variable: `VLLM_API_KEY`
3. Configuration file: `vllm_api_key` field
4. None (no authentication)

**Security Requirements**:
- API keys never logged (redacted in debug output)
- API keys stored in memory only during session
- HTTPS recommended when using API keys (warn if HTTP)

**Example**:
```rust
// Method 1: Builder
let backend = VllmBackend::new(...)
    .with_api_key("sk-...".to_string());

// Method 2: Environment variable
std::env::set_var("VLLM_API_KEY", "sk-...");
let backend = VllmBackend::new(...);  // Auto-loads from env

// Method 3: From config
let config = ConfigManager::load()?;
let backend = VllmBackend::new(
    Url::parse(&config.vllm_url.unwrap())?,
    config.vllm_model.unwrap()
)?
.with_api_key(config.vllm_api_key.unwrap_or_default());
```

---

## Integration Points

### With Safety Validator
```rust
let backend = VllmBackend::new(...)?
    .with_api_key(std::env::var("VLLM_API_KEY")?);
let validator = SafetyValidator::new(SafetyConfig::moderate())?;

let command = backend.generate_command(&request).await?;
let validation = validator.validate_command(&command.command, request.shell).await?;

if !validation.allowed {
    return Err(GeneratorError::UnsafeCommand { details: validation.explanation });
}
```

### With Configuration Management
```rust
let config = ConfigManager::load()?;
let backend = VllmBackend::new(
    Url::parse(&config.vllm_url.unwrap_or_default())?,
    config.vllm_model.unwrap_or("meta-llama/Llama-2-7b-hf".to_string())
)?
.with_api_key(config.vllm_api_key.unwrap_or_default());
```

### With Backend Fallback
```rust
let vllm = VllmBackend::new(...)?;
let ollama = OllamaBackend::new(...)?;

let backend = if vllm.is_available().await {
    Box::new(vllm) as Box<dyn CommandGenerator>
} else if ollama.is_available().await {
    Box::new(ollama) as Box<dyn CommandGenerator>
} else {
    return Err(GeneratorError::NoBackendsAvailable);
};
```

---

## Non-Functional Requirements

| Requirement | Target | Measurement |
|-------------|--------|-------------|
| NFR-001: Connection timeout | 5 seconds | HTTP client config |
| NFR-002: Generation timeout | 30 seconds | Request timeout |
| NFR-003: Retry attempts | 2 retries | Retry policy |
| NFR-004: Health check cache | 60 seconds | Timestamp comparison |
| NFR-007: Secure credentials | Redacted logs | Log sanitization |
| NFR-008: HTTPS support | Required for production | URL scheme check |
| NFR-009: Remote prompt warning | User notification | Log warning on first use |

---

## Test Coverage Requirements

- ✅ Happy path: Successful command generation
- ✅ Error: vLLM unreachable
- ✅ Error: Authentication failure (401)
- ✅ Error: Model not found (404)
- ✅ Error: Rate limit (429)
- ✅ Error: Server overload (503)
- ✅ Error: Malformed JSON response
- ✅ Error: Timeout exceeded
- ✅ Retry: Transient failure → success
- ✅ Retry: All attempts exhausted
- ✅ Health check: Available backend
- ✅ Health check: Unavailable backend
- ✅ Health check: Auth failure
- ✅ Constructor: Valid configuration
- ✅ Constructor: Invalid URL
- ✅ Builder: API key redaction
- ✅ Builder: HTTPS warning for HTTP URLs
- ✅ Integration: With safety validator
- ✅ Integration: With config management
- ✅ Integration: Backend fallback logic

**Minimum Coverage**: 90% line coverage for `vllm.rs`

---

## Fallback Behavior (NEW - Feature 004 Update)

### FR-NEW-001: Embedded Model Fallback
**MUST** fallback to embedded model when vLLM backend fails or is unavailable.

**Behavior**:
1. vLLM backend attempts connection/inference
2. If failure (connection refused, auth failure, timeout, error response):
   - Log warning: `warn!("vLLM backend failed, falling back to embedded model: {}", error)`
   - Transparently fallback to `EmbeddedModelBackend`
   - No error returned to user (seamless experience)
3. If fallback also fails: Return error (embedded model should never fail)

**Test**:
```rust
#[tokio::test]
async fn test_vllm_fallback_to_embedded() {
    // Configure vLLM backend with invalid URL
    let vllm = VllmBackend::new(
        Url::parse("https://nonexistent.example.com").unwrap(),
        "codellama/CodeLlama-7b-hf".to_string()
    ).unwrap()
    .with_api_key("invalid_key".to_string());

    let embedded = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        test_model_path()
    ).unwrap();

    // Attempt generation (should fallback automatically)
    let request = CommandRequest::new("find large files", ShellType::Bash);

    // In actual implementation, BackendSelector handles fallback
    let result = match vllm.generate_command(&request).await {
        Ok(cmd) => Ok(cmd),
        Err(_) => {
            warn!("vLLM failed, using embedded model");
            embedded.generate_command(&request).await
        }
    };

    assert!(result.is_ok(), "Must succeed via fallback");
    assert_eq!(result.unwrap().backend_used, "embedded");
}
```

### FR-NEW-002: Authentication Failure Fallback
**MUST** fallback on authentication failures (401/403) without retry.

**Test**:
```rust
#[tokio::test]
async fn test_vllm_auth_failure_fallback() {
    let vllm = VllmBackend::new(
        Url::parse("https://vllm.example.com").unwrap(),
        "codellama/CodeLlama-7b-hf".to_string()
    ).unwrap()
    .with_api_key("invalid_key".to_string());

    // Mock vLLM server returning 401
    let mock_server = setup_vllm_mock_401();

    let request = CommandRequest::new("list files", ShellType::Bash);
    let result = vllm.generate_command(&request).await;

    // Should fail immediately (no retry on auth failure)
    assert!(result.is_err());

    // BackendSelector should fallback to embedded model
    // (tested at integration level, not backend level)
}
```

### FR-NEW-003: Retry Before Fallback
**MUST** respect retry policy for transient errors before falling back.

**Test**:
```rust
#[tokio::test]
async fn test_vllm_retry_before_fallback() {
    let vllm = VllmBackend::new(
        Url::parse("https://vllm.example.com").unwrap(),
        "codellama/CodeLlama-7b-hf".to_string()
    ).unwrap()
    .with_retry_policy(RetryPolicy::default());  // 2 retries

    // Mock vLLM server that returns 503 (transient error)
    let mock_server = setup_vllm_mock_503(3);

    let request = CommandRequest::new("list files", ShellType::Bash);
    let result = vllm.generate_command(&request).await;

    // Should attempt 3 times (initial + 2 retries), then fail
    assert!(result.is_err());
    assert_eq!(mock_server.call_count(), 3);
}
```

### FR-NEW-004: Optional Backend Status
**MUST** indicate vLLM is optional enhancement, not required.

**Update to `is_available()` behavior**:
- Returns `false` when vLLM not configured or unreachable
- System continues with embedded model (no blocking error)
- Users can optionally configure vLLM via `caro init --backend vllm`

### FR-NEW-005: HTTPS Enforcement
**SHOULD** warn when using HTTP (not HTTPS) for remote vLLM servers.

**Behavior**:
- If `vllm_url` starts with `http://` (not `https://`):
  - Log warning: `warn!("vLLM URL uses HTTP, credentials may be exposed")`
  - Still allow connection (user choice)
- If `vllm_url` is localhost/127.0.0.1: No warning (local development)

---

**Contract Status**: ✅ **UPDATED** - Includes embedded model fallback behavior
