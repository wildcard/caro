//! Slash command parsing and validation
//!
//! Handles detection and parsing of slash commands from user input,
//! providing structured command representation for execution.

use std::collections::HashMap;

/// Parsed slash command with arguments
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCommand {
    pub command: SlashCommand,
    pub args: Vec<String>,
    pub raw_input: String,
}

/// Available slash commands
#[derive(Debug, Clone, PartialEq)]
pub enum SlashCommand {
    Test,
    Help,
    Config,
    History,
    Stats,
    Clear,
    Exit,
    Unknown(String),
}

impl SlashCommand {
    /// Get command description for help
    pub fn description(&self) -> &'static str {
        match self {
            SlashCommand::Test => "Launch interactive testing system",
            SlashCommand::Help => "Show available slash commands",
            SlashCommand::Config => "Access configuration settings",
            SlashCommand::History => "View command history",
            SlashCommand::Stats => "Show session statistics",
            SlashCommand::Clear => "Clear the screen",
            SlashCommand::Exit => "Exit cmdai",
            SlashCommand::Unknown(_) => "Unknown command",
        }
    }

    /// Get command usage syntax
    pub fn usage(&self) -> &'static str {
        match self {
            SlashCommand::Test => "/test [category]",
            SlashCommand::Help => "/help [command]",
            SlashCommand::Config => "/config [show|edit|reset]",
            SlashCommand::History => "/history [count]",
            SlashCommand::Stats => "/stats",
            SlashCommand::Clear => "/clear",
            SlashCommand::Exit => "/exit",
            SlashCommand::Unknown(_) => "/<unknown>",
        }
    }

    /// Check if command requires special privileges
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, SlashCommand::Clear | SlashCommand::Exit)
    }
}

/// Slash command parser
pub struct SlashCommandParser {
    command_map: HashMap<String, SlashCommand>,
}

impl SlashCommandParser {
    /// Create a new slash command parser
    pub fn new() -> Self {
        let mut command_map = HashMap::new();
        
        // Register all available commands
        command_map.insert("test".to_string(), SlashCommand::Test);
        command_map.insert("help".to_string(), SlashCommand::Help);
        command_map.insert("config".to_string(), SlashCommand::Config);
        command_map.insert("history".to_string(), SlashCommand::History);
        command_map.insert("stats".to_string(), SlashCommand::Stats);
        command_map.insert("clear".to_string(), SlashCommand::Clear);
        command_map.insert("exit".to_string(), SlashCommand::Exit);
        
        // Add aliases
        command_map.insert("h".to_string(), SlashCommand::Help);
        command_map.insert("c".to_string(), SlashCommand::Config);
        command_map.insert("s".to_string(), SlashCommand::Stats);
        command_map.insert("q".to_string(), SlashCommand::Exit);
        command_map.insert("quit".to_string(), SlashCommand::Exit);

        Self { command_map }
    }

    /// Check if input is a slash command
    pub fn is_slash_command(input: &str) -> bool {
        input.trim().starts_with('/')
    }

    /// Parse input into a slash command
    pub fn parse(&self, input: &str) -> Option<ParsedCommand> {
        let trimmed = input.trim();
        
        if !trimmed.starts_with('/') {
            return None;
        }

        // Remove the leading slash
        let without_slash = &trimmed[1..];
        let parts: Vec<&str> = without_slash.split_whitespace().collect();
        
        if parts.is_empty() {
            return Some(ParsedCommand {
                command: SlashCommand::Help, // Default to help for bare "/"
                args: vec![],
                raw_input: input.to_string(),
            });
        }

        let command_name = parts[0].to_lowercase();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        let command = self.command_map
            .get(&command_name)
            .cloned()
            .unwrap_or_else(|| SlashCommand::Unknown(command_name));

        Some(ParsedCommand {
            command,
            args,
            raw_input: input.to_string(),
        })
    }

    /// Get all available commands
    pub fn get_available_commands(&self) -> Vec<&SlashCommand> {
        let mut commands: Vec<_> = self.command_map.values().collect();
        commands.sort_by_key(|cmd| format!("{:?}", cmd));
        commands.dedup_by_key(|cmd| format!("{:?}", cmd));
        commands
    }

