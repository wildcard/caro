# Quickstart: Embedded Model + Remote Backend Integration Tests

**Feature**: 004-implement-ollama-and
**Date**: 2025-10-14 (Updated)
**Purpose**: End-to-end integration test scenarios for embedded Qwen model (primary) and remote backends (Ollama, vLLM - optional)

---

## Overview

This document defines comprehensive integration test scenarios that validate the complete workflow from user input through backend selection, command generation, safety validation, and result delivery. Each scenario maps directly to acceptance criteria from [spec.md](./spec.md).

**Architecture**: Embedded Qwen model (always available) → Remote backends (optional enhancements with fallback)

---

## Scenario 1: First-Time User with Embedded Model (Batteries-Included Experience)

**User Story**: As a first-time caro user, I want to generate my first command immediately without any setup so that I can start using caro right away.

**Preconditions**:
- Fresh caro installation (embedded Qwen model included in binary)
- No remote backends installed (no Ollama, no vLLM)
- No custom configuration file (using defaults)
- Works completely offline (no network required)

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_1_first_time_embedded_model() {
    // Step 1: Initialize CLI with default config (embedded model auto-detected)
    let cli = CliApp::new().await.expect("CLI initialization failed");

    // Step 2: User provides natural language input (no setup required)
    let args = TestArgs {
        prompt: Some("list all files in current directory".to_string()),
        shell: None,  // Default to detected shell
        backend: None,  // Default to embedded model (FR-012)
        ..Default::default()
    };

    // Step 3: Generate command (offline, batteries-included)
    let start = Instant::now();
    let result = cli.run_with_args(args).await;
    let duration = start.elapsed();

    // Assertions
    assert!(result.is_ok(), "Command generation should succeed offline");
    let cli_result = result.unwrap();

    // FR-001: Embedded model used (batteries-included)
    assert_eq!(cli_result.backend_used, "embedded");
    assert!(cli_result.model_variant == "mlx" || cli_result.model_variant == "cpu");

    // FR-024: Offline operation (no network calls)
    assert_eq!(cli_result.network_requests, 0, "Must work completely offline");

    // FR-005: Command generated
    assert!(!cli_result.generated_command.is_empty());
    assert!(cli_result.generated_command.contains("ls") ||
            cli_result.generated_command.contains("dir"));

    // FR-008: Safety validation applied (same as remote backends)
    assert!(cli_result.validation_passed);
    assert_eq!(cli_result.risk_level, RiskLevel::Safe);

    // FR-025: Performance targets (platform-specific)
    if cli_result.model_variant == "mlx" {
        // MLX GPU: <2s inference (FR-025)
        assert!(duration < Duration::from_secs(2),
                "MLX inference took {}ms", duration.as_millis());
    } else {
        // Candle CPU: <5s acceptable
        assert!(duration < Duration::from_secs(5),
                "CPU inference took {}ms", duration.as_millis());
    }

    // FR-027: Startup time included in first call
    // (Lazy load on first inference, <100ms for MLX, <500ms for CPU)

    // User experience: Explanation provided
    assert!(!cli_result.explanation.is_empty());

    println!("✓ Scenario 1 passed: Generated '{}' in {}ms using {} variant",
             cli_result.generated_command, duration.as_millis(), cli_result.model_variant);
}
```

**Expected Output** (MLX on Apple Silicon):
```
Generated command: ls -la
Explanation: Lists all files including hidden ones in current directory
Backend: embedded (qwen2.5-coder-1.5b-q4, mlx variant)
Risk level: Safe
Generation time: 1.8s (includes 95ms model load)
Status: ✓ Offline, no network required
```

---

## Scenario 2: Switching to vLLM Backend

**User Story**: As a caro user with vLLM configured, I want to use vLLM instead of Ollama so that I can leverage enterprise models.

**Preconditions**:
- Both Ollama and vLLM available
- vLLM configured in `~/.caro/config.toml`
- User specifies `--backend vllm` flag

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_2_vllm_backend_selection() {
    // Setup: Configure vLLM
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(&config_path, r#"
        preferred_backend = "vllm"

        [backends.vllm]
        url = "https://vllm-test.example.com"
        model = "meta-llama/Llama-2-7b-hf"
        api_key = "test-key-12345"
    "#).unwrap();

    let cli = CliApp::with_config(config_path).await.unwrap();

    // User explicitly requests vLLM
    let args = TestArgs {
        prompt: Some("find large files over 100MB".to_string()),
        backend: Some("vllm".to_string()),
        ..Default::default()
    };

    let result = cli.run_with_args(args).await;
    assert!(result.is_ok());

    let cli_result = result.unwrap();

    // FR-002: vLLM backend used
    assert_eq!(cli_result.backend_used, "vllm");

    // FR-009: Backend preference respected
    assert!(cli_result.debug_info.unwrap().contains("vllm"));

    // FR-010: Configuration loaded
    assert_eq!(cli_result.model_used, "meta-llama/Llama-2-7b-hf");

    println!("✓ Scenario 2 passed: vLLM backend used successfully");
}
```

