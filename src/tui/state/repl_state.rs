/// REPL State
///
/// State specific to the REPL (Read-Eval-Print-Loop) mode.
/// This includes user input, generated commands, and validation results.
use super::events::{GeneratedCommandEvent, ValidationResultEvent};

/// State for the REPL mode
#[derive(Debug, Clone, Default)]
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

    /// Convert character position to byte index
    ///
    /// The cursor_position field tracks character count, but Rust's String methods
    /// (insert, remove) require byte indices. This helper converts from character
    /// position to the corresponding byte index in the UTF-8 encoded string.
    ///
    /// # Arguments
    /// * `char_pos` - Character position (0-indexed)
    ///
    /// # Returns
    /// Byte index corresponding to the character position
    fn char_pos_to_byte_index(&self, char_pos: usize) -> usize {
        self.input_buffer
            .chars()
            .take(char_pos)
            .map(|c| c.len_utf8())
            .sum()
    }

    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, c: char) {
        let byte_pos = self.char_pos_to_byte_index(self.cursor_position);
        self.input_buffer.insert(byte_pos, c);
        self.cursor_position += 1;
        self.clear_results();
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char_before(&mut self) {
        if self.cursor_position > 0 {
            let byte_pos = self.char_pos_to_byte_index(self.cursor_position - 1);
            self.input_buffer.remove(byte_pos);
            self.cursor_position -= 1;
            self.clear_results();
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_at(&mut self) {
        if self.cursor_position < self.input_buffer.chars().count() {
            let byte_pos = self.char_pos_to_byte_index(self.cursor_position);
            self.input_buffer.remove(byte_pos);
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

    // ====================================================================
    // Unicode Handling Tests
    // ====================================================================

    #[test]
    fn test_insert_emoji_characters() {
        let mut state = ReplState::new();

        // Insert multiple emoji (4-byte UTF-8 characters)
        state.insert_char('🚀');
        state.insert_char('🎉');
        state.insert_char('💻');

        assert_eq!(state.input_buffer, "🚀🎉💻");
        assert_eq!(state.cursor_position, 3); // 3 characters
        assert_eq!(state.input_buffer.len(), 12); // 12 bytes (3 * 4)
    }

    #[test]
    fn test_insert_chinese_characters() {
        let mut state = ReplState::new();

        // Insert Chinese characters (3-byte UTF-8)
        state.insert_char('你');
        state.insert_char('好');
        state.insert_char('世');
        state.insert_char('界');

        assert_eq!(state.input_buffer, "你好世界");
        assert_eq!(state.cursor_position, 4); // 4 characters
        assert_eq!(state.input_buffer.len(), 12); // 12 bytes (4 * 3)
    }

    #[test]
    fn test_mixed_ascii_and_unicode() {
        let mut state = ReplState::new();

        // Mix ASCII and unicode
        state.insert_char('h');
        state.insert_char('e');
        state.insert_char('l');
        state.insert_char('l');
        state.insert_char('o');
        state.insert_char(' ');
        state.insert_char('世');
        state.insert_char('界');

        assert_eq!(state.input_buffer, "hello 世界");
        assert_eq!(state.cursor_position, 8); // 8 characters
    }

    #[test]
    fn test_delete_emoji_at_end() {
        let mut state = ReplState::new();

        state.insert_char('🚀');
        state.insert_char('🎉');
        assert_eq!(state.input_buffer, "🚀🎉");
        assert_eq!(state.cursor_position, 2);

        state.delete_char_before();
        assert_eq!(state.input_buffer, "🚀");
        assert_eq!(state.cursor_position, 1);

        state.delete_char_before();
        assert_eq!(state.input_buffer, "");
        assert_eq!(state.cursor_position, 0);
    }

    #[test]
    fn test_delete_emoji_in_middle() {
        let mut state = ReplState::new();

        state.insert_char('🚀');
        state.insert_char('🎉');
        state.insert_char('💻');
        assert_eq!(state.input_buffer, "🚀🎉💻");

        // Move cursor to position 2 (between 🎉 and 💻)
        state.cursor_position = 2;

        // Delete 🎉
        state.delete_char_before();
        assert_eq!(state.input_buffer, "🚀💻");
        assert_eq!(state.cursor_position, 1);
    }

    #[test]
    fn test_insert_unicode_in_middle() {
        let mut state = ReplState::new();

        state.insert_char('a');
        state.insert_char('c');
        assert_eq!(state.input_buffer, "ac");

        // Move cursor to position 1 (between a and c)
        state.cursor_position = 1;

        // Insert emoji
        state.insert_char('🚀');
        assert_eq!(state.input_buffer, "a🚀c");
        assert_eq!(state.cursor_position, 2);
    }

    #[test]
    fn test_char_pos_to_byte_index_ascii() {
        let mut state = ReplState::new();
        state.input_buffer = "hello".to_string();

        // ASCII: byte index == char position
        assert_eq!(state.char_pos_to_byte_index(0), 0);
        assert_eq!(state.char_pos_to_byte_index(1), 1);
        assert_eq!(state.char_pos_to_byte_index(3), 3);
        assert_eq!(state.char_pos_to_byte_index(5), 5);
    }

    #[test]
    fn test_char_pos_to_byte_index_unicode() {
        let mut state = ReplState::new();
        state.input_buffer = "🚀🎉💻".to_string(); // 3 emoji, 4 bytes each

        // First emoji at byte 0
        assert_eq!(state.char_pos_to_byte_index(0), 0);
        // Second emoji at byte 4
        assert_eq!(state.char_pos_to_byte_index(1), 4);
        // Third emoji at byte 8
        assert_eq!(state.char_pos_to_byte_index(2), 8);
        // End at byte 12
        assert_eq!(state.char_pos_to_byte_index(3), 12);
    }

    #[test]
    fn test_char_pos_to_byte_index_mixed() {
        let mut state = ReplState::new();
        state.input_buffer = "a🚀b".to_string(); // 1 byte + 4 bytes + 1 byte = 6 bytes

        assert_eq!(state.char_pos_to_byte_index(0), 0); // 'a' at byte 0
        assert_eq!(state.char_pos_to_byte_index(1), 1); // '🚀' at byte 1
        assert_eq!(state.char_pos_to_byte_index(2), 5); // 'b' at byte 5
        assert_eq!(state.char_pos_to_byte_index(3), 6); // end at byte 6
    }

    #[test]
    fn test_delete_char_at_with_unicode() {
        let mut state = ReplState::new();

        state.insert_char('🚀');
        state.insert_char('🎉');
        state.insert_char('💻');
        assert_eq!(state.input_buffer, "🚀🎉💻");

        // Move cursor to start
        state.cursor_position = 0;

        // Delete first emoji
        state.delete_char_at();
        assert_eq!(state.input_buffer, "🎉💻");
        assert_eq!(state.cursor_position, 0);

        // Delete second emoji (now at position 0)
        state.delete_char_at();
        assert_eq!(state.input_buffer, "💻");
        assert_eq!(state.cursor_position, 0);
    }

    #[test]
    fn test_very_long_unicode_string() {
        let mut state = ReplState::new();

        // Insert 100 emoji
        for _ in 0..100 {
            state.insert_char('🚀');
        }

        assert_eq!(state.input_buffer.chars().count(), 100);
        assert_eq!(state.cursor_position, 100);
        assert_eq!(state.input_buffer.len(), 400); // 100 * 4 bytes

        // Delete half
        for _ in 0..50 {
            state.delete_char_before();
        }

        assert_eq!(state.input_buffer.chars().count(), 50);
        assert_eq!(state.cursor_position, 50);
        assert_eq!(state.input_buffer.len(), 200); // 50 * 4 bytes
    }
}
