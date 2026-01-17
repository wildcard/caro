//! Terminal User Interface for Feedback Submission
//!
//! This module provides an interactive terminal interface for users to submit
//! feedback. It guides users through:
//!
//! - Viewing captured context
//! - Entering a description of the issue
//! - Optionally providing reproduction steps
//! - Reviewing and confirming submission

use crate::feedback::types::*;
use crate::feedback::FeedbackError;
use chrono::Utc;
use colored::Colorize;
use dialoguer::{Confirm, Editor, Input};
use std::io::IsTerminal;

// =============================================================================
// Public API
// =============================================================================

/// Run the interactive feedback submission interface
///
/// This function guides the user through the feedback submission process,
/// capturing their description and optionally reproduction steps.
///
/// # Arguments
/// * `context` - The captured feedback context
///
/// # Returns
/// Result containing Some(Feedback) if submitted, None if cancelled
pub fn run_feedback_interface(context: FeedbackContext) -> Result<Option<Feedback>, FeedbackError> {
    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() {
        return Err(FeedbackError::InputError(
            "Feedback submission requires an interactive terminal".to_string(),
        ));
    }

    // Display header
    print_header();

    // Show context preview
    print_context_preview(&context);

    // Get user description
    let description = match get_user_description() {
        Ok(Some(desc)) => desc,
        Ok(None) => {
            println!("{}", "Feedback cancelled.".yellow());
            return Ok(None);
        }
        Err(e) => return Err(e),
    };

    // Optional: Get reproduction steps
    let reproduction_steps = get_reproduction_steps()?;

    // Review option
    offer_context_review(&context)?;

    // Confirm submission
    if !confirm_submission()? {
        println!("{}", "Feedback cancelled.".yellow());
        return Ok(None);
    }

    // Create feedback
    let feedback = Feedback {
        id: FeedbackId::generate(),
        timestamp: Utc::now(),
        user_description: description,
        reproduction_steps,
        context,
        github_issue_url: None,
        status: FeedbackStatus::Submitted,
    };

    // Show success message
    println!();
    println!(
        "{} Feedback created with ID: {}",
        "✓".green(),
        feedback.id.to_string().cyan()
    );

    Ok(Some(feedback))
}

/// Run feedback submission in non-interactive mode (for CI/CD)
///
/// # Arguments
/// * `context` - The captured feedback context
/// * `description` - User's description of the issue
/// * `reproduction_steps` - Optional reproduction steps
///
/// # Returns
/// Result containing the created Feedback
pub fn create_feedback_non_interactive(
    context: FeedbackContext,
    description: String,
    reproduction_steps: Option<String>,
) -> Result<Feedback, FeedbackError> {
    // Validate description
    validate_description(&description)?;

    let feedback = Feedback {
        id: FeedbackId::generate(),
        timestamp: Utc::now(),
        user_description: description,
        reproduction_steps,
        context,
        github_issue_url: None,
        status: FeedbackStatus::Submitted,
    };

    Ok(feedback)
}

// =============================================================================
// TUI Components
// =============================================================================

/// Print the feedback interface header
fn print_header() {
    println!();
    println!("{}", "━".repeat(60).dimmed());
    println!("{}", "  Submit Feedback".bold().cyan());
    println!("{}", "━".repeat(60).dimmed());
    println!();
}

/// Print a preview of the captured context
fn print_context_preview(context: &FeedbackContext) {
    println!("{}", "Captured Context:".bold());
    println!();

    // Environment
    println!(
        "  {} {} {} ({})",
        "●".cyan(),
        "Environment:".dimmed(),
        context.environment.os,
        context.environment.arch
    );

    // Shell
    println!(
        "  {} {} {}",
        "●".cyan(),
        "Shell:".dimmed(),
        context.environment.shell
    );

    // Version
    println!(
        "  {} {} {}",
        "●".cyan(),
        "cmdai Version:".dimmed(),
        context.cmdai_version
    );

    // Backend
    println!(
        "  {} {} {}",
        "●".cyan(),
        "Backend:".dimmed(),
        context.command_info.backend
    );

    // Prompt
    println!(
        "  {} {} {}",
        "●".cyan(),
        "Prompt:".dimmed(),
        truncate_string(&context.command_info.user_prompt, 50)
    );

    // Generated command
    println!(
        "  {} {} {}",
        "●".cyan(),
        "Command:".dimmed(),
        truncate_string(&context.command_info.generated_command, 50)
    );

    // Error if present
    if let Some(ref error) = context.error_info {
        println!(
            "  {} {} {}",
            "●".red(),
            "Error:".dimmed(),
            truncate_string(&error.error_message, 50)
        );
    }

    // Git context if present
    if let Some(ref git) = context.git_context {
        println!(
            "  {} {} {} ({})",
            "●".cyan(),
            "Git:".dimmed(),
            git.current_branch,
            if git.has_uncommitted_changes {
                "modified"
            } else {
                "clean"
            }
        );
    }

    println!();
}

