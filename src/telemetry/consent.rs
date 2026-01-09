//! First-run telemetry consent prompt

use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm};

/// Prompt user for telemetry consent on first run
///
/// Displays a clear explanation of:
/// - What data we collect
/// - What we don't collect
/// - How to opt-out later
/// - Link to privacy policy
///
/// Returns `true` if user consents, `false` otherwise.
pub fn prompt_consent() -> bool {
    eprintln!();
    eprintln!("{}", "━".repeat(70).bright_blue());
    eprintln!("{}", "📊  Telemetry & Privacy".bright_white().bold());
    eprintln!("{}", "━".repeat(70).bright_blue());
    eprintln!();

    eprintln!(
        "{}",
        "Caro is in beta and collects anonymous usage data to improve the product.".bright_white()
    );
    eprintln!();

    eprintln!("{}", "We collect:".bright_white().bold());
    eprintln!("  {} Session timing and performance metrics", "✓".green());
    eprintln!("  {} Platform info (OS, shell type)", "✓".green());
    eprintln!("  {} Error categories and safety events", "✓".green());
    eprintln!();

    eprintln!("{}", "We NEVER collect:".bright_white().bold());
    eprintln!("  {} Your commands or natural language input", "✗".red());
    eprintln!("  {} File paths or environment variables", "✗".red());
    eprintln!("  {} Any personally identifiable information", "✗".red());
    eprintln!();

    eprintln!("Learn more: {}", "https://caro.sh/telemetry".cyan());
    eprintln!(
        "{}",
        "You can disable telemetry anytime with:".bright_black()
    );
    eprintln!(
        "{}",
        "  caro config set telemetry.enabled false".bright_black()
    );
    eprintln!();

    eprintln!("{}", "━".repeat(70).bright_blue());
    eprintln!();

    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Enable telemetry to help improve Caro?")
        .default(true)
        .interact()
        .unwrap_or(false)
}

/// Show telemetry disabled confirmation
pub fn show_disabled_message() {
    eprintln!();
    eprintln!(
        "{}",
        "✓ Telemetry disabled. No data will be collected.".green()
    );
    eprintln!();
}

/// Show telemetry enabled confirmation
pub fn show_enabled_message() {
    eprintln!();
    eprintln!(
        "{}",
        "✓ Telemetry enabled. Thank you for helping improve Caro!".green()
    );
    eprintln!(
        "{}",
        "  View what's collected: caro telemetry show".bright_black()
    );
    eprintln!(
        "{}",
        "  Disable anytime: caro config set telemetry.enabled false".bright_black()
    );
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_dont_crash() {
        // Just ensure the message functions don't panic
        // (Can't actually test interactive prompt in unit tests)
        show_disabled_message();
        show_enabled_message();
    }
}
