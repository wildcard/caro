//! Shell history analysis for command patterns

use super::{Result, SuggestionError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The command that was executed
    pub command: String,

    /// Timestamp if available
    pub timestamp: Option<i64>,

    /// Exit code if available
    pub exit_code: Option<i32>,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            timestamp: None,
            exit_code: None,
        }
    }

    /// Get the base command (first word)
    pub fn base_command(&self) -> &str {
        self.command
            .split_whitespace()
            .next()
            .unwrap_or(&self.command)
    }
}

/// Command usage patterns extracted from history
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandPatterns {
    /// Most frequently used commands with counts
    pub top_commands: Vec<(String, u32)>,

    /// Common command sequences (e.g., git add -> git commit)
    pub common_sequences: Vec<Vec<String>>,

    /// Hour-of-day usage distribution (0-23)
    pub usage_hours: [u32; 24],

    /// Total number of commands analyzed
    pub total_commands: usize,

    /// Number of unique commands
    pub unique_commands: usize,
}

/// Analyzes shell history files
pub struct HistoryAnalyzer {
    /// Maximum number of history entries to analyze
    max_entries: usize,
}

impl Default for HistoryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryAnalyzer {
    /// Create a new history analyzer
    pub fn new() -> Self {
        Self { max_entries: 10000 }
    }

    /// Set maximum entries to analyze
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Analyze all available history files
    pub async fn analyze(&self) -> Result<CommandPatterns> {
        let mut all_entries = Vec::new();

        // Try each shell's history file
        for path in self.find_history_files() {
            if let Ok(entries) = self.parse_history_file(&path).await {
                all_entries.extend(entries);
            }
        }

        // Limit to max entries (most recent)
        if all_entries.len() > self.max_entries {
            all_entries = all_entries
                .into_iter()
                .rev()
                .take(self.max_entries)
                .collect();
        }

        self.extract_patterns(&all_entries)
    }

    /// Find all history files on the system
    fn find_history_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if let Some(home) = dirs::home_dir() {
            // Bash history
            let bash_history = home.join(".bash_history");
            if bash_history.exists() {
                files.push(bash_history);
            }

            // Zsh history
            let zsh_history = home.join(".zsh_history");
            if zsh_history.exists() {
                files.push(zsh_history);
            }

            // Alternative zsh history location
            let zsh_hist_alt = home.join(".zhistory");
            if zsh_hist_alt.exists() {
                files.push(zsh_hist_alt);
            }

            // Fish history
            let fish_history = home.join(".local/share/fish/fish_history");
            if fish_history.exists() {
                files.push(fish_history);
            }
        }

