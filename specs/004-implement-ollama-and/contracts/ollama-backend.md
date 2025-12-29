# Contract: Ollama Backend

**Component**: `OllamaBackend`
**Trait**: `CommandGenerator`
**Module**: `caro::backends::ollama`
**Purpose**: Define behavioral contract for Ollama HTTP API backend implementation

---

## Contract Overview

The `OllamaBackend` struct implements the `CommandGenerator` trait to provide command generation via Ollama's local LLM HTTP API. This contract specifies the expected behavior, error handling, and performance characteristics.

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

**Purpose**: Generate a shell command from natural language input using Ollama.

**Pre-conditions**:
- `request.input` is not empty
- `request.shell` is a valid `ShellType`
- Backend is available (health check passed within last 60s)

**Post-conditions**:
- Returns `GeneratedCommand` with non-empty `command` field
- Returns `GeneratedCommand.backend_used` == "ollama"
- Returns `GeneratedCommand.generation_time_ms` > 0

**Behavior**:

1. **HTTP Request Construction**:
   - Endpoint: `POST {self.connection.url}/api/generate`
   - Headers: `Content-Type: application/json`
   - Body: `OllamaRequest` with:
     - `model`: from `self.model`
     - `prompt`: system prompt + `request.input`
     - `stream`: false
     - `options.temperature`: from `self.temperature`
     - `options.num_predict`: from `self.max_tokens`

2. **Request Execution**:
   - Timeout: `self.timeout` (default 30s)
   - Retry: According to `self.retry_policy`
   - Log: `debug!("Ollama request: model={}, prompt_len={}", model, prompt.len())`

3. **Response Handling**:
   - Parse JSON response to `OllamaResponse`
   - Extract `response.response` field (the generated command)
   - Fallback parsers if JSON malformed (fuzzy → regex)
   - Log: `debug!("Ollama response: duration={}ms", duration)`

