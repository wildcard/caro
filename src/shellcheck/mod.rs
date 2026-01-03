//! ShellCheck integration module for post-processing generated commands
//!
//! This module integrates with [ShellCheck](https://github.com/koalaman/shellcheck),
//! a static analysis tool for shell scripts, to validate generated commands and
//! provide feedback for regeneration when issues are detected.
//!
//! # Overview
//!
//! ShellCheck can identify:
//! - Syntax errors
//! - Semantic issues (e.g., unquoted variables)
//! - Common pitfalls (e.g., useless cat)
//! - Style issues
//!
//! # Example
//!
//! ```no_run
//! use caro::shellcheck::{ShellCheckAnalyzer, ShellDialect};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let analyzer = ShellCheckAnalyzer::new();
//!
//! if analyzer.is_available() {
//!     let result = analyzer.analyze("echo $var", ShellDialect::Bash).await?;
//!     if result.needs_regeneration() {
//!         println!("Issues found: {:?}", result.issues);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// ShellCheck issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Style suggestions for better practices
    Style,
    /// Informational notes
    Info,
    /// Warnings about potential issues
    Warning,
    /// Errors that will likely cause problems
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Style => write!(f, "style"),
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// A single issue reported by ShellCheck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCheckIssue {
    /// Source file (will be "-" for stdin)
    pub file: String,
    /// Line number where issue was found
    pub line: u32,
    /// End line (for multi-line issues)
    #[serde(rename = "endLine")]
    pub end_line: Option<u32>,
    /// Column number where issue starts
    pub column: u32,
    /// End column
    #[serde(rename = "endColumn")]
    pub end_column: Option<u32>,
    /// Severity level
    pub level: Severity,
    /// ShellCheck error code (e.g., SC2086)
    pub code: u32,
    /// Human-readable message describing the issue
    pub message: String,
    /// Fix suggestion if available
    #[serde(default)]
    pub fix: Option<ShellCheckFix>,
}

/// Suggested fix for an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCheckFix {
    /// Replacement text
    pub replacements: Vec<ShellCheckReplacement>,
}

/// A single text replacement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCheckReplacement {
    /// Line to replace
    pub line: u32,
    /// End line
    #[serde(rename = "endLine")]
    pub end_line: u32,
    /// Column to start replacement
    pub column: u32,
    /// End column
    #[serde(rename = "endColumn")]
    pub end_column: u32,
    /// Text to insert
    #[serde(rename = "insertionPoint")]
    pub insertion_point: Option<String>,
    /// Replacement text
    pub replacement: String,
}

impl ShellCheckIssue {
    /// Format the issue as a single-line summary for prompts
    pub fn to_prompt_string(&self) -> String {
        format!("SC{} ({}): {}", self.code, self.level, self.message)
    }

    /// Check if this issue is significant enough to warrant regeneration
    pub fn is_significant(&self) -> bool {
        matches!(self.level, Severity::Error | Severity::Warning)
    }
}

/// Shell dialect for ShellCheck analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellDialect {
    /// Bourne shell (sh)
    Sh,
    /// Bourne Again Shell
    Bash,
    /// Debian Almquist Shell
    Dash,
    /// Korn Shell
    Ksh,
}

impl ShellDialect {
    /// Convert to shellcheck's -s flag value
    pub fn as_shellcheck_arg(&self) -> &'static str {
        match self {
            ShellDialect::Sh => "sh",
            ShellDialect::Bash => "bash",
            ShellDialect::Dash => "dash",
            ShellDialect::Ksh => "ksh",
        }
    }
}

impl Default for ShellDialect {
    fn default() -> Self {
        ShellDialect::Bash
    }
}

/// Result of ShellCheck analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Original command that was analyzed
    pub command: String,
    /// Issues found by ShellCheck
    pub issues: Vec<ShellCheckIssue>,
    /// Whether ShellCheck ran successfully
    pub success: bool,
    /// Error message if ShellCheck failed
    pub error: Option<String>,
}

impl AnalysisResult {
    /// Check if the command needs regeneration based on issues found
    pub fn needs_regeneration(&self) -> bool {
        self.issues.iter().any(|issue| issue.is_significant())
    }

    /// Get only significant issues (errors and warnings)
    pub fn significant_issues(&self) -> Vec<&ShellCheckIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.is_significant())
            .collect()
    }

    /// Count issues by severity
    pub fn count_by_severity(&self) -> (usize, usize, usize, usize) {
        let mut errors = 0;
        let mut warnings = 0;
        let mut infos = 0;
        let mut styles = 0;

        for issue in &self.issues {
            match issue.level {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => infos += 1,
                Severity::Style => styles += 1,
            }
        }

        (errors, warnings, infos, styles)
    }

    /// Generate a summary for inclusion in regeneration prompts
    pub fn to_prompt_feedback(&self) -> String {
        if self.issues.is_empty() {
            return String::new();
        }

        let (errors, warnings, infos, styles) = self.count_by_severity();
        let mut feedback = format!(
            "ShellCheck analysis found {} issue(s): {} error(s), {} warning(s), {} info, {} style.\n\n",
            self.issues.len(),
            errors,
            warnings,
            infos,
            styles
        );

        // Include significant issues in the feedback
        feedback.push_str("Issues to address:\n");
        for issue in self.significant_issues() {
            feedback.push_str(&format!("- {}\n", issue.to_prompt_string()));
        }

        feedback
    }

    /// Check if the command is valid (no errors)
    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|issue| matches!(issue.level, Severity::Error))
    }
}

