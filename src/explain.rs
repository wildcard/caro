//! Explain Mode Module
//!
//! This module provides functionality for explaining generated shell commands
//! and their executed outputs in detail. It uses the LLM backend to generate
//! human-readable explanations.
//!
//! # Features
//!
//! - **Command Explanation**: Detailed breakdown of what a command does
//! - **Output Explanation**: Analysis of command execution results
//! - **Error Explanation**: Helpful explanations of error messages
//!
//! # Example
//!
//! ```rust,ignore
//! use caro::explain::{ExplainService, CommandExplanation};
//!
//! let service = ExplainService::new();
//! let explanation = service.explain_output("ls -la", stdout, stderr, exit_code).await?;
//! ```

use crate::backends::CommandGenerator;
use crate::models::ShellType;
use std::sync::Arc;

/// Service for generating explanations of commands and outputs
pub struct ExplainService {
    backend: Option<Arc<dyn CommandGenerator>>,
}

impl ExplainService {
    /// Create a new explain service
    pub fn new() -> Self {
        Self { backend: None }
    }

    /// Create an explain service with a backend for LLM-powered explanations
    pub fn with_backend(backend: Arc<dyn CommandGenerator>) -> Self {
        Self {
            backend: Some(backend),
        }
    }

    /// Explain what a command does
    ///
    /// Returns a detailed explanation of the command's purpose, components,
    /// and any flags or options used.
    pub fn explain_command(&self, command: &str, shell: ShellType) -> CommandExplanation {
        // Parse the command into components
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return CommandExplanation {
                summary: "Empty command".to_string(),
                components: vec![],
                flags_explained: vec![],
                pipeline_stages: vec![],
                safety_notes: vec!["No command provided".to_string()],
                equivalent_commands: vec![],
            };
        }

        // Check for pipeline
        let pipeline_stages: Vec<String> = command
            .split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let is_pipeline = pipeline_stages.len() > 1;

        // Explain each component
        let mut components = Vec::new();
        let mut flags_explained = Vec::new();
        let mut safety_notes = Vec::new();

        for stage in &pipeline_stages {
            let stage_parts: Vec<&str> = stage.split_whitespace().collect();
            if let Some(cmd) = stage_parts.first() {
                let (cmd_desc, cmd_flags, cmd_safety) =
                    self.explain_single_command(cmd, &stage_parts[1..], shell);
                components.push(ComponentExplanation {
                    component: stage.clone(),
                    description: cmd_desc,
                });
                flags_explained.extend(cmd_flags);
                safety_notes.extend(cmd_safety);
            }
        }

        // Generate summary
        let summary = if is_pipeline {
            format!(
                "Pipeline with {} stages: {}",
                pipeline_stages.len(),
                self.summarize_pipeline(&pipeline_stages)
            )
        } else {
            components
                .first()
                .map(|c| c.description.clone())
                .unwrap_or_else(|| "Unknown command".to_string())
        };

