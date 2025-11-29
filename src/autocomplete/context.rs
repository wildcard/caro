//! Completion context - Command signature metadata and parsing
//!
//! This module provides the data structures and parsing logic for understanding
//! command structure and providing relevant context for autocomplete inference.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::AutocompleteError;

/// Completion context for command autocomplete
pub struct CompletionContext {
    /// Map of command names to their signatures
    signatures: HashMap<String, CommandSignature>,
}

impl CompletionContext {
    /// Create new empty completion context
    pub fn new() -> Result<Self, AutocompleteError> {
        let mut context = Self {
            signatures: HashMap::new(),
        };

        // Load built-in command signatures
        context.load_builtin_signatures()?;

        Ok(context)
    }

    /// Load built-in command signatures for common commands
    fn load_builtin_signatures(&mut self) -> Result<(), AutocompleteError> {
        // Git commands
        self.add_signature(CommandSignature {
            command: "git".to_string(),
            description: "Git version control system".to_string(),
            subcommands: vec![
                SubcommandSpec {
                    name: "commit".to_string(),
                    description: "Record changes to the repository".to_string(),
                    flags: vec![
                        FlagSpec {
                            short: Some('m'),
                            long: Some("message".to_string()),
                            description: "Commit message".to_string(),
                            takes_value: true,
                            value_spec: Some(ArgumentSpec::String {
                                pattern: None,
                                examples: vec!["Initial commit".to_string(), "Fix bug".to_string()],
                            }),
                        },
                        FlagSpec {
                            short: Some('a'),
                            long: Some("all".to_string()),
                            description: "Automatically stage modified and deleted files".to_string(),
                            takes_value: false,
                            value_spec: None,
                        },
                    ],
                    arguments: vec![],
                },
                SubcommandSpec {
                    name: "add".to_string(),
                    description: "Add file contents to the index".to_string(),
                    flags: vec![],
                    arguments: vec![ArgumentSpec::File {
                        must_exist: false,
                        extensions: None,
                    }],
                },
                SubcommandSpec {
                    name: "status".to_string(),
                    description: "Show the working tree status".to_string(),
                    flags: vec![
                        FlagSpec {
                            short: Some('s'),
                            long: Some("short".to_string()),
                            description: "Show status in short format".to_string(),
                            takes_value: false,
                            value_spec: None,
                        },
                    ],
                    arguments: vec![],
                },
            ],
            global_flags: vec![
                FlagSpec {
                    short: None,
                    long: Some("version".to_string()),
                    description: "Show version information".to_string(),
                    takes_value: false,
                    value_spec: None,
                },
            ],
        });

        // Cargo commands
        self.add_signature(CommandSignature {
            command: "cargo".to_string(),
            description: "Rust package manager".to_string(),
            subcommands: vec![
                SubcommandSpec {
                    name: "build".to_string(),
                    description: "Compile the current package".to_string(),
                    flags: vec![
                        FlagSpec {
                            short: None,
                            long: Some("release".to_string()),
                            description: "Build in release mode".to_string(),
                            takes_value: false,
                            value_spec: None,
                        },
                    ],
                    arguments: vec![],
                },
                SubcommandSpec {
                    name: "test".to_string(),
                    description: "Run tests".to_string(),
                    flags: vec![],
                    arguments: vec![ArgumentSpec::String {
                        pattern: None,
                        examples: vec!["test_name".to_string()],
                    }],
                },
            ],
            global_flags: vec![],
        });

        // Add more built-in commands as needed
        Ok(())
    }

    /// Get completion context for a partial command
    pub fn get_context_for_command(
        &self,
        partial_command: &str,
        cursor_position: usize,
    ) -> Result<CommandContext, AutocompleteError> {
        if cursor_position > partial_command.len() {
            return Err(AutocompleteError::InvalidCursorPosition {
                position: cursor_position,
                length: partial_command.len(),
            });
        }

        // Parse the command to extract base command
        let tokens: Vec<&str> = partial_command.split_whitespace().collect();
        if tokens.is_empty() {
            return Ok(CommandContext::empty());
        }

        let base_command = tokens[0];
        let signature = self.signatures.get(base_command);

        // Check if there's trailing whitespace to detect flag value completion
        let has_trailing_space = partial_command.ends_with(' ') || partial_command.ends_with('\t');

        // Determine what we're completing (subcommand, flag, or argument)
        let completion_type = self.determine_completion_type(&tokens, cursor_position, has_trailing_space);

        Ok(CommandContext {
            base_command: base_command.to_string(),
            signature: signature.cloned(),
            tokens: tokens.iter().map(|s| s.to_string()).collect(),
            cursor_position,
            completion_type,
        })
    }