/// Errors that can occur during ShellCheck analysis
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ShellCheckError {
    #[error("ShellCheck is not installed or not found in PATH")]
    NotInstalled,

    #[error("Failed to execute ShellCheck: {message}")]
    ExecutionFailed { message: String },

    #[error("Failed to parse ShellCheck output: {message}")]
    ParseError { message: String },

    #[error("ShellCheck timed out after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("Invalid shell dialect: {dialect}")]
    InvalidDialect { dialect: String },
}

/// ShellCheck analyzer for validating shell commands
#[derive(Debug, Clone)]
pub struct ShellCheckAnalyzer {
    /// Path to shellcheck binary (if found)
    shellcheck_path: Option<String>,
    /// Timeout for analysis in seconds
    timeout_seconds: u64,
    /// Excluded error codes (optional)
    excluded_codes: Vec<u32>,
}

impl Default for ShellCheckAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellCheckAnalyzer {
    /// Create a new ShellCheck analyzer
    pub fn new() -> Self {
        let shellcheck_path = which::which("shellcheck")
            .ok()
            .map(|p| p.to_string_lossy().to_string());

        if shellcheck_path.is_some() {
            info!("ShellCheck found, enabling command validation");
        } else {
            warn!("ShellCheck not found in PATH, command validation disabled");
        }

        Self {
            shellcheck_path,
            timeout_seconds: 5,
            excluded_codes: Vec::new(),
        }
    }

    /// Create analyzer with custom timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Exclude specific ShellCheck error codes
    pub fn with_excluded_codes(mut self, codes: Vec<u32>) -> Self {
        self.excluded_codes = codes;
        self
    }

    /// Check if ShellCheck is available
    pub fn is_available(&self) -> bool {
        self.shellcheck_path.is_some()
    }

