// EmbeddedModelBackend implementation for offline command generation

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::backends::embedded::{CpuBackend, EmbeddedConfig, InferenceBackend, ModelVariant};
use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{BackendType, CommandRequest, GeneratedCommand};
use crate::safety::{SafetyConfig, SafetyValidator};
use crate::ModelLoader;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::backends::embedded::MlxBackend;

/// Regex pattern to extract command from malformed JSON with unescaped quotes
/// Handles cases like: {"cmd": "find . -type f -name "*.txt""}
/// The greedy .+ captures everything between the first quote after "cmd": and the last "}
static CMD_EXTRACT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{\s*"cmd"\s*:\s*"(.+)"\s*\}"#).expect("Invalid regex pattern"));

/// Primary command generator using embedded Qwen model with platform-specific inference
#[derive(Clone)]
pub struct EmbeddedModelBackend {
    model_variant: ModelVariant,
    model_path: PathBuf,
    backend: Arc<Mutex<Box<dyn InferenceBackend>>>,
    config: EmbeddedConfig,
    model_loader: ModelLoader,
    safety_validator: Arc<SafetyValidator>,
}

impl EmbeddedModelBackend {
    /// Create a new embedded model backend with auto-detected platform variant
    pub fn new() -> Result<Self, GeneratorError> {
        let variant = ModelVariant::detect();
        let model_loader = ModelLoader::new().map_err(|e| GeneratorError::ConfigError {
            message: format!("Failed to initialize model loader: {}", e),
        })?;
        let model_path =
            model_loader
                .get_embedded_model_path()
                .map_err(|e| GeneratorError::ConfigError {
                    message: format!("Failed to get model path: {}", e),
                })?;

        Self::with_variant_and_path(variant, model_path)
    }

    /// Create a new embedded model backend with specific variant and model path
    pub fn with_variant_and_path(
        variant: ModelVariant,
        model_path: PathBuf,
    ) -> Result<Self, GeneratorError> {
        // Create the appropriate backend based on variant
        let backend: Box<dyn InferenceBackend> = match variant {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            ModelVariant::MLX => Box::new(MlxBackend::new(model_path.clone()).map_err(|e| {
                GeneratorError::ConfigError {
                    message: format!("Failed to create MLX backend: {}", e),
                }
            })?),
            ModelVariant::CPU => Box::new(CpuBackend::new(model_path.clone()).map_err(|e| {
                GeneratorError::ConfigError {
                    message: format!("Failed to create CPU backend: {}", e),
                }
            })?),
        };

        let model_loader = ModelLoader::new().map_err(|e| GeneratorError::ConfigError {
            message: format!("Failed to initialize model loader: {}", e),
        })?;

        // Initialize safety validator with moderate config
        let safety_validator = Arc::new(
            SafetyValidator::new(SafetyConfig::moderate())
                .expect("Failed to initialize SafetyValidator with default config"),
        );

        Ok(Self {
            model_variant: variant,
            model_path,
            backend: Arc::new(Mutex::new(backend)),
            config: EmbeddedConfig::default(),
            model_loader,
            safety_validator,
        })
    }

    /// Update the embedded configuration
    pub fn with_config(mut self, config: EmbeddedConfig) -> Self {
        self.config = config;
        self
    }

    /// Update the safety configuration
    pub fn with_safety_config(mut self, safety_config: SafetyConfig) -> Self {
        self.safety_validator = Arc::new(
            SafetyValidator::new(safety_config).expect("Failed to initialize SafetyValidator"),
        );
        self
    }

    /// Get the model variant this backend uses
    pub fn variant(&self) -> ModelVariant {
        self.model_variant
    }

    /// Get the model path
    pub fn model_path(&self) -> &PathBuf {
        &self.model_path
    }

    /// Explicitly load the model (usually not needed as loading is lazy)
    pub async fn load_model(&mut self) -> Result<(), GeneratorError> {
        // Ensure model is downloaded
        self.model_loader
            .download_model_if_missing(self.model_variant)
            .await
            .map_err(|e| GeneratorError::BackendUnavailable {
                reason: format!("Failed to download model: {}", e),
            })?;

        // Load the model in the backend
        let mut backend = self.backend.lock().await;
        backend
            .load()
            .await
            .map_err(|e| GeneratorError::GenerationFailed {
                details: format!("Failed to load model: {}", e),
            })
    }

