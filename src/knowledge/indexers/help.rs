//! --help output indexer
//!
//! Indexes command --help output into the Docs collection.
//! Captures inline documentation from executables.

use super::{IndexStats, Indexer, ProgressCallback};
use crate::knowledge::{backends::VectorBackend, collections::CollectionType, index::KnowledgeEntry, schema::EntryType, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::process::Command;
use std::sync::Arc;

/// --help output indexer
///
/// Discovers commands in PATH and indexes their --help output.
/// Provides a fallback for commands without man pages or tldr documentation.
pub struct HelpIndexer {
    /// Commands to index (if None, auto-discover from PATH)
    pub commands: Option<Vec<String>>,
    /// Whether to try both --help and -h
    pub try_both_flags: bool,
}

impl HelpIndexer {
    /// Create a new help indexer
    ///
    /// # Arguments
    /// * `commands` - Specific commands to index (None = auto-discover)
    /// * `try_both_flags` - Try both --help and -h
    pub fn new(commands: Option<Vec<String>>, try_both_flags: bool) -> Self {
        Self {
            commands,
            try_both_flags,
        }
    }

    /// Create indexer that auto-discovers commands from PATH
    pub fn auto_discover() -> Self {
        Self::new(None, true)
    }

    /// Create indexer for specific commands
    pub fn for_commands(commands: Vec<String>) -> Self {
        Self::new(Some(commands), true)
    }

    /// Get help output for a command
    fn get_help_output(&self, command: &str) -> Result<String> {
        // Try --help first
        if let Ok(output) = Command::new(command).arg("--help").output() {
            if output.status.success() || !output.stdout.is_empty() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }

        // Try -h if configured
        if self.try_both_flags {
            if let Ok(output) = Command::new(command).arg("-h").output() {
                if output.status.success() || !output.stdout.is_empty() {
                    return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                }
            }
        }

        Err(crate::knowledge::KnowledgeError::Indexing(
            format!("No help output available for {}", command)
        ))
    }
}

#[async_trait]
impl Indexer for HelpIndexer {
    fn name(&self) -> &'static str {
        "help"
    }

    async fn index_all(
        &self,
        backend: Arc<dyn VectorBackend>,
        progress: Option<ProgressCallback>,
    ) -> Result<IndexStats> {
        let mut stats = IndexStats::new();

        // Get command list
        let commands = match &self.commands {
            Some(cmds) => cmds.clone(),
            None => {
                // Auto-discovery not implemented yet - would need to parse PATH
                return Ok(stats);
            }
        };

        let total = commands.len();
        if total == 0 {
            return Ok(stats);
        }

        // Index each command
        for (idx, command) in commands.iter().enumerate() {
            if let Some(ref callback) = progress {
                callback(idx, total);
            }

            if !self.should_index(command) {
                stats.record_skip();
                continue;
            }

            // Get help output
            match self.get_help_output(command) {
                Ok(help_text) => {
                    let entry = KnowledgeEntry {
                        request: command.clone(),
                        command: help_text,
                        context: Some(format!("help:{}", command)),
                        similarity: 0.0,
                        timestamp: Utc::now(),
                        entry_type: EntryType::Success,
                        original_command: None,
                        feedback: None,
                    };

                    match backend.add_entry(entry, CollectionType::Docs).await {
                        Ok(()) => stats.record_success(),
                        Err(_) => stats.record_failure(),
                    }
                }
                Err(_) => stats.record_failure(),
            }
        }

        if let Some(ref callback) = progress {
            callback(total, total);
        }

        Ok(stats)
    }

    async fn index_one(
        &self,
        backend: Arc<dyn VectorBackend>,
        item: &str,
    ) -> Result<bool> {
        // Try to get help output
        let help_text = self.get_help_output(item)?;

        let entry = KnowledgeEntry {
            request: item.to_string(),
            command: help_text,
            context: Some(format!("help:{}", item)),
            similarity: 0.0,
            timestamp: Utc::now(),
            entry_type: EntryType::Success,
            original_command: None,
            feedback: None,
        };

        backend.add_entry(entry, CollectionType::Docs).await?;
        Ok(true)
    }

    fn should_index(&self, item: &str) -> bool {
        // Skip known problematic commands
        let skip_list = ["bash", "sh", "zsh", "fish", "vim", "emacs", "nano"];

        !skip_list.contains(&item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_indexer_new() {
        let commands = vec!["ls".to_string(), "grep".to_string()];
        let indexer = HelpIndexer::new(Some(commands.clone()), true);
        assert_eq!(indexer.commands, Some(commands));
        assert!(indexer.try_both_flags);
        assert_eq!(indexer.name(), "help");
    }

    #[test]
    fn test_help_indexer_auto_discover() {
        let indexer = HelpIndexer::auto_discover();
        assert!(indexer.commands.is_none());
        assert!(indexer.try_both_flags);
    }

    #[test]
    fn test_help_indexer_for_commands() {
        let commands = vec!["git".to_string(), "cargo".to_string()];
        let indexer = HelpIndexer::for_commands(commands.clone());
        assert_eq!(indexer.commands, Some(commands));
    }
}