    /// Analyze a shell command using ShellCheck
    pub async fn analyze(
        &self,
        command: &str,
        dialect: ShellDialect,
    ) -> Result<AnalysisResult, ShellCheckError> {
        let shellcheck_path = self
            .shellcheck_path
            .as_ref()
            .ok_or(ShellCheckError::NotInstalled)?;

        debug!("Analyzing command with ShellCheck: {}", command);

        // Build the command arguments
        let mut args = vec![
            "-f",
            "json", // JSON output format
            "-s",
            dialect.as_shellcheck_arg(), // Shell dialect
            "-", // Read from stdin
        ];

        // Add excluded codes
        let excluded_str: Vec<String> = self
            .excluded_codes
            .iter()
            .map(|c| format!("SC{}", c))
            .collect();
        for code in &excluded_str {
            args.push("-e");
            args.push(code);
        }

        // Run shellcheck with the command as stdin
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_seconds),
            self.run_shellcheck(shellcheck_path, &args, command),
        )
        .await
        .map_err(|_| ShellCheckError::Timeout {
            seconds: self.timeout_seconds,
        })?;

        result
    }

    /// Execute ShellCheck and parse output
    async fn run_shellcheck(
        &self,
        path: &str,
        args: &[&str],
        input: &str,
    ) -> Result<AnalysisResult, ShellCheckError> {
        use tokio::io::AsyncWriteExt;

        let mut child = Command::new(path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ShellCheckError::ExecutionFailed {
                message: e.to_string(),
            })?;

        // Write command to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .await
                .map_err(|e| ShellCheckError::ExecutionFailed {
                    message: format!("Failed to write to stdin: {}", e),
                })?;
        }

        let output = child.wait_with_output().await.map_err(|e| {
            ShellCheckError::ExecutionFailed {
                message: e.to_string(),
            }
        })?;

        // ShellCheck returns exit code 0 for no issues, 1 for issues found
        // Exit code 2+ indicates an error
        if output.status.code().unwrap_or(2) > 1 {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ShellCheckError::ExecutionFailed {
                message: stderr.to_string(),
            });
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Empty output means no issues
        if stdout.trim().is_empty() || stdout.trim() == "[]" {
            return Ok(AnalysisResult {
                command: input.to_string(),
                issues: Vec::new(),
                success: true,
                error: None,
            });
        }

        let issues: Vec<ShellCheckIssue> =
            serde_json::from_str(&stdout).map_err(|e| ShellCheckError::ParseError {
                message: format!("Failed to parse JSON: {} - Output: {}", e, stdout),
            })?;

        debug!("ShellCheck found {} issue(s)", issues.len());

        Ok(AnalysisResult {
            command: input.to_string(),
            issues,
            success: true,
            error: None,
        })
    }

    /// Quick validation - returns true if command has no errors
    pub async fn is_valid(&self, command: &str, dialect: ShellDialect) -> bool {
        match self.analyze(command, dialect).await {
            Ok(result) => result.is_valid(),
            Err(_) => true, // If ShellCheck fails, assume valid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
        assert!(Severity::Info > Severity::Style);
    }

    #[test]
    fn test_issue_is_significant() {
        let error_issue = ShellCheckIssue {
            file: "-".to_string(),
            line: 1,
            end_line: None,
            column: 1,
            end_column: None,
            level: Severity::Error,
            code: 1000,
            message: "Test error".to_string(),
            fix: None,
        };
        assert!(error_issue.is_significant());

        let style_issue = ShellCheckIssue {
            file: "-".to_string(),
            line: 1,
            end_line: None,
            column: 1,
            end_column: None,
            level: Severity::Style,
            code: 1000,
            message: "Test style".to_string(),
            fix: None,
        };
        assert!(!style_issue.is_significant());
    }

    #[test]
    fn test_analysis_result_needs_regeneration() {
        let result = AnalysisResult {
            command: "echo $var".to_string(),
            issues: vec![ShellCheckIssue {
                file: "-".to_string(),
                line: 1,
                end_line: None,
                column: 6,
                end_column: None,
                level: Severity::Warning,
                code: 2086,
                message: "Double quote to prevent globbing and word splitting.".to_string(),
                fix: None,
            }],
            success: true,
            error: None,
        };

        assert!(result.needs_regeneration());
        assert!(result.is_valid()); // Warnings don't make it invalid
    }

    #[test]
    fn test_shell_dialect_args() {
        assert_eq!(ShellDialect::Bash.as_shellcheck_arg(), "bash");
        assert_eq!(ShellDialect::Sh.as_shellcheck_arg(), "sh");
        assert_eq!(ShellDialect::Dash.as_shellcheck_arg(), "dash");
        assert_eq!(ShellDialect::Ksh.as_shellcheck_arg(), "ksh");
    }

    #[test]
    fn test_prompt_feedback_generation() {
        let result = AnalysisResult {
            command: "echo $var".to_string(),
            issues: vec![
                ShellCheckIssue {
                    file: "-".to_string(),
                    line: 1,
                    end_line: None,
                    column: 6,
                    end_column: None,
                    level: Severity::Warning,
                    code: 2086,
                    message: "Double quote to prevent globbing and word splitting.".to_string(),
                    fix: None,
                },
                ShellCheckIssue {
                    file: "-".to_string(),
                    line: 1,
                    end_line: None,
                    column: 1,
                    end_column: None,
                    level: Severity::Info,
                    code: 2034,
                    message: "var appears unused.".to_string(),
                    fix: None,
                },
            ],
            success: true,
            error: None,
        };

        let feedback = result.to_prompt_feedback();
        assert!(feedback.contains("2 issue(s)"));
        assert!(feedback.contains("SC2086"));
        assert!(feedback.contains("Double quote"));
        // Info issues shouldn't be in the significant issues list
        assert!(!feedback.contains("SC2034"));
    }

    #[test]
    fn test_count_by_severity() {
        let result = AnalysisResult {
            command: "test".to_string(),
            issues: vec![
                ShellCheckIssue {
                    file: "-".to_string(),
                    line: 1,
                    end_line: None,
                    column: 1,
                    end_column: None,
                    level: Severity::Error,
                    code: 1000,
                    message: "Error".to_string(),
                    fix: None,
                },
                ShellCheckIssue {
                    file: "-".to_string(),
                    line: 1,
                    end_line: None,
                    column: 1,
                    end_column: None,
                    level: Severity::Warning,
                    code: 1001,
                    message: "Warning".to_string(),
                    fix: None,
                },
                ShellCheckIssue {
                    file: "-".to_string(),
                    line: 1,
                    end_line: None,
                    column: 1,
                    end_column: None,
                    level: Severity::Warning,
                    code: 1002,
                    message: "Warning 2".to_string(),
                    fix: None,
                },
            ],
            success: true,
            error: None,
        };

        let (errors, warnings, infos, styles) = result.count_by_severity();
        assert_eq!(errors, 1);
        assert_eq!(warnings, 2);
        assert_eq!(infos, 0);
        assert_eq!(styles, 0);
    }

    #[tokio::test]
    async fn test_analyzer_availability() {
        let analyzer = ShellCheckAnalyzer::new();
        // This test just ensures the analyzer can be created
        // Availability depends on whether shellcheck is installed
        let _ = analyzer.is_available();
    }
}
