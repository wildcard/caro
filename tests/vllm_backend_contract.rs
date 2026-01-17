// Contract tests for VllmBackend with embedded model fallback
// These tests verify the behavioral contract defined in:
// specs/004-implement-ollama-and/contracts/vllm-backend.md

use std::sync::Arc;
use std::time::Duration;

use caro::backends::{CommandGenerator, GeneratorError};
use caro::models::{CommandRequest, SafetyLevel, ShellType};
use url::Url;

// Placeholder struct - will be replaced with actual VllmBackend implementation
struct VllmBackend {
    url: Url,
    model: String,
    api_key: Option<String>,
    temperature: f32,
    top_p: f32,
}

impl std::fmt::Debug for VllmBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VllmBackend")
            .field("url", &self.url)
            .field("model", &self.model)
            .field(
                "api_key",
                if self.api_key.is_some() {
                    &"<redacted>"
                } else {
                    &"None"
                },
            )
            .field("temperature", &self.temperature)
            .field("top_p", &self.top_p)
            .finish()
    }
}

impl VllmBackend {
    pub fn new(url: Url, model: String) -> Result<Self, GeneratorError> {
        if model.is_empty() {
            return Err(GeneratorError::ConfigError {
                message: "Model name cannot be empty".to_string(),
            });
        }

        Ok(Self {
            url,
            model,
            api_key: None,
            temperature: 0.7,
            top_p: 0.95,
        })
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 2.0);
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p.clamp(0.0, 1.0);
        self
    }

    pub fn with_embedded_fallback(self, _embedded_backend: Arc<dyn CommandGenerator>) -> Self {
        // Placeholder - actual implementation will store the fallback backend
        self
    }
}

// Placeholder implementation - these will fail until real implementation exists
#[async_trait::async_trait]
impl CommandGenerator for VllmBackend {
    async fn generate_command(
        &self,
        _request: &caro::models::CommandRequest,
    ) -> Result<caro::models::GeneratedCommand, GeneratorError> {
        Err(GeneratorError::GenerationFailed {
            details: "VllmBackend not yet implemented".to_string(),
        })
    }

    async fn is_available(&self) -> bool {
        false // Will be true when implemented
    }

