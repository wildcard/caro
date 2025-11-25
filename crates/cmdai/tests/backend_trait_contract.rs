// Backend trait contract tests - THESE MUST FAIL INITIALLY (TDD)
// Tests validate the CommandGenerator trait contract from specs/contracts/backend-trait.rs

use async_trait::async_trait;
use std::time::{Duration, Instant};

// Import types from the contract specification
// NOTE: These imports will fail until we implement the actual types
use cmdai::{
    backends::{BackendInfo, CommandGenerator, GeneratorError},
    models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel, SafetyLevel, ShellType},
};

/// Mock backend implementation for contract testing
struct MockBackend {
    name: String,
    should_fail: bool,
    latency: Duration,
    available: bool,
}

#[async_trait]
impl CommandGenerator for MockBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        if self.should_fail {
            return Err(GeneratorError::BackendUnavailable {
                reason: "Mock failure".to_string(),
            });
        }

        // Simulate processing time
        tokio::time::sleep(self.latency).await;

        Ok(GeneratedCommand {
            command: format!("ls {}", request.input),
            explanation: format!("List files matching '{}'", request.input),
            safety_level: RiskLevel::Safe,
            estimated_impact: Default::default(),
            alternatives: vec!["find . -name '*'".to_string()],
            backend_used: self.name.clone(),
            generation_time_ms: self.latency.as_millis() as u64,
            confidence_score: 0.95,
        })
    }

    async fn is_available(&self) -> bool {
        self.available
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Ollama,
            model_name: "mock-model".to_string(),
            supports_streaming: false,
            max_tokens: 1000,
            typical_latency_ms: self.latency.as_millis() as u64,
            memory_usage_mb: 100,
            version: "1.0.0".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        // Mock implementation always succeeds
        Ok(())
    }
}

impl MockBackend {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_fail: false,
            latency: Duration::from_millis(100),
            available: true,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = latency;
        self
    }

    fn unavailable(mut self) -> Self {
        self.available = false;
        self
    }
}

#[tokio::test]
async fn test_basic_command_generation() {
    // CONTRACT: Basic command generation with valid request
    let backend = MockBackend::new("test");
    let request = CommandRequest {
        input: "list files".to_string(),
        context: None,
        shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        backend_preference: None,
    };

    let result = backend.generate_command(&request).await;

    assert!(result.is_ok(), "Basic command generation should succeed");
    let command = result.unwrap();
    assert!(
        !command.command.is_empty(),
        "Generated command should not be empty"
    );
    assert!(
        !command.explanation.is_empty(),
        "Explanation should not be empty"
    );
    assert_eq!(
        command.backend_used, "test",
        "Backend name should be recorded"
    );
    assert!(
        command.confidence_score >= 0.0 && command.confidence_score <= 1.0,
        "Confidence score should be between 0.0 and 1.0"
    );
}

#[tokio::test]
async fn test_invalid_request_handling() {
    // CONTRACT: Handle malformed or empty requests
    let backend = MockBackend::new("test");

    // Empty input request
    let empty_request = CommandRequest {
        input: "".to_string(),
        context: None,
        shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        backend_preference: None,
    };

    // Note: Mock backend doesn't validate input, but real backends should
    // This test validates the contract interface, actual validation tested in implementation
    let _result = backend.generate_command(&empty_request).await;
    // Mock succeeds, but real backends should handle empty requests appropriately
}

#[tokio::test]
async fn test_backend_unavailable_error() {
    // CONTRACT: Handle backend unavailability gracefully
    let backend = MockBackend::new("failing").with_failure();
    let request = CommandRequest {
        input: "test command".to_string(),
        context: None,
        shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        backend_preference: None,
    };

    let result = backend.generate_command(&request).await;

    assert!(result.is_err(), "Unavailable backend should return error");

    match result.unwrap_err() {
        GeneratorError::BackendUnavailable { reason } => {
            assert!(!reason.is_empty(), "Error should include reason");
        }
        _ => panic!("Expected BackendUnavailable error"),
    }
}

#[tokio::test]
async fn test_timeout_behavior() {
    // CONTRACT: Handle timeout scenarios
    let slow_backend = MockBackend::new("slow").with_latency(Duration::from_millis(2000));
    let request = CommandRequest {
        input: "slow operation".to_string(),
        context: None,
        shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        backend_preference: None,
    };

    let start = Instant::now();

    // Test with timeout (this is a behavioral test - actual timeout logic in implementation)
    let result = tokio::time::timeout(
        Duration::from_millis(1000),
        slow_backend.generate_command(&request),
    )
    .await;

    let elapsed = start.elapsed();
    assert!(result.is_err(), "Should timeout for slow operations");
    assert!(
        elapsed < Duration::from_millis(1500),
        "Should timeout within reasonable time"
    );
}

