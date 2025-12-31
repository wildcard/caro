//! Tip display formatting
//!
//! Format tips for terminal output with colors and styling.

use super::types::{Tip, TipCategory};
use colored::Colorize;

/// Display style for tips
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayStyle {
    /// Inline display (single line)
    #[default]
    Inline,
    /// Box display (multi-line with border)
    Box,
    /// Minimal display (just the message)
    Minimal,
}

/// Formatter for displaying tips
pub struct TipDisplay {
    style: DisplayStyle,
    show_savings: bool,
    show_source: bool,
}

impl Default for TipDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl TipDisplay {
    /// Create a new tip display formatter
    pub fn new() -> Self {
        Self {
            style: DisplayStyle::Inline,
            show_savings: true,
            show_source: false,
        }
    }

    /// Set the display style
    pub fn with_style(mut self, style: DisplayStyle) -> Self {
        self.style = style;
        self
    }

    /// Set whether to show savings
    pub fn with_savings(mut self, show: bool) -> Self {
        self.show_savings = show;
        self
    }

    /// Set whether to show source
    pub fn with_source(mut self, show: bool) -> Self {
        self.show_source = show;
        self
    }

    /// Format a tip for display
    pub fn format(&self, tip: &Tip) -> String {
        match self.style {
            DisplayStyle::Inline => self.format_inline(tip),
            DisplayStyle::Box => self.format_box(tip),
            DisplayStyle::Minimal => self.format_minimal(tip),
        }
    }

    /// Format tip as inline text
    fn format_inline(&self, tip: &Tip) -> String {
        let prefix = self.category_prefix(tip.category);
        let mut output = format!("{} {}", prefix, tip.message);

        if self.show_savings {
            if let Some(chars) = tip.chars_saved {
                if chars > 0 {
                    output.push_str(&format!(" {}", format!("(saves {} chars)", chars).dimmed()));
                }
            }
        }

        output
    }

    /// Format tip as a box
    fn format_box(&self, tip: &Tip) -> String {
        let prefix = self.category_prefix(tip.category);
        let width = 60;

        let mut lines = Vec::new();
        lines.push(format!("{}  {}", "Did you know?".bold().cyan(), prefix));
        lines.push(String::new());

        // Word wrap the message
        let wrapped = self.word_wrap(&tip.message, width - 4);
        for line in wrapped {
            lines.push(format!("  {}", line));
        }

        if self.show_savings {
            if let Some(chars) = tip.chars_saved {
                if chars > 0 {
                    lines.push(String::new());
                    lines.push(format!("  {}", format!("Saves {} keystrokes!", chars).green()));
                }
            }
        }

        if self.show_source {
            if let Some(ref source) = tip.source {
                lines.push(format!("  {}", format!("Source: {}", source).dimmed()));
            }
        }

        lines.join("\n")
    }

    /// Format tip as minimal text
    fn format_minimal(&self, tip: &Tip) -> String {
        tip.message.clone()
    }

    /// Get the colored prefix for a category
    fn category_prefix(&self, category: TipCategory) -> String {
        match category {
            TipCategory::AliasShortcut => "Tip:".bright_blue().bold().to_string(),
            TipCategory::PluginRecommendation => "Plugin:".bright_magenta().bold().to_string(),
            TipCategory::BestPractice => "Tip:".bright_green().bold().to_string(),
            TipCategory::SafetyTip => "Safety:".bright_yellow().bold().to_string(),
        }
    }

    /// Word wrap text to a given width
    fn word_wrap(&self, text: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }
}

/// Print a tip to stdout
pub fn print_tip(tip: &Tip) {
    let display = TipDisplay::new();
    println!("{}", display.format(tip));
}

/// Print a tip with box style
pub fn print_tip_box(tip: &Tip) {
    let display = TipDisplay::new().with_style(DisplayStyle::Box);
    println!("{}", display.format(tip));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tips::shell::Alias;

    #[test]
    fn test_inline_format() {
        let alias = Alias::new("gst", "git status");
        let tip = Tip::alias_suggestion(&alias, "git status");

        let display = TipDisplay::new();
        let output = display.format(&tip);

        assert!(output.contains("gst"));
        assert!(output.contains("saves"));
    }

    #[test]
    fn test_box_format() {
        let alias = Alias::new("gst", "git status");
        let tip = Tip::alias_suggestion(&alias, "git status");

        let display = TipDisplay::new().with_style(DisplayStyle::Box);
        let output = display.format(&tip);

        assert!(output.contains("Did you know?"));
        assert!(output.contains("gst"));
    }

    #[test]
    fn test_minimal_format() {
        let tip = Tip::new("test", TipCategory::BestPractice, "Test message");

        let display = TipDisplay::new().with_style(DisplayStyle::Minimal);
        let output = display.format(&tip);

        assert_eq!(output, "Test message");
    }

    #[test]
    fn test_word_wrap() {
        let display = TipDisplay::new();
        let text = "This is a long message that should be wrapped to multiple lines";
        let wrapped = display.word_wrap(text, 20);

        assert!(wrapped.len() > 1);
        for line in &wrapped {
            assert!(line.len() <= 20);
        }
    }
}