        CommandExplanation {
            summary,
            components,
            flags_explained,
            pipeline_stages: if is_pipeline {
                pipeline_stages
                    .iter()
                    .enumerate()
                    .map(|(i, s)| PipelineStage {
                        stage: i + 1,
                        command: s.clone(),
                        purpose: self.stage_purpose(s),
                    })
                    .collect()
            } else {
                vec![]
            },
            safety_notes,
            equivalent_commands: vec![],
        }
    }

    /// Explain the output of an executed command
    ///
    /// Analyzes stdout, stderr, and exit code to provide a human-readable
    /// explanation of what happened.
    pub async fn explain_output(
        &self,
        command: &str,
        stdout: Option<&str>,
        stderr: Option<&str>,
        exit_code: Option<i32>,
    ) -> OutputExplanation {
        let exit_code = exit_code.unwrap_or(-1);
        let stdout = stdout.unwrap_or("");
        let stderr = stderr.unwrap_or("");

        // Determine overall status
        let status = if exit_code == 0 {
            ExecutionStatus::Success
        } else if !stderr.is_empty() {
            ExecutionStatus::Error
        } else {
            ExecutionStatus::Failed
        };

        // Generate summary based on status
        let summary = match status {
            ExecutionStatus::Success => {
                if stdout.is_empty() {
                    "Command completed successfully with no output.".to_string()
                } else {
                    let line_count = stdout.lines().count();
                    if line_count == 1 {
                        "Command completed successfully with 1 line of output.".to_string()
                    } else {
                        format!(
                            "Command completed successfully with {} lines of output.",
                            line_count
                        )
                    }
                }
            }
            ExecutionStatus::Error => format!(
                "Command failed with exit code {}. See error details below.",
                exit_code
            ),
            ExecutionStatus::Failed => {
                format!("Command exited with non-zero status ({}).", exit_code)
            }
        };

        // Analyze stdout
        let stdout_analysis = if !stdout.is_empty() {
            Some(self.analyze_stdout(command, stdout))
        } else {
            None
        };

        // Analyze stderr
        let stderr_analysis = if !stderr.is_empty() {
            Some(self.analyze_stderr(stderr))
        } else {
            None
        };

        // Generate suggestions if there was an error
        let suggestions = if exit_code != 0 {
            self.generate_error_suggestions(command, stderr, exit_code)
        } else {
            vec![]
        };

        OutputExplanation {
            status,
            exit_code,
            summary,
            stdout_analysis,
            stderr_analysis,
            suggestions,
        }
    }

    fn explain_single_command(
        &self,
        cmd: &str,
        args: &[&str],
        shell: ShellType,
    ) -> (String, Vec<FlagExplanation>, Vec<String>) {
        let mut flags = Vec::new();
        let mut safety_notes = Vec::new();

        let description = match cmd {
            "ls" => {
                for arg in args {
                    if arg.starts_with('-') {
                        flags.extend(self.explain_ls_flags(arg));
                    }
                }
                "List directory contents".to_string()
            }
            "find" => {
                for arg in args {
                    flags.extend(self.explain_find_flags(arg, shell));
                }
                "Search for files in a directory hierarchy".to_string()
            }
            "grep" => {
                for arg in args {
                    if arg.starts_with('-') {
                        flags.extend(self.explain_grep_flags(arg));
                    }
                }
                "Search for patterns in files".to_string()
            }
            "rm" => {
                safety_notes.push("WARNING: This command deletes files permanently!".to_string());
                for arg in args {
                    if arg.starts_with('-') {
                        flags.extend(self.explain_rm_flags(arg));
                    }
                }
                "Remove files or directories".to_string()
            }
            "mv" => {
                safety_notes.push("Note: This command moves/renames files".to_string());
                "Move or rename files".to_string()
            }
            "cp" => "Copy files and directories".to_string(),
            "cat" => "Concatenate and display file contents".to_string(),
            "head" => {
                for arg in args {
                    if arg.starts_with('-') {
                        flags.extend(self.explain_head_flags(arg));
                    }
                }
                "Output the first part of files".to_string()
            }
            "tail" => {
                for arg in args {
                    if arg.starts_with('-') {
                        flags.extend(self.explain_tail_flags(arg));
                    }
                }
                "Output the last part of files".to_string()
            }
            "sort" => {
                for arg in args {
                    if arg.starts_with('-') {
                        flags.extend(self.explain_sort_flags(arg));
                    }
                }
                "Sort lines of text".to_string()
            }
            "wc" => {
                for arg in args {
                    if arg.starts_with('-') {
                        flags.extend(self.explain_wc_flags(arg));
                    }
                }
                "Count lines, words, and bytes".to_string()
            }
            "du" => {
                for arg in args {
                    if arg.starts_with('-') {
                        flags.extend(self.explain_du_flags(arg));
                    }
                }
                "Estimate file space usage".to_string()
            }
            "ps" => {
                for arg in args {
                    if !arg.starts_with('-') && arg.chars().all(|c| c.is_alphabetic()) {
                        flags.extend(self.explain_ps_flags(arg));
                    }
                }
                "Report process status".to_string()
            }
            "awk" => "Pattern scanning and processing language".to_string(),
            "sed" => "Stream editor for filtering and transforming text".to_string(),
            "xargs" => {
                for arg in args {
                    if arg.starts_with('-') {
                        flags.extend(self.explain_xargs_flags(arg));
                    }
                }
                "Build and execute command lines from standard input".to_string()
            }
            "chmod" => {
                safety_notes.push("Note: This command changes file permissions".to_string());
                "Change file mode/permissions".to_string()
            }
            "chown" => {
                safety_notes.push("Note: This command changes file ownership".to_string());
                "Change file owner and group".to_string()
            }
            "sudo" => {
                safety_notes.push("WARNING: Command runs with elevated privileges!".to_string());
                "Execute a command as another user (typically root)".to_string()
            }
            "git" => {
                if let Some(subcmd) = args.first() {
                    format!("Git version control: {}", subcmd)
                } else {
                    "Git version control system".to_string()
                }
            }
            "docker" => {
                if let Some(subcmd) = args.first() {
                    format!("Docker container: {}", subcmd)
                } else {
                    "Docker container management".to_string()
                }
            }
            "curl" => "Transfer data from or to a server".to_string(),
            "wget" => "Download files from the web".to_string(),
            "tar" => "Archive utility".to_string(),
            "unzip" => "Extract files from ZIP archives".to_string(),
            "echo" => "Display a line of text".to_string(),
            "pwd" => "Print working directory".to_string(),
            "cd" => "Change directory".to_string(),
            "mkdir" => "Create directories".to_string(),
            "touch" => "Create empty files or update timestamps".to_string(),
            "stat" => "Display file status".to_string(),
            "file" => "Determine file type".to_string(),
            "cut" => "Remove sections from lines".to_string(),
            "tr" => "Translate or delete characters".to_string(),
            "uniq" => "Report or omit repeated lines".to_string(),
            "tee" => "Read stdin and write to stdout and files".to_string(),
            "diff" => "Compare files line by line".to_string(),
            "comm" => "Compare two sorted files line by line".to_string(),
            "less" | "more" => "View file contents with pagination".to_string(),
            "netstat" => "Network statistics".to_string(),
            "lsof" => "List open files".to_string(),
            "ss" => "Socket statistics".to_string(),
            "kill" => {
                safety_notes.push("Note: This command terminates processes".to_string());
                "Send signal to a process".to_string()
            }
            "killall" => {
                safety_notes.push("WARNING: This command can terminate multiple processes!".to_string());
                "Kill processes by name".to_string()
            }
            _ => format!("Execute '{}'", cmd),
        };

        (description, flags, safety_notes)
    }

    fn explain_ls_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        let mut explanations = Vec::new();
        for c in flag.chars().skip(1) {
            let explanation = match c {
                'l' => Some(("l", "Long format with detailed information")),
                'a' => Some(("a", "Show hidden files (starting with .)")),
                'h' => Some(("h", "Human-readable file sizes (K, M, G)")),
                'r' => Some(("r", "Reverse sort order")),
                't' => Some(("t", "Sort by modification time")),
                'S' => Some(("S", "Sort by file size")),
                'R' => Some(("R", "List subdirectories recursively")),
                '1' => Some(("1", "One entry per line")),
                _ => None,
            };
            if let Some((flag, desc)) = explanation {
                explanations.push(FlagExplanation {
                    flag: format!("-{}", flag),
                    description: desc.to_string(),
                });
            }
        }
        explanations
    }

    fn explain_find_flags(&self, arg: &str, _shell: ShellType) -> Vec<FlagExplanation> {
        match arg {
            "-type" => vec![FlagExplanation {
                flag: "-type".to_string(),
                description: "Filter by type (f=file, d=directory, l=symlink)".to_string(),
            }],
            "-name" => vec![FlagExplanation {
                flag: "-name".to_string(),
                description: "Match filename pattern (case-sensitive)".to_string(),
            }],
            "-iname" => vec![FlagExplanation {
                flag: "-iname".to_string(),
                description: "Match filename pattern (case-insensitive)".to_string(),
            }],
            "-size" => vec![FlagExplanation {
                flag: "-size".to_string(),
                description: "Filter by file size (e.g., +100M for >100MB)".to_string(),
            }],
            "-mtime" => vec![FlagExplanation {
                flag: "-mtime".to_string(),
                description: "Filter by modification time in days".to_string(),
            }],
            "-mmin" => vec![FlagExplanation {
                flag: "-mmin".to_string(),
                description: "Filter by modification time in minutes".to_string(),
            }],
            "-exec" => vec![FlagExplanation {
                flag: "-exec".to_string(),
                description: "Execute command on each found file".to_string(),
            }],
            "-print0" => vec![FlagExplanation {
                flag: "-print0".to_string(),
                description: "Print null-separated (safe for filenames with spaces)".to_string(),
            }],
            "-printf" => vec![FlagExplanation {
                flag: "-printf".to_string(),
                description: "Print with custom format (GNU extension)".to_string(),
            }],
            _ => vec![],
        }
    }

    fn explain_grep_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        let mut explanations = Vec::new();

        // Handle long flags
        if flag.starts_with("--") {
            match flag {
                "--include" => return vec![FlagExplanation {
                    flag: "--include".to_string(),
                    description: "Search only files matching pattern".to_string(),
                }],
                "--exclude" => return vec![FlagExplanation {
                    flag: "--exclude".to_string(),
                    description: "Skip files matching pattern".to_string(),
                }],
                _ => return vec![],
            }
        }

        for c in flag.chars().skip(1) {
            let explanation = match c {
                'r' | 'R' => Some(("r/R", "Recursive search in directories")),
                'i' => Some(("i", "Case-insensitive matching")),
                'n' => Some(("n", "Show line numbers")),
                'l' => Some(("l", "Show only filenames with matches")),
                'c' => Some(("c", "Count matching lines")),
                'v' => Some(("v", "Invert match (show non-matching lines)")),
                'E' => Some(("E", "Extended regular expressions")),
                'P' => Some(("P", "Perl-compatible regular expressions")),
                'o' => Some(("o", "Show only the matching part")),
                'w' => Some(("w", "Match whole words only")),
                _ => None,
            };
            if let Some((flag, desc)) = explanation {
                explanations.push(FlagExplanation {
                    flag: format!("-{}", flag),
                    description: desc.to_string(),
                });
            }
        }
        explanations
    }

    fn explain_rm_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        let mut explanations = Vec::new();
        for c in flag.chars().skip(1) {
            let explanation = match c {
                'r' | 'R' => Some(("r", "Remove directories recursively")),
                'f' => Some(("f", "Force removal without confirmation")),
                'i' => Some(("i", "Prompt before each removal")),
                'v' => Some(("v", "Verbose - explain what is being done")),
                _ => None,
            };
            if let Some((flag, desc)) = explanation {
                explanations.push(FlagExplanation {
                    flag: format!("-{}", flag),
                    description: desc.to_string(),
                });
            }
        }
        explanations
    }

    fn explain_head_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        if flag.starts_with("-n") {
            vec![FlagExplanation {
                flag: "-n".to_string(),
                description: "Number of lines to show".to_string(),
            }]
        } else {
            vec![]
        }
    }

    fn explain_tail_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        let mut explanations = Vec::new();
        if flag.starts_with("-n") {
            explanations.push(FlagExplanation {
                flag: "-n".to_string(),
                description: "Number of lines to show".to_string(),
            });
        }
        if flag.contains('f') {
            explanations.push(FlagExplanation {
                flag: "-f".to_string(),
                description: "Follow - output appended data as file grows".to_string(),
            });
        }
        explanations
    }

    fn explain_sort_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        let mut explanations = Vec::new();

        // Handle long flags
        if flag.starts_with("--") {
            return vec![];
        }

        // Handle -k flag with field number
        if flag.starts_with("-k") {
            return vec![FlagExplanation {
                flag: "-k".to_string(),
                description: "Sort by specific field/column".to_string(),
            }];
        }

        for c in flag.chars().skip(1) {
            let explanation = match c {
                'n' => Some(("n", "Numeric sort")),
                'r' => Some(("r", "Reverse sort order")),
                'h' => Some(("h", "Human-readable numbers (2K, 1G)")),
                'u' => Some(("u", "Unique - remove duplicate lines")),
                't' => Some(("t", "Field separator character")),
                _ => None,
            };
            if let Some((flag, desc)) = explanation {
                explanations.push(FlagExplanation {
                    flag: format!("-{}", flag),
                    description: desc.to_string(),
                });
            }
        }
        explanations
    }

    fn explain_wc_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        let mut explanations = Vec::new();
        for c in flag.chars().skip(1) {
            let explanation = match c {
                'l' => Some(("l", "Count lines")),
                'w' => Some(("w", "Count words")),
                'c' => Some(("c", "Count bytes")),
                'm' => Some(("m", "Count characters")),
                _ => None,
            };
            if let Some((flag, desc)) = explanation {
                explanations.push(FlagExplanation {
                    flag: format!("-{}", flag),
                    description: desc.to_string(),
                });
            }
        }
        explanations
    }

    fn explain_du_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        let mut explanations = Vec::new();

        // Handle long flags
        if flag == "--max-depth" {
            return vec![FlagExplanation {
                flag: "--max-depth".to_string(),
                description: "Maximum depth of directories to show".to_string(),
            }];
        }

        for c in flag.chars().skip(1) {
            let explanation = match c {
                's' => Some(("s", "Display only a total for each argument")),
                'h' => Some(("h", "Human-readable sizes (K, M, G)")),
                'a' => Some(("a", "Show all files, not just directories")),
                'd' => Some(("d", "Max depth of directories")),
                _ => None,
            };
            if let Some((flag, desc)) = explanation {
                explanations.push(FlagExplanation {
                    flag: format!("-{}", flag),
                    description: desc.to_string(),
                });
            }
        }
        explanations
    }

    fn explain_ps_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        let mut explanations = Vec::new();
        for c in flag.chars() {
            let explanation = match c {
                'a' => Some(("a", "All users' processes")),
                'u' => Some(("u", "User-oriented format with more details")),
                'x' => Some(("x", "Include processes without controlling terminal")),
                'e' => Some(("e", "Show environment")),
                'f' => Some(("f", "Full format listing")),
                _ => None,
            };
            if let Some((flag, desc)) = explanation {
                explanations.push(FlagExplanation {
                    flag: flag.to_string(),
                    description: desc.to_string(),
                });
            }
        }
        explanations
    }

    fn explain_xargs_flags(&self, flag: &str) -> Vec<FlagExplanation> {
        let mut explanations = Vec::new();
        if flag == "-0" {
            explanations.push(FlagExplanation {
                flag: "-0".to_string(),
                description: "Input items separated by null character".to_string(),
            });
        } else if flag == "-I" {
            explanations.push(FlagExplanation {
                flag: "-I".to_string(),
                description: "Replace string placeholder".to_string(),
            });
        } else if flag == "-n" {
            explanations.push(FlagExplanation {
                flag: "-n".to_string(),
                description: "Maximum arguments per command line".to_string(),
            });
        }
        explanations
    }

    fn summarize_pipeline(&self, stages: &[String]) -> String {
        stages
            .iter()
            .map(|s| {
                let cmd = s.split_whitespace().next().unwrap_or("");
                cmd.to_string()
            })
            .collect::<Vec<_>>()
            .join(" -> ")
    }

    fn stage_purpose(&self, stage: &str) -> String {
        let cmd = stage.split_whitespace().next().unwrap_or("");
        match cmd {
            "find" => "Find files matching criteria".to_string(),
            "grep" => "Filter lines matching pattern".to_string(),
            "sort" => "Sort the output".to_string(),
            "head" => "Take first N items".to_string(),
            "tail" => "Take last N items".to_string(),
            "wc" => "Count items".to_string(),
            "cut" => "Extract specific fields".to_string(),
            "awk" => "Process and transform data".to_string(),
            "sed" => "Edit/transform text".to_string(),
            "xargs" => "Pass arguments to another command".to_string(),
            "uniq" => "Remove duplicate lines".to_string(),
            "tr" => "Translate characters".to_string(),
            "tee" => "Duplicate output to file".to_string(),
            _ => format!("Execute {}", cmd),
        }
    }

    fn analyze_stdout(&self, command: &str, stdout: &str) -> StdoutAnalysis {
        let lines: Vec<&str> = stdout.lines().collect();
        let line_count = lines.len();

        // Detect output type based on command and content
        let output_type = self.detect_output_type(command, &lines);

        // Generate insights based on output type
        let insights = self.generate_stdout_insights(command, &lines, &output_type);

        StdoutAnalysis {
            line_count,
            output_type,
            insights,
            sample_lines: if line_count > 5 {
                lines[..5].iter().map(|s| s.to_string()).collect()
            } else {
                lines.iter().map(|s| s.to_string()).collect()
            },
        }
    }

    fn detect_output_type(&self, command: &str, lines: &[&str]) -> OutputType {
        let cmd = command.split_whitespace().next().unwrap_or("");

        match cmd {
            "ls" => OutputType::FileList,
            "find" => OutputType::FileList,
            "ps" => OutputType::ProcessList,
            "du" => OutputType::DiskUsage,
            "df" => OutputType::DiskUsage,
            "grep" => OutputType::SearchResults,
            "wc" => OutputType::Count,
            "git" => {
                if command.contains("log") {
                    OutputType::GitLog
                } else if command.contains("status") {
                    OutputType::GitStatus
                } else {
                    OutputType::Text
                }
            }
            _ => {
                // Try to detect by content
                if lines.iter().any(|l| l.contains("total") && l.contains("K")) {
                    OutputType::DiskUsage
                } else if lines.iter().any(|l| l.starts_with("drwx") || l.starts_with("-rw")) {
                    OutputType::FileList
                } else if lines.iter().any(|l| l.contains("PID") || l.contains("CPU")) {
                    OutputType::ProcessList
                } else {
                    OutputType::Text
                }
            }
        }
    }

    fn generate_stdout_insights(&self, _command: &str, lines: &[&str], output_type: &OutputType) -> Vec<String> {
        let mut insights = Vec::new();

        match output_type {
            OutputType::FileList => {
                let file_count = lines.len();
                insights.push(format!("Found {} items", file_count));

                // Check for any directories
                let dir_count = lines.iter().filter(|l| l.starts_with('d') || l.ends_with('/')).count();
                if dir_count > 0 {
                    insights.push(format!("{} directories", dir_count));
                }
            }
            OutputType::ProcessList => {
                let process_count = lines.len().saturating_sub(1); // Subtract header
                insights.push(format!("{} processes found", process_count));
            }
            OutputType::DiskUsage => {
                insights.push("Showing disk space usage".to_string());
            }
            OutputType::SearchResults => {
                let match_count = lines.len();
                insights.push(format!("{} matches found", match_count));
            }
            OutputType::Count => {
                if let Some(line) = lines.first() {
                    insights.push(format!("Count: {}", line.trim()));
                }
            }
            _ => {
                if !lines.is_empty() {
                    insights.push(format!("{} lines of output", lines.len()));
                }
            }
        }

        insights
    }

    fn analyze_stderr(&self, stderr: &str) -> StderrAnalysis {
        let _lines: Vec<&str> = stderr.lines().collect();

        // Categorize the error
        let error_type = self.categorize_error(stderr);

        // Generate helpful message
        let explanation = self.explain_error(&error_type, stderr);

        StderrAnalysis {
            error_type,
            explanation,
            raw_message: stderr.to_string(),
        }
    }

    fn categorize_error(&self, stderr: &str) -> ErrorType {
        let stderr_lower = stderr.to_lowercase();

        if stderr_lower.contains("permission denied") {
            ErrorType::PermissionDenied
        } else if stderr_lower.contains("no such file or directory") {
            ErrorType::FileNotFound
        } else if stderr_lower.contains("command not found") {
            ErrorType::CommandNotFound
        } else if stderr_lower.contains("invalid option") || stderr_lower.contains("unrecognized option") {
            ErrorType::InvalidOption
        } else if stderr_lower.contains("syntax error") {
            ErrorType::SyntaxError
        } else if stderr_lower.contains("disk full") || stderr_lower.contains("no space left") {
            ErrorType::DiskFull
        } else if stderr_lower.contains("connection refused") || stderr_lower.contains("network") {
            ErrorType::NetworkError
        } else if stderr_lower.contains("timeout") {
            ErrorType::Timeout
        } else {
            ErrorType::Unknown
        }
    }

    fn explain_error(&self, error_type: &ErrorType, stderr: &str) -> String {
        match error_type {
            ErrorType::PermissionDenied => {
                "Access denied. You may need elevated privileges (try sudo) or check file permissions.".to_string()
            }
            ErrorType::FileNotFound => {
                "The specified file or directory does not exist. Check the path and filename.".to_string()
            }
            ErrorType::CommandNotFound => {
                "The command is not installed or not in PATH. Install the required package or check spelling.".to_string()
            }
            ErrorType::InvalidOption => {
                "An unrecognized flag or option was used. Check the command syntax with --help.".to_string()
            }
            ErrorType::SyntaxError => {
                "There is a syntax error in the command. Check quotes, brackets, and special characters.".to_string()
            }
            ErrorType::DiskFull => {
                "No disk space available. Free up space or use a different location.".to_string()
            }
            ErrorType::NetworkError => {
                "Network connection issue. Check your internet connection or the server address.".to_string()
            }
            ErrorType::Timeout => {
                "The operation timed out. The server may be slow or unreachable.".to_string()
            }
            ErrorType::Unknown => {
                format!("An error occurred: {}", stderr.lines().next().unwrap_or("Unknown error"))
            }
        }
    }

    fn generate_error_suggestions(&self, command: &str, stderr: &str, _exit_code: i32) -> Vec<String> {
        let mut suggestions = Vec::new();
        let stderr_lower = stderr.to_lowercase();
        let cmd = command.split_whitespace().next().unwrap_or("");

        if stderr_lower.contains("permission denied") {
            suggestions.push(format!("Try: sudo {}", command));
            suggestions.push("Check file permissions with: ls -la <path>".to_string());
        }

        if stderr_lower.contains("no such file or directory") {
            suggestions.push("Verify the file exists with: ls <path>".to_string());
            suggestions.push("Check for typos in the path".to_string());
        }

        if stderr_lower.contains("command not found") {
            suggestions.push(format!("Install {} using your package manager", cmd));
            suggestions.push(format!("Check if {} is in your PATH", cmd));
        }

        if stderr_lower.contains("invalid option") {
            suggestions.push(format!("Check available options: {} --help", cmd));
            suggestions.push("The flag may not be supported on this system".to_string());
        }

        suggestions
    }
}

