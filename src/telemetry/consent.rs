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
    println!();
    println!("{}", "━".repeat(70).bright_blue());
    println!("{}", "📊  Telemetry & Privacy".bright_white().bold());
    println!("{}", "━".repeat(70).bright_blue());
    println!();

    println!(
        "{}",
        "Caro is in beta and collects anonymous usage data to improve the product.".bright_white()
    );
    println!();

    println!("{}", "We collect:".bright_white().bold());
    println!("  {} Session timing and performance metrics", "✓".green());
    println!("  {} Platform info (OS, shell type)", "✓".green());
    println!("  {} Error categories and safety events", "✓".green());
    println!();

    println!("{}", "We NEVER collect:".bright_white().bold());
    println!("  {} Your commands or natural language input", "✗".red());
    println!("  {} File paths or environment variables", "✗".red());
    println!("  {} Any personally identifiable information", "✗".red());
    println!();

    println!(
        "{}",
        format!("Learn more: {}", "https://caro.sh/telemetry".cyan())
    );
    println!(
        "{}",
        "You can disable telemetry anytime with:".bright_black()
    );
    println!(
        "{}",
        "  caro config set telemetry.enabled false".bright_black()
    );
    println!();

    println!("{}", "━".repeat(70).bright_blue());
    println!();

    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Enable telemetry to help improve Caro?")
        .default(true)
        .interact()
        .unwrap_or(false)
}

/// Show telemetry disabled confirmation
pub fn show_disabled_message() {
    println!();
    println!(
        "{}",
        "✓ Telemetry disabled. No data will be collected.".green()
    );
    println!();
}

/// Show telemetry enabled confirmation
pub fn show_enabled_message() {
    println!();
    println!(
        "{}",
        "✓ Telemetry enabled. Thank you for helping improve Caro!".green()
    );
    println!(
        "{}",
        "  View what's collected: caro telemetry show".bright_black()
    );
    println!(
        "{}",
        "  Disable anytime: caro config set telemetry.enabled false".bright_black()
    );
    println!();
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
