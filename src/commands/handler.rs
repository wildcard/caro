//! Slash command execution and response handling
//!
//! Processes parsed slash commands and coordinates with various cmdai subsystems
//! to execute commands and provide appropriate responses to the user.

use colored::Colorize;
use std::io::{self, Write};

use crate::{
    commands::{ParsedCommand, SessionManager, SessionMode, SlashCommand},
    config::ConfigManager,
    testing::ManualTestRunner,
};

/// Response from executing a slash command
#[derive(Debug, Clone)]
pub struct CommandResponse {
    /// Whether the command was successful
    pub success: bool,
    /// Response message to display to user
    pub message: String,
    /// Whether to continue the session
    pub continue_session: bool,
    /// Optional new session mode
    pub new_mode: Option<SessionMode>,
}

impl CommandResponse {
    /// Create a successful response
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            continue_session: true,
            new_mode: None,
        }
    }

    /// Create an error response
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            continue_session: true,
            new_mode: None,
        }
    }

    /// Create a response that exits the session
    pub fn exit(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            continue_session: false,
            new_mode: Some(SessionMode::Exiting),
        }
    }

    /// Create a response with mode change
    pub fn with_mode(message: &str, mode: SessionMode) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            continue_session: true,
            new_mode: Some(mode),
        }
    }
}

/// Context information for command execution
pub struct CommandContext {
    /// Configuration manager
    pub config_manager: Option<ConfigManager>,
    /// Whether running in verbose mode
    pub verbose: bool,
    /// Current working directory
    pub cwd: String,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            config_manager: None,
            verbose: false,
            cwd: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }
    }
}

/// Main slash command handler
pub struct SlashCommandHandler {
    session_manager: SessionManager,
    context: CommandContext,
}

impl SlashCommandHandler {
    /// Create a new slash command handler
    pub fn new(context: CommandContext) -> Self {
        Self {
            session_manager: SessionManager::new(),
            context,
        }
    }