**Expected Output**:
```
Generated command: find . -type f -size +100M
Backend: vllm (meta-llama/Llama-2-7b-hf)
Configuration: Loaded from ~/.caro/config.toml
```

---

## Scenario 3: Automatic Fallback on Failure

**User Story**: As a caro user, I want the tool to automatically try alternative backends if my primary choice fails so that I get results even with network issues.

**Preconditions**:
- Ollama configured but not running (simulate failure)
- vLLM configured and available
- Config: `preferred_backend = "ollama"`

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_3_automatic_fallback() {
    // Setup: Ollama unavailable, vLLM available
    let cli = CliApp::new().await.unwrap();

    // Simulate Ollama failure by using invalid port
    let args = TestArgs {
        prompt: Some("create a directory named test".to_string()),
        backend: Some("auto".to_string()),  // Auto-detect with fallback
        ..Default::default()
    };

    let result = cli.run_with_args(args).await;
    assert!(result.is_ok(), "Should fallback to vLLM");

    let cli_result = result.unwrap();

    // FR-004: Fallback occurred
    assert_eq!(cli_result.backend_used, "vllm");

    // NFR-006: User informed of fallback
    assert!(cli_result.warnings.iter().any(|w|
        w.contains("Ollama") && w.contains("fallback")
    ));

    // Command still generated successfully
    assert!(!cli_result.generated_command.is_empty());
    assert!(cli_result.generated_command.contains("mkdir"));

    println!("✓ Scenario 3 passed: Automatic fallback from Ollama to vLLM");
}
```

**Expected Output**:
```
⚠ Warning: Ollama backend unavailable, falling back to vLLM
Generated command: mkdir test
Backend: vllm (fallback)
```

---

## Scenario 4: No Backends Available (Error Handling)

**User Story**: As a caro user with no backends available, I want clear guidance on how to fix the issue so that I can set up a backend correctly.

**Preconditions**:
- No Ollama running
- No vLLM configured
- No other backends available

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_4_no_backends_available() {
    // Setup: All backends unavailable
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("test command".to_string()),
        ..Default::default()
    };

    let result = cli.run_with_args(args).await;

    // FR-013: Clear error message
    assert!(result.is_err());
    let error = result.unwrap_err();

    // NFR-004: Actionable troubleshooting steps
    let error_msg = error.to_string();
    assert!(error_msg.contains("No LLM backends available"));
    assert!(error_msg.contains("ollama serve") || error_msg.contains("Install"));
    assert!(error_msg.contains("config.toml") || error_msg.contains("vLLM"));

    // Error should provide specific next steps
    assert!(error_msg.contains("1.") || error_msg.contains("To use"));

    println!("✓ Scenario 4 passed: Clear error with troubleshooting steps");
}
```

**Expected Output**:
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

---

## Scenario 5: Configuration Persistence

