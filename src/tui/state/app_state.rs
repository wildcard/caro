/// Application State
///
/// Central state management for the TUI application.
/// This is the single source of truth for all UI state.

use anyhow::Result;

use crate::config::UserConfiguration;
use crate::models::{ShellType, SafetyLevel};

use super::{AppEvent, AppMode, ReplState, SideEffect};
use super::events::{GeneratedCommandEvent, ValidationResultEvent};

/// Central application state - single source of truth
///
/// All UI state lives here. Components read from this state
/// and emit events to modify it.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current application mode
    pub current_mode: AppMode,

    /// REPL mode state
    pub repl: ReplState,

    /// User configuration
    pub config: UserConfiguration,

    /// Backend status
    pub backend_status: BackendStatus,

    /// Whether to show help modal
    pub show_help_modal: bool,

    /// Current error message (if any)
    pub error_message: Option<String>,

    /// Whether the application should quit
    pub should_quit: bool,
}

/// Backend availability status
#[derive(Debug, Clone)]
pub struct BackendStatus {
    /// Backend name (e.g., "Ollama", "vLLM")
    pub name: String,

    /// Whether the backend is available
    pub available: bool,

    /// Current model (if known)
    pub model: Option<String>,
}

impl Default for BackendStatus {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            available: false,
            model: None,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_mode: AppMode::default(),
            repl: ReplState::default(),
            config: UserConfiguration::default(),
            backend_status: BackendStatus::default(),
            show_help_modal: false,
            error_message: None,
            should_quit: false,
        }
    }
}

impl AppState {
    /// Create a new AppState with the given configuration
    pub fn new(config: UserConfiguration) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Handle an application event and return side effects
    ///
    /// This is the main state update function. It takes an event,
    /// updates the state, and returns a list of side effects that
    /// need to be performed (async operations, I/O, etc.).
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut state = AppState::default();
    /// let effects = state.handle_event(AppEvent::TextInput('l'));
    /// // effects now contains any side effects to perform
    /// ```
    pub fn handle_event(&mut self, event: AppEvent) -> Result<Vec<SideEffect>> {
        let effects = match event {
            // ===== Text Input Events =====
            AppEvent::TextInput(c) => {
                self.repl.insert_char(c);
                vec![]
            }

            AppEvent::Backspace => {
                self.repl.delete_char_before();
                vec![]
            }

            AppEvent::Delete => {
                self.repl.delete_char_at();
                vec![]
            }

            AppEvent::ClearInput => {
                self.repl.clear_input();
                vec![]
            }

            // ===== Command Generation =====
            AppEvent::Enter => {
                if self.repl.has_input() {
                    self.repl.set_generating(true);
                    let shell = self.shell();
                    vec![SideEffect::GenerateCommand {
                        input: self.repl.input().to_string(),
                        shell,
                        safety: self.config.safety_level,
                    }]
                } else {
                    vec![]
                }
            }

            AppEvent::GenerateCommand => {
                if self.repl.has_input() {
                    self.repl.set_generating(true);
                    let shell = self.shell();
                    vec![SideEffect::GenerateCommand {
                        input: self.repl.input().to_string(),
                        shell,
                        safety: self.config.safety_level,
                    }]
                } else {
                    vec![]
                }
            }

            AppEvent::CommandGenerated(cmd) => {
                self.repl.set_generated_command(cmd.clone());

                // Trigger validation
                let shell = self.shell();
                vec![SideEffect::ValidateCommand {
                    command: cmd.command,
                    shell,
                }]
            }

            AppEvent::GenerationFailed(error) => {
                self.repl.set_generation_error(error);
                vec![]
            }

            // ===== Validation =====
            AppEvent::ValidateCommand => {
                // Clone command before setting validating state to avoid borrow conflicts
                let command_opt = self.repl.generated_command.as_ref().map(|cmd| cmd.command.clone());

                if let Some(command) = command_opt {
                    self.repl.set_validating(true);
                    let shell = self.shell();
                    vec![SideEffect::ValidateCommand {
                        command,
                        shell,
                    }]
                } else {
                    vec![]
                }
            }

            AppEvent::ValidationComplete(result) => {
                self.repl.set_validation_result(result);
                vec![]
            }

            // ===== Mode Changes =====
            AppEvent::SwitchMode(mode) => {
                self.current_mode = mode;
                vec![]
            }

            // ===== Control =====
            AppEvent::Quit => {
                self.should_quit = true;
                vec![]
            }

            AppEvent::Resize(_, _) => {
                // Terminal resized - no state change needed
                vec![]
            }

            // ===== Other =====
            AppEvent::KeyPress(_) => {
                // Unhandled key press
                vec![]
            }
        };

        Ok(effects)
    }

