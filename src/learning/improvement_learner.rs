//! Improvement learner - learns from user edits to generated commands
//!
//! When a user modifies a generated command, this module analyzes the differences
//! and extracts improvement patterns that can be applied to future commands.

use crate::learning::pattern_db::PatternDB;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

/// Improvement pattern learned from user edits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementPattern {
    pub original_template: String,
    pub improvement_template: String,
    pub frequency: u32,
    pub contexts: Vec<String>,
    pub description: String,
}

/// Learns from command edits and suggests improvements
pub struct ImprovementLearner {
    db: PatternDB,
}

impl ImprovementLearner {
    /// Create a new improvement learner
    pub fn new(db: PatternDB) -> Self {
        Self { db }
    }

    /// Analyze an edit and extract improvement patterns
    pub async fn analyze_edit(&self, original: &str, edited: &str) -> Option<ImprovementPattern> {
        // Don't learn from empty edits
        if original == edited || edited.trim().is_empty() {
            return None;
        }

        // Compute text diff
        let diff = TextDiff::from_lines(original, edited);

        // Extract what was added or changed
        let mut additions = Vec::new();
        let mut deletions = Vec::new();

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Insert => additions.push(change.value().trim().to_string()),
                ChangeTag::Delete => deletions.push(change.value().trim().to_string()),
                ChangeTag::Equal => {}
            }
        }

        // If only additions, extract the pattern
        if !additions.is_empty() {
            let improvement = self.extract_improvement_pattern(original, edited, &additions);

            if let Some(pattern) = improvement {
                // Store the pattern
                let contexts = vec!["general".to_string()];
                let _ = self
                    .db
                    .store_improvement_pattern(original, edited, &contexts)
                    .await;

                return Some(pattern);
            }
        }

        None
    }

    /// Extract improvement pattern from additions
    fn extract_improvement_pattern(
        &self,
        original: &str,
        edited: &str,
        _additions: &[String],
    ) -> Option<ImprovementPattern> {
        // Detect common improvement patterns

        // Pattern 1: Added flags
        if let Some(flags) = self.extract_added_flags(original, edited) {
            return Some(ImprovementPattern {
                original_template: self.generalize_command(original),
                improvement_template: self.generalize_command(edited),
                frequency: 1,
                contexts: vec!["flag_addition".to_string()],
                description: format!("Added flags: {}", flags.join(", ")),
            });
        }

        // Pattern 2: Added pipe to another command
        if edited.contains('|') && !original.contains('|') {
            let pipe_cmd = edited.split('|').nth(1).unwrap_or("").trim();
            return Some(ImprovementPattern {
                original_template: self.generalize_command(original),
                improvement_template: self.generalize_command(edited),
                frequency: 1,
                contexts: vec!["pipe_addition".to_string()],
                description: format!("Added pipe to: {}", pipe_cmd),
            });
        }

        // Pattern 3: Added output redirection
        if (edited.contains('>') || edited.contains(">>")) &&
           !(original.contains('>') || original.contains(">>")) {
            return Some(ImprovementPattern {
                original_template: self.generalize_command(original),
                improvement_template: self.generalize_command(edited),
                frequency: 1,
                contexts: vec!["redirection_addition".to_string()],
                description: "Added output redirection".to_string(),
            });
        }

        // Pattern 4: Changed to use safer/better alternative
        if let Some(alt) = self.detect_alternative_command(original, edited) {
            return Some(ImprovementPattern {
                original_template: self.generalize_command(original),
                improvement_template: self.generalize_command(edited),
                frequency: 1,
                contexts: vec!["command_alternative".to_string()],
                description: format!("Preferred alternative: {}", alt),
            });
        }

        None
    }

    /// Extract flags that were added
    fn extract_added_flags(&self, original: &str, edited: &str) -> Option<Vec<String>> {
        // Only check the first command (before any pipe)
        let orig_cmd = original.split('|').next().unwrap_or(original);
        let edited_cmd = edited.split('|').next().unwrap_or(edited);

        let orig_parts: Vec<&str> = orig_cmd.split_whitespace().collect();
        let edited_parts: Vec<&str> = edited_cmd.split_whitespace().collect();

        let mut added_flags = Vec::new();

        for part in &edited_parts {
            if part.starts_with('-') && !orig_parts.contains(part) {
                added_flags.push(part.to_string());
            }
        }

        if added_flags.is_empty() {
            None
        } else {
            Some(added_flags)
        }
    }

    /// Detect if a command was replaced with an alternative
    fn detect_alternative_command(&self, original: &str, edited: &str) -> Option<String> {
        let orig_cmd = original.split_whitespace().next()?;
        let edited_cmd = edited.split_whitespace().next()?;

        if orig_cmd != edited_cmd {
            // Check for known alternatives
            let alternatives = vec![
                ("cat", "bat"),          // bat is a better cat
                ("ls", "exa"),           // exa is a better ls
                ("find", "fd"),          // fd is a better find
                ("grep", "rg"),          // ripgrep is faster
                ("du", "dust"),          // dust is better du
                ("ps", "procs"),         // procs is better ps
            ];

            for (old, new) in alternatives {
                if orig_cmd == old && edited_cmd == new {
                    return Some(new.to_string());
                }
            }
        }

        None
    }

    /// Generalize a command to create a template
    fn generalize_command(&self, command: &str) -> String {
        // For now, return the command as-is
        // In future: replace specific paths/values with placeholders
        command.to_string()
    }

    /// Suggest improvements for a command based on learned patterns
    pub async fn suggest_improvements(&self, command: &str) -> Result<Vec<ImprovementPattern>> {
        let patterns = self.db.get_improvement_patterns().await?;

        let mut suggestions = Vec::new();

        for pattern in patterns {
            // Check if this pattern might apply
            if self.pattern_matches(command, &pattern.original_template) {
                let contexts: Vec<String> = serde_json::from_str(&pattern.contexts)
                    .unwrap_or_else(|_| vec!["general".to_string()]);

                suggestions.push(ImprovementPattern {
                    original_template: pattern.original_template,
                    improvement_template: pattern.improvement_template,
                    frequency: pattern.frequency as u32,
                    contexts,
                    description: format!(
                        "Based on {} similar edits",
                        pattern.frequency
                    ),
                });
            }
        }

        // Sort by frequency (most common patterns first)
        suggestions.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        Ok(suggestions)
    }

    /// Check if a pattern matches a command
    fn pattern_matches(&self, command: &str, pattern: &str) -> bool {
        // Simple matching: check if commands start with same base command
        let cmd_base = command.split_whitespace().next().unwrap_or("");
        let pattern_base = pattern.split_whitespace().next().unwrap_or("");

        cmd_base == pattern_base
    }

    /// Apply a learned pattern to improve a command
    pub async fn apply_learned_pattern(
        &self,
        command: &str,
    ) -> Option<String> {
        let suggestions = self.suggest_improvements(command).await.ok()?;

        // Return the most frequent improvement
        suggestions.first().map(|s| s.improvement_template.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_analyze_flag_addition() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        let learner = ImprovementLearner::new(db);

        let original = "ls";
        let edited = "ls -la";

        let pattern = learner.analyze_edit(original, edited).await;
        assert!(pattern.is_some());

        let pattern = pattern.unwrap();
        assert!(pattern.description.contains("-la"));
    }

    #[tokio::test]
    async fn test_analyze_pipe_addition() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        let learner = ImprovementLearner::new(db);

        let original = "find . -name '*.log'";
        let edited = "find . -name '*.log' | wc -l";

        let pattern = learner.analyze_edit(original, edited).await;
        assert!(pattern.is_some());

        let pattern = pattern.unwrap();
        assert!(pattern.contexts.contains(&"pipe_addition".to_string()));
    }

    #[tokio::test]
    async fn test_no_pattern_for_same_command() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        let learner = ImprovementLearner::new(db);

        let pattern = learner.analyze_edit("ls -la", "ls -la").await;
        assert!(pattern.is_none());
    }

    #[tokio::test]
    async fn test_extract_added_flags() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        let learner = ImprovementLearner::new(db);

        let flags = learner.extract_added_flags("grep test", "grep -r -i test");
        assert!(flags.is_some());

        let flags = flags.unwrap();
        assert!(flags.contains(&"-r".to_string()));
        assert!(flags.contains(&"-i".to_string()));
    }
}
