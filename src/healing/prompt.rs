//! User confirmation prompts for self-healing actions

use colored::Colorize;
use dialoguer::Confirm;

/// Prompt the user to confirm retrying a command with sudo
///
/// Shows:
/// - Warning icon and message
/// - Original command
/// - Corrected command with sudo
/// - Explanation
/// - Confirmation prompt [Y/n]
///
/// Returns true if user confirms, false if declined
pub fn confirm_sudo_retry(
    original_command: &str,
    explanation: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Display warning and commands
    eprintln!();
    eprintln!(
        "{} {}",
        "⚠️ ".yellow(),
        "Permission error detected.".yellow()
    );
    eprintln!();
    eprintln!("  {}: {}", "Original".dimmed(), original_command.dimmed());
    eprintln!(
        "  {}: {}",
        "Suggested".cyan(),
        format!("sudo {}", original_command).cyan()
    );
    eprintln!();
    eprintln!("  {}: {}", "Note".dimmed(), explanation.dimmed());
    eprintln!();

    // Prompt for confirmation (default Yes)
    Ok(Confirm::new()
        .with_prompt("Would you like to retry with sudo?")
        .default(true)
        .interact()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests can't be fully automated since they require user input
    // They're here to document the expected behavior

    #[test]
    fn test_confirm_sudo_retry_format() {
        // This test just verifies the function signature and that it compiles
        // Actual behavior requires manual testing with real terminal input
        let _result = confirm_sudo_retry("touch /etc/test", "Run with elevated privileges");
        // Can't assert on result since it would block waiting for input
    }
}