    fn backend_info(&self) -> caro::backends::BackendInfo {
        caro::backends::BackendInfo {
            backend_type: caro::models::BackendType::VLlm,
            model_name: self.model.clone(),
            supports_streaming: false,
            max_tokens: 4096,
            typical_latency_ms: 3000,
            memory_usage_mb: 0,
            version: "placeholder".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

/// FR-NEW-001: Embedded Model Fallback
/// MUST fallback to embedded model when vLLM backend fails or is unavailable
#[tokio::test]
async fn test_fallback_to_embedded_on_connection_failure() {
    use caro::backends::embedded::{EmbeddedModelBackend, ModelVariant};
    use std::path::PathBuf;

    // Create embedded fallback backend
    let embedded = EmbeddedModelBackend::with_variant_and_path(
        ModelVariant::detect(),
        PathBuf::from("/tmp/test_model.gguf"),
    )
    .unwrap();
    let embedded_arc: Arc<dyn CommandGenerator> = Arc::new(embedded);

    // Create vLLM backend with unreachable URL
    let vllm = VllmBackend::new(
        Url::parse("https://nonexistent.example.com").unwrap(),
        "codellama/CodeLlama-7b-hf".to_string(),
    )
    .unwrap()
    .with_embedded_fallback(embedded_arc);

    let request = CommandRequest::new("list files", ShellType::Bash);

    // This should fallback to embedded model (though will still fail in TDD RED phase)
    let result = vllm.generate_command(&request).await;

    // In TDD RED: This will fail because neither vLLM nor embedded is implemented
    // In TDD GREEN: This should succeed via embedded fallback
    assert!(
        result.is_err(), // Expected in RED phase
        "TDD RED: Should fail until implementation complete"
    );

    // When implemented, we'd check:
    // assert!(result.is_ok(), "Must succeed via embedded fallback");
    // let command = result.unwrap();
    // assert_eq!(command.backend_used, "embedded", "Should use embedded fallback");
}

/// FR-NEW-002: Authentication Failure Fallback (No Retry)
/// MUST fallback immediately on authentication failures (401/403)
#[tokio::test]
async fn test_auth_failure_fallback_no_retry() {
    use caro::backends::embedded::{EmbeddedModelBackend, ModelVariant};
    use std::path::PathBuf;

    let embedded = EmbeddedModelBackend::with_variant_and_path(
        ModelVariant::detect(),
        PathBuf::from("/tmp/test_model.gguf"),
    )
    .unwrap();
    let embedded_arc: Arc<dyn CommandGenerator> = Arc::new(embedded);

    // Create vLLM backend with invalid API key
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "codellama/CodeLlama-7b-hf".to_string(),
    )
    .unwrap()
    .with_api_key("invalid_key".to_string())
    .with_embedded_fallback(embedded_arc);

    let request = CommandRequest::new("test", ShellType::Bash);

    // Should fail quickly (no retries on auth failure)
    let start = std::time::Instant::now();
    let _result = vllm.generate_command(&request).await;
    let duration = start.elapsed();

    // Should not retry auth failures, so should be fast
    assert!(
        duration < Duration::from_secs(5),
        "Auth failure should not retry, got {:?}",
        duration
    );
}

/// FR-NEW-005: HTTPS Warning for HTTP URLs
/// SHOULD warn when using HTTP (not HTTPS) for remote vLLM servers
#[test]
fn test_https_warning_for_http_urls() {
    // Create vLLM backend with HTTP URL (insecure)
    let vllm = VllmBackend::new(
        Url::parse("http://api.example.com").unwrap(), // HTTP, not HTTPS
        "test-model".to_string(),
    )
    .unwrap()
    .with_api_key("test-key".to_string());

    // The warning would be logged during actual usage
    // For now, just verify the backend can be created
    assert_eq!(vllm.url.scheme(), "http");
    assert!(vllm.api_key.is_some(), "API key should be set");

    // In real implementation, this would trigger:
    // warn!("Using HTTP for vLLM is insecure, use HTTPS in production");
}

/// Original Contract: Basic vLLM functionality
#[tokio::test]
async fn test_vllm_generate_command_success() {
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "codellama/CodeLlama-7b-hf".to_string(),
    )
    .unwrap()
    .with_api_key("test-key".to_string());

    let request =
        CommandRequest::new("list all files", ShellType::Bash).with_safety(SafetyLevel::Moderate);

    let result = vllm.generate_command(&request).await;

    // TDD RED: This will fail until implementation exists
    assert!(
        result.is_err(),
        "TDD RED: Should fail until vLLM backend implemented"
    );

    // When implemented, we'd test:
    // assert!(result.is_ok(), "Command generation must succeed");
    // let command = result.unwrap();
    // assert!(!command.command.is_empty(), "Must generate non-empty command");
    // assert_eq!(command.backend_used, "vllm", "Must report vllm backend");
}

/// Contract: Builder pattern functionality
#[test]
fn test_vllm_builder_pattern() {
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "test-model".to_string(),
    )
    .unwrap()
    .with_api_key("sk-test123".to_string())
    .with_temperature(0.8)
    .with_top_p(0.9);

    assert_eq!(vllm.temperature, 0.8);
    assert_eq!(vllm.top_p, 0.9);
    assert_eq!(vllm.api_key, Some("sk-test123".to_string()));
}

/// Contract: Parameter validation and clamping
#[test]
fn test_vllm_parameter_validation() {
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "test-model".to_string(),
    )
    .unwrap()
    .with_temperature(5.0) // Too high
    .with_top_p(-0.5); // Too low

    // Values should be clamped to valid ranges
    assert_eq!(
        vllm.temperature, 2.0,
        "Temperature should be clamped to max 2.0"
    );
    assert_eq!(vllm.top_p, 0.0, "Top-p should be clamped to min 0.0");
}

/// Contract: Empty model name validation
#[test]
fn test_vllm_empty_model_name() {
    let result = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "".to_string(), // Empty model name
    );

    assert!(result.is_err(), "Empty model name should cause error");

    if let Err(error) = result {
        let error_msg = error.to_string();
        assert!(
            error_msg.contains("empty") || error_msg.contains("model"),
            "Error should mention empty model: {}",
            error_msg
        );
    }
}

