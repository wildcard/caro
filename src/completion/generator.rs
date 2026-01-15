//! Shell completion script generation using clap_complete

use crate::models::ShellType;
use clap_complete::{generate, Shell};
use std::io::BufWriter;

/// Generate shell completion script for the given shell type
///
/// Returns the completion script as a string that can be sourced by the shell.
pub fn generate_completions(shell: ShellType) -> String {
    // We need to create a clap Command that matches our CLI structure
    // Since we can't directly access the private Cli struct from main.rs,
    // we'll build a compatible command here

    let mut cmd = build_caro_command();
    let shell = shell_type_to_clap_shell(shell);

    let mut buf = BufWriter::new(Vec::new());
    generate(shell, &mut cmd, "caro", &mut buf);

    String::from_utf8(buf.into_inner().unwrap_or_default()).unwrap_or_default()
}

/// Convert our ShellType to clap_complete's Shell enum
fn shell_type_to_clap_shell(shell: ShellType) -> Shell {
    match shell {
        ShellType::Bash => Shell::Bash,
        ShellType::Zsh => Shell::Zsh,
        ShellType::Fish => Shell::Fish,
        // For shells not directly supported by clap_complete, fallback to Bash
        ShellType::Sh => Shell::Bash,
        ShellType::PowerShell => Shell::PowerShell,
        ShellType::Cmd | ShellType::Unknown => Shell::Bash, // Fallback to Bash
    }
}

/// Build a clap Command that represents the Caro CLI structure
///
/// This mirrors the structure in main.rs but is accessible here for completion generation.
fn build_caro_command() -> clap::Command {
    use clap::{Arg, ArgAction, Command};

    Command::new("caro")
        .about("Convert natural language to shell commands using local LLMs")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("prompt")
                .short('p')
                .long("prompt")
                .help("Natural language command description")
                .value_name("PROMPT"),
        )
        .arg(
            Arg::new("backend")
                .short('b')
                .long("backend")
                .help("Inference backend to use")
                .value_name("BACKEND")
                .value_parser(["embedded", "ollama", "exo", "vllm", "static"]),
        )
        .arg(
            Arg::new("shell")
                .short('s')
                .long("shell")
                .help("Target shell for command generation")
                .value_name("SHELL")
                .value_parser(["bash", "zsh", "fish", "sh", "powershell", "cmd"]),
        )
        .arg(
            Arg::new("safety")
                .long("safety")
                .help("Safety validation level")
                .value_name("LEVEL")
                .value_parser(["strict", "moderate", "permissive"]),
        )
        .arg(
            Arg::new("execute")
                .short('x')
                .long("execute")
                .help("Execute command after confirmation")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("yes")
                .short('y')
                .long("yes")
                .help("Skip confirmation prompts")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Output in JSON format")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Suppress non-essential output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue),
        )
        .subcommand(Command::new("doctor").about("Run system diagnostics and health checks"))
        .subcommand(
            Command::new("init")
                .about("Generate shell integration script")
                .arg(
                    Arg::new("shell")
                        .help("Shell to generate init script for")
                        .required(true)
                        .value_parser(["bash", "zsh", "fish"]),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("Manage configuration settings")
                .subcommand(Command::new("show").about("Display current configuration"))
                .subcommand(Command::new("path").about("Show configuration file path"))
                .subcommand(Command::new("edit").about("Open configuration in editor"))
                .subcommand(Command::new("reset").about("Reset configuration to defaults")),
        )
        .subcommand(
            Command::new("test")
                .about("Run evaluation tests")
                .arg(
                    Arg::new("backend")
                        .short('b')
                        .long("backend")
                        .help("Backend to test")
                        .default_value("static"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show verbose output")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion scripts")
                .arg(
                    Arg::new("shell")
                        .help("Shell to generate completions for")
                        .required(true)
                        .value_parser(["bash", "zsh", "fish", "powershell"]),
                ),
        )
        .subcommand(
            Command::new("suggest")
                .about("Suggest commands matching a description")
                .arg(
                    Arg::new("query")
                        .help("Partial command description")
                        .required(true),
                )
                .arg(
                    Arg::new("limit")
                        .short('l')
                        .long("limit")
                        .help("Maximum number of suggestions")
                        .default_value("5"),
                ),
        )
}