4. **Command Construction**:
   - Create `GeneratedCommand` with:
     - `command`: extracted from `response.response`
     - `explanation`: generated from prompt echo or empty
     - `alternatives`: empty (Ollama doesn't provide)
     - `backend_used`: "ollama"
     - `generation_time_ms`: from `response.total_duration` (convert ns → ms)
     - `confidence_score`: 0.85 (default, Ollama doesn't provide)

**Error Cases**:

| Condition | Error Type | Error Message | Recovery |
|-----------|------------|---------------|----------|
| Ollama not running | `GeneratorError::BackendUnavailable` | "Ollama not running at {url}. Start with: ollama serve" | User action required |
| Model not found (404) | `GeneratorError::ModelNotFound` | "Model '{model}' not found. Pull with: ollama pull {model}" | User action required |
| Connection timeout | `GeneratorError::Timeout` | "Ollama request timed out after {timeout}s" | Retry exhausted |
| Malformed response | `GeneratorError::MalformedResponse` | "Could not parse Ollama response: {details}" | Log raw response |
| Rate limit (429) | `GeneratorError::RateLimited` | "Ollama rate limit exceeded. Retry in {seconds}s." | Automatic retry |

**Performance Requirements**:
- P50 latency: < 5 seconds (local inference)
- P95 latency: < 10 seconds
- Timeout: 30 seconds (configurable)
- Retry overhead: < 3 seconds total (2 retries with backoff)

**Example Test**:
```rust
#[tokio::test]
async fn test_ollama_generate_command_success() {
    let backend = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string()
    ).unwrap();

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
    assert_eq!(command.backend_used, "ollama");
    assert!(command.generation_time_ms > 0);
}
```

---

### Method 2: `is_available`

**Signature**:
```rust
async fn is_available(&self) -> bool
```

**Purpose**: Check if Ollama backend is reachable and responsive.

**Behavior**:

1. **Health Check Request**:
   - Endpoint: `GET {self.connection.url}/api/tags`
   - Timeout: 5 seconds
   - No retries (fast fail)

2. **Success Criteria**:
   - HTTP 200 OK response
   - Valid JSON body (list of models)
   - Response time < 5 seconds

3. **Return Value**:
   - `true`: Backend healthy, models available
   - `false`: Connection failed, timeout, or invalid response

4. **Side Effects**:
   - Update `self.connection.health_status`
   - Update `self.connection.last_health_check` timestamp
   - Log: `debug!("Ollama health check: status={}", status)`

**Performance Requirements**:
- Latency: < 500ms typical
- Timeout: 5 seconds
- Cache: Result valid for 60 seconds

**Example Test**:
```rust
#[tokio::test]
async fn test_ollama_is_available_healthy() {
    let backend = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string()
    ).unwrap();

    assert!(backend.is_available().await);
}

#[tokio::test]
async fn test_ollama_is_available_unreachable() {
    let backend = OllamaBackend::new(
        Url::parse("http://localhost:9999").unwrap(),  // Wrong port
        "codellama:7b".to_string()
    ).unwrap();

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
    backend_type: BackendType::Ollama,
    model_name: self.model.clone(),
    supports_streaming: false,
    max_tokens: self.max_tokens.unwrap_or(100),
    typical_latency_ms: 3000,  // Local inference estimate
    memory_usage_mb: 4000,  // Approximate for 7B model
    version: "0.1.0".to_string(),  // Ollama API version
}
```

**Behavior**:
- Pure function, no I/O
- Returns current configuration
- No side effects

**Example Test**:
```rust
#[test]
fn test_ollama_backend_info() {
    let backend = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string()
    ).unwrap();

    let info = backend.backend_info();
    assert_eq!(info.backend_type, BackendType::Ollama);
    assert_eq!(info.model_name, "codellama:7b");
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
- No-op for Ollama (stateless client)

**Return Value**:
- Always returns `Ok(())`

**Example Test**:
```rust
#[tokio::test]
async fn test_ollama_shutdown() {
    let backend = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string()
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
- `model` is non-empty string

**Post-conditions**:
- Returns `OllamaBackend` with default configuration
- `retry_policy` set to default (3 attempts, exponential backoff)
- `timeout` set to 30 seconds
- `temperature` set to 0.7

**Error Cases**:
- `GeneratorError::InvalidConfiguration`: If URL invalid or model empty

**Example Test**:
```rust
#[test]
fn test_ollama_new_valid() {
    let result = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string()
    );
    assert!(result.is_ok());
}

#[test]
fn test_ollama_new_empty_model() {
    let result = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "".to_string()
    );
    assert!(result.is_err());
}
```

---

## Builder Methods

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

---

## Integration Points

### With Safety Validator
```rust
let backend = OllamaBackend::new(...)?;
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
let backend = OllamaBackend::new(
    Url::parse(&config.ollama_url.unwrap_or_default())?,
    config.ollama_model.unwrap_or("codellama:7b".to_string())
)?;
```

---

## Non-Functional Requirements

| Requirement | Target | Measurement |
|-------------|--------|-------------|
| NFR-001: Connection timeout | 5 seconds | HTTP client config |
| NFR-002: Generation timeout | 30 seconds | Request timeout |
| NFR-003: Retry attempts | 2 retries | Retry policy |
| NFR-004: Health check cache | 60 seconds | Timestamp comparison |
| NFR-005: Logging overhead | < 1ms | No blocking I/O |

---

## Test Coverage Requirements

- ✅ Happy path: Successful command generation
- ✅ Error: Ollama not running
- ✅ Error: Model not found (404)
- ✅ Error: Malformed JSON response
- ✅ Error: Timeout exceeded
- ✅ Retry: Transient failure → success
- ✅ Retry: All attempts exhausted
- ✅ Health check: Available backend
- ✅ Health check: Unavailable backend
- ✅ Constructor: Valid configuration
- ✅ Constructor: Invalid URL
- ✅ Builder: Temperature clamping
- ✅ Integration: With safety validator
- ✅ Integration: With config management

**Minimum Coverage**: 90% line coverage for `ollama.rs`

---

## Fallback Behavior (NEW - Feature 004 Update)

### FR-NEW-001: Embedded Model Fallback
**MUST** fallback to embedded model when Ollama backend fails or is unavailable.

**Behavior**:
1. Ollama backend attempts connection/inference
2. If failure (connection refused, timeout, error response):
   - Log warning: `warn!("Ollama backend failed, falling back to embedded model: {}", error)`
   - Transparently fallback to `EmbeddedModelBackend`
   - No error returned to user (seamless experience)
3. If fallback also fails: Return error (embedded model should never fail)

**Test**:
```rust
#[tokio::test]
async fn test_ollama_fallback_to_embedded() {
    // Configure Ollama backend with unreachable URL
    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:99999").unwrap(),  // Invalid port
        "codellama:7b".to_string()
    ).unwrap();

    let embedded = EmbeddedModelBackend::new(
        ModelVariant::detect(),
        test_model_path()
    ).unwrap();

    // Attempt generation (should fallback automatically)
    let request = CommandRequest::new("list files", ShellType::Bash);

    // In actual implementation, BackendSelector handles fallback
    let result = match ollama.generate_command(&request).await {
        Ok(cmd) => Ok(cmd),
        Err(_) => {
            warn!("Ollama failed, using embedded model");
            embedded.generate_command(&request).await
        }
    };

    assert!(result.is_ok(), "Must succeed via fallback");
    assert_eq!(result.unwrap().backend_used, "embedded");
}
```

### FR-NEW-002: Retry Before Fallback
**MUST** respect retry policy (up to 2 retries) before falling back to embedded model.

**Test**:
```rust
#[tokio::test]
async fn test_ollama_retry_before_fallback() {
    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string()
    ).unwrap()
    .with_retry_policy(RetryPolicy::default());  // 2 retries

    // Mock Ollama server that fails 3 times
    let mock_server = setup_failing_ollama_mock(3);

    let request = CommandRequest::new("list files", ShellType::Bash);
    let result = ollama.generate_command(&request).await;

    // Should attempt 3 times (initial + 2 retries), then fail
    assert!(result.is_err());
    assert_eq!(mock_server.call_count(), 3);
}
```

### FR-NEW-003: Optional Backend Status
**MUST** indicate Ollama is optional enhancement, not required.

**Update to `is_available()` behavior**:
- Returns `false` when Ollama not installed/running
- System continues with embedded model (no blocking error)
- Users can optionally configure Ollama via `caro init --backend ollama`

---

**Contract Status**: ✅ **UPDATED** - Includes embedded model fallback behavior
