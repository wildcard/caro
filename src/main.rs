use clap::{Parser, Subcommand};
use caro::cli::{CliApp, CliError, IntoCliArgs};
use caro::config::ConfigManager;
use caro::resources::{OnboardingFlow, ResourceAssessment};
use std::process;

/// caro - Convert natural language to shell commands using local LLMs
#[derive(Parser, Clone)]
#[command(name = "caro")]
#[command(about = "Convert natural language to shell commands using local LLMs")]
#[command(
    long_about = "caro converts natural language descriptions into safe POSIX shell commands using local language models. Features safety validation, multiple output formats, and configurable backends."
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

    /// Subcommands for initialization and configuration
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available subcommands
#[derive(Subcommand, Clone)]
enum Commands {
    /// Initialize or reconfigure Caro with resource assessment
    Init {
        /// Skip interactive prompts and use auto-detected settings
        #[arg(long, help = "Auto-configure without prompts")]
        auto: bool,

        /// Force re-initialization even if already configured
        #[arg(long, help = "Force reconfiguration")]
        force: bool,

        /// Show detailed resource information
        #[arg(long, short, help = "Show detailed system information")]
        verbose: bool,
    },

    /// Show system resources and available models
    Status {
        /// Show detailed resource information
        #[arg(long, short, help = "Show detailed information")]
        verbose: bool,
    },

    /// List available model tiers and their requirements
    Models {
        /// Show all models, including incompatible ones
        #[arg(long, help = "Show all models")]
        all: bool,
    },
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
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_level(true)
            .init();
    } else {
        // Hide all logs in non-verbose mode for clean output
        tracing_subscriber::fmt()
            .with_env_filter("caro=warn")
            .without_time()
            .init();
    }

    // Handle subcommands first
    if let Some(command) = &cli.command {
        match command {
            Commands::Init { auto, force, verbose } => {
                match run_init(*auto, *force, *verbose).await {
                    Ok(()) => process::exit(0),
                    Err(e) => {
                        eprintln!("Error during initialization: {}", e);
                        process::exit(1);
                    }
                }
            }
            Commands::Status { verbose } => {
                match show_status(*verbose).await {
                    Ok(()) => process::exit(0),
                    Err(e) => {
                        eprintln!("Error showing status: {}", e);
                        process::exit(1);
                    }
                }
            }
            Commands::Models { all } => {
                match show_models(*all).await {
                    Ok(()) => process::exit(0),
                    Err(e) => {
                        eprintln!("Error showing models: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
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
        eprintln!("Usage: caro [OPTIONS] <PROMPT>");
        eprintln!("       caro <COMMAND>");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  init     Initialize or reconfigure Caro");
        eprintln!("  status   Show system resources and current configuration");
        eprintln!("  models   List available model tiers");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  caro \"list all files\"");
        eprintln!("  caro --shell zsh \"find large files\"");
        eprintln!("  caro --safety strict \"delete temporary files\"");
        eprintln!("  caro init           # Run interactive setup");
        eprintln!("  caro init --auto    # Auto-configure based on resources");
        eprintln!();
        eprintln!("Run 'caro --help' for more information.");
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
                    eprintln!("caro is currently in development.");
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
        caro::cli::OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&result).map_err(|e| CliError::Internal {
                message: format!("JSON serialization failed: {}", e),
            })?;
            println!("{}", json);
        }
        caro::cli::OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&result).map_err(|e| CliError::Internal {
                message: format!("YAML serialization failed: {}", e),
            })?;
            println!("{}", yaml);
        }
        caro::cli::OutputFormat::Plain => {
            print_plain_output(&mut result, cli).await?;
        }
    }

    Ok(())
}