        files
    }

    /// Parse a history file
    async fn parse_history_file(&self, path: &PathBuf) -> Result<Vec<HistoryEntry>> {
        let contents = tokio::fs::read_to_string(path).await.map_err(|e| {
            SuggestionError::HistoryReadError(format!("{}: {}", path.display(), e))
        })?;

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let entries = if filename.contains("fish") {
            self.parse_fish_history(&contents)
        } else if filename.contains("zsh") || filename == ".zhistory" {
            self.parse_zsh_history(&contents)
        } else {
            self.parse_bash_history(&contents)
        };

        Ok(entries)
    }

    /// Parse bash history format (simple line-based)
    fn parse_bash_history(&self, contents: &str) -> Vec<HistoryEntry> {
        contents
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| HistoryEntry::new(line.trim()))
            .collect()
    }

    /// Parse zsh history format (handles extended format)
    fn parse_zsh_history(&self, contents: &str) -> Vec<HistoryEntry> {
        let mut entries = Vec::new();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Zsh extended history format: : timestamp:duration;command
            if line.starts_with(": ") {
                if let Some(semicolon_pos) = line.find(';') {
                    let command = &line[semicolon_pos + 1..];
                    if !command.is_empty() {
                        let mut entry = HistoryEntry::new(command);

                        // Try to extract timestamp
                        let time_part = &line[2..semicolon_pos];
                        if let Some(colon_pos) = time_part.find(':') {
                            if let Ok(ts) = time_part[..colon_pos].parse::<i64>() {
                                entry.timestamp = Some(ts);
                            }
                        }

                        entries.push(entry);
                    }
                }
            } else {
                // Simple format
                entries.push(HistoryEntry::new(line));
            }
        }

        entries
    }

    /// Parse fish history format (YAML-like)
    fn parse_fish_history(&self, contents: &str) -> Vec<HistoryEntry> {
        let mut entries = Vec::new();
        let mut current_cmd: Option<String> = None;
        let mut current_when: Option<i64> = None;

        for line in contents.lines() {
            if line.starts_with("- cmd: ") {
                // Save previous entry if exists
                if let Some(cmd) = current_cmd.take() {
                    let mut entry = HistoryEntry::new(cmd);
                    entry.timestamp = current_when.take();
                    entries.push(entry);
                }
                current_cmd = Some(line[7..].to_string());
            } else if line.starts_with("  when: ") {
                if let Ok(ts) = line[8..].parse::<i64>() {
                    current_when = Some(ts);
                }
            }
        }

        // Don't forget the last entry
        if let Some(cmd) = current_cmd {
            let mut entry = HistoryEntry::new(cmd);
            entry.timestamp = current_when;
            entries.push(entry);
        }

        entries
    }

    /// Extract patterns from history entries
    fn extract_patterns(&self, entries: &[HistoryEntry]) -> Result<CommandPatterns> {
        let mut command_counts: HashMap<String, u32> = HashMap::new();
        let mut base_command_counts: HashMap<String, u32> = HashMap::new();
        let mut usage_hours = [0u32; 24];
        let mut sequences: HashMap<String, u32> = HashMap::new();

        let mut prev_command: Option<&str> = None;

        for entry in entries {
            let base_cmd = entry.base_command().to_string();

            // Count base commands
            *base_command_counts.entry(base_cmd.clone()).or_insert(0) += 1;

            // Count full commands (limited to avoid noise)
            if entry.command.len() < 100 {
                *command_counts.entry(entry.command.clone()).or_insert(0) += 1;
            }

            // Track usage hours
            if let Some(ts) = entry.timestamp {
                let hour = (ts / 3600) % 24;
                if hour >= 0 && hour < 24 {
                    usage_hours[hour as usize] += 1;
                }
            }

            // Track sequences
            if let Some(prev) = prev_command {
                let seq = format!("{} -> {}", prev, base_cmd);
                *sequences.entry(seq).or_insert(0) += 1;
            }
            prev_command = Some(entry.base_command());
        }

        // Get top commands
        let mut top_commands: Vec<(String, u32)> = base_command_counts.into_iter().collect();
        top_commands.sort_by(|a, b| b.1.cmp(&a.1));
        top_commands.truncate(20);

        // Get common sequences
        let mut seq_vec: Vec<(String, u32)> = sequences.into_iter().collect();
        seq_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let common_sequences: Vec<Vec<String>> = seq_vec
            .into_iter()
            .take(10)
            .filter(|(_, count)| *count > 2)
            .map(|(seq, _)| seq.split(" -> ").map(String::from).collect())
            .collect();

        Ok(CommandPatterns {
            top_commands,
            common_sequences,
            usage_hours,
            total_commands: entries.len(),
            unique_commands: command_counts.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_entry() {
        let entry = HistoryEntry::new("git status");
        assert_eq!(entry.base_command(), "git");
    }

    #[test]
    fn test_parse_bash_history() {
        let analyzer = HistoryAnalyzer::new();
        let content = "ls -la\ncd /tmp\ngit status\n";
        let entries = analyzer.parse_bash_history(content);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].command, "ls -la");
        assert_eq!(entries[0].base_command(), "ls");
    }

    #[test]
    fn test_parse_zsh_extended_history() {
        let analyzer = HistoryAnalyzer::new();
        let content = ": 1699000000:0;ls -la\n: 1699000001:0;git status\n";
        let entries = analyzer.parse_zsh_history(content);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "ls -la");
        assert!(entries[0].timestamp.is_some());
    }

    #[test]
    fn test_parse_fish_history() {
        let analyzer = HistoryAnalyzer::new();
        let content = "- cmd: ls -la\n  when: 1699000000\n- cmd: git status\n  when: 1699000001\n";
        let entries = analyzer.parse_fish_history(content);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "ls -la");
        assert_eq!(entries[0].timestamp, Some(1699000000));
    }

    #[test]
    fn test_extract_patterns() {
        let analyzer = HistoryAnalyzer::new();
        let entries = vec![
            HistoryEntry::new("git status"),
            HistoryEntry::new("git add ."),
            HistoryEntry::new("git commit -m 'test'"),
            HistoryEntry::new("git status"),
            HistoryEntry::new("ls -la"),
        ];

        let patterns = analyzer.extract_patterns(&entries).unwrap();

        assert_eq!(patterns.total_commands, 5);
        assert!(!patterns.top_commands.is_empty());

        // git should be the top command
        let top_cmd = &patterns.top_commands[0];
        assert_eq!(top_cmd.0, "git");
        assert_eq!(top_cmd.1, 4);
    }
}
