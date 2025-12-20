//! Application state and logic for interactive mode

use std::path::PathBuf;
use chrono::{DateTime, Local};

/// Configuration for the interactive application
#[derive(Debug, Clone)]
pub struct InteractiveConfig {
    /// User name for display
    pub user_name: Option<String>,
    /// Working directory
    pub working_directory: PathBuf,
    /// Whether verbose mode is enabled
    pub verbose: bool,
}

impl Default for InteractiveConfig {
    fn default() -> Self {
        Self {
            user_name: whoami::fallible::realname().ok(),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            verbose: false,
        }
    }
}

/// Recent activity entry
#[derive(Debug, Clone)]
pub struct ActivityEntry {
    pub timestamp: DateTime<Local>,
    pub description: String,
    pub command: Option<String>,
}

/// Tips for getting started
pub const TIPS: &[&str] = &[
    "Type a natural language description to generate a shell command",
    "Use /help to see available commands",
    "Use /config to view your current configuration",
    "Press Ctrl+C to cancel the current operation",
    "Press Ctrl+D or type /exit to quit",
];

/// Main application state for interactive mode
pub struct InteractiveApp {
    /// Application configuration
    pub config: InteractiveConfig,
    /// Current input buffer
    pub input: String,
    /// Cursor position in input
    pub cursor_position: usize,
    /// Command history
    pub history: Vec<String>,
    /// History index for navigation
    pub history_index: Option<usize>,
    /// Recent activity
    pub recent_activity: Vec<ActivityEntry>,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Current tip index to display
    pub current_tip_index: usize,
    /// Scroll offset for output
    pub scroll_offset: u16,
    /// Output messages
    pub output_messages: Vec<OutputMessage>,
    /// Whether currently processing a command
    pub is_processing: bool,
}

/// Output message with styling
#[derive(Debug, Clone)]
pub struct OutputMessage {
    pub content: String,
    pub style: MessageStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStyle {
    Normal,
    Success,
    Warning,
    Error,
    Info,
    Command,
    Explanation,
}

impl InteractiveApp {
    /// Create a new interactive application
    pub fn new(config: InteractiveConfig) -> Self {
        Self {
            config,
            input: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_index: None,
            recent_activity: Vec::new(),
            should_quit: false,
            current_tip_index: 0,
            scroll_offset: 0,
            output_messages: Vec::new(),
            is_processing: false,
        }
    }

    /// Handle character input
    pub fn enter_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }

    /// Delete character at cursor
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input.len() {
            self.input.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    /// Navigate to previous history entry
    pub fn history_previous(&mut self) {
        if self.history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            Some(0) => Some(0),
            Some(i) => Some(i.saturating_sub(1)),
            None => Some(self.history.len() - 1),
        };

        if let Some(idx) = new_index {
            self.history_index = Some(idx);
            self.input = self.history[idx].clone();
            self.cursor_position = self.input.len();
        }
    }

    /// Navigate to next history entry
    pub fn history_next(&mut self) {
        if let Some(idx) = self.history_index {
            if idx + 1 < self.history.len() {
                self.history_index = Some(idx + 1);
                self.input = self.history[idx + 1].clone();
            } else {
                self.history_index = None;
                self.input.clear();
            }
            self.cursor_position = self.input.len();
        }
    }

    /// Submit the current input
    pub fn submit(&mut self) -> Option<String> {
        let input = self.input.trim().to_string();
        if input.is_empty() {
            return None;
        }

        // Add to history
        if self.history.last() != Some(&input) {
            self.history.push(input.clone());
        }
        self.history_index = None;

        // Clear input
        self.input.clear();
        self.cursor_position = 0;

        Some(input)
    }

    /// Add an output message
    pub fn add_output(&mut self, content: String, style: MessageStyle) {
        self.output_messages.push(OutputMessage { content, style });
    }

    /// Add activity entry
    pub fn add_activity(&mut self, description: String, command: Option<String>) {
        self.recent_activity.push(ActivityEntry {
            timestamp: Local::now(),
            description,
            command,
        });

        // Keep only last 10 entries
        if self.recent_activity.len() > 10 {
            self.recent_activity.remove(0);
        }
    }

    /// Cycle to next tip
    pub fn next_tip(&mut self) {
        self.current_tip_index = (self.current_tip_index + 1) % TIPS.len();
    }

    /// Get current tip
    pub fn current_tip(&self) -> &str {
        TIPS[self.current_tip_index]
    }

    /// Handle built-in slash commands
    pub fn handle_slash_command(&mut self, cmd: &str) -> bool {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts.first().map(|s| s.to_lowercase());

        match command.as_deref() {
            Some("/exit") | Some("/quit") | Some("/q") => {
                self.should_quit = true;
                true
            }
            Some("/help") | Some("/h") | Some("/?") => {
                self.add_output("Available commands:".to_string(), MessageStyle::Info);
                self.add_output("  /help, /h, /?  - Show this help message".to_string(), MessageStyle::Normal);
                self.add_output("  /exit, /quit, /q - Exit interactive mode".to_string(), MessageStyle::Normal);
                self.add_output("  /config - Show current configuration".to_string(), MessageStyle::Normal);
                self.add_output("  /clear - Clear the screen".to_string(), MessageStyle::Normal);
                self.add_output("  /history - Show command history".to_string(), MessageStyle::Normal);
                self.add_output("".to_string(), MessageStyle::Normal);
                self.add_output("Shortcuts:".to_string(), MessageStyle::Info);
                self.add_output("  Ctrl+C - Cancel current operation".to_string(), MessageStyle::Normal);
                self.add_output("  Ctrl+D - Exit interactive mode".to_string(), MessageStyle::Normal);
                self.add_output("  Up/Down - Navigate command history".to_string(), MessageStyle::Normal);
                self.add_output("  Ctrl+A - Move to start of line".to_string(), MessageStyle::Normal);
                self.add_output("  Ctrl+E - Move to end of line".to_string(), MessageStyle::Normal);
                true
            }
            Some("/clear") | Some("/cls") => {
                self.output_messages.clear();
                true
            }
            Some("/history") => {
                if self.history.is_empty() {
                    self.add_output("No command history yet.".to_string(), MessageStyle::Info);
                } else {
                    self.add_output("Command history:".to_string(), MessageStyle::Info);
                    // Collect messages first to avoid borrow conflict
                    let history_messages: Vec<_> = self.history.iter().enumerate()
                        .map(|(i, cmd)| format!("  {}. {}", i + 1, cmd))
                        .collect();
                    for msg in history_messages {
                        self.add_output(msg, MessageStyle::Normal);
                    }
                }
                true
            }
            Some("/config") => {
                self.add_output("Current configuration:".to_string(), MessageStyle::Info);
                self.add_output(format!("  Working directory: {}", self.config.working_directory.display()), MessageStyle::Normal);
                if let Some(ref name) = self.config.user_name {
                    self.add_output(format!("  User: {}", name), MessageStyle::Normal);
                }
                self.add_output(format!("  Verbose: {}", self.config.verbose), MessageStyle::Normal);
                true
            }
            _ => false,
        }
    }

    /// Scroll output up
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll output down
    pub fn scroll_down(&mut self, max_scroll: u16) {
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }
}
