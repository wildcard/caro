use caro::backends::embedded::EmbeddedModelBackend;
use caro::backends::{CommandGenerator, StaticMatcher};
use caro::cli::{CliApp, CliError, IntoCliArgs};
use caro::config::ConfigManager;
use caro::eval::{CategoryResults, EvalResults, EvalSuite, IndividualResult};
use caro::models::{CommandRequest, ShellType};
use caro::prompts::CapabilityProfile;
use clap::Parser;
use std::collections::HashMap;
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

/// Export format for assessment results
#[derive(Debug, Clone, clap::ValueEnum)]
enum ExportFormat {
    Json,
    Markdown,
}

/// Config subcommands
#[derive(Parser, Clone)]
#[command(arg_required_else_help = true)]
enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration key (backend, model-name, shell, safety)
        key: String,
        /// Value to set
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key (backend, model-name, shell, safety)
        key: String,
    },
    /// Show all configuration
    Show,
    /// Reset configuration to defaults
    Reset,
}

/// Profile management subcommands
#[cfg(feature = "knowledge")]
#[derive(Parser, Clone)]
#[command(arg_required_else_help = true)]
enum ProfileCommands {
    /// Create a new user profile
    Create {
        /// Profile name (e.g., "work", "personal-laptop")
        name: String,

        /// Profile type
        #[arg(long, value_enum, default_value = "personal")]
        profile_type: caro::models::profile::ProfileType,

        /// Optional description
        #[arg(long, short = 'd')]
        description: Option<String>,
    },

    /// List all user profiles
    List,

    /// Switch to a different profile
    Switch {
        /// Profile name to switch to
        name: String,
    },

    /// Delete a user profile
    Delete {
        /// Profile name to delete
        name: String,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// Show the currently active profile
    Show,
}

/// Knowledge index management subcommands
#[cfg(feature = "knowledge")]
#[derive(Parser, Clone)]
#[command(arg_required_else_help = true)]
enum KnowledgeCommands {
    /// Index man pages into the documentation collection
    IndexMan {
        /// Specific man page to index (e.g., "ls", "grep")
        #[arg(help = "Man page to index (omit to index all)")]
        page: Option<String>,

        /// Man page sections to index (e.g., "1,8")
        #[arg(long, value_delimiter = ',')]
        sections: Option<Vec<u8>>,

        /// Show progress during indexing
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Index tldr pages into the documentation collection
    IndexTldr {
        /// Specific command to index (e.g., "git", "docker")
        #[arg(help = "Command to index (omit to index all)")]
        command: Option<String>,

        /// Platform filter (linux, osx, windows, common)
        #[arg(long, value_delimiter = ',')]
        platforms: Option<Vec<String>>,

        /// Show progress during indexing
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Index command --help output into the documentation collection
    IndexHelp {
        /// Specific command to index (e.g., "cargo", "npm")
        #[arg(help = "Command to index (omit to auto-discover from PATH)")]
        command: Option<String>,

        /// List of commands to index
        #[arg(long, value_delimiter = ',')]
        commands: Option<Vec<String>>,

        /// Show progress during indexing
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Index GitHub repository documentation
    IndexGitHub {
        /// GitHub repository in format owner/repo (e.g., "wildcard/caro")
        repo: String,

        /// Show progress during indexing
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Show knowledge index statistics
    Stats,

    /// Search for similar commands in the knowledge index
    Search {
        /// Query to search for
        query: String,

        /// Maximum number of results to return
        #[arg(long, short = 'n', default_value = "5")]
        limit: usize,
    },

    /// Export knowledge index to JSON file
    Export {
        /// Output file path
        path: std::path::PathBuf,
    },

    /// Import knowledge from JSON file
    Import {
        /// Input file path
        path: std::path::PathBuf,

        /// Merge with existing knowledge (default: replace)
        #[arg(long)]
        merge: bool,
    },

    /// Clear the knowledge index
    Clear {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

/// Subcommands for caro
#[derive(Parser, Clone)]
enum Commands {
    /// Run system diagnostics and health checks
    Doctor,

    /// Generate shell integration script for edit mode support
    Init {
        /// Shell to generate init script for (zsh, bash, fish)
        shell: String,
    },

    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Manage knowledge index (requires --features knowledge)
    #[cfg(feature = "knowledge")]
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommands,
    },

    /// Manage user profiles for personalized knowledge (requires --features knowledge)
    #[cfg(feature = "knowledge")]
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    // NOTE: Assess and Telemetry subcommands are disabled in v1.1.0-beta.1
    // They will be implemented in a future release
    //
    // /// Assess system resources and get model recommendations
    // Assess {
    //     /// Export format (json, markdown)
    //     #[arg(long, value_enum)]
    //     export: Option<ExportFormat>,
    //
    //     /// Output file path
    //     #[arg(long, short = 'o')]
    //     output: Option<String>,
    // },
    /// Run evaluation tests on command generation quality
    Test {
        /// Backend to test (static, mlx, ollama, or embedded)
        #[arg(short, long, default_value = "static")]
        backend: String,

        /// Show verbose output including all test cases
        #[arg(short, long)]
        verbose: bool,

        /// Path to YAML test suite file
        #[arg(long)]
        suite: Option<String>,

        /// Filter tests by profile ID (e.g., bt_001)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Generate shell completion scripts
    Completion {
        /// Shell to generate completions for (bash, zsh, fish)
        shell: String,
    },

    /// Suggest commands matching a natural language description
    Suggest {
        /// Partial command description
        query: String,

        /// Maximum number of suggestions
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
    // /// Manage telemetry data and settings
    // Telemetry {
    //     #[command(subcommand)]
    //     command: caro::cli::telemetry::TelemetryCommands,
    // },
}

/// caro - Convert natural language to shell commands using local LLMs
#[derive(Parser, Clone)]
#[command(name = "caro")]
#[command(about = "Convert natural language to shell commands using local LLMs")]
#[command(
    long_about = "caro converts natural language descriptions into safe POSIX shell commands using local language models. Features safety validation, multiple output formats, and configurable backends."
)]
#[command(version)]
#[command(args_conflicts_with_subcommands = true)]
#[command(subcommand_required = false)]
#[command(arg_required_else_help = false)]
struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,

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

    /// Backend to use for inference
    #[arg(
        short = 'b',
        long,
        help = "Inference backend (embedded, ollama, exo, vllm)"
    )]
    backend: Option<String>,

    /// Model name to use with the backend
    #[arg(
        short = 'm',
        long = "model-name",
        help = "Model name for the backend (e.g., codellama:7b for ollama)"
    )]
    model_name: Option<String>,