    /// Explicitly unload the model to free memory
    pub async fn unload_model(&mut self) -> Result<(), GeneratorError> {
        let mut backend = self.backend.lock().await;
        backend
            .unload()
            .await
            .map_err(|e| GeneratorError::Internal {
                message: format!("Failed to unload model: {}", e),
            })
    }

    /// Generate system prompt for shell command generation
    fn create_system_prompt(&self, request: &CommandRequest) -> String {
        let base_prompt = format!(
            r#"You are a shell command generator. Convert natural language to POSIX shell commands.

OUTPUT FORMAT: Respond with ONLY valid JSON:
{{"cmd": "your_command_here"}}

CRITICAL RULES:
1. ALWAYS use current directory "." as the starting path (NEVER use "/" root)
2. Use BSD-compatible flags (macOS). AVOID GNU-only flags like --max-depth
3. NEVER add flags that were not requested:
   - If request says "list files" -> use ONLY "ls" (NOT "ls -a", NOT "ls -l", NOT "ls -la")
   - If request says "show hidden" -> use ONLY "ls -a" (NOT "ls -la")
   - If request says "with details" -> use ONLY "ls -l" (NOT "ls -la")
   - ONLY combine flags (like -la or -lt) if BOTH things are explicitly mentioned
5. Include ALL relevant filters in find commands:
   - For file types: ALWAYS add -name "*.ext" pattern when extension mentioned
   - For files only: add -type f
   - For directories only: add -type d
6. Time filters with find -mtime:
   - -mtime -7 = modified within last 7 days
   - -mtime 7 = modified exactly 7 days ago
   - -mtime +7 = modified more than 7 days ago
   - -mtime 0 = modified today
   - -mtime 1 = modified yesterday (exactly 1 day ago)
7. For disk usage: use "du -sh */ | sort -rh | head -10" (BSD compatible)
8. Quote paths with spaces using double quotes
9. Use RELATIVE paths - never assume ~ (home directory)
   - "move to documents" = documents/ (NOT ~/Documents)
   - "copy to backup" = backup/ (NOT ~/backup)
10. Target shell: {}
11. NEVER generate destructive commands (rm -rf, mkfs, dd, etc.)

EXAMPLES (use exact flags shown):
- "list all files in the current directory" -> ls
- "show hidden files" -> ls -a
- "list files with detailed information" -> ls -l
- "list files sorted by modification time" -> ls -lt
- "show the current working directory" -> pwd
- "count files in current directory" -> ls -1 | wc -l
- "find all text files in current directory" -> find . -name "*.txt"
- "files modified today" -> find . -type f -mtime 0

IMPORTANT TOOL SELECTION RULES:
- If request mentions "docker" or "container" (but NOT "pod"): use docker command
- If request mentions "k8s", "kubernetes", "pod", "deployment", or "service" (k8s context): use kubectl command
- "containers" alone = docker ps
- "pods" alone = kubectl get pods

DOCKER COMMANDS (for containers):
- "list docker containers" -> docker ps
- "list all docker containers" -> docker ps -a
- "list running containers" -> docker ps
- "list docker images" -> docker images
- "show docker logs" -> docker logs <container>
- "stop all containers" -> docker stop $(docker ps -q)

KUBERNETES COMMANDS (for pods, k8s, kubernetes):
- "list k8s pods" -> kubectl get pods
- "list kubernetes pods" -> kubectl get pods
- "list pods" -> kubectl get pods
- "list all pods" -> kubectl get pods -A
- "list pods in namespace" -> kubectl get pods -n <namespace>
- "list k8s services" -> kubectl get services
- "list services" -> kubectl get services
- "list k8s deployments" -> kubectl get deployments
- "list deployments" -> kubectl get deployments
- "describe pod" -> kubectl describe pod <pod-name>
- "get pod logs" -> kubectl logs <pod-name>

Request: {}
"#,
            request.shell, request.input
        );

        // Append context if available
        if let Some(context) = &request.context {
            format!("{}\n\n{}", base_prompt, context)
        } else {
            base_prompt
        }
    }

