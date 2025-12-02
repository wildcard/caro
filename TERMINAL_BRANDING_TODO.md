# cmdai Terminal Branding Implementation Roadmap

> Actionable plan for implementing branded terminal output across cmdai

## Overview

This document provides a phased approach to implementing the terminal branding specified in `/home/user/cmdai/docs/TERMINAL_OUTPUT_SPEC.md`. The implementation is broken into manageable tasks with clear priorities and effort estimates.

---

## Quick Start

### Prerequisites

1. Review `/home/user/cmdai/docs/TERMINAL_OUTPUT_SPEC.md`
2. Review brand assets in `/home/user/cmdai/brand-assets/`
3. Ensure `colored` crate is in Cargo.toml (currently in use at main.rs:184)

### Current State Analysis

**Files Currently Using Terminal Output:**
- `/home/user/cmdai/src/main.rs` - Uses `colored` crate for basic coloring
- `/home/user/cmdai/src/cli/mod.rs` - Defines output structures
- `/home/user/cmdai/src/safety/mod.rs` - Generates validation messages

**Current Terminal Output:**
```
Warning: <message>  (yellow bold)
Blocked: <message>  (red bold)
Command: <command>  (bright cyan bold)
✓ Confirmed. Proceeding...  (green)
```

**What's Missing:**
- Box-drawing characters
- Structured output templates
- Safety level indicators (bars, colors)
- ASCII logo rendering
- Consistent formatting across all output types
- Helper module for terminal output

---

## Phase 1: Foundation (High Priority)

**Goal**: Create reusable terminal output infrastructure

### Task 1.1: Create Terminal Output Module

**File**: `/home/user/cmdai/src/ui/mod.rs` (new)

**Effort**: 2-3 hours

**Dependencies**: None

**Implementation**:
```rust
// src/ui/mod.rs
//! Terminal user interface module
//!
//! Provides branded output formatting for cmdai

pub mod colors;
pub mod boxes;
pub mod indicators;
pub mod templates;

pub use colors::*;
pub use boxes::*;
pub use indicators::*;
pub use templates::*;
```

**Files to Create**:
1. `src/ui/mod.rs` - Module entry point
2. `src/ui/colors.rs` - Color constants and helpers
3. `src/ui/boxes.rs` - Box drawing utilities
4. `src/ui/indicators.rs` - Safety level indicators
5. `src/ui/templates.rs` - Output templates

**Testing**:
- Unit tests for each submodule
- Manual testing in terminal

---

### Task 1.2: Implement Color Constants

**File**: `/home/user/cmdai/src/ui/colors.rs` (new)

**Effort**: 1 hour

**Dependencies**: `colored` crate (already present)

**Implementation**:
```rust
use colored::*;

// Primary brand colors
pub const TERMINAL_GREEN: Color = Color::BrightGreen;    // #00FF41
pub const CYBER_CYAN: Color = Color::BrightCyan;         // #00D9FF
pub const WARNING_AMBER: Color = Color::BrightYellow;    // #FFB800
pub const ALERT_ORANGE: Color = Color::TrueColor { r: 255, g: 107, b: 0 }; // #FF6B00
pub const CRITICAL_RED: Color = Color::BrightRed;        // #FF0055

// Helper functions
pub fn safe_text(s: &str) -> ColoredString {
    s.color(TERMINAL_GREEN).bold()
}

pub fn command_text(s: &str) -> ColoredString {
    s.color(CYBER_CYAN).bold()
}

pub fn moderate_text(s: &str) -> ColoredString {
    s.color(WARNING_AMBER).bold()
}

pub fn high_text(s: &str) -> ColoredString {
    s.color(ALERT_ORANGE).bold()
}

pub fn critical_text(s: &str) -> ColoredString {
    s.color(CRITICAL_RED).bold()
}

pub fn dim_text(s: &str) -> ColoredString {
    s.dimmed()
}

// Risk level colorizer
pub fn risk_level_text(level: &crate::models::RiskLevel) -> ColoredString {
    use crate::models::RiskLevel;
    match level {
        RiskLevel::Safe => safe_text("✓ SAFE"),
        RiskLevel::Moderate => moderate_text("⚠ MODERATE"),
        RiskLevel::High => high_text("⚠ HIGH"),
        RiskLevel::Critical => critical_text("✗ CRITICAL"),
    }
}
```