    /// Knowledge backend for command history and learning
    #[arg(
        long = "knowledge-backend",
        help = "Vector database backend for knowledge index (lancedb, chromadb)",
        env = "CARO_KNOWLEDGE_BACKEND"
    )]
    knowledge_backend: Option<String>,

    /// ChromaDB server URL (when using chromadb backend)
    #[arg(
        long = "chromadb-url",
        help = "ChromaDB server URL (default: http://localhost:8000)",
        env = "CHROMADB_URL",
        default_value = "http://localhost:8000"
    )]
    chromadb_url: String,

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

    /// Force LLM inference (bypass static pattern matcher)
    #[arg(
        long,
        help = "Force LLM inference, bypassing the static pattern matcher"
    )]
    force_llm: bool,

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

    fn backend(&self) -> Option<String> {
        self.backend.clone()
    }

    fn model_name(&self) -> Option<String> {
        self.model_name.clone()
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

    fn force_llm(&self) -> bool {
        self.force_llm
    }
}

// =============================================================================
// Shell Integration
// =============================================================================

/// Exit code indicating edit mode - shell wrapper should capture command
pub const EXIT_CODE_EDIT: i32 = 201;

/// Copy text to system clipboard
/// Returns true if successful, false if clipboard is unavailable
fn copy_to_clipboard(text: &str) -> bool {
    use std::process::{Command, Stdio};

    #[cfg(target_os = "macos")]
    {
        if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                if stdin.write_all(text.as_bytes()).is_ok() {
                    return child.wait().map(|s| s.success()).unwrap_or(false);
                }
            }
        }
        false
    }

    #[cfg(target_os = "linux")]
    {
        // Try xclip first, then xsel
        for cmd in &["xclip", "xsel"] {
            let args: &[&str] = if *cmd == "xclip" {
                &["-selection", "clipboard"]
            } else {
                &["--clipboard", "--input"]
            };

            if let Ok(mut child) = Command::new(cmd).args(args).stdin(Stdio::piped()).spawn() {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    if stdin.write_all(text.as_bytes()).is_ok()
                        && child.wait().map(|s| s.success()).unwrap_or(false)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = text;
        false
    }
}

/// Print shell integration script for the specified shell
fn print_shell_init_script(shell: &str) {
    let script = match shell.to_lowercase().as_str() {
        "zsh" => {
            r#"# Caro shell integration for zsh
# Add to ~/.zshrc: eval "$(caro init zsh)"

caro() {
    local output exit_code
    local tmpfile=$(mktemp)

    # Run caro with wrapper flag, capture stdout only (stderr goes to terminal for display)
    CARO_WRAPPER=1 command caro "$@" > "$tmpfile"
    exit_code=$?
    output=$(cat "$tmpfile")
    rm -f "$tmpfile"

    if [[ $exit_code -eq 201 ]]; then
        # Edit mode: put command in buffer for user to edit
        print -z "$output"
    else
        # Normal mode: print stdout (display already shown via stderr)
        [[ -n "$output" ]] && echo "$output"
    fi
    return $exit_code
}
"#
        }
        "bash" => {
            r#"# Caro shell integration for bash
# Add to ~/.bashrc: eval "$(caro init bash)"

caro() {
    local output exit_code
    local tmpfile=$(mktemp)

    # Run caro with wrapper flag, capture stdout only (stderr goes to terminal for display)
    CARO_WRAPPER=1 command caro "$@" > "$tmpfile"
    exit_code=$?
    output=$(cat "$tmpfile")
    rm -f "$tmpfile"

    if [[ $exit_code -eq 201 ]]; then
        # Edit mode: use readline to pre-fill command
        # This requires bash 4.0+ with readline support
        read -e -i "$output" -p "" edited_cmd
        if [[ -n "$edited_cmd" ]]; then
            eval "$edited_cmd"
        fi
    else
        # Normal mode: print stdout (display already shown via stderr)
        [[ -n "$output" ]] && echo "$output"
    fi
    return $exit_code
}
"#
        }
        "fish" => {
            r#"# Caro shell integration for fish
# Add to ~/.config/fish/config.fish: caro init fish | source

function caro
    set -l tmpfile (mktemp)

    # Run caro with wrapper flag, capture stdout only (stderr goes to terminal for display)
    set -x CARO_WRAPPER 1
    command caro $argv > $tmpfile
    set -l exit_code $status
    set -l output (cat $tmpfile)
    rm -f $tmpfile
    set -e CARO_WRAPPER

    if test $exit_code -eq 201
        # Edit mode: put command in buffer
        commandline -r "$output"
    else
        # Normal mode: print stdout (display already shown via stderr)
        test -n "$output"; and echo "$output"
    end
    return $exit_code
end
"#
        }
        _ => {
            eprintln!(
                "Unsupported shell: {}. Supported shells: zsh, bash, fish",
                shell
            );
            std::process::exit(1);
        }
    };

    print!("{}", script);
}

// =============================================================================
// Assessment Command
// =============================================================================

/// Run assessment command with optional export
#[allow(dead_code)]
async fn run_assessment_command(
    export_format: Option<ExportFormat>,
    output_path: Option<String>,
) -> Result<(), String> {
    use caro::assessment::{formatters, AssessmentResult, Recommender, SystemProfile};

    let profile = SystemProfile::detect().map_err(|e| format!("Assessment failed: {}", e))?;

    let recommendations = Recommender::recommend(&profile);
    let warnings = vec![]; // Collect any warnings during detection

    let result = AssessmentResult::new(profile, recommendations, warnings);

    if let Some(format) = export_format {
        let content = match format {
            ExportFormat::Json => formatters::json::format(&result)
                .map_err(|e| format!("JSON serialization failed: {}", e))?,
            ExportFormat::Markdown => formatters::markdown::format(&result),
        };

        if let Some(path) = output_path {
            std::fs::write(&path, &content)
                .map_err(|e| format!("Failed to write to {}: {}", path, e))?;
            println!("Assessment exported to: {}", path);
        } else {
            println!("{}", content);
        }
    } else {
        // Default: human-readable format
        let formatted = formatters::human::format(&result);
        println!("{}", formatted);
    }

    Ok(())
}

// =============================================================================
// Knowledge Backend Configuration
// =============================================================================

/// Build knowledge backend configuration from CLI arguments
#[cfg(feature = "knowledge")]
fn build_knowledge_backend_config(
    knowledge_backend: Option<&str>,
    chromadb_url: &str,
) -> caro::models::KnowledgeBackendConfig {
    use caro::models::KnowledgeBackendConfig;

    match knowledge_backend {
        Some("chromadb") | Some("chroma") => {
            // Check for Chroma Cloud API key in environment
            let auth_token = std::env::var("CHROMA_API_KEY").ok();
            KnowledgeBackendConfig::chromadb(chromadb_url.to_string(), None, auth_token)
        }
        Some("lancedb") | Some("lance") | None => {
            // Default to LanceDB
            KnowledgeBackendConfig::lancedb(caro::knowledge::default_knowledge_path())
        }
        Some(other) => {
            eprintln!(
                "Warning: Unknown knowledge backend '{}'. Defaulting to LanceDB.",
                other
            );
            KnowledgeBackendConfig::lancedb(caro::knowledge::default_knowledge_path())
        }
    }
}

// =============================================================================
// Configuration Commands
// =============================================================================

/// Handle configuration subcommands
fn handle_config_command(command: ConfigCommands) -> Result<(), String> {
    use colored::Colorize;

    let config_manager =
        ConfigManager::new().map_err(|e| format!("Failed to access config: {}", e))?;

    match command {
        ConfigCommands::Set { key, value } => {
            let mut config = config_manager
                .load()
                .map_err(|e| format!("Failed to load config: {}", e))?;

            match key.to_lowercase().as_str() {
                "backend" => {
                    // Validate backend name
                    let valid_backends = ["embedded", "ollama", "exo", "vllm"];
                    if !valid_backends.contains(&value.to_lowercase().as_str()) {
                        return Err(format!(
                            "Invalid backend '{}'. Valid options: {}",
                            value,
                            valid_backends.join(", ")
                        ));
                    }
                    config.default_model = Some(value.to_lowercase());
                    println!(
                        "{} Set default backend to '{}'",
                        "✓".green(),
                        config.default_model.as_ref().unwrap()
                    );
                }
                "model-name" | "model_name" => {
                    config.model_name = Some(value.clone());
                    println!("{} Set model name to '{}'", "✓".green(), value);
                }
                "shell" => {
                    let shell: caro::models::ShellType = value
                        .parse()
                        .map_err(|e| format!("Invalid shell '{}': {}", value, e))?;
                    config.default_shell = Some(shell);
                    println!("{} Set default shell to '{:?}'", "✓".green(), shell);
                }
                "safety" => {
                    let level: caro::models::SafetyLevel = value
                        .parse()
                        .map_err(|e| format!("Invalid safety level '{}': {}", value, e))?;
                    config.safety_level = level;
                    println!("{} Set safety level to '{:?}'", "✓".green(), level);
                }
                _ => {
                    return Err(format!(
                        "Unknown config key '{}'. Valid keys: backend, model-name, shell, safety",
                        key
                    ));
                }
            }

            config_manager
                .save(&config)
                .map_err(|e| format!("Failed to save config: {}", e))?;

            println!(
                "{}",
                format!(
                    "Config saved to: {}",
                    config_manager.config_path().display()
                )
                .dimmed()
            );
        }
        ConfigCommands::Get { key } => {
            let config = config_manager
                .load()
                .map_err(|e| format!("Failed to load config: {}", e))?;

            match key.to_lowercase().as_str() {
                "backend" => {
                    let value = config.default_model.as_deref().unwrap_or("(auto-detect)");
                    println!("{}: {}", "backend".bold(), value);
                }
                "model-name" | "model_name" => {
                    let value = config.model_name.as_deref().unwrap_or("(default)");
                    println!("{}: {}", "model-name".bold(), value);
                }
                "shell" => {
                    let value = config
                        .default_shell
                        .map(|s| format!("{:?}", s))
                        .unwrap_or_else(|| "(auto-detect)".to_string());
                    println!("{}: {}", "shell".bold(), value);
                }
                "safety" => {
                    println!("{}: {:?}", "safety".bold(), config.safety_level);
                }
                _ => {
                    return Err(format!(
                        "Unknown config key '{}'. Valid keys: backend, model-name, shell, safety",
                        key
                    ));
                }
            }
        }
        ConfigCommands::Show => {
            let config = config_manager
                .load()
                .map_err(|e| format!("Failed to load config: {}", e))?;

            println!("{}", "Current Configuration:".bold());
            println!();
            println!(
                "  {}: {}",
                "backend".cyan(),
                config.default_model.as_deref().unwrap_or("(auto-detect)")
            );
            println!(
                "  {}: {}",
                "model-name".cyan(),
                config.model_name.as_deref().unwrap_or("(default)")
            );
            println!(
                "  {}: {}",
                "shell".cyan(),
                config
                    .default_shell
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_else(|| "(auto-detect)".to_string())
            );
            println!("  {}: {:?}", "safety".cyan(), config.safety_level);
            println!("  {}: {:?}", "log_level".cyan(), config.log_level);
            println!(
                "  {}: {} GB",
                "cache_max_size".cyan(),
                config.cache_max_size_gb
            );
            println!(
                "  {}: {} days",
                "log_rotation".cyan(),
                config.log_rotation_days
            );
            println!(
                "  {}: {}",
                "telemetry".cyan(),
                if config.telemetry.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            );
            println!();
            println!(
                "{}",
                format!("Config file: {}", config_manager.config_path().display()).dimmed()
            );
        }
        ConfigCommands::Reset => {
            let config = caro::models::UserConfiguration::default();
            config_manager
                .save(&config)
                .map_err(|e| format!("Failed to save config: {}", e))?;
            println!("{} Configuration reset to defaults", "✓".green());
        }
    }

    Ok(())
}

// =============================================================================
// Knowledge Index Commands
// =============================================================================

/// Handle knowledge subcommands
#[cfg(feature = "knowledge")]
async fn handle_knowledge_command(
    command: KnowledgeCommands,
    backend_config: caro::models::KnowledgeBackendConfig,
) -> Result<(), String> {
    use caro::knowledge::indexers::{help::HelpIndexer, man::ManPageIndexer, tldr::TldrIndexer};
    use caro::knowledge::{EntryType, Indexer, KnowledgeEntry, KnowledgeIndex};
    use colored::Colorize;

    match command {
        KnowledgeCommands::IndexMan {
            page,
            sections,
            verbose,
        } => {
            println!("{} Initializing man page indexer...", "►".cyan());

            // Create indexer
            let indexer = if let Some(sections) = sections {
                ManPageIndexer::new(sections)
            } else {
                ManPageIndexer::user_commands()
            };

            // Create backend
            let index = KnowledgeIndex::from_config(&backend_config)
                .await
                .map_err(|e| format!("Failed to initialize knowledge index: {}", e))?;

            let backend = index.backend();

            // Index
            if let Some(page_name) = page {
                println!("{} Indexing man page: {}", "→".cyan(), page_name.bold());
                match indexer.index_one(backend, &page_name).await {
                    Ok(true) => {
                        println!("{} Successfully indexed {}", "✓".green(), page_name.bold())
                    }
                    Ok(false) => println!("{} Man page not found: {}", "✗".red(), page_name),
                    Err(e) => return Err(format!("Indexing failed: {}", e)),
                }
            } else {
                println!("{} Indexing all man pages (section 1)...", "→".cyan());

                let progress = if verbose {
                    Some(Box::new(|current: usize, total: usize| {
                        print!("\r{} Indexed {}/{} pages", "→".cyan(), current, total);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    })
                        as Box<dyn Fn(usize, usize) + Send + Sync>)
                } else {
                    None
                };

                match indexer.index_all(backend, progress).await {
                    Ok(stats) => {
                        if verbose {
                            println!(); // Newline after progress
                        }
                        println!("{} Indexing complete!", "✓".green());
                        println!("  Successful: {}", stats.successful);
                        println!("  Failed: {}", stats.failed);
                        println!("  Skipped: {}", stats.skipped);
                    }
                    Err(e) => return Err(format!("Indexing failed: {}", e)),
                }
            }

            Ok(())
        }

        KnowledgeCommands::IndexTldr {
            command,
            platforms,
            verbose,
        } => {
            println!("{} Initializing tldr indexer...", "►".cyan());

            let indexer = if let Some(platforms) = platforms {
                TldrIndexer::new(None, platforms)
            } else {
                TldrIndexer::current_platform()
            };

            let index = KnowledgeIndex::from_config(&backend_config)
                .await
                .map_err(|e| format!("Failed to initialize knowledge index: {}", e))?;

            let backend = index.backend();

            if let Some(cmd) = command {
                println!("{} Indexing tldr page: {}", "→".cyan(), cmd.bold());
                match indexer.index_one(backend, &cmd).await {
                    Ok(true) => println!("{} Successfully indexed {}", "✓".green(), cmd.bold()),
                    Ok(false) => println!("{} Tldr page not found: {}", "✗".red(), cmd),
                    Err(e) => return Err(format!("Indexing failed: {}", e)),
                }
            } else {
                println!("{} Indexing all tldr pages...", "→".cyan());

                let progress = if verbose {
                    Some(Box::new(|current: usize, total: usize| {
                        print!("\r{} Indexed {}/{} pages", "→".cyan(), current, total);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    })
                        as Box<dyn Fn(usize, usize) + Send + Sync>)
                } else {
                    None
                };

                match indexer.index_all(backend, progress).await {
                    Ok(stats) => {
                        if verbose {
                            println!();
                        }
                        println!("{} Indexing complete!", "✓".green());
                        println!("  Successful: {}", stats.successful);
                        println!("  Failed: {}", stats.failed);
                        println!("  Skipped: {}", stats.skipped);
                    }
                    Err(e) => return Err(format!("Indexing failed: {}", e)),
                }
            }

            Ok(())
        }

        KnowledgeCommands::IndexHelp {
            command,
            commands,
            verbose,
        } => {
            println!("{} Initializing help indexer...", "►".cyan());

            let indexer = if let Some(commands) = commands {
                HelpIndexer::for_commands(commands)
            } else if let Some(cmd) = &command {
                HelpIndexer::for_commands(vec![cmd.clone()])
            } else {
                HelpIndexer::auto_discover()
            };

            let index = KnowledgeIndex::from_config(&backend_config)
                .await
                .map_err(|e| format!("Failed to initialize knowledge index: {}", e))?;

            let backend = index.backend();

            if let Some(cmd) = command {
                println!("{} Indexing --help output for: {}", "→".cyan(), cmd.bold());
                match indexer.index_one(backend, &cmd).await {
                    Ok(true) => println!("{} Successfully indexed {}", "✓".green(), cmd.bold()),
                    Ok(false) => println!("{} Help output not available: {}", "✗".red(), cmd),
                    Err(e) => return Err(format!("Indexing failed: {}", e)),
                }
            } else {
                println!("{} Indexing --help output...", "→".cyan());

                let progress = if verbose {
                    Some(Box::new(|current: usize, total: usize| {
                        print!("\r{} Indexed {}/{} commands", "→".cyan(), current, total);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    })
                        as Box<dyn Fn(usize, usize) + Send + Sync>)
                } else {
                    None
                };

                match indexer.index_all(backend, progress).await {
                    Ok(stats) => {
                        if verbose {
                            println!();
                        }
                        println!("{} Indexing complete!", "✓".green());
                        println!("  Successful: {}", stats.successful);
                        println!("  Failed: {}", stats.failed);
                        println!("  Skipped: {}", stats.skipped);
                    }
                    Err(e) => return Err(format!("Indexing failed: {}", e)),
                }
            }

            Ok(())
        }

        KnowledgeCommands::IndexGitHub { repo, verbose } => {
            use caro::knowledge::indexers::github::GitHubDocsIndexer;

            println!("{} Initializing GitHub docs indexer...", "►".cyan());

            let indexer = GitHubDocsIndexer::new()
                .map_err(|e| format!("Failed to create GitHub indexer: {}", e))?;

            let index = KnowledgeIndex::from_config(&backend_config)
                .await
                .map_err(|e| format!("Failed to initialize knowledge index: {}", e))?;

            let backend = index.backend();

            println!("{} Fetching README from: {}", "→".cyan(), repo.bold());

            if verbose {
                println!("{} Downloading and parsing documentation...", "→".cyan());
            }

            match indexer.index_one(backend, &repo).await {
                Ok(true) => {
                    println!("{} Successfully indexed GitHub repo: {}", "✓".green(), repo.bold());
                    println!("  Documentation added to knowledge base");
                }
                Ok(false) => {
                    println!("{} No useful documentation found for: {}", "✗".yellow(), repo);
                }
                Err(e) => return Err(format!("GitHub indexing failed: {}", e)),
            }

            Ok(())
        }

        KnowledgeCommands::Stats => {
            println!("{} Knowledge Index Statistics", "📊".cyan());
            println!();

            let index = KnowledgeIndex::from_config(&backend_config)
                .await
                .map_err(|e| format!("Failed to initialize knowledge index: {}", e))?;

            match index.stats().await {
                Ok(stats) => {
                    println!(
                        "  Total entries: {}",
                        stats.total_entries.to_string().bold()
                    );
                    println!("  Success count: {}", stats.success_count);
                    println!("  Correction count: {}", stats.correction_count);
                }
                Err(e) => return Err(format!("Failed to get stats: {}", e)),
            }

            Ok(())
        }

        KnowledgeCommands::Clear { force } => {
            if !force {
                print!(
                    "{} Are you sure you want to clear the knowledge index? (y/N) ",
                    "⚠".yellow()
                );
                use std::io::{self, Write};
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            println!("{} Clearing knowledge index...", "►".cyan());

            let index = KnowledgeIndex::from_config(&backend_config)
                .await
                .map_err(|e| format!("Failed to initialize knowledge index: {}", e))?;

            match index.clear().await {
                Ok(()) => println!("{} Knowledge index cleared successfully", "✓".green()),
                Err(e) => return Err(format!("Failed to clear index: {}", e)),
            }

            Ok(())
        }

        KnowledgeCommands::Search { query, limit } => {
            println!("{} Searching knowledge index...", "🔍".cyan());
            println!();

            let index = KnowledgeIndex::from_config(&backend_config)
                .await
                .map_err(|e| format!("Failed to initialize knowledge index: {}", e))?;

            match index.find_similar(&query, limit).await {
                Ok(results) => {
                    if results.is_empty() {
                        println!("{} No results found", "ℹ".yellow());
                    } else {
                        println!(
                            "{} Found {} result(s):",
                            "✓".green(),
                            results.len().to_string().bold()
                        );
                        println!();

                        for (i, entry) in results.iter().enumerate() {
                            println!("{}. {} (similarity: {:.2}%)",
                                (i + 1).to_string().bold(),
                                entry.command.bright_cyan(),
                                entry.similarity * 100.0
                            );
                            println!("   Request: {}", entry.request.dimmed());

                            if let Some(ref context) = entry.context {
                                println!("   Context: {}", context.dimmed());
                            }

                            if let Some(ref original) = entry.original_command {
                                println!("   Original: {}", original.dimmed());
                            }

                            if let Some(ref feedback) = entry.feedback {
                                println!("   Feedback: {}", feedback.dimmed());
                            }

                            println!();
                        }
                    }
                }
                Err(e) => return Err(format!("Failed to search: {}", e)),
            }

            Ok(())
        }

        KnowledgeCommands::Export { path } => {
            println!("{} Exporting knowledge index...", "📦".cyan());

            let index = KnowledgeIndex::from_config(&backend_config)
                .await
                .map_err(|e| format!("Failed to initialize knowledge index: {}", e))?;

            // Get all entries by searching with empty query
            let entries = index
                .find_similar("", 10000)
                .await
                .map_err(|e| format!("Failed to retrieve entries: {}", e))?;

            // Serialize to JSON
            let json = serde_json::to_string_pretty(&entries)
                .map_err(|e| format!("Failed to serialize entries: {}", e))?;

            // Write to file
            std::fs::write(&path, json)
                .map_err(|e| format!("Failed to write file: {}", e))?;

            println!(
                "{} Exported {} entries to {}",
                "✓".green(),
                entries.len().to_string().bold(),
                path.display().to_string().bright_cyan()
            );

            Ok(())
        }

        KnowledgeCommands::Import { path, merge } => {
            if !merge {
                print!(
                    "{} This will replace all existing knowledge. Continue? (y/N) ",
                    "⚠".yellow()
                );
                use std::io::{self, Write};
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            println!("{} Importing knowledge index...", "📥".cyan());

            // Read and parse JSON file
            let json = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read file: {}", e))?;

            let entries: Vec<KnowledgeEntry> = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let index = KnowledgeIndex::from_config(&backend_config)
                .await
                .map_err(|e| format!("Failed to initialize knowledge index: {}", e))?;

            // Clear existing if not merging
            if !merge {
                index
                    .clear()
                    .await
                    .map_err(|e| format!("Failed to clear index: {}", e))?;
            }

            // Import entries
            let mut imported = 0;
            for entry in &entries {
                let result = if entry.entry_type == EntryType::Correction {
                    index
                        .record_correction(
                            &entry.request,
                            entry.original_command.as_deref().unwrap_or(""),
                            &entry.command,
                            entry.feedback.as_deref(),
                        )
                        .await
                } else {
                    index
                        .record_success(&entry.request, &entry.command, entry.context.as_deref())
                        .await
                };

                match result {
                    Ok(_) => imported += 1,
                    Err(e) => {
                        eprintln!("{} Failed to import entry: {}", "⚠".yellow(), e);
                    }
                }
            }

            println!(
                "{} Imported {}/{} entries from {}",
                "✓".green(),
                imported.to_string().bold(),
                entries.len(),
                path.display().to_string().bright_cyan()
            );

            Ok(())
        }
    }
}

/// Handle profile subcommands
#[cfg(feature = "knowledge")]
async fn handle_profile_command(command: ProfileCommands) -> Result<(), String> {
    use caro::config::ConfigManager;
    use caro::models::profile::{ProfileConfig, UserProfile};
    use colored::Colorize;
    use std::io::{self, Write};

    let config_manager =
        ConfigManager::new().map_err(|e| format!("Failed to create config manager: {}", e))?;
    let config_dir = config_manager
        .config_path()
        .parent()
        .ok_or_else(|| "Invalid config path".to_string())?;
    let profile_path = config_dir.join("profiles.toml");

    let mut profile_config = if profile_path.exists() {
        let content = std::fs::read_to_string(&profile_path)
            .map_err(|e| format!("Failed to read profiles: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse profiles: {}", e))?
    } else {
        ProfileConfig::new()
    };

    match command {
        ProfileCommands::Create {
            name,
            profile_type,
            description,
        } => {
            println!("{} Creating profile: {}", "►".cyan(), name.bold());

            let mut profile = UserProfile::new(name.clone(), profile_type);
            if let Some(desc) = description {
                profile.description = Some(desc);
            }

            profile_config
                .add_profile(profile)
                .map_err(|e| e.to_string())?;

            let content = toml::to_string_pretty(&profile_config)
                .map_err(|e| format!("Failed to serialize profiles: {}", e))?;
            std::fs::create_dir_all(config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
            std::fs::write(&profile_path, content)
                .map_err(|e| format!("Failed to write profiles: {}", e))?;

            println!(
                "{} Profile created: {} ({})",
                "✓".green(),
                name.bold(),
                profile_type
            );
            Ok(())
        }

        ProfileCommands::List => {
            if profile_config.profiles.is_empty() {
                println!("{} No profiles found", "✗".yellow());
                println!("  Create a profile with: caro profile create <name>");
                return Ok(());
            }

            println!("{} User Profiles:", "►".cyan());
            println!();

            for profile in &profile_config.profiles {
                let active_marker = if Some(&profile.name) == profile_config.active_profile.as_ref()
                {
                    " (active)".green()
                } else {
                    "".normal()
                };

                println!("  {} {}{}", "●".cyan(), profile.name.bold(), active_marker);
                println!("    Type: {}", profile.profile_type);
                if let Some(desc) = &profile.description {
                    println!("    Description: {}", desc);
                }
                println!("    Created: {}", profile.created.format("%Y-%m-%d %H:%M"));
                if let Some(last_used) = profile.last_used {
                    println!("    Last used: {}", last_used.format("%Y-%m-%d %H:%M"));
                }
                println!("    Commands: {}", profile.command_count);
                println!();
            }

            Ok(())
        }

        ProfileCommands::Switch { name } => {
            println!("{} Switching to profile: {}", "►".cyan(), name.bold());

            profile_config
                .switch_profile(&name)
                .map_err(|e| e.to_string())?;

            let content = toml::to_string_pretty(&profile_config)
                .map_err(|e| format!("Failed to serialize profiles: {}", e))?;
            std::fs::write(&profile_path, content)
                .map_err(|e| format!("Failed to write profiles: {}", e))?;

            println!("{} Switched to profile: {}", "✓".green(), name.bold());
            Ok(())
        }

        ProfileCommands::Delete { name, force } => {
            if !force {
                print!("{} Delete profile '{}'? [y/N]: ", "?".yellow(), name);
                io::stdout().flush().ok();

                let mut response = String::new();
                io::stdin().read_line(&mut response).ok();

                if !response.trim().eq_ignore_ascii_case("y") {
                    println!("{} Deletion cancelled", "✗".yellow());
                    return Ok(());
                }
            }

            profile_config
                .remove_profile(&name)
                .map_err(|e| e.to_string())?;

            let content = toml::to_string_pretty(&profile_config)
                .map_err(|e| format!("Failed to serialize profiles: {}", e))?;
            std::fs::write(&profile_path, content)
                .map_err(|e| format!("Failed to write profiles: {}", e))?;

            println!("{} Profile deleted: {}", "✓".green(), name.bold());
            Ok(())
        }

        ProfileCommands::Show => {
            if let Some(active_name) = &profile_config.active_profile {
                if let Some(profile) = profile_config.get_active() {
                    println!("{} Active Profile: {}", "►".cyan(), active_name.bold());
                    println!();
                    println!("  Type: {}", profile.profile_type);
                    if let Some(desc) = &profile.description {
                        println!("  Description: {}", desc);
                    }
                    println!("  Created: {}", profile.created.format("%Y-%m-%d %H:%M"));
                    if let Some(last_used) = profile.last_used {
                        println!("  Last used: {}", last_used.format("%Y-%m-%d %H:%M"));
                    }
                    println!("  Commands: {}", profile.command_count);
                } else {
                    println!("{} Active profile not found: {}", "✗".red(), active_name);
                }
            } else {
                println!("{} No active profile", "✗".yellow());
                println!("  Switch to a profile with: caro profile switch <name>");
            }
            Ok(())
        }
    }
}

// =============================================================================
// Evaluation Tests
// =============================================================================

/// Run evaluation tests on command generation
async fn run_evaluation_tests(
    backend_name: &str,
    _verbose: bool,
    suite_path: Option<&str>,
    profile_id: Option<&str>,
) -> Result<(), String> {
    println!("Running evaluation tests with backend: {}", backend_name);
    println!();

    // Create backend (boxed to allow different types)
    let backend: Box<dyn CommandGenerator> = match backend_name {
        "static" => {
            let profile = CapabilityProfile::detect_or_cached().await;
            Box::new(StaticMatcher::new(profile))
        }
        "embedded" => Box::new(
            EmbeddedModelBackend::new()
                .map_err(|e| format!("Failed to create embedded backend: {}", e))?,
        ),
        _ => {
            return Err(format!(
                "Unknown backend: {}. Supported: static, embedded",
                backend_name
            ));
        }
    };

    // Load test suite
    let mut suite = if let Some(path) = suite_path {
        println!("Loading test suite from: {}", path);
        EvalSuite::from_yaml(path)
            .map_err(|e| format!("Failed to load test suite from {}: {}", path, e))?
    } else {
        EvalSuite::default_suite()
    };

    // Filter by profile if specified
    if let Some(profile) = profile_id {
        println!("Filtering tests for profile: {}", profile);
        suite = suite.filter_by_profile(profile);
    }

    println!("Loaded test suite: {}", suite.name);
    println!("Description: {}", suite.description);
    println!("Total test cases: {}", suite.test_cases.len());
    println!();

    // Run tests
    let mut results = EvalResults {
        suite_name: suite.name.clone(),
        backend: backend_name.to_string(),
        total_cases: suite.test_cases.len(),
        passed: 0,
        failed: 0,
        results_by_category: HashMap::new(),
        individual_results: Vec::new(),
    };

    for test_case in &suite.test_cases {
        let request = CommandRequest::new(&test_case.input, ShellType::Bash);

        let result = backend.generate_command(&request).await;

        let (passed, actual, error) = match result {
            Ok(cmd) => {
                let matches = test_case.expected_outputs.contains(&cmd.command);
                (matches, Some(cmd.command), None)
            }
            Err(e) => (false, None, Some(e.to_string())),
        };

        if passed {
            results.passed += 1;
        } else {
            results.failed += 1;
        }

        results.individual_results.push(IndividualResult {
            input: test_case.input.clone(),
            expected: test_case.expected_outputs.clone(),
            actual,
            passed,
            category: test_case.category,
            error,
        });

        // Update category stats
        let category_key = format!("{}", test_case.category);
        let cat_stats =
            results
                .results_by_category
                .entry(category_key)
                .or_insert(CategoryResults {
                    total: 0,
                    passed: 0,
                    pass_rate: 0.0,
                });
        cat_stats.total += 1;
        if passed {
            cat_stats.passed += 1;
        }
    }

    // Calculate pass rates
    for cat_stats in results.results_by_category.values_mut() {
        cat_stats.pass_rate = if cat_stats.total > 0 {
            (cat_stats.passed as f64 / cat_stats.total as f64) * 100.0
        } else {
            0.0
        };
    }

    // Print results
    results.print_summary();

    // Exit with error if tests failed
    if results.failed > 0 {
        Err(format!("{} tests failed", results.failed))
    } else {
        Ok(())
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

    let mut cli = Cli::parse();

    // Handle subcommands first
    match cli.command {
        Some(Commands::Doctor) => match caro::doctor::run_diagnostics().await {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error running diagnostics: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Config { command }) => match handle_config_command(command) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        #[cfg(feature = "knowledge")]
        Some(Commands::Knowledge { command }) => {
            let backend_config =
                build_knowledge_backend_config(cli.knowledge_backend.as_deref(), &cli.chromadb_url);

            match handle_knowledge_command(command, backend_config).await {
                Ok(()) => process::exit(0),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        #[cfg(feature = "knowledge")]
        Some(Commands::Profile { command }) => match handle_profile_command(command).await {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Init { shell }) => {
            print_shell_init_script(&shell);
            process::exit(0);
        }
        // NOTE: Assess subcommand disabled in v1.1.0-beta.1
        // Some(Commands::Assess { export, output }) => {
        //     match run_assessment_command(export, output).await {
        //         Ok(()) => process::exit(0),
        //         Err(e) => {
        //             eprintln!("Error running assessment: {}", e);
        //             process::exit(1);
        //         }
        //     }
        // }
        Some(Commands::Test {
            backend,
            verbose,
            suite,
            profile,
        }) => {
            match run_evaluation_tests(&backend, verbose, suite.as_deref(), profile.as_deref())
                .await
            {
                Ok(()) => process::exit(0),
                Err(e) => {
                    eprintln!("Error running tests: {}", e);
                    process::exit(1);
                }
            }
        }
        Some(Commands::Completion { shell }) => {
            use std::str::FromStr;
            let shell_type = caro::ShellType::from_str(&shell).unwrap_or(caro::ShellType::Bash);
            let script = caro::generate_completions(shell_type);
            print!("{}", script);
            process::exit(0);
        }
        Some(Commands::Suggest { query, limit }) => {
            let suggestions = caro::suggest_commands(&query, limit);
            if suggestions.is_empty() {
                eprintln!("No suggestions found for '{}'", query);
                process::exit(1);
            }
            for s in suggestions {
                println!("{}", s.description);
                println!("  {}", s.command);
                println!();
            }
            process::exit(0);
        }
        // NOTE: Telemetry subcommand disabled in v1.1.0-beta.1
        // Some(Commands::Telemetry { command }) => {
        //     let storage_path = dirs::data_dir()
        //         .unwrap_or_else(|| std::env::current_dir().unwrap())
        //         .join("caro")
        //         .join("telemetry")
        //         .join("events.db");
        //
        //     match caro::cli::telemetry::handle_telemetry(command, storage_path).await {
        //         Ok(()) => process::exit(0),
        //         Err(e) => {
        //             eprintln!("Error: {}", e);
        //             process::exit(1);
        //         }
        //     }
        // }
        None => {
            // Continue to regular command generation
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

    // Initialize telemetry
    let telemetry_storage_path = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("caro")
        .join("telemetry")
        .join("events.db");

    // Load config to get telemetry settings
    // Keep ConfigManager reference to save changes later
    let config_manager = ConfigManager::new().ok();
    let mut user_config = config_manager
        .as_ref()
        .and_then(|cm| cm.load().ok())
        .unwrap_or_default();

    // Check for first-run consent
    // Skip interactive consent for non-human output formats (json, yaml)
    let is_interactive_output = cli
        .output
        .as_deref()
        .is_none_or(|format| format != "json" && format != "yaml");

    if user_config.telemetry.first_run && is_interactive_output {
        // Prompt user for consent
        let consent = caro::telemetry::consent::prompt_consent();

        // Update config with consent result
        user_config.telemetry.first_run = false;
        user_config.telemetry.enabled = consent;

        // Show confirmation message
        if consent {
            caro::telemetry::consent::show_enabled_message();
        } else {
            caro::telemetry::consent::show_disabled_message();
        }

        // Persist config to disk
        if let Some(ref cm) = config_manager {
            if let Err(e) = cm.save(&user_config) {
                tracing::warn!("Failed to save telemetry preferences: {}", e);
            }
        }
    } else if user_config.telemetry.first_run && !is_interactive_output {
        // Non-interactive mode (JSON/YAML output): use default setting without prompting
        // Mark first_run as false to prevent future prompts
        user_config.telemetry.first_run = false;

        // Save the updated config silently
        if let Some(ref cm) = config_manager {
            let _ = cm.save(&user_config);
        }
    }

    // Create telemetry storage and collector (optional, don't fail if it errors)
    if let Ok(telemetry_storage) = caro::TelemetryStorage::new(telemetry_storage_path) {
        let telemetry_storage = std::sync::Arc::new(telemetry_storage);
        let telemetry_collector = std::sync::Arc::new(caro::TelemetryCollector::new(
            telemetry_storage.clone(),
            user_config.telemetry.enabled,
        ));

        // Set as global collector for easy access from all components
        caro::set_global_collector(telemetry_collector.clone());

        // Start telemetry uploader if enabled and not air-gapped
        if user_config.telemetry.enabled && !user_config.telemetry.air_gapped {
            let uploader = std::sync::Arc::new(caro::telemetry::uploader::TelemetryUploader::new(
                telemetry_storage.clone(),
                user_config.telemetry.clone(),
            ));
            uploader.start();
        }

        // Emit SessionStart event
        let backend_available: Vec<String> = vec!["static".to_string(), "embedded".to_string()];

        telemetry_collector.emit(caro::TelemetryEventType::SessionStart {
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            shell_type: user_config
                .default_shell
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "unknown".to_string()),
            backend_available,
        });
    } else {
        tracing::warn!("Telemetry initialization failed, continuing without telemetry");
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
        Ok(was_blocked) => {
            // Exit with code 1 if command was blocked by safety validation
            // Exit with code 0 for successful or safe commands
            process::exit(if was_blocked { 1 } else { 0 })
        }
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

async fn run_cli(cli: &Cli) -> Result<bool, CliError> {
    // Create CLI application with optional backend and model overrides
    let app = CliApp::with_overrides(
        caro::cli::CliConfig::default(),
        cli.backend.clone(),
        cli.model_name.clone(),
        cli.force_llm,
    )
    .await?;

    // Run command generation
    let mut result = app.run_with_args(cli.clone()).await?;

    // Check if command was blocked by safety validation
    let was_blocked = result.blocked_reason.is_some();

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

    Ok(was_blocked)
}

async fn print_plain_output(result: &mut caro::cli::CliResult, cli: &Cli) -> Result<(), CliError> {
    use colored::Colorize;
    use std::io::IsTerminal;

    // When running through shell wrapper, use stderr for display output
    // so stdout is reserved for the command in edit mode
    let in_wrapper = std::env::var("CARO_WRAPPER").is_ok();

    // Helper macro to print to stderr when in wrapper mode
    macro_rules! display {
        ($($arg:tt)*) => {
            if in_wrapper {
                eprintln!($($arg)*);
            } else {
                println!($($arg)*);
            }
        };
    }

    // Print warnings first
    for warning in &result.warnings {
        eprintln!("{} {}", "Warning:".yellow().bold(), warning);
    }

    // Handle blocked commands
    if let Some(blocked_reason) = &result.blocked_reason {
        eprintln!("{} {}", "Blocked:".red().bold(), blocked_reason);
        std::process::exit(1);
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
                display!("{}", "Operation cancelled by user.".yellow());
                std::process::exit(1);
            }

            display!("{}", "✓ Confirmed. Command is safe to execute.".green());
        } else {
            // Non-interactive environment - show confirmation message and exit
            display!("{}", result.confirmation_prompt.yellow());
            display!("{}", "Use --confirm/-y flag to auto-confirm dangerous commands in non-interactive environments.".dimmed());
            std::process::exit(1);
        }
    }

    // Print the main command
    display!("{}", "Command:".bold());
    display!("  {}", result.generated_command.bright_cyan().bold());
    display!("");

    // Print explanation only in verbose mode
    if cli.verbose && !result.explanation.is_empty() {
        display!("{}", "Explanation:".bold());
        display!("  {}", result.explanation);
        display!("");
    }

    // Handle dry-run mode
    if cli.dry_run {
        display!("{}", "Dry Run Mode:".bold().cyan());
        display!(
            "  The command would be executed with shell: {:?}",
            result.shell_used
        );
        if result.blocked_reason.is_some() || result.requires_confirmation {
            display!(
                "  {} This command would be blocked or require confirmation",
                "⚠".yellow()
            );
        } else {
            display!("  {} This command would execute successfully", "✓".green());
        }
        display!("");
    }
    // If command wasn't executed yet and passes safety checks, ask user if they want to execute
    else if result.exit_code.is_none() && result.executed && !cli.execute && !cli.interactive {
        use dialoguer::Select;

        // Check if we're in a terminal environment
        if std::io::stdin().is_terminal() {
            let options = &["Yes - execute", "No - skip", "Edit - modify in shell"];
            let selection = Select::new()
                .with_prompt("Execute this command?")
                .items(options)
                .default(1) // Default to "No"
                .interact()
                .map_err(|e| CliError::Internal {
                    message: format!("Failed to get user selection: {}", e),
                })?;

            match selection {
                0 => {
                    // Yes - execute
                    display!("");
                    display!("{}", "Executing command...".dimmed());

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
                    display!("");
                }
                2 => {
                    // Edit mode
                    if in_wrapper {
                        // Running through shell wrapper - output command to stdout and exit with code 201
                        // The wrapper will capture stdout and put it in the readline buffer
                        println!("{}", result.generated_command);
                        std::process::exit(EXIT_CODE_EDIT);
                    } else {
                        // Not running through wrapper - copy to clipboard as fallback
                        let cmd = &result.generated_command;
                        if copy_to_clipboard(cmd) {
                            println!(
                                "{} Command copied to clipboard. Paste with {} to edit.",
                                "✓".green(),
                                if cfg!(target_os = "macos") {
                                    "Cmd+V"
                                } else {
                                    "Ctrl+V"
                                }
                            );
                        } else {
                            // Clipboard copy failed - just print the command
                            println!("{}", "Command (copy manually):".yellow());
                            println!("  {}", cmd);
                        }
                        println!();
                        println!(
                            "{}",
                            "Tip: Add shell integration for seamless editing:".dimmed()
                        );
                        println!("  {}", "eval \"$(caro init zsh)\"  # or bash/fish".dimmed());
                        println!();
                    }
                }
                _ => {
                    // No - skip
                    display!("{}", "Execution skipped.".yellow());
                    display!("");
                }
            }
        } else {
            // Non-interactive environment - show message
            display!(
                "{}",
                "Use --execute/-x flag to auto-execute commands in non-interactive environments."
                    .dimmed()
            );
            display!("");
        }
    }

    // Print execution results if command was actually executed
    if result.exit_code.is_some() {
        display!("{}", "Execution Results:".bold().green());

        // Print exit code
        if let Some(exit_code) = result.exit_code {
            let status_msg = if exit_code == 0 {
                format!("✓ Success (exit code: {})", exit_code).green()
            } else {
                format!("✗ Failed (exit code: {})", exit_code).red()
            };
            display!("  {}", status_msg);
        }

        // Print execution time
        if result.timing_info.execution_time_ms > 0 {
            display!(
                "  Execution time: {}ms",
                result.timing_info.execution_time_ms
            );
        }

        // Print stdout if present
        if let Some(stdout) = &result.stdout {
            if !stdout.trim().is_empty() {
                display!("");
                display!("{}", "Standard Output:".bold());
                for line in stdout.lines() {
                    display!("  {}", line);
                }
            }
        }

        // Print stderr if present
        if let Some(stderr) = &result.stderr {
            if !stderr.trim().is_empty() {
                display!("");
                display!("{}", "Standard Error:".bold().yellow());
                for line in stderr.lines() {
                    display!("  {}", line.yellow());
                }
            }
        }

        // Print execution error if present
        if let Some(error) = &result.execution_error {
            display!("");
            display!("{} {}", "Execution Error:".red().bold(), error.red());
        }

        display!("");
    } else if cli.execute || cli.interactive {
        // User requested execution but it didn't happen
        display!(
            "{}",
            "Command was not executed (blocked by safety checks or user cancelled).".yellow()
        );
        display!("");
    }

    // Print alternatives if available
    if !result.alternatives.is_empty() {
        display!("{}", "Alternatives:".bold());
        for alt in &result.alternatives {
            display!("  • {}", alt.dimmed());
        }
        display!("");
    }

    // Print debug information if verbose
    if let Some(debug_info) = &result.debug_info {
        display!("{}", "Debug Info:".dimmed());
        display!("  {}", debug_info.dimmed());
    }

    if !result.generation_details.is_empty() {
        display!("  {}", result.generation_details.dimmed());
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

    #[test]
    fn test_quoted_trailing_args_backward_compat() {
        // Shell passes quoted args as single string (e.g., caro "list files" → ["list files"])
        // Verify backward compatibility with quoted prompts
        let resolved = resolve_prompt(None, None, vec!["list files".into()]);
        assert_eq!(resolved.text, "list files");
        assert_eq!(resolved.source, PromptSource::TrailingArgs);
    }

    #[test]
    fn test_single_word_trailing_arg() {
        // Single-word prompts (e.g., caro version → ["version"])
        let resolved = resolve_prompt(None, None, vec!["version".into()]);
        assert_eq!(resolved.text, "version");
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
