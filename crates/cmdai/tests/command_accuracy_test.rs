// Comprehensive accuracy test suite for enhanced command generation
// Tests the specific scenarios mentioned by the user and ensures correct responses

use cmdai::{
    backends::CommandGenerator,
    models::{CommandRequest, ShellType},
    cli::{CliApp, CliConfig, OutputFormat},
    models::SafetyLevel,
};

#[tokio::test]
async fn test_pdf_file_operations() {
    let cli_app = create_test_cli().await;
    
    // Test case 1: PDF files less than 5MB
    let result = test_command(&cli_app, "list all pdf files size less than 5mb").await;
    assert_eq!(result.generated_command, r#"find . -type f -iname "*.pdf" -size -5M"#);
    println!("✅ PDF < 5MB: {}", result.generated_command);
    
    // Test case 2: PDF files greater than 10MB
    let result = test_command(&cli_app, "find pdf files greater than 10mb").await;
    assert_eq!(result.generated_command, r#"find . -type f -iname "*.pdf" -size +10M"#);
    println!("✅ PDF > 10MB: {}", result.generated_command);
    
    // Test case 3: All PDF files
    let result = test_command(&cli_app, "list all pdf files").await;
    assert_eq!(result.generated_command, r#"find . -type f -iname "*.pdf""#);
    println!("✅ All PDFs: {}", result.generated_command);
}

#[tokio::test]
async fn test_image_file_operations() {
    let cli_app = create_test_cli().await;
    
    // Test case 1: Image files greater than 10MB (as mentioned by user)
    let result = test_command(&cli_app, "find img files greater than 10mb").await;
    let expected = r#"find . -type f \( -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" -o -iname "*.gif" -o -iname "*.bmp" -o -iname "*.tiff" \) -size +10M"#;
    assert_eq!(result.generated_command, expected);
    println!("✅ Images > 10MB: {}", result.generated_command);
    
    // Test case 2: All image files
    let result = test_command(&cli_app, "list all image files").await;
    let expected = r#"find . -type f \( -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" -o -iname "*.gif" -o -iname "*.bmp" -o -iname "*.tiff" \)"#;
    assert_eq!(result.generated_command, expected);
    println!("✅ All images: {}", result.generated_command);
    
    // Test case 3: Small image files
    let result = test_command(&cli_app, "find image files less than 1mb").await;
    let expected = r#"find . -type f \( -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" -o -iname "*.gif" -o -iname "*.bmp" -o -iname "*.tiff" \) -size -1M"#;
    assert_eq!(result.generated_command, expected);
    println!("✅ Images < 1MB: {}", result.generated_command);
}

#[tokio::test]
async fn test_basic_file_operations() {
    let cli_app = create_test_cli().await;
    
    // Test case 1: List all files (should be ls -la for detailed listing)
    let result = test_command(&cli_app, "list all files").await;
    assert_eq!(result.generated_command, "ls -la");
    println!("✅ List all files: {}", result.generated_command);
    
    // Test case 2: List files (basic listing)
    let result = test_command(&cli_app, "list files").await;
    assert_eq!(result.generated_command, "ls -l");
    println!("✅ List files: {}", result.generated_command);
    
    // Test case 3: Show directory contents
    let result = test_command(&cli_app, "display directory contents").await;
    assert_eq!(result.generated_command, "ls -la");
    println!("✅ Directory contents: {}", result.generated_command);
}

#[tokio::test]
async fn test_directory_navigation() {
    let cli_app = create_test_cli().await;
    
    // Test case 1: Current directory
    let result = test_command(&cli_app, "show current directory").await;
    assert_eq!(result.generated_command, "pwd");
    println!("✅ Current directory: {}", result.generated_command);
    
    // Test case 2: Where am I
    let result = test_command(&cli_app, "where am i").await;
    assert_eq!(result.generated_command, "pwd");
    println!("✅ Where am I: {}", result.generated_command);
    
    // Test case 3: Current location
    let result = test_command(&cli_app, "current location").await;
    assert_eq!(result.generated_command, "pwd");
    println!("✅ Current location: {}", result.generated_command);
}

#[tokio::test]
async fn test_file_type_operations() {
    let cli_app = create_test_cli().await;
    
    // Test case 1: Text files
    let result = test_command(&cli_app, "find all text files").await;
    assert_eq!(result.generated_command, r#"find . -type f -iname "*.txt""#);
    println!("✅ Text files: {}", result.generated_command);
    
    // Test case 2: Document files
    let result = test_command(&cli_app, "list document files").await;
    let expected = r#"find . -type f \( -iname "*.doc" -o -iname "*.docx" -o -iname "*.pdf" -o -iname "*.txt" -o -iname "*.rtf" -o -iname "*.odt" \)"#;
    assert_eq!(result.generated_command, expected);
    println!("✅ Document files: {}", result.generated_command);
    
    // Test case 3: Video files larger than 100MB
    let result = test_command(&cli_app, "find video files greater than 100mb").await;
    let expected = r#"find . -type f \( -iname "*.mp4" -o -iname "*.avi" -o -iname "*.mkv" -o -iname "*.mov" -o -iname "*.wmv" -o -iname "*.flv" \) -size +100M"#;
    assert_eq!(result.generated_command, expected);
    println!("✅ Large videos: {}", result.generated_command);
}

#[tokio::test]
async fn test_system_information() {
    let cli_app = create_test_cli().await;
    
    // Test case 1: Current time
    let result = test_command(&cli_app, "what time is it").await;
    assert_eq!(result.generated_command, "date");
    println!("✅ Current time: {}", result.generated_command);
    
    // Test case 2: Current user
    let result = test_command(&cli_app, "show current user").await;
    assert_eq!(result.generated_command, "whoami");
    println!("✅ Current user: {}", result.generated_command);
    
    // Test case 3: System info
    let result = test_command(&cli_app, "show system info").await;
    assert_eq!(result.generated_command, "uname -a");
    println!("✅ System info: {}", result.generated_command);
}

#[tokio::test]
async fn test_dangerous_operations() {
    let cli_app = create_test_cli().await;
    
    // Test case 1: Delete system files (should be blocked but command generated for safety test)
    let result = test_command(&cli_app, "delete system files").await;
    assert_eq!(result.generated_command, "rm -rf /");
    assert!(result.requires_confirmation);
    println!("✅ Dangerous delete (blocked): {}", result.generated_command);
    
    // Test case 2: Remove temporary files
    let result = test_command(&cli_app, "remove temporary files").await;
    assert_eq!(result.generated_command, "rm -rf /tmp/*");
    println!("✅ Remove temp files: {}", result.generated_command);
}

#[tokio::test]
async fn test_shell_specific_commands() {
    // Test the same command across different shells
    let shells = vec![
        (ShellType::Bash, "ls -la"),
        (ShellType::PowerShell, "Get-ChildItem -Force"),
        (ShellType::Cmd, "dir /A"),
    ];
    
    for (shell, expected) in shells {
        let config = CliConfig {
            default_shell: shell,
            safety_level: SafetyLevel::Moderate,
            output_format: OutputFormat::Plain,
            auto_confirm: false,
        };
        
        let cli_app = CliApp::with_config(config).await.expect("Failed to create CLI app");
        let result = test_command(&cli_app, "list all files").await;
        
        assert_eq!(result.generated_command, expected);
        println!("✅ {:?}: {}", shell, result.generated_command);
    }
}

#[tokio::test]
async fn test_complex_file_searches() {
    let cli_app = create_test_cli().await;
    
    // Test case 1: Specific file extension
    let result = test_command(&cli_app, "find .js files").await;
    assert_eq!(result.generated_command, r#"find . -type f -iname "*.js""#);
    println!("✅ JS files: {}", result.generated_command);
    
    // Test case 2: Archive operations
    let result = test_command(&cli_app, "compress files").await;
    assert_eq!(result.generated_command, "tar -czf archive.tar.gz .");
    println!("✅ Compress: {}", result.generated_command);
    
    // Test case 3: Memory information
    let result = test_command(&cli_app, "show memory usage").await;
    assert_eq!(result.generated_command, "free -h");
    println!("✅ Memory usage: {}", result.generated_command);
}

#[tokio::test]
async fn test_fallback_behavior() {
    let cli_app = create_test_cli().await;
    
    // Test unrecognized command
    let result = test_command(&cli_app, "do something completely unknown").await;
    assert_eq!(result.generated_command, "echo 'Could not understand command. Please be more specific.'");
    println!("✅ Fallback: {}", result.generated_command);
}

// Helper functions
async fn create_test_cli() -> CliApp {
    let config = CliConfig {
        default_shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        output_format: OutputFormat::Plain,
        auto_confirm: false,
    };
    
    CliApp::with_config(config).await.expect("Failed to create CLI app")
}

async fn test_command(cli_app: &CliApp, command: &str) -> TestResult {
    let args = MockArgs {
        prompt: Some(command.to_string()),
        shell: None,
        safety: None,
        output: None,
        confirm: false,
        verbose: false,
        config_file: None,
    };
    
    let result = cli_app.run_with_args(args).await.expect("Command failed");
    
    TestResult {
        generated_command: result.generated_command,
        requires_confirmation: result.requires_confirmation,
        backend_used: result.generation_details,
    }
}

#[derive(Debug)]
struct TestResult {
    generated_command: String,
    requires_confirmation: bool,
    backend_used: String,
}

#[derive(Debug, Clone)]
struct MockArgs {
    pub prompt: Option<String>,
    pub shell: Option<String>,
    pub safety: Option<String>,
    pub output: Option<String>,
    pub confirm: bool,
    pub verbose: bool,
    pub config_file: Option<String>,
}

impl cmdai::cli::IntoCliArgs for MockArgs {
    fn prompt(&self) -> Option<String> { self.prompt.clone() }
    fn shell(&self) -> Option<String> { self.shell.clone() }
    fn safety(&self) -> Option<String> { self.safety.clone() }
    fn output(&self) -> Option<String> { self.output.clone() }
    fn confirm(&self) -> bool { self.confirm }
    fn verbose(&self) -> bool { self.verbose }
    fn config_file(&self) -> Option<String> { self.config_file.clone() }
}