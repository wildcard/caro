use caro::backends::embedded::EmbeddedModelBackend;
use caro::backends::{CommandGenerator, StaticMatcher};
use caro::cli::{CliApp, CliError, IntoCliArgs};
use caro::config::ConfigManager;
use caro::eval::{CategoryResults, EvalResults, EvalSuite, IndividualResult};
use caro::models::{CommandRequest, ShellType};
use caro::prompts::CapabilityProfile;
use caro::setup::{needs_setup, SetupWizard};
use clap::Parser;
use std::collections::HashMap;
use std::io::IsTerminal;
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
    /// Show warning but continue (serious issues that may produce poor results)
    Warning { message: String },
    /// Show hint but continue (minor issues, only in verbose mode)
    Hint { message: String },
    /// Proceed with the prompt (valid content provided)
    ProceedWithPrompt,
}

/// Validate a prompt and determine the appropriate action
///
/// Checks for common issues that may produce poor results:
/// - Empty/whitespace-only prompts → ShowHelp
/// - Prompts with only flags/operators → Warning
/// - Very short/ambiguous prompts → Hint
/// - Valid prompts → ProceedWithPrompt
///
/// # Arguments
/// * `prompt` - The prompt text to validate
///
/// # Returns
/// ValidationAction indicating what to do with the prompt
pub fn validate_prompt(prompt: &str) -> ValidationAction {
    const SHELL_OPERATORS: &[&str] = &[">", "|", "<", ">>", "2>", "&", ";"];

    let trimmed = prompt.trim();

    // Empty or whitespace-only prompts
    if trimmed.is_empty() {
        return ValidationAction::ShowHelp;
    }

    // Check if prompt contains only flags or operators
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    let has_content_words = words
        .iter()
        .any(|word| !word.starts_with('-') && !SHELL_OPERATORS.contains(word));

    if !has_content_words {
        return ValidationAction::Warning {
            message:
                "⚠️  Warning: No command description found in your query. \
                     Try describing what you want to do (e.g., 'list files' instead of just flags)."
                    .to_string(),
        };
    }

    // Check for very short prompts (less than 3 characters)
    if trimmed.len() < 3 {
        return ValidationAction::Hint {
            message: "💡 Hint: Your query is very short. Consider being more specific \
                     about what you want to do."
                .to_string(),
        };
    }

    // Check for single-word prompts (may be too ambiguous)
    if words.len() == 1 && trimmed.len() < 8 {
        return ValidationAction::Hint {
            message: format!(
                "💡 Hint: Single word '{}' may be ambiguous. Try adding more details \
                 like 'list files' or 'show processes'.",
                trimmed
            ),
        };
    }

    ValidationAction::ProceedWithPrompt
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

/// `caro skill` subcommands.
#[derive(Parser, Clone)]
enum SkillSubcommand {
    /// Install the bundled `caro-scaffold` skill into `~/.claude/skills/`.
    Install,
    /// Remove the installed `caro-scaffold` skill.
    Uninstall,
}

/// Subcommands for caro
#[derive(Parser, Clone)]
enum Commands {
    /// Run system diagnostics and health checks
    Doctor,

    /// Generate shell integration script for edit mode support
    Integration {
        /// Shell to generate init script for (zsh, bash, fish)
        shell: String,
    },

    /// Run the interactive setup wizard to configure caro
    Init {
        /// Use minimal ASCII art banner (for smaller terminals)
        #[arg(long, help = "Use minimal banner for smaller terminals")]
        minimal: bool,

        /// Force re-run setup even if already configured
        #[arg(short, long, help = "Force setup even if already configured")]
        force: bool,
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

    /// Generate an interactive AI command via conversational session (Atuin-AI-style).
    ///
    /// By default, a recent session is resumed (see `ai.session_continue_minutes` in config).
    /// Stdout is the generated command only, so shell widgets can inject it straight
    /// into the prompt buffer.
    Ai {
        /// Force a new session instead of resuming the most recent one.
        #[arg(long)]
        new_session: bool,

        /// Resume a recent session if available (default behavior). Kept as an
        /// explicit flag so shell hooks can be unambiguous.
        #[arg(long)]
        continue_session: bool,

        /// Run one turn and return — no TTY REPL. The only mode supported today.
        #[arg(long)]
        once: bool,

        /// Natural-language prompt. If omitted, stdin is read.
        #[arg(trailing_var_arg = true, num_args = 0..)]
        prompt: Vec<String>,
    },

    /// Emit shell integration script with an optional `?` AI keybinding.
    ///
    /// Add to your shell rc: `eval "$(caro shell-init bash)"` / similar for zsh, fish.
    ShellInit {
        /// Shell to emit integration for (bash, zsh, fish).
        shell: String,

        /// Skip the `?` AI keybinding even if `ai.enabled` is true in config.
        #[arg(long)]
        disable_ai: bool,
    },

    /// Validate a CaroML (`.caro`) task file: parse, lint, report errors with line numbers.
    ///
    /// No LLM calls, no execution, no regen. Suitable for editor integration and CI.
    Check {
        /// Path to a `.caro` file.
        file: std::path::PathBuf,
    },

    /// List CaroML tasks discovered in `./tasks/` and `~/.caro/library/`.
    ///
    /// By default lists both project and global tasks (project shadows global by name).
    List {
        /// Show only global library tasks (`~/.caro/library/`).
        #[arg(long)]
        global: bool,
    },

    /// List jobs declared in the project's Carofile, if present.
    ///
    /// Carofile recognized at `./Carofile` or `./Carofile.caro`. No execution.
    Jobs,

    /// Scaffold a starter `.caro` task file from a template.
    ///
    /// Writes to `./tasks/<name>.caro`. Creates `./tasks/` if missing.
    /// Refuses to overwrite an existing file. No LLM in v0.1 — just a
    /// fill-in-the-blanks template; LLM-assisted scaffolding lands in PR 4.
    New {
        /// Task name (becomes `tasks/<name>.caro`); may include `/` for subdirectories.
        name: String,
    },

    /// Generate or refresh a `.caro.lock` from a `.caro` task file.
    ///
    /// Calls the configured backend per step, runs the validator chain,
    /// and produces per-platform variants. Writes `tasks/<name>.caro.lock`
    /// atomically. With `--platform`, generates only that platform; without,
    /// uses the platforms declared by the task's `ON` pragmas (or the
    /// current platform as fallback).
    Generate {
        /// Task name (resolved via `tasks/<name>.caro` or `~/.caro/library/<name>.caro`).
        name: String,

        /// Restrict generation to one platform: `macos` / `linux` / `windows` / `posix`.
        #[arg(long)]
        platform: Option<String>,

        /// Override the backend (e.g. `mock`); defaults to the configured backend.
        #[arg(long)]
        backend: Option<String>,
    },

    /// Execute a CaroML task on the current platform.
    ///
    /// Reads the lock, picks the active variant for the current platform,
    /// prints the plan, asks for confirmation (unless `-y`), then executes.
    /// Stops on first non-zero exit.
    Run {
        /// Task name to run.
        name: String,

        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,

        /// Override platform.
        #[arg(long)]
        platform: Option<String>,

        /// Print the plan and exit without executing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Write the per-platform `.<platform>.sh` runbook from the active
    /// variant in the lock. The committed runbook lets non-Caro users run
    /// the task with plain `bash`.
    Export {
        /// Task name.
        name: String,

        /// Platform to export (default: current).
        #[arg(long)]
        platform: Option<String>,

        /// Output path. If unset, writes to `tasks/<name>.<platform>.sh`.
        #[arg(short = 'o', long)]
        output: Option<std::path::PathBuf>,
    },

    /// Generate an A/B challenger variant alongside the existing active.
    ///
    /// The challenger is added with `active = false`. Use `caro adopt` to
    /// promote it once you've reviewed it.
    Experiment {
        /// Task name.
        name: String,

        /// Platform to experiment on (default: current).
        #[arg(long)]
        platform: Option<String>,

        /// Backend to use for the new variant (default: `mock` in v0.1).
        #[arg(long)]
        backend: Option<String>,
    },

    /// Promote a challenger variant to active for its platform.
    ///
    /// The previously-active variant for that platform is retired
    /// (set to `active = false`, `retired_at = now`) but kept in the lock
    /// for reference.
    Adopt {
        /// Task name.
        name: String,

        /// Variant generation_id to promote (e.g. `gen_2026-04-26_macos_b`).
        #[arg(long)]
        variant: String,
    },

    /// Show the generation lineage from the lock plus a per-variant
    /// run-summary derived from the local journal.
    History {
        /// Task name.
        name: String,
    },

    /// Explain the RegenEvaluator's decision for the next `caro run`.
    Why {
        /// Task name.
        name: String,
    },

    /// Run a Carofile JOB, an external-alias, or a bare task.
    ///
    /// Resolution order:
    /// 1. JOB matching `<name>` in `./Carofile` / `./Carofile.caro`
    /// 2. USE alias matching `<name>` (external command or native task)
    /// 3. Fallback: `caro run <name>` (treats `<name>` as a bare task name)
    Do {
        /// Job, alias, or task name.
        name: String,

        /// Print the resolved dispatch plan and exit.
        #[arg(long)]
        dry_run: bool,
    },

    /// Render a CaroML task as Markdown documentation.
    Render {
        /// Task name.
        name: String,

        /// Output path. If unset, prints to stdout.
        #[arg(short = 'o', long)]
        output: Option<std::path::PathBuf>,
    },

    /// Manage Caro's bundled coder-agent skill.
    Skill {
        #[command(subcommand)]
        command: SkillSubcommand,
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

    /// Enable explanation mode with detailed command explanations
    #[arg(
        long,
        help = "Enable explanation mode: shows detailed breakdowns of commands and options"
    )]
    explain: bool,

    /// Suppress timing / progress output (show only the command and safety result)
    #[arg(
        short = 'q',
        long,
        help = "Suppress timing and progress output (show only command + safety result)"
    )]
    quiet: bool,

    /// Disable telemetry for this invocation only (does not modify config)
    #[arg(
        long = "no-telemetry",
        help = "Disable telemetry for this invocation only (session-scoped override)"
    )]
    no_telemetry: bool,

    /// Print available backends with their status and exit
    #[arg(
        long = "backend-info",
        help = "List available inference backends with their status and exit"
    )]
    backend_info: bool,

    /// Aggression level for directory context gathering
    ///
    /// `minimal` cuts prompt tokens (project type + git only).
    /// `normal` is the default (full inventory of build tools, scripts, etc.).
    /// `aggressive` adds a bounded code-signature scan on top of `normal`.
    ///
    /// Pattern idea-borrowed from rtk-ai/rtk's `rtk read -l <level>`.
    #[arg(
        long = "context-level",
        help = "Directory context aggression: minimal|normal|aggressive (default: normal)"
    )]
    context_level: Option<String>,

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

    fn explain(&self) -> bool {
        self.explain
    }

    fn quiet(&self) -> bool {
        self.quiet
    }

    fn no_telemetry(&self) -> bool {
        self.no_telemetry
    }

    fn backend_info(&self) -> bool {
        self.backend_info
    }

    fn context_level(&self) -> Option<String> {
        self.context_level.clone()
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
// AI shell-init + `caro ai` handlers
// =============================================================================

/// Emit the `caro shell-init <shell>` script (with optional `?` AI keybinding).
fn handle_shell_init(shell: &str, disable_ai: bool) -> Result<(), String> {
    use caro::ai::shell_init::{render, SupportedShell};

    let Some(sh) = SupportedShell::parse(shell) else {
        return Err(format!(
            "unsupported shell: {}. Supported: bash, zsh, fish",
            shell
        ));
    };

    // Peek at the user's AI-enabled flag without requiring a full CliApp bring-up.
    let ai_enabled = caro::config::ConfigManager::new()
        .and_then(|m| m.load())
        .map(|c| c.ai.enabled)
        .unwrap_or(true);

    print!("{}", render(sh, ai_enabled, disable_ai));
    Ok(())
}

/// One-shot execution of `caro ai` — generate a command, run safety, persist session.
///
/// stdout is the command text only (so shell widgets can inject it directly).
/// Status, warnings, and errors go to stderr.
async fn run_ai_once(cli: &Cli, new_session: bool, trailing: Vec<String>) -> Result<(), String> {
    use caro::ai::{run_once, AiInvocation};
    use caro::backends::CommandGenerator;
    use caro::context::ExecutionContext;
    use caro::models::{SafetyLevel, ShellType};
    use std::str::FromStr;
    use std::sync::Arc;

    // Resolve prompt (flag > stdin > trailing).
    let stdin_text = if is_stdin_available() {
        read_stdin().ok().filter(|s| !s.is_empty())
    } else {
        None
    };
    let resolved = resolve_prompt(cli.prompt.clone(), stdin_text, trailing);
    if resolved.text.trim().is_empty() {
        return Err("no prompt provided (pass text, pipe stdin, or use -p)".into());
    }

    // Load config — defaults are fine for a first-time user.
    let cfg_mgr = caro::config::ConfigManager::new().map_err(|e| format!("config: {}", e))?;
    let user_cfg = cfg_mgr.load().map_err(|e| format!("config: {}", e))?;

    if !user_cfg.ai.enabled {
        return Err("AI feature is disabled (set [ai] enabled = true in config).".into());
    }

    // Build a backend via the normal CLI path so the feature respects --backend, env var, config.
    let mut cli_app = caro::cli::CliApp::with_overrides(
        caro::cli::CliConfig::default(),
        cli.backend.clone(),
        cli.model_name.clone(),
        cli.force_llm,
    )
    .await
    .map_err(|e| format!("initializing backend: {}", e))?;
    if let Some(ref lvl) = cli.context_level {
        cli_app
            .set_context_level(lvl)
            .map_err(|e| format!("context-level: {}", e))?;
    }
    let backend: Arc<dyn CommandGenerator> = cli_app.backend_arc();
    // Derive the backend name from the *actually constructed* backend so the
    // off-host privacy warning is accurate even when auto-detection picks a
    // different backend than the config/CLI hint suggested. The string must
    // match the lowercase identifiers `privacy::may_leak_context_offhost`
    // checks for (`ollama`, `vllm`, `exo`, `claude`, `embedded`, ...).
    let backend_name = match backend.backend_info().backend_type {
        caro::models::BackendType::Embedded => "embedded".to_string(),
        caro::models::BackendType::Ollama => "ollama".to_string(),
        caro::models::BackendType::VLlm => "vllm".to_string(),
        caro::models::BackendType::Exo => "exo".to_string(),
        caro::models::BackendType::Claude => "claude".to_string(),
        caro::models::BackendType::Mlx => "mlx".to_string(),
        caro::models::BackendType::Mock => "mock".to_string(),
        caro::models::BackendType::OpenRouter => "openrouter".to_string(),
    };

    let exec_ctx = ExecutionContext::detect();
    let shell_type = cli
        .shell
        .as_deref()
        .and_then(|s| ShellType::from_str(s).ok())
        .unwrap_or_else(|| ShellType::from_str(&exec_ctx.shell).unwrap_or(ShellType::Bash));

    let safety_level = cli
        .safety
        .as_deref()
        .and_then(|s| SafetyLevel::from_str(s).ok())
        .unwrap_or(user_cfg.safety_level);

    let store_path = user_cfg
        .ai
        .db_path
        .clone()
        .or_else(|| caro::ai::store::default_store_path().ok())
        .ok_or_else(|| "no data directory available for ai_sessions.json".to_string())?;

    let validator = caro::ai::build_validator(safety_level);

    let session_mode = if new_session {
        caro::ai::runner::SessionMode::New
    } else {
        caro::ai::runner::SessionMode::ResumeOrNew
    };

    let last_command_hint = std::env::var("CARO_LAST_COMMAND").ok();

    let outcome = run_once(AiInvocation {
        prompt: resolved.text.trim(),
        ai_cfg: &user_cfg.ai,
        backend,
        backend_name,
        exec_ctx,
        validator,
        safety_level,
        shell: shell_type,
        store_path,
        session_mode,
        last_command_hint,
    })
    .await
    .map_err(|e| format!("{}", e))?;

    // Stderr: human-readable risk annotation and warnings. Stdout: the command.
    use colored::Colorize;
    if outcome.warns_offhost {
        eprintln!(
            "{} opt-in context may be sent to remote backend",
            "⚠ privacy:".yellow()
        );
    }
    if !outcome.allowed {
        eprintln!(
            "{} command blocked by safety validator ({:?})",
            "✗".red(),
            outcome.risk
        );
        for w in &outcome.warnings {
            eprintln!("  - {}", w);
        }
        return Err("command rejected for safety reasons".into());
    }
    if matches!(
        outcome.risk,
        caro::models::RiskLevel::High | caro::models::RiskLevel::Critical
    ) {
        eprintln!(
            "{} generated command is HIGH risk — double-check before running",
            "⚠".yellow()
        );
    }
    eprintln!(
        "# caro-ai: session {}{} confidence={:.2} risk={:?}",
        outcome.session_id,
        if outcome.resumed { " (resumed)" } else { "" },
        outcome.confidence,
        outcome.risk
    );

    println!("{}", outcome.command);
    Ok(())
}

// =============================================================================
// CaroML CLI handlers
// =============================================================================

/// Print the discovered tasks. Default lists project + global with project
/// shadowing global by name; `--global` filters to global only.
fn run_caroml_list(global_only: bool) {
    use caro::caroml::discovery::{list_all, list_global_tasks, list_project_tasks, TaskSource};

    let entries = if global_only {
        list_global_tasks()
    } else {
        list_all()
    };

    if entries.is_empty() {
        if global_only {
            println!("(no tasks in ~/.caro/library/)");
        } else {
            println!("(no tasks in ./tasks/ or ~/.caro/library/)");
        }
        return;
    }

    let max_name = entries.iter().map(|e| e.name.len()).max().unwrap_or(0);
    for entry in entries {
        let source = match entry.source {
            TaskSource::Project => "project",
            TaskSource::Global => "global",
        };
        println!(
            "{:<width$}  {}  {}",
            entry.name,
            source,
            entry.path.display(),
            width = max_name
        );
    }

    if !global_only && !list_project_tasks().is_empty() {
        // Project tasks take precedence; list_all already applied that.
        // No-op; the layout above is sufficient.
    }
}

/// Print the JOBs declared in the project's Carofile, if present.
fn run_caroml_jobs() -> Result<(), String> {
    use caro::caroml::{carofile, discovery};

    let path = match discovery::find_carofile() {
        Some(p) => p,
        None => {
            println!("(no Carofile in current directory; create one to define jobs)");
            return Ok(());
        }
    };

    let src = std::fs::read_to_string(&path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let cf = carofile::parse_with_path(&src, Some(path.clone()))
        .map_err(|e| format!("{}: {}", path.display(), e))?;

    println!("{}: {}", path.display(), cf.title);
    if cf.jobs.is_empty() {
        println!("(no JOBs declared)");
        return Ok(());
    }

    let max_name = cf.jobs.iter().map(|j| j.name.len()).max().unwrap_or(0);
    for job in &cf.jobs {
        println!(
            "{:<width$}  runs: {}",
            job.name,
            job.runs.join(", "),
            width = max_name
        );
    }
    Ok(())
}

/// Scaffold a starter `.caro` from a template at `tasks/<name>.caro`.
///
/// v0.1 is template-only — no LLM. PR 4 adds `caro new <name> "<description>"`
/// for LLM-assisted scaffolding.
fn run_caroml_new(name: &str) -> Result<std::path::PathBuf, String> {
    use caro::caroml::discovery::project_tasks_dir;

    if name.trim().is_empty() {
        return Err("task name cannot be empty".to_string());
    }
    if name.contains("..") {
        return Err("task name cannot contain `..`".to_string());
    }

    let mut path = project_tasks_dir();
    for segment in name.split('/') {
        path = path.join(segment);
    }
    let final_path = path.with_extension("caro");

    if final_path.exists() {
        return Err(format!(
            "{} already exists; refusing to overwrite",
            final_path.display()
        ));
    }

    if let Some(parent) = final_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("creating {}: {}", parent.display(), e))?;
    }

    let template = format!(
        "REM Scaffolded by `caro new {}` — fill in the blanks below.\n\
         REM Run `caro check tasks/{}.caro` to validate as you edit.\n\
         \n\
         TASK <one-line title for this task>\n\
         WHY  <why this task exists; runs when?>\n\
         \n\
         REM Optional: declare prerequisites\n\
         REM NEED sudo\n\
         REM ON   macos PREFER bsd-tools\n\
         REM ON   linux PREFER gnu-tools\n\
         \n\
         REM Optional: declare authoring-time parameters; reference as {{name}} in DO lines.\n\
         REM LET path = /var/log\n\
         \n\
         REM Steps — one natural-language intent per DO line.\n\
         DO   <first thing to do>\n\
         DO   <second thing to do>\n",
        name, name,
    );

    std::fs::write(&final_path, template)
        .map_err(|e| format!("writing {}: {}", final_path.display(), e))?;

    Ok(final_path)
}

