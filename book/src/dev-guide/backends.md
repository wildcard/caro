# Backend Development

This guide explains how to implement and integrate new LLM backends into cmdai.

> **💡 Quick Reference:** For the trait definition and architecture overview, see [Architecture: Backend Trait System](./architecture.md#backend-trait-system).

## Backend Architecture

All backends implement the `CommandGenerator` trait, providing a unified interface for command generation.

**Available Backends:**

| Backend | Type | Platform | Status |
|---------|------|----------|--------|
| **Embedded** | Local | All | ✅ Implemented |
| **MLX** | Local | Apple Silicon | ✅ Implemented |
| **Ollama** | Local API | All | ✅ Implemented |
| **vLLM** | Remote API | All | ✅ Implemented |
| **Custom** | Flexible | All | 📖 This guide |

## The CommandGenerator Trait

```rust
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    /// Generate a shell command from a natural language prompt
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError>;

    /// Check if the backend is available and healthy
    async fn is_available(&self) -> bool;

    /// Get backend metadata and capabilities
    fn backend_info(&self) -> BackendInfo;
}
```

### Key Requirements

1. **Thread Safety**: `Send + Sync` for concurrent access
2. **Async Support**: All operations are async
3. **Error Handling**: Return `Result` types
4. **Availability Check**: Health checks before use
5. **Metadata**: Provide backend information

## Data Types

### CommandRequest

Input to the command generator:

```rust
pub struct CommandRequest {
    /// Natural language prompt
    pub prompt: String,

    /// Target shell (bash, zsh, fish, etc.)
    pub shell: Shell,

    /// Additional context (optional)
    pub context: Option<Context>,

    /// Generation parameters
    pub parameters: GenerationParameters,
}
```

### GeneratedCommand

Output from the command generator:

```rust
pub struct GeneratedCommand {
    /// The generated shell command
    pub command: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Alternative commands (optional)
    pub alternatives: Vec<String>,

    /// Explanation (optional)
    pub explanation: Option<String>,
}
```

### BackendInfo

Metadata about the backend:

```rust
pub struct BackendInfo {
    /// Backend name
    pub name: String,

    /// Backend version
    pub version: String,

    /// Model name/identifier
    pub model: String,

    /// Capabilities
    pub capabilities: BackendCapabilities,
}
```

## Implementing a Backend

### Step 1: Create Backend Structure

```rust
pub struct CustomBackend {
    config: CustomConfig,
    client: HttpClient,  // or other resources
}

impl CustomBackend {
    pub fn new(config: CustomConfig) -> Result<Self> {
        Ok(Self {
            config,
            client: HttpClient::new()?,
        })
    }
}
```

### Step 2: Implement CommandGenerator

```rust
#[async_trait]
impl CommandGenerator for CustomBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand> {
        // 1. Build the prompt
        let prompt = build_system_prompt(request);

        // 2. Call the LLM
        let response = self.client
            .generate(&prompt)
            .await?;

        // 3. Parse the response
        let command = parse_response(&response)?;

        // 4. Validate and return
        Ok(GeneratedCommand {
            command,
            confidence: calculate_confidence(&response),
            alternatives: vec![],
            explanation: Some(response.explanation),
        })
    }

    async fn is_available(&self) -> bool {
        // Check if the backend is accessible
        self.client.health_check().await.is_ok()
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            name: "custom".to_string(),
            version: "1.0.0".to_string(),
            model: self.config.model_name.clone(),
            capabilities: BackendCapabilities::default(),
        }
    }
}
```

### Step 3: Add Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CustomConfig {
    pub base_url: String,
    pub model_name: String,
    pub api_key: Option<String>,
    pub timeout: u64,
}
```

### Step 4: Register the Backend

Add to `backends/mod.rs`:

```rust
pub mod custom;

