// Contract tests for OllamaBackend with embedded model fallback
// These tests verify the behavioral contract defined in:
// specs/004-implement-ollama-and/contracts/ollama-backend.md

use std::sync::Arc;
use std::time::Duration;

use cmdai::backends::{CommandGenerator, GeneratorError};
use cmdai::models::{CommandRequest, SafetyLevel, ShellType};
use url::Url;

// Placeholder struct - will be replaced with actual OllamaBackend implementation
#[allow(dead_code)]
struct OllamaBackend {
    url: Url,
    model: String,
}

impl OllamaBackend {
    pub fn new(url: Url, model: String) -> Result<Self, GeneratorError> {
        Ok(Self { url, model })
    }

    pub fn with_embedded_fallback(self, _embedded_backend: Arc<dyn CommandGenerator>) -> Self {
        // Placeholder - actual implementation will store the fallback backend
        self
    }
}

// Placeholder implementation - these will fail until real implementation exists
#[async_trait::async_trait]
impl CommandGenerator for OllamaBackend {
    async fn generate_command(
        &self,
        _request: &cmdai::models::CommandRequest,
    ) -> Result<cmdai::models::GeneratedCommand, GeneratorError> {
        Err(GeneratorError::GenerationFailed {
            details: "OllamaBackend not yet implemented".to_string(),
        })
    }

    async fn is_available(&self) -> bool {
        false // Will be true when implemented
    }

    fn backend_info(&self) -> cmdai::backends::BackendInfo {
        cmdai::backends::BackendInfo {
            backend_type: cmdai::models::BackendType::Ollama,
            model_name: self.model.clone(),
            supports_streaming: false,
            max_tokens: 4096,
            typical_latency_ms: 2000,
            memory_usage_mb: 0,
            version: "placeholder".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

/// FR-NEW-001: Embedded Model Fallback
/// MUST fallback to embedded model when Ollama backend fails or is unavailable
#[tokio::test]
async fn test_fallback_to_embedded_on_connection_failure() {
    use cmdai::backends::embedded::{EmbeddedModelBackend, ModelVariant};
    use std::path::PathBuf;

    // Create embedded fallback backend
    let embedded = EmbeddedModelBackend::with_variant_and_path(
        ModelVariant::detect(),
        PathBuf::from("/tmp/test_model.gguf"),
    )
    .unwrap();
    let embedded_arc: Arc<dyn CommandGenerator> = Arc::new(embedded);

    // Create Ollama backend with unreachable URL
    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:65534").unwrap(), // Unreachable port
        "codellama:7b".to_string(),
    )
    .unwrap()
    .with_embedded_fallback(embedded_arc);

    let request = CommandRequest::new("list files", ShellType::Bash);

    // This should fallback to embedded model (though will still fail in TDD RED phase)
    let result = ollama.generate_command(&request).await;

    // In TDD RED: This will fail because neither Ollama nor embedded is implemented
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

/// FR-NEW-002: Retry Before Fallback
/// MUST attempt retry according to retry policy before falling back
#[tokio::test]
async fn test_retry_before_fallback() {
    use cmdai::backends::embedded::{EmbeddedModelBackend, ModelVariant};
    use std::path::PathBuf;

    let embedded = EmbeddedModelBackend::with_variant_and_path(
        ModelVariant::detect(),
        PathBuf::from("/tmp/test_model.gguf"),
    )
    .unwrap();
    let embedded_arc: Arc<dyn CommandGenerator> = Arc::new(embedded);

    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(), // Potentially valid URL
        "codellama:7b".to_string(),
    )
    .unwrap()
    .with_embedded_fallback(embedded_arc);

    let request = CommandRequest::new("ls", ShellType::Bash);

    // Measure time to ensure retries are attempted
    let start = std::time::Instant::now();
    let _result = ollama.generate_command(&request).await;
    let duration = start.elapsed();

    // Should take some time if retries are attempted
    // (In real implementation, this would be more precise)
    assert!(
        duration >= Duration::from_millis(100),
        "Should attempt retries before fallback, got {:?}",
        duration
    );
}

/// FR-NEW-003: Optional Backend Status (Non-blocking)
/// MUST return false from is_available() when Ollama unreachable
#[tokio::test]
async fn test_optional_backend_status_non_blocking() {
    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:65534").unwrap(),
        "codellama:7b".to_string(),
    )
    .unwrap();

    // Health check should be fast and non-blocking
    let start = std::time::Instant::now();
    let available = ollama.is_available().await;
    let duration = start.elapsed();

    assert!(!available, "Should be unavailable with invalid URL");
    assert!(
        duration < Duration::from_secs(5),
        "Health check must be non-blocking, got {:?}",
        duration
    );
}

/// Original Contract: Basic Ollama functionality
#[tokio::test]
async fn test_ollama_generate_command_success() {
    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string(),
    )
    .unwrap();