    /// Get session manager
    pub fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }

    /// Get mutable session manager
    pub fn session_manager_mut(&mut self) -> &mut SessionManager {
        &mut self.session_manager
    }

    /// Execute a parsed slash command
    pub async fn execute(&mut self, parsed: &ParsedCommand) -> CommandResponse {
        // Record the slash command execution
        self.session_manager.record_slash_command(&parsed.raw_input);

        match &parsed.command {
            SlashCommand::Test => self.handle_test(parsed).await,
            SlashCommand::Help => self.handle_help(parsed),
            SlashCommand::Config => self.handle_config(parsed).await,
            SlashCommand::History => self.handle_history(parsed),
            SlashCommand::Stats => self.handle_stats(),
            SlashCommand::Clear => self.handle_clear(),
            SlashCommand::Exit => self.handle_exit(),
            SlashCommand::Unknown(name) => CommandResponse::error(&format!(
                "Unknown command: '{}'. Type /help for available commands.",
                name
            )),
        }
    }

    /// Handle /test command
    async fn handle_test(&mut self, parsed: &ParsedCommand) -> CommandResponse {
        println!("{}", "🧪 Launching Interactive Testing System...".cyan().bold());
        
        match ManualTestRunner::new().await {
            Ok(mut test_runner) => {
                // If category specified, try to run that category
                if let Some(category_str) = parsed.args.first() {
                    // Parse category
                    let category = match category_str.to_lowercase().as_str() {
                        "basic" | "safety" => crate::testing::TestCategory::BasicSafety,
                        "dangerous" | "danger" => crate::testing::TestCategory::DangerousCommands,
                        "edge" => crate::testing::TestCategory::EdgeCases,
                        "learning" | "adaptive" => crate::testing::TestCategory::AdaptiveLearning,
                        "performance" | "perf" => crate::testing::TestCategory::PerformanceBenchmarks,
                        "integration" => crate::testing::TestCategory::IntegrationTests,
                        "custom" => crate::testing::TestCategory::CustomUser,
                        _ => {
                            return CommandResponse::error(&format!(
                                "Unknown test category: '{}'. Available: basic, dangerous, edge, learning, performance, integration, custom",
                                category_str
                            ));
                        }
                    };

                    // Run specific category
                    match test_runner.run_category_tests(category).await {
                        Ok(()) => CommandResponse::success("Test category completed successfully!"),
                        Err(e) => {
                            self.session_manager.record_error();
                            CommandResponse::error(&format!("Test execution failed: {}", e))
                        }
                    }
                } else {
                    // Run full interactive session
                    match test_runner.run_interactive_session().await {
                        Ok(()) => CommandResponse::success("Testing session completed!"),
                        Err(e) => {
                            self.session_manager.record_error();
                            CommandResponse::error(&format!("Testing session failed: {}", e))
                        }
                    }
                }
            }
            Err(e) => {
                self.session_manager.record_error();
                CommandResponse::error(&format!("Failed to initialize test runner: {}", e))
            }
        }
    }

    /// Handle /help command
    fn handle_help(&self, parsed: &ParsedCommand) -> CommandResponse {
        if let Some(command_name) = parsed.args.first() {
            // Help for specific command
            self.show_command_help(command_name)
        } else {
            // General help
            self.show_general_help()
        }
    }

    /// Handle /config command
    async fn handle_config(&mut self, parsed: &ParsedCommand) -> CommandResponse {
        let action = parsed.args.first().map(|s| s.as_str()).unwrap_or("show");

        match action {
            "show" => self.show_config().await,
            "edit" => self.edit_config().await,
            "reset" => self.reset_config().await,
            _ => CommandResponse::error("Valid config actions: show, edit, reset"),
        }
    }

    /// Handle /history command
    fn handle_history(&self, parsed: &ParsedCommand) -> CommandResponse {
        let count = if let Some(count_str) = parsed.args.first() {
            match count_str.parse::<usize>() {
                Ok(n) => Some(n),
                Err(_) => {
                    return CommandResponse::error("History count must be a number");
                }
            }
        } else {
            None
        };

        let command_history = self.session_manager.get_command_history(count);
        let generated_history = self.session_manager.get_generated_history(count);

        let mut output = String::new();
        output.push_str(&format!("{}\n", "📜 Command History:".cyan().bold()));
        
        if command_history.is_empty() {
            output.push_str("  No commands in history.\n");
        } else {
            for (i, cmd) in command_history.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, cmd));
            }
        }

        output.push_str(&format!("\n{}\n", "🤖 Generated Commands:".green().bold()));
        if generated_history.is_empty() {
            output.push_str("  No generated commands in history.\n");
        } else {
            for (i, cmd) in generated_history.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, cmd));
            }
        }

        CommandResponse::success(&output.trim())
    }

    /// Handle /stats command
    fn handle_stats(&self) -> CommandResponse {
        let summary = self.session_manager.get_session_summary();
        CommandResponse::success(&format!("{}\n{}", "📊 Session Statistics:".cyan().bold(), summary))
    }

    /// Handle /clear command
    fn handle_clear(&self) -> CommandResponse {
        // Clear the terminal screen
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap_or(());
        
        CommandResponse::success("Screen cleared.")
    }

    /// Handle /exit command
    fn handle_exit(&mut self) -> CommandResponse {
        let summary = self.session_manager.get_session_summary();
        let message = format!(
            "{}\n\n{}\n\n{}",
            "👋 Goodbye! Thanks for using cmdai.".yellow().bold(),
            summary,
            "Session ended.".dimmed()
        );
        
        self.session_manager.request_exit();
        CommandResponse::exit(&message)
    }

    /// Show help for a specific command
    fn show_command_help(&self, command_name: &str) -> CommandResponse {
        // Try to find the command
        let parser = crate::commands::SlashCommandParser::new();
        if let Some(parsed) = parser.parse(&format!("/{}", command_name)) {
            let cmd = &parsed.command;
            let help_text = format!(
                "{}: {}\nUsage: {}",
                format!("{:?}", cmd).cyan().bold(),
                cmd.description(),
                cmd.usage().yellow()
            );
            CommandResponse::success(&help_text)
        } else {
            CommandResponse::error(&format!("Unknown command: '{}'", command_name))
        }
    }

    /// Show general help
    fn show_general_help(&self) -> CommandResponse {
        let parser = crate::commands::SlashCommandParser::new();
        let commands = parser.get_available_commands();

        let mut help_text = format!("{}\n", "🔧 Available Slash Commands:".cyan().bold());
        help_text.push_str("Type /help <command> for detailed help on a specific command.\n\n");

        for cmd in commands {
            if !matches!(cmd, SlashCommand::Unknown(_)) {
                help_text.push_str(&format!(
                    "  {} - {}\n",
                    cmd.usage().yellow(),
                    cmd.description()
                ));
            }
        }

        help_text.push_str(&format!(
            "\n{}\n",
            "💡 Tip: You can use slash commands at any time during your cmdai session!".green()
        ));

        CommandResponse::success(&help_text.trim())
    }

    /// Show current configuration
    async fn show_config(&self) -> CommandResponse {
        if let Some(ref config_manager) = self.context.config_manager {
            match config_manager.load() {
                Ok(config) => {
                    let config_text = format!(
                        "{}\n{}",
                        "⚙️  Current Configuration:".cyan().bold(),
                        serde_json::to_string_pretty(&config).unwrap_or_default()
                    );
                    CommandResponse::success(&config_text)
                }
                Err(e) => CommandResponse::error(&format!("Failed to load configuration: {}", e)),
            }
        } else {
            CommandResponse::error("Configuration manager not available")
        }
    }

    /// Edit configuration interactively
    async fn edit_config(&self) -> CommandResponse {
        CommandResponse::success("Interactive configuration editing is not yet implemented. Use 'cmdai --configure' for now.")
    }

    /// Reset configuration to defaults
    async fn reset_config(&self) -> CommandResponse {
        if let Some(ref config_manager) = self.context.config_manager {
            // Create default configuration and save it
            let default_config = crate::models::UserConfiguration::default();
            match config_manager.save(&default_config) {
                Ok(()) => CommandResponse::success("Configuration reset to defaults."),
                Err(e) => CommandResponse::error(&format!("Failed to reset configuration: {}", e)),
            }
        } else {
            CommandResponse::error("Configuration manager not available")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{SlashCommandParser, SlashCommand};

    #[tokio::test]
    async fn test_help_command() {
        let mut handler = SlashCommandHandler::new(CommandContext::default());
        let parser = SlashCommandParser::new();
        
        let parsed = parser.parse("/help").unwrap();
        let response = handler.execute(&parsed).await;
        
        assert!(response.success);
        assert!(response.message.contains("Available Slash Commands"));
    }

    #[tokio::test]
    async fn test_stats_command() {
        let mut handler = SlashCommandHandler::new(CommandContext::default());
        let parser = SlashCommandParser::new();
        
        // Record some activity
        handler.session_manager_mut().record_command("test");
        handler.session_manager_mut().record_generated_command("ls");
        
        let parsed = parser.parse("/stats").unwrap();
        let response = handler.execute(&parsed).await;
        
        assert!(response.success);
        assert!(response.message.contains("Session Statistics"));
        assert!(response.message.contains("Commands Generated: 1"));
    }

    #[tokio::test]
    async fn test_history_command() {
        let mut handler = SlashCommandHandler::new(CommandContext::default());
        let parser = SlashCommandParser::new();
        
        // Add some history
        handler.session_manager_mut().record_command("first command");
        handler.session_manager_mut().record_command("second command");
        
        let parsed = parser.parse("/history").unwrap();
        let response = handler.execute(&parsed).await;
        
        assert!(response.success);
        assert!(response.message.contains("Command History"));
        assert!(response.message.contains("first command"));
    }

    #[tokio::test]
    async fn test_exit_command() {
        let mut handler = SlashCommandHandler::new(CommandContext::default());
        let parser = SlashCommandParser::new();
        
        let parsed = parser.parse("/exit").unwrap();
        let response = handler.execute(&parsed).await;
        
        assert!(response.success);
        assert!(!response.continue_session);
        assert!(matches!(response.new_mode, Some(SessionMode::Exiting)));
    }

    #[tokio::test]
    async fn test_unknown_command() {
        let mut handler = SlashCommandHandler::new(CommandContext::default());
        let parser = SlashCommandParser::new();
        
        let parsed = parser.parse("/nonexistent").unwrap();
        let response = handler.execute(&parsed).await;
        
        assert!(!response.success);
        assert!(response.message.contains("Unknown command"));
    }
}