async fn print_plain_output(result: &mut caro::cli::CliResult, cli: &Cli) -> Result<(), CliError> {
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
                use caro::execution::CommandExecutor;

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

/// Run the initialization/onboarding flow
async fn run_init(auto: bool, _force: bool, verbose: bool) -> Result<(), CliError> {
    use caro::resources::RecommendationEngine;
    use colored::Colorize;

    println!();
    println!("{}", "Caro Initialization".bold().cyan());
    println!("{}", "===================".cyan());
    println!();

    // Assess system resources
    println!("{}", "Assessing system resources...".dimmed());
    let resources = ResourceAssessment::assess().map_err(|e| CliError::Internal {
        message: format!("Failed to assess resources: {}", e),
    })?;

    if verbose {
        println!();
        println!("{}", resources);
    }

    let engine = RecommendationEngine::new(resources.clone());

    if auto {
        // Auto-configure based on resources
        let recommendation = engine.recommend();
        println!();
        println!("{} {}", "Auto-selected model:".bold(), recommendation.model.name.cyan());
        println!("  Tier: {}", recommendation.tier);
        println!("  Size: {:.1} GB", recommendation.model.size_gb());
        println!();
        println!("{}", recommendation.reasoning);

        if !recommendation.warnings.is_empty() {
            println!();
            println!("{}", "Warnings:".yellow().bold());
            for warning in &recommendation.warnings {
                println!("  {} {}", "!".yellow(), warning);
            }
        }

        // Save configuration
        save_model_config(&recommendation.tier, &resources).await?;

        println!();
        println!("{}", "Configuration saved!".green().bold());
        println!("Run 'caro status' to view your configuration.");
    } else {
        // Interactive setup
        let flow = OnboardingFlow::with_resources(resources);

        match flow.run().await {
            Ok(result) => {
                println!();
                println!("{}", "Initialization complete!".green().bold());
                println!();
                println!("Selected: {} ({})", result.model_config.model.name, result.selected_tier);

                if result.needs_download {
                    println!();
                    println!(
                        "{} Model download required ({:.1} GB)",
                        "Note:".yellow().bold(),
                        result.download_size_mb as f64 / 1024.0
                    );
                    println!("The model will be downloaded on first use.");
                }

                // Save configuration
                save_model_config(&result.selected_tier, &result.resources).await?;

                println!();
                println!("You can now use Caro:");
                println!("  {} {}", "caro".cyan(), "\"your command description\"".dimmed());
            }
            Err(caro::resources::ResourceError::UserCancelled) => {
                println!();
                println!("{}", "Initialization cancelled.".yellow());
            }
            Err(e) => {
                return Err(CliError::Internal {
                    message: format!("Initialization failed: {}", e),
                });
            }
        }
    }

    Ok(())
}

/// Save model configuration
async fn save_model_config(
    tier: &caro::resources::ModelTier,
    _resources: &caro::resources::SystemResources,
) -> Result<(), CliError> {
    use caro::resources::ModelInfo;

    let config_manager = ConfigManager::new().map_err(|e| CliError::ConfigurationError {
        message: format!("Failed to create config manager: {}", e),
    })?;

    let mut config = config_manager.load().map_err(|e| CliError::ConfigurationError {
        message: format!("Failed to load configuration: {}", e),
    })?;

    // Set the default model based on tier
    let model = ModelInfo::for_tier(*tier);
    config.default_model = Some(model.model_id);

    // Adjust cache size based on model requirements
    let model_size_gb = model.size_mb / 1024;
    if config.cache_max_size_gb < model_size_gb + 2 {
        config.cache_max_size_gb = model_size_gb + 5; // Model + 5GB buffer
    }

    config_manager.save(&config).map_err(|e| CliError::ConfigurationError {
        message: format!("Failed to save configuration: {}", e),
    })?;

    Ok(())
}

/// Show system status and current configuration
async fn show_status(verbose: bool) -> Result<(), CliError> {
    use caro::resources::RecommendationEngine;
    use colored::Colorize;

    println!();
    println!("{}", "Caro Status".bold().cyan());
    println!("{}", "===========".cyan());
    println!();

    // Show system resources
    let resources = ResourceAssessment::assess().map_err(|e| CliError::Internal {
        message: format!("Failed to assess resources: {}", e),
    })?;

    println!("{}", "System Resources:".bold());
    println!("  CPU: {} ({} cores)", resources.cpu_brand, resources.cpu_cores);
    println!("  RAM: {} GB total, {} GB available", resources.ram_gb(), resources.available_ram_gb());

    if let Some(gpu) = &resources.gpu {
        println!("  GPU: {} ({})", gpu.name, gpu.vendor);
        if resources.is_apple_silicon {
            println!("  Unified Memory: {} GB effective", resources.effective_gpu_memory_gb());
        } else if gpu.vram_mb > 0 {
            println!("  VRAM: {} GB", gpu.vram_mb / 1024);
        }
    } else {
        println!("  GPU: {} (CPU mode)", "Not detected".dimmed());
    }

    println!("  Storage: {} GB available", resources.available_storage_gb());

    let engine = RecommendationEngine::new(resources.clone());
    println!();
    println!("  Machine Class: {}", engine.machine_class().bold());

    // Show current configuration
    println!();
    println!("{}", "Current Configuration:".bold());

    let config_manager = ConfigManager::new().map_err(|e| CliError::ConfigurationError {
        message: format!("Failed to create config manager: {}", e),
    })?;

    let config = config_manager.load().map_err(|e| CliError::ConfigurationError {
        message: format!("Failed to load configuration: {}", e),
    })?;

    if let Some(model) = &config.default_model {
        println!("  Default Model: {}", model.cyan());
    } else {
        println!("  Default Model: {} (run 'caro init' to configure)", "Not set".yellow());
    }

    println!("  Safety Level: {:?}", config.safety_level);
    println!("  Cache Size: {} GB max", config.cache_max_size_gb);

    if verbose {
        println!();
        println!("{}", "Recommended Model:".bold());
        let recommendation = engine.recommend();
        println!("  {} ({})", recommendation.model.name, recommendation.tier);
        println!("  {}", recommendation.reasoning.dimmed());
    }

    println!();

    Ok(())
}

/// Show available models
async fn show_models(all: bool) -> Result<(), CliError> {
    use caro::resources::{ModelInfo, ModelTier, RecommendationEngine};
    use colored::Colorize;

    println!();
    println!("{}", "Available Models".bold().cyan());
    println!("{}", "================".cyan());
    println!();

    let resources = ResourceAssessment::assess().map_err(|e| CliError::Internal {
        message: format!("Failed to assess resources: {}", e),
    })?;

    let engine = RecommendationEngine::new(resources.clone());
    let recommendation = engine.recommend();

    for tier in ModelTier::presets() {
        let model = ModelInfo::for_tier(*tier);
        let rec = caro::resources::ModelRecommendation::new(*tier, &resources);

        // Skip incompatible models unless --all is specified
        if !all && !rec.is_compatible {
            continue;
        }

        let is_recommended = *tier == recommendation.tier;
        let status = if is_recommended {
            " [RECOMMENDED]".green().bold()
        } else if !rec.is_compatible {
            " [INCOMPATIBLE]".red()
        } else {
            "".normal()
        };

        println!("{}{}", format!("{} ({})", model.name, tier).bold(), status);
        println!("  Parameters: {:.1}B", model.parameters_b);
        println!("  Size: {:.1} GB download", model.size_gb());
        println!("  Latency: ~{:.1}s typical", model.typical_latency_s);

        let mut features = Vec::new();
        if model.supports_thinking {
            features.push("Thinking");
        }
        if model.supports_tool_calling {
            features.push("Tool Calling");
        }
        if features.is_empty() {
            features.push("Basic");
        }
        println!("  Features: {}", features.join(", "));

        println!("  Requirements: {} GB RAM, {} GB storage", model.min_ram_gb, model.min_storage_gb);

        if !rec.warnings.is_empty() {
            for warning in &rec.warnings {
                println!("  {} {}", "!".yellow(), warning.yellow());
            }
        }

        println!("  {}", model.description.dimmed());
        println!();
    }

    println!("{}", "Custom Model:".bold());
    println!("  You can specify any Hugging Face model during 'caro init'.");
    println!("  The model must provide GGUF format files.");
    println!();

    Ok(())
}
