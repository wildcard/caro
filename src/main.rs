use caro::cli::{CliApp, CliError, IntoCliArgs};
use caro::config::ConfigManager;
use caro::tips::{DisplayStyle, ShellIntelligence, SuggestionResult, TipDisplay, TipsEngine};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process;

// =============================================================================
// Feature 002: Prompt Source Resolution
// =============================================================================

/// Source of the prompt input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptSource {
    /// From -p/--prompt flag (highest priority)
    Flag,
    /// From piped stdin (medium priority)
    Stdin,
    /// From trailing command-line arguments (lowest priority)
    TrailingArgs,
}

/// Resolved prompt with its source
#[derive(Debug, Clone)]
pub struct ResolvedPrompt {
    pub text: String,
    pub source: PromptSource,
}

/// Resolve prompt from multiple input sources following priority order
///
/// Priority: -p/--prompt flag > stdin > trailing arguments
///
/// # Arguments
/// * `flag` - Optional prompt from -p/--prompt flag
/// * `stdin` - Optional prompt from piped stdin
/// * `trailing_args` - Prompt from command-line trailing words
///
/// # Returns
/// ResolvedPrompt with text and source indication
fn resolve_prompt(
    flag: Option<String>,
    stdin: Option<String>,
    trailing_args: Vec<String>,
) -> ResolvedPrompt {
    if let Some(text) = flag {
        ResolvedPrompt {
            text,
            source: PromptSource::Flag,
        }
    } else if let Some(text) = stdin {
        ResolvedPrompt {
            text,
            source: PromptSource::Stdin,
        }
    } else {
        ResolvedPrompt {
            text: trailing_args.join(" "),
            source: PromptSource::TrailingArgs,
        }
    }
}

/// Check if stdin has available input (pipe or redirect)
///
/// Returns true if stdin is not a terminal (i.e., piped or redirected)
fn is_stdin_available() -> bool {
    use std::io::IsTerminal;
    !std::io::stdin().is_terminal()
}

/// Read all content from stdin
///
/// Returns the complete stdin content as a String, or an error if reading fails
fn read_stdin() -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

// =============================================================================
// Prompt Validation
// =============================================================================

/// Action to take after validating a prompt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationAction {
    /// Show help message and exit (for empty/whitespace-only prompts)
    ShowHelp,
    /// Proceed with the prompt (valid content provided)
    ProceedWithPrompt,
}

/// Validate a prompt and determine the appropriate action
///
/// Empty or whitespace-only prompts should display help.
/// Valid prompts should proceed to inference.
/// Special characters are preserved (not validated).
///
/// # Arguments
/// * `prompt` - The prompt text to validate
///
/// # Returns
/// ValidationAction indicating whether to show help or proceed
pub fn validate_prompt(prompt: &str) -> ValidationAction {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        ValidationAction::ShowHelp
    } else {
        ValidationAction::ProceedWithPrompt
    }
}

// =============================================================================
// Shell Operator Detection
// =============================================================================

/// Truncate arguments at the first POSIX shell operator
///
/// Detects standalone shell operators and removes them along with everything after.
/// This handles edge cases where shell operators appear in quoted commands or scripts.
/// In normal usage, the shell processes operators before caro sees them.
///
/// Detected operators: >, |, <, >>, 2>, &, ;
///
/// # Arguments
/// * `args` - Vector of argument strings
///
/// # Returns
/// Truncated vector stopping at the first operator
///
/// # Examples
/// ```
/// let args = vec!["list".into(), "files".into(), ">".into(), "output.txt".into()];
/// let result = truncate_at_shell_operator(args);
/// assert_eq!(result, vec!["list", "files"]);
/// ```
pub fn truncate_at_shell_operator(args: Vec<String>) -> Vec<String> {
    const SHELL_OPERATORS: &[&str] = &[">", "|", "<", ">>", "2>", "&", ";"];

    args.into_iter()
        .take_while(|arg| !SHELL_OPERATORS.contains(&arg.as_str()))
        .collect()
}

// =============================================================================
// CLI Argument Parsing
// =============================================================================

