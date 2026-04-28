//! Coloured risk badges for the command-decision moment.
//!
//! The badge mirrors the design system's CLI UI kit: a filled square glyph
//! plus the canonical level label, framed in brackets and tinted with the
//! status colour from [`crate::ui::palette`].
//!
//! ```text
//! [ ■ SAFE ]      green
//! [ ■ MODERATE ]  yellow
//! [ ■ HIGH RISK ] alarm-red
//! [ ■ CRITICAL ]  brand-red
//! ```
//!
//! `colored::Colorize::truecolor` produces a no-op result under `NO_COLOR`
//! or when stdout is not a TTY, so the bracketed label remains readable in
//! every environment.

use colored::Colorize;

use crate::models::RiskLevel;
use crate::ui::palette::{Rgb, STATUS_CRITICAL, STATUS_HIGH, STATUS_MODERATE, STATUS_SAFE};

/// Render a coloured, single-line risk badge for the given level.
pub fn render(level: RiskLevel) -> String {
    let (label, (r, g, b)): (&str, Rgb) = match level {
        RiskLevel::Safe => ("SAFE", STATUS_SAFE),
        RiskLevel::Moderate => ("MODERATE", STATUS_MODERATE),
        RiskLevel::High => ("HIGH RISK", STATUS_HIGH),
        RiskLevel::Critical => ("CRITICAL", STATUS_CRITICAL),
    };
    let body = format!("[ ■ {label} ]");
    body.truecolor(r, g, b).bold().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_ansi(s: &str) -> String {
        // Strip CSI SGR sequences (ESC[...m) without corrupting multi-byte
        // characters in the body — the brand glyph ■ is multi-byte UTF-8.
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' && chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                for inner in chars.by_ref() {
                    if inner == 'm' {
                        break;
                    }
                }
                continue;
            }
            out.push(c);
        }
        out
    }

    #[test]
    fn safe_badge_label() {
        let raw = render(RiskLevel::Safe);
        assert!(strip_ansi(&raw).contains("SAFE"));
    }

    #[test]
    fn high_badge_label() {
        let raw = render(RiskLevel::High);
        let plain = strip_ansi(&raw);
        assert!(plain.contains("HIGH RISK"));
        assert!(plain.starts_with("[ ■"));
        assert!(plain.ends_with("]"));
    }

    #[test]
    fn critical_badge_label() {
        let raw = render(RiskLevel::Critical);
        assert!(strip_ansi(&raw).contains("CRITICAL"));
    }

    #[test]
    fn moderate_badge_label() {
        let raw = render(RiskLevel::Moderate);
        assert!(strip_ansi(&raw).contains("MODERATE"));
    }

    #[test]
    fn distinct_colours_per_level() {
        // The colored crate emits the RGB triple inside an ANSI 38;2;R;G;B
        // sequence. Two different levels should produce two distinct strings.
        let safe = render(RiskLevel::Safe);
        let critical = render(RiskLevel::Critical);
        assert_ne!(safe, critical);
    }
}
