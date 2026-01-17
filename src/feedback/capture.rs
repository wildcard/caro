//! Context Capture Engine
//!
//! This module provides functionality to capture rich context from the user's
//! environment when submitting feedback. This includes:
//!
//! - Operating system and architecture information
//! - Shell and terminal environment
//! - Command generation context
//! - Git repository information
//! - System state (available backends, paths, etc.)

use crate::feedback::types::*;
use crate::feedback::FeedbackError;
use chrono::Utc;
use std::env;
use std::path::PathBuf;
use std::process::Command;

// =============================================================================
// Public API
// =============================================================================

/// Capture complete feedback context from current environment
///
/// This function gathers all relevant context information that will be useful
/// for debugging and understanding the user's environment when they submit
/// feedback.
///
/// # Arguments
/// * `user_prompt` - The original natural language prompt from the user
/// * `generated_command` - The shell command that was generated
/// * `backend` - The backend used for generation
/// * `error` - Optional error that occurred
/// * `command_history` - Recent command history
///
/// # Returns
/// Result containing the complete FeedbackContext or an error
pub fn capture_context(
    user_prompt: &str,
    generated_command: &str,
    backend: &str,
    error: Option<&dyn std::error::Error>,
    command_history: &[HistoryEntry],
) -> Result<FeedbackContext, FeedbackError> {
    Ok(FeedbackContext {
        timestamp: Utc::now(),
        cmdai_version: env!("CARGO_PKG_VERSION").to_string(),
        environment: capture_environment_info()?,
        command_info: capture_command_info(user_prompt, generated_command, backend, command_history),
        error_info: error.map(capture_error_info),
        system_state: capture_system_state()?,
        git_context: capture_git_context().ok(),
    })
}

/// Capture environment information from the system
pub fn capture_environment_info() -> Result<EnvironmentInfo, FeedbackError> {
    let os = env::consts::OS.to_string();
    let arch = env::consts::ARCH.to_string();

    // Get OS version
    let os_version = get_os_version().unwrap_or_else(|| "unknown".to_string());

    // Get shell from environment
    let shell = env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "unknown".to_string());

    // Get terminal from environment
    let terminal = detect_terminal();

    // Get Rust version (optional)
    let rust_version = get_rust_version();

    Ok(EnvironmentInfo {
        os,
        os_version,
        arch,
        shell,
        terminal,
        rust_version,
    })
}

/// Capture command-related information
fn capture_command_info(
    user_prompt: &str,
    generated_command: &str,
    backend: &str,
    command_history: &[HistoryEntry],
) -> CommandInfo {
    // Limit command history to prevent data leaks
    const MAX_HISTORY_ENTRIES: usize = 5;

    let limited_history = command_history
        .iter()
        .rev()
        .take(MAX_HISTORY_ENTRIES)
        .cloned()
        .collect();

    CommandInfo {
        user_prompt: user_prompt.to_string(),
        generated_command: generated_command.to_string(),
        backend: backend.to_string(),
        model: detect_model_info(),
        command_history: limited_history,
    }
}

/// Capture error information from an error
fn capture_error_info(error: &dyn std::error::Error) -> ErrorInfo {
    ErrorInfo {
        exit_code: None,
        stderr: String::new(),
        stdout: String::new(),
        error_message: error.to_string(),
        error_type: Some(std::any::type_name_of_val(error).to_string()),
    }
}

/// Capture error information with full output details
pub fn capture_error_info_with_output(
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    error_message: String,
) -> ErrorInfo {
    ErrorInfo {
        exit_code,
        stderr,
        stdout,
        error_message,
        error_type: None,
    }
}

/// Capture system state information
fn capture_system_state() -> Result<SystemState, FeedbackError> {
    // Detect available backends
    let available_backends = detect_available_backends();

    // Get cache directory
    let cache_dir = get_cache_dir();

    // Get config file path
    let config_file = get_config_file_path();

    // Detect CI environment
    let is_ci = detect_ci_environment();

    // Check if running interactively
    let is_interactive = is_terminal_interactive();

    Ok(SystemState {
        available_backends,
        cache_dir,
        config_file,
        is_ci,
        is_interactive,
    })
}

