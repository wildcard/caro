/// REPL State
///
/// State specific to the REPL (Read-Eval-Print-Loop) mode.
/// This includes user input, generated commands, and validation results.

use super::events::{GeneratedCommandEvent, ValidationResultEvent};

/// State for the REPL mode
#[derive(Debug, Clone)]
pub struct ReplState {
    // ===== Input State =====
    /// User's current input buffer
    pub input_buffer: String,

    /// Cursor position in the input buffer (0-based)
    pub cursor_position: usize,

    // ===== Generation State =====
    /// Whether we're currently generating a command
    pub generating: bool,

    /// The generated command (if any)
    pub generated_command: Option<GeneratedCommandEvent>,

    /// Error from generation (if any)
    pub generation_error: Option<String>,

    // ===== Validation State =====
    /// Whether we're currently validating
    pub validating: bool,

    /// Validation result (if any)
    pub validation_result: Option<ValidationResultEvent>,
}

impl Default for ReplState {
    fn default() -> Self {
        Self {
            input_buffer: String::new(),
            cursor_position: 0,
            generating: false,
            generated_command: None,
            generation_error: None,
            validating: false,
            validation_result: None,
        }
    }
}

impl ReplState {
    /// Create a new ReplState
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current input buffer
    pub fn input(&self) -> &str {
        &self.input_buffer
    }

    /// Check if there's any input
    pub fn has_input(&self) -> bool {
        !self.input_buffer.is_empty()
    }

    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.clear_results();
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char_before(&mut self) {
        if self.cursor_position > 0 {
            self.input_buffer.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.clear_results();
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_at(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
            self.clear_results();
        }
    }

    /// Clear the input buffer
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.clear_results();
    }

    /// Clear generated results
    fn clear_results(&mut self) {
        self.generated_command = None;
        self.generation_error = None;
        self.validation_result = None;
    }

    /// Set generating state
    pub fn set_generating(&mut self, generating: bool) {
        self.generating = generating;
        if generating {
            self.generation_error = None;
        }
    }

    /// Set generated command
    pub fn set_generated_command(&mut self, command: GeneratedCommandEvent) {
        self.generated_command = Some(command);
        self.generating = false;
        self.generation_error = None;
    }

    /// Set generation error
    pub fn set_generation_error(&mut self, error: String) {
        self.generation_error = Some(error);
        self.generating = false;
        self.generated_command = None;
    }

    /// Set validating state
    pub fn set_validating(&mut self, validating: bool) {
        self.validating = validating;
    }

    /// Set validation result
    pub fn set_validation_result(&mut self, result: ValidationResultEvent) {
        self.validation_result = Some(result);
        self.validating = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::events::RiskLevel;

    #[test]
    fn test_repl_state_default() {
        let state = ReplState::default();

        assert_eq!(state.input_buffer, "");
        assert_eq!(state.cursor_position, 0);
        assert!(!state.generating);
        assert!(state.generated_command.is_none());
    }

    #[test]
    fn test_insert_char() {
        let mut state = ReplState::new();

        state.insert_char('l');
        state.insert_char('s');

        assert_eq!(state.input_buffer, "ls");
        assert_eq!(state.cursor_position, 2);
    }

    #[test]
    fn test_delete_char_before() {
        let mut state = ReplState::new();
        state.input_buffer = "hello".to_string();
        state.cursor_position = 5;

        state.delete_char_before();

        assert_eq!(state.input_buffer, "hell");
        assert_eq!(state.cursor_position, 4);
    }

    #[test]
    fn test_delete_char_before_at_start() {
        let mut state = ReplState::new();
        state.input_buffer = "hello".to_string();
        state.cursor_position = 0;

        state.delete_char_before();

        assert_eq!(state.input_buffer, "hello");
        assert_eq!(state.cursor_position, 0);
    }

    #[test]
    fn test_delete_char_at() {
        let mut state = ReplState::new();
        state.input_buffer = "hello".to_string();
        state.cursor_position = 1;

        state.delete_char_at();

        assert_eq!(state.input_buffer, "hllo");
        assert_eq!(state.cursor_position, 1);
    }

    #[test]
    fn test_clear_input() {
        let mut state = ReplState::new();
        state.input_buffer = "hello".to_string();
        state.cursor_position = 5;

        state.clear_input();

        assert_eq!(state.input_buffer, "");
        assert_eq!(state.cursor_position, 0);
    }

    #[test]
    fn test_set_generated_command() {
        let mut state = ReplState::new();
        state.generating = true;

        state.set_generated_command(GeneratedCommandEvent {
            command: "ls -la".to_string(),
            explanation: "List all files".to_string(),
            risk_level: RiskLevel::Safe,
        });

        assert!(!state.generating);
        assert!(state.generated_command.is_some());
        assert_eq!(state.generated_command.as_ref().unwrap().command, "ls -la");
    }

    #[test]
    fn test_set_generation_error() {
        let mut state = ReplState::new();
        state.generating = true;

        state.set_generation_error("Backend unavailable".to_string());

        assert!(!state.generating);
        assert!(state.generation_error.is_some());
        assert!(state.generated_command.is_none());
    }

    #[test]
    fn test_has_input() {
        let mut state = ReplState::new();
        assert!(!state.has_input());

        state.insert_char('a');
        assert!(state.has_input());
    }
}