    let request =
        CommandRequest::new("list all files", ShellType::Bash).with_safety(SafetyLevel::Moderate);

    let result = ollama.generate_command(&request).await;

    // TDD RED: This will fail until implementation exists
    assert!(
        result.is_err(),
        "TDD RED: Should fail until Ollama backend implemented"
    );

    // When implemented, we'd test:
    // assert!(result.is_ok(), "Command generation must succeed");
    // let command = result.unwrap();
    // assert!(!command.command.is_empty(), "Must generate non-empty command");
    // assert_eq!(command.backend_used, "ollama", "Must report ollama backend");
}

/// Contract: Health check functionality
#[tokio::test]
async fn test_ollama_health_check() {
    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string(),
    )
    .unwrap();

    // Multiple health checks should be consistent
    let health1 = ollama.is_available().await;
    let health2 = ollama.is_available().await;

    // Both should return the same result (likely false in test environment)
    assert_eq!(
        health1, health2,
        "Health check results should be consistent"
    );
}

/// Contract: Backend info reporting
#[test]
fn test_ollama_backend_info() {
    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string(),
    )
    .unwrap();

    let info = ollama.backend_info();

    assert_eq!(info.backend_type, cmdai::models::BackendType::Ollama);
    assert_eq!(info.model_name, "codellama:7b");
    assert!(
        !info.supports_streaming,
        "Ollama backend doesn't support streaming"
    );
    assert!(info.max_tokens > 0, "Max tokens must be positive");
    assert!(
        info.typical_latency_ms > 0,
        "Latency estimate must be positive"
    );
}

/// Contract: Graceful shutdown
#[tokio::test]
async fn test_ollama_shutdown() {
    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "codellama:7b".to_string(),
    )
    .unwrap();

    let shutdown_result = ollama.shutdown().await;
    assert!(
        shutdown_result.is_ok(),
        "Shutdown must succeed: {:?}",
        shutdown_result
    );
}

/// Contract: Error handling for invalid model
#[tokio::test]
async fn test_ollama_invalid_model_error() {
    let ollama = OllamaBackend::new(
        Url::parse("http://localhost:11434").unwrap(),
        "nonexistent:model".to_string(),
    )
    .unwrap();

    let request = CommandRequest::new("test", ShellType::Bash);
    let result = ollama.generate_command(&request).await;

    // Should fail (in both RED and GREEN phases - invalid model)
    assert!(result.is_err(), "Invalid model should cause error");

    // Check error type when implemented
    if let Err(error) = result {
        let error_msg = error.to_string();
        assert!(
            error_msg.contains("model")
                || error_msg.contains("not found")
                || error_msg.contains("failed"),
            "Error should mention model issue: {}",
            error_msg
        );
    }
}

/// Contract: Timeout handling
#[tokio::test]
async fn test_ollama_timeout_handling() {
    let ollama = OllamaBackend::new(
        Url::parse("http://httpbin.org/delay/10").unwrap(), // 10 second delay
        "test:model".to_string(),
    )
    .unwrap();

    let request = CommandRequest::new("test", ShellType::Bash);

    // Should timeout quickly (not wait 10 seconds)
    let start = std::time::Instant::now();
    let result = ollama.generate_command(&request).await;
    let duration = start.elapsed();

    assert!(result.is_err(), "Should timeout");
    assert!(
        duration < Duration::from_secs(8),
        "Should timeout before 8s, got {:?}",
        duration
    );
}

/// Contract: Concurrent request handling
#[tokio::test]
async fn test_ollama_concurrent_requests() {
    let ollama = Arc::new(
        OllamaBackend::new(
            Url::parse("http://localhost:11434").unwrap(),
            "codellama:7b".to_string(),
        )
        .unwrap(),
    );

    let mut handles = vec![];

    for i in 0..3 {
        let ollama_clone = Arc::clone(&ollama);
        let handle = tokio::spawn(async move {
            let request = CommandRequest::new(format!("test {}", i), ShellType::Bash);
            ollama_clone.generate_command(&request).await
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