/// caro - Convert natural language to shell commands using local LLMs
#[derive(Parser, Clone)]
#[command(name = "caro")]
#[command(about = "Convert natural language to shell commands using local LLMs")]
#[command(
    long_about = "caro converts natural language descriptions into safe POSIX shell commands using local language models. Features safety validation, multiple output formats, and configurable backends."
)]
#[command(version)]
struct Cli {
    /// Explicit prompt via -p/--prompt flag (highest priority)
    #[arg(
        short = 'p',
        long = "prompt",
        help = "Explicit prompt text (overrides stdin and trailing args)"
    )]
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

    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,

    /// Trailing unquoted arguments forming the prompt
    #[arg(trailing_var_arg = true, num_args = 0..)]
    trailing_args: Vec<String>,
}

/// Available subcommands
#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Show tips for shell commands based on your configuration
    #[command(alias = "tip")]
    Tips {
        /// Command to get tips for (e.g., "git status")
        #[arg(trailing_var_arg = true, num_args = 0..)]
        command: Vec<String>,

        /// Display style (inline, box, minimal)
        #[arg(short, long, default_value = "inline")]
        style: String,

        /// Show tip source information
        #[arg(long)]
        show_source: bool,
    },

    /// List and manage shell aliases
    Aliases {
        /// Filter aliases by name or expansion
        #[arg(short, long)]
        filter: Option<String>,

        /// Show only aliases from a specific source (user, plugin, system)
        #[arg(long)]
        source: Option<String>,

        /// Output format (plain, json)
        #[arg(short, long, default_value = "plain")]
        output: String,
    },

    /// Show shell intelligence information
    #[command(alias = "shell")]
    Info {
        /// Show detected aliases
        #[arg(long)]
        aliases: bool,

        /// Show detected plugins
        #[arg(long)]
        plugins: bool,

        /// Output format (plain, json)
        #[arg(short, long, default_value = "plain")]
        output: String,
    },
}

