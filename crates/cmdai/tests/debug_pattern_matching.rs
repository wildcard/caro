// Debug test to see which patterns are matching
use cmdai::{
    cli::{CliApp, CliConfig, OutputFormat},
    models::SafetyLevel,
    models::ShellType,
};

#[tokio::test]
async fn debug_pattern_matching() {
    let config = CliConfig {
        default_shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        output_format: OutputFormat::Plain,
        auto_confirm: false,
    };
    
    let cli_app = CliApp::with_config(config).await.expect("Failed to create CLI app");
    
    let test_commands = vec![
        "list all pdf files size less than 5mb",
        "find pdf files greater than 10mb",
        "list all files",
        "find img files greater than 10mb",
    ];
    
    for command in test_commands {
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
        
        println!("Input: '{}'", command);
        println!("  Generated: '{}'", result.generated_command);
        println!("  Backend: {}", result.generation_details);
        println!("  Contains 'pdf': {}", command.to_lowercase().contains("pdf"));
        println!("  Contains 'files': {}", command.to_lowercase().contains("files"));
        println!("  Contains 'greater than': {}", command.to_lowercase().contains("greater than"));
        println!("  Contains '10mb': {}", command.to_lowercase().contains("10mb"));
        println!("---");
    }
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