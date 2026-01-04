use caro::cli::{CliApp, CliError, IntoCliArgs};
use caro::config::ConfigManager;
use caro::model_manager::ModelCliHandler;
use clap::Parser;
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

    /// Trailing unquoted arguments forming the prompt
    #[arg(trailing_var_arg = true, num_args = 0..)]
    trailing_args: Vec<String>,
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
    // Check for --version (with or without --verbose) before clap parsing
    // to provide custom version output instead of clap's default
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--version".to_string()) || args.contains(&"-V".to_string()) {
        // Show verbose version if --verbose flag is present
        if args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string()) {
            println!("{}", caro::version::long());
        } else {
            // Show short version (matches cargo/rustc format)
            println!("{}", caro::version::short());
        }
        process::exit(0);
    }

    // Handle 'model' subcommand before clap parsing
    if args.len() >= 2 && args[1] == "model" {
        match handle_model_command(&args[2..]).await {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }

    let mut cli = Cli::parse();

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
            println!();
            println!("Examples:");
            println!("  caro list files");
            println!("  caro -p \"list files\"");
            println!("  echo \"list files\" | caro");
            println!("  caro --shell zsh \"find large files\"");
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

/// Handle the 'model' subcommand
async fn handle_model_command(args: &[String]) -> Result<(), anyhow::Error> {
    let handler = ModelCliHandler::new()?;

    if args.is_empty() {
        // Default to showing help
        handler.handle("help", &[]).await
    } else {
        let subcommand = &args[0];
        let sub_args: Vec<String> = args.iter().skip(1).cloned().collect();
        handler.handle(subcommand, &sub_args).await
    }
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
