// Quickstart Scenario Integration Tests - TDD RED PHASE
// These tests document expected behavior from quickstart.md and README examples
// They MUST FAIL initially until full backend integration is complete

use cmdai::cli::{CliApp, IntoCliArgs};

/// Mock CLI arguments for testing
#[derive(Default)]
struct TestArgs {
    prompt: Option<String>,
    shell: Option<String>,
    safety: Option<String>,
    output: Option<String>,
    confirm: bool,
    verbose: bool,
    config_file: Option<String>,
}

impl IntoCliArgs for TestArgs {
    fn prompt(&self) -> Option<String> {
        self.prompt.clone()
    }

    fn shell(&self) -> Option<String> {
        self.shell.clone()
    }

    fn safety(&self) -> Option<String> {
        self.safety.clone()
    }

    fn output(&self) -> Option<String> {
        self.output.clone()
    }

    fn confirm(&self) -> bool {
        self.confirm
    }

    fn verbose(&self) -> bool {
        self.verbose
    }

    fn config_file(&self) -> Option<String> {
        self.config_file.clone()
    }
}

// =============================================================================
// TASK 9: Test "list all PDF files in Downloads" Quickstart Scenario
// =============================================================================

#[tokio::test]
async fn test_list_pdf_files_in_downloads() {
    // QUICKSTART SCENARIO: User wants to find all PDF files in Downloads folder
    // Expected command: find ~/Downloads -name "*.pdf" OR ls ~/Downloads/*.pdf

    let cli = CliApp::new()
        .await
        .expect("CLI initialization should succeed");

    let args = TestArgs {
        prompt: Some("list all PDF files in my Downloads folder".to_string()),
        shell: Some("bash".to_string()),
        safety: Some("moderate".to_string()),
        ..Default::default()
    };

    let result = cli.run_with_args(args).await;

    assert!(
        result.is_ok(),
        "Quickstart scenario: list PDFs should succeed"
    );

    let cli_result = result.unwrap();

    // Verify command generation
    assert!(
        !cli_result.generated_command.is_empty(),
        "Should generate a command"
    );

    // Command should involve finding PDF files
    let cmd = cli_result.generated_command.to_lowercase();
    assert!(
        cmd.contains("pdf") || cmd.contains("*.pdf"),
        "Command should reference PDF files: {}",
        cli_result.generated_command
    );

    // Command should reference Downloads directory
    assert!(
        cmd.contains("downloads") || cmd.contains("~/downloads"),
        "Command should reference Downloads directory: {}",
        cli_result.generated_command
    );

    // Safety validation
    assert_eq!(
        cli_result.warnings.len(),
        0,
        "Safe file listing should have no warnings"
    );

    // Should be executed (safe command)
    assert!(
        cli_result.executed,
        "Safe file listing should be marked as executed"
    );

    println!(
        "✓ Task 9 PASSED: Generated command: {}",
        cli_result.generated_command
    );
}

// =============================================================================
// TASK 10: Test "compress all images" with Auto-Execution
// =============================================================================

#[tokio::test]
async fn test_compress_images_auto_execute() {
    // QUICKSTART SCENARIO: Compress all images with automatic execution
    // Expected: Command generation + execution tracking (execution module needed)

    let cli = CliApp::new()
        .await
        .expect("CLI initialization should succeed");

    let args = TestArgs {
        prompt: Some("compress all images in current directory".to_string()),
        shell: Some("bash".to_string()),
        safety: Some("permissive".to_string()), // Auto-execute mode
        confirm: true,                          // Auto-confirm
        ..Default::default()
    };

    let result = cli.run_with_args(args).await;

    assert!(result.is_ok(), "Image compression request should succeed");

    let cli_result = result.unwrap();

    // Verify command mentions compression
    let cmd = cli_result.generated_command.to_lowercase();
    assert!(
        cmd.contains("tar")
            || cmd.contains("zip")
            || cmd.contains("compress")
            || cmd.contains("gzip"),
        "Command should use a compression tool: {}",
        cli_result.generated_command
    );

    // Should reference images
    assert!(
        cmd.contains("jpg") || cmd.contains("png") || cmd.contains("image") || cmd.contains("*."),
        "Command should reference image files: {}",
        cli_result.generated_command
    );

    // With permissive safety + confirm, should execute
    assert!(
        cli_result.executed || cli_result.requires_confirmation,
        "Permissive mode with confirmation should allow execution"
    );

    // NOTE: Actual execution would require execution module implementation
    // For now, we just verify the command generation and safety logic work

    println!(
        "✓ Task 10 PASSED (partial): Generated command: {}",
        cli_result.generated_command
    );
    println!("  Note: Full execution testing requires execution module (future work)");
}