#[tokio::test]
async fn test_concurrent_requests() {
    // CONTRACT: Handle concurrent requests safely
    let backend = MockBackend::new("concurrent");
    let requests = (0..5)
        .map(|i| CommandRequest {
            input: format!("request {}", i),
            context: None,
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            backend_preference: None,
        })
        .collect::<Vec<_>>();

    // Launch concurrent requests
    let handles: Vec<_> = requests
        .into_iter()
        .map(|req| {
            let backend_ref = &backend;
            async move { backend_ref.generate_command(&req).await }
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    // All requests should succeed
    for (i, result) in results.into_iter().enumerate() {
        assert!(result.is_ok(), "Concurrent request {} should succeed", i);
    }
}

#[tokio::test]
async fn test_availability_check() {
    // CONTRACT: Availability check should be fast and accurate
    let available_backend = MockBackend::new("available");
    let unavailable_backend = MockBackend::new("unavailable").unavailable();

    let start = Instant::now();
    assert!(
        available_backend.is_available().await,
        "Available backend should report available"
    );
    let availability_check_time = start.elapsed();

    assert!(
        !unavailable_backend.is_available().await,
        "Unavailable backend should report unavailable"
    );

    // Availability check should be fast (<100ms)
    assert!(
        availability_check_time < Duration::from_millis(100),
        "Availability check should be fast"
    );
}

#[tokio::test]
async fn test_backend_info() {
    // CONTRACT: Backend info should provide complete metadata
    let backend = MockBackend::new("info_test");
    let info = backend.backend_info();

    assert!(
        !info.model_name.is_empty(),
        "Model name should not be empty"
    );
    assert!(info.max_tokens > 0, "Max tokens should be positive");
    assert!(
        info.typical_latency_ms > 0,
        "Typical latency should be positive"
    );
    assert!(info.memory_usage_mb > 0, "Memory usage should be positive");
    assert!(!info.version.is_empty(), "Version should not be empty");
}

#[tokio::test]
async fn test_shutdown_cleanup() {
    // CONTRACT: Shutdown should cleanup resources gracefully
    let backend = MockBackend::new("shutdown_test");

    let result = backend.shutdown().await;
    assert!(
        result.is_ok(),
        "Shutdown should succeed for healthy backend"
    );

    // Test shutdown of failed backend
    let failed_backend = MockBackend::new("failed").with_failure();
    let failed_result = failed_backend.shutdown().await;
    // Shutdown should still succeed even if backend had failures
    assert!(
        failed_result.is_ok(),
        "Shutdown should succeed even for failed backend"
    );
}

#[tokio::test]
async fn test_error_serialization() {
    // CONTRACT: Errors should be serializable for logging/debugging
    let error = GeneratorError::BackendUnavailable {
        reason: "Test error".to_string(),
    };

    let serialized = serde_json::to_string(&error);
    assert!(serialized.is_ok(), "Errors should be serializable");

    let deserialized: Result<GeneratorError, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok(), "Errors should be deserializable");
}

#[tokio::test]
async fn test_response_time_requirements() {
    // CONTRACT: Response times should meet performance targets (<2s)
    let backend = MockBackend::new("performance");
    let request = CommandRequest {
        input: "performance test".to_string(),
        context: None,
        shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        backend_preference: None,
    };

    let start = Instant::now();
    let result = backend.generate_command(&request).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Performance test should succeed");
    assert!(
        elapsed < Duration::from_secs(2),
        "Response time should be under 2 seconds"
    );

    let command = result.unwrap();
    assert!(
        command.generation_time_ms < 2000,
        "Reported generation time should be under 2000ms"
    );
}

// Additional contract test for trait object safety
#[tokio::test]
async fn test_trait_object_usage() {
    // CONTRACT: Trait should work as dynamic trait object
    let backend: Box<dyn CommandGenerator> = Box::new(MockBackend::new("dynamic"));

    let request = CommandRequest {
        input: "trait object test".to_string(),
        context: None,
        shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        backend_preference: None,
    };

    let result = backend.generate_command(&request).await;
    assert!(result.is_ok(), "Trait object should work correctly");

    assert!(
        backend.is_available().await,
        "Trait object availability check should work"
    );

    let info = backend.backend_info();
    assert!(!info.model_name.is_empty(), "Trait object info should work");
}