pub fn create_backend(
    backend_type: BackendType,
    config: &Config,
) -> Result<Box<dyn CommandGenerator>> {
    match backend_type {
        BackendType::Custom => {
            let backend = custom::CustomBackend::new(config.custom.clone())?;
            Ok(Box::new(backend))
        }
        // ... other backends
    }
}
```

## Backend Examples

### HTTP-Based Backend (Ollama/vLLM)

```rust
pub struct HttpBackend {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

#[async_trait]
impl CommandGenerator for HttpBackend {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand>
    {
        // Build request payload
        let payload = serde_json::json!({
            "model": self.model,
            "prompt": build_prompt(request),
            "stream": false,
        });

        // Send HTTP request
        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&payload)
            .send()
            .await?;

        // Parse response
        let data: Response = response.json().await?;
        let command = extract_command(&data.response)?;

        Ok(GeneratedCommand {
            command,
            confidence: 0.9,
            alternatives: vec![],
            explanation: None,
        })
    }

    async fn is_available(&self) -> bool {
        self.client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await
            .is_ok()
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            name: "http".to_string(),
            version: "1.0.0".to_string(),
            model: self.model.clone(),
            capabilities: BackendCapabilities::default(),
        }
    }
}
```

### Native/FFI Backend (MLX)

```rust
pub struct MLXBackend {
    model: Arc<Model>,
    tokenizer: Arc<Tokenizer>,
}

#[async_trait]
impl CommandGenerator for MLXBackend {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand>
    {
        // Tokenize input
        let tokens = self.tokenizer.encode(&request.prompt)?;

        // Run inference
        let output = self.model.generate(tokens).await?;

        // Decode output
        let command = self.tokenizer.decode(&output)?;

        Ok(GeneratedCommand {
            command: extract_command(&command)?,
            confidence: 0.95,
            alternatives: vec![],
            explanation: None,
        })
    }

    async fn is_available(&self) -> bool {
        // Check if MLX is available (Apple Silicon only)
        cfg!(target_os = "macos") && cfg!(target_arch = "aarch64")
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            name: "mlx".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            model: "Qwen2.5-Coder-1.5B".to_string(),
            capabilities: BackendCapabilities {
                offline: true,
                streaming: false,
                embeddings: false,
            },
        }
    }
}
```

## Prompt Engineering

### System Prompt Template

```rust
fn build_system_prompt(request: &CommandRequest) -> String {
    format!(
        r#"You are a command-line expert. Generate a single, safe {shell} command.

Rules:
1. Output ONLY a valid {shell} command
2. Use POSIX-compliant utilities when possible
3. Quote file paths with spaces properly
4. Avoid destructive operations
5. Respond ONLY with JSON: {{"cmd": "command here"}}

User request: {prompt}

Remember: Generate a safe, working command in JSON format."#,
        shell = request.shell,
        prompt = request.prompt
    )
}
```

### Response Parsing

```rust
fn parse_response(response: &str) -> Result<String> {
    // Try JSON parsing first
    if let Ok(json) = serde_json::from_str::<CommandResponse>(response) {
        return Ok(json.cmd);
    }

    // Fallback: Extract code blocks
    if let Some(cmd) = extract_code_block(response) {
        return Ok(cmd);
    }

    // Fallback: Use entire response
    Ok(response.trim().to_string())
}

#[derive(Deserialize)]
struct CommandResponse {
    cmd: String,
}

fn extract_code_block(text: &str) -> Option<String> {
    // Extract from ```bash ... ``` blocks
    let re = regex::Regex::new(r"```(?:bash|sh)?\s*\n(.*?)\n```").unwrap();
    re.captures(text)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}
```

## Error Handling

### Backend-Specific Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Backend unavailable: {0}")]
    Unavailable(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
```

### Graceful Degradation