    /// Get the current shell type (defaults to Bash if not configured)
    pub fn shell(&self) -> ShellType {
        self.config.default_shell.clone().unwrap_or(ShellType::Bash)
    }

    /// Get the current safety level
    pub fn safety_level(&self) -> SafetyLevel {
        self.config.safety_level
    }

    /// Set backend status
    pub fn set_backend_status(&mut self, name: String, available: bool, model: Option<String>) {
        self.backend_status = BackendStatus {
            name,
            available,
            model,
        };
    }

    /// Show an error message
    pub fn show_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::events::RiskLevel;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();

        assert_eq!(state.current_mode, AppMode::Repl);
        assert!(!state.should_quit);
        assert!(state.error_message.is_none());
    }

    #[test]
    fn test_handle_text_input() {
        let mut state = AppState::default();

        let effects = state.handle_event(AppEvent::TextInput('l')).unwrap();

        assert_eq!(state.repl.input(), "l");
        assert_eq!(effects.len(), 0);
    }

    #[test]
    fn test_handle_backspace() {
        let mut state = AppState::default();
        state.repl.insert_char('l');
        state.repl.insert_char('s');

        let effects = state.handle_event(AppEvent::Backspace).unwrap();

        assert_eq!(state.repl.input(), "l");
        assert_eq!(effects.len(), 0);
    }

    #[test]
    fn test_handle_enter_with_input() {
        let mut state = AppState::default();
        state.repl.insert_char('l');
        state.repl.insert_char('s');

        let effects = state.handle_event(AppEvent::Enter).unwrap();

        assert!(state.repl.generating);
        assert_eq!(effects.len(), 1);

        match &effects[0] {
            SideEffect::GenerateCommand { input, .. } => {
                assert_eq!(input, "ls");
            }
            _ => panic!("Expected GenerateCommand side effect"),
        }
    }

    #[test]
    fn test_handle_enter_without_input() {
        let mut state = AppState::default();

        let effects = state.handle_event(AppEvent::Enter).unwrap();

        assert!(!state.repl.generating);
        assert_eq!(effects.len(), 0);
    }

    #[test]
    fn test_handle_command_generated() {
        let mut state = AppState::default();

        let event = AppEvent::CommandGenerated(GeneratedCommandEvent {
            command: "ls -la".to_string(),
            explanation: "List all files".to_string(),
            risk_level: RiskLevel::Safe,
        });

        let effects = state.handle_event(event).unwrap();

        assert!(!state.repl.generating);
        assert!(state.repl.generated_command.is_some());
        assert_eq!(effects.len(), 1);

        match &effects[0] {
            SideEffect::ValidateCommand { command, .. } => {
                assert_eq!(command, "ls -la");
            }
            _ => panic!("Expected ValidateCommand side effect"),
        }
    }

    #[test]
    fn test_handle_validation_complete() {
        let mut state = AppState::default();
        state.repl.set_validating(true);

        let event = AppEvent::ValidationComplete(ValidationResultEvent {
            risk_level: RiskLevel::Safe,
            warnings: vec![],
            suggestions: vec![],
            matched_patterns: vec![],
        });

        let effects = state.handle_event(event).unwrap();

        assert!(!state.repl.validating);
        assert!(state.repl.validation_result.is_some());
        assert_eq!(effects.len(), 0);
    }

    #[test]
    fn test_handle_quit() {
        let mut state = AppState::default();

        let effects = state.handle_event(AppEvent::Quit).unwrap();

        assert!(state.should_quit);
        assert_eq!(effects.len(), 0);
    }

    #[test]
    fn test_handle_clear_input() {
        let mut state = AppState::default();
        state.repl.insert_char('h');
        state.repl.insert_char('i');

        let effects = state.handle_event(AppEvent::ClearInput).unwrap();

        assert_eq!(state.repl.input(), "");
        assert_eq!(effects.len(), 0);
    }

    #[test]
    fn test_set_backend_status() {
        let mut state = AppState::default();

        state.set_backend_status(
            "Ollama".to_string(),
            true,
            Some("qwen2.5-coder:7b".to_string()),
        );

        assert_eq!(state.backend_status.name, "Ollama");
        assert!(state.backend_status.available);
        assert_eq!(
            state.backend_status.model.as_ref().unwrap(),
            "qwen2.5-coder:7b"
        );
    }
}
