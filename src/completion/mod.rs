//! Shell completion generation for Caro CLI
//!
//! This module provides:
//! - Static shell completion script generation (bash, zsh, fish)
//! - Natural language command suggestions from the pattern library
//!
//! # Example
//!
//! ```bash
//! # Generate bash completions
//! caro completion bash > ~/.bash_completion.d/caro
//!
//! # Get command suggestions
//! caro suggest "find files"
//! ```

mod generator;
mod suggest;

pub use generator::generate_completions;
pub use suggest::{suggest_commands, CommandSuggestion};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ShellType;

    #[test]
    fn test_generate_bash_completions() {
        let script = generate_completions(ShellType::Bash);
        assert!(
            !script.is_empty(),
            "Bash completion script should not be empty"
        );
        assert!(
            script.contains("caro"),
            "Bash completion should contain command name"
        );
        // Bash completions typically contain _caro or complete -F
        assert!(
            script.contains("complete") || script.contains("_caro"),
            "Bash completion should have completion function"
        );
    }

    #[test]
    fn test_generate_zsh_completions() {
        let script = generate_completions(ShellType::Zsh);
        assert!(
            !script.is_empty(),
            "Zsh completion script should not be empty"
        );
        assert!(
            script.contains("caro"),
            "Zsh completion should contain command name"
        );
        // Zsh completions typically use #compdef or _arguments
        assert!(
            script.contains("#compdef") || script.contains("_arguments"),
            "Zsh completion should have compdef or _arguments"
        );
    }

    #[test]
    fn test_generate_fish_completions() {
        let script = generate_completions(ShellType::Fish);
        assert!(
            !script.is_empty(),
            "Fish completion script should not be empty"
        );
        assert!(
            script.contains("caro"),
            "Fish completion should contain command name"
        );
        // Fish completions use complete -c
        assert!(
            script.contains("complete -c"),
            "Fish completion should use 'complete -c'"
        );
    }

    #[test]
    fn test_suggest_disk_commands() {
        let suggestions = suggest_commands("disk space", 5);
        assert!(
            !suggestions.is_empty(),
            "Should return suggestions for 'disk space'"
        );

        // At least one suggestion should contain 'du'
        let has_du = suggestions.iter().any(|s| s.command.contains("du"));
        assert!(has_du, "Disk space suggestions should include 'du' command");
    }

    #[test]
    fn test_suggest_find_commands() {
        let suggestions = suggest_commands("find files", 5);
        assert!(
            !suggestions.is_empty(),
            "Should return suggestions for 'find files'"
        );

        // At least one suggestion should contain 'find'
        let has_find = suggestions.iter().any(|s| s.command.contains("find"));
        assert!(has_find, "Find suggestions should include 'find' command");
    }

    #[test]
    fn test_suggest_returns_limited_results() {
        let suggestions = suggest_commands("files", 3);
        assert!(
            suggestions.len() <= 3,
            "Should return at most 3 suggestions when limit is 3"
        );
    }

    #[test]
    fn test_suggest_empty_query_returns_empty() {
        let suggestions = suggest_commands("", 5);
        assert!(
            suggestions.is_empty(),
            "Empty query should return no suggestions"
        );
    }

    #[test]
    fn test_command_suggestion_has_description() {
        let suggestions = suggest_commands("list files", 1);
        if !suggestions.is_empty() {
            assert!(
                !suggestions[0].description.is_empty(),
                "Suggestion should have a description"
            );
        }
    }
}