```rust
impl CustomBackend {
    async fn generate_with_retry(&self, request: &CommandRequest)
        -> Result<GeneratedCommand>
    {
        let mut retries = 3;
        let mut delay = Duration::from_secs(1);

        loop {
            match self.generate_command(request).await {
                Ok(cmd) => return Ok(cmd),
                Err(e) if retries > 0 => {
                    eprintln!("Backend error, retrying: {}", e);
                    tokio::time::sleep(delay).await;
                    retries -= 1;
                    delay *= 2;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

## Testing Backends

### Contract Tests

All backends must pass these tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_generation() {
        let backend = CustomBackend::new(test_config()).unwrap();

        let request = CommandRequest {
            prompt: "list files".to_string(),
            shell: Shell::Bash,
            context: None,
            parameters: Default::default(),
        };

        let result = backend.generate_command(&request).await;
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert!(!cmd.command.is_empty());
        assert!(cmd.confidence >= 0.0 && cmd.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_availability() {
        let backend = CustomBackend::new(test_config()).unwrap();
        let available = backend.is_available().await;
        // Should either be available or gracefully report unavailability
        assert!(available || !available);
    }

    #[tokio::test]
    async fn test_backend_info() {
        let backend = CustomBackend::new(test_config()).unwrap();
        let info = backend.backend_info();

        assert!(!info.name.is_empty());
        assert!(!info.model.is_empty());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_real_backend() {
    let backend = CustomBackend::new(production_config()).unwrap();

    // Ensure backend is available
    assert!(backend.is_available().await);

    // Test real generation
    let request = CommandRequest {
        prompt: "show current directory".to_string(),
        shell: Shell::Bash,
        context: None,
        parameters: Default::default(),
    };

    let cmd = backend.generate_command(&request).await.unwrap();
    assert!(cmd.command.contains("pwd") || cmd.command.contains("ls"));
}
```

## Best Practices

### 1. Lazy Initialization

Load resources only when needed:

```rust
pub struct LazyBackend {
    config: Config,
    client: OnceCell<HttpClient>,
}

impl LazyBackend {
    fn get_client(&self) -> Result<&HttpClient> {
        self.client.get_or_try_init(|| {
            HttpClient::new(&self.config)
        })
    }
}
```

### 2. Connection Pooling

Reuse HTTP connections:

```rust
lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
}
```

### 3. Timeout Handling

Set reasonable timeouts:

```rust
async fn generate_with_timeout(&self, request: &CommandRequest)
    -> Result<GeneratedCommand>
{
    tokio::time::timeout(
        Duration::from_secs(30),
        self.generate_command(request)
    )
    .await
    .map_err(|_| BackendError::Timeout)?
}
```

### 4. Caching

Cache model weights or responses:

```rust
pub struct CachedBackend {
    inner: Box<dyn CommandGenerator>,
    cache: Arc<Mutex<LruCache<String, GeneratedCommand>>>,
}
```

## Next Steps

**Developer Guides:**
- [Architecture](./architecture.md) - Overall system design and backend selection
- [Testing Strategy](./testing.md) - Contract tests for backends
- [TDD Workflow](./tdd-workflow.md) - Test-driven development process
- [Contributing](./contributing.md) - Submit your backend to the project

**Technical Deep Dives:**
- [MLX Integration](../technical/mlx-integration.md) - Example of FFI backend implementation
- [Safety Validation](../technical/safety-validation.md) - How backends integrate with safety
- [Rust Learnings](../technical/rust-learnings.md) - Rust patterns used in backends

**User Guides:**
- [Configuration](../user-guide/configuration.md) - Configure backend selection and fallback

---

## See Also

**Backend Examples:**
- [HTTP-Based Backend](#http-based-backend-ollamavllm) - Remote API integration
- [Native/FFI Backend](#nativeffi-backend-mlx) - Low-level integration
- [Error Handling](#error-handling) - Backend error management

**Related Documentation:**
- [Architecture: Backend Selection Algorithm](./architecture.md#backend-selection-algorithm) - How backends are chosen
- [Configuration: Backend Config](../user-guide/configuration.md#backend-configuration) - User-facing configuration
- [Testing: Contract Tests](./testing.md) - Backend test requirements

**External Resources:**
- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md) - Ollama backend reference
- [vLLM Documentation](https://vllm.readthedocs.io/) - vLLM backend reference
- [MLX Documentation](https://ml-explore.github.io/mlx/) - MLX backend reference

**Community:**
- [Contributing Guide](./contributing.md) - How to submit a backend implementation
- [Active Development](../community/active-development.md) - Ongoing backend work