/// Get the user's description of the issue
fn get_user_description() -> Result<Option<String>, FeedbackError> {
    println!("{}", "What went wrong?".bold());
    println!(
        "{}",
        "Describe the issue you encountered (Ctrl+C to cancel):".dimmed()
    );
    println!();

    let description: String = Input::new()
        .with_prompt("Description")
        .validate_with(|input: &String| {
            if input.trim().is_empty() {
                Err("Description cannot be empty")
            } else if input.len() > 5000 {
                Err("Description too long (max 5000 characters)")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .map_err(|e| {
            if e.to_string().contains("interrupted") {
                FeedbackError::InputError("cancelled".to_string())
            } else {
                FeedbackError::InputError(format!("Failed to get input: {}", e))
            }
        })?;

    if description.trim().is_empty() {
        return Ok(None);
    }

    Ok(Some(description))
}

/// Get optional reproduction steps
fn get_reproduction_steps() -> Result<Option<String>, FeedbackError> {
    println!();
    let add_steps = Confirm::new()
        .with_prompt("Add reproduction steps? (optional)")
        .default(false)
        .interact()
        .map_err(|e| FeedbackError::InputError(format!("Failed to get confirmation: {}", e)))?;

    if !add_steps {
        return Ok(None);
    }

    println!();
    println!(
        "{}",
        "Opening editor for reproduction steps...".dimmed()
    );
    println!(
        "{}",
        "Tip: Start each step on a new line. Save and close to continue.".dimmed()
    );

    let template = "# Steps to reproduce the issue:\n1. \n2. \n3. \n";

    let steps = Editor::new()
        .edit(template)
        .map_err(|e| FeedbackError::InputError(format!("Failed to open editor: {}", e)))?;

    // Clean up the template if user didn't change much
    let steps = steps.and_then(|s| {
        let cleaned = s
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        if cleaned.trim().is_empty() || cleaned == "1. \n2. \n3. " {
            None
        } else {
            Some(cleaned)
        }
    });

    Ok(steps)
}

/// Offer to review full context
fn offer_context_review(context: &FeedbackContext) -> Result<(), FeedbackError> {
    println!();
    let review = Confirm::new()
        .with_prompt("Review full context before submitting?")
        .default(false)
        .interact()
        .map_err(|e| FeedbackError::InputError(format!("Failed to get confirmation: {}", e)))?;

    if review {
        println!();
        println!("{}", "Full Context (JSON):".bold());
        println!("{}", "─".repeat(60).dimmed());

        if let Ok(json) = serde_json::to_string_pretty(context) {
            // Print with some color highlighting
            for line in json.lines() {
                if line.contains(':') && !line.contains('{') && !line.contains('[') {
                    // Key-value line
                    if let Some(colon_idx) = line.find(':') {
                        let (key, value) = line.split_at(colon_idx);
                        println!("{}{}", key.cyan(), value);
                    } else {
                        println!("{}", line);
                    }
                } else {
                    println!("{}", line.dimmed());
                }
            }
        } else {
            println!("{}", "Error formatting context".red());
        }

        println!("{}", "─".repeat(60).dimmed());
    }

    Ok(())
}

/// Confirm submission
fn confirm_submission() -> Result<bool, FeedbackError> {
    println!();
    Confirm::new()
        .with_prompt("Submit feedback?")
        .default(true)
        .interact()
        .map_err(|e| FeedbackError::InputError(format!("Failed to get confirmation: {}", e)))
}

// =============================================================================
// Validation Functions
// =============================================================================

/// Validate user description
pub fn validate_description(desc: &str) -> Result<(), FeedbackError> {
    if desc.trim().is_empty() {
        return Err(FeedbackError::InputError(
            "Description cannot be empty".to_string(),
        ));
    }

    if desc.len() > 5000 {
        return Err(FeedbackError::InputError(
            "Description too long (max 5000 characters)".to_string(),
        ));
    }

    Ok(())
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Truncate a string to a maximum length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format the context preview for display
pub fn format_context_preview(context: &FeedbackContext) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "Environment: {} {} ({})\n",
        context.environment.os, context.environment.os_version, context.environment.arch
    ));

    output.push_str(&format!("Shell: {}\n", context.environment.shell));
    output.push_str(&format!("cmdai Version: {}\n", context.cmdai_version));
    output.push_str(&format!("Backend: {}\n", context.command_info.backend));
    output.push_str(&format!(
        "Prompt: {}\n",
        truncate_string(&context.command_info.user_prompt, 50)
    ));
    output.push_str(&format!(
        "Command: {}\n",
        truncate_string(&context.command_info.generated_command, 50)
    ));

    if let Some(ref error) = context.error_info {
        output.push_str(&format!(
            "Error: {}\n",
            truncate_string(&error.error_message, 50)
        ));
    }

    output
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // =========================================================================
    // Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_description_valid() {
        assert!(validate_description("Test description").is_ok());
        assert!(validate_description("A".repeat(5000).as_str()).is_ok());
    }

    #[test]
    fn test_validate_description_empty() {
        let result = validate_description("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FeedbackError::InputError(_)));
    }

    #[test]
    fn test_validate_description_whitespace_only() {
        let result = validate_description("   \t\n  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_description_too_long() {
        let long_desc = "x".repeat(5001);
        let result = validate_description(&long_desc);
        assert!(result.is_err());
    }

    // =========================================================================
    // String Truncation Tests
    // =========================================================================

    #[test]
    fn test_truncate_string_short() {
        let s = "short";
        assert_eq!(truncate_string(s, 10), "short");
    }

    #[test]
    fn test_truncate_string_exact() {
        let s = "1234567890";
        assert_eq!(truncate_string(s, 10), "1234567890");
    }

    #[test]
    fn test_truncate_string_long() {
        let s = "this is a very long string that should be truncated";
        let result = truncate_string(s, 20);
        assert_eq!(result.len(), 20);
        assert!(result.ends_with("..."));
    }

    // =========================================================================
    // Context Preview Tests
    // =========================================================================

    #[test]
    fn test_format_context_preview() {
        let context = create_test_context();
        let preview = format_context_preview(&context);

        assert!(preview.contains("Environment:"));
        assert!(preview.contains("macos"));
        assert!(preview.contains("Backend:"));
        assert!(preview.len() < 500); // Should be concise
    }

    #[test]
    fn test_format_context_preview_with_error() {
        let mut context = create_test_context();
        context.error_info = Some(ErrorInfo {
            exit_code: Some(1),
            stderr: "error output".to_string(),
            stdout: "".to_string(),
            error_message: "Command failed".to_string(),
            error_type: None,
        });

        let preview = format_context_preview(&context);
        assert!(preview.contains("Error:"));
    }

    // =========================================================================
    // Non-Interactive Creation Tests
    // =========================================================================

    #[test]
    fn test_create_feedback_non_interactive() {
        let context = create_test_context();
        let result = create_feedback_non_interactive(
            context,
            "Test description".to_string(),
            Some("1. Step 1\n2. Step 2".to_string()),
        );

        assert!(result.is_ok());
        let feedback = result.unwrap();
        assert_eq!(feedback.user_description, "Test description");
        assert!(feedback.reproduction_steps.is_some());
    }

    #[test]
    fn test_create_feedback_non_interactive_empty_description() {
        let context = create_test_context();
        let result = create_feedback_non_interactive(context, "".to_string(), None);

        assert!(result.is_err());
    }

    // =========================================================================
    // Helper Functions
    // =========================================================================

    fn create_test_context() -> FeedbackContext {
        FeedbackContext {
            timestamp: Utc::now(),
            cmdai_version: "1.0.0".to_string(),
            environment: EnvironmentInfo {
                os: "macos".to_string(),
                os_version: "14.0".to_string(),
                arch: "arm64".to_string(),
                shell: "zsh".to_string(),
                terminal: "Terminal.app".to_string(),
                rust_version: Some("1.75.0".to_string()),
            },
            command_info: CommandInfo {
                user_prompt: "list files".to_string(),
                generated_command: "ls -la".to_string(),
                backend: "static".to_string(),
                model: None,
                command_history: vec![],
            },
            error_info: None,
            system_state: SystemState {
                available_backends: vec!["static".to_string()],
                cache_dir: PathBuf::from("/tmp/cache"),
                config_file: None,
                is_ci: false,
                is_interactive: true,
            },
            git_context: None,
        }
    }
}
