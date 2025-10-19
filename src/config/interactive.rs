//! Interactive configuration UI module
//!
//! Provides a full-screen terminal interface for managing cmdai configuration,
//! inspired by Atuin's advanced search interface with modern UX patterns.

use super::schema::ConfigurationState;
use crate::models::{LogLevel, SafetyLevel, ShellType};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Editor, Select};
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};
use thiserror::Error;

/// Interactive configuration errors
#[derive(Debug, Error)]
pub enum InteractiveConfigError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Dialog error: {0}")]
    DialogError(#[from] dialoguer::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Interactive configuration manager with full-screen UI
pub struct InteractiveConfigUI {
    theme: ColorfulTheme,
    current_config: ConfigurationState,
    original_config: ConfigurationState,
    changes_made: bool,
}

/// Configuration section for organized UI navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSection {
    General,
    Backends,
    History,
    Safety,
    Logging,
    Cache,
    UserInterface,
    Privacy,
    Advanced,
    Review,
    Exit,
}

impl fmt::Display for ConfigSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::General => write!(f, "🌟 General Settings"),
            Self::Backends => write!(f, "🚀 Backend Configuration"),
            Self::History => write!(f, "📚 Command History"),
            Self::Safety => write!(f, "🛡️  Safety & Validation"),
            Self::Logging => write!(f, "📝 Logging Settings"),
            Self::Cache => write!(f, "💾 Cache Settings"),
            Self::UserInterface => write!(f, "🎨 User Interface"),
            Self::Privacy => write!(f, "🔒 Privacy Settings"),
            Self::Advanced => write!(f, "⚙️  Advanced Options"),
            Self::Review => write!(f, "👀 Review Configuration"),
            Self::Exit => write!(f, "🚪 Save & Exit"),
        }
    }
}

/// Result of interactive configuration session
#[derive(Debug, Clone)]
pub struct ConfigResult {
    pub config: ConfigurationState,
    pub changes_made: bool,
    pub cancelled: bool,
}

impl InteractiveConfigUI {
    /// Create a new interactive configuration UI
    pub fn new(current_config: ConfigurationState) -> Self {
        Self {
            theme: ColorfulTheme::default(),
            original_config: current_config.clone(),
            current_config,
            changes_made: false,
        }
    }

    /// Run the interactive configuration workflow
    pub fn run(&mut self) -> Result<ConfigResult, InteractiveConfigError> {
        self.clear_screen();
        self.show_welcome_banner();

        loop {
            let section = self.show_main_menu()?;

            match section {
                ConfigSection::General => self.configure_general()?,
                ConfigSection::Backends
                | ConfigSection::History
                | ConfigSection::UserInterface
                | ConfigSection::Privacy => self.show_placeholder(section)?,
                ConfigSection::Safety => self.configure_safety()?,
                ConfigSection::Logging => self.configure_logging()?,
                ConfigSection::Cache => self.configure_cache()?,
                ConfigSection::Advanced => self.configure_advanced()?,
                ConfigSection::Review => self.show_configuration_review()?,
                ConfigSection::Exit => {
                    if self.changes_made {
                        if self.confirm_save_changes()? {
                            return Ok(ConfigResult {
                                config: self.current_config.clone(),
                                changes_made: true,
                                cancelled: false,
                            });
                        }
                    } else {
                        return Ok(ConfigResult {
                            config: self.current_config.clone(),
                            changes_made: false,
                            cancelled: false,
                        });
                    }
                }
            }
        }
    }

    /// Show placeholder for sections that are not yet interactive
    fn show_placeholder(&self, section: ConfigSection) -> Result<(), InteractiveConfigError> {
        self.clear_screen();
        self.show_section_header(&format!("{}", section));
        println!(
            "{}",
            "This section is coming soon. Configuration is managed via the TOML file for now."
                .dimmed()
        );
        self.press_any_key();
        Ok(())
    }