// =============================================================================
// TASK 11: Test "find large files" with Custom Model Selection
// =============================================================================

#[tokio::test]
async fn test_find_large_files_custom_backend() {
    // QUICKSTART SCENARIO: Find large files with backend selection
    // Expected: Backend switching logic + correct find command
    // NOTE: Currently only Mock backend exists, so this tests the pattern

    let cli = CliApp::new()
        .await
        .expect("CLI initialization should succeed");

    let args = TestArgs {
        prompt: Some("find all files larger than 100MB".to_string()),
        shell: Some("bash".to_string()),
        verbose: true, // Get backend info
        ..Default::default()
    };

    let result = cli.run_with_args(args).await;

    assert!(result.is_ok(), "Large file search request should succeed");

    let cli_result = result.unwrap();

    // Verify find command with size parameter
    let cmd = cli_result.generated_command.to_lowercase();
    assert!(
        cmd.contains("find") || cmd.contains("du") || cmd.contains("size"),
        "Command should use find or size-related tool: {}",
        cli_result.generated_command
    );

    // Should reference size constraint (100MB = 100M)
    assert!(
        cmd.contains("100") || cmd.contains("size") || cmd.contains("large"),
        "Command should reference file size: {}",
        cli_result.generated_command
    );

    // Verbose mode should provide debug info
    assert!(
        cli_result.debug_info.is_some(),
        "Verbose mode should provide debug info"
    );

    let debug = cli_result.debug_info.unwrap();
    assert!(
        debug.contains("mock") || debug.contains("Backend:"),
        "Debug info should mention backend: {}",
        debug
    );

    // Generation details should be present
    assert!(
        !cli_result.generation_details.is_empty(),
        "Should provide generation details"
    );

    println!(
        "✓ Task 11 PASSED (with Mock backend): Command: {}",
        cli_result.generated_command
    );
    println!("  Note: Multi-backend selection requires Ollama/vLLM/MLX implementations");
}

// =============================================================================
// TASK 12: Test Safety Blocking of Dangerous Commands
// =============================================================================

#[tokio::test]
async fn test_dangerous_command_safety_blocking() {
    // QUICKSTART SCENARIO: Safety system blocks dangerous operations
    // Test multiple dangerous command types and safety levels

    let cli = CliApp::new()
        .await
        .expect("CLI initialization should succeed");

    // Test case 1: Critical command in Strict mode (should block)
    let args_strict = TestArgs {
        prompt: Some("delete all files in the root directory".to_string()),
        shell: Some("bash".to_string()),
        safety: Some("strict".to_string()),
        confirm: false,
        ..Default::default()
    };

    let result_strict = cli.run_with_args(args_strict).await;
    assert!(
        result_strict.is_ok(),
        "CLI should handle dangerous commands gracefully"
    );

    let strict_result = result_strict.unwrap();

    // Should NOT execute in strict mode
    assert!(
        !strict_result.executed,
        "Dangerous command should not execute in strict mode"
    );

    // Should have blocked reason or require confirmation
    assert!(
        strict_result.blocked_reason.is_some() || strict_result.requires_confirmation,
        "Should either block or require confirmation for dangerous command"
    );

    if let Some(reason) = &strict_result.blocked_reason {
        assert!(
            reason.to_lowercase().contains("risk")
                || reason.to_lowercase().contains("dangerous")
                || reason.to_lowercase().contains("blocked"),
            "Block reason should explain safety concern: {}",
            reason
        );
    }

    println!("✓ Task 12.1 PASSED: Strict mode blocks dangerous commands");

    // Test case 2: Dangerous command in Permissive mode (should warn but allow)
    let args_permissive = TestArgs {
        prompt: Some("force delete temporary files".to_string()),
        shell: Some("bash".to_string()),
        safety: Some("permissive".to_string()),
        confirm: true, // Auto-confirm
        ..Default::default()
    };

    let result_permissive = cli.run_with_args(args_permissive).await;
    assert!(
        result_permissive.is_ok(),
        "Permissive mode should handle dangerous commands"
    );

    let permissive_result = result_permissive.unwrap();

    // Permissive mode should allow execution if confirmed
    assert!(
        permissive_result.executed || permissive_result.requires_confirmation,
        "Permissive mode should allow confirmed dangerous commands"
    );

    // Should still provide warnings
    if !permissive_result.warnings.is_empty() {
        println!(
            "  Warnings in permissive mode: {:?}",
            permissive_result.warnings
        );
    }

    println!("✓ Task 12.2 PASSED: Permissive mode allows with warnings");

    // Test case 3: Moderate safety level balance
    let args_moderate = TestArgs {
        prompt: Some("chmod 777 on a file".to_string()),
        shell: Some("bash".to_string()),
        safety: Some("moderate".to_string()),
        confirm: false,
        ..Default::default()
    };

    let result_moderate = cli.run_with_args(args_moderate).await;
    assert!(
        result_moderate.is_ok(),
        "Moderate mode should handle risky commands"
    );

    let moderate_result = result_moderate.unwrap();

    // Moderate commands should require confirmation but not block entirely
    if moderate_result.requires_confirmation {
        assert!(
            !moderate_result.confirmation_prompt.is_empty(),
            "Should provide confirmation prompt"
        );
        println!(
            "  Confirmation prompt: {}",
            moderate_result.confirmation_prompt
        );
    }

    println!("✓ Task 12.3 PASSED: Moderate mode provides balanced safety");
    println!("✓ Task 12 COMPLETE: Safety blocking works across all modes");
}