/// Run `caro generate <name>` — call the configured backend per step,
/// validate, and write `tasks/<name>.caro.lock`.
async fn run_caroml_generate(
    name: &str,
    platform_override: Option<&str>,
    backend_override: Option<&str>,
) -> Result<std::path::PathBuf, String> {
    use caro::caroml::{check_file, discovery, interpreter, platform as caro_platform, validators};

    let path = discovery::resolve_task_path(name)
        .ok_or_else(|| format!("could not find task `{}`", name))?;

    let task = check_file(&path).map_err(|e| format!("{}: {}", path.display(), e))?;

    // Resolve target platform.
    let target_platform = match platform_override {
        Some(p) if caro_platform::is_known(p) => p.to_string(),
        Some(p) => return Err(format!("unknown platform `{}`", p)),
        None => caro_platform::current().to_string(),
    };

    // Resolve backend. v0.1: only `mock` is wired through here (deterministic);
    // other backends arrive in PR 5+ when execution is needed.
    let backend =
        build_caroml_backend(backend_override).map_err(|e| format!("backend setup: {}", e))?;
    let backend_ref: &dyn caro::backends::CommandGenerator = &*backend;

    let chain = validators::default_chain();
    let cfg = interpreter::GenerateConfig::for_intent(path.to_string_lossy().into_owned());

    let lock = if platform_override.is_some() {
        interpreter::generate_lock_for_platform(&task, &target_platform, backend_ref, &chain, &cfg)
            .await
            .map_err(|e| format!("generation failed: {}", e))?
    } else {
        interpreter::generate_lock_for_all_platforms(
            &task,
            &target_platform,
            backend_ref,
            &chain,
            &cfg,
        )
        .await
        .map_err(|e| format!("generation failed: {}", e))?
    };

    let lock_path = path.with_extension("caro.lock");
    lock.write_path(&lock_path)
        .map_err(|e| format!("writing {}: {}", lock_path.display(), e))?;
    Ok(lock_path)
}

