use clap::{Parser, Subcommand};
use cmdai::cli::{CliApp, CliError, IntoCliArgs};
use cmdai::config::ConfigManager;
use std::process;

/// cmdai V2 - Intelligent Shell Assistant with Context Awareness
#[derive(Parser, Clone)]
#[command(name = "cmdai")]
#[command(about = "V2: Intelligent Shell Assistant with Context Awareness & Learning")]
#[command(
    long_about = "cmdai V2 converts natural language to safe POSIX shell commands using local LLMs.\n\
                  Features: Context Intelligence, ML-based Safety, Pattern Learning, Interactive Tutorials.\n\
                  All data stored locally. Privacy-first design."
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

    /// Disable context intelligence (faster but less accurate)
    #[arg(long, help = "Disable context intelligence analysis")]
    no_context: bool,

    // ============ V2 NEW FLAGS ============
    /// Show detected context before generating command
    #[arg(long, help = "Display detected environment context")]
    show_context: bool,

    /// Explain what a command does (instead of generating)
    #[arg(long, value_name = "COMMAND", help = "Explain how a shell command works")]
    explain: Option<String>,

    /// Execute command in sandbox (safe rollback)
    #[arg(long, help = "Execute in sandbox with rollback capability")]
    sandbox: bool,

    /// Show learning statistics
    #[arg(long, help = "Show learning stats (patterns, achievements)")]
    stats: bool,

    /// Enable audit logging (compliance mode)
    #[arg(long, help = "Enable audit logging for compliance")]
    audit: bool,

    /// Export audit logs to file
    #[arg(long, value_name = "PATH", help = "Export audit logs to specified path")]
    export_audit: Option<String>,

    /// Backend to use (mlx, vllm, ollama, cpu)
    #[arg(short = 'b', long, help = "Backend to use for inference")]
    backend: Option<String>,

    /// Model name or path
    #[arg(short = 'm', long, help = "Model name or path")]
    model: Option<String>,

    /// V2 Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

/// V2 Subcommands
#[derive(Subcommand, Clone)]
enum Commands {
    /// List available tutorials
    Tutorials,

    /// Start an interactive tutorial
    #[command(name = "tutorial")]
    Tutorial {
        /// Tutorial name (e.g., find-basics, grep-basics)
        name: String,
    },

    /// Explain a shell command
    Explain {
        /// Command to explain
        command: String,
    },

    /// Show learning statistics
    Stats,

    /// Manage context intelligence
    Context {
        #[command(subcommand)]
        action: ContextAction,
    },
}