    /// Show the main configuration menu
    fn show_main_menu(&self) -> Result<ConfigSection, InteractiveConfigError> {
        self.clear_screen();
        self.show_header();

        let sections = vec![
            ConfigSection::General,
            ConfigSection::Safety,
            ConfigSection::Logging,
            ConfigSection::Cache,
            ConfigSection::Advanced,
            ConfigSection::Review,
            ConfigSection::Exit,
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select configuration section")
            .items(&sections)
            .default(0)
            .interact()?;

        Ok(sections[selection])
    }

    /// Configure general settings
    fn configure_general(&mut self) -> Result<(), InteractiveConfigError> {
        self.clear_screen();
        self.show_section_header("General Settings");

        // Default shell configuration
        let shells = vec![
            ("Auto-detect", None),
            ("Bash", Some(ShellType::Bash)),
            ("Zsh", Some(ShellType::Zsh)),
            ("Fish", Some(ShellType::Fish)),
            ("Sh", Some(ShellType::Sh)),
            ("PowerShell", Some(ShellType::PowerShell)),
            ("Cmd", Some(ShellType::Cmd)),
        ];

        let current_index = shells
            .iter()
            .position(|(_, shell)| *shell == self.current_config.default_shell)
            .unwrap_or(0);

        let shell_selection = Select::with_theme(&self.theme)
            .with_prompt("Default shell")
            .items(&shells.iter().map(|(name, _)| *name).collect::<Vec<_>>())
            .default(current_index)
            .interact()?;

        let new_shell = shells[shell_selection].1;
        if new_shell != self.current_config.default_shell {
            self.current_config.default_shell = new_shell;
            self.changes_made = true;
        }

        // Default model configuration
        if let Some(ref current_model) = self.current_config.default_model {
            println!(
                "\n{} {}",
                "Current default model:".cyan(),
                current_model.yellow()
            );
        } else {
            println!("\n{}", "No default model set (will auto-select)".dimmed());
        }

        if Confirm::with_theme(&self.theme)
            .with_prompt("Configure default model?")
            .default(false)
            .interact()?
        {
            let models = vec![
                "Auto-select best available",
                "Qwen2.5-Coder-3B-Instruct",
                "Qwen2.5-Coder-7B-Instruct",
                "CodeLlama-7B-Instruct",
                "Custom model (enter manually)",
            ];

            let model_selection = Select::with_theme(&self.theme)
                .with_prompt("Select default model")
                .items(&models)
                .default(0)
                .interact()?;

            let new_model = match model_selection {
                0 => None,
                1 => Some("Qwen/Qwen2.5-Coder-3B-Instruct".to_string()),
                2 => Some("Qwen/Qwen2.5-Coder-7B-Instruct".to_string()),
                3 => Some("codellama/CodeLlama-7b-Instruct-hf".to_string()),
                4 => {
                    if let Some(editor_result) = Editor::new()
                        .extension("txt")
                        .edit("# Enter model ID (e.g., Qwen/Qwen2.5-Coder-3B-Instruct)\n")?
                    {
                        Some(editor_result.trim().to_string())
                    } else {
                        self.current_config.default_model.clone()
                    }
                }
                _ => unreachable!(),
            };

            if new_model != self.current_config.default_model {
                self.current_config.default_model = new_model;
                self.changes_made = true;
            }
        }

        self.press_any_key();
        Ok(())
    }

    /// Configure safety settings
    fn configure_safety(&mut self) -> Result<(), InteractiveConfigError> {
        self.clear_screen();
        self.show_section_header("Safety & Validation");

        let safety_levels = vec![
            (
                "Strict",
                SafetyLevel::Strict,
                "🔒 Blocks High & Critical, confirms Moderate",
            ),
            (
                "Moderate",
                SafetyLevel::Moderate,
                "⚖️  Blocks Critical, confirms High",
            ),
            (
                "Permissive",
                SafetyLevel::Permissive,
                "⚠️  Warns about dangerous commands",
            ),
        ];

        println!("{}", "Safety Level Configuration".bold().underline());
        println!();

        for (name, level, description) in &safety_levels {
            let marker = if *level == self.current_config.safety_level {
                "●".green()
            } else {
                "○".dimmed()
            };
            println!("  {} {} - {}", marker, name.bold(), description);
        }
        println!();

        let current_index = safety_levels
            .iter()
            .position(|(_, level, _)| *level == self.current_config.safety_level)
            .unwrap_or(1);

        let safety_selection = Select::with_theme(&self.theme)
            .with_prompt("Select safety level")
            .items(
                &safety_levels
                    .iter()
                    .map(|(name, _, desc)| format!("{} - {}", name, desc))
                    .collect::<Vec<_>>(),
            )
            .default(current_index)
            .interact()?;

        let new_safety_level = safety_levels[safety_selection].1;
        if new_safety_level != self.current_config.safety_level {
            self.current_config.safety_level = new_safety_level;
            self.changes_made = true;
        }

        self.press_any_key();
        Ok(())
    }

    /// Configure logging settings
    fn configure_logging(&mut self) -> Result<(), InteractiveConfigError> {
        self.clear_screen();
        self.show_section_header("Logging & Debugging");

        let log_levels = vec![
            ("Debug", LogLevel::Debug, "🔍 Verbose debugging information"),
            ("Info", LogLevel::Info, "ℹ️  General information messages"),
            ("Warn", LogLevel::Warn, "⚠️  Warning messages only"),
            ("Error", LogLevel::Error, "❌ Error messages only"),
        ];

        let current_index = log_levels
            .iter()
            .position(|(_, level, _)| *level == self.current_config.log_level)
            .unwrap_or(1);

        let log_selection = Select::with_theme(&self.theme)
            .with_prompt("Log level")
            .items(
                &log_levels
                    .iter()
                    .map(|(name, _, desc)| format!("{} - {}", name, desc))
                    .collect::<Vec<_>>(),
            )
            .default(current_index)
            .interact()?;

        let new_log_level = log_levels[log_selection].1;
        if new_log_level != self.current_config.log_level {
            self.current_config.log_level = new_log_level;
            self.changes_made = true;
        }

        // Log rotation configuration
        println!(
            "\n{} {} days",
            "Current log rotation:".cyan(),
            self.current_config.log_rotation_days.to_string().yellow()
        );

        if Confirm::with_theme(&self.theme)
            .with_prompt("Change log rotation period?")
            .default(false)
            .interact()?
        {
            let rotation_options = vec![
                ("1 day", 1),
                ("3 days", 3),
                ("7 days", 7),
                ("14 days", 14),
                ("30 days", 30),
                ("90 days", 90),
                ("Never (365 days)", 365),
            ];

            let current_index = rotation_options
                .iter()
                .position(|(_, days)| *days == self.current_config.log_rotation_days)
                .unwrap_or(2);

            let rotation_selection = Select::with_theme(&self.theme)
                .with_prompt("Log rotation period")
                .items(
                    &rotation_options
                        .iter()
                        .map(|(name, _)| *name)
                        .collect::<Vec<_>>(),
                )
                .default(current_index)
                .interact()?;

            let new_rotation = rotation_options[rotation_selection].1;
            if new_rotation != self.current_config.log_rotation_days {
                self.current_config.log_rotation_days = new_rotation;
                self.changes_made = true;
            }
        }

        self.press_any_key();
        Ok(())
    }

    /// Configure cache settings
    fn configure_cache(&mut self) -> Result<(), InteractiveConfigError> {
        self.clear_screen();
        self.show_section_header("Cache Management");

        let cache_sizes = vec![
            ("1 GB", 1),
            ("5 GB", 5),
            ("10 GB", 10),
            ("20 GB", 20),
            ("50 GB", 50),
            ("100 GB", 100),
        ];

        println!(
            "{} {} GB",
            "Current cache limit:".cyan(),
            self.current_config.cache_max_size_gb.to_string().yellow()
        );
        println!();

        let current_index = cache_sizes
            .iter()
            .position(|(_, size)| *size == self.current_config.cache_max_size_gb)
            .unwrap_or(2);

        let cache_selection = Select::with_theme(&self.theme)
            .with_prompt("Maximum cache size")
            .items(
                &cache_sizes
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<_>>(),
            )
            .default(current_index)
            .interact()?;

        let new_cache_size = cache_sizes[cache_selection].1;
        if new_cache_size != self.current_config.cache_max_size_gb {
            self.current_config.cache_max_size_gb = new_cache_size;
            self.changes_made = true;
        }

        // Show cache statistics info
        println!("\n{}", "Cache Information".bold().underline());
        println!("• Models are cached in user cache directory");
        println!("• LRU (Least Recently Used) cleanup automatically manages space");
        println!("• Larger cache allows more models to be kept locally");
        println!("• Cache integrity is verified on startup");

        self.press_any_key();
        Ok(())
    }

    /// Configure advanced options
    fn configure_advanced(&mut self) -> Result<(), InteractiveConfigError> {
        self.clear_screen();
        self.show_section_header("Advanced Options");

        println!("{}", "Future Feature Previews".bold().underline());
        println!();
        println!("• 🔮 Semantic command understanding (Phase 2)");
        println!("• 📚 Rich command history with search (Phase 2)");
        println!("• 🎯 Goal-oriented multi-step planning (Phase 3)");
        println!("• 🔗 Deep shell integration (Phase 3)");
        println!("• 🔌 Plugin system (Phase 4)");
        println!();
        println!(
            "{}",
            "These features will be configurable in future releases.".dimmed()
        );

        self.press_any_key();
        Ok(())
    }

    /// Show configuration review
    fn show_configuration_review(&self) -> Result<(), InteractiveConfigError> {
        self.clear_screen();
        self.show_section_header("Configuration Review");

        let mut config_display = HashMap::new();

        config_display.insert(
            "General".to_string(),
            vec![
                (
                    "Default Shell",
                    match &self.current_config.default_shell {
                        Some(shell) => shell.to_string(),
                        None => "Auto-detect".to_string(),
                    },
                ),
                (
                    "Default Model",
                    match &self.current_config.default_model {
                        Some(model) => model.clone(),
                        None => "Auto-select".to_string(),
                    },
                ),
            ],
        );

        config_display.insert(
            "Safety".to_string(),
            vec![("Safety Level", self.current_config.safety_level.to_string())],
        );

        config_display.insert(
            "Logging".to_string(),
            vec![
                ("Log Level", self.current_config.log_level.to_string()),
                (
                    "Log Rotation",
                    format!("{} days", self.current_config.log_rotation_days),
                ),
            ],
        );

        config_display.insert(
            "Cache".to_string(),
            vec![(
                "Max Size",
                format!("{} GB", self.current_config.cache_max_size_gb),
            )],
        );

        for (section, items) in &config_display {
            println!("{}", section.bold().cyan());
            for (key, value) in items {
                println!("  {} {}", format!("{}:", key).dimmed(), value.yellow());
            }
            println!();
        }

        if self.changes_made {
            println!("{}", "✨ Changes made - remember to save!".green().bold());
        } else {
            println!("{}", "No changes made.".dimmed());
        }

        self.press_any_key();
        Ok(())
    }

    /// Confirm saving changes
    fn confirm_save_changes(&self) -> Result<bool, InteractiveConfigError> {
        println!("\n{}", "Save Changes?".bold().green());
        println!("Your configuration changes will be written to disk.");

        Ok(Confirm::with_theme(&self.theme)
            .with_prompt("Save configuration changes?")
            .default(true)
            .interact()?)
    }

    /// Clear the terminal screen
    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }

    /// Show welcome banner
    fn show_welcome_banner(&self) {
        println!(
            "{}",
            "╭─────────────────────────────────────────────────────────────╮".cyan()
        );
        println!(
            "{}",
            "│                                                             │".cyan()
        );
        println!(
            "{}",
            "│                    ✨ cmdai Configuration ✨                │".cyan()
        );
        println!(
            "{}",
            "│                                                             │".cyan()
        );
        println!(
            "{}",
            "│        Configure your AI-powered command generation         │".cyan()
        );
        println!(
            "{}",
            "│                                                             │".cyan()
        );
        println!(
            "{}",
            "╰─────────────────────────────────────────────────────────────╯".cyan()
        );
        println!();
    }

    /// Show header for current configuration state
    fn show_header(&self) {
        println!(
            "{} {}",
            "cmdai".bold().cyan(),
            "Configuration Manager".bold()
        );

        if self.changes_made {
            println!(
                "{} {}",
                "Status:".dimmed(),
                "Modified (unsaved changes)".yellow()
            );
        } else {
            println!("{} {}", "Status:".dimmed(), "Clean".green());
        }
        println!();
    }

    /// Show section header
    fn show_section_header(&self, section_name: &str) {
        println!("{}", format!("━━━ {} ━━━", section_name).bold().cyan());
        println!();
    }

    /// Wait for user to press any key
    fn press_any_key(&self) {
        println!("\n{}", "Press Enter to continue...".dimmed());
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }
}

/// Convenience function to run interactive configuration
pub fn run_interactive_config(
    current_config: ConfigurationState,
) -> Result<ConfigResult, InteractiveConfigError> {
    let mut ui = InteractiveConfigUI::new(current_config);
    ui.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_section_display() {
        assert_eq!(ConfigSection::General.to_string(), "🌟 General Settings");
        assert_eq!(ConfigSection::Safety.to_string(), "🛡️  Safety & Validation");
        assert_eq!(ConfigSection::Exit.to_string(), "🚪 Save & Exit");
    }

    #[test]
    fn test_interactive_ui_creation() {
        let config = ConfigurationState::default();
        let ui = InteractiveConfigUI::new(config.clone());
        assert_eq!(ui.current_config, config);
        assert!(!ui.changes_made);
    }

    #[test]
    fn test_config_result() {
        let config = ConfigurationState::default();
        let result = ConfigResult {
            config: config.clone(),
            changes_made: true,
            cancelled: false,
        };
        assert_eq!(result.config, config);
        assert!(result.changes_made);
        assert!(!result.cancelled);
    }
}