    /// Parse JSON response from model inference
    fn parse_command_response(&self, response: &str) -> Result<String, GeneratorError> {
        // Try structured JSON parsing first
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                if !cmd.is_empty() {
                    return Ok(cmd.trim().to_string());
                }
            }
        }

        // Fallback: Try to extract JSON from response
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_part = &response[start..=end];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_part) {
                    if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                        if !cmd.is_empty() {
                            return Ok(cmd.trim().to_string());
                        }
                    }
                }
            }
        }

        // Final fallback: Look for command-like patterns
        for line in response.lines() {
            let line = line.trim();
            if line.starts_with("cmd") && line.contains(':') {
                if let Some(cmd_part) = line.split(':').nth(1) {
                    let cmd = cmd_part.trim().trim_matches('"').trim_matches('\'');
                    if !cmd.is_empty() && !cmd.contains('{') && !cmd.contains('}') {
                        return Ok(cmd.to_string());
                    }
                }
            }
        }

        // Regex fallback: Handle malformed JSON with unescaped quotes
        // e.g., {"cmd": "find . -type f -name "*.txt""}
        if let Some(caps) = CMD_EXTRACT_REGEX.captures(response) {
            if let Some(cmd_match) = caps.get(1) {
                let cmd = cmd_match.as_str().trim();
                if !cmd.is_empty() {
                    return Ok(cmd.to_string());
                }
            }
        }

        Err(GeneratorError::ParseError {
            content: response.to_string(),
        })
    }

    /// Infer and parse with retry on parse failure.
    /// The backend must already be loaded before calling this method.
    /// Only `GeneratorError::ParseError` triggers retry; all other errors fail fast.
    async fn infer_and_parse_with_retry(
        &self,
        backend: &dyn InferenceBackend,
        system_prompt: &str,
    ) -> Result<String, GeneratorError> {
        let max_retries = self.config.max_parse_retries.min(5);
        let mut last_raw_response = String::new();
        let mut command: Option<String> = None;

        for attempt in 0..=max_retries {
            let prompt = if attempt == 0 {
                system_prompt.to_string()
            } else {
                Self::build_parse_retry_prompt(system_prompt, &last_raw_response)
            };

            let raw_response = backend
                .infer(&prompt, &self.config)
                .await
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Inference failed: {}", e),
                })?;

            match self.parse_command_response(&raw_response) {
                Ok(cmd) => {
                    if attempt > 0 {
                        tracing::info!(
                            "Parse succeeded on attempt {}/{} after retry",
                            attempt + 1,
                            max_retries + 1
                        );
                    }
                    command = Some(cmd);
                    break;
                }
                Err(GeneratorError::ParseError { .. }) => {
                    // Log with truncated output to avoid flooding logs with large LLM responses
                    let log_output: String = raw_response.chars().take(200).collect();
                    if attempt < max_retries {
                        tracing::debug!(
                            "Parse attempt {}/{} raw output: {}{}",
                            attempt + 1,
                            max_retries + 1,
                            log_output,
                            if raw_response.len() > 200 { "..." } else { "" }
                        );
                        tracing::warn!(
                            "Parse attempt {}/{} failed, retrying with correction prompt",
                            attempt + 1,
                            max_retries + 1
                        );
                    }
                    last_raw_response = raw_response;
                }
                Err(other) => {
                    // Non-parse errors (generation failures, timeouts, etc.) fail fast
                    return Err(other);
                }
            }
        }

        match command {
            Some(cmd) => Ok(cmd),
            None => {
                // All attempts exhausted — return enriched error with retry count
                let last_output: String = last_raw_response.chars().take(200).collect();
                let truncation_note = if last_raw_response.len() > 200 { "..." } else { "" };
                Err(GeneratorError::ParseError {
                    content: format!(
                        "Response parsing failed after {} attempt{} (last response: {}{})",
                        max_retries + 1,
                        if max_retries + 1 == 1 { "" } else { "s" },
                        last_output,
                        truncation_note
                    ),
                })
            }
        }
    }

    /// Build a correction prompt that includes the original prompt plus the malformed output.
    /// Always uses original_prompt + latest malformed output only — never chains prior retry
    /// prompts to prevent exponential prompt growth.
    fn build_parse_retry_prompt(original_prompt: &str, malformed_output: &str) -> String {
        // Truncate malformed output cleanly at a UTF-8 character boundary
        const MAX_MALFORMED_LEN: usize = 500;
        let truncated = if malformed_output.len() > MAX_MALFORMED_LEN {
            let cut = malformed_output
                .char_indices()
                .take_while(|(i, _)| *i < MAX_MALFORMED_LEN)
                .last()
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(MAX_MALFORMED_LEN);
            format!("{}... [truncated]", &malformed_output[..cut])
        } else {
            malformed_output.to_string()
        };

        format!(
            r#"{original_prompt}

CORRECTION REQUIRED: Your previous response could not be parsed. The malformed output was:
{truncated}

You MUST respond with ONLY valid JSON — no prose, no markdown fences, no explanation.
Use this exact format and escape any double quotes inside the command with backslash:
{{"cmd": "your_command_here"}}

Example with escaped quotes: {{"cmd": "find . -type f -name \"*.conf\""}}"#
        )
    }
}