/// Capture Git repository context if in a repo
pub fn capture_git_context() -> Result<GitContext, FeedbackError> {
    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| FeedbackError::CaptureError(format!("Failed to check git status: {}", e)))?;

    if !status.status.success() {
        return Err(FeedbackError::CaptureError(
            "Not in a git repository".to_string(),
        ));
    }

    // Get current branch
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok();

    let current_branch = branch_output
        .as_ref()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout.clone()).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "HEAD".to_string());

    // Get remote URL (sanitize to remove credentials)
    let remote_output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok();

    let repo_url = remote_output
        .as_ref()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout.clone()).ok())
        .map(|s| sanitize_git_url(s.trim()));

    // Check for uncommitted changes
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok();

    let has_uncommitted_changes = status_output
        .as_ref()
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    // Get last commit hash
    let hash_output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok();

    let last_commit_hash = hash_output
        .as_ref()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout.clone()).ok())
        .map(|s| s.trim().to_string());

    Ok(GitContext {
        repo_url,
        current_branch,
        has_uncommitted_changes,
        last_commit_hash,
    })
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Get OS version based on current platform
fn get_os_version() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("sw_vers")
            .args(["-productVersion"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    }

    #[cfg(target_os = "linux")]
    {
        // Try to read from /etc/os-release
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("VERSION_ID="))
                    .map(|line| line.trim_start_matches("VERSION_ID=").trim_matches('"').to_string())
            })
            .or_else(|| {
                // Fall back to uname
                Command::new("uname")
                    .args(["-r"])
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
            })
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Detect the terminal emulator being used
fn detect_terminal() -> String {
    // Check common terminal environment variables
    let terminal_vars = [
        ("TERM_PROGRAM", None),
        ("TERMINAL_EMULATOR", None),
        ("KONSOLE_VERSION", Some("Konsole")),
        ("GNOME_TERMINAL_SCREEN", Some("gnome-terminal")),
        ("ITERM_SESSION_ID", Some("iTerm.app")),
        ("KITTY_WINDOW_ID", Some("kitty")),
        ("ALACRITTY_LOG", Some("alacritty")),
        ("WEZTERM_PANE", Some("WezTerm")),
    ];

    for (var, fixed_name) in terminal_vars {
        if let Ok(value) = env::var(var) {
            if let Some(name) = fixed_name {
                return name.to_string();
            }
            if !value.is_empty() {
                return value;
            }
        }
    }

    // Fall back to TERM variable
    env::var("TERM").unwrap_or_else(|_| "unknown".to_string())
}

/// Get Rust version if available
fn get_rust_version() -> Option<String> {
    Command::new("rustc")
        .args(["--version"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| {
            // Parse "rustc X.Y.Z (hash date)" to just "X.Y.Z"
            s.split_whitespace()
                .nth(1)
                .unwrap_or(s.trim())
                .to_string()
        })
}

/// Detect available inference backends
fn detect_available_backends() -> Vec<String> {
    let mut backends = vec!["static".to_string()];

    // Check for embedded backend
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        backends.push("mlx".to_string());
    }

    backends.push("embedded".to_string());

    // Check for Ollama
    if Command::new("ollama")
        .args(["--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        backends.push("ollama".to_string());
    }

    backends
}

/// Get the cache directory path
fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .map(|p| p.join("caro"))
        .unwrap_or_else(|| PathBuf::from("/tmp/caro/cache"))
}

/// Get the config file path if it exists
fn get_config_file_path() -> Option<PathBuf> {
    let config_path = dirs::config_dir()?.join("caro").join("config.toml");

    if config_path.exists() {
        Some(config_path)
    } else {
        None
    }
}