impl Default for ExplainService {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed explanation of a command
#[derive(Debug, Clone)]
pub struct CommandExplanation {
    /// One-line summary of what the command does
    pub summary: String,
    /// Breakdown of each component
    pub components: Vec<ComponentExplanation>,
    /// Explanation of flags/options used
    pub flags_explained: Vec<FlagExplanation>,
    /// Pipeline stages if command uses pipes
    pub pipeline_stages: Vec<PipelineStage>,
    /// Any safety warnings or notes
    pub safety_notes: Vec<String>,
    /// Alternative commands that achieve similar results
    pub equivalent_commands: Vec<String>,
}

/// Explanation of a single command component
#[derive(Debug, Clone)]
pub struct ComponentExplanation {
    /// The component (e.g., "ls -la" or "grep -r 'pattern'")
    pub component: String,
    /// What this component does
    pub description: String,
}

/// Explanation of a flag or option
#[derive(Debug, Clone)]
pub struct FlagExplanation {
    /// The flag (e.g., "-l", "--verbose")
    pub flag: String,
    /// What the flag does
    pub description: String,
}

/// Description of a pipeline stage
#[derive(Debug, Clone)]
pub struct PipelineStage {
    /// Stage number (1-indexed)
    pub stage: usize,
    /// The command at this stage
    pub command: String,
    /// Purpose of this stage
    pub purpose: String,
}

/// Explanation of command output
#[derive(Debug, Clone)]
pub struct OutputExplanation {
    /// Overall status of execution
    pub status: ExecutionStatus,
    /// Exit code from the command
    pub exit_code: i32,
    /// Summary of what happened
    pub summary: String,
    /// Analysis of stdout if present
    pub stdout_analysis: Option<StdoutAnalysis>,
    /// Analysis of stderr if present
    pub stderr_analysis: Option<StderrAnalysis>,
    /// Suggestions for next steps or fixes
    pub suggestions: Vec<String>,
}

/// Status of command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    Failed,
    Error,
}