**User Story**: As a caro user, I want my backend preferences saved so that I don't have to specify them every time.

**Preconditions**:
- First run: No config file exists
- User specifies backend via flag
- Subsequent runs: Config file exists

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_5_configuration_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // First run: No config, specify via CLI
    let cli1 = CliApp::with_config(config_path.clone()).await.unwrap();
    let args1 = TestArgs {
        prompt: Some("test command".to_string()),
        backend: Some("vllm".to_string()),
        save_preferences: Some(true),
        ..Default::default()
    };

    let result1 = cli1.run_with_args(args1).await;
    assert!(result1.is_ok());

    // FR-012: Configuration persisted
    assert!(config_path.exists());
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("preferred_backend"));
    assert!(config_content.contains("vllm"));

    // Second run: Config auto-loaded
    let cli2 = CliApp::with_config(config_path).await.unwrap();
    let args2 = TestArgs {
        prompt: Some("another test".to_string()),
        // No backend specified - should use saved preference
        ..Default::default()
    };

    let result2 = cli2.run_with_args(args2).await;
    assert!(result2.is_ok());

    let cli_result2 = result2.unwrap();
    assert_eq!(cli_result2.backend_used, "vllm");

    println!("✓ Scenario 5 passed: Configuration persisted and reused");
}
```

**Expected Output (Run 1)**:
```
Backend: vllm
Configuration saved to ~/.caro/config.toml
```

**Expected Output (Run 2)**:
```
Backend: vllm (from config)
Configuration loaded from ~/.caro/config.toml
```

---

## Scenario 6: Network Timeout with Retry

**User Story**: As a caro user with slow network, I want automatic retries so that temporary issues don't cause immediate failures.

**Preconditions**:
- vLLM configured (remote backend)
- Simulated slow network (1st request fails, 2nd succeeds)

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_6_network_timeout_retry() {
    // Setup: Mock server with first-request failure
    let mock_server = MockServer::start().await;

    // First request: Timeout
    Mock::given(method("POST"))
        .and(path("/v1/completions"))
        .respond_with(ResponseTemplate::new(503).set_delay(Duration::from_secs(6)))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Second request: Success
    Mock::given(method("POST"))
        .and(path("/v1/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"text": "ls -la", "finish_reason": "stop"}],
            "usage": {"total_tokens": 10}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let backend = VllmBackend::new(
        Url::parse(&mock_server.uri()).unwrap(),
        "test-model".to_string()
    ).unwrap()
    .with_retry_policy(RetryPolicy::default());

    let request = CommandRequest::new("test", ShellType::Bash);

    let start = Instant::now();
    let result = backend.generate_command(&request).await;
    let duration = start.elapsed();

    // NFR-003: Retry occurred
    assert!(result.is_ok());
    assert!(duration > Duration::from_secs(1)); // Backoff delay

    // Command generated after retry
    let command = result.unwrap();
    assert_eq!(command.command, "ls -la");

    println!("✓ Scenario 6 passed: Retry succeeded after initial timeout");
}
```

**Expected Output**:
```
⚠ Warning: vLLM request timed out, retrying (attempt 1/2)
✓ Success: Command generated after 1 retry
Generated command: ls -la
Total time: 2.1s (including retry delay)
```

---

## Scenario 7: Malformed JSON Response

**User Story**: As a caro user, I want the tool to handle backend errors gracefully so that I get useful results even with imperfect API responses.