/// Context management actions
#[derive(Subcommand, Clone)]
enum ContextAction {
    /// Show current context
    Show,
    /// Clear context cache
    Clear,
    /// Opt out of history analysis
    OptOut {
        /// Module to opt out of (history, git, tools)
        module: String,
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

    // Handle V2 subcommands
    if let Some(command) = &cli.command {
        match handle_subcommand(command, &cli).await {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle --explain flag
    if let Some(command) = &cli.explain {
        match handle_explain(command).await {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error explaining command: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle --stats flag
    if cli.stats {
        match handle_stats().await {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error showing stats: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle --export-audit flag
    if let Some(export_path) = &cli.export_audit {
        match handle_export_audit(export_path).await {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error exporting audit logs: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle missing prompt for command generation
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
        eprintln!("V2 Features:");
        eprintln!("  cmdai --explain \"find . -name '*.rs'\"");
        eprintln!("  cmdai tutorial find-basics");
        eprintln!("  cmdai --stats");
        eprintln!("  cmdai --sandbox \"delete old logs\"");
        eprintln!();
        eprintln!("Run 'cmdai --help' for more information.");
        process::exit(1);
    }

    // Run the CLI application for command generation
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

    // Print context summary if verbose
    if cli.verbose {
        if let Some(summary) = &result.context_summary {
            eprintln!("{} {}", "Context:".dimmed(), summary.dimmed());
            eprintln!();
        }
    }

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

// ============ V2 COMMAND HANDLERS ============

/// Handle V2 subcommands
async fn handle_subcommand(command: &Commands, cli: &Cli) -> Result<(), CliError> {
    match command {
        Commands::Tutorials => handle_list_tutorials().await,
        Commands::Tutorial { name } => handle_run_tutorial(name).await,
        Commands::Explain { command } => handle_explain(command).await,
        Commands::Stats => handle_stats().await,
        Commands::Context { action } => handle_context_action(action, cli).await,
    }
}

/// List available tutorials
async fn handle_list_tutorials() -> Result<(), CliError> {
    use cmdai::learning::{Tutorial, Difficulty};
    use colored::Colorize;

    println!("{}", "Available Tutorials".bold().cyan());
    println!("{}", "===================".dimmed());
    println!();

    // Built-in tutorials (from learning engine)
    let tutorials = vec![
        ("find-basics", "Mastering the find Command", Difficulty::Beginner),
        ("grep-basics", "Mastering the grep Command", Difficulty::Beginner),
    ];

    for (id, title, difficulty) in tutorials {
        let difficulty_color = match difficulty {
            Difficulty::Beginner => "Beginner".green(),
            Difficulty::Intermediate => "Intermediate".yellow(),
            Difficulty::Advanced => "Advanced".red(),
        };
        println!("  {} - {} [{}]", id.bold(), title, difficulty_color);
    }

    println!();
    println!("{}", "Usage:".dimmed());
    println!("  cmdai tutorial <tutorial-name>");
    println!();
    println!("{}", "Example:".dimmed());
    println!("  cmdai tutorial find-basics");

    Ok(())
}

/// Run an interactive tutorial
async fn handle_run_tutorial(tutorial_name: &str) -> Result<(), CliError> {
    use cmdai::learning::Tutorial;
    use colored::Colorize;
    use std::io::{self, Write};

    // Load tutorial
    let tutorial = match Tutorial::load(tutorial_name) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("{} Tutorial '{}' not found", "Error:".red().bold(), tutorial_name);
            eprintln!();
            eprintln!("Available tutorials:");
            eprintln!("  - find-basics");
            eprintln!("  - grep-basics");
            eprintln!();
            eprintln!("Run 'cmdai tutorials' to see all available tutorials.");
            return Err(CliError::InvalidArgument {
                message: format!("Tutorial '{}' not found", tutorial_name),
            });
        }
    };

    println!("{}", "========================================".cyan());
    println!("{}", format!("Tutorial: {}", tutorial.title).bold().cyan());
    println!("{}", format!("Difficulty: {:?}", tutorial.difficulty).dimmed());
    println!("{}", "========================================".cyan());
    println!();

    // Run through lessons
    for (idx, lesson) in tutorial.lessons.iter().enumerate() {
        println!("{}", format!("Lesson {}/{}: {}", idx + 1, tutorial.lessons.len(), lesson.title).bold());
        println!();
        println!("{}", lesson.explanation);
        println!();

        if !lesson.example_command.is_empty() {
            println!("{}", "Example:".dimmed());
            println!("  $ {}", lesson.example_command.bright_cyan());
            println!();
            if !lesson.expected_output.is_empty() {
                println!("{}", "This will:".dimmed());
                println!("  {}", lesson.expected_output);
                println!();
            }
        }

        if !lesson.hints.is_empty() {
            println!("{}", "Hints:".dimmed());
            for hint in &lesson.hints {
                println!("  • {}", hint);
            }
            println!();
        }

        // Quiz (if present)
        if let Some(quiz) = &lesson.quiz {
            println!("{}", format!("Quiz: {}", quiz.question).yellow());
            print!("Your answer: ");
            io::stdout().flush().map_err(|e| CliError::Internal {
                message: format!("IO error: {}", e),
            })?;

            let mut answer = String::new();
            io::stdin().read_line(&mut answer).map_err(|e| CliError::Internal {
                message: format!("Failed to read input: {}", e),
            })?;

            let answer = answer.trim();
            if answer == quiz.answer {
                println!("{}", "✓ Correct!".green().bold());
            } else {
                println!("{}", format!("✗ Not quite. Expected: {}", quiz.answer).red());
                if !quiz.hints.is_empty() {
                    println!("{}", "Hints:".dimmed());
                    for hint in &quiz.hints {
                        println!("  • {}", hint);
                    }
                }
            }
            println!();
        }

        // Pause between lessons
        if idx < tutorial.lessons.len() - 1 {
            println!("{}", "Press Enter to continue...".dimmed());
            let mut _pause = String::new();
            io::stdin().read_line(&mut _pause).ok();
            println!();
        }
    }

    println!("{}", "🎉 Tutorial complete!".green().bold());
    println!();

    Ok(())
}

/// Explain a command
async fn handle_explain(command: &str) -> Result<(), CliError> {
    use cmdai::learning::CommandExplainer;
    use colored::Colorize;

    let explainer = CommandExplainer::new().map_err(|e| CliError::Internal {
        message: format!("Failed to create explainer: {}", e),
    })?;

    let explanation = explainer.explain(command).map_err(|e| CliError::Internal {
        message: format!("Failed to explain command: {}", e),
    })?;

    println!("{}", format!("Command: {}", command).bold().cyan());
    println!();

    // Print breakdown
    println!("{}", "Breakdown:".bold());
    for part in &explanation.breakdown {
        let type_label = match part.part_type {
            cmdai::learning::PartType::Command => "[Command]".bright_green(),
            cmdai::learning::PartType::Flag => "[Flag]".bright_yellow(),
            cmdai::learning::PartType::Argument => "[Argument]".bright_blue(),
            cmdai::learning::PartType::Pipe => "[Pipe]".bright_magenta(),
            cmdai::learning::PartType::Redirect => "[Redirect]".bright_cyan(),
            cmdai::learning::PartType::Operator => "[Operator]".bright_white(),
        };
        println!("  {} {}", type_label, part.part.bold());
        println!("    → {}", part.explanation.dimmed());
    }
    println!();

    // Safety warnings
    if !explanation.safety_notes.is_empty() {
        println!("{}", "Safety Warnings:".red().bold());
        for note in &explanation.safety_notes {
            println!("  {} {}", "⚠️".yellow(), note);
        }
        println!();
    }

    // Alternatives
    if !explanation.alternatives.is_empty() {
        println!("{}", "Alternatives:".bold());
        for (idx, alt) in explanation.alternatives.iter().enumerate() {
            println!("  {}. {}", idx + 1, alt.command.bright_cyan());
            println!("     Reason: {}", alt.reason.dimmed());
            if !alt.benefits.is_empty() {
                println!("     Benefits:");
                for benefit in &alt.benefits {
                    println!("       - {}", benefit.dimmed());
                }
            }
            println!();
        }
    }

    Ok(())
}

/// Show learning statistics
async fn handle_stats() -> Result<(), CliError> {
    use cmdai::learning::PatternDB;
    use colored::Colorize;

    // Get database path
    let db_path = dirs::data_local_dir()
        .or_else(|| dirs::home_dir())
        .ok_or_else(|| CliError::Internal {
            message: "Failed to determine database directory".to_string(),
        })?
        .join("cmdai")
        .join("patterns.db");

    let db = PatternDB::new(db_path).await.map_err(|e| CliError::Internal {
        message: format!("Failed to open pattern database: {}", e),
    })?;

    // Get statistics
    let total_commands = db.count_patterns().await.map_err(|e| CliError::Internal {
        message: format!("Failed to count patterns: {}", e),
    })?;

    let total_edits = db.count_edited_patterns().await.map_err(|e| CliError::Internal {
        message: format!("Failed to count edited patterns: {}", e),
    })?;

    let edit_rate = if total_commands > 0 {
        (total_edits as f64 / total_commands as f64) * 100.0
    } else {
        0.0
    };

    println!("{}", "📊 Learning Statistics".bold().cyan());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("Commands Generated: {}", total_commands.to_string().bold());
    println!("User Edits: {} ({:.1}% edit rate)",
             total_edits.to_string().bold(),
             edit_rate);
    println!();

    // Show recent patterns
    let recent_patterns = db.get_edited_patterns(5).await.unwrap_or_default();
    if !recent_patterns.is_empty() {
        println!("{}", "🔥 Recent Edited Commands:".bold());
        for (idx, pattern) in recent_patterns.iter().enumerate() {
            println!("  {}. {}",
                     (idx + 1).to_string().dimmed(),
                     pattern.generated_command.bright_cyan());
            if let Some(final_cmd) = &pattern.final_command {
                println!("     → {}", final_cmd.green());
            }
        }
        println!();
    }

    Ok(())
}

/// Handle context management actions
async fn handle_context_action(action: &ContextAction, cli: &Cli) -> Result<(), CliError> {
    use cmdai::intelligence::ContextGraph;
    use colored::Colorize;

    match action {
        ContextAction::Show => {
            // Build and show current context
            let context = ContextGraph::build(&std::env::current_dir().unwrap_or_default())
                .await
                .map_err(|e| CliError::Internal {
                    message: format!("Failed to build context: {}", e),
                })?;

            println!("{}", "Current Environment Context".bold().cyan());
            println!("{}", "===========================".dimmed());
            println!();
            println!("{}", context.summary());

            if cli.verbose {
                println!();
                println!("{}", "LLM Context String:".dimmed());
                println!("{}", context.to_llm_context());
            }
        }
        ContextAction::Clear => {
            println!("{}", "Context cache cleared (not implemented yet)".yellow());
        }
        ContextAction::OptOut { module } => {
            println!("{}", format!("Opted out of {} analysis (not implemented yet)", module).yellow());
        }
    }

    Ok(())
}

/// Export audit logs
async fn handle_export_audit(export_path: &str) -> Result<(), CliError> {
    use colored::Colorize;

    // Note: Full audit logger implementation will be added in future update
    // For now, provide informative message
    println!("{}", "⚠️  Audit logging export not yet fully integrated".yellow());
    println!("{}", format!("Export path: {}", export_path).dimmed());
    println!();
    println!("{}", "Audit logging is planned for a future update with:".dimmed());
    println!("  • Comprehensive command execution logs");
    println!("  • Compliance exports (CSV, Splunk, Elasticsearch)");
    println!("  • Risk-based filtering and search");
    println!();
    println!("{}", "Track progress at: https://github.com/wildcard/cmdai/issues".dimmed());

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