/// Build the backend used by `caro generate`. v0.1 supports only `mock`
/// (an inline echo-style backend) for deterministic CLI tests; the real
/// `embedded`/`ollama` etc. backends arrive in PR 5 when execution lands.
fn build_caroml_backend(
    name: Option<&str>,
) -> Result<Box<dyn caro::backends::CommandGenerator>, String> {
    match name.unwrap_or("mock") {
        "mock" => Ok(Box::new(InlineMockBackend)),
        other => Err(format!(
            "backend `{}` not yet wired into `caro generate` in v0.1; \
             only `mock` is supported until PR 5",
            other
        )),
    }
}

/// Status of the on-disk runbook relative to the lock's stamped hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunbookStatus {
    Missing,
    Clean,
    Drift,
    NoStamp,
}

fn runbook_status_for_active(
    lock: &caro::caroml::lock::Lock,
    platform: &str,
    runbook_path: &std::path::Path,
) -> RunbookStatus {
    if !runbook_path.exists() {
        return RunbookStatus::Missing;
    }
    let stamp = lock
        .steps
        .iter()
        .find_map(|s| s.active_variant(platform).map(|v| v.runbook_hash.clone()))
        .unwrap_or_default();
    if stamp.is_empty() {
        return RunbookStatus::NoStamp;
    }
    match caro::caroml::runbook::read_and_hash(runbook_path) {
        Ok(actual) if actual == stamp => RunbookStatus::Clean,
        Ok(_) => RunbookStatus::Drift,
        Err(_) => RunbookStatus::Missing,
    }
}