**Preconditions**:
- Ollama running
- Simulated malformed JSON response

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_7_malformed_json_fallback() {
    let mock_server = MockServer::start().await;

    // Return JSON with trailing comma (invalid)
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{
            "response": "pwd",
            "done": true,
        }"#))  // Note trailing comma - invalid JSON
        .mount(&mock_server)
        .await;

    let backend = OllamaBackend::new(
        Url::parse(&mock_server.uri()).unwrap(),
        "test-model".to_string()
    ).unwrap();

    let request = CommandRequest::new("show directory", ShellType::Bash);
    let result = backend.generate_command(&request).await;

    // FR-015: Fallback parser handles malformed JSON
    assert!(result.is_ok(), "Fallback parser should succeed");

    let command = result.unwrap();
    assert_eq!(command.command, "pwd");

    println!("✓ Scenario 7 passed: Fallback parser recovered from malformed JSON");
}
```

**Expected Output**:
```
⚠ Warning: Standard JSON parser failed, using fallback parser
Generated command: pwd
```

---

## Scenario 8: Safety Validation Integration

**User Story**: As a caro user, I want all generated commands validated for safety regardless of which backend generated them so that I'm protected from dangerous commands.

**Preconditions**:
- Ollama backend available
- Safety validator enabled (default)

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_8_safety_validation_integration() {
    let cli = CliApp::new().await.unwrap();

    // Request a potentially dangerous command
    let args = TestArgs {
        prompt: Some("delete all temporary files recursively".to_string()),
        safety: Some("strict".to_string()),
        confirm: false,  // No auto-confirm
        ..Default::default()
    };

    let result = cli.run_with_args(args).await;
    assert!(result.is_ok());

    let cli_result = result.unwrap();

    // FR-008: Safety validation applied
    assert!(cli_result.validation_performed);
    assert_eq!(cli_result.risk_level, RiskLevel::Moderate);

    // Command blocked due to safety level
    assert!(!cli_result.executed);
    assert!(cli_result.requires_confirmation);

    // Explanation provided
    assert!(!cli_result.blocked_reason.unwrap().is_empty());
    assert!(!cli_result.confirmation_prompt.is_empty());

    // Alternatives suggested
    assert!(!cli_result.alternatives.is_empty());

    println!("✓ Scenario 8 passed: Safety validation protected user");
}
```

**Expected Output**:
```
Generated command: rm -rf /tmp/*
Risk level: Moderate
Status: ⚠ Requires confirmation

This command will recursively delete files. Proceed? (y/N)

Safer alternatives:
  1. rm /tmp/*.tmp  (delete only .tmp files)
  2. find /tmp -mtime +7 -delete  (delete files older than 7 days)
```

---

## Scenario 9: Performance Monitoring

**User Story**: As a caro developer, I want detailed performance metrics so that I can identify bottlenecks and optimize the system.

**Preconditions**:
- Verbose mode enabled
- Ollama backend available

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_9_performance_monitoring() {
    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("list files".to_string()),
        verbose: true,
        ..Default::default()
    };

    let result = cli.run_with_args(args).await;
    assert!(result.is_ok());

    let cli_result = result.unwrap();

    // NFR-005: Performance metrics visible in verbose mode
    assert!(cli_result.debug_info.is_some());
    let debug = cli_result.debug_info.unwrap();

    assert!(debug.contains("backend_selection_ms"));
    assert!(debug.contains("generation_time_ms"));
    assert!(debug.contains("safety_validation_ms"));
    assert!(debug.contains("total_time_ms"));

    // FR-017: Backend selection < 500ms
    let timing = cli_result.timing_info;
    assert!(timing.backend_selection_ms < 500);

    // FR-019: Health check cached
    assert!(debug.contains("cache_hit") || debug.contains("cache_miss"));

    println!("✓ Scenario 9 passed: Performance metrics captured");
    println!("  Backend selection: {}ms", timing.backend_selection_ms);
    println!("  Generation: {}ms", timing.generation_time_ms);
    println!("  Safety validation: {}ms", timing.safety_validation_ms);
    println!("  Total: {}ms", timing.total_time_ms);
}
```

**Expected Output**:
```
Performance Metrics:
  Backend selection: 142ms (cache hit)
  Generation: 2,341ms (ollama)
  Safety validation: 23ms
  Total: 2,506ms