**Testing**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_text() {
        let output = safe_text("SAFE");
        assert!(output.to_string().contains("SAFE"));
    }
}
```

---

### Task 1.3: Implement Box Drawing Utilities

**File**: `/home/user/cmdai/src/ui/boxes.rs` (new)

**Effort**: 2-3 hours

**Dependencies**: `textwrap` crate (add to Cargo.toml)

**Implementation**:
```rust
use crate::models::RiskLevel;

pub struct BoxStyle {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub t_left: char,
    pub t_right: char,
}

impl BoxStyle {
    /// Single-line box (for safe/normal output)
    pub const SINGLE: BoxStyle = BoxStyle {
        top_left: '┌',
        top_right: '┐',
        bottom_left: '└',
        bottom_right: '┘',
        horizontal: '─',
        vertical: '│',
        t_left: '├',
        t_right: '┤',
    };

    /// Double-line box (for critical/blocked output)
    pub const DOUBLE: BoxStyle = BoxStyle {
        top_left: '╔',
        top_right: '╗',
        bottom_left: '╚',
        bottom_right: '╝',
        horizontal: '═',
        vertical: '║',
        t_left: '╠',
        t_right: '╣',
    };

    /// Select style based on risk level
    pub fn for_risk_level(level: RiskLevel) -> &'static BoxStyle {
        match level {
            RiskLevel::Critical => &Self::DOUBLE,
            _ => &Self::SINGLE,
        }
    }
}

pub struct BoxBuilder {
    style: &'static BoxStyle,
    width: usize,
    title: Option<String>,
    sections: Vec<BoxSection>,
}

pub struct BoxSection {
    pub title: Option<String>,
    pub lines: Vec<String>,
}

impl BoxBuilder {
    pub fn new(width: usize) -> Self {
        Self {
            style: &BoxStyle::SINGLE,
            width,
            title: Some("cmdai".to_string()),
            sections: Vec::new(),
        }
    }

    pub fn with_style(mut self, style: &'static BoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_risk_level(mut self, level: RiskLevel) -> Self {
        self.style = BoxStyle::for_risk_level(level);
        self
    }

