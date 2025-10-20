/// Integration tests for remote backend error handling and fallback
/// These tests use placeholder structs until the remote backends are implemented
use std::sync::Arc;

use cmdai::backends::embedded::{EmbeddedModelBackend, ModelVariant};
use cmdai::backends::{BackendInfo, CommandGenerator, GeneratorError};
use cmdai::models::{BackendType, CommandRequest, GeneratedCommand, ShellType};
use url::Url;

// Placeholder structs for remote backends until they are implemented
#[allow(dead_code)]
struct OllamaBackend {
    url: Url,
    model: String,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
}

#[allow(dead_code)]
impl OllamaBackend {
    pub fn new(url: Url, model: String) -> Result<Self, GeneratorError> {
        Ok(Self {
            url,
            model,
            embedded_fallback: None,
        })
    }

    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }
}

#[async_trait::async_trait]
impl CommandGenerator for OllamaBackend {
    async fn generate_command(
        &self,
        _request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        Err(GeneratorError::GenerationFailed {
            details: "OllamaBackend not yet implemented".to_string(),
        })
    }

    async fn is_available(&self) -> bool {
        false
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Ollama,
            model_name: self.model.clone(),
            supports_streaming: false,
            max_tokens: 100,
            typical_latency_ms: 2000,
            memory_usage_mb: 0,
            version: "placeholder".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

#[allow(dead_code)]
struct VllmBackend {
    url: Url,
    model: String,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
}

#[allow(dead_code)]
impl VllmBackend {
    pub fn new(url: Url, model: String) -> Result<Self, GeneratorError> {
        Ok(Self {
            url,
            model,
            embedded_fallback: None,
        })
    }

    pub fn with_api_key(self, _api_key: String) -> Self {
        self
    }

    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }
}

#[async_trait::async_trait]
impl CommandGenerator for VllmBackend {
    async fn generate_command(
        &self,
        _request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        Err(GeneratorError::GenerationFailed {
            details: "VllmBackend not yet implemented".to_string(),
        })
    }

