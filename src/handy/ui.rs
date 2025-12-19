//! UI utilities for Handy.Computer integration
//!
//! This module provides user interface functions for displaying
//! Handy.Computer status and integration hints.

use colored::Colorize;
use crate::handy::{HandyStatus, HANDY_WEBSITE};

/// Show Handy welcome message based on current status
pub fn show_handy_welcome(status: &HandyStatus) {
    match status {
        HandyStatus::InstalledAndRunning { .. } => {
            println!("{}", "".dimmed());
            println!("{} {}", "🎤".bright_green(), "Handy.Computer detected and running".bright_green().bold());
            println!("{}", "   Press your Handy shortcut to speak commands".dimmed());
        }
        HandyStatus::InstalledNotRunning { .. } => {
            println!("{}", "".dimmed());
            println!("{} {}", "⚠️ ".yellow(), "Handy.Computer installed but not running".yellow());
            println!("{}", "   Start Handy to enable voice input".dimmed());
        }
        HandyStatus::NotInstalled => {
            println!("{}", "".dimmed());
            println!("{} {}", "💡".bright_blue(), "Enhanced voice input available with Handy.Computer".bright_blue());
            println!("{} {}", "   Download at:".dimmed(), HANDY_WEBSITE.cyan().underline());
        }
    }
}

/// Format a single-line status message for Handy
pub fn format_handy_status_line(status: &HandyStatus) -> String {
    match status {
        HandyStatus::InstalledAndRunning { .. } => {
            format!("{} {}", "🎤", "Voice input ready (Handy.Computer)".green())
        }
        HandyStatus::InstalledNotRunning { .. } => {
            format!("{} {}", "⚠️", "Handy installed but not running".yellow())
        }
        HandyStatus::NotInstalled => {
            format!("{} {}", "💡", format!("Install Handy for voice input: {}", HANDY_WEBSITE).dimmed())
        }
    }
}

/// Format a hint message for using Handy
pub fn format_handy_hint(status: &HandyStatus) -> Option<String> {
    match status {
        HandyStatus::InstalledAndRunning { .. } => {
            Some(format!("{} Press your Handy shortcut to use voice input", "Tip:".bright_blue().bold()))
        }
        HandyStatus::InstalledNotRunning { .. } => {
            Some(format!("{} Start Handy to enable voice commands", "Tip:".yellow().bold()))
        }
        HandyStatus::NotInstalled => None,
    }
}

/// Create a formatted box with Handy information
pub fn format_handy_info_box(status: &HandyStatus) -> Vec<String> {
    let mut lines = vec![];

    lines.push("┌─────────────────────────────────────────┐".dimmed().to_string());

    match status {
        HandyStatus::InstalledAndRunning { .. } => {
            lines.push(format!("│ {} {}│", "🎤".bright_green(), "Push-to-talk available via Handy    ".bright_green()));
            lines.push(format!("│ {}                              │", "Press your configured shortcut     ".dimmed()));
        }
        HandyStatus::InstalledNotRunning { .. } => {
            lines.push(format!("│ {} {}│", "⚠️ ", "Handy installed but not running    ".yellow()));
            lines.push(format!("│ {}                              │", "Start Handy to enable voice input  ".dimmed()));
        }
        HandyStatus::NotInstalled => {
            lines.push(format!("│ {} {}│", "💡", "Push-to-talk available via Handy    ".bright_blue()));
            lines.push("│                                         │".dimmed().to_string());
            lines.push(format!("│ {}                              │", "Not installed? Download at:        ".dimmed()));
            lines.push(format!("│ {}                              │", format!("{:<39}", HANDY_WEBSITE).cyan()));
        }
    }

    lines.push("└─────────────────────────────────────────┘".dimmed().to_string());

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_status_line() {
        let status = HandyStatus::NotInstalled;
        let line = format_handy_status_line(&status);
        assert!(line.contains("Install Handy"));

        let status = HandyStatus::InstalledNotRunning {
            install_path: PathBuf::from("/Applications/Handy.app"),
        };
        let line = format_handy_status_line(&status);
        assert!(line.contains("not running"));

        let status = HandyStatus::InstalledAndRunning {
            pid: 1234,
            install_path: PathBuf::from("/Applications/Handy.app"),
        };
        let line = format_handy_status_line(&status);
        assert!(line.contains("ready"));
    }

    #[test]
    fn test_format_hint() {
        let status = HandyStatus::NotInstalled;
        assert!(format_handy_hint(&status).is_none());

        let status = HandyStatus::InstalledNotRunning {
            install_path: PathBuf::from("/Applications/Handy.app"),
        };
        let hint = format_handy_hint(&status);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("Start Handy"));

        let status = HandyStatus::InstalledAndRunning {
            pid: 1234,
            install_path: PathBuf::from("/Applications/Handy.app"),
        };
        let hint = format_handy_hint(&status);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("Press your Handy shortcut"));
    }

    #[test]
    fn test_format_info_box() {
        let status = HandyStatus::NotInstalled;
        let box_lines = format_handy_info_box(&status);
        assert!(box_lines.len() > 2); // At least top, content, and bottom
        assert!(box_lines[0].contains("┌"));
        assert!(box_lines[box_lines.len() - 1].contains("└"));
    }
}
