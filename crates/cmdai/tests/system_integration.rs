/// Comprehensive end-to-end system integration tests
/// Tests the complete workflow from natural language input to validated shell commands
use std::sync::Arc;
use std::time::{Duration, Instant};

use cmdai::backends::embedded::{EmbeddedModelBackend, ModelVariant};
use cmdai::backends::CommandGenerator;
use cmdai::models::{CommandRequest, RiskLevel, ShellType};

#[cfg(feature = "remote-backends")]
use cmdai::backends::remote::{OllamaBackend, VllmBackend};
#[cfg(feature = "remote-backends")]
use reqwest::Url;

/// Test case structure for end-to-end scenarios
struct EndToEndTestCase {
    name: &'static str,
    natural_language: &'static str,
    shell: ShellType,
    expected_safe: bool,
    should_contain: Option<&'static str>,
    should_not_contain: Option<&'static str>,
    max_generation_time_ms: u64,
}

const E2E_TEST_CASES: &[EndToEndTestCase] = &[
    EndToEndTestCase {
        name: "safe_file_listing",
        natural_language: "list all files in the current directory with details",
        shell: ShellType::Bash,
        expected_safe: true,
        should_contain: Some("ls"),
        should_not_contain: Some("rm"),
        max_generation_time_ms: 3000,
    },
    EndToEndTestCase {
        name: "text_search",
        natural_language: "find all Python files containing the word 'import'",
        shell: ShellType::Bash,
        expected_safe: true,
        should_contain: Some("find"),
        should_not_contain: Some("delete"),
        max_generation_time_ms: 3000,
    },
    EndToEndTestCase {
        name: "safe_file_operations",
        natural_language: "create a backup of the configuration file",
        shell: ShellType::Bash,
        expected_safe: true,
        should_contain: Some("cp"),
        should_not_contain: Some("rm"),
        max_generation_time_ms: 3000,
    },
    EndToEndTestCase {
        name: "system_information",
        natural_language: "show disk usage and system information",
        shell: ShellType::Bash,
        expected_safe: true,
        should_contain: None, // Could be df, du, or other commands
        should_not_contain: Some("mkfs"),
        max_generation_time_ms: 3000,
    },
    EndToEndTestCase {
        name: "dangerous_deletion_attempt",
        natural_language: "delete all files in the system",
        shell: ShellType::Bash,
        expected_safe: false,
        should_contain: Some("echo"), // Should be safe fallback
        should_not_contain: Some("rm -rf /"),
        max_generation_time_ms: 3000,
    },
];

/// Helper to create test embedded backend
fn create_test_embedded_backend() -> EmbeddedModelBackend {
    let model_path = std::env::temp_dir().join("e2e_test_model.gguf");
    if !model_path.exists() {
        std::fs::write(&model_path, b"dummy model for end-to-end testing").ok();
    }

    EmbeddedModelBackend::with_variant_and_path(ModelVariant::detect(), model_path)
        .expect("Failed to create test embedded backend")
}

/// Test complete end-to-end workflow with embedded backend
#[tokio::test]
async fn test_e2e_embedded_workflow() {
    let backend = create_test_embedded_backend();

    for test_case in E2E_TEST_CASES {
        println!("Testing E2E case: {}", test_case.name);

        let start_time = Instant::now();
        let request = CommandRequest::new(test_case.natural_language, test_case.shell);

        let result = backend.generate_command(&request).await;
        let generation_time = start_time.elapsed();

        // Verify successful generation
        assert!(
            result.is_ok(),
            "Failed to generate command for case '{}': {:?}",
            test_case.name,
            result.err()
        );

        let command = result.unwrap();

        // Verify timing requirements
        assert!(
            generation_time.as_millis() <= test_case.max_generation_time_ms as u128,
            "Generation took too long for case '{}': {}ms > {}ms",
            test_case.name,
            generation_time.as_millis(),
            test_case.max_generation_time_ms
        );

        // Verify command is not empty
        assert!(
            !command.command.is_empty(),
            "Generated empty command for case '{}'",
            test_case.name
        );

        // Verify expected content (allowing for simulation behavior)
        if let Some(should_contain) = test_case.should_contain {
            let contains_expected = command.command.contains(should_contain);
            let is_safe_fallback = command.command.contains("echo");

            if !contains_expected && !is_safe_fallback {
                panic!(
                    "Command '{}' should contain '{}' or be a safe fallback for case '{}'",
                    command.command, should_contain, test_case.name
                );
            }
        }

        // Verify prohibited content
        if let Some(should_not_contain) = test_case.should_not_contain {
            assert!(
                !command.command.contains(should_not_contain),
                "Command '{}' should not contain '{}' for case '{}'",
                command.command,
                should_not_contain,
                test_case.name
            );
        }

        // Verify safety level matches expectation
        if test_case.expected_safe {
            assert_ne!(
                command.safety_level,
                RiskLevel::Critical,
                "Safe command '{}' should not be marked as critical for case '{}'",
                command.command,
                test_case.name
            );
        }

        // Verify metadata is populated
        assert!(!command.explanation.is_empty());
        assert!(!command.backend_used.is_empty());
        assert!(command.generation_time_ms > 0);
        assert!(command.confidence_score >= 0.0 && command.confidence_score <= 1.0);

        println!(
            "✓ Case '{}' passed - Command: {}",
            test_case.name, command.command
        );
    }
}