#[async_trait]
impl CommandGenerator for EmbeddedModelBackend {
    /// Generate a shell command from natural language input
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let start_time = std::time::Instant::now();

        // Ensure model is downloaded if needed
        self.model_loader
            .download_model_if_missing(self.model_variant)
            .await
            .map_err(|e| GeneratorError::BackendUnavailable {
                reason: format!("Failed to download model: {}", e),
            })?;

        // Create system prompt
        let system_prompt = self.create_system_prompt(request);

        // Acquire lock on backend and perform inference
        let mut backend = self.backend.lock().await;

        // Load model if not already loaded (lazy loading)
        backend
            .load()
            .await
            .map_err(|e| GeneratorError::GenerationFailed {
                details: format!("Failed to load model: {}", e),
            })?;

        // Run inference with retry on parse failure
        let command = self
            .infer_and_parse_with_retry(&**backend, &system_prompt)
            .await?;

        // SAFETY VALIDATION: Validate the GENERATED command
        let safety_result = self
            .safety_validator
            .validate_command(&command, request.shell)
            .await
            .map_err(|e| GeneratorError::ValidationFailed {
                reason: format!("Safety validation error: {}", e),
            })?;

        // If generated command is unsafe, return error
        if !safety_result.allowed {
            return Err(GeneratorError::Unsafe {
                reason: safety_result.explanation.clone(),
                risk_level: safety_result.risk_level,
                warnings: safety_result.warnings.clone(),
            });
        }

        let generation_time = start_time.elapsed().as_millis() as u64;