    pub fn add_section(mut self, section: BoxSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn build(self) -> String {
        let mut output = String::new();

        // Top border with title
        if let Some(title) = &self.title {
            let title_str = format!(" {} ", title);
            let remaining = self.width.saturating_sub(title_str.len() + 2);
            output.push(self.style.top_left);
            output.push(self.style.horizontal);
            output.push_str(&title_str);
            output.push_str(&self.style.horizontal.to_string().repeat(remaining));
            output.push(self.style.top_right);
            output.push('\n');
        } else {
            output.push(self.style.top_left);
            output.push_str(&self.style.horizontal.to_string().repeat(self.width - 2));
            output.push(self.style.top_right);
            output.push('\n');
        }

        // Sections
        for (i, section) in self.sections.iter().enumerate() {
            // Section divider
            if i > 0 {
                if let Some(section_title) = &section.title {
                    let title_str = format!(" {} ", section_title);
                    let remaining = self.width.saturating_sub(title_str.len() + 2);
                    output.push(self.style.t_left);
                    output.push(self.style.horizontal);
                    output.push_str(&title_str);
                    output.push_str(&self.style.horizontal.to_string().repeat(remaining));
                    output.push(self.style.t_right);
                    output.push('\n');
                }
            }

            // Section lines
            for line in &section.lines {
                output.push(self.style.vertical);
                output.push(' ');
                output.push_str(line);
                let padding = self.width.saturating_sub(line.len() + 3);
                output.push_str(&" ".repeat(padding));
                output.push(' ');
                output.push(self.style.vertical);
                output.push('\n');
            }
        }

        // Bottom border
        output.push(self.style.bottom_left);
        output.push_str(&self.style.horizontal.to_string().repeat(self.width - 2));
        output.push(self.style.bottom_right);
        output.push('\n');

        output
    }
}

// Helper function for quick boxes
pub fn simple_box(title: &str, content: &[String], width: usize) -> String {
    BoxBuilder::new(width)
        .add_section(BoxSection {
            title: None,
            lines: content.to_vec(),
        })
        .build()
}
```

**Testing**:
```rust
#[test]
fn test_simple_box() {
    let output = simple_box("cmdai", &["Test line".to_string()], 30);
    assert!(output.contains("┌"));
    assert!(output.contains("└"));
    assert!(output.contains("Test line"));
}
```

---

### Task 1.4: Implement Safety Indicators

**File**: `/home/user/cmdai/src/ui/indicators.rs` (new)

**Effort**: 1-2 hours

**Dependencies**: Colors module

**Implementation**:
```rust
use crate::models::RiskLevel;
use crate::ui::colors::*;
use colored::Colorize;

/// Generate a risk level progress bar
pub fn risk_bar(level: RiskLevel) -> String {
    let (filled, empty, percentage, symbol) = match level {
        RiskLevel::Safe => (10, 0, 100, "✓"),
        RiskLevel::Moderate => (6, 4, 60, "⚠"),
        RiskLevel::High => (4, 6, 40, "⚠"),
        RiskLevel::Critical => (1, 9, 10, "✗"),
    };

    let filled_bar = "▓".repeat(filled);
    let empty_bar = "░".repeat(empty);
    let bar = format!("{}{}", filled_bar, empty_bar);

    let colored_bar = match level {
        RiskLevel::Safe => bar.green(),
        RiskLevel::Moderate => bar.yellow(),
        RiskLevel::High => bar.color(ALERT_ORANGE),
        RiskLevel::Critical => bar.red(),
    };

    let label = risk_level_text(&level);

    format!("{} {:3}%  {} {}", colored_bar, percentage, label, symbol)
}

/// Generate safety checklist
pub fn safety_checklist(checks: &[(bool, &str)]) -> Vec<String> {
    checks
        .iter()
        .map(|(passed, description)| {
            if *passed {
                format!("{} {}", "✓".green(), description)
            } else {
                format!("{} {}", "✗".red(), description)
            }
        })
        .collect()
}

/// Generate risk level label with color
pub fn risk_label(level: RiskLevel) -> String {
    format!("[{}]", risk_level_text(&level))
}
```

**Testing**:
```rust
#[test]
fn test_risk_bar() {
    let bar = risk_bar(RiskLevel::Safe);
    assert!(bar.contains("100%"));
    assert!(bar.contains("▓▓▓▓▓▓▓▓▓▓"));
}
```

---

### Task 1.5: Update Cargo.toml

**File**: `/home/user/cmdai/Cargo.toml`

**Effort**: 15 minutes

**Add Dependencies**:
```toml
[dependencies]
# ... existing dependencies ...
colored = "2.1"      # Already present
textwrap = "0.16"    # Add for text wrapping in boxes
```

---

## Phase 2: Integration (Medium Priority)

**Goal**: Replace current output with branded templates

### Task 2.1: Create Output Templates

**File**: `/home/user/cmdai/src/ui/templates.rs` (new)

**Effort**: 3-4 hours

**Dependencies**: All Phase 1 tasks

**Implementation**:
```rust
use crate::cli::CliResult;
use crate::models::RiskLevel;
use crate::safety::ValidationResult;
use crate::ui::{boxes::*, colors::*, indicators::*};
use colored::Colorize;

pub struct CommandOutputTemplate;

impl CommandOutputTemplate {
    /// Render safe command output
    pub fn render_safe(
        prompt: &str,
        command: &str,
        explanation: &str,
        validation: &ValidationResult,
    ) -> String {
        let width = 60;
        let mut sections = Vec::new();

        // Request section
        sections.push(BoxSection {
            title: None,
            lines: vec![
                "".to_string(),
                format!("▸ Your request:"),
                format!("  \"{}\"", prompt),
                "".to_string(),
            ],
        });

        // Generated command section
        sections.push(BoxSection {
            title: Some("Generated Command".to_string()),
            lines: vec![
                "".to_string(),
                format!("  {}", command_text(command)),
                "".to_string(),
            ],
        });

        // Safety analysis section
        let mut safety_lines = vec!["".to_string()];
        for warning in &validation.warnings {
            safety_lines.push(format!("  ✓ {}", warning));
        }
        safety_lines.push("".to_string());
        safety_lines.push(format!("  Risk Level:  {}", risk_bar(validation.risk_level)));
        safety_lines.push("".to_string());

        sections.push(BoxSection {
            title: Some("Safety Analysis".to_string()),
            lines: safety_lines,
        });

        // Execute prompt
        sections.push(BoxSection {
            title: Some("Execute".to_string()),
            lines: vec![
                format!("  ⚡ Run this command? [Y/n]"),
                "".to_string(),
            ],
        });

        let mut builder = BoxBuilder::new(width).with_risk_level(validation.risk_level);
        for section in sections {
            builder = builder.add_section(section);
        }

        builder.build()
    }