/// Run `caro run <name>` — read lock, build plan, confirm, execute.
fn run_caroml_run(
    name: &str,
    platform_override: Option<&str>,
    yes: bool,
    dry_run: bool,
) -> Result<(), String> {
    use caro::caroml::{discovery, lock::Lock, platform as caro_platform, runner};
    use std::io::{self, Write};

    let task_path = discovery::resolve_task_path(name)
        .ok_or_else(|| format!("could not find task `{}`", name))?;
    let lock_path = task_path.with_extension("caro.lock");
    if !lock_path.exists() {
        return Err(format!(
            "{}: no lock found; run `caro generate {}` first",
            lock_path.display(),
            name
        ));
    }
    let lock =
        Lock::read_path(&lock_path).map_err(|e| format!("{}: {}", lock_path.display(), e))?;

    let target_platform = match platform_override {
        Some(p) if caro_platform::is_known(p) => p.to_string(),
        Some(p) => return Err(format!("unknown platform `{}`", p)),
        None => caro_platform::current().to_string(),
    };

    let plan = runner::plan_run(&lock, &target_platform)
        .map_err(|e| format!("{}: {}", lock_path.display(), e))?;

    // Runbook-first execution: if the per-platform `.sh` runbook exists and
    // is hash-clean, prefer running it directly (matches what a non-Caro
    // user would `bash`). Falls back to step-by-step otherwise.
    use caro::caroml::runbook;
    let runbook_path = runbook::runbook_path(&task_path, &target_platform);
    let runbook_status = runbook_status_for_active(&lock, &target_platform, &runbook_path);

    println!("{}", runner::render_plan(&plan));
    match &runbook_status {
        RunbookStatus::Missing => {
            println!("(no runbook on disk; will execute step-by-step from the lock)")
        }
        RunbookStatus::Clean => {
            println!(
                "(runbook clean — will run `bash {}`)",
                runbook_path.display()
            )
        }
        RunbookStatus::Drift => {
            eprintln!(
                "warning: {} has been edited since `caro export` last ran.\n\
                 Step-by-step execution from the lock will be used instead.\n\
                 Run `caro export {}` to refresh the runbook from the lock.",
                runbook_path.display(),
                name
            );
        }
        RunbookStatus::NoStamp => {
            eprintln!(
                "note: lock has no runbook_hash stamp yet. Run `caro export {}` after\n\
                 successful runs to enable drift detection.",
                name
            );
        }
    }

    if dry_run {
        return Ok(());
    }

    if !yes {
        print!("Proceed? [y/N] ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    use caro::caroml::history;
    use std::time::Instant;
    let started = Instant::now();
    // Prefer runbook execution when it's hash-clean; fall back to per-step.
    let result = match runbook_status {
        RunbookStatus::Clean => match runner::execute_runbook(&runbook_path) {
            Ok(0) => Ok(vec![]),
            Ok(other) => Err(runner::RunError::StepFailed {
                line: 0,
                intent: format!("bash {}", runbook_path.display()),
                exit_code: other,
                stderr: String::new(),
            }),
            Err(e) => Err(e),
        },
        _ => runner::execute_plan(&plan),
    };
    let elapsed_ms = started.elapsed().as_millis() as u64;

    // Journal the outcome (best-effort; don't fail the run if journal write fails).
    let (exit_code, note, stderr) = match &result {
        Ok(_) => (0i32, None, String::new()),
        Err(runner::RunError::StepFailed {
            line,
            intent,
            exit_code,
            stderr,
        }) => (
            *exit_code,
            Some(format!("step on line {}: {}", line, intent)),
            stderr.clone(),
        ),
        Err(other) => (255, Some(format!("{}", other)), String::new()),
    };
    let variant_id = plan
        .steps
        .first()
        .map(|s| s.generation_id.clone())
        .unwrap_or_default();
    let entry = history::JournalEntry {
        timestamp: chrono::Utc::now(),
        intent_hash: lock.meta.intent_hash.clone(),
        variant_id,
        platform: target_platform,
        exit_code,
        duration_ms: elapsed_ms,
        stderr_digest: history::stderr_digest(&stderr),
        note,
    };
    if let Err(e) = history::append(&entry) {
        eprintln!("warning: could not write run journal: {}", e);
    }

    let results = result.map_err(|e| e.to_string())?;
    println!(
        "All {} steps completed in {} ms.",
        results.len(),
        elapsed_ms
    );
    Ok(())
}

/// `caro experiment <name>` — generate a fresh challenger variant.
async fn run_caroml_experiment(
    name: &str,
    platform_override: Option<&str>,
    backend_override: Option<&str>,
) -> Result<(std::path::PathBuf, String), String> {
    use caro::caroml::{
        check_file, discovery, interpreter, lock::Lock, platform as caro_platform, validators,
        variants as variant_helpers,
    };

    let task_path = discovery::resolve_task_path(name)
        .ok_or_else(|| format!("could not find task `{}`", name))?;
    let task = check_file(&task_path).map_err(|e| format!("{}: {}", task_path.display(), e))?;

    let lock_path = task_path.with_extension("caro.lock");
    if !lock_path.exists() {
        return Err(format!(
            "{}: no lock found; run `caro generate {}` first",
            lock_path.display(),
            name
        ));
    }
    let mut lock =
        Lock::read_path(&lock_path).map_err(|e| format!("{}: {}", lock_path.display(), e))?;

    let target_platform = match platform_override {
        Some(p) if caro_platform::is_known(p) => p.to_string(),
        Some(p) => return Err(format!("unknown platform `{}`", p)),
        None => caro_platform::current().to_string(),
    };

    let backend = build_caroml_backend(backend_override).map_err(|e| format!("backend: {}", e))?;
    let chain = validators::default_chain();
    let cfg = interpreter::GenerateConfig::for_intent(task_path.to_string_lossy().into_owned());

    let fresh =
        interpreter::generate_lock_for_platform(&task, &target_platform, &*backend, &chain, &cfg)
            .await
            .map_err(|e| format!("generation failed: {}", e))?;

    // Append the new variants to the existing lock as challengers.
    // Bump the generation_id suffix so we don't clash with existing IDs.
    let existing_count = variant_helpers::all_challengers_for(&lock, &target_platform).count()
        + variant_helpers::active_count_for(&lock, &target_platform);
    let new_gen_id =
        variant_helpers::generation_id(chrono::Utc::now(), &target_platform, existing_count);

    let mut adopted_id = String::new();
    for (i, fresh_step) in fresh.steps.into_iter().enumerate() {
        if let Some(existing_step) = lock.steps.get_mut(i) {
            for mut variant in fresh_step.variants {
                variant.active = false;
                variant.generation_id = format!("{}_step{}", new_gen_id, i);
                adopted_id = variant.generation_id.clone();
                existing_step.variants.push(variant);
            }
        }
    }
    // Rewrite the imported history entries' generation_id to match this
    // experiment's id, and tag them as challenger entries so `caro history`
    // can distinguish initial-gen rows from challenger-add rows.
    for mut h in fresh.history {
        h.generation_id = new_gen_id.clone();
        h.trigger = "experiment".into();
        h.notes = Some(format!(
            "Challenger added by `caro experiment` on platform `{}`.",
            target_platform
        ));
        lock.history.push(h);
    }

    lock.write_path(&lock_path)
        .map_err(|e| format!("writing {}: {}", lock_path.display(), e))?;

    Ok((lock_path, adopted_id))
}

/// `caro adopt <name> --variant <id>`.
fn run_caroml_adopt(name: &str, variant_id: &str) -> Result<(), String> {
    use caro::caroml::{adopt as adopt_mod, discovery, lock::Lock};

    let task_path = discovery::resolve_task_path(name)
        .ok_or_else(|| format!("could not find task `{}`", name))?;
    let lock_path = task_path.with_extension("caro.lock");
    let mut lock =
        Lock::read_path(&lock_path).map_err(|e| format!("{}: {}", lock_path.display(), e))?;
    adopt_mod::adopt(&mut lock, variant_id).map_err(|e| e.to_string())?;
    lock.write_path(&lock_path)
        .map_err(|e| format!("writing {}: {}", lock_path.display(), e))?;
    Ok(())
}

/// `caro history <name>`.
fn run_caroml_history(name: &str) -> Result<(), String> {
    use caro::caroml::{discovery, history, lock::Lock};

    let task_path = discovery::resolve_task_path(name)
        .ok_or_else(|| format!("could not find task `{}`", name))?;
    let lock_path = task_path.with_extension("caro.lock");
    let lock =
        Lock::read_path(&lock_path).map_err(|e| format!("{}: {}", lock_path.display(), e))?;

    println!("Lock history for {}:", lock_path.display());
    if lock.history.is_empty() {
        println!("(no entries)");
    } else {
        for h in &lock.history {
            println!(
                "  {} [{}] {} on {} (model: {}, trigger: {})",
                h.generation_id,
                h.generated_at.format("%Y-%m-%d %H:%M:%SZ"),
                h.platform,
                h.backend,
                h.model,
                h.trigger
            );
        }
    }

    let entries = history::read_all(&lock.meta.intent_hash).unwrap_or_default();
    println!("\nLocal run journal ({} entries):", entries.len());
    let recent: Vec<_> = entries.iter().rev().take(10).collect();
    for e in recent {
        println!(
            "  {} variant={} platform={} exit={} ({} ms)",
            e.timestamp.format("%Y-%m-%d %H:%M:%SZ"),
            e.variant_id,
            e.platform,
            e.exit_code,
            e.duration_ms
        );
    }
    Ok(())
}

/// `caro render <name>` — write Markdown docs from a `.caro` task.
fn run_caroml_render(name: &str, output: Option<&std::path::Path>) -> Result<(), String> {
    use caro::caroml::{check_file, discovery, render};

    let path = discovery::resolve_task_path(name)
        .ok_or_else(|| format!("could not find task `{}`", name))?;
    let task = check_file(&path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let md = render::render_markdown(&task);
    match output {
        Some(out) => {
            if let Some(parent) = out.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("creating {}: {}", parent.display(), e))?;
                }
            }
            std::fs::write(out, md).map_err(|e| format!("writing {}: {}", out.display(), e))?;
            println!("{}: rendered", out.display());
        }
        None => print!("{}", md),
    }
    Ok(())
}