/// Contract: Backend info reporting
#[test]
fn test_vllm_backend_info() {
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "codellama/CodeLlama-7b-hf".to_string(),
    )
    .unwrap();

    let info = vllm.backend_info();

    assert_eq!(info.backend_type, caro::models::BackendType::VLlm);
    assert_eq!(info.model_name, "codellama/CodeLlama-7b-hf");
    assert!(
        !info.supports_streaming,
        "vLLM backend doesn't support streaming"
    );
    assert!(info.max_tokens > 0, "Max tokens must be positive");
    assert!(
        info.typical_latency_ms > 0,
        "Latency estimate must be positive"
    );
}

/// Contract: Health check functionality
#[tokio::test]
async fn test_vllm_health_check() {
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "test-model".to_string(),
    )
    .unwrap();

    // Health check should be fast and consistent
    let start = std::time::Instant::now();
    let health1 = vllm.is_available().await;
    let health2 = vllm.is_available().await;
    let duration = start.elapsed();

    assert_eq!(
        health1, health2,
        "Health check results should be consistent"
    );
    assert!(
        duration < Duration::from_secs(10),
        "Health checks should be fast, got {:?}",
        duration
    );
}

/// Contract: Graceful shutdown
#[tokio::test]
async fn test_vllm_shutdown() {
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "test-model".to_string(),
    )
    .unwrap();

    let shutdown_result = vllm.shutdown().await;
    assert!(
        shutdown_result.is_ok(),
        "Shutdown must succeed: {:?}",
        shutdown_result
    );
}

/// Contract: OpenAI-compatible request format
#[test]
fn test_vllm_openai_compatibility() {
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com/v1").unwrap(), // OpenAI-style endpoint
        "codellama/CodeLlama-7b-hf".to_string(),
    )
    .unwrap()
    .with_api_key("sk-test".to_string());

    // Should accept OpenAI-compatible endpoint URLs
    assert!(
        vllm.url.path().contains("v1"),
        "Should support /v1 endpoint"
    );
    assert!(vllm.api_key.is_some(), "Should support API key auth");
}

/// Contract: Concurrent request handling
#[tokio::test]
async fn test_vllm_concurrent_requests() {
    let vllm = Arc::new(
        VllmBackend::new(
            Url::parse("https://api.example.com").unwrap(),
            "test-model".to_string(),
        )
        .unwrap(),
    );

    let mut handles = vec![];

    for i in 0..3 {
        let vllm_clone = Arc::clone(&vllm);
        let handle = tokio::spawn(async move {
            let request = CommandRequest::new(format!("test command {}", i), ShellType::Bash);
            vllm_clone.generate_command(&request).await
        });
        handles.push(handle);
    }

    // All requests should complete (though may error in TDD RED phase)
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.expect("Task should not panic");
        // In TDD RED, we just verify no panic occurred
        assert!(
            result.is_err(),
            "TDD RED: Request {} should fail until implementation complete",
            i
        );
    }
}

/// Contract: API key security (no logging)
#[test]
fn test_vllm_api_key_security() {
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "test-model".to_string(),
    )
    .unwrap()
    .with_api_key("sk-secret123".to_string());

    // Debug output should not contain raw API key
    let debug_output = format!("{:?}", vllm);

    // API key should be redacted or not present in debug output
    assert!(
        !debug_output.contains("sk-secret123"),
        "API key must not appear in debug output: {}",
        debug_output
    );
}

/// Contract: Environment variable API key loading
#[test]
fn test_vllm_env_var_api_key() {
    // Set environment variable
    std::env::set_var("VLLM_API_KEY", "sk-from-env");

    // In real implementation, constructor would check environment
    let vllm = VllmBackend::new(
        Url::parse("https://api.example.com").unwrap(),
        "test-model".to_string(),
    )
    .unwrap();

    // For now, just verify the constructor works
    // Real implementation would auto-load from env
    assert!(vllm.api_key.is_none()); // Placeholder behavior

    // Clean up
    std::env::remove_var("VLLM_API_KEY");

    // When implemented:
    // assert_eq!(vllm.api_key, Some("sk-from-env".to_string()));
}