/// Analysis of stdout content
#[derive(Debug, Clone)]
pub struct StdoutAnalysis {
    /// Number of output lines
    pub line_count: usize,
    /// Detected type of output
    pub output_type: OutputType,
    /// Key insights from the output
    pub insights: Vec<String>,
    /// Sample of first few lines
    pub sample_lines: Vec<String>,
}

/// Type of output detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputType {
    FileList,
    ProcessList,
    DiskUsage,
    SearchResults,
    Count,
    GitLog,
    GitStatus,
    Text,
}

/// Analysis of stderr content
#[derive(Debug, Clone)]
pub struct StderrAnalysis {
    /// Type of error detected
    pub error_type: ErrorType,
    /// Human-readable explanation
    pub explanation: String,
    /// Raw error message
    pub raw_message: String,
}

/// Type of error detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    PermissionDenied,
    FileNotFound,
    CommandNotFound,
    InvalidOption,
    SyntaxError,
    DiskFull,
    NetworkError,
    Timeout,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_simple_command() {
        let service = ExplainService::new();
        let explanation = service.explain_command("ls -la", ShellType::Bash);

        assert!(explanation.summary.contains("List"));
        assert!(!explanation.flags_explained.is_empty());
    }

    #[test]
    fn test_explain_pipeline() {
        let service = ExplainService::new();
        let explanation = service.explain_command("find . -type f | wc -l", ShellType::Bash);

        assert!(explanation.summary.contains("Pipeline"));
        assert_eq!(explanation.pipeline_stages.len(), 2);
    }

    #[test]
    fn test_explain_dangerous_command() {
        let service = ExplainService::new();
        let explanation = service.explain_command("rm -rf /tmp/test", ShellType::Bash);

        assert!(!explanation.safety_notes.is_empty());
        assert!(explanation.safety_notes[0].contains("WARNING"));
    }

    #[tokio::test]
    async fn test_explain_successful_output() {
        let service = ExplainService::new();
        let explanation = service.explain_output(
            "ls -la",
            Some("total 0\nfile1.txt\nfile2.txt"),
            None,
            Some(0),
        ).await;

        assert_eq!(explanation.status, ExecutionStatus::Success);
        assert!(explanation.stdout_analysis.is_some());
    }

    #[tokio::test]
    async fn test_explain_error_output() {
        let service = ExplainService::new();
        let explanation = service.explain_output(
            "ls /nonexistent",
            None,
            Some("ls: cannot access '/nonexistent': No such file or directory"),
            Some(2),
        ).await;

        assert_eq!(explanation.status, ExecutionStatus::Error);
        assert!(explanation.stderr_analysis.is_some());
        let stderr = explanation.stderr_analysis.unwrap();
        assert_eq!(stderr.error_type, ErrorType::FileNotFound);
    }

    #[test]
    fn test_categorize_permission_error() {
        let service = ExplainService::new();
        let error_type = service.categorize_error("bash: /root/test: Permission denied");
        assert_eq!(error_type, ErrorType::PermissionDenied);
    }

    #[test]
    fn test_categorize_command_not_found() {
        let service = ExplainService::new();
        let error_type = service.categorize_error("foobar: command not found");
        assert_eq!(error_type, ErrorType::CommandNotFound);
    }
}