/// `caro skill install|uninstall` — manage the bundled coder-agent skill.
fn run_caroml_skill(command: &SkillSubcommand) -> Result<(), String> {
    use caro::caroml::skill;

    let dest = skill::default_install_dir().map_err(|e| e.to_string())?;
    match command {
        SkillSubcommand::Install => {
            let source = skill::bundled_source_dir();
            let installed = skill::install(&source, &dest).map_err(|e| e.to_string())?;
            println!("Installed `caro-scaffold` skill to {}", installed.display());
            Ok(())
        }
        SkillSubcommand::Uninstall => {
            let removed = skill::uninstall(&dest).map_err(|e| e.to_string())?;
            if removed {
                println!("Removed {}", dest.display());
            } else {
                println!("No skill installed at {}", dest.display());
            }
            Ok(())
        }
    }
}

/// `caro do <name>` — Carofile JOB / external-alias / native-alias / fallback.
fn run_caroml_do(name: &str, dry_run: bool) -> Result<(), String> {
    use caro::caroml::{carofile, discovery, jobs, platform as caro_platform};

    // Try to load the Carofile (optional).
    let carofile_path = discovery::find_carofile();
    let carofile = if let Some(path) = carofile_path.as_ref() {
        let src = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Some(
            carofile::parse_with_path(&src, Some(path.clone()))
                .map_err(|e| format!("{}: {}", path.display(), e))?,
        )
    } else {
        None
    };

    let resolution = jobs::resolve(name, carofile.as_ref());
    print!(
        "{}",
        jobs::render_plan(name, &resolution, carofile.as_ref())
    );

    if dry_run {
        return Ok(());
    }

    let platform = caro_platform::current().to_string();
    let results = jobs::dispatch(name, carofile.as_ref(), |alias_or_name, task_path_opt| {
        // Native task or bare-task: load the lock and execute it.
        let task_path = match task_path_opt {
            Some(p) => p.to_path_buf(),
            None => discovery::resolve_task_path(alias_or_name).ok_or_else(|| {
                jobs::DoError::Other(format!("could not find task `{}`", alias_or_name))
            })?,
        };
        jobs::run_native_task(&task_path, &platform)
    })
    .map_err(|e| e.to_string())?;

    println!("Completed {} step(s) in caro do {}.", results.len(), name);
    Ok(())
}