    /// Render blocked command output
    pub fn render_blocked(
        prompt: &str,
        command: &str,
        validation: &ValidationResult,
        suggestions: &[String],
    ) -> String {
        let width = 60;
        let mut sections = Vec::new();

        // Request section
        sections.push(BoxSection {
            title: None,
            lines: vec![
                "".to_string(),
                format!("▸ Your request:"),
                format!("  \"{}\"", prompt),
                "".to_string(),
            ],
        });

        // Generated command section
        sections.push(BoxSection {
            title: Some("Generated Command".to_string()),
            lines: vec![
                "".to_string(),
                format!("  {}", command.red().bold()),
                "".to_string(),
            ],
        });

        // Safety analysis section
        let mut safety_lines = vec!["".to_string()];
        for warning in &validation.warnings {
            safety_lines.push(format!("  ✗ {}", warning.red()));
        }
        safety_lines.push("".to_string());
        safety_lines.push(format!("  Risk Level:  {}", risk_bar(validation.risk_level)));
        safety_lines.push("".to_string());

        sections.push(BoxSection {
            title: Some("Safety Analysis".to_string()),
            lines: safety_lines,
        });

        // Action blocked section
        let mut blocked_lines = vec![
            "".to_string(),
            format!("  🛡️  cmdai has BLOCKED this command for your safety."),
            "".to_string(),
            format!("  This operation is dangerous and could cause damage."),
            "".to_string(),
        ];

        if !suggestions.is_empty() {
            blocked_lines.push(format!("  💡 Perhaps you meant to:"));
            for suggestion in suggestions {
                blocked_lines.push(format!("    • {}", suggestion));
            }
            blocked_lines.push("".to_string());
        }

        sections.push(BoxSection {
            title: Some("ACTION BLOCKED".to_string()),
            lines: blocked_lines,
        });

        let mut builder = BoxBuilder::new(width).with_risk_level(RiskLevel::Critical);
        for section in sections {
            builder = builder.add_section(section);
        }

        builder.build()
    }

