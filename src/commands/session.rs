//! Session state management for cmdai interactive mode
//!
//! Manages the application state, user session data, and mode transitions
//! between normal command generation and slash command execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current mode of the cmdai session
#[derive(Debug, Clone, PartialEq)]
pub enum SessionMode {
    /// Normal command generation mode
    Normal,
    /// Interactive testing mode
    Testing,
    /// Configuration mode
    Configuration,
    /// Help/documentation mode
    Help,
    /// Exiting the application
    Exiting,
}

/// Session statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub start_time: DateTime<Utc>,
    pub commands_generated: usize,
    pub slash_commands_executed: usize,
    pub total_interactions: usize,
    pub errors_encountered: usize,
    pub last_activity: DateTime<Utc>,
}

impl Default for SessionStats {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            start_time: now,
            commands_generated: 0,
            slash_commands_executed: 0,
            total_interactions: 0,
            errors_encountered: 0,
            last_activity: now,
        }
    }
}

/// Current state of the user session
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Current session mode
    pub mode: SessionMode,
    /// Session statistics
    pub stats: SessionStats,
    /// Command history (last N commands)
    pub command_history: Vec<String>,
    /// Generated commands history
    pub generated_history: Vec<String>,
    /// Session-specific settings
    pub session_config: HashMap<String, String>,
    /// Whether the session should continue running
    pub should_continue: bool,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            mode: SessionMode::Normal,
            stats: SessionStats::default(),
            command_history: Vec::new(),
            generated_history: Vec::new(),
            session_config: HashMap::new(),
            should_continue: true,
        }
    }
}

