/// State Management Module
///
/// This module handles all application state for the TUI.
/// It follows a Redux-inspired pattern with:
/// - Central `AppState` (single source of truth)
/// - `AppEvent` enum (all possible state changes)
/// - Pure state reducers (testable functions)
///
/// # Architecture
///
/// ```text
/// User Input → AppEvent → AppState::handle_event() → New State → Re-render
/// ```
///
/// # Example
///
/// ```rust
/// use cmdai::tui::state::{AppState, AppEvent};
///
/// let mut state = AppState::default();
///
/// // User types a character
/// let effects = state.handle_event(AppEvent::TextInput('l'));
///
/// assert_eq!(state.repl.input_buffer, "l");
/// ```

use anyhow::Result;
use crate::models::{ShellType, SafetyLevel};

pub mod app_state;
pub mod repl_state;
pub mod events;

pub use app_state::AppState;
pub use repl_state::ReplState;
pub use events::{AppEvent, SideEffect};

/// Current application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// REPL mode - interactive command generation
    Repl,

    /// History browser mode (future)
    #[allow(dead_code)]
    History,

    /// Configuration editor mode (future)
    #[allow(dead_code)]
    Config,

    /// Help screen (future)
    #[allow(dead_code)]
    Help,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Repl
    }
}

impl AppMode {
    /// Get human-readable name for this mode
    pub fn name(&self) -> &'static str {
        match self {
            AppMode::Repl => "REPL",
            AppMode::History => "History",
            AppMode::Config => "Config",
            AppMode::Help => "Help",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_mode_default() {
        assert_eq!(AppMode::default(), AppMode::Repl);
    }

    #[test]
    fn test_app_mode_names() {
        assert_eq!(AppMode::Repl.name(), "REPL");
        assert_eq!(AppMode::History.name(), "History");
        assert_eq!(AppMode::Config.name(), "Config");
        assert_eq!(AppMode::Help.name(), "Help");
    }
}