    /// Render error message
    pub fn render_error(error_type: &str, message: &str, suggestions: &[String]) -> String {
        let width = 60;
        let mut lines = vec![
            format!("  ✗ Error: {}", error_type),
            "".to_string(),
            format!("  {}", message),
            "".to_string(),
        ];

        if !suggestions.is_empty() {
            lines.push(format!("  💡 Try:"));
            for suggestion in suggestions {
                lines.push(format!("    • {}", suggestion));
            }
            lines.push("".to_string());
        }

        lines.push(format!("  Need help? Run: {}", command_text("cmdai --help")));
        lines.push("".to_string());

        BoxBuilder::new(width)
            .add_section(BoxSection {
                title: None,
                lines,
            })
            .build()
    }
}
```

---

### Task 2.2: Update main.rs print_plain_output()

**File**: `/home/user/cmdai/src/main.rs`

**Effort**: 2 hours

**Location**: Lines 183-259

**Current Code**:
```rust
async fn print_plain_output(result: &cmdai::cli::CliResult, cli: &Cli) -> Result<(), CliError> {
    use colored::Colorize;

    // Print warnings first
    for warning in &result.warnings {
        eprintln!("{} {}", "Warning:".yellow().bold(), warning);
    }
    // ... rest of current implementation
}
```

**Replace With**:
```rust
async fn print_plain_output(result: &cmdai::cli::CliResult, cli: &Cli) -> Result<(), CliError> {
    use cmdai::ui::templates::CommandOutputTemplate;

    // Handle blocked commands
    if let Some(blocked_reason) = &result.blocked_reason {
        let output = CommandOutputTemplate::render_blocked(
            &result.detected_context,
            &result.generated_command,
            &get_validation_from_result(result),
            &result.alternatives,
        );
        println!("{}", output);
        return Ok(());
    }

    // Handle safe/moderate commands
    let output = CommandOutputTemplate::render_safe(
        &result.detected_context,
        &result.generated_command,
        &result.explanation,
        &get_validation_from_result(result),
    );
    println!("{}", output);

    // Handle confirmation if needed
    if result.requires_confirmation && !cli.confirm {
        use dialoguer::Confirm;

        if atty::is(atty::Stream::Stdin) {
            let confirmed = Confirm::new()
                .with_prompt("Proceed?")
                .default(false)
                .interact()
                .map_err(|e| CliError::Internal {
                    message: format!("Failed to get user confirmation: {}", e),
                })?;

            if !confirmed {
                println!("{}", "⚠ Operation cancelled by user".yellow());
                return Ok(());
            }
        }
    }

    // Print performance info if verbose
    if cli.verbose {
        println!("\n{}", result.generation_details.dimmed());
    }

    Ok(())
}

// Helper to reconstruct ValidationResult from CliResult
fn get_validation_from_result(result: &cmdai::cli::CliResult) -> cmdai::safety::ValidationResult {
    use cmdai::models::RiskLevel;
    use cmdai::safety::ValidationResult;

    let risk_level = if result.blocked_reason.is_some() {
        RiskLevel::Critical
    } else if result.requires_confirmation {
        RiskLevel::Moderate
    } else {
        RiskLevel::Safe
    };

    ValidationResult {
        allowed: result.blocked_reason.is_none(),
        risk_level,
        explanation: result.explanation.clone(),
        warnings: result.warnings.clone(),
        matched_patterns: Vec::new(),
        confidence_score: 0.95,
    }
}
```

---

### Task 2.3: Add Version Banner

**File**: `/home/user/cmdai/src/main.rs`

**Effort**: 1 hour

**Add Function**:
```rust
fn print_version_banner() {
    println!(r#"
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║         ██████╗███╗   ███╗██████╗      █████╗ ██╗               ║
║        ██╔════╝████╗ ████║██╔══██╗    ██╔══██╗██║               ║
║        ██║     ██╔████╔██║██║  ██║    ███████║██║               ║
║        ██║     ██║╚██╔╝██║██║  ██║    ██╔══██║██║               ║
║        ╚██████╗██║ ╚═╝ ██║██████╔╝    ██║  ██║██║               ║
║         ╚═════╝╚═╝     ╚═╝╚═════╝     ╚═╝  ╚═╝╚═╝               ║
║                                                                   ║
║                  ⚡ AI-Powered · Human-Safe 🛡️                    ║
║                                                                   ║
║                      Version {}                           ║
║                  Built with Rust • AGPL-3.0                       ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
"#, env!("CARGO_PKG_VERSION"));
}
```

**Update**: Call this in `main()` when `--version` flag is detected (Clap handles this, but we can add custom version output).

---

## Phase 3: Polish (Low Priority)

**Goal**: Add advanced features and refinements

### Task 3.1: Add Progress Indicators

**File**: `/home/user/cmdai/src/ui/progress.rs` (new)

**Effort**: 2 hours

**Dependencies**: `indicatif` crate

**Implementation**:
```rust
use indicatif::{ProgressBar, ProgressStyle};

pub fn validation_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Validating command safety...");
    pb
}

pub fn generation_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Generating command...");
    pb
}
```

---

### Task 3.2: Add Configuration Detection

**File**: `/home/user/cmdai/src/ui/terminal.rs` (new)

**Effort**: 1-2 hours

**Implementation**:
```rust
use std::env;