/// Detect if running in a CI environment
fn detect_ci_environment() -> bool {
    // Check common CI environment variables
    let ci_vars = [
        "CI",
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "CIRCLECI",
        "TRAVIS",
        "JENKINS_URL",
        "BUILDKITE",
        "TEAMCITY_VERSION",
    ];

    ci_vars.iter().any(|var| env::var(var).is_ok())
}

/// Check if the terminal is interactive (TTY)
fn is_terminal_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

/// Detect model information from configuration or environment
fn detect_model_info() -> Option<String> {
    // Try to get from environment variable
    env::var("CARO_MODEL").ok().or_else(|| {
        // Could also check config file, but keep it simple for now
        None
    })
}

/// Sanitize git URL to remove any embedded credentials
fn sanitize_git_url(url: &str) -> String {
    // Remove credentials from URLs like https://user:pass@github.com/...
    if url.contains('@') && url.starts_with("https://") {
        if let Some(at_pos) = url.find('@') {
            return format!("https://{}", &url[at_pos + 1..]);
        }
    }
    url.to_string()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // =========================================================================
    // Environment Info Tests
    // =========================================================================

    #[test]
    fn test_capture_environment_info() {
        let env_info = capture_environment_info().expect("Should capture environment");

        // OS should be detected
        assert!(!env_info.os.is_empty(), "OS should not be empty");
        assert!(
            ["macos", "linux", "windows"].contains(&env_info.os.as_str()),
            "OS should be a known value"
        );

        // Arch should be detected
        assert!(!env_info.arch.is_empty(), "Arch should not be empty");
        assert!(
            ["x86_64", "aarch64", "arm64", "arm"].contains(&env_info.arch.as_str())
                || env_info.arch.starts_with("arm"),
            "Arch should be a known value: {}",
            env_info.arch
        );

        // Shell should be detected (might be "unknown" in some test environments)
        assert!(!env_info.shell.is_empty(), "Shell should not be empty");
    }

    #[test]
    fn test_capture_environment_detects_shell() {
        // Save original SHELL
        let original_shell = env::var("SHELL").ok();

        // Set a known shell
        env::set_var("SHELL", "/bin/bash");

        let env_info = capture_environment_info().expect("Should capture environment");
        assert_eq!(env_info.shell, "bash");

        // Restore original
        if let Some(shell) = original_shell {
            env::set_var("SHELL", shell);
        } else {
            env::remove_var("SHELL");
        }
    }

    // =========================================================================
    // Command Info Tests
    // =========================================================================

    #[test]
    fn test_capture_command_info() {
        let history = vec![
            HistoryEntry {
                prompt: "list files".to_string(),
                command: "ls".to_string(),
                timestamp: Utc::now(),
                success: true,
            },
            HistoryEntry {
                prompt: "find large files".to_string(),
                command: "find . -size +100M".to_string(),
                timestamp: Utc::now(),
                success: true,
            },
        ];

        let cmd_info = capture_command_info("test prompt", "echo test", "static", &history);

        assert_eq!(cmd_info.user_prompt, "test prompt");
        assert_eq!(cmd_info.generated_command, "echo test");
        assert_eq!(cmd_info.backend, "static");
        assert_eq!(cmd_info.command_history.len(), 2);
    }

    #[test]
    fn test_command_history_limited() {
        // Create more than MAX_HISTORY_ENTRIES
        let mut history = Vec::new();
        for i in 0..10 {
            history.push(HistoryEntry {
                prompt: format!("prompt {}", i),
                command: format!("cmd {}", i),
                timestamp: Utc::now(),
                success: true,
            });
        }

        let cmd_info = capture_command_info("test", "test", "static", &history);

        // Should be limited to 5 entries
        assert_eq!(cmd_info.command_history.len(), 5);
    }

    // =========================================================================
    // Error Info Tests
    // =========================================================================

    #[test]
    fn test_capture_error_info_from_error() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error_info = capture_error_info(&error);

        assert_eq!(error_info.error_message, "File not found");
        assert!(error_info.error_type.is_some());
    }

    #[test]
    fn test_capture_error_info_with_output() {
        let error_info = capture_error_info_with_output(
            Some(1),
            "stdout content".to_string(),
            "stderr content".to_string(),
            "Command failed".to_string(),
        );

        assert_eq!(error_info.exit_code, Some(1));
        assert_eq!(error_info.stdout, "stdout content");
        assert_eq!(error_info.stderr, "stderr content");
        assert_eq!(error_info.error_message, "Command failed");
    }

    // =========================================================================
    // System State Tests
    // =========================================================================

    #[test]
    fn test_capture_system_state() {
        let state = capture_system_state().expect("Should capture system state");

        // Should have at least static backend
        assert!(
            state.available_backends.contains(&"static".to_string()),
            "Static backend should be available"
        );

        // Cache dir should be set
        assert!(!state.cache_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_detect_ci_environment() {
        // Save and clear CI vars
        let ci_vars = ["CI", "GITHUB_ACTIONS", "GITLAB_CI"];
        let saved: Vec<_> = ci_vars.iter().map(|v| (v, env::var(v).ok())).collect();

        for var in &ci_vars {
            env::remove_var(var);
        }

        // Should be false without CI vars
        assert!(!detect_ci_environment() || env::var("JENKINS_URL").is_ok());

        // Set CI var
        env::set_var("CI", "true");
        assert!(detect_ci_environment());

        // Restore
        env::remove_var("CI");
        for (var, value) in saved {
            if let Some(v) = value {
                env::set_var(var, v);
            }
        }
    }

    // =========================================================================
    // Git Context Tests
    // =========================================================================

    #[test]
    fn test_git_context_capture() {
        // This test will pass in a git repo and return error outside
        let result = capture_git_context();

        // In this project directory, we should be in a git repo
        if result.is_ok() {
            let ctx = result.unwrap();
            assert!(!ctx.current_branch.is_empty());
            // Commit hash might be None in edge cases, but branch should exist
        }
        // If not in a git repo, the error case is also valid
    }

    #[test]
    fn test_sanitize_git_url() {
        // HTTPS with credentials
        let url = "https://user:password@github.com/org/repo.git";
        let sanitized = sanitize_git_url(url);
        assert_eq!(sanitized, "https://github.com/org/repo.git");
        assert!(!sanitized.contains("user"));
        assert!(!sanitized.contains("password"));

        // Plain HTTPS (should be unchanged)
        let plain_url = "https://github.com/org/repo.git";
        assert_eq!(sanitize_git_url(plain_url), plain_url);

        // SSH URL (should be unchanged)
        let ssh_url = "git@github.com:org/repo.git";
        assert_eq!(sanitize_git_url(ssh_url), ssh_url);
    }

    // =========================================================================
    // Full Context Capture Test
    // =========================================================================

    #[test]
    fn test_full_context_capture() {
        let history = vec![HistoryEntry {
            prompt: "test".to_string(),
            command: "echo test".to_string(),
            timestamp: Utc::now(),
            success: true,
        }];

        let context = capture_context("list files", "ls -la", "static", None, &history)
            .expect("Should capture full context");

        // Verify all major fields are populated
        assert!(!context.cmdai_version.is_empty());
        assert!(!context.environment.os.is_empty());
        assert_eq!(context.command_info.user_prompt, "list files");
        assert_eq!(context.command_info.generated_command, "ls -la");
        assert_eq!(context.command_info.backend, "static");
        assert!(!context.system_state.available_backends.is_empty());

        // Error info should be None when no error provided
        assert!(context.error_info.is_none());
    }

    #[test]
    fn test_full_context_capture_with_error() {
        let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");

        let context = capture_context("delete files", "rm -rf /", "static", Some(&error), &[])
            .expect("Should capture context with error");

        assert!(context.error_info.is_some());
        let error_info = context.error_info.unwrap();
        assert!(error_info.error_message.contains("Access denied"));
    }
}