    async fn is_available(&self) -> bool {
        false
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::VLlm,
            model_name: self.model.clone(),
            supports_streaming: false,
            max_tokens: 100,
            typical_latency_ms: 3000,
            memory_usage_mb: 0,
            version: "placeholder".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

/// Helper to create a test embedded backend for fallback
fn create_embedded_fallback() -> Arc<dyn CommandGenerator> {
    let model_path = std::env::temp_dir().join("test_model.gguf");
    if !model_path.exists() {
        std::fs::write(&model_path, b"dummy model data").ok();
    }

    Arc::new(
        EmbeddedModelBackend::with_variant_and_path(ModelVariant::detect(), model_path)
            .expect("Failed to create embedded backend"),
    )
}

/// Test Ollama backend creation and basic info
#[tokio::test]
async fn test_ollama_backend_creation() {
    let url = Url::parse("http://localhost:11434").unwrap();
    let backend = OllamaBackend::new(url, "codellama:7b".to_string());
    assert!(backend.is_ok());

    let backend = backend.unwrap();
    let info = backend.backend_info();
    assert_eq!(info.model_name, "codellama:7b");
    assert_eq!(info.typical_latency_ms, 2000);
}

/// Test vLLM backend creation and authentication setup
#[tokio::test]
async fn test_vllm_backend_creation() {
    let url = Url::parse("https://api.example.com").unwrap();
    let backend = VllmBackend::new(url, "codellama/CodeLlama-7b-hf".to_string())
        .unwrap()
        .with_api_key("test-key".to_string());

    let info = backend.backend_info();
    assert_eq!(info.model_name, "codellama/CodeLlama-7b-hf");
    assert_eq!(info.typical_latency_ms, 3000);
}

/// Test Ollama backend fallback when server is unavailable
#[tokio::test]
async fn test_ollama_fallback_on_connection_failure() {
    let embedded_fallback = create_embedded_fallback();

    // Create Ollama backend with unreachable URL
    let url = Url::parse("http://localhost:11435").unwrap(); // Different port (likely unused)
    let backend = OllamaBackend::new(url, "codellama:7b".to_string())
        .unwrap()
        .with_embedded_fallback(embedded_fallback);

    // Should not be available
    assert!(!backend.is_available().await);

    // Generate command should fallback to embedded
    let request = CommandRequest::new("list files", ShellType::Bash);
    let result = backend.generate_command(&request).await;

    assert!(result.is_ok(), "Should fallback to embedded backend");
    let command = result.unwrap();
    assert!(command.backend_used.contains("Embedded"));
    assert!(command.backend_used.contains("Ollama fallback"));
}

/// Test vLLM backend fallback when server is unavailable
#[tokio::test]
async fn test_vllm_fallback_on_connection_failure() {
    let embedded_fallback = create_embedded_fallback();

    // Create vLLM backend with unreachable URL
    let url = Url::parse("https://nonexistent.example.com").unwrap();
    let backend = VllmBackend::new(url, "codellama/CodeLlama-7b-hf".to_string())
        .unwrap()
        .with_embedded_fallback(embedded_fallback);

    // Should not be available
    assert!(!backend.is_available().await);

    // Generate command should fallback to embedded
    let request = CommandRequest::new("find text files", ShellType::Bash);
    let result = backend.generate_command(&request).await;

    assert!(result.is_ok(), "Should fallback to embedded backend");
    let command = result.unwrap();
    assert!(command.backend_used.contains("Embedded"));
    assert!(command.backend_used.contains("vLLM fallback"));
}

/// Test Ollama backend without fallback fails gracefully
#[tokio::test]
async fn test_ollama_without_fallback_fails() {
    // Create Ollama backend without fallback
    let url = Url::parse("http://localhost:11435").unwrap();
    let backend = OllamaBackend::new(url, "codellama:7b".to_string()).unwrap();

    let request = CommandRequest::new("test command", ShellType::Bash);
    let result = backend.generate_command(&request).await;

    assert!(result.is_err(), "Should fail when no fallback available");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("unavailable"));
}

/// Test vLLM backend without fallback fails gracefully
#[tokio::test]
async fn test_vllm_without_fallback_fails() {
    // Create vLLM backend without fallback
    let url = Url::parse("https://nonexistent.example.com").unwrap();
    let backend = VllmBackend::new(url, "test-model".to_string()).unwrap();

    let request = CommandRequest::new("test command", ShellType::Bash);
    let result = backend.generate_command(&request).await;

    assert!(result.is_err(), "Should fail when no fallback available");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("unavailable"));
}

/// Test backend availability checking
#[tokio::test]
async fn test_backend_availability() {
    // Test with unreachable servers
    let ollama_url = Url::parse("http://localhost:11435").unwrap();
    let ollama_backend = OllamaBackend::new(ollama_url, "test".to_string()).unwrap();
    assert!(!ollama_backend.is_available().await);

    let vllm_url = Url::parse("https://nonexistent.example.com").unwrap();
    let vllm_backend = VllmBackend::new(vllm_url, "test".to_string()).unwrap();
    assert!(!vllm_backend.is_available().await);
}

/// Test shutdown behavior
#[tokio::test]
async fn test_backend_shutdown() {
    let ollama_url = Url::parse("http://localhost:11434").unwrap();
    let ollama_backend = OllamaBackend::new(ollama_url, "test".to_string()).unwrap();
    let result = ollama_backend.shutdown().await;
    assert!(result.is_ok());

    let vllm_url = Url::parse("https://api.example.com").unwrap();
    let vllm_backend = VllmBackend::new(vllm_url, "test".to_string()).unwrap();
    let result = vllm_backend.shutdown().await;
    assert!(result.is_ok());
}

/// Test backend info and configuration
#[tokio::test]
async fn test_backend_configuration() {
    let ollama_url = Url::parse("http://localhost:11434").unwrap();
    let ollama_backend = OllamaBackend::new(ollama_url, "test".to_string()).unwrap();

    let info = ollama_backend.backend_info();
    assert_eq!(info.model_name, "test");
    assert!(!info.supports_streaming);
    assert_eq!(info.max_tokens, 100);

    let vllm_url = Url::parse("https://api.example.com").unwrap();
    let vllm_backend = VllmBackend::new(vllm_url, "test-model".to_string()).unwrap();

    let info = vllm_backend.backend_info();
    assert_eq!(info.model_name, "test-model");
    assert!(!info.supports_streaming);
    assert_eq!(info.max_tokens, 100);
}

/// Test performance and timing
#[tokio::test]
async fn test_fallback_performance() {
    use std::time::Instant;

    let embedded_fallback = create_embedded_fallback();

    // Create backend that will fail quickly
    let url = Url::parse("http://localhost:11435").unwrap();
    let backend = OllamaBackend::new(url, "test".to_string())
        .unwrap()
        .with_embedded_fallback(embedded_fallback);

    let request = CommandRequest::new("list files", ShellType::Bash);
    let start = Instant::now();
    let result = backend.generate_command(&request).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    // Should complete fallback within reasonable time
    assert!(
        duration.as_secs() < 5,
        "Fallback took too long: {:?}",
        duration
    );
}

/// Test concurrent requests with fallback
#[tokio::test]
async fn test_concurrent_requests_with_fallback() {
    let embedded_fallback = create_embedded_fallback();

    let url = Url::parse("http://localhost:11435").unwrap();
    let backend = Arc::new(
        OllamaBackend::new(url, "test".to_string())
            .unwrap()
            .with_embedded_fallback(embedded_fallback),
    );

    let mut handles = vec![];
    for i in 0..3 {
        let backend_clone = backend.clone();
        let handle = tokio::spawn(async move {
            let request = CommandRequest::new(format!("test command {}", i), ShellType::Bash);
            backend_clone.generate_command(&request).await
        });
        handles.push(handle);
    }

    // All requests should succeed with fallback
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(
            result.is_ok(),
            "Concurrent request should succeed with fallback"
        );
    }
}