    /// Determine what type of completion is needed based on cursor position
    fn determine_completion_type(&self, tokens: &[&str], _cursor_position: usize, has_trailing_space: bool) -> CompletionType {
        if tokens.len() <= 1 {
            return CompletionType::Command;
        }

        // If last token starts with - and we have trailing space, we're completing the flag's value
        if let Some(last) = tokens.last() {
            if last.starts_with('-') {
                if has_trailing_space {
                    // After "git commit -m ", we're completing the value for -m
                    return CompletionType::FlagValue;
                } else {
                    // In the middle of typing "git commit -m", we're completing the flag itself
                    return CompletionType::Flag;
                }
            }
        }

        // If previous token was a flag that takes a value and we don't have a trailing space
        if tokens.len() >= 2 && !has_trailing_space {
            let prev = tokens[tokens.len() - 2];
            if prev.starts_with('-') {
                return CompletionType::FlagValue;
            }
        }

        // Check if we're completing a subcommand
        if tokens.len() == 2 && !has_trailing_space {
            return CompletionType::Subcommand;
        }

        CompletionType::Argument
    }

    /// Add a command signature to the context
    pub fn add_signature(&mut self, signature: CommandSignature) {
        self.signatures
            .insert(signature.command.clone(), signature);
    }

    /// Load completion definitions from file
    pub fn load_from_file(&mut self, path: impl AsRef<Path>) -> Result<(), AutocompleteError> {
        let content = std::fs::read_to_string(path)?;
        let signatures: Vec<CommandSignature> = serde_json::from_str(&content)
            .map_err(|e| AutocompleteError::ContextError {
                message: format!("Failed to parse completion file: {}", e),
            })?;

        for signature in signatures {
            self.add_signature(signature);
        }

        Ok(())
    }
}

/// Context for a specific command being completed
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Base command name (e.g., "git", "cargo")
    pub base_command: String,
    /// Command signature if available
    pub signature: Option<CommandSignature>,
    /// Parsed tokens from the command
    pub tokens: Vec<String>,
    /// Cursor position in the original command
    pub cursor_position: usize,
    /// Type of completion needed
    pub completion_type: CompletionType,
}

impl CommandContext {
    fn empty() -> Self {
        Self {
            base_command: String::new(),
            signature: None,
            tokens: Vec::new(),
            cursor_position: 0,
            completion_type: CompletionType::Command,
        }
    }
}

impl std::fmt::Display for CommandContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Command: {}, Type: {:?}, Tokens: {}",
            self.base_command,
            self.completion_type,
            self.tokens.len()
        )
    }
}

/// Type of completion being performed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionType {
    /// Completing the base command
    Command,
    /// Completing a subcommand
    Subcommand,
    /// Completing a flag
    Flag,
    /// Completing a value for a flag
    FlagValue,
    /// Completing a positional argument
    Argument,
}

/// Complete signature for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSignature {
    /// Command name (e.g., "git", "cargo")
    pub command: String,
    /// Brief description of what the command does
    pub description: String,
    /// Available subcommands
    pub subcommands: Vec<SubcommandSpec>,
    /// Global flags applicable to all subcommands
    pub global_flags: Vec<FlagSpec>,
}

/// Specification for a subcommand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcommandSpec {
    /// Subcommand name (e.g., "commit", "build")
    pub name: String,
    /// Brief description
    pub description: String,
    /// Flags specific to this subcommand
    pub flags: Vec<FlagSpec>,
    /// Positional arguments
    pub arguments: Vec<ArgumentSpec>,
}

/// Specification for a command flag/option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagSpec {
    /// Short flag (e.g., 'm' for -m)
    pub short: Option<char>,
    /// Long flag (e.g., "message" for --message)
    pub long: Option<String>,
    /// Description of what this flag does
    pub description: String,
    /// Whether this flag takes a value
    pub takes_value: bool,
    /// Specification for the value if takes_value is true
    pub value_spec: Option<ArgumentSpec>,
}

/// Specification for command arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ArgumentSpec {
    /// String argument with optional pattern and examples
    String {
        pattern: Option<String>,
        examples: Vec<String>,
    },
    /// File path argument
    File {
        must_exist: bool,
        extensions: Option<Vec<String>>,
    },
    /// Directory path argument
    Directory { must_exist: bool },
    /// Enum with fixed set of values
    Enum { values: Vec<String> },
    /// Integer with optional range
    Integer { min: Option<i64>, max: Option<i64> },
    /// Boolean flag
    Boolean,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = CompletionContext::new().unwrap();
        assert!(context.signatures.contains_key("git"));
        assert!(context.signatures.contains_key("cargo"));
    }

    #[test]
    fn test_get_context_for_git_commit() {
        let context = CompletionContext::new().unwrap();
        let cmd_context = context
            .get_context_for_command("git commit -m ", 14)
            .unwrap();

        assert_eq!(cmd_context.base_command, "git");
        assert!(cmd_context.signature.is_some());
        assert_eq!(cmd_context.completion_type, CompletionType::FlagValue);
    }

    #[test]
    fn test_invalid_cursor_position() {
        let context = CompletionContext::new().unwrap();
        let result = context.get_context_for_command("git", 10);
        assert!(result.is_err());
    }
}
