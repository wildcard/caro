use cmdai::cli::{CliApp, IntoCliArgs};
use tokio_test;

/// Test command generation accuracy for basic operations as documented in QA-001
/// 
/// These tests are designed to FAIL initially to demonstrate the current accuracy issue
/// where basic commands like "list all files" generate complex find commands instead
/// of simple ls commands.
/// 
/// Reference: qa/000-test-case-command-generation-accuracy.md

#[derive(Clone)]
struct TestCliArgs {
    prompt: Option<String>,
    shell: Option<String>,
    safety: Option<String>,
    output: Option<String>,
    confirm: bool,
    verbose: bool,
    config_file: Option<String>,
}

impl TestCliArgs {
    fn new(prompt: &str) -> Self {
        Self {
            prompt: Some(prompt.to_string()),
            shell: Some("bash".to_string()),
            safety: Some("moderate".to_string()),
            output: Some("plain".to_string()),
            confirm: true, // Auto-confirm to avoid interactive prompts in tests
            verbose: false,
            config_file: None,
        }
    }
}

impl IntoCliArgs for TestCliArgs {
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

/// Test case 1 from QA-001: Basic file listing
/// 
/// EXPECTED TO FAIL: Currently generates `find . -name '*.txt'`
/// SHOULD GENERATE: `ls -la` or `ls -l`
#[tokio::test]
async fn test_basic_file_listing_accuracy() {
    let args = TestCliArgs::new("list all files");
    let app = CliApp::new().await.expect("Failed to create CLI app");
    
    let result = app.run_with_args(args).await.expect("Command generation failed");
    
    // Check that we get a simple ls command, not a complex find command
    let command = result.generated_command.trim();
    
    // Expected simple commands for "list all files"
    let expected_commands = ["ls -la", "ls -l", "ls -al", "ls"];
    let is_simple_ls = expected_commands.iter().any(|&expected| command == expected);
    
    // Should NOT be a find command
    let is_complex_find = command.starts_with("find") && command.contains("*.txt");
    
    assert!(
        is_simple_ls,
        "Expected simple ls command for 'list all files', but got: {}",
        command
    );
    
    assert!(
        !is_complex_find,
        "Got complex find command instead of simple ls: {}",
        command
    );
}

/// Test case 2 from QA-001: Current directory display
/// 
/// EXPECTED TO PASS: Should generate `pwd`
#[tokio::test]
async fn test_current_directory_accuracy() {
    let args = TestCliArgs::new("show current directory");
    let app = CliApp::new().await.expect("Failed to create CLI app");
    
    let result = app.run_with_args(args).await.expect("Command generation failed");
    
    let command = result.generated_command.trim();
    
    assert_eq!(
        command, "pwd",
        "Expected 'pwd' for 'show current directory', but got: {}",
        command
    );
}

/// Test comprehensive basic command mappings that should be simple
#[tokio::test]
async fn test_comprehensive_basic_command_accuracy() {
    let test_cases = vec![
        // (prompt, expected_commands)
        ("list files", vec!["ls", "ls -l", "ls -la", "ls -al"]),
        ("show files", vec!["ls", "ls -l", "ls -la", "ls -al"]),
        ("list all files", vec!["ls -la", "ls -al", "ls -l", "ls"]),
        ("show hidden files", vec!["ls -la", "ls -al", "ls -a"]),
        ("list directory contents", vec!["ls", "ls -l", "ls -la"]),
        ("what files are here", vec!["ls", "ls -l", "ls -la"]),
        ("show current folder", vec!["pwd"]),
        ("where am I", vec!["pwd"]),
        ("current directory", vec!["pwd"]),
        ("what directory am I in", vec!["pwd"]),
        ("show date", vec!["date"]),
        ("current time", vec!["date"]),
        ("what time is it", vec!["date"]),
    ];
    
    for (prompt, expected_commands) in test_cases {
        let args = TestCliArgs::new(prompt);
        let app = CliApp::new().await.expect("Failed to create CLI app");
        
        let result = app.run_with_args(args).await.expect(&format!("Command generation failed for: {}", prompt));
        
        let command = result.generated_command.trim();
        
        let is_expected = expected_commands.iter().any(|&expected| command == expected);
        
        assert!(
            is_expected,
            "For prompt '{}', expected one of {:?}, but got: {}",
            prompt, expected_commands, command
        );
        
        // Additional check: should not be overly complex
        assert!(
            command.split_whitespace().count() <= 3,
            "Command too complex for basic prompt '{}': {}",
            prompt, command
        );
    }
}

/// Test that we avoid complex commands for simple intents
#[tokio::test]
async fn test_avoid_complex_commands_for_simple_intents() {
    let test_cases = vec![
        ("list files", "find"), // Should not use find for simple listing
        ("show files", "find"), // Should not use find
        ("list all files", "grep"), // Should not use grep
        ("show directory", "tree"), // Should not use tree for pwd
    ];
    
    for (prompt, forbidden_command) in test_cases {
        let args = TestCliArgs::new(prompt);
        let app = CliApp::new().await.expect("Failed to create CLI app");
        
        let result = app.run_with_args(args).await.expect(&format!("Command generation failed for: {}", prompt));
        
        let command = result.generated_command.trim();
        
        assert!(
            !command.starts_with(forbidden_command),
            "For simple prompt '{}', should not use complex command '{}', but got: {}",
            prompt, forbidden_command, command
        );
    }
}

/// Performance test: Ensure generation time meets requirements
#[tokio::test]
async fn test_generation_performance_maintained() {
    use std::time::Instant;
    
    let args = TestCliArgs::new("list all files");
    let app = CliApp::new().await.expect("Failed to create CLI app");
    
    let start = Instant::now();
    let _result = app.run_with_args(args).await.expect("Command generation failed");
    let duration = start.elapsed();
    
    // From QA test case: generation time should be < 2000ms, target ~10ms
    assert!(
        duration.as_millis() < 2000,
        "Generation time {} ms exceeds 2000ms requirement",
        duration.as_millis()
    );
}

/// Property-based test: Simple prompts should generate simple commands
#[tokio::test]
async fn test_simple_prompts_generate_simple_commands() {
    let simple_prompts = vec![
        "list files",
        "show files", 
        "current directory",
        "show date",
        "display time",
        "show user",
        "current user",
    ];
    
    for prompt in simple_prompts {
        let args = TestCliArgs::new(prompt);
        let app = CliApp::new().await.expect("Failed to create CLI app");
        
        let result = app.run_with_args(args).await.expect(&format!("Command generation failed for: {}", prompt));
        
        let command = result.generated_command.trim();
        
        // Simple commands should:
        // 1. Be short (< 4 words typically)
        // 2. Use standard Unix utilities
        // 3. Not have complex pipes or redirections
        
        let word_count = command.split_whitespace().count();
        assert!(
            word_count <= 4,
            "Simple prompt '{}' generated complex command with {} words: {}",
            prompt, word_count, command
        );
        
        // Should not contain complex operators
        let complex_operators = ["|", ">>", "<<", "&&", "||", ";"];
        let has_complex_ops = complex_operators.iter().any(|&op| command.contains(op));
        assert!(
            !has_complex_ops,
            "Simple prompt '{}' generated command with complex operators: {}",
            prompt, command
        );
    }
}

/// Integration test: End-to-end command generation accuracy
#[tokio::test]
async fn test_end_to_end_command_accuracy() {
    // Test the exact scenario from QA-001
    let args = TestCliArgs::new("list all files");
    let app = CliApp::new().await.expect("Failed to create CLI app");
    
    let result = app.run_with_args(args).await.expect("Command generation failed");
    
    // Verify all aspects mentioned in QA test case
    assert!(result.generated_command.trim().len() > 0, "No command generated");
    assert!(!result.generated_command.contains("*.txt"), "Should not filter for .txt files specifically");
    assert!(result.generated_command.starts_with("ls"), "Should use ls command for file listing");
    
    // Check performance metrics from QA test case
    assert!(!result.blocked_reason.is_some(), "Command should not be blocked");
    assert_eq!(result.warnings.len(), 0, "Should not have warnings for basic ls command");
}