/// `caro why <name>` — explain the RegenEvaluator decision.
fn run_caroml_why(name: &str) -> Result<(), String> {
    use caro::caroml::{
        check_file, discovery, lock::Lock, platform as caro_platform, regen_evaluator,
    };

    let task_path = discovery::resolve_task_path(name)
        .ok_or_else(|| format!("could not find task `{}`", name))?;
    let task = check_file(&task_path).map_err(|e| format!("{}: {}", task_path.display(), e))?;

    let lock_path = task_path.with_extension("caro.lock");
    let lock = if lock_path.exists() {
        Some(Lock::read_path(&lock_path).map_err(|e| format!("{}: {}", lock_path.display(), e))?)
    } else {
        None
    };

    let platform = caro_platform::current().to_string();
    let input = regen_evaluator::EvalInput {
        task: &task,
        lock: lock.as_ref(),
        platform: &platform,
        current_caro_version: env!("CARGO_PKG_VERSION"),
        current_model: "mock-inline",
        current_backend: "mock",
        mode: regen_evaluator::Mode::default(),
    };
    let decision = regen_evaluator::decide(&input);

    println!("Task:     {}", name);
    println!("Path:     {}", task_path.display());
    println!("Platform: {}", platform);
    println!(
        "Decision: {:?}",
        match &decision {
            regen_evaluator::Decision::UseCache => "UseCache",
            regen_evaluator::Decision::HardRegen { .. } => "HardRegen",
            regen_evaluator::Decision::SoftExplore { .. } => "SoftExplore",
        }
    );
    let reasons = decision.reasons();
    if reasons.is_empty() {
        println!("(no reasons; cache is fresh)");
    } else {
        println!("Reasons:");
        for r in reasons {
            println!("  - {}", r);
        }
    }
    Ok(())
}

/// Run `caro export <name>` — write `tasks/<name>.<platform>.sh` from the lock.
fn run_caroml_export(
    name: &str,
    platform_override: Option<&str>,
    output: Option<&std::path::Path>,
) -> Result<std::path::PathBuf, String> {
    use caro::caroml::{discovery, lock::Lock, platform as caro_platform, runbook};

    let task_path = discovery::resolve_task_path(name)
        .ok_or_else(|| format!("could not find task `{}`", name))?;
    let lock_path = task_path.with_extension("caro.lock");
    if !lock_path.exists() {
        return Err(format!(
            "{}: no lock found; run `caro generate {}` first",
            lock_path.display(),
            name
        ));
    }
    let lock =
        Lock::read_path(&lock_path).map_err(|e| format!("{}: {}", lock_path.display(), e))?;

    let platform = match platform_override {
        Some(p) if caro_platform::is_known(p) => p.to_string(),
        Some(p) => return Err(format!("unknown platform `{}`", p)),
        None => caro_platform::current().to_string(),
    };

    let body = runbook::build_runbook(&lock, &platform).map_err(|e| e.to_string())?;
    let body_hash = runbook::compute_runbook_hash(&body);

    let path = if let Some(out) = output {
        if let Some(parent) = out.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("creating {}: {}", parent.display(), e))?;
            }
        }
        std::fs::write(out, &body).map_err(|e| format!("writing {}: {}", out.display(), e))?;
        out.to_path_buf()
    } else {
        runbook::write_runbook(&lock, &platform, &task_path).map_err(|e| e.to_string())?
    };

    // Stamp the runbook_hash onto every active variant for this platform.
    // This is what `caro run` later compares against the on-disk runbook to
    // detect manual edits.
    let mut updated = lock;
    for step in updated.steps.iter_mut() {
        for v in step.variants.iter_mut() {
            if v.active && v.platform == platform {
                v.runbook_hash = body_hash.clone();
            }
        }
    }
    updated
        .write_path(&lock_path)
        .map_err(|e| format!("updating {}: {}", lock_path.display(), e))?;

    Ok(path)
}

/// Inline echo-style deterministic backend for `caro generate --backend mock`.
/// Always emits `echo "<intent>"`. Used by tests and demos in v0.1.
struct InlineMockBackend;

