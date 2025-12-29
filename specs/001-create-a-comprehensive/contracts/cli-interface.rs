// CLI Interface Contract
// This defines the command-line interface specification

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Main CLI application structure
#[derive(Parser)]
#[command(name = "caro")]
#[command(about = "Convert natural language to shell commands", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Natural language task description
    #[arg(value_name = "DESCRIPTION")]
    pub prompt: Option<String>,
    
    /// Backend to use for inference
    #[arg(long, value_enum, default_value = "auto")]
    pub backend: BackendChoice,
    
    /// Model to use (auto-downloads if not cached)
    #[arg(long)]
    pub model: Option<String>,
    
    /// Custom endpoint for remote backends
    #[arg(long)]
    pub endpoint: Option<String>,
    
    /// Auto-execute without confirmation
    #[arg(long)]
    pub auto: bool,
    
    /// Allow dangerous commands
    #[arg(long)]
    pub allow_dangerous: bool,
    
    /// Safety level override
    #[arg(long, value_enum)]
    pub safety: Option<SafetyLevel>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Log level
    #[arg(long, value_enum)]
    pub log_level: Option<LogLevel>,
    
    /// Custom config file path
    #[arg(long)]
    pub config: Option<String>,
    
    /// Show explanation without executing
    #[arg(long)]
    pub explain: bool,
    
    /// Output format
    #[arg(long, value_enum, default_value = "human")]
    pub format: OutputFormat,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Subcommands for configuration and management
#[derive(Subcommand)]
pub enum Commands {
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Model management
    Models {
        #[command(subcommand)]
        action: ModelAction,
    },
    /// Backend management
    Backends {
        #[command(subcommand)]
        action: BackendAction,
    },
    /// Command history
    History {
        #[command(subcommand)]
        action: HistoryAction,
    },
    /// Benchmark backends
    Benchmark {
        /// Number of test commands to run
        #[arg(short, long, default_value = "10")]
        count: u32,
        
        /// Include specific backend
        #[arg(long)]
        backend: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set configuration value
    Set {
        key: String,
        value: String,
    },
    /// Reset to defaults
    Reset,
    /// Validate configuration
    Validate,
}

#[derive(Subcommand)]
pub enum ModelAction {
    /// List available models
    List,
    /// Download a model
    Download {
        model_id: String,
    },
    /// Remove cached model
    Remove {
        model_id: String,
    },
    /// Show cache usage
    Cache,
    /// Update model metadata
    Update,
}

#[derive(Subcommand)]
pub enum BackendAction {
    /// List available backends
    List,
    /// Test backend connectivity
    Test {
        backend: Option<String>,
    },
    /// Show backend status
    Status,
}

#[derive(Subcommand)]
pub enum HistoryAction {
    /// Show command history
    Show {
        #[arg(short, long, default_value = "10")]
        count: u32,
    },
    /// Clear history
    Clear,
    /// Export history
    Export {
        file: String,
    },
}

/// Backend selection options
#[derive(clap::ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum BackendChoice {
    Auto,
    MLX,
    VLLM,
    Ollama,
}

/// Log level options
#[derive(clap::ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Output format options
#[derive(clap::ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    Human,
    JSON,
    YAML,
}

/// Exit codes for the application
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    BackendUnavailable = 2,
    SafetyViolation = 3,
    ConfigurationError = 4,
    ModelNotFound = 5,
    UserCancelled = 6,
    ValidationFailed = 7,
}

/// CLI interface contract tests
pub trait CliContractTests {
    /// Test argument parsing with valid inputs
    fn test_valid_argument_parsing(&self);
    
    /// Test handling of invalid arguments
    fn test_invalid_argument_handling(&self);
    
    /// Test help text generation
    fn test_help_text_generation(&self);
    
    /// Test version display
    fn test_version_display(&self);
    
    /// Test configuration file loading
    fn test_config_file_loading(&self);
    
    /// Test environment variable handling
    fn test_environment_variables(&self);
    
    /// Test subcommand functionality
    fn test_subcommand_functionality(&self);
    
    /// Test output formatting options
    fn test_output_formatting(&self);
}