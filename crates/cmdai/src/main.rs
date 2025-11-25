use clap::Parser;
use cmdai::cli::{CliApp, CliError, IntoCliArgs};
use cmdai::config::{run_interactive_config, ConfigManager, ConfigurationState, UserConfiguration};
use cmdai::testing::ManualTestRunner;
use std::process;

/// cmdai - Convert natural language to shell commands using local LLMs
#[derive(Parser, Clone)]
#[command(name = "cmdai")]
#[command(about = "Convert natural language to shell commands using local LLMs")]
#[command(
    long_about = "cmdai converts natural language descriptions into safe POSIX shell commands using local language models. Features safety validation, multiple output formats, and configurable backends."
)]
#[command(version)]
struct Cli {
    /// Natural language task description
    #[arg(help = "Natural language description of the task")]
    prompt: Option<String>,

    /// Target shell type
    #[arg(
        short,
        long,
        help = "Shell type (bash, zsh, fish, sh, powershell, cmd)"
    )]
    shell: Option<String>,

    /// Safety level for command validation
    #[arg(long, help = "Safety level (strict, moderate, permissive)")]
    safety: Option<String>,

    /// Output format
    #[arg(short, long, help = "Output format (json, yaml, plain)")]
    output: Option<String>,

    /// Auto-confirm dangerous commands
    #[arg(
        short = 'y',
        long,
        help = "Auto-confirm dangerous commands without prompting"
    )]
    confirm: bool,

    /// Verbose output with debug information
    #[arg(short, long, help = "Enable verbose output with timing and debug info")]
    verbose: bool,

    /// Custom configuration file path
    #[arg(short, long, help = "Path to configuration file")]
    config_file: Option<String>,

    /// Show configuration information
    #[arg(long, help = "Show current configuration and exit")]
    show_config: bool,

    /// Launch interactive configuration UI
    #[arg(long, help = "Launch interactive configuration interface")]
    configure: bool,

    /// Launch interactive manual testing interface
    #[arg(long, help = "Launch interactive manual testing system")]
    test: bool,

    /// Launch interactive mode with slash commands
    #[arg(short, long, help = "Launch interactive mode with slash command support")]
    interactive: bool,
}

impl IntoCliArgs for Cli {
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize tracing/logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_level(true)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("cmdai=info")
            .without_time()
            .init();
    }

    // Handle --show-config
    if cli.show_config {
        match show_configuration(&cli).await {
            Ok(config_info) => {
                println!("{}", config_info);
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error showing configuration: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle --configure
    if cli.configure {
        match run_interactive_configuration(&cli).await {
            Ok(()) => {
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error in configuration: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle --test
    if cli.test {
        match run_interactive_testing().await {
            Ok(()) => {
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error in testing: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle --interactive
    if cli.interactive {
        match run_interactive_mode().await {
            Ok(()) => {
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error in interactive mode: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle missing prompt (but allow interactive mode)
    if cli.prompt.is_none() && !cli.interactive {
        eprintln!("Error: No prompt provided");
        eprintln!();
        eprintln!("Usage: cmdai [OPTIONS] <PROMPT>");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  cmdai \"list all files\"");
        eprintln!("  cmdai --shell zsh \"find large files\"");
        eprintln!("  cmdai --safety strict \"delete temporary files\"");
        eprintln!();
        eprintln!("Interactive modes:");
        eprintln!("  cmdai --interactive        # Launch interactive mode with slash commands");
        eprintln!("  cmdai --configure          # Launch interactive configuration");
        eprintln!("  cmdai --test               # Launch interactive testing system");
        eprintln!();
        eprintln!("Configuration:");
        eprintln!("  cmdai --show-config        # Show current configuration");
        eprintln!();
        eprintln!("Run 'cmdai --help' for more information.");
        process::exit(1);
    }

    // Run the CLI application
    match run_cli(&cli).await {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            match e {
                CliError::NotImplemented => {
                    eprintln!();
                    eprintln!("This functionality is not yet implemented.");
                    eprintln!("cmdai is currently in development.");
                }
                CliError::ConfigurationError { .. } => {
                    eprintln!();
                    eprintln!("Please check your configuration and try again.");
                }
                _ => {}
            }
            process::exit(1);
        }
    }
}

async fn run_cli(cli: &Cli) -> Result<(), CliError> {
    // Create CLI application
    let app = CliApp::new().await?;

    // Run command generation
    let result = app.run_with_args(cli.clone()).await?;

    // Display result
    match result.output_format {
        cmdai::cli::OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&result).map_err(|e| CliError::Internal {
                message: format!("JSON serialization failed: {}", e),
            })?;
            println!("{}", json);
        }
        cmdai::cli::OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&result).map_err(|e| CliError::Internal {
                message: format!("YAML serialization failed: {}", e),
            })?;
            println!("{}", yaml);
        }
        cmdai::cli::OutputFormat::Plain => {
            print_plain_output(&result, cli).await?;
        }
    }

    Ok(())
}

async fn print_plain_output(result: &cmdai::cli::CliResult, cli: &Cli) -> Result<(), CliError> {
    use colored::Colorize;

    // Print warnings first
    for warning in &result.warnings {
        eprintln!("{} {}", "Warning:".yellow().bold(), warning);
    }

    // Handle blocked commands
    if let Some(blocked_reason) = &result.blocked_reason {
        eprintln!("{} {}", "Blocked:".red().bold(), blocked_reason);
        return Ok(());
    }

    // Handle confirmation required
    if result.requires_confirmation && !cli.confirm {
        use dialoguer::Confirm;

        // Check if we're in a terminal environment
        if atty::is(atty::Stream::Stdin) {
            let confirmed = Confirm::new()
                .with_prompt(&result.confirmation_prompt)
                .default(false)
                .interact()
                .map_err(|e| CliError::Internal {
                    message: format!("Failed to get user confirmation: {}", e),
                })?;

            if !confirmed {
                println!("{}", "Operation cancelled by user.".yellow());
                return Ok(());
            }

            println!(
                "{}",
                "✓ Confirmed. Proceeding with command execution.".green()
            );
        } else {
            // Non-interactive environment - show confirmation message and exit
            println!("{}", result.confirmation_prompt.yellow());
            println!("{}", "Use --confirm/-y flag to auto-confirm dangerous commands in non-interactive environments.".dimmed());
            return Ok(());
        }
    }

    // Print the main command
    println!("{}", "Command:".bold());
    println!("  {}", result.generated_command.bright_cyan().bold());
    println!();

    // Print explanation
    if !result.explanation.is_empty() {
        println!("{}", "Explanation:".bold());
        println!("  {}", result.explanation);
        println!();
    }

    // Print alternatives if available
    if !result.alternatives.is_empty() {
        println!("{}", "Alternatives:".bold());
        for alt in &result.alternatives {
            println!("  • {}", alt.dimmed());
        }
        println!();
    }

    // Print debug information if verbose
    if let Some(debug_info) = &result.debug_info {
        println!("{}", "Debug Info:".dimmed());
        println!("  {}", debug_info.dimmed());
    }

    if !result.generation_details.is_empty() {
        println!("  {}", result.generation_details.dimmed());
    }

    Ok(())
}

async fn show_configuration(cli: &Cli) -> Result<String, CliError> {
    let config_manager = if let Some(config_file) = &cli.config_file {
        ConfigManager::with_config_path(config_file.into()).map_err(|e| {
            CliError::ConfigurationError {
                message: format!("Failed to create config manager: {}", e),
            }
        })?
    } else {
        ConfigManager::new().map_err(|e| CliError::ConfigurationError {
            message: format!("Failed to create config manager: {}", e),
        })?
    };

    let config = config_manager
        .load()
        .map_err(|e| CliError::ConfigurationError {
            message: format!("Failed to load configuration: {}", e),
        })?;

    let config_path = config_manager.config_path();

    let mut output = String::new();
    output.push_str(&format!("Configuration file: {}\n", config_path.display()));
    output.push_str(&format!(
        "Configuration exists: {}\n",
        config_manager.config_path().exists()
    ));
    output.push_str("\nCurrent configuration:\n");
    output.push_str(&format!("  Default shell: {:?}\n", config.default_shell));
    output.push_str(&format!("  Safety level: {:?}\n", config.safety_level));
    output.push_str(&format!("  Log level: {:?}\n", config.log_level));
    output.push_str(&format!(
        "  Cache max size: {} GB\n",
        config.cache_max_size_gb
    ));
    output.push_str(&format!(
        "  Log rotation: {} days\n",
        config.log_rotation_days
    ));

    if let Some(model) = &config.default_model {
        output.push_str(&format!("  Default model: {}\n", model));
    }

    Ok(output)
}

fn user_config_to_state(user: &UserConfiguration) -> ConfigurationState {
    let mut state = ConfigurationState::default();
    state.default_shell = user.default_shell;
    state.default_model = user.default_model.clone();
    state.safety_level = user.safety_level;
    state.log_level = user.log_level;
    state.cache_max_size_gb = user.cache_max_size_gb;
    state.log_rotation_days = user.log_rotation_days;
    state
}

fn state_to_user_config(state: &ConfigurationState) -> UserConfiguration {
    UserConfiguration {
        default_shell: state.default_shell,
        safety_level: state.safety_level,
        default_model: state.default_model.clone(),
        log_level: state.log_level,
        cache_max_size_gb: state.cache_max_size_gb,
        log_rotation_days: state.log_rotation_days,
    }
}

async fn run_interactive_configuration(cli: &Cli) -> Result<(), CliError> {
    use colored::Colorize;

    // Create config manager
    let config_manager = if let Some(config_file) = &cli.config_file {
        ConfigManager::with_config_path(config_file.into()).map_err(|e| {
            CliError::ConfigurationError {
                message: format!("Failed to create config manager: {}", e),
            }
        })?
    } else {
        ConfigManager::new().map_err(|e| CliError::ConfigurationError {
            message: format!("Failed to create config manager: {}", e),
        })?
    };

    // Load current configuration
    let current_config = config_manager
        .load()
        .map_err(|e| CliError::ConfigurationError {
            message: format!("Failed to load configuration: {}", e),
        })?;

    let state = user_config_to_state(&current_config);

    // Run interactive configuration
    let result = run_interactive_config(state).map_err(|e| CliError::Internal {
        message: format!("Interactive configuration failed: {}", e),
    })?;

    if result.cancelled {
        println!("{}", "Configuration cancelled.".yellow());
        return Ok(());
    }

    if result.changes_made {
        // Save the updated configuration
        let updated_user_config = state_to_user_config(&result.config);

        config_manager
            .save(&updated_user_config)
            .map_err(|e| CliError::ConfigurationError {
                message: format!("Failed to save configuration: {}", e),
            })?;

        println!("{}", "✅ Configuration saved successfully!".green().bold());
        println!(
            "{}",
            format!(
                "Configuration written to: {}",
                config_manager.config_path().display()
            )
            .dimmed()
        );
    } else {
        println!("{}", "No changes made to configuration.".dimmed());
    }

    Ok(())
}

/// Run interactive testing session
async fn run_interactive_testing() -> Result<(), Box<dyn std::error::Error>> {
    // Create and run the manual test runner
    let mut test_runner = ManualTestRunner::new().await?;
    test_runner.run_interactive_session().await?;
    Ok(())
}

/// Run interactive mode with slash command support
async fn run_interactive_mode() -> Result<(), CliError> {
    // Create CLI application
    let app = CliApp::new().await?;
    
    // Run interactive mode
    app.run_interactive().await?;
    
    Ok(())
}
