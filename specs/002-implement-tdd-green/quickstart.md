# Quickstart Guide: Command Validation Workflow

**Feature**: 002-implement-tdd-green | **Date**: 2025-10-01 | **Phase**: 1

This guide demonstrates the complete command generation and validation workflow using caro's core APIs.

## Overview

The quickstart workflow illustrates the complete lifecycle of a command generation request:
1. Create a CommandRequest from natural language
2. Generate a command using a backend
3. Validate the command for safety issues
4. Display results to the user

This example uses the MockBackend for demonstration, but the workflow is identical for all backend implementations (Ollama, vLLM, MLX).

## Prerequisites

**Dependencies**:
```toml
[dependencies]
caro = "0.1.0"
tokio = { version = "1.35", features = ["full"] }
```

**Imports**:
```rust
use caro::models::{CommandRequest, ShellType, SafetyLevel, RiskLevel};
use caro::backends::MockBackend;
use caro::safety::SafetyValidator;
```

## Complete Workflow Example

### Step 1: Create a Command Request

Convert user's natural language input into a structured request:

```rust
// User input: "list all files"
let request = CommandRequest::new("list all files", ShellType::Bash);

// Output:
// CommandRequest {
//     input: "list all files",
//     shell_type: ShellType::Bash,
//     safety_level: SafetyLevel::Moderate,  // default
//     context: None,
// }
```

**With custom safety level**:
```rust
let request = CommandRequest::new("list all files", ShellType::Bash)
    .with_safety(SafetyLevel::Strict);
```

**With execution context**:
```rust
let request = CommandRequest::new("list all files", ShellType::Bash)
    .with_context("/home/user/projects");
```

### Step 2: Generate Command with Backend

Use an async backend to generate the command:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize backend
    let backend = MockBackend::new("quickstart-demo");

    // Check availability
    if !backend.is_available().await {
        eprintln!("Backend not available");
        return Ok(());
    }

    // Generate command
    let command = backend.generate_command(&request).await?;

    // Output:
    // GeneratedCommand {
    //     command: "ls -la",
    //     explanation: "Lists all files in long format including hidden files",
    //     safety_level: RiskLevel::Safe,
    //     estimated_impact: "Read-only operation, no system changes",
    //     alternatives: vec!["find . -type f", "tree -a"],
    //     backend_used: "mock-backend-v1",
    //     generation_time_ms: 142,
    //     confidence_score: 0.95,
    // }

    Ok(())
}
```

### Step 3: Validate Command Safety

Validate the generated command for dangerous patterns:

```rust
// Initialize safety validator
let validator = SafetyValidator::new();

// Validate the generated command
let validation_result = validator.validate(&command.command, ShellType::Bash);

// Check risk level
match validation_result.risk_level {
    RiskLevel::Safe => {
        println!("✓ Command is safe to execute");
    }
    RiskLevel::Moderate => {
        println!("⚠ Command requires caution");
        println!("  Reason: {}", validation_result.explanation);
    }
    RiskLevel::High => {
        println!("⚠ High-risk command detected");
        println!("  Reason: {}", validation_result.explanation);
        // Request user confirmation
    }
    RiskLevel::Critical => {
        println!("✗ CRITICAL: Dangerous command blocked");
        println!("  Reason: {}", validation_result.explanation);
        // Block execution
    }
}
```

### Step 4: Display Results to User

Format and display the generated command:

```rust
// Display generated command
println!("\nGenerated Command:");
println!("  {}", command.command);
println!("\nExplanation:");
println!("  {}", command.explanation);
println!("\nSafety Assessment: {}", command.safety_level);
println!("Impact: {}", command.estimated_impact);
println!("Confidence: {:.0}%", command.confidence_score * 100.0);

if !command.alternatives.is_empty() {
    println!("\nAlternatives:");
    for (i, alt) in command.alternatives.iter().enumerate() {
        println!("  {}. {}", i + 1, alt);
    }
}

println!("\nBackend: {} ({}ms)", command.backend_used, command.generation_time_ms);
```

**Expected Output**:
```
Generated Command:
  ls -la

