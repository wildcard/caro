//! --help output indexer
//!
//! Indexes command --help output into the Docs collection.
//! Captures inline documentation from executables.

use super::{IndexStats, Indexer, ProgressCallback};
use crate::knowledge::{backends::VectorBackend, Result};
use async_trait::async_trait;
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
}

#[async_trait]
impl Indexer for HelpIndexer {
    fn name(&self) -> &'static str {
        "help"
    }

    async fn index_all(
        &self,
        _backend: Arc<dyn VectorBackend>,
        _progress: Option<ProgressCallback>,
    ) -> Result<IndexStats> {
        // TODO: Implement --help output discovery and indexing
        // 1. Get list of commands:
        //    - If commands specified, use that list
        //    - Otherwise, discover from PATH (parse $PATH, list executables)
        // 2. For each command:
        //    - Try running `command --help` (capture stdout/stderr)
        //    - If that fails and try_both_flags, try `command -h`
        //    - Parse help output for structure (usage, options, examples)
        //    - Create KnowledgeEntry with command name as request
        //    - Add to Docs collection via backend.add_entry()
        // 3. Report progress via callback
        // 4. Handle errors gracefully (some commands may not support --help)

        Ok(IndexStats::new())
    }

    async fn index_one(
        &self,
        _backend: Arc<dyn VectorBackend>,
        _item: &str,
    ) -> Result<bool> {
        // TODO: Index --help output for a specific command
        // 1. Check if command exists in PATH
        // 2. Run --help or -h
        // 3. Parse and index output

        Ok(false)
    }

    fn should_index(&self, _item: &str) -> bool {
        // TODO: Implement filtering logic
        // - Skip shell builtins (they don't have --help)
        // - Skip non-executable files
        // - Skip known problematic commands (e.g., interactive shells)

        true
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