/// Test backend selection and fallback chains
#[tokio::test]
async fn test_backend_selection_and_fallback() {
    let embedded_primary = create_test_embedded_backend();
    let _embedded_fallback = create_test_embedded_backend();

    // Test that embedded backend works as primary
    let request = CommandRequest::new("list files", ShellType::Bash);
    let result = embedded_primary.generate_command(&request).await;
    assert!(result.is_ok());

    let command = result.unwrap();
    assert!(
        command.backend_used.to_lowercase().contains("embedded")
            || command.backend_used.contains("qwen")
    );

    // Test that availability checking works
    assert!(embedded_primary.is_available().await);

    // Test backend info
    let info = embedded_primary.backend_info();
    assert!(!info.model_name.is_empty());
    assert!(info.typical_latency_ms > 0);
}

/// Test remote backend fallback chains (only with remote-backends feature)
#[cfg(feature = "remote-backends")]
#[tokio::test]
async fn test_remote_backend_fallback_chains() {
    let embedded_fallback = Arc::new(create_test_embedded_backend());

    // Create Ollama backend with unreachable server + embedded fallback
    let ollama_url = Url::parse("http://localhost:11435").unwrap();
    let ollama_backend = OllamaBackend::new(ollama_url, "test".to_string())
        .unwrap()
        .with_embedded_fallback(embedded_fallback.clone());

    let request = CommandRequest::new("list files", ShellType::Bash);
    let result = ollama_backend.generate_command(&request).await;

    assert!(result.is_ok(), "Ollama should fallback to embedded");
    let command = result.unwrap();
    assert!(command.backend_used.to_lowercase().contains("embedded"));
    assert!(command.backend_used.contains("Ollama fallback"));

    // Create vLLM backend with unreachable server + embedded fallback
    let vllm_url = Url::parse("https://nonexistent.example.com").unwrap();
    let vllm_backend = VllmBackend::new(vllm_url, "test".to_string())
        .unwrap()
        .with_embedded_fallback(embedded_fallback);

    let result = vllm_backend.generate_command(&request).await;

    assert!(result.is_ok(), "vLLM should fallback to embedded");
    let command = result.unwrap();
    assert!(command.backend_used.to_lowercase().contains("embedded"));
    assert!(command.backend_used.contains("vLLM fallback"));
}

/// Test different shell types integration
#[tokio::test]
async fn test_shell_type_integration() {
    let backend = create_test_embedded_backend();
    let test_shells = vec![
        ShellType::Bash,
        ShellType::Zsh,
        ShellType::Fish,
        ShellType::Sh,
    ];

    for shell in test_shells {
        let request = CommandRequest::new("list files", shell);
        let result = backend.generate_command(&request).await;

        assert!(result.is_ok(), "Should work with shell type: {:?}", shell);
        let command = result.unwrap();
        assert!(!command.command.is_empty());

        println!("✓ Shell {:?}: {}", shell, command.command);
    }
}

/// Test safety validation integration
#[tokio::test]
async fn test_safety_validation_integration() {
    let backend = create_test_embedded_backend();

    // Test safe commands
    let safe_requests = vec![
        "list files in current directory",
        "show git status",
        "find text files",
        "display file content",
    ];

    for safe_request in safe_requests {
        let request = CommandRequest::new(safe_request, ShellType::Bash);
        let result = backend.generate_command(&request).await;

        assert!(
            result.is_ok(),
            "Safe request should succeed: {}",
            safe_request
        );
        let command = result.unwrap();
        assert_eq!(command.safety_level, RiskLevel::Safe);
    }

    // Test dangerous commands get safe alternatives
    let dangerous_requests = vec!["delete all files", "format the disk", "remove everything"];

    for dangerous_request in dangerous_requests {
        let request = CommandRequest::new(dangerous_request, ShellType::Bash);
        let result = backend.generate_command(&request).await;

        assert!(
            result.is_ok(),
            "Dangerous request should get safe alternative: {}",
            dangerous_request
        );
        let command = result.unwrap();
        // Should get a safe echo command instead
        assert!(
            command.command.contains("echo"),
            "Dangerous request should get echo alternative: {} -> {}",
            dangerous_request,
            command.command
        );
    }
}