Backend health: Healthy
Last check: 45s ago
Average latency: 2.1s
```

---

## Test Execution Matrix

| Scenario | Embedded | Ollama | vLLM | Safety | Config | Network |
|----------|----------|--------|------|--------|--------|---------|
| 1. First-time user | ✓ | - | - | ✓ | Default | Offline |
| 2. Backend switch | - | ✓ | ✓ | ✓ | Custom | Remote |
| 3. Auto fallback | ✓ | ✗ | ✓ | ✓ | Custom | Mixed |
| 4. No backends | ✓ | ✗ | ✗ | ✓ | Default | Offline |
| 5. Config persist | - | - | ✓ | ✓ | Save | Remote |
| 6. Network retry | ✓ | - | ✓ | ✓ | Custom | Slow |
| 7. Malformed JSON | ✓ | ✓ | - | ✓ | Default | Local |
| 8. Safety block | ✓ | ✓ | - | ✓ | Strict | Local |
| 9. Performance | ✓ | ✓ | - | ✓ | Default | Local |
| 10. MLX vs CPU | ✓ | - | - | ✓ | Default | Offline |
| 11. Embedded→Ollama | ✓ | ✓ | - | ✓ | Custom | Local |
| 12. Offline guarantee | ✓ | - | - | ✓ | Default | Offline |

**Legend**: ✓ Used, ✗ Unavailable, - Not primary focus

---

## Scenario 10: MLX vs CPU Performance Comparison

**User Story**: As a developer, I want to verify that MLX GPU inference is faster than CPU fallback on Apple Silicon.

**Preconditions**:
- macOS with Apple Silicon (M1/M2/M3/M4)
- Both MLX and CPU backends compiled
- Same Qwen model weights available

**Test Steps**:

```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test]
async fn test_scenario_10_mlx_vs_cpu_performance() {
    let cli = CliApp::new().await.unwrap();

    // Test MLX variant
    let args_mlx = TestArgs {
        prompt: Some("find files larger than 100MB".to_string()),
        backend: Some("embedded".to_string()),
        ..Default::default()
    };

    let start_mlx = Instant::now();
    let result_mlx = cli.run_with_args(args_mlx).await.unwrap();
    let duration_mlx = start_mlx.elapsed();

    assert_eq!(result_mlx.model_variant, "mlx");
    assert!(duration_mlx < Duration::from_secs(2), "MLX should be <2s");

    // Force CPU variant (for comparison only)
    std::env::set_var("CARO_FORCE_CPU", "1");
    let cli_cpu = CliApp::new().await.unwrap();

    let args_cpu = TestArgs {
        prompt: Some("find files larger than 100MB".to_string()),
        backend: Some("embedded".to_string()),
        ..Default::default()
    };

    let start_cpu = Instant::now();
    let result_cpu = cli_cpu.run_with_args(args_cpu).await.unwrap();
    let duration_cpu = start_cpu.elapsed();

    assert_eq!(result_cpu.model_variant, "cpu");

    // MLX should be at least 2x faster than CPU
    assert!(duration_mlx < duration_cpu,
            "MLX ({}ms) should be faster than CPU ({}ms)",
            duration_mlx.as_millis(), duration_cpu.as_millis());

    println!("✓ Scenario 10 passed: MLX {}ms vs CPU {}ms ({}x speedup)",
             duration_mlx.as_millis(), duration_cpu.as_millis(),
             duration_cpu.as_millis() / duration_mlx.as_millis());
}
```

**Expected Output**:
```
MLX GPU: 1.8s
CPU fallback: 4.2s
Speedup: 2.3x faster with MLX
```

---

## Scenario 11: Embedded Model Fallback from Ollama Failure

**User Story**: As a user with Ollama configured, I want caro to automatically fallback to embedded model when Ollama fails, so my workflow isn't interrupted.

**Preconditions**:
- Config file with Ollama as preferred backend
- Ollama service stopped (not running)
- Embedded model available

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_11_embedded_fallback_from_ollama() {
    // Create config preferring Ollama
    let config = UserConfiguration {
        preferred_backend: "ollama".to_string(),
        ollama_url: Some("http://localhost:11434".to_string()),
        ollama_model: Some("codellama:7b".to_string()),
        ..Default::default()
    };

    save_test_config(&config).await.unwrap();

    // Ensure Ollama is NOT running
    assert!(!is_ollama_running(), "Ollama must be stopped for this test");

    let cli = CliApp::new().await.unwrap();

    let args = TestArgs {
        prompt: Some("delete temporary files".to_string()),
        backend: None,  // Use config preference (Ollama)
        ..Default::default()
    };

    let start = Instant::now();
    let result = cli.run_with_args(args).await;
    let duration = start.elapsed();

    // Should succeed via embedded model fallback
    assert!(result.is_ok(), "Must fallback to embedded model");
    let cli_result = result.unwrap();

    // FR-026: Automatic fallback without user intervention
    assert_eq!(cli_result.backend_used, "embedded",
               "Should fallback from Ollama to embedded");

    // User should see warning in logs
    assert!(cli_result.warnings.iter().any(|w| w.contains("Ollama")),
            "Should warn about Ollama failure");

    // Command still generated successfully
    assert!(!cli_result.generated_command.is_empty());

    // Performance should meet embedded model targets
    assert!(duration < Duration::from_secs(5));

    println!("✓ Scenario 11 passed: Ollama failed → embedded model ({}ms)",
             duration.as_millis());
}
```