        Ok(GeneratedCommand {
            command,
            explanation: format!("Generated using {} backend", self.model_variant),
            safety_level: safety_result.risk_level, // Use actual risk level from validation
            estimated_impact: if safety_result.warnings.is_empty() {
                "Minimal system impact".to_string()
            } else {
                format!("Warnings: {}", safety_result.warnings.join(", "))
            },
            alternatives: vec![], // Embedded model generates single command
            backend_used: "embedded".to_string(),
            generation_time_ms: generation_time,
            confidence_score: 0.85, // Default confidence for embedded model
        })
    }

    /// Check if this backend is currently available for use
    async fn is_available(&self) -> bool {
        // Embedded model is always available (offline operation)
        true
    }

    /// Get information about this backend's capabilities and performance
    fn backend_info(&self) -> BackendInfo {
        let (typical_latency, memory_usage) = match self.model_variant {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            ModelVariant::MLX => (1800, 1200), // MLX: ~1.8s, ~1.2GB
            ModelVariant::CPU => (4000, 1500), // CPU: ~4s, ~1.5GB
        };

        BackendInfo {
            backend_type: BackendType::Embedded,
            model_name: "qwen2.5-coder-1.5b-instruct-q4_k_m".to_string(),
            supports_streaming: false,
            max_tokens: self.config.max_tokens as u32,
            typical_latency_ms: typical_latency,
            memory_usage_mb: memory_usage,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Perform any necessary cleanup when shutting down
    async fn shutdown(&self) -> Result<(), GeneratorError> {
        let mut backend = self.backend.lock().await;
        backend
            .unload()
            .await
            .map_err(|e| GeneratorError::Internal {
                message: format!("Failed to unload model: {}", e),
            })?;

        tracing::debug!("Embedded model backend shutdown complete");
        Ok(())
    }
}

impl Default for EmbeddedModelBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create default embedded model backend")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ShellType;

    #[test]
    fn test_embedded_backend_creation() {
        let backend = EmbeddedModelBackend::new();
        assert!(
            backend.is_ok(),
            "Should create embedded backend successfully"
        );

        if let Ok(backend) = backend {
            // Verify variant matches platform
            let expected_variant = ModelVariant::detect();
            assert_eq!(backend.variant(), expected_variant);
        }
    }

    #[test]
    fn test_system_prompt_generation() {
        let backend = EmbeddedModelBackend::new().unwrap();
        let request = CommandRequest::new("list files", ShellType::Bash);

        let prompt = backend.create_system_prompt(&request);

        assert!(prompt.contains("list files"));
        assert!(prompt.contains("bash"));
        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("cmd"));
    }

    #[test]
    fn test_json_response_parsing() {
        let backend = EmbeddedModelBackend::new().unwrap();

        // Test valid JSON
        let response = r#"{"cmd": "ls -la"}"#;
        let result = backend.parse_command_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ls -la");

        // Test JSON with extra content
        let response = r#"Here's the command: {"cmd": "find . -name '*.txt'"} - that should work!"#;
        let result = backend.parse_command_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "find . -name '*.txt'");

        // Test malformed JSON with unescaped nested quotes (regression test)
        // LLMs sometimes output: {"cmd": "find . -type f -name "*.llm""}
        // instead of properly escaped: {"cmd": "find . -type f -name \"*.llm\""}
        let response = r#"{"cmd": "find . -type f -name "*.llm""}"#;
        let result = backend.parse_command_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"find . -type f -name "*.llm""#);

        // Test malformed response
        let response = "This is not JSON at all";
        let result = backend.parse_command_response(response);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_available_always_true() {
        let backend = EmbeddedModelBackend::new().unwrap();
        assert!(
            backend.is_available().await,
            "Embedded backend must always be available"
        );
    }

    #[test]
    fn test_backend_info() {
        let backend = EmbeddedModelBackend::new().unwrap();
        let info = backend.backend_info();

        assert_eq!(info.backend_type, BackendType::Embedded);
        assert_eq!(info.model_name, "qwen2.5-coder-1.5b-instruct-q4_k_m");
        assert!(!info.supports_streaming);
        assert!(info.max_tokens > 0);
        assert!(info.typical_latency_ms > 0);
        assert!(info.memory_usage_mb > 0);
    }

    // --- Parse retry prompt tests ---

    #[test]
    fn test_parse_retry_prompt_contains_malformed_output() {
        let original = "original system prompt";
        let malformed = r#"{"cmd": "find . -name "*.conf""}"#;
        let prompt = EmbeddedModelBackend::build_parse_retry_prompt(original, malformed);
        assert!(prompt.contains(malformed), "Prompt must include the malformed output");
        assert!(prompt.starts_with(original), "Prompt must start with original prompt");
    }

    #[test]
    fn test_parse_retry_prompt_contains_no_prose_instruction() {
        let prompt = EmbeddedModelBackend::build_parse_retry_prompt("p", "bad output");
        assert!(
            prompt.contains("no prose") || prompt.contains("no markdown"),
            "Prompt must instruct model to avoid prose and markdown fences"
        );
        assert!(prompt.contains("ONLY valid JSON"), "Prompt must say 'ONLY valid JSON'");
    }

    #[test]
    fn test_parse_retry_prompt_contains_escaping_example() {
        let prompt = EmbeddedModelBackend::build_parse_retry_prompt("p", "bad");
        assert!(
            prompt.contains(r#"\".conf\""#) || prompt.contains(r#"\"*.conf\""#),
            "Prompt must include an escaping example with backslash-escaped quotes"
        );
    }

    #[test]
    fn test_parse_retry_prompt_uses_original_prompt_not_chained() {
        // Calling build_parse_retry_prompt twice with a previous retry prompt as original
        // should NOT result in double-nested CORRECTION sections
        let original = "original prompt";
        let _first_retry = EmbeddedModelBackend::build_parse_retry_prompt(original, "bad1");
        let second_retry = EmbeddedModelBackend::build_parse_retry_prompt(original, "bad2");

        // Second retry is built from original, not from first_retry — no chaining
        assert!(!second_retry.contains("bad1"), "Second retry must not include first malformed output");
        assert!(second_retry.contains("bad2"), "Second retry must include latest malformed output");
        assert!(second_retry.starts_with(original));
        // Should contain exactly one CORRECTION block
        assert_eq!(
            second_retry.matches("CORRECTION REQUIRED").count(),
            1,
            "Must have exactly one CORRECTION block"
        );
    }

    #[test]
    fn test_parse_retry_prompt_truncates_long_output() {
        let long_output = "x".repeat(600);
        let prompt = EmbeddedModelBackend::build_parse_retry_prompt("p", &long_output);
        assert!(
            prompt.contains("[truncated]"),
            "Long malformed output must be truncated with [truncated] marker"
        );
        // Verify the truncated portion appears in the prompt (not full 600 chars repeated)
        assert!(
            !prompt.contains(&"x".repeat(501)),
            "Prompt must not contain more than 500 chars of the malformed output"
        );
    }

    // --- Behavioral retry tests using a mock InferenceBackend ---

    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc as StdArc;

    struct MockInferenceBackend {
        /// Queue of Ok responses to return; last one repeats if exhausted
        ok_responses: StdArc<std::sync::Mutex<std::collections::VecDeque<String>>>,
        /// If set, return this error instead of Ok (overrides ok_responses)
        error_response: Option<String>,
        call_count: StdArc<AtomicU32>,
        /// Records prompts passed on each call
        prompts_received: StdArc<tokio::sync::Mutex<Vec<String>>>,
    }

    impl MockInferenceBackend {
        fn ok_responses(responses: Vec<&str>) -> Self {
            Self {
                ok_responses: StdArc::new(std::sync::Mutex::new(
                    responses.into_iter().map(str::to_string).collect(),
                )),
                error_response: None,
                call_count: StdArc::new(AtomicU32::new(0)),
                prompts_received: StdArc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }

        fn always_error(message: &str) -> Self {
            Self {
                ok_responses: StdArc::new(std::sync::Mutex::new(std::collections::VecDeque::new())),
                error_response: Some(message.to_string()),
                call_count: StdArc::new(AtomicU32::new(0)),
                prompts_received: StdArc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl InferenceBackend for MockInferenceBackend {
        async fn infer(&self, prompt: &str, _config: &EmbeddedConfig) -> Result<String, GeneratorError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            self.prompts_received.lock().await.push(prompt.to_string());

            if let Some(ref msg) = self.error_response {
                return Err(GeneratorError::GenerationFailed { details: msg.clone() });
            }

            let mut queue = self.ok_responses.lock().unwrap();
            if queue.len() > 1 {
                Ok(queue.pop_front().unwrap())
            } else {
                // Last element repeats
                Ok(queue.front().cloned().unwrap_or_default())
            }
        }

        fn variant(&self) -> ModelVariant {
            ModelVariant::CPU
        }

        async fn load(&mut self) -> Result<(), GeneratorError> {
            Ok(())
        }

        async fn unload(&mut self) -> Result<(), GeneratorError> {
            Ok(())
        }
    }

    /// Helper: create a minimal EmbeddedModelBackend with custom config for unit tests.
    /// Tests call `infer_and_parse_with_retry` directly to avoid model download.
    fn make_test_backend(max_parse_retries: u32) -> EmbeddedModelBackend {
        let mut backend = EmbeddedModelBackend::new().unwrap();
        backend.config = EmbeddedConfig::default().with_max_parse_retries(max_parse_retries);
        backend
    }

    #[tokio::test]
    async fn test_retry_succeeds_on_second_attempt() {
        // First response: completely non-JSON (fails all 4 parsing stages)
        // Second response: valid JSON
        let mock = MockInferenceBackend::ok_responses(vec![
            "Sure! The command would be: find . -type f -name *.conf", // no JSON at all
            r#"{"cmd": "find . -type f -name \"*.conf\""}"#,           // valid
        ]);
        let call_count = mock.call_count.clone();
        let prompts = mock.prompts_received.clone();

        let backend = make_test_backend(2);
        let result = backend
            .infer_and_parse_with_retry(&mock, "find nginx config files")
            .await;

        assert!(result.is_ok(), "Should succeed after retry: {:?}", result);
        assert_eq!(call_count.load(Ordering::SeqCst), 2, "infer must be called exactly twice");

        // Verify second call used a correction prompt containing exactly one CORRECTION block
        let prompts = prompts.lock().await;
        assert!(
            prompts[1].contains("CORRECTION REQUIRED"),
            "Second attempt must use a correction prompt"
        );
        assert_eq!(
            prompts[1].matches("CORRECTION REQUIRED").count(),
            1,
            "Second attempt must have exactly one CORRECTION block"
        );
    }

    #[tokio::test]
    async fn test_retry_exhausts_all_attempts() {
        // Always return malformed output
        let mock = MockInferenceBackend::ok_responses(vec!["not json at all"]);
        let call_count = mock.call_count.clone();

        let backend = make_test_backend(2);
        let result = backend
            .infer_and_parse_with_retry(&mock, "find files")
            .await;

        assert!(result.is_err(), "Should fail after exhausting retries");
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            3, // 1 initial + 2 retries
            "infer must be called max_retries+1 times total"
        );
        if let Err(GeneratorError::ParseError { content }) = result {
            assert!(
                content.contains("3 attempt"),
                "Error must indicate 3 attempts were made, got: {content}"
            );
        } else {
            panic!("Expected ParseError");
        }
    }

    #[tokio::test]
    async fn test_no_retry_on_first_success() {
        let mock = MockInferenceBackend::ok_responses(vec![r#"{"cmd": "ls -la"}"#]);
        let call_count = mock.call_count.clone();

        let backend = make_test_backend(2);
        let result = backend
            .infer_and_parse_with_retry(&mock, "list files")
            .await;

        assert!(result.is_ok(), "Should succeed without retry: {:?}", result);
        assert_eq!(call_count.load(Ordering::SeqCst), 1, "infer must be called exactly once");
    }

    #[tokio::test]
    async fn test_no_retry_on_non_parse_error() {
        let mock = MockInferenceBackend::always_error("model crashed");
        let call_count = mock.call_count.clone();

        let backend = make_test_backend(2);
        let result = backend
            .infer_and_parse_with_retry(&mock, "list files")
            .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1, "Non-parse errors must not trigger retry");
        assert!(
            matches!(result, Err(GeneratorError::GenerationFailed { .. })),
            "Error type must be preserved"
        );
    }

    #[tokio::test]
    async fn test_zero_retries_single_attempt_only() {
        // With max_parse_retries=0, only one call is made even on parse failure
        let mock = MockInferenceBackend::ok_responses(vec!["not json"]);
        let call_count = mock.call_count.clone();

        let backend = make_test_backend(0);
        let result = backend
            .infer_and_parse_with_retry(&mock, "list files")
            .await;

        assert!(result.is_err());
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            1,
            "With max_parse_retries=0, exactly one inference call must be made"
        );
    }
}