#[async_trait::async_trait]
impl caro::backends::CommandGenerator for InlineMockBackend {
    async fn generate_command(
        &self,
        request: &caro::models::CommandRequest,
    ) -> Result<caro::models::GeneratedCommand, caro::backends::GeneratorError> {
        let safe_intent = request.input.replace('"', "\\\"");
        Ok(caro::models::GeneratedCommand {
            command: format!("echo \"{}\"", safe_intent),
            explanation: format!("Mock echo of the intent: {}", request.input),
            safety_level: caro::models::RiskLevel::Safe,
            estimated_impact: "writes the intent text to stdout (no side effects)".into(),
            alternatives: vec![],
            backend_used: "mock-inline".into(),
            generation_time_ms: 1,
            confidence_score: 0.5,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn backend_info(&self) -> caro::backends::BackendInfo {
        caro::backends::BackendInfo {
            backend_type: caro::models::BackendType::Mock,
            model_name: "mock-inline".into(),
            supports_streaming: false,
            max_tokens: 256,
            typical_latency_ms: 1,
            memory_usage_mb: 0,
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    async fn shutdown(&self) -> Result<(), caro::backends::GeneratorError> {
        Ok(())
    }
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
                    println!(
                        "{} Successfully indexed GitHub repo: {}",
                        "✓".green(),
                        repo.bold()
                    );
                    println!("  Documentation added to knowledge base");
                }
                Ok(false) => {
                    println!(
                        "{} No useful documentation found for: {}",
                        "✗".yellow(),
                        repo
                    );
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
                            println!(
                                "{}. {} (similarity: {:.2}%)",
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
            std::fs::write(&path, json).map_err(|e| format!("Failed to write file: {}", e))?;

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

            let entries: Vec<KnowledgeEntry> =
                serde_json::from_str(&json).map_err(|e| format!("Failed to parse JSON: {}", e))?;

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
                            entry.profile.as_deref(),
                        )
                        .await
                } else {
                    index
                        .record_success(
                            &entry.request,
                            &entry.command,
                            entry.context.as_deref(),
                            entry.profile.as_deref(),
                        )
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

    // Handle --backend-info as a meta flag (like --version): print the status
    // table and exit without requiring a prompt or touching telemetry/setup.
    if cli.backend_info {
        print_backend_info();
        process::exit(0);
    }

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
        Some(Commands::Integration { shell }) => {
            print_shell_init_script(&shell);
            process::exit(0);
        }
        Some(Commands::Init { .. }) => {
            // Handled below after telemetry initialization
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
        Some(Commands::ShellInit { shell, disable_ai }) => {
            match handle_shell_init(&shell, disable_ai) {
                Ok(()) => process::exit(0),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        Some(Commands::Check { ref file }) => match caro::caroml::check_file(file) {
            Ok(task) => {
                println!(
                    "{}: ok ({} steps, {} pragmas, {} params)",
                    file.display(),
                    task.steps.len(),
                    task.platform_pragmas.len(),
                    task.params.len()
                );
                process::exit(0);
            }
            Err(caro::caroml::CheckError::Io(e)) => {
                eprintln!("{}: {}", file.display(), e);
                process::exit(1);
            }
            Err(caro::caroml::CheckError::Parse(e)) => {
                eprintln!("{}: {}", file.display(), e);
                process::exit(1);
            }
        },
        Some(Commands::List { global }) => {
            run_caroml_list(global);
            process::exit(0);
        }
        Some(Commands::Jobs) => match run_caroml_jobs() {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::New { ref name }) => match run_caroml_new(name) {
            Ok(path) => {
                println!("{}: created", path.display());
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Generate {
            ref name,
            ref platform,
            ref backend,
        }) => match run_caroml_generate(name, platform.as_deref(), backend.as_deref()).await {
            Ok(lock_path) => {
                println!("{}: generated", lock_path.display());
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Run {
            ref name,
            yes,
            ref platform,
            dry_run,
        }) => match run_caroml_run(name, platform.as_deref(), yes, dry_run) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Export {
            ref name,
            ref platform,
            ref output,
        }) => match run_caroml_export(name, platform.as_deref(), output.as_deref()) {
            Ok(path) => {
                println!("{}: written", path.display());
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Experiment {
            ref name,
            ref platform,
            ref backend,
        }) => match run_caroml_experiment(name, platform.as_deref(), backend.as_deref()).await {
            Ok((path, gen_id)) => {
                println!("{}: challenger {} added", path.display(), gen_id);
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Adopt {
            ref name,
            ref variant,
        }) => match run_caroml_adopt(name, variant) {
            Ok(()) => {
                println!("Adopted {} as active.", variant);
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::History { ref name }) => match run_caroml_history(name) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Why { ref name }) => match run_caroml_why(name) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Do { ref name, dry_run }) => match run_caroml_do(name, dry_run) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Render {
            ref name,
            ref output,
        }) => match run_caroml_render(name, output.as_deref()) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Skill { ref command }) => match run_caroml_skill(command) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Ai {
            new_session,
            continue_session: _,
            once: _,
            ref prompt,
        }) => {
            let ai_trailing = prompt.clone();
            let new = new_session;
            match run_ai_once(&cli, new, ai_trailing).await {
                Ok(()) => process::exit(0),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
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

    // Session-scoped telemetry override: --no-telemetry disables telemetry
    // for this invocation only. We do NOT persist this to user_config — the
    // next invocation will use whatever the stored preference is.
    let telemetry_enabled_for_session = user_config.telemetry.enabled && !cli.no_telemetry;

    // Check for first-run consent
    // Skip interactive consent for non-human output formats (json, yaml)
    // and also when the user has explicitly asked to run with --no-telemetry
    // (it would be hostile to prompt them for consent when they just said
    // "no, thanks, not this time").
    let is_interactive_output = cli
        .output
        .as_deref()
        .is_none_or(|format| format != "json" && format != "yaml");

    if user_config.telemetry.first_run && is_interactive_output && !cli.no_telemetry {
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
            telemetry_enabled_for_session,
        ));

        // Set as global collector for easy access from all components
        caro::set_global_collector(telemetry_collector.clone());

        // Start telemetry uploader if enabled and not air-gapped
        if telemetry_enabled_for_session && !user_config.telemetry.air_gapped {
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

    // Handle init subcommand
    if let Some(Commands::Init { minimal, force }) = &cli.command {
        match run_init_wizard(*minimal, *force) {
            Ok(completed) => {
                process::exit(if completed { 0 } else { 1 });
            }
            Err(e) => {
                eprintln!("Error running setup wizard: {}", e);
                process::exit(1);
            }
        }
    }

    // Check for first-time setup (if running without subcommand and no config exists)
    if cli.command.is_none() && cli.prompt.is_none() && !cli.show_config {
        // Check if this is a first-time run
        if needs_setup() {
            use colored::Colorize;
            println!();
            println!(
                "{}",
                "Welcome to caro! It looks like this is your first time running the tool.".bold()
            );
            println!();

            // Check if we're in a terminal for interactive setup
            if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
                use dialoguer::Confirm;

                let run_setup = Confirm::new()
                    .with_prompt("Would you like to run the setup wizard now?")
                    .default(true)
                    .interact()
                    .unwrap_or(false);

                if run_setup {
                    match run_init_wizard(false, false) {
                        Ok(true) => process::exit(0),
                        Ok(false) => {
                            // User cancelled, show usage
                            print_usage();
                            process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("Error running setup wizard: {}", e);
                            process::exit(1);
                        }
                    }
                } else {
                    println!();
                    println!(
                        "{}",
                        "You can run 'caro init' at any time to configure the tool.".dimmed()
                    );
                    println!();
                    print_usage();
                    process::exit(0);
                }
            } else {
                // Non-interactive: show message and usage
                println!(
                    "{}",
                    "Run 'caro init' in an interactive terminal to configure the tool.".dimmed()
                );
                println!();
                print_usage();
                process::exit(0);
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

    // Validate prompt and show help/warnings/hints as needed
    let prompt_text = cli.prompt.as_deref().unwrap_or("");
    match validate_prompt(prompt_text) {
        ValidationAction::ShowHelp => {
            // Show help message for empty or whitespace-only prompts
            print_usage();
            process::exit(0);
        }
        ValidationAction::Warning { message } => {
            // Always show warnings (serious issues)
            eprintln!("{}", message);
            eprintln!();
            // Continue with command generation despite warning
        }
        ValidationAction::Hint { message } => {
            // Show hints only in verbose mode (minor issues)
            if cli.verbose {
                eprintln!("{}", message);
                eprintln!();
            }
            // Continue with command generation
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

/// Run the init setup wizard
fn run_init_wizard(minimal: bool, force: bool) -> Result<bool, caro::setup::SetupError> {
    use colored::Colorize;

    let wizard = SetupWizard::new()?.use_minimal_banner(minimal);

    // Check if already configured
    if !force && !wizard.needs_setup() {
        println!();
        println!("{}", "caro is already configured!".green().bold());
        println!();

        // Check if we're in a terminal
        if std::io::stdin().is_terminal() {
            use dialoguer::Confirm;

            let reconfigure = Confirm::new()
                .with_prompt("Would you like to reconfigure?")
                .default(false)
                .interact()
                .unwrap_or(false);

            if !reconfigure {
                println!();
                println!(
                    "{}",
                    "Configuration unchanged. Use 'caro init --force' to reset configuration."
                        .dimmed()
                );
                return Ok(true);
            }
        } else {
            println!("{}", "Use 'caro init --force' to reconfigure.".dimmed());
            return Ok(true);
        }
    }

    // Run the wizard
    let result = wizard.run()?;
    Ok(result.completed)
}

/// Print usage information
fn print_usage() {
    println!("caro - Convert natural language to shell commands using local LLMs");
    println!();
    println!("Usage: caro [OPTIONS] <PROMPT>");
    println!("       caro init              Run the setup wizard");
    println!();
    println!("Examples:");
    println!("  caro list files");
    println!("  caro -p \"list files\"");
    println!("  echo \"list files\" | caro");
    println!("  caro --shell zsh \"find large files\"");
    println!();
    println!("Run 'caro --help' for more information.");
}

async fn run_cli(cli: &Cli) -> Result<bool, CliError> {
    // Create CLI application with optional backend and model overrides
    let mut app = CliApp::with_overrides(
        caro::cli::CliConfig::default(),
        cli.backend.clone(),
        cli.model_name.clone(),
        cli.force_llm,
    )
    .await?;

    // Apply --context-level if user supplied one.
    if let Some(ref lvl) = cli.context_level {
        app.set_context_level(lvl)?;
    }

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

    // Handle explain mode output (educational format like crush)
    if result.explain_mode {
        if let Some(ref explanation) = result.detailed_explanation {
            // Print tool identification and summary
            display!(
                "Use `{}` {}:",
                explanation.tool_used.bright_cyan().bold(),
                if explanation.summary.is_empty() {
                    "for this task".to_string()
                } else {
                    explanation.summary.clone()
                }
            );
            display!("");

            // Print the main command with comment
            display!("  {} Primary command", "#".dimmed());
            display!("  {}", result.generated_command.bright_cyan().bold());
            display!("");

            // Print usage examples
            if !explanation.examples.is_empty() {
                for example in &explanation.examples {
                    display!("  {} {}", "#".dimmed(), example.description.dimmed());
                    display!("  {}", example.command);
                    display!("");
                }
            }

            // Print option breakdown
            if !explanation.option_breakdown.is_empty() {
                display!("{}", "Options explained:".bold());
                for opt in &explanation.option_breakdown {
                    let example_str = opt
                        .example_value
                        .as_ref()
                        .map(|v| format!(" (e.g., {})", v))
                        .unwrap_or_default();
                    display!(
                        "  {}: {}{}",
                        opt.option.cyan(),
                        opt.description,
                        example_str.dimmed()
                    );
                }
                display!("");
            }

            // Print alternatives if available
            if !explanation.alternatives.is_empty() {
                display!("{}", "Alternatives:".bold());
                for alt in &explanation.alternatives {
                    display!("  {} - {}", alt.command.yellow(), alt.reason.dimmed());
                }
                display!("");
            }
        } else {
            // Fallback if detailed explanation not available
            display!("{}", "Command:".bold());
            display!("  {}", result.generated_command.bright_cyan().bold());
            display!("");
            if !result.explanation.is_empty() {
                display!("{}", "Explanation:".bold());
                display!("  {}", result.explanation);
                display!("");
            }
        }
    } else {
        // Standard output (non-explain mode)
        display!("{}", "Command:".bold());
        display!("  {}", result.generated_command.bright_cyan().bold());
        display!("");

        // Print explanation only in verbose mode
        if cli.verbose && !result.explanation.is_empty() {
            display!("{}", "Explanation:".bold());
            display!("  {}", result.explanation);
            display!("");
        }
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

        // Print execution time (suppressed by --quiet)
        if result.timing_info.execution_time_ms > 0 && !cli.quiet {
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

/// Print the list of inference backends along with their current status.
///
/// Invoked by the `--backend-info` meta flag. Exits the caller with code 0.
/// Reports each built-in backend and whether remote backends have the
/// environment variables / typical endpoints set that would make them
/// usable. This is a best-effort snapshot, not a live health check.
fn print_backend_info() {
    use colored::Colorize;

    // Status helpers. For remote backends we use environment variables as
    // a lightweight "configured?" signal — probing each endpoint would
    // turn `--backend-info` into a slow diagnostic command.
    let env_or = |keys: &[&str]| keys.iter().any(|k| std::env::var(k).is_ok());

    let status_for = |keys: &[&str]| {
        if env_or(keys) {
            "configured"
        } else {
            "not configured"
        }
    };

    let row = |backend: &str, status: &str, notes: &str| {
        println!("  {:<12}  {:<16}  {}", backend, status, notes);
    };

    println!("{}", "Available inference backends".bold());
    println!();
    println!(
        "  {:<12}  {:<16}  {}",
        "Backend".bold(),
        "Status".bold(),
        "Notes".bold()
    );
    row("-------", "------", "-----");

    // Built-in, always-available backends.
    row("static", "available", "template-based; no model required");
    row(
        "embedded",
        "available",
        "local LLM (MLX/CPU); downloads model on first use",
    );

    // Remote backends: we report "configured" if a credential / endpoint
    // env var is set, otherwise "not configured".
    row(
        "ollama",
        status_for(&["OLLAMA_HOST", "CARO_OLLAMA_URL"]),
        "remote Ollama HTTP API (OLLAMA_HOST)",
    );
    row(
        "vllm",
        status_for(&["VLLM_BASE_URL", "CARO_VLLM_URL"]),
        "remote vLLM HTTP API (VLLM_BASE_URL)",
    );
    row(
        "claude",
        status_for(&["ANTHROPIC_API_KEY"]),
        "Anthropic Claude API (ANTHROPIC_API_KEY)",
    );

    println!();
    println!(
        "{}",
        "Use --backend <name> to force a specific backend.".dimmed()
    );
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

    #[test]
    fn test_flags_only_shows_warning() {
        // T027: Prompts with only flags should show warning
        match validate_prompt("-la") {
            ValidationAction::Warning { message } => {
                assert!(message.contains("No command description"));
            }
            _ => panic!("Expected Warning for flags-only prompt"),
        }
        match validate_prompt("-rf -v") {
            ValidationAction::Warning { .. } => {}
            _ => panic!("Expected Warning for multiple flags only"),
        }
    }

    #[test]
    fn test_operators_only_shows_warning() {
        // T028: Prompts with only operators should show warning
        match validate_prompt(">") {
            ValidationAction::Warning { message } => {
                assert!(message.contains("No command description"));
            }
            _ => panic!("Expected Warning for operator-only prompt"),
        }
        match validate_prompt(">>") {
            ValidationAction::Warning { .. } => {}
            _ => panic!("Expected Warning for operator-only prompt"),
        }
    }

    #[test]
    fn test_flags_with_args_proceeds() {
        // T029: Flags with arguments should proceed (has content)
        assert_eq!(
            validate_prompt("-rf /tmp"),
            ValidationAction::ProceedWithPrompt
        );
    }

    #[test]
    fn test_operators_with_commands_proceeds() {
        // T030: Operators with commands should proceed (has content)
        assert_eq!(
            validate_prompt("| grep"),
            ValidationAction::ProceedWithPrompt
        );
    }

    #[test]
    fn test_very_short_shows_hint() {
        // T031: Very short prompts (< 3 chars) should show hint
        match validate_prompt("do") {
            ValidationAction::Hint { message } => {
                assert!(message.contains("very short"));
            }
            _ => panic!("Expected Hint for very short prompt"),
        }
        match validate_prompt("ls") {
            ValidationAction::Hint { .. } => {}
            _ => panic!("Expected Hint for very short prompt"),
        }
    }

    #[test]
    fn test_single_word_shows_hint() {
        // T032: Single word prompts (< 8 chars) should show hint
        match validate_prompt("list") {
            ValidationAction::Hint { message } => {
                assert!(message.contains("ambiguous"));
            }
            _ => panic!("Expected Hint for single-word prompt"),
        }
        match validate_prompt("show") {
            ValidationAction::Hint { .. } => {}
            _ => panic!("Expected Hint for single-word prompt"),
        }
    }

    #[test]
    fn test_long_single_word_proceeds() {
        // T033: Long single words (>= 8 chars) should proceed
        assert_eq!(
            validate_prompt("processes"),
            ValidationAction::ProceedWithPrompt
        );
    }

    #[test]
    fn test_multi_word_prompt_proceeds() {
        // T034: Multi-word prompts should proceed even if short
        assert_eq!(
            validate_prompt("list files"),
            ValidationAction::ProceedWithPrompt
        );
        assert_eq!(
            validate_prompt("show all"),
            ValidationAction::ProceedWithPrompt
        );
    }

    // WP06: Shell Operator Handling Tests

    #[test]
    fn test_all_operators() {
        // T035: Test all 7 POSIX shell operators are detected
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
