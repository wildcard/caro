// ASCII cinema recorder for demo sessions

use crate::cli::CliError;
use crate::models::GeneratedCommand;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// ASCII cinema recording format
/// Based on asciinema file format v2
/// See: https://docs.asciinema.org/manual/asciicast/v2/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiCast {
    /// Format version (always 2)
    pub version: u8,
    /// Terminal width
    pub width: u16,
    /// Terminal height
    pub height: u16,
    /// Recording timestamp
    pub timestamp: i64,
    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<AsciiCastEnv>,
    /// Recording title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiCastEnv {
    #[serde(rename = "SHELL")]
    pub shell: String,
    #[serde(rename = "TERM")]
    pub term: String,
}

/// Recording event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CastEvent {
    /// [timestamp, event_type, data]
    Output(f64, String, String),
}

/// ASCII cinema recorder
pub struct AsciiCinemaRecorder {
    output_path: PathBuf,
    header: AsciiCast,
    events: Vec<CastEvent>,
    start_time: Instant,
    last_event_time: f64,
}

impl AsciiCinemaRecorder {
    /// Create a new recorder
    pub fn new(output_path: PathBuf) -> Result<Self, CliError> {
        let header = AsciiCast {
            version: 2,
            width: Self::get_terminal_width(),
            height: Self::get_terminal_height(),
            timestamp: Utc::now().timestamp(),
            env: Some(AsciiCastEnv {
                shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
                term: std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string()),
            }),
            title: Some("cmdai demo session".to_string()),
        };

        Ok(Self {
            output_path,
            header,
            events: Vec::new(),
            start_time: Instant::now(),
            last_event_time: 0.0,
        })
    }

    /// Get terminal width (default to 80 if can't detect)
    fn get_terminal_width() -> u16 {
        term_size::dimensions()
            .map(|(w, _)| w as u16)
            .unwrap_or(80)
    }

    /// Get terminal height (default to 24 if can't detect)
    fn get_terminal_height() -> u16 {
        term_size::dimensions()
            .map(|(_, h)| h as u16)
            .unwrap_or(24)
    }

    /// Get current timestamp relative to recording start
    fn get_timestamp(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Record user input
    pub fn record_input(&mut self, input: &str) -> Result<(), CliError> {
        let timestamp = self.get_timestamp();
        let formatted_input = format!("$ cmdai \"{}\"\r\n", input);

        self.events.push(CastEvent::Output(
            timestamp,
            "o".to_string(),
            formatted_input,
        ));
        self.last_event_time = timestamp;

        Ok(())
    }

    /// Record command output
    pub fn record_output(&mut self, generated: &GeneratedCommand) -> Result<(), CliError> {
        // Add small delay for realism
        std::thread::sleep(Duration::from_millis(100));
        let timestamp = self.get_timestamp();

        let mut output = String::new();

        // Add critique if present (demo mode specific)
        if generated.explanation.contains("CRITIQUE") {
            output.push_str("\x1b[33m");  // Yellow color
            output.push_str("⚠️  DEMO MODE CRITIQUE\x1b[0m\r\n");
        }

        // Command header
        output.push_str("\x1b[1mCommand:\x1b[0m\r\n");
        output.push_str(&format!("  \x1b[96m{}\x1b[0m\r\n\r\n", generated.command));

        // Explanation
        if !generated.explanation.is_empty() {
            output.push_str("\x1b[1mExplanation:\x1b[0m\r\n");
            for line in generated.explanation.lines() {
                output.push_str(&format!("  {}\r\n", line));
            }
            output.push_str("\r\n");
        }

        // Alternatives
        if !generated.alternatives.is_empty() {
            output.push_str("\x1b[1mAlternatives:\x1b[0m\r\n");
            for alt in &generated.alternatives {
                output.push_str(&format!("  \x1b[2m• {}\x1b[0m\r\n", alt));
            }
            output.push_str("\r\n");
        }

        self.events
            .push(CastEvent::Output(timestamp, "o".to_string(), output));
        self.last_event_time = timestamp;

        Ok(())
    }

    /// Record a text message
    pub fn record_message(&mut self, message: &str) -> Result<(), CliError> {
        let timestamp = self.get_timestamp();
        let formatted_message = format!("{}\r\n", message);

        self.events.push(CastEvent::Output(
            timestamp,
            "o".to_string(),
            formatted_message,
        ));
        self.last_event_time = timestamp;

        Ok(())
    }

    /// Save the recording to file
    pub fn save(&self) -> Result<(), CliError> {
        let mut file = File::create(&self.output_path).map_err(|e| CliError::Internal {
            message: format!("Failed to create recording file: {}", e),
        })?;

        // Write header
        let header_json = serde_json::to_string(&self.header).map_err(|e| CliError::Internal {
            message: format!("Failed to serialize header: {}", e),
        })?;
        writeln!(file, "{}", header_json).map_err(|e| CliError::Internal {
            message: format!("Failed to write header: {}", e),
        })?;

        // Write events
        for event in &self.events {
            match event {
                CastEvent::Output(timestamp, event_type, data) => {
                    let event_json = serde_json::to_string(&(*timestamp, event_type, data))
                        .map_err(|e| CliError::Internal {
                            message: format!("Failed to serialize event: {}", e),
                        })?;
                    writeln!(file, "{}", event_json).map_err(|e| CliError::Internal {
                        message: format!("Failed to write event: {}", e),
                    })?;
                }
            }
        }

        Ok(())
    }
}

// Terminal size detection module
mod term_size {
    pub fn dimensions() -> Option<(usize, usize)> {
        // Try to get terminal size using termion or similar
        // For now, return None to use defaults
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_recorder_creation() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("demo.cast");

        let recorder = AsciiCinemaRecorder::new(output_path);
        assert!(recorder.is_ok());
    }

    #[test]
    fn test_record_and_save() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("demo.cast");

        let mut recorder = AsciiCinemaRecorder::new(output_path.clone()).unwrap();
        recorder.record_input("list all files").unwrap();
        recorder.save().unwrap();

        assert!(output_path.exists());
    }
}