pub struct TerminalCapabilities {
    pub supports_color: bool,
    pub supports_unicode: bool,
    pub width: usize,
}

impl TerminalCapabilities {
    pub fn detect() -> Self {
        let supports_color = !env::var("NO_COLOR").is_ok()
            && atty::is(atty::Stream::Stdout);

        let supports_unicode = env::var("LANG")
            .map(|lang| lang.contains("UTF"))
            .unwrap_or(true);

        let width = term_size::dimensions()
            .map(|(w, _)| w)
            .unwrap_or(80);

        Self {
            supports_color,
            supports_unicode,
            width,
        }
    }

    pub fn box_width(&self) -> usize {
        self.width.min(80).max(60)
    }
}
```

---

### Task 3.3: Add Tests

**File**: `/home/user/cmdai/tests/terminal_output_tests.rs` (new)

**Effort**: 2-3 hours

**Implementation**:
```rust
#[cfg(test)]
mod terminal_output_tests {
    use cmdai::ui::*;
    use cmdai::models::RiskLevel;

    #[test]
    fn test_safe_command_box() {
        let output = boxes::simple_box("cmdai", &["Test".to_string()], 30);
        assert!(output.contains("┌"));
        assert!(output.contains("└"));
    }

    #[test]
    fn test_risk_indicators() {
        let bar = indicators::risk_bar(RiskLevel::Safe);
        assert!(bar.contains("100%"));
    }

    #[test]
    fn test_critical_box_uses_double_lines() {
        let style = boxes::BoxStyle::for_risk_level(RiskLevel::Critical);
        assert_eq!(style.top_left, '╔');
    }
}
```

---

## Priority Order

### Week 1 (Must Have)
1. ✅ Task 1.1: Create terminal output module structure
2. ✅ Task 1.2: Implement color constants
3. ✅ Task 1.3: Implement box drawing utilities
4. ✅ Task 1.4: Implement safety indicators
5. ✅ Task 1.5: Update Cargo.toml

### Week 2 (Should Have)
6. Task 2.1: Create output templates
7. Task 2.2: Update main.rs print_plain_output()
8. Task 2.3: Add version banner

### Week 3 (Nice to Have)
9. Task 3.1: Add progress indicators
10. Task 3.2: Add terminal capability detection
11. Task 3.3: Add comprehensive tests

---

## Effort Estimates

| Phase | Tasks | Total Effort | Priority |
|-------|-------|--------------|----------|
| Phase 1: Foundation | 5 | 6-8 hours | HIGH |
| Phase 2: Integration | 3 | 6-7 hours | MEDIUM |
| Phase 3: Polish | 3 | 5-7 hours | LOW |
| **TOTAL** | **11** | **17-22 hours** | - |

---

## Dependencies to Add

```toml
[dependencies]
# Core (already present)
colored = "2.1"
dialoguer = "0.11"
atty = "0.2"

# New additions
textwrap = "0.16"       # For text wrapping in boxes
indicatif = "0.17"      # For progress indicators (Phase 3)
term_size = "0.3"       # For terminal dimension detection (Phase 3)
```

---

## Testing Strategy

### Manual Testing

```bash
# Test safe command
cargo run -- "list all files"

# Test moderate risk
cargo run -- "delete temporary files"

# Test blocked command
cargo run -- "wipe the entire system"

# Test error handling
cargo run -- "xyzabc123"

# Test version banner
cargo run -- --version

# Test without colors
NO_COLOR=1 cargo run -- "list files"

