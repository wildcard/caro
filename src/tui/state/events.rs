/// Application Events
///
/// All state changes flow through the `AppEvent` enum. This makes the
/// application predictable and easy to test.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::models::SafetyLevel;

use super::AppMode;

/// All possible application events
///
/// Events are dispatched from user input, backend responses, and timers.
/// The main application handles these events and updates state accordingly.
#[derive(Debug, Clone)]
pub enum AppEvent {
    // ===== Input Events =====
    /// User pressed a key
    KeyPress(KeyEvent),

    /// User typed a character (filtered from KeyPress)
    TextInput(char),

    /// User pressed Backspace
    Backspace,

    /// User pressed Delete
    Delete,

    /// User pressed Enter
    Enter,

    // ===== Mode Changes =====
    /// Switch to a different application mode
    SwitchMode(AppMode),

    // ===== Command Generation =====
    /// Request to generate a command from current input
    GenerateCommand,

    /// Command generation completed
    CommandGenerated(GeneratedCommandEvent),

    /// Command generation failed
    GenerationFailed(String),

    // ===== Validation =====
    /// Request to validate the current command
    ValidateCommand,

    /// Validation completed
    ValidationComplete(ValidationResultEvent),

    // ===== Control =====
    /// Clear the input buffer
    ClearInput,

    /// Request to quit the application
    Quit,

    /// Terminal was resized
    Resize(u16, u16),
}

/// Generated command event data
#[derive(Debug, Clone)]
pub struct GeneratedCommandEvent {
    /// The generated shell command
    pub command: String,

    /// Explanation of what the command does
    pub explanation: String,

    /// Risk level assessment
    pub risk_level: RiskLevel,
}

/// Validation result event data
#[derive(Debug, Clone)]
pub struct ValidationResultEvent {
    /// Risk level of the command
    pub risk_level: RiskLevel,

    /// List of warnings
    pub warnings: Vec<String>,

    /// List of suggestions
    pub suggestions: Vec<String>,

    /// Matched dangerous patterns
    pub matched_patterns: Vec<String>,
}

/// Risk level for a command
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Command is safe to execute
    Safe,

    /// Command has moderate risk
    Moderate,

    /// Command has high risk
    High,

    /// Command is critically dangerous
    Critical,
}

impl RiskLevel {
    /// Get color for this risk level
    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            RiskLevel::Safe => Color::Green,
            RiskLevel::Moderate => Color::Yellow,
            RiskLevel::High => Color::Red,
            RiskLevel::Critical => Color::Red,
        }
    }

    /// Get icon for this risk level
    pub fn icon(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "✓",
            RiskLevel::Moderate => "⚠",
            RiskLevel::High => "❌",
            RiskLevel::Critical => "🛑",
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "SAFE",
            RiskLevel::Moderate => "MODERATE",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        }
    }
}

impl From<crate::models::RiskLevel> for RiskLevel {
    fn from(level: crate::models::RiskLevel) -> Self {
        match level {
            crate::models::RiskLevel::Safe => RiskLevel::Safe,
            crate::models::RiskLevel::Moderate => RiskLevel::Moderate,
            crate::models::RiskLevel::High => RiskLevel::High,
            crate::models::RiskLevel::Critical => RiskLevel::Critical,
        }
    }
}

/// Side effects that need to be performed outside of state updates
///
/// These represent async operations or I/O that can't be done
/// in the pure state update functions.
#[derive(Debug, Clone)]
pub enum SideEffect {
    /// Trigger command generation (async backend call)
    GenerateCommand {
        input: String,
        shell: crate::models::ShellType,
        safety: SafetyLevel,
    },

    /// Trigger command validation
    ValidateCommand {
        command: String,
        shell: crate::models::ShellType,
    },

    /// Execute a shell command
    #[allow(dead_code)]
    ExecuteCommand(String),

    /// Show error message
    #[allow(dead_code)]
    ShowError(String),
}

/// Helper to convert KeyEvent to AppEvent
impl From<KeyEvent> for AppEvent {
    fn from(key: KeyEvent) -> Self {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => AppEvent::Quit,
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                AppEvent::ClearInput
            }
            KeyCode::Char(c) => AppEvent::TextInput(c),
            KeyCode::Backspace => AppEvent::Backspace,
            KeyCode::Delete => AppEvent::Delete,
            KeyCode::Enter => AppEvent::Enter,
            _ => AppEvent::KeyPress(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Safe < RiskLevel::Moderate);
        assert!(RiskLevel::Moderate < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_risk_level_icons() {
        assert_eq!(RiskLevel::Safe.icon(), "✓");
        assert_eq!(RiskLevel::Moderate.icon(), "⚠");
        assert_eq!(RiskLevel::High.icon(), "❌");
        assert_eq!(RiskLevel::Critical.icon(), "🛑");
    }

    #[test]
    fn test_key_event_conversion_ctrl_c() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let event: AppEvent = key.into();

        match event {
            AppEvent::Quit => {}
            _ => panic!("Expected Quit event"),
        }
    }

    #[test]
    fn test_key_event_conversion_char() {
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let event: AppEvent = key.into();

        match event {
            AppEvent::TextInput('a') => {}
            _ => panic!("Expected TextInput('a')"),
        }
    }

    #[test]
    fn test_key_event_conversion_enter() {
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let event: AppEvent = key.into();

        match event {
            AppEvent::Enter => {}
            _ => panic!("Expected Enter event"),
        }
    }
}