Explanation:
  Lists all files in long format including hidden files

Safety Assessment: SAFE
Impact: Read-only operation, no system changes
Confidence: 95%

Alternatives:
  1. find . -type f
  2. tree -a

Backend: mock-backend-v1 (142ms)
```

## Complete Integration Test

This is the canonical quickstart test from `tests/integration_tests.rs`:

```rust
#[tokio::test]
async fn quickstart_command_generation() {
    // Step 1: Create request from user input
    let request = CommandRequest::new("list all files", ShellType::Bash);

    // Step 2: Generate command with backend
    let backend = MockBackend::new("quickstart");
    let command = backend.generate_command(&request).await.unwrap();

    // Step 3: Validate safety
    let validator = SafetyValidator::new();
    let result = validator.validate(&command.command, ShellType::Bash);

    // Step 4: Assert safety and verify output
    assert_eq!(result.risk_level, RiskLevel::Safe);
    assert!(!command.command.is_empty());
    assert!(command.confidence_score >= 0.0 && command.confidence_score <= 1.0);

    // Display for demonstration
    println!("Generated: {}", command.command);
    println!("Safety: {}", result.risk_level);
}
```

## Workflow Variations

### Example 1: Dangerous Command Detection

```rust
#[tokio::test]
async fn dangerous_command_validation() {
    // Request a dangerous operation
    let request = CommandRequest::new("delete everything", ShellType::Bash);

    // Generate (mock will create a dangerous command for demo)
    let backend = MockBackend::new("dangerous");
    let command = backend.generate_command(&request).await.unwrap();

    // Validate - should detect danger
    let validator = SafetyValidator::new();
    let result = validator.validate(&command.command, ShellType::Bash);

    // Assert critical risk detected
    assert_eq!(result.risk_level, RiskLevel::Critical);
    assert!(result.explanation.contains("dangerous"));

    println!("⚠ Blocked: {}", result.explanation);
}
```

### Example 2: Cross-Platform Validation

```rust
#[tokio::test]
async fn windows_command_generation() {
    // Windows PowerShell request
    let request = CommandRequest::new("list all files", ShellType::PowerShell);

    let backend = MockBackend::new("windows");
    let command = backend.generate_command(&request).await.unwrap();

    // Expected: PowerShell syntax (Get-ChildItem)
    assert!(command.command.contains("Get-ChildItem") || command.command.contains("ls"));

    // Validate with Windows validator
    let validator = SafetyValidator::new();
    let result = validator.validate(&command.command, ShellType::PowerShell);

    assert_eq!(result.risk_level, RiskLevel::Safe);
}
```

### Example 3: Safety Level Enforcement

```rust
#[tokio::test]
async fn strict_safety_enforcement() {
    // Strict safety mode
    let request = CommandRequest::new("move files", ShellType::Bash)
        .with_safety(SafetyLevel::Strict);

    let backend = MockBackend::new("strict");
    let command = backend.generate_command(&request).await.unwrap();

    // Command might be "mv source dest" (moderate risk)
    let validator = SafetyValidator::new();
    let result = validator.validate(&command.command, ShellType::Bash);

    // Strict mode requires confirmation for Moderate+ risk
    let requires_confirmation = result.risk_level.requires_confirmation(request.safety_level);

    if requires_confirmation {
        println!("User confirmation required for: {}", command.command);
    }
}
```

### Example 4: Backend Metadata

```rust
#[tokio::test]
async fn backend_information() {
    let backend = MockBackend::new("info");

    // Get backend metadata
    let info = backend.backend_info();

    println!("Backend: {}", info.model_name);
    println!("Type: {:?}", info.backend_type);
    println!("Streaming: {}", info.supports_streaming);
    println!("Max tokens: {}", info.max_tokens);
    println!("Typical latency: {}ms", info.typical_latency_ms);

    // Use metadata for backend selection
    assert_eq!(info.backend_type, BackendType::Mock);
    assert!(info.typical_latency_ms < 200); // Fast mock backend
}
```

## Error Handling

### Backend Errors

```rust
#[tokio::test]
async fn handle_backend_error() {
    let backend = MockBackend::new("error-test");

    // Empty request should fail
    let request = CommandRequest::new("", ShellType::Bash);
    let result = backend.generate_command(&request).await;

    match result {
        Ok(_) => panic!("Expected error for empty input"),
        Err(e) => {
            println!("Error handled: {}", e);
            assert!(e.to_string().contains("empty"));
        }
    }
}
```

### Validation Errors

```rust
#[tokio::test]
async fn handle_validation_error() {
    let validator = SafetyValidator::new();

    // Invalid command syntax
    let result = validator.validate("rm -rf /", ShellType::Bash);

    // Critical risk detected
    assert_eq!(result.risk_level, RiskLevel::Critical);
    assert!(result.explanation.contains("root filesystem"));

    println!("Validation blocked: {}", result.explanation);
}
```

## CLI Integration

### Command-Line Usage

Once implemented, the CLI workflow will match this pattern:

```bash
# Basic command generation
$ caro generate "list all files"
Generated: ls -la
Safety: SAFE
Explanation: Lists all files in long format