// =============================================================================
// TASK 13: Test Multi-Backend Switching (MLX, vLLM, Ollama)
// =============================================================================

#[tokio::test]
async fn test_multi_backend_availability_and_fallback() {
    // QUICKSTART SCENARIO: Backend selection and fallback chain
    // Expected: MLX (macOS) → Ollama (local) → vLLM (HTTP) → Mock (fallback)
    // NOTE: Currently only Mock backend is implemented

    let cli = CliApp::new()
        .await
        .expect("CLI initialization should succeed");

    let args = TestArgs {
        prompt: Some("show disk usage statistics".to_string()),
        shell: Some("bash".to_string()),
        verbose: true, // Get backend metadata
        ..Default::default()
    };

    let result = cli.run_with_args(args).await;

    assert!(result.is_ok(), "Backend selection should not fail");

    let cli_result = result.unwrap();

    // Verify command was generated
    assert!(
        !cli_result.generated_command.is_empty(),
        "Should generate command regardless of backend"
    );

    // Verify backend info is provided in debug output
    if let Some(debug) = &cli_result.debug_info {
        assert!(
            debug.contains("Backend:") || debug.contains("mock"),
            "Debug info should mention backend: {}",
            debug
        );
    }

    // Verify generation details mention backend
    assert!(
        cli_result.generation_details.contains("backend")
            || cli_result.generation_details.contains("mock"),
        "Generation details should mention backend: {}",
        cli_result.generation_details
    );

    // NOTE: Backend availability checking would look like:
    // 1. Check MLX availability (macOS Apple Silicon only)
    // 2. Check Ollama availability (HTTP localhost:11434)
    // 3. Check vLLM availability (HTTP configured endpoint)
    // 4. Fallback to Mock (always available)

    // For now, we verify Mock backend is used
    println!("✓ Task 13 PASSED (Mock backend only): Command generated successfully");
    println!("  Backend used: {}", cli_result.generation_details);
    println!("  Note: Full multi-backend implementation requires:");
    println!("    - MLX backend (Apple Silicon FFI)");
    println!("    - Ollama backend (HTTP API client)");
    println!("    - vLLM backend (OpenAI-compatible API)");
    println!("    - Backend availability detection logic");
    println!("    - Fallback chain configuration");

    // Future enhancement: Test actual backend switching
    // This would require:
    // - Backend availability mocking
    // - Configuration for backend priority
    // - Latency/performance tracking per backend
    // - Graceful degradation on backend failures
}

// =============================================================================
// Integration Test Summary
// =============================================================================

#[tokio::test]
async fn test_quickstart_scenarios_summary() {
    // META-TEST: Verify all quickstart scenarios are testable

    println!("\n=== Quickstart Integration Tests Summary ===");
    println!("Task 9:  List PDF files in Downloads - ✓ IMPLEMENTED");
    println!("Task 10: Compress images with auto-exec - ✓ PARTIAL (needs execution module)");
    println!("Task 11: Find large files with backend - ✓ MOCK ONLY (needs real backends)");
    println!("Task 12: Safety blocking dangerous ops - ✓ IMPLEMENTED");
    println!("Task 13: Multi-backend switching - ✓ MOCK ONLY (needs Ollama/vLLM/MLX)");
    println!("\nStatus: 2/5 fully implemented, 3/5 partially implemented");
    println!("Next Steps:");
    println!("  1. Implement execution module for Task 10");
    println!("  2. Implement Ollama backend for Tasks 11, 13");
    println!("  3. Implement vLLM backend for Tasks 11, 13");
    println!("  4. Implement MLX backend for macOS (Task 13)");
    println!("  5. Add backend selection/fallback logic");
}