**Expected Output**:
```
⚠ Warning: Ollama backend unavailable, using embedded model
Generated command: find /tmp -type f -mtime +7 -delete
Backend: embedded (fallback from ollama)
Risk level: High (requires confirmation)
Generation time: 1.9s
```

---

## Scenario 12: Complete Offline Operation

**User Story**: As a user in an offline environment, I want caro to work without any network access, proving true batteries-included capability.

**Preconditions**:
- Network completely disabled (airplane mode)
- No remote backends configured
- Fresh caro installation with embedded model

**Test Steps**:

```rust
#[tokio::test]
async fn test_scenario_12_complete_offline_operation() {
    // Disable all network (simulate airplane mode)
    std::env::set_var("NO_NETWORK", "1");
    block_all_network_access();

    let cli = CliApp::new().await.expect("Should work offline");

    // Test multiple commands to ensure consistency
    let test_prompts = vec![
        "list all files",
        "find text in files",
        "check disk usage",
        "create a directory",
        "compress a folder",
    ];

    for prompt in test_prompts {
        let args = TestArgs {
            prompt: Some(prompt.to_string()),
            backend: None,  // Default to embedded
            ..Default::default()
        };

        let result = cli.run_with_args(args).await;

        assert!(result.is_ok(),
                "Prompt '{}' should work offline", prompt);

        let cli_result = result.unwrap();

        // FR-024: No network dependency
        assert_eq!(cli_result.network_requests, 0,
                   "Must make zero network requests");

        // FR-031: Works on all platforms offline
        assert_eq!(cli_result.backend_used, "embedded");

        // Command generated
        assert!(!cli_result.generated_command.is_empty());

        println!("✓ Offline test passed: '{}'", prompt);
    }

    println!("✓ Scenario 12 passed: All commands work completely offline");
}
```

**Expected Output**:
```
Testing offline operation (5 prompts)...
  ✓ list all files → ls -la
  ✓ find text in files → grep -r "pattern" .
  ✓ check disk usage → du -sh *
  ✓ create a directory → mkdir -p new_folder
  ✓ compress a folder → tar -czf archive.tar.gz folder/

Network requests: 0
Backend: embedded (offline mode)
Status: ✓ All commands generated offline
```

---

## Success Criteria

All scenarios must:
- ✅ Pass without panics or unhandled errors
- ✅ Complete within performance targets (NFRs)
- ✅ Produce user-friendly output
- ✅ Log appropriate debug information
- ✅ Handle errors gracefully with actionable messages

**Acceptance**: 12/12 scenarios passing = Feature ready for release

---

**Status**: ✅ **SCENARIOS UPDATED** - Includes embedded model integration (Scenarios 1, 10-12 updated/added)
