//! Permission error detection and correction
//!
//! Detects when commands fail due to insufficient permissions and suggests
//! retrying with `sudo`.

use crate::execution::ExecutionResult;
use crate::Platform;

/// Detects permission errors from command execution results
#[derive(Debug, Clone, Copy)]
pub struct PermissionErrorDetector;

/// A suggested correction for a permission error
#[derive(Debug, Clone)]
pub struct SudoSuggestion {
    /// The original command that failed
    pub original_command: String,
    /// The corrected command with sudo prefix
    pub corrected_command: String,
    /// Explanation of what sudo does
    pub explanation: String,
}

impl PermissionErrorDetector {
    /// Detect if an execution result indicates a permission error
    ///
    /// Checks for:
    /// - Exit codes 1 or 126 (permission denied)
    /// - "Permission denied" in stderr
    /// - "Operation not permitted" in stderr
    /// - "Access denied" in stderr (Windows Git Bash)
    pub fn detect(result: &ExecutionResult) -> bool {
        // Check exit code
        let has_error_code = result.exit_code == 1 || result.exit_code == 126;

        if !has_error_code {
            return false;
        }

        // Check stderr for permission keywords (case-insensitive)
        let stderr_lower = result.stderr.to_lowercase();

        stderr_lower.contains("permission denied")
            || stderr_lower.contains("operation not permitted")
            || stderr_lower.contains("access denied")
            || stderr_lower.contains("eacces") // POSIX error code
    }

    /// Check if a command already has sudo
    fn has_sudo(command: &str) -> bool {
        let trimmed = command.trim_start();
        trimmed.starts_with("sudo ") || trimmed.starts_with("sudo\t")
    }

    /// Generate a sudo suggestion for the failed command
    ///
    /// Returns None if:
    /// - Command already has sudo
    /// - Platform doesn't support sudo (Windows without Git Bash)
    pub fn suggest_correction(
        command: &str,
        platform: Platform,
    ) -> Option<SudoSuggestion> {
        // Don't suggest sudo if command already has it
        if Self::has_sudo(command) {
            return None;
        }

        // Windows native (non-Git Bash) doesn't have sudo
        #[cfg(target_os = "windows")]
        {
            // For Windows, we'd need UAC elevation which is more complex
            // Defer to future work
            if !Self::is_git_bash() {
                return None;
            }
        }

        // Skip on Windows for now (UAC requires different approach)
        if matches!(platform, Platform::Windows) {
            return None;
        }

        Some(SudoSuggestion {
            original_command: command.to_string(),
            corrected_command: format!("sudo {}", command),
            explanation: "Run with elevated privileges (sudo grants root access)".to_string(),
        })
    }

    /// Detect if running in Git Bash on Windows
    #[cfg(target_os = "windows")]
    fn is_git_bash() -> bool {
        std::env::var("MSYSTEM")
            .map(|v| v.starts_with("MINGW"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(exit_code: i32, stderr: &str) -> ExecutionResult {
        ExecutionResult {
            exit_code,
            stdout: String::new(),
            stderr: stderr.to_string(),
            execution_time_ms: 0,
            success: exit_code == 0,
        }
    }

    #[test]
    fn test_detect_permission_denied() {
        let result = make_result(1, "touch: cannot touch 'file': Permission denied");
        assert!(PermissionErrorDetector::detect(&result));
    }

    #[test]
    fn test_detect_operation_not_permitted() {
        let result = make_result(1, "mkdir: cannot create directory: Operation not permitted");
        assert!(PermissionErrorDetector::detect(&result));
    }

    #[test]
    fn test_detect_access_denied() {
        let result = make_result(1, "Access denied");
        assert!(PermissionErrorDetector::detect(&result));
    }

    #[test]
    fn test_detect_eacces() {
        let result = make_result(1, "error: EACCES: permission denied, open '/var/log/test.log'");
        assert!(PermissionErrorDetector::detect(&result));
    }

    #[test]
    fn test_detect_case_insensitive() {
        let result = make_result(1, "PERMISSION DENIED");
        assert!(PermissionErrorDetector::detect(&result));
    }

    #[test]
    fn test_no_false_positive_on_success() {
        let result = make_result(0, "");
        assert!(!PermissionErrorDetector::detect(&result));
    }

    #[test]
    fn test_no_false_positive_on_other_errors() {
        let result = make_result(127, "command not found");
        assert!(!PermissionErrorDetector::detect(&result));
    }

    #[test]
    fn test_no_false_positive_wrong_exit_code() {
        let result = make_result(2, "Permission denied");
        assert!(!PermissionErrorDetector::detect(&result));
    }

    #[test]
    fn test_suggest_correction() {
        let suggestion =
            PermissionErrorDetector::suggest_correction("touch /etc/test", Platform::Linux)
                .unwrap();

        assert_eq!(suggestion.original_command, "touch /etc/test");
        assert_eq!(suggestion.corrected_command, "sudo touch /etc/test");
        assert!(suggestion.explanation.contains("sudo"));
    }

    #[test]
    fn test_no_suggestion_if_already_sudo() {
        let suggestion =
            PermissionErrorDetector::suggest_correction("sudo touch /etc/test", Platform::Linux);
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_no_suggestion_on_windows() {
        let suggestion =
            PermissionErrorDetector::suggest_correction("touch file", Platform::Windows);
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_has_sudo() {
        assert!(PermissionErrorDetector::has_sudo("sudo touch file"));
        assert!(PermissionErrorDetector::has_sudo("  sudo touch file"));
        assert!(PermissionErrorDetector::has_sudo("sudo\ttouch file"));
        assert!(!PermissionErrorDetector::has_sudo("sudoedit file"));
        assert!(!PermissionErrorDetector::has_sudo("touch sudo file"));
    }
}