# Test in narrow terminal
stty cols 60 && cargo run -- "list files"
```

### Unit Tests

```bash
# Run all tests
cargo test

# Run terminal UI tests
cargo test --test terminal_output_tests

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Run E2E tests
cargo test --test e2e_tests

# Test in different terminal emulators
# - iTerm2 (macOS)
# - Terminal.app (macOS)
# - GNOME Terminal (Linux)
# - Windows Terminal
```

---

## Code Examples

### Example 1: Using the Box Builder

```rust
use cmdai::ui::boxes::*;

let output = BoxBuilder::new(60)
    .with_risk_level(RiskLevel::Safe)
    .add_section(BoxSection {
        title: None,
        lines: vec![
            "Command: ls -la".to_string(),
            "Status: Safe".to_string(),
        ],
    })
    .build();

println!("{}", output);
```

### Example 2: Using Templates

```rust
use cmdai::ui::templates::CommandOutputTemplate;

let output = CommandOutputTemplate::render_safe(
    "list all files",
    "ls -la",
    "Lists all files in current directory",
    &validation_result,
);

println!("{}", output);
```

### Example 3: Using Color Helpers

```rust
use cmdai::ui::colors::*;

println!("{}", safe_text("✓ SAFE"));
println!("{}", moderate_text("⚠ MODERATE"));
println!("{}", critical_text("✗ CRITICAL"));
```

---

## Migration Path

### Backward Compatibility

**Option 1: Feature Flag** (Recommended)
```toml
[features]
default = ["fancy-ui"]
fancy-ui = ["colored", "textwrap"]
plain-ui = []
```

**Option 2: Environment Variable**
```bash
CMDAI_PLAIN_OUTPUT=1 cmdai "list files"
```

**Option 3: CLI Flag**
```bash
cmdai --plain "list files"
```

### Gradual Rollout

1. **Phase 1**: Implement new UI behind feature flag
2. **Phase 2**: Make fancy UI default, plain UI optional
3. **Phase 3**: Deprecate plain UI after user feedback

---

## Success Criteria

### Definition of Done

- [ ] All Phase 1 tasks completed
- [ ] All Phase 2 tasks completed
- [ ] Unit tests passing (>80% coverage)
- [ ] Manual testing in 3+ terminal emulators
- [ ] Documentation updated
- [ ] Examples added to README
- [ ] No regression in performance (<5ms overhead)

### User Acceptance

- [ ] Output is visually appealing
- [ ] Risk levels are immediately clear
- [ ] Works in light and dark terminals
- [ ] Accessible to color-blind users
- [ ] Terminal width responsive (60-200 chars)

---

## Troubleshooting

### Common Issues

**Issue**: Box characters not rendering
```
Solution: Check terminal supports UTF-8
$ echo $LANG  # Should contain UTF-8
```

**Issue**: Colors not showing
```
Solution: Check NO_COLOR environment variable
$ unset NO_COLOR
```

**Issue**: Text wrapping issues
```
Solution: Adjust box width based on terminal
let width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
```

---

## Future Enhancements (Post-MVP)

1. **Themes** - Light/dark/cyberpunk themes
2. **Animation** - Smooth transitions and effects
3. **Rich TUI** - Interactive command editing with `ratatui`
4. **Localization** - Multi-language support
5. **Custom Templates** - User-defined output formats

---

## References

- Terminal Output Spec: `/home/user/cmdai/docs/TERMINAL_OUTPUT_SPEC.md`
- Brand Assets: `/home/user/cmdai/brand-assets/`
- Current Output: `/home/user/cmdai/src/main.rs` (lines 183-259)
- Safety Module: `/home/user/cmdai/src/safety/mod.rs`

---

## Questions?

For questions about this implementation plan:
1. Review the Terminal Output Spec
2. Check brand assets for examples
3. Open a GitHub issue with `[terminal-ui]` tag

---

**Version**: 1.0.0
**Last Updated**: 2025-11-19
**Status**: Ready for Implementation

⚡🛡️ Think Fast. Stay Safe.