impl IntoCliArgs for Cli {
    fn prompt(&self) -> Option<String> {
        // Prompt is already resolved in main() from flag/stdin/trailing_args
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
    let mut cli = Cli::parse();

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
            Commands::Tips {
                command,
                style,
                show_source,
            } => {
                run_tips_command(command.clone(), style, *show_source);
                process::exit(0);
            }
            Commands::Aliases {
                filter,
                source,
                output,
            } => {
                run_aliases_command(filter.as_deref(), source.as_deref(), output);
                process::exit(0);
            }
            Commands::Info {
                aliases,
                plugins,
                output,
            } => {
                run_info_command(*aliases, *plugins, output);
                process::exit(0);
            }
        }
    }

    // Truncate trailing args at shell operators (handles edge cases)
    cli.trailing_args = truncate_at_shell_operator(cli.trailing_args);

    // Resolve prompt from multiple sources (flag > stdin > trailing args)
    let stdin_content = if is_stdin_available() {
        match read_stdin() {
            Ok(content) if !content.is_empty() => Some(content),
            _ => None,
        }
    } else {
        None
    };

    let resolved = resolve_prompt(cli.prompt.clone(), stdin_content, cli.trailing_args.clone());

    // Store resolved prompt back into cli for downstream usage
    cli.prompt = Some(resolved.text);

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

    // Validate prompt and show help if empty/whitespace-only
    let prompt_text = cli.prompt.as_deref().unwrap_or("");
    match validate_prompt(prompt_text) {
        ValidationAction::ShowHelp => {
            // Show help message for empty or whitespace-only prompts
            println!("caro - Convert natural language to shell commands using local LLMs");
            println!();
            println!("Usage: caro [OPTIONS] <PROMPT>");
            println!("       caro <COMMAND>");
            println!();
            println!("Commands:");
            println!("  tips       Show tips for shell commands based on your configuration");
            println!("  aliases    List and manage shell aliases");
            println!("  info       Show shell intelligence information");
            println!();
            println!("Examples:");
            println!("  caro list files");
            println!("  caro -p \"list files\"");
            println!("  echo \"list files\" | caro");
            println!("  caro --shell zsh \"find large files\"");
            println!("  caro tips git status");
            println!("  caro aliases --filter git");
            println!();
            println!("Run 'caro --help' for more information.");
            process::exit(0);
        }
        ValidationAction::ProceedWithPrompt => {
            // Continue with command generation
        }
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
    use std::io::IsTerminal;

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
        if std::io::stdin().is_terminal() {
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

            println!("{}", "✓ Confirmed. Command is safe to execute.".green());
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
        println!(
            "  The command would be executed with shell: {:?}",
            result.shell_used
        );
        if result.blocked_reason.is_some() || result.requires_confirmation {
            println!(
                "  {} This command would be blocked or require confirmation",
                "⚠".yellow()
            );
        } else {
            println!("  {} This command would execute successfully", "✓".green());
        }
        println!();
    }
    // If command wasn't executed yet and passes safety checks, ask user if they want to execute
    else if result.exit_code.is_none() && result.executed && !cli.execute && !cli.interactive {
        use dialoguer::Confirm;

        // Check if we're in a terminal environment
        if std::io::stdin().is_terminal() {
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
                            Some(format!(
                                "Command exited with code {}",
                                exec_result.exit_code
                            ))
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
            println!(
                "{}",
                "Use --execute/-x flag to auto-execute commands in non-interactive environments."
                    .dimmed()
            );
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
            println!(
                "  Execution time: {}ms",
                result.timing_info.execution_time_ms
            );
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
        println!(
            "{}",
            "Command was not executed (blocked by safety checks or user cancelled).".yellow()
        );
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

// =============================================================================
// Tips Subcommand Handlers
// =============================================================================

/// Handle the `caro tips` subcommand
fn run_tips_command(command: Vec<String>, style: &str, show_source: bool) {
    let command_str = command.join(" ");

    // Parse display style
    let display_style = match style.to_lowercase().as_str() {
        "box" => DisplayStyle::Box,
        "minimal" => DisplayStyle::Minimal,
        _ => DisplayStyle::Inline,
    };

    // Try to create tips engine
    let mut engine = TipsEngine::new();

    if command_str.is_empty() {
        // No command provided - show usage
        println!(
            "{}",
            "caro tips - Get shell tips and alias suggestions".bold()
        );
        println!();
        println!("Usage: caro tips <command>");
        println!();
        println!("Examples:");
        println!("  caro tips git status");
        println!("  caro tips docker ps");
        println!("  caro tips kubectl get pods");
        println!();
        println!("Options:");
        println!("  -s, --style <style>  Display style: inline, box, minimal");
        println!("  --show-source        Show tip source information");
        return;
    }

    // Configure display
    let display = TipDisplay::new()
        .with_style(display_style)
        .with_source(show_source);

    // Try to get a tip for the command
    match engine.suggest(&command_str) {
        SuggestionResult::Found(tip) => {
            println!("{}", display.format(&tip));
        }
        SuggestionResult::Cooldown | SuggestionResult::SessionLimitReached => {
            // Silently skip - rate limited
            if show_source {
                println!("{}", "Tip rate limited (shown recently)".dimmed());
            }
        }
        SuggestionResult::Disabled => {
            println!("{}", "Tips are currently disabled".dimmed());
        }
        SuggestionResult::NoMatch => {
            println!(
                "{} No tips available for '{}'",
                "Info:".bright_blue().bold(),
                command_str
            );
            println!();
            println!("Try these commands to see tips:");
            println!("  caro tips git status");
            println!("  caro tips docker ps");
            println!("  caro tips ls -la");
        }
    }
}

/// Handle the `caro aliases` subcommand
fn run_aliases_command(filter: Option<&str>, source_filter: Option<&str>, output: &str) {
    // Try to detect shell intelligence
    let Some(intel) = ShellIntelligence::detect() else {
        eprintln!("{}", "Could not detect shell configuration".red());
        return;
    };

    let aliases = intel.aliases();

    // Filter aliases
    let filtered: Vec<_> = aliases
        .values()
        .filter(|alias| {
            // Filter by name/expansion
            if let Some(f) = filter {
                let f_lower = f.to_lowercase();
                if !alias.name.to_lowercase().contains(&f_lower)
                    && !alias.expansion.to_lowercase().contains(&f_lower)
                {
                    return false;
                }
            }

            // Filter by source
            if let Some(src) = source_filter {
                let src_lower = src.to_lowercase();
                match &alias.source {
                    caro::tips::AliasSource::UserConfig(_) => {
                        if src_lower != "user" && src_lower != "config" {
                            return false;
                        }
                    }
                    caro::tips::AliasSource::Plugin(plugin) => {
                        if src_lower != "plugin" && !plugin.to_lowercase().contains(&src_lower) {
                            return false;
                        }
                    }
                    caro::tips::AliasSource::System => {
                        if src_lower != "system" {
                            return false;
                        }
                    }
                    caro::tips::AliasSource::Unknown => {
                        if src_lower != "unknown" {
                            return false;
                        }
                    }
                }
            }

            true
        })
        .collect();

    if output == "json" {
        // JSON output
        let json_aliases: Vec<_> = filtered
            .iter()
            .map(|a| {
                serde_json::json!({
                    "name": a.name,
                    "expansion": a.expansion,
                    "source": format!("{:?}", a.source),
                    "chars_saved": a.chars_saved()
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json_aliases).unwrap_or_default()
        );
    } else {
        // Plain text output
        println!(
            "{} ({} aliases found)",
            "Shell Aliases".bold().cyan(),
            filtered.len()
        );
        println!();

        if filtered.is_empty() {
            println!("{}", "No aliases found matching your criteria.".dimmed());
            return;
        }

        // Sort by name
        let mut sorted: Vec<_> = filtered.into_iter().collect();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        for alias in sorted {
            let saved = alias.chars_saved();
            let source_str = match &alias.source {
                caro::tips::AliasSource::UserConfig(path) => {
                    // Extract just the filename from the path string
                    let filename = path.rsplit('/').next().unwrap_or(path);
                    format!("~/{}", filename)
                }
                caro::tips::AliasSource::Plugin(name) => format!("plugin:{}", name),
                caro::tips::AliasSource::System => "system".to_string(),
                caro::tips::AliasSource::Unknown => "unknown".to_string(),
            };

            print!(
                "  {} {} {}",
                alias.name.bright_green().bold(),
                "→".dimmed(),
                alias.expansion.bright_white()
            );

            if saved > 0 {
                print!(" {}", format!("(saves {})", saved).dimmed());
            }

            print!(" {}", format!("[{}]", source_str).dimmed());
            println!();
        }
    }
}

/// Handle the `caro info` subcommand
fn run_info_command(show_aliases: bool, show_plugins: bool, output: &str) {
    let Some(intel) = ShellIntelligence::detect() else {
        eprintln!("{}", "Could not detect shell configuration".red());
        return;
    };

    let env = intel.environment();

    if output == "json" {
        let mut info = serde_json::json!({
            "shell_type": format!("{:?}", env.shell_type),
            "shell_path": env.shell_path.to_string_lossy(),
            "config_paths": env.config_paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
            "is_interactive": env.is_interactive,
            "is_login_shell": env.is_login_shell,
            "alias_count": intel.aliases().len(),
            "plugin_manager_count": intel.plugin_managers().len()
        });

        if show_aliases {
            let aliases: Vec<_> = intel
                .aliases()
                .values()
                .map(|a| {
                    serde_json::json!({
                        "name": a.name,
                        "expansion": a.expansion,
                        "source": format!("{:?}", a.source)
                    })
                })
                .collect();
            info["aliases"] = serde_json::json!(aliases);
        }

        if show_plugins {
            let managers: Vec<_> = intel
                .plugin_managers()
                .iter()
                .map(|m| format!("{:?}", m))
                .collect();
            info["plugin_managers"] = serde_json::json!(managers);
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
    } else {
        // Plain text output
        println!("{}", "Shell Intelligence Report".bold().cyan());
        println!("{}", "=".repeat(40));
        println!();

        println!("{}", "Environment:".bold());
        println!("  Shell type:    {:?}", env.shell_type);
        println!("  Shell path:    {}", env.shell_path.display());
        println!("  Interactive:   {}", env.is_interactive);
        println!("  Login shell:   {}", env.is_login_shell);
        println!();

        println!("{}", "Configuration files:".bold());
        for path in &env.config_paths {
            let exists = path.exists();
            let status = if exists { "✓".green() } else { "✗".red() };
            println!("  {} {}", status, path.display());
        }
        println!();

        println!("{}", "Statistics:".bold());
        println!("  Aliases detected:      {}", intel.aliases().len());
        println!("  Plugin managers:       {}", intel.plugin_managers().len());
        println!();

        // Show plugin managers
        if !intel.plugin_managers().is_empty() {
            println!("{}", "Plugin Managers:".bold());
            for manager in intel.plugin_managers() {
                match manager {
                    caro::tips::PluginManager::OhMyZsh {
                        path,
                        plugins,
                        theme,
                    } => {
                        println!("  {} Oh My Zsh", "•".bright_magenta());
                        println!("    Path: {}", path.display());
                        if let Some(t) = theme {
                            println!("    Theme: {}", t.bright_cyan());
                        }
                        if !plugins.is_empty() {
                            println!("    Plugins ({}):", plugins.len());
                            for plugin in plugins.iter().take(10) {
                                println!("      - {}", plugin);
                            }
                            if plugins.len() > 10 {
                                println!("      ... and {} more", plugins.len() - 10);
                            }
                        }
                    }
                    caro::tips::PluginManager::Prezto { path, modules } => {
                        println!("  {} Prezto", "•".bright_magenta());
                        println!("    Path: {}", path.display());
                        if !modules.is_empty() {
                            println!("    Modules ({}):", modules.len());
                            for module in modules.iter().take(10) {
                                println!("      - {}", module);
                            }
                            if modules.len() > 10 {
                                println!("      ... and {} more", modules.len() - 10);
                            }
                        }
                    }
                    caro::tips::PluginManager::Zinit { path, plugins } => {
                        println!("  {} Zinit", "•".bright_magenta());
                        println!("    Path: {}", path.display());
                        if !plugins.is_empty() {
                            println!("    Plugins ({}):", plugins.len());
                            for plugin in plugins.iter().take(10) {
                                println!("      - {}", plugin);
                            }
                            if plugins.len() > 10 {
                                println!("      ... and {} more", plugins.len() - 10);
                            }
                        }
                    }
                    caro::tips::PluginManager::Fisher { path, plugins } => {
                        println!("  {} Fisher", "•".bright_magenta());
                        println!("    Path: {}", path.display());
                        if !plugins.is_empty() {
                            println!("    Plugins ({}):", plugins.len());
                            for plugin in plugins.iter().take(10) {
                                println!("      - {}", plugin);
                            }
                            if plugins.len() > 10 {
                                println!("      ... and {} more", plugins.len() - 10);
                            }
                        }
                    }
                    caro::tips::PluginManager::Antigen { plugins } => {
                        println!("  {} Antigen", "•".bright_magenta());
                        if !plugins.is_empty() {
                            println!("    Plugins ({}):", plugins.len());
                            for plugin in plugins.iter().take(10) {
                                println!("      - {}", plugin);
                            }
                            if plugins.len() > 10 {
                                println!("      ... and {} more", plugins.len() - 10);
                            }
                        }
                    }
                    caro::tips::PluginManager::Zplug { plugins } => {
                        println!("  {} Zplug", "•".bright_magenta());
                        if !plugins.is_empty() {
                            println!("    Plugins ({}):", plugins.len());
                            for plugin in plugins.iter().take(10) {
                                println!("      - {}", plugin);
                            }
                            if plugins.len() > 10 {
                                println!("      ... and {} more", plugins.len() - 10);
                            }
                        }
                    }
                }
            }
            println!();
        }

        // Show top aliases if requested
        if show_aliases {
            println!("{}", "Top Aliases (by chars saved):".bold());
            let mut aliases: Vec<_> = intel.aliases().values().collect();
            aliases.sort_by(|a, b| b.chars_saved().cmp(&a.chars_saved()));

            for alias in aliases.iter().take(15) {
                let saved = alias.chars_saved();
                print!(
                    "  {} {} {}",
                    alias.name.bright_green(),
                    "→".dimmed(),
                    alias.expansion
                );
                if saved > 0 {
                    print!(" {}", format!("(saves {})", saved).dimmed());
                }
                println!();
            }

            if aliases.len() > 15 {
                println!("  {} more...", aliases.len() - 15);
            }
            println!();
        }

        // Show plugins if requested
        if show_plugins && intel.has_ohmyzsh() {
            if let Some(caro::tips::PluginManager::OhMyZsh { plugins, .. }) = intel.ohmyzsh() {
                println!("{}", "Installed Oh My Zsh Plugins:".bold());
                for plugin in plugins {
                    println!("  - {}", plugin.bright_cyan());
                }
                println!();
            }
        }

        println!(
            "{}",
            "Run 'caro aliases' for full alias list.".dimmed().italic()
        );
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // WP03: Prompt Source Resolution Tests

    #[test]
    fn test_flag_overrides_all() {
        let resolved = resolve_prompt(
            Some("flag".into()),
            Some("stdin".into()),
            vec!["trailing".into()],
        );
        assert_eq!(resolved.text, "flag");
        assert_eq!(resolved.source, PromptSource::Flag);
    }

    #[test]
    fn test_stdin_overrides_trailing() {
        let resolved = resolve_prompt(None, Some("stdin".into()), vec!["trailing".into()]);
        assert_eq!(resolved.text, "stdin");
        assert_eq!(resolved.source, PromptSource::Stdin);
    }

    #[test]
    fn test_trailing_args_default() {
        let resolved = resolve_prompt(None, None, vec!["list".into(), "files".into()]);
        assert_eq!(resolved.text, "list files");
        assert_eq!(resolved.source, PromptSource::TrailingArgs);
    }

    #[test]
    fn test_empty_trailing_args() {
        let resolved = resolve_prompt(None, None, vec![]);
        assert_eq!(resolved.text, "");
        assert_eq!(resolved.source, PromptSource::TrailingArgs);
    }

    // WP05: Prompt Validation Tests

    #[test]
    fn test_empty_shows_help() {
        assert_eq!(validate_prompt(""), ValidationAction::ShowHelp);
    }

    #[test]
    fn test_whitespace_shows_help() {
        assert_eq!(validate_prompt("   "), ValidationAction::ShowHelp);
        assert_eq!(validate_prompt("\t"), ValidationAction::ShowHelp);
        assert_eq!(validate_prompt("\n"), ValidationAction::ShowHelp);
        assert_eq!(validate_prompt("  \t\n  "), ValidationAction::ShowHelp);
    }

    #[test]
    fn test_valid_prompt_proceeds() {
        assert_eq!(
            validate_prompt("list files"),
            ValidationAction::ProceedWithPrompt
        );
    }

    #[test]
    fn test_special_characters_preserved() {
        // T026: Special characters should be preserved and prompt should proceed
        assert_eq!(
            validate_prompt("find *.txt"),
            ValidationAction::ProceedWithPrompt
        );
        assert_eq!(
            validate_prompt("grep 'pattern' file.txt"),
            ValidationAction::ProceedWithPrompt
        );
        assert_eq!(
            validate_prompt("echo $HOME"),
            ValidationAction::ProceedWithPrompt
        );
    }

    // WP06: Shell Operator Handling Tests

    #[test]
    fn test_all_operators() {
        // T031: Test all 7 POSIX shell operators are detected
        for op in &[">", "|", "<", ">>", "2>", "&", ";"] {
            let args = vec!["cmd".to_string(), op.to_string(), "arg".to_string()];
            let result = truncate_at_shell_operator(args);
            assert_eq!(
                result,
                vec!["cmd"],
                "Failed to truncate at operator: {}",
                op
            );
        }
    }

    #[test]
    fn test_embedded_operator_not_detected() {
        // T032: Embedded operators (not standalone) should be ignored
        let args = vec!["find".to_string(), "files>output.txt".to_string()];
        let result = truncate_at_shell_operator(args);
        assert_eq!(
            result,
            vec!["find", "files>output.txt"],
            "Should not truncate embedded operator"
        );

        // Additional embedded operator cases
        let args2 = vec!["grep".to_string(), "pattern|other".to_string()];
        let result2 = truncate_at_shell_operator(args2);
        assert_eq!(result2, vec!["grep", "pattern|other"]);
    }

    #[test]
    fn test_operator_first() {
        // T033: Operator as first argument should result in empty vector
        let result = truncate_at_shell_operator(vec![">".to_string(), "file".to_string()]);
        assert!(result.is_empty(), "Should be empty when operator is first");

        let result2 = truncate_at_shell_operator(vec!["|".to_string(), "grep".to_string()]);
        assert!(result2.is_empty());
    }

    #[test]
    fn test_multiple_operators() {
        // T034: Should stop at the first operator
        let args = vec![
            "cmd".to_string(),
            ">".to_string(),
            "out".to_string(),
            "|".to_string(),
            "grep".to_string(),
        ];
        let result = truncate_at_shell_operator(args);
        assert_eq!(result, vec!["cmd"], "Should stop at first operator (>)");

        // Test with different operator order
        let args2 = vec![
            "find".to_string(),
            "files".to_string(),
            ";".to_string(),
            "ls".to_string(),
        ];
        let result2 = truncate_at_shell_operator(args2);
        assert_eq!(result2, vec!["find", "files"]);
    }
}