    /// Validate command arguments
    pub fn validate_args(&self, parsed: &ParsedCommand) -> Result<(), String> {
        match &parsed.command {
            SlashCommand::Test => {
                // Optional category argument
                if parsed.args.len() > 1 {
                    return Err("Usage: /test [category]".to_string());
                }
                Ok(())
            }
            SlashCommand::Help => {
                // Optional command name argument
                if parsed.args.len() > 1 {
                    return Err("Usage: /help [command]".to_string());
                }
                Ok(())
            }
            SlashCommand::Config => {
                // Optional action argument
                if parsed.args.len() > 1 {
                    return Err("Usage: /config [show|edit|reset]".to_string());
                }
                if let Some(action) = parsed.args.first() {
                    match action.as_str() {
                        "show" | "edit" | "reset" => Ok(()),
                        _ => Err("Valid config actions: show, edit, reset".to_string()),
                    }
                } else {
                    Ok(())
                }
            }
            SlashCommand::History => {
                // Optional count argument
                if parsed.args.len() > 1 {
                    return Err("Usage: /history [count]".to_string());
                }
                if let Some(count_str) = parsed.args.first() {
                    if count_str.parse::<usize>().is_err() {
                        return Err("History count must be a number".to_string());
                    }
                }
                Ok(())
            }
            SlashCommand::Stats | SlashCommand::Clear | SlashCommand::Exit => {
                if !parsed.args.is_empty() {
                    return Err(format!("Command {:?} takes no arguments", parsed.command));
                }
                Ok(())
            }
            SlashCommand::Unknown(name) => {
                Err(format!("Unknown command: '{}'. Type /help for available commands", name))
            }
        }
    }
}

impl Default for SlashCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slash_command_detection() {
        assert!(SlashCommandParser::is_slash_command("/test"));
        assert!(SlashCommandParser::is_slash_command("  /help  "));
        assert!(!SlashCommandParser::is_slash_command("normal command"));
        assert!(!SlashCommandParser::is_slash_command(""));
    }

    #[test]
    fn test_command_parsing() {
        let parser = SlashCommandParser::new();
        
        // Test basic command
        let parsed = parser.parse("/test").unwrap();
        assert_eq!(parsed.command, SlashCommand::Test);
        assert!(parsed.args.is_empty());
        
        // Test command with arguments
        let parsed = parser.parse("/config show").unwrap();
        assert_eq!(parsed.command, SlashCommand::Config);
        assert_eq!(parsed.args, vec!["show"]);
        
        // Test alias
        let parsed = parser.parse("/h").unwrap();
        assert_eq!(parsed.command, SlashCommand::Help);
        
        // Test unknown command
        let parsed = parser.parse("/unknown").unwrap();
        assert!(matches!(parsed.command, SlashCommand::Unknown(_)));
        
        // Test non-slash command
        assert!(parser.parse("normal input").is_none());
    }

    #[test]
    fn test_argument_validation() {
        let parser = SlashCommandParser::new();
        
        // Valid commands
        let parsed = parser.parse("/test basic").unwrap();
        assert!(parser.validate_args(&parsed).is_ok());
        
        let parsed = parser.parse("/help").unwrap();
        assert!(parser.validate_args(&parsed).is_ok());
        
        // Invalid commands
        let parsed = parser.parse("/stats extra").unwrap();
        assert!(parser.validate_args(&parsed).is_err());
        
        let parsed = parser.parse("/history not_a_number").unwrap();
        assert!(parser.validate_args(&parsed).is_err());
    }

    #[test]
    fn test_command_descriptions() {
        assert!(!SlashCommand::Test.description().is_empty());
        assert!(!SlashCommand::Help.description().is_empty());
        assert!(!SlashCommand::Unknown("test".to_string()).description().is_empty());
    }

    #[test]
    fn test_bare_slash() {
        let parser = SlashCommandParser::new();
        let parsed = parser.parse("/").unwrap();
        assert_eq!(parsed.command, SlashCommand::Help);
    }
}