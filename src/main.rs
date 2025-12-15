use clap::Parser;
use cmdai::cli::{CliApp, CliError, IntoCliArgs};
use cmdai::config::ConfigManager;
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

    /// Execute the generated command
    #[arg(
        short = 'x',
        long,
        help = "Execute the generated command after validation"
    )]
    execute: bool,

    /// Dry run mode (show what would be executed)
    #[arg(long, help = "Show execution plan without running the command")]
    dry_run: bool,

    /// Interactive execution mode
    #[arg(
        short = 'i',
        long,
        help = "Interactive mode with step-by-step confirmation"
    )]
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

    fn execute(&self) -> bool {
        self.execute
    }

    fn dry_run(&self) -> bool {
        self.dry_run
    }

    fn interactive(&self) -> bool {
        self.interactive
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize tracing/logging
    if cli.verbose {
        // In verbose mode, show all logs (info, debug, trace) with timestamps
        // Allow RUST_LOG env var to override if set
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("cmdai=info"));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_level(true)
            .init();
    } else {
        // Only show warnings and errors by default for cleaner output
        tracing_subscriber::fmt()
            .with_env_filter("cmdai=warn")
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

    // Handle missing prompt
    if cli.prompt.is_none() {
        eprintln!("Error: No prompt provided");
        eprintln!();
        eprintln!("Usage: cmdai [OPTIONS] <PROMPT>");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  cmdai \"list all files\"");
        eprintln!("  cmdai --shell zsh \"find large files\"");
        eprintln!("  cmdai --safety strict \"delete temporary files\"");
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
    let mut result = app.run_with_args(cli.clone()).await?;

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
            print_plain_output(&mut result, cli).await?;
        }
    }

    Ok(())
}

async fn print_plain_output(result: &mut cmdai::cli::CliResult, cli: &Cli) -> Result<(), CliError> {
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

    // Handle confirmation required for dangerous commands
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
                "✓ Confirmed. Command is safe to execute.".green()
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

    // Print explanation only in verbose mode
    if cli.verbose && !result.explanation.is_empty() {
        println!("{}", "Explanation:".bold());
        println!("  {}", result.explanation);
        println!();
    }

    // Handle dry-run mode
    if cli.dry_run {
        println!("{}", "Dry Run Mode:".bold().cyan());
        println!("  The command would be executed with shell: {:?}", result.shell_used);
        if result.blocked_reason.is_some() || result.requires_confirmation {
            println!("  {} This command would be blocked or require confirmation", "⚠".yellow());
        } else {
            println!("  {} This command would execute successfully", "✓".green());
        }
        println!();
    }
    // If command wasn't executed yet and passes safety checks, ask user if they want to execute
    else if result.exit_code.is_none() && result.executed && !cli.execute && !cli.interactive {
        use dialoguer::Confirm;

        // Check if we're in a terminal environment
        if atty::is(atty::Stream::Stdin) {
            let should_execute = Confirm::new()
                .with_prompt("Execute this command?")
                .default(false)
                .interact()
                .map_err(|e| CliError::Internal {
                    message: format!("Failed to get user confirmation: {}", e),
                })?;

            if should_execute {
                println!();
                println!("{}", "Executing command...".dimmed());

                // Execute the command
                use cmdai::execution::CommandExecutor;

                let executor = CommandExecutor::new(result.shell_used);

                match executor.execute(&result.generated_command) {
                    Ok(exec_result) => {
                        result.exit_code = Some(exec_result.exit_code);
                        result.stdout = Some(exec_result.stdout);
                        result.stderr = Some(exec_result.stderr);
                        result.execution_error = if !exec_result.success {
                            Some(format!("Command exited with code {}", exec_result.exit_code))
                        } else {
                            None
                        };
                        result.timing_info.execution_time_ms = exec_result.execution_time_ms;
                    }
                    Err(e) => {
                        result.execution_error = Some(format!("Execution failed: {}", e));
                    }
                }
                println!();
            } else {
                println!("{}", "Execution skipped.".yellow());
                println!();
            }
        } else {
            // Non-interactive environment - show message
            println!("{}", "Use --execute/-x flag to auto-execute commands in non-interactive environments.".dimmed());
            println!();
        }
    }

    // Print execution results if command was actually executed
    if result.exit_code.is_some() {
        println!("{}", "Execution Results:".bold().green());

        // Print exit code
        if let Some(exit_code) = result.exit_code {
            let status_msg = if exit_code == 0 {
                format!("✓ Success (exit code: {})", exit_code).green()
            } else {
                format!("✗ Failed (exit code: {})", exit_code).red()
            };
            println!("  {}", status_msg);
        }

        // Print execution time
        if result.timing_info.execution_time_ms > 0 {
            println!("  Execution time: {}ms", result.timing_info.execution_time_ms);
        }

        // Print stdout if present
        if let Some(stdout) = &result.stdout {
            if !stdout.trim().is_empty() {
                println!();
                println!("{}", "Standard Output:".bold());
                for line in stdout.lines() {
                    println!("  {}", line);
                }
            }
        }

        // Print stderr if present
        if let Some(stderr) = &result.stderr {
            if !stderr.trim().is_empty() {
                println!();
                println!("{}", "Standard Error:".bold().yellow());
                for line in stderr.lines() {
                    println!("  {}", line.yellow());
                }
            }
        }

        // Print execution error if present
        if let Some(error) = &result.execution_error {
            println!();
            println!("{} {}", "Execution Error:".red().bold(), error.red());
        }

        println!();
    } else if cli.execute || cli.interactive {
        // User requested execution but it didn't happen
        println!("{}", "Command was not executed (blocked by safety checks or user cancelled).".yellow());
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
