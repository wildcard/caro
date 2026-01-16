//! Self-healing capability for automatic error recovery
//!
//! Detects common error patterns and suggests corrections to users.
//!
//! ## Supported Error Types
//!
//! - **Permission errors**: Suggests retrying with `sudo`
//! - More coming in future versions...
//!
//! ## Usage
//!
//! ```rust
//! use caro::healing::{HealingEngine, PermissionErrorDetector};
//! use caro::execution::ExecutionResult;
//! use caro::platform::Platform;
//!
//! // After command execution fails
//! let result = execute_command("touch /etc/test").await?;
//!
//! if !result.success {
//!     // Check if it's a permission error
//!     if PermissionErrorDetector::detect(&result) {
//!         if let Some(suggestion) = PermissionErrorDetector::suggest_correction(
//!             &result.command,
//!             Platform::current()
//!         ) {
//!             // Prompt user and retry with sudo if confirmed
//!             if confirm_sudo_retry(&suggestion.original_command, &suggestion.explanation)? {
//!                 let retry_result = execute_command(&suggestion.corrected_command).await?;
//!                 // Handle retry result...
//!             }
//!         }
//!     }
//! }
//! ```

pub mod permission;
pub mod prompt;

pub use permission::{PermissionErrorDetector, SudoSuggestion};
pub use prompt::confirm_sudo_retry;

use crate::execution::ExecutionResult;
use crate::Platform;

#[cfg(feature = "knowledge")]
use crate::knowledge::KnowledgeIndex;
#[cfg(feature = "knowledge")]
use std::sync::Arc;

/// Main self-healing engine
///
/// Analyzes execution failures and suggests corrections.
pub struct HealingEngine {
    platform: Platform,

    #[cfg(feature = "knowledge")]
    knowledge: Option<Arc<KnowledgeIndex>>,
}

impl HealingEngine {
    /// Create a new healing engine
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            #[cfg(feature = "knowledge")]
            knowledge: None,
        }
    }

    /// Enable knowledge index integration
    #[cfg(feature = "knowledge")]
    pub fn with_knowledge(mut self, knowledge: Arc<KnowledgeIndex>) -> Self {
        self.knowledge = Some(knowledge);
        self
    }

    /// Try to heal a failed command execution
    ///
    /// Returns Some(corrected_command) if a correction is suggested, None otherwise.
    pub fn try_heal(&self, result: &ExecutionResult, command: &str) -> Option<String> {
        // Try permission error healing
        if PermissionErrorDetector::detect(result) {
            if let Some(suggestion) =
                PermissionErrorDetector::suggest_correction(command, self.platform)
            {
                return Some(suggestion.corrected_command);
            }
        }

        // Future: Add more error type detectors here
        // - Command not found
        // - File not found
        // - Safety validation failures

        None
    }

    /// Record a successful healing correction to the knowledge index
    #[cfg(feature = "knowledge")]
    pub async fn record_correction(
        &self,
        original_prompt: &str,
        original_command: &str,
        corrected_command: &str,
        feedback: &str,
    ) {
        if let Some(ref knowledge) = self.knowledge {
            if let Err(e) = knowledge
                .record_correction(original_prompt, original_command, corrected_command, Some(feedback))
                .await
            {
                log::debug!("Failed to record healing correction: {}", e);
            } else {
                log::debug!("Recorded healing correction to knowledge index");
            }
        }
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
    fn test_healing_engine_permission_error() {
        let engine = HealingEngine::new(Platform::Linux);
        let result = make_result(1, "Permission denied");

        let corrected = engine.try_heal(&result, "touch /etc/test");
        assert_eq!(corrected, Some("sudo touch /etc/test".to_string()));
    }

    #[test]
    fn test_healing_engine_no_heal_on_success() {
        let engine = HealingEngine::new(Platform::Linux);
        let result = make_result(0, "");

        let corrected = engine.try_heal(&result, "touch test");
        assert_eq!(corrected, None);
    }

    #[test]
    fn test_healing_engine_no_heal_on_other_error() {
        let engine = HealingEngine::new(Platform::Linux);
        let result = make_result(127, "command not found");

        let corrected = engine.try_heal(&result, "nonexistent-command");
        assert_eq!(corrected, None);
    }
}