# With output format
$ caro generate "list all files" --format json
{
  "command": "ls -la",
  "safety_level": "safe",
  "confidence_score": 0.95
}

# Validate existing command
$ caro validate "rm -rf /"
Safety: CRITICAL
⚠ BLOCKED: Recursive deletion of root filesystem

# With custom safety level
$ caro generate "move files" --safety strict
Generated: mv source dest
Safety: MODERATE
⚠ Confirmation required
Execute? [y/N]:
```

### CLI Implementation Sketch

```rust
// src/cli/mod.rs
pub async fn run(args: CliApp) -> Result<()> {
    match args.command {
        Commands::Generate { input, shell, safety } => {
            // Step 1: Create request
            let request = CommandRequest::new(input, shell.unwrap_or_default())
                .with_safety(safety.unwrap_or_default());

            // Step 2: Generate
            let backend = MockBackend::new("cli");
            let command = backend.generate_command(&request).await?;

            // Step 3: Validate
            let validator = SafetyValidator::new();
            let result = validator.validate(&command.command, request.shell_type);

            // Step 4: Display
            display_command(&command, &result, args.format);
        }
        Commands::Validate { command } => {
            let validator = SafetyValidator::new();
            let result = validator.validate(&command, ShellType::detect());
            display_validation(&result, args.format);
        }
    }
    Ok(())
}
```

## Performance Expectations

Based on the feature specification targets:

| Operation | Target | MockBackend |
|-----------|--------|-------------|
| Startup time | <100ms | ~10ms |
| Safety validation | <50ms | ~5ms |
| Command generation | <2s | ~150ms |
| Total workflow | <2.2s | ~165ms |

## Testing the Workflow

### Run Integration Test

```bash
# Run quickstart test
cargo test --test integration_tests quickstart_command_generation -- --nocapture

# Run all integration tests
cargo test --test integration_tests

# Run with timing
cargo test --test integration_tests -- --nocapture --test-threads=1
```

### Manual Testing

```bash
# Build the project
cargo build --release

# Test basic generation (once CLI implemented)
./target/release/caro generate "list all files"

# Test validation
./target/release/caro validate "ls -la"

# Test dangerous command
./target/release/caro validate "rm -rf /"
```

## Next Steps

1. Implement the core models (`src/models/`)
2. Implement safety validator (`src/safety/`)
3. Implement backend trait and MockBackend (`src/backends/`)
4. Implement CLI interface (`src/cli/`)
5. Run this quickstart workflow as integration test
6. Verify all 80 contract tests pass

## Summary

The quickstart workflow demonstrates:
- Simple API for command generation
- Clear safety validation feedback
- Extensible backend system
- Type-safe error handling
- Cross-platform support
- User-friendly output formatting

This workflow forms the foundation for all caro interactions and validates the core design decisions from the research phase.

---
*Quickstart guide complete - Ready for implementation*