/// Session manager for handling state transitions and persistence
pub struct SessionManager {
    state: SessionState,
    max_history_size: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            state: SessionState::default(),
            max_history_size: 50, // Keep last 50 commands
        }
    }

    /// Get current session state
    pub fn get_state(&self) -> &SessionState {
        &self.state
    }

    /// Get mutable session state
    pub fn get_state_mut(&mut self) -> &mut SessionState {
        &mut self.state
    }

    /// Change session mode
    pub fn set_mode(&mut self, mode: SessionMode) {
        self.state.mode = mode;
        self.update_activity();
    }

    /// Record a user command
    pub fn record_command(&mut self, command: &str) {
        self.state.command_history.push(command.to_string());
        self.state.stats.total_interactions += 1;
        
        // Keep history size manageable
        if self.state.command_history.len() > self.max_history_size {
            self.state.command_history.remove(0);
        }
        
        self.update_activity();
    }

    /// Record a generated command
    pub fn record_generated_command(&mut self, command: &str) {
        self.state.generated_history.push(command.to_string());
        self.state.stats.commands_generated += 1;
        
        // Keep history size manageable
        if self.state.generated_history.len() > self.max_history_size {
            self.state.generated_history.remove(0);
        }
        
        self.update_activity();
    }

    /// Record a slash command execution
    pub fn record_slash_command(&mut self, command: &str) {
        self.state.stats.slash_commands_executed += 1;
        self.record_command(command);
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.state.stats.errors_encountered += 1;
        self.update_activity();
    }

    /// Get command history
    pub fn get_command_history(&self, count: Option<usize>) -> Vec<&String> {
        let limit = count.unwrap_or(self.state.command_history.len());
        self.state.command_history
            .iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get generated command history
    pub fn get_generated_history(&self, count: Option<usize>) -> Vec<&String> {
        let limit = count.unwrap_or(self.state.generated_history.len());
        self.state.generated_history
            .iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Set session configuration value
    pub fn set_config(&mut self, key: &str, value: &str) {
        self.state.session_config.insert(key.to_string(), value.to_string());
        self.update_activity();
    }

    /// Get session configuration value
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.state.session_config.get(key)
    }

    /// Request session termination
    pub fn request_exit(&mut self) {
        self.state.should_continue = false;
        self.state.mode = SessionMode::Exiting;
    }

    /// Check if session should continue
    pub fn should_continue(&self) -> bool {
        self.state.should_continue
    }

    /// Get session duration
    pub fn get_session_duration(&self) -> chrono::Duration {
        Utc::now() - self.state.stats.start_time
    }

    /// Get formatted session summary
    pub fn get_session_summary(&self) -> String {
        let duration = self.get_session_duration();
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        let seconds = duration.num_seconds() % 60;

        format!(
            "Session Summary:
  Duration: {}h {}m {}s
  Commands Generated: {}
  Slash Commands: {}
  Total Interactions: {}
  Errors: {}
  Success Rate: {:.1}%",
            hours,
            minutes,
            seconds,
            self.state.stats.commands_generated,
            self.state.stats.slash_commands_executed,
            self.state.stats.total_interactions,
            self.state.stats.errors_encountered,
            if self.state.stats.total_interactions > 0 {
                ((self.state.stats.total_interactions - self.state.stats.errors_encountered) as f64
                    / self.state.stats.total_interactions as f64) * 100.0
            } else {
                100.0
            }
        )
    }

    /// Reset session statistics
    pub fn reset_stats(&mut self) {
        self.state.stats = SessionStats::default();
        self.state.command_history.clear();
        self.state.generated_history.clear();
    }

    /// Export session data
    pub fn export_session(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct SessionExport {
            stats: SessionStats,
            command_history: Vec<String>,
            generated_history: Vec<String>,
            session_config: HashMap<String, String>,
            duration_seconds: i64,
        }

        let export = SessionExport {
            stats: self.state.stats.clone(),
            command_history: self.state.command_history.clone(),
            generated_history: self.state.generated_history.clone(),
            session_config: self.state.session_config.clone(),
            duration_seconds: self.get_session_duration().num_seconds(),
        };

        serde_json::to_string_pretty(&export)
    }

    fn update_activity(&mut self) {
        self.state.stats.last_activity = Utc::now();
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = SessionManager::new();
        assert_eq!(session.get_state().mode, SessionMode::Normal);
        assert!(session.should_continue());
        assert_eq!(session.get_state().stats.commands_generated, 0);
    }

    #[test]
    fn test_mode_transitions() {
        let mut session = SessionManager::new();
        
        session.set_mode(SessionMode::Testing);
        assert_eq!(session.get_state().mode, SessionMode::Testing);
        
        session.set_mode(SessionMode::Configuration);
        assert_eq!(session.get_state().mode, SessionMode::Configuration);
    }

    #[test]
    fn test_command_recording() {
        let mut session = SessionManager::new();
        
        session.record_command("test command");
        assert_eq!(session.get_state().stats.total_interactions, 1);
        assert_eq!(session.get_command_history(None).len(), 1);
        
        session.record_generated_command("ls -la");
        assert_eq!(session.get_state().stats.commands_generated, 1);
        assert_eq!(session.get_generated_history(None).len(), 1);
    }

    #[test]
    fn test_history_limits() {
        let mut session = SessionManager::new();
        session.max_history_size = 3;
        
        // Add more commands than the limit
        for i in 0..5 {
            session.record_command(&format!("command {}", i));
        }
        
        // Should only keep the last 3
        assert_eq!(session.get_command_history(None).len(), 3);
        let history = session.get_command_history(None);
        assert_eq!(*history[0], "command 2");
        assert_eq!(*history[2], "command 4");
    }

    #[test]
    fn test_session_config() {
        let mut session = SessionManager::new();
        
        session.set_config("theme", "dark");
        assert_eq!(session.get_config("theme"), Some(&"dark".to_string()));
        assert_eq!(session.get_config("nonexistent"), None);
    }

    #[test]
    fn test_session_termination() {
        let mut session = SessionManager::new();
        
        assert!(session.should_continue());
        
        session.request_exit();
        assert!(!session.should_continue());
        assert_eq!(session.get_state().mode, SessionMode::Exiting);
    }

    #[test]
    fn test_session_export() {
        let mut session = SessionManager::new();
        session.record_command("test");
        session.record_generated_command("ls");
        
        let export = session.export_session();
        assert!(export.is_ok());
        
        let json = export.unwrap();
        assert!(json.contains("commands_generated"));
        assert!(json.contains("command_history"));
    }
}