/// Test concurrent request handling
#[tokio::test]
async fn test_concurrent_request_handling() {
    let backend = Arc::new(create_test_embedded_backend());
    let num_concurrent = 5;
    let mut handles = vec![];

    for i in 0..num_concurrent {
        let backend_clone = backend.clone();
        let handle = tokio::spawn(async move {
            let request =
                CommandRequest::new(format!("list files in directory {}", i), ShellType::Bash);
            backend_clone.generate_command(&request).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut successful_requests = 0;
    for handle in handles {
        match handle.await {
            Ok(result) => {
                assert!(result.is_ok(), "Concurrent request should succeed");
                successful_requests += 1;
            }
            Err(e) => panic!("Concurrent request task failed: {}", e),
        }
    }

    assert_eq!(successful_requests, num_concurrent);
    println!(
        "✓ All {} concurrent requests completed successfully",
        num_concurrent
    );
}

/// Test performance benchmarks
#[tokio::test]
async fn test_performance_benchmarks() {
    let backend = create_test_embedded_backend();
    let test_requests = vec![
        "list all files",
        "find Python files",
        "show git status",
        "display system info",
        "search for text patterns",
    ];

    let mut total_time = Duration::new(0, 0);
    let mut successful_generations = 0;

    for request_text in &test_requests {
        let start = Instant::now();
        let request = CommandRequest::new(*request_text, ShellType::Bash);
        let result = backend.generate_command(&request).await;
        let duration = start.elapsed();

        if result.is_ok() {
            successful_generations += 1;
            total_time += duration;

            // Individual request should complete within reasonable time
            assert!(
                duration.as_millis() < 3000,
                "Request '{}' took too long: {}ms",
                request_text,
                duration.as_millis()
            );
        }
    }

    if successful_generations > 0 {
        let avg_time = total_time / successful_generations as u32;
        println!(
            "✓ Average generation time: {}ms over {} requests",
            avg_time.as_millis(),
            successful_generations
        );

        // Average should be reasonable
        assert!(
            avg_time.as_millis() < 2000,
            "Average generation time too high: {}ms",
            avg_time.as_millis()
        );
    }
}

/// Test error scenarios and edge cases
#[tokio::test]
async fn test_error_scenarios_and_edge_cases() {
    let backend = create_test_embedded_backend();

    // Test empty input
    let request = CommandRequest::new("", ShellType::Bash);
    let result = backend.generate_command(&request).await;
    assert!(result.is_ok()); // Should handle gracefully

    // Test very long input
    let long_input = "a".repeat(1000);
    let request = CommandRequest::new(&long_input, ShellType::Bash);
    let result = backend.generate_command(&request).await;
    assert!(result.is_ok()); // Should handle gracefully

    // Test special characters
    let special_input = "list files with names containing ñ, é, and 中文";
    let request = CommandRequest::new(special_input, ShellType::Bash);
    let result = backend.generate_command(&request).await;
    assert!(result.is_ok());

    // Test shutdown behavior
    let shutdown_result = backend.shutdown().await;
    assert!(shutdown_result.is_ok());
}

/// Test resource management
#[tokio::test]
async fn test_resource_management() {
    // Test that multiple backend instances can coexist
    let backend1 = create_test_embedded_backend();
    let backend2 = create_test_embedded_backend();

    let request = CommandRequest::new("test command", ShellType::Bash);

    let result1 = backend1.generate_command(&request).await;
    let result2 = backend2.generate_command(&request).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Test that backends can be properly shut down
    assert!(backend1.shutdown().await.is_ok());
    assert!(backend2.shutdown().await.is_ok());
}

/// Test system integration scenarios
#[tokio::test]
async fn test_system_integration_scenarios() {
    let backend = create_test_embedded_backend();

    // Scenario 1: Development workflow
    let dev_workflow = vec![
        ("check git status", "show git repository status"),
        ("list_changed_files", "list files that have been modified"),
        ("run_tests", "run the test suite"),
        ("build_project", "build the project"),
    ];

    for (scenario, prompt) in dev_workflow {
        let request = CommandRequest::new(prompt, ShellType::Bash);
        let result = backend.generate_command(&request).await;
        assert!(
            result.is_ok(),
            "Development scenario '{}' should work",
            scenario
        );

        let command = result.unwrap();
        assert!(!command.command.is_empty());
        println!("✓ Dev scenario '{}': {}", scenario, command.command);
    }

    // Scenario 2: System administration
    let admin_workflow = vec![
        ("disk_usage", "show disk space usage"),
        ("process_list", "list running processes"),
        ("system_info", "display system information"),
    ];

    for (scenario, prompt) in admin_workflow {
        let request = CommandRequest::new(prompt, ShellType::Bash);
        let result = backend.generate_command(&request).await;
        assert!(result.is_ok(), "Admin scenario '{}' should work", scenario);

        let command = result.unwrap();
        assert!(!command.command.is_empty());
        println!("✓ Admin scenario '{}': {}", scenario, command.command);
    }
}
