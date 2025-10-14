//! Interactive configuration UI module with dialoguer integration
//!
//! Provides a comprehensive terminal interface for managing cmdai configuration
//! with support for all backend settings, history management, and safety options.

use super::schema::{
    BackendConfig, ConfigurationState, PrivacyLevel, RetentionPolicy, VerbosityLevel,
    ValidationRules,
};
use crate::models::{BackendType, RiskLevel, SafetyLevel};
use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};

/// Interactive configuration manager with comprehensive settings
pub struct InteractiveConfigUI {
    theme: ColorfulTheme,
    config: ConfigurationState,
    original_config: ConfigurationState,
    changes_made: bool,
}

/// Configuration sections for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigSection {
    Backends,
    History,
    Safety,
    UserInterface,
    Privacy,
    Advanced,
    Review,
    Exit,
}

impl fmt::Display for ConfigSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backends => write!(f, "🚀 Backend Configuration"),
            Self::History => write!(f, "📚 Command History"),
            Self::Safety => write!(f, "🛡️  Safety & Validation"),
            Self::UserInterface => write!(f, "🎨 User Interface"),
            Self::Privacy => write!(f, "🔒 Privacy Settings"),
            Self::Advanced => write!(f, "⚙️  Advanced Options"),
            Self::Review => write!(f, "👀 Review Changes"),
            Self::Exit => write!(f, "🚪 Save & Exit"),
        }
    }
}

/// Result of configuration session
#[derive(Debug, Clone)]
pub struct ConfigResult {
    pub config: ConfigurationState,
    pub changes_made: bool,
}

impl InteractiveConfigUI {
    /// Create a new interactive configuration UI
    pub fn new(config: ConfigurationState) -> Self {
        Self {
            theme: ColorfulTheme::default(),
            original_config: config.clone(),
            config,
            changes_made: false,
        }
    }

    /// Run the interactive configuration session
    pub async fn run(mut self) -> Result<ConfigResult> {
        loop {
            self.clear_screen();
            self.show_header();

            let sections = vec![
                ConfigSection::Backends,
                ConfigSection::History,
                ConfigSection::Safety,
                ConfigSection::UserInterface,
                ConfigSection::Privacy,
                ConfigSection::Advanced,
                ConfigSection::Review,
                ConfigSection::Exit,
            ];

            let selection = Select::with_theme(&self.theme)
                .with_prompt("Select configuration section")
                .items(&sections)
                .default(0)
                .interact()?;

            match sections[selection] {
                ConfigSection::Backends => self.configure_backends()?,
                ConfigSection::History => self.configure_history()?,
                ConfigSection::Safety => self.configure_safety()?,
                ConfigSection::UserInterface => self.configure_ui()?,
                ConfigSection::Privacy => self.configure_privacy()?,
                ConfigSection::Advanced => self.configure_advanced()?,
                ConfigSection::Review => self.review_changes()?,
                ConfigSection::Exit => {
                    if self.confirm_exit()? {
                        break;
                    }
                }
            }
        }

        Ok(ConfigResult {
            config: self.config,
            changes_made: self.changes_made,
        })
    }

    /// Configure backend settings
    fn configure_backends(&mut self) -> Result<()> {
        self.clear_screen();
        println!("{}", "Backend Configuration".bold().cyan());
        println!();

        // Select preferred backend
        let backend_types = vec![
            BackendType::Auto,
            BackendType::MLX,
            BackendType::Mock,
            BackendType::Ollama,
            BackendType::VLlm,
        ];

        let current_idx = backend_types
            .iter()
            .position(|&b| b == self.config.preferred_backend)
            .unwrap_or(0);

        let selected = Select::with_theme(&self.theme)
            .with_prompt("Select preferred backend")
            .items(&backend_types)
            .default(current_idx)
            .interact()?;

        if backend_types[selected] != self.config.preferred_backend {
            self.config.preferred_backend = backend_types[selected];
            self.changes_made = true;
        }

        // Configure specific backend
        if Confirm::with_theme(&self.theme)
            .with_prompt("Configure backend-specific settings?")
            .default(false)
            .interact()?
        {
            self.configure_backend_details()?;
        }

        // Configure fallback chain
        if Confirm::with_theme(&self.theme)
            .with_prompt("Configure fallback chain?")
            .default(false)
            .interact()?
        {
            let available_backends = vec![
                BackendType::MLX,
                BackendType::Mock,
                BackendType::Ollama,
                BackendType::VLlm,
            ];

            let selected_indices = MultiSelect::with_theme(&self.theme)
                .with_prompt("Select fallback backends (in order)")
                .items(&available_backends)
                .interact()?;

            let new_chain: Vec<BackendType> = selected_indices
                .iter()
                .map(|&i| available_backends[i])
                .collect();

            if new_chain != self.config.fallback_chain {
                self.config.fallback_chain = new_chain;
                self.changes_made = true;
            }
        }

        Ok(())
    }

    /// Configure backend-specific details
    fn configure_backend_details(&mut self) -> Result<()> {
        let backend_name = Select::with_theme(&self.theme)
            .with_prompt("Select backend to configure")
            .items(&["mlx", "ollama", "vllm", "mock"])
            .interact()?;

        let backend_key = ["mlx", "ollama", "vllm", "mock"][backend_name];

        let mut config = self
            .config
            .backend_configs
            .get(backend_key)
            .cloned()
            .unwrap_or_default();

        // Enable/disable backend
        config.enabled = Confirm::with_theme(&self.theme)
            .with_prompt(format!("Enable {} backend?", backend_key))
            .default(config.enabled)
            .interact()?;

        // Configure endpoint if applicable
        if backend_key == "ollama" || backend_key == "vllm" {
            let current_endpoint = config.endpoint.clone().unwrap_or_else(|| {
                if backend_key == "ollama" {
                    "http://localhost:11434".to_string()
                } else {
                    "http://localhost:8000".to_string()
                }
            });

            let endpoint: String = Input::with_theme(&self.theme)
                .with_prompt("API endpoint")
                .default(current_endpoint)
                .interact()?;

            config.endpoint = Some(endpoint);
        }

        // Configure model name
        if let Some(current_model) = &config.model_name {
            let model: String = Input::with_theme(&self.theme)
                .with_prompt("Model name")
                .default(current_model.clone())
                .allow_empty(true)
                .interact()?;

            if !model.is_empty() {
                config.model_name = Some(model);
            }
        } else if Confirm::with_theme(&self.theme)
            .with_prompt("Specify model name?")
            .default(false)
            .interact()?
        {
            let model: String = Input::with_theme(&self.theme)
                .with_prompt("Model name")
                .interact()?;
            config.model_name = Some(model);
        }

        // Configure timeout
        config.timeout_seconds = Input::with_theme(&self.theme)
            .with_prompt("Timeout (seconds)")
            .default(config.timeout_seconds)
            .validate_with(|input: &u32| {
                if *input > 0 && *input <= 300 {
                    Ok(())
                } else {
                    Err("Timeout must be between 1 and 300 seconds")
                }
            })
            .interact()?;

        self.config
            .backend_configs
            .insert(backend_key.to_string(), config);
        self.changes_made = true;

        Ok(())
    }

    /// Configure command history settings
    fn configure_history(&mut self) -> Result<()> {
        self.clear_screen();
        println!("{}", "Command History Settings".bold().cyan());
        println!();

        // Enable/disable history
        self.config.history_enabled = Confirm::with_theme(&self.theme)
            .with_prompt("Enable command history?")
            .default(self.config.history_enabled)
            .interact()?;

        if self.config.history_enabled {
            // Configure retention policy
            println!("\n{}", "Retention Policy".bold());

            // Max entries
            if Confirm::with_theme(&self.theme)
                .with_prompt("Set maximum number of history entries?")
                .default(self.config.retention_policy.max_entries.is_some())
                .interact()?
            {
                let max_entries: usize = Input::with_theme(&self.theme)
                    .with_prompt("Maximum entries")
                    .default(
                        self.config
                            .retention_policy
                            .max_entries
                            .unwrap_or(10000),
                    )
                    .validate_with(|input: &usize| {
                        if *input > 0 && *input <= 1000000 {
                            Ok(())
                        } else {
                            Err("Must be between 1 and 1,000,000")
                        }
                    })
                    .interact()?;
                self.config.retention_policy.max_entries = Some(max_entries);
            } else {
                self.config.retention_policy.max_entries = None;
            }

            // Max age
            if Confirm::with_theme(&self.theme)
                .with_prompt("Set maximum age for entries?")
                .default(self.config.retention_policy.max_age_days.is_some())
                .interact()?
            {
                let max_age: u32 = Input::with_theme(&self.theme)
                    .with_prompt("Maximum age (days)")
                    .default(self.config.retention_policy.max_age_days.unwrap_or(90))
                    .validate_with(|input: &u32| {
                        if *input > 0 && *input <= 365 {
                            Ok(())
                        } else {
                            Err("Must be between 1 and 365 days")
                        }
                    })
                    .interact()?;
                self.config.retention_policy.max_age_days = Some(max_age);
            } else {
                self.config.retention_policy.max_age_days = None;
            }

            // Preserve favorites
            self.config.retention_policy.preserve_favorites = Confirm::with_theme(&self.theme)
                .with_prompt("Preserve favorite commands?")
                .default(self.config.retention_policy.preserve_favorites)
                .interact()?;

            // Preserve frequently used
            self.config.retention_policy.preserve_frequently_used =
                Confirm::with_theme(&self.theme)
                    .with_prompt("Preserve frequently used commands?")
                    .default(self.config.retention_policy.preserve_frequently_used)
                    .interact()?;

            // Auto cleanup days
            self.config.auto_cleanup_days = Input::with_theme(&self.theme)
                .with_prompt("Auto cleanup interval (days)")
                .default(self.config.auto_cleanup_days)
                .validate_with(|input: &u32| {
                    if *input > 0 && *input <= 365 {
                        Ok(())
                    } else {
                        Err("Must be between 1 and 365 days")
                    }
                })
                .interact()?;
        }

        self.changes_made = true;
        Ok(())
    }

    /// Configure safety settings
    fn configure_safety(&mut self) -> Result<()> {
        self.clear_screen();
        println!("{}", "Safety Configuration".bold().cyan());
        println!();

        // Safety level
        let safety_levels = vec![
            ("Strict", SafetyLevel::Strict, "Blocks High & Critical risks"),
            (
                "Moderate",
                SafetyLevel::Moderate,
                "Blocks Critical, confirms High",
            ),
            (
                "Permissive",
                SafetyLevel::Permissive,
                "Only warns about risks",
            ),
        ];

        let current_idx = safety_levels
            .iter()
            .position(|(_, level, _)| *level == self.config.safety_level)
            .unwrap_or(1);

        println!("{}", "Safety Level".bold());
        for (i, (name, level, desc)) in safety_levels.iter().enumerate() {
            let marker = if i == current_idx {
                "●".green()
            } else {
                "○".dimmed()
            };
            println!("  {} {} - {}", marker, name.bold(), desc.dimmed());
        }
        println!();

        let selected = Select::with_theme(&self.theme)
            .with_prompt("Select safety level")
            .items(&safety_levels.iter().map(|(n, _, _)| n).collect::<Vec<_>>())
            .default(current_idx)
            .interact()?;

        if safety_levels[selected].1 != self.config.safety_level {
            self.config.safety_level = safety_levels[selected].1;
            self.changes_made = true;
        }

        // Confirmation requirements
        println!("\n{}", "Confirmation Requirements".bold());
        let risk_levels = vec![
            RiskLevel::Safe,
            RiskLevel::Moderate,
            RiskLevel::High,
            RiskLevel::Critical,
        ];

        let current_confirmations = &self.config.confirmation_required;
        let defaults: Vec<bool> = risk_levels
            .iter()
            .map(|r| current_confirmations.contains(r))
            .collect();

        let new_indices = MultiSelect::with_theme(&self.theme)
            .with_prompt("Require confirmation for risk levels")
            .items(&risk_levels)
            .defaults(&defaults)
            .interact()?;

        let new_confirmations: Vec<RiskLevel> =
            new_indices.iter().map(|&i| risk_levels[i]).collect();

        if new_confirmations != self.config.confirmation_required {
            self.config.confirmation_required = new_confirmations;
            self.changes_made = true;
        }

        // Custom safety patterns
        if Confirm::with_theme(&self.theme)
            .with_prompt("Manage custom safety patterns?")
            .default(false)
            .interact()?
        {
            self.manage_custom_patterns()?;
        }

        Ok(())
    }

    /// Manage custom safety patterns
    fn manage_custom_patterns(&mut self) -> Result<()> {
        loop {
            println!("\n{}", "Custom Safety Patterns".bold());
            if self.config.custom_safety_patterns.is_empty() {
                println!("  No custom patterns configured");
            } else {
                for (i, pattern) in self.config.custom_safety_patterns.iter().enumerate() {
                    println!("  {}. {}", i + 1, pattern);
                }
            }
            println!();

            let options = vec!["Add pattern", "Remove pattern", "Clear all", "Done"];
            let choice = Select::with_theme(&self.theme)
                .with_prompt("Action")
                .items(&options)
                .default(3)
                .interact()?;

            match choice {
                0 => {
                    // Add pattern
                    let pattern: String = Input::with_theme(&self.theme)
                        .with_prompt("Enter regex pattern")
                        .validate_with(|input: &String| {
                            if regex::Regex::new(input).is_ok() {
                                Ok(())
                            } else {
                                Err("Invalid regex pattern")
                            }
                        })
                        .interact()?;
                    self.config.custom_safety_patterns.push(pattern);
                    self.changes_made = true;
                }
                1 => {
                    // Remove pattern
                    if !self.config.custom_safety_patterns.is_empty() {
                        let idx = Select::with_theme(&self.theme)
                            .with_prompt("Select pattern to remove")
                            .items(&self.config.custom_safety_patterns)
                            .interact()?;
                        self.config.custom_safety_patterns.remove(idx);
                        self.changes_made = true;
                    }
                }
                2 => {
                    // Clear all
                    if Confirm::with_theme(&self.theme)
                        .with_prompt("Clear all custom patterns?")
                        .default(false)
                        .interact()?
                    {
                        self.config.custom_safety_patterns.clear();
                        self.changes_made = true;
                    }
                }
                3 => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Configure user interface settings
    fn configure_ui(&mut self) -> Result<()> {
        self.clear_screen();
        println!("{}", "User Interface Settings".bold().cyan());
        println!();

        // Streaming
        self.config.streaming_enabled = Confirm::with_theme(&self.theme)
            .with_prompt("Enable streaming output?")
            .default(self.config.streaming_enabled)
            .interact()?;

        // Color output
        self.config.color_output = Confirm::with_theme(&self.theme)
            .with_prompt("Enable colored output?")
            .default(self.config.color_output)
            .interact()?;

        // Verbosity level
        let verbosity_levels = vec![
            VerbosityLevel::Quiet,
            VerbosityLevel::Normal,
            VerbosityLevel::Verbose,
            VerbosityLevel::Debug,
        ];

        let current_idx = verbosity_levels
            .iter()
            .position(|&v| v == self.config.verbosity_level)
            .unwrap_or(1);

        let selected = Select::with_theme(&self.theme)
            .with_prompt("Verbosity level")
            .items(&verbosity_levels)
            .default(current_idx)
            .interact()?;

        if verbosity_levels[selected] != self.config.verbosity_level {
            self.config.verbosity_level = verbosity_levels[selected];
            self.changes_made = true;
        }

        Ok(())
    }

    /// Configure privacy settings
    fn configure_privacy(&mut self) -> Result<()> {
        self.clear_screen();
        println!("{}", "Privacy Settings".bold().cyan());
        println!();

        let privacy_levels = vec![
            (
                "None",
                PrivacyLevel::None,
                "Store all commands including sensitive data",
            ),
            (
                "Basic",
                PrivacyLevel::Basic,
                "Filter obvious sensitive patterns",
            ),
            (
                "Strict",
                PrivacyLevel::Strict,
                "Aggressive filtering of potential sensitive data",
            ),
            (
                "Paranoid",
                PrivacyLevel::Paranoid,
                "Only store safe commands, block everything else",
            ),
        ];

        let current_idx = privacy_levels
            .iter()
            .position(|(_, level, _)| *level == self.config.privacy_mode)
            .unwrap_or(1);

        println!("{}", "Privacy Level".bold());
        for (i, (name, _, desc)) in privacy_levels.iter().enumerate() {
            let marker = if i == current_idx {
                "●".green()
            } else {
                "○".dimmed()
            };
            println!("  {} {} - {}", marker, name.bold(), desc.dimmed());
        }
        println!();

        let selected = Select::with_theme(&self.theme)
            .with_prompt("Select privacy level")
            .items(&privacy_levels.iter().map(|(n, _, _)| n).collect::<Vec<_>>())
            .default(current_idx)
            .interact()?;

        if privacy_levels[selected].1 != self.config.privacy_mode {
            self.config.privacy_mode = privacy_levels[selected].1;
            self.changes_made = true;
        }

        Ok(())
    }

    /// Configure advanced settings
    fn configure_advanced(&mut self) -> Result<()> {
        self.clear_screen();
        println!("{}", "Advanced Settings".bold().cyan());
        println!();

        println!("These are advanced settings. Modify with caution!");
        println!();

        if !Confirm::with_theme(&self.theme)
            .with_prompt("Continue with advanced configuration?")
            .default(false)
            .interact()?
        {
            return Ok(());
        }

        // Add any advanced settings here
        println!("Advanced settings configuration complete.");

        Ok(())
    }

    /// Review configuration changes
    fn review_changes(&self) -> Result<()> {
        self.clear_screen();
        println!("{}", "Configuration Review".bold().cyan());
        println!();

        if !self.changes_made {
            println!("No changes have been made.");
        } else {
            println!("{}", "Changes Summary:".bold());
            println!();

            // Display differences
            self.show_config_diff();
        }

        println!();
        println!("Press any key to continue...");
        let _ = std::io::stdin().read_line(&mut String::new());

        Ok(())
    }

    /// Show configuration differences
    fn show_config_diff(&self) {
        // Compare backends
        if self.config.preferred_backend != self.original_config.preferred_backend {
            println!(
                "  Preferred Backend: {} → {}",
                self.original_config.preferred_backend,
                self.config.preferred_backend.to_string().green()
            );
        }

        // Compare safety
        if self.config.safety_level != self.original_config.safety_level {
            println!(
                "  Safety Level: {:?} → {:?}",
                self.original_config.safety_level,
                format!("{:?}", self.config.safety_level).green()
            );
        }

        // Compare privacy
        if self.config.privacy_mode != self.original_config.privacy_mode {
            println!(
                "  Privacy Mode: {:?} → {:?}",
                self.original_config.privacy_mode,
                format!("{:?}", self.config.privacy_mode).green()
            );
        }

        // Compare history
        if self.config.history_enabled != self.original_config.history_enabled {
            println!(
                "  History: {} → {}",
                self.original_config.history_enabled,
                self.config.history_enabled.to_string().green()
            );
        }

        // Compare UI
        if self.config.color_output != self.original_config.color_output {
            println!(
                "  Color Output: {} → {}",
                self.original_config.color_output,
                self.config.color_output.to_string().green()
            );
        }

        if self.config.streaming_enabled != self.original_config.streaming_enabled {
            println!(
                "  Streaming: {} → {}",
                self.original_config.streaming_enabled,
                self.config.streaming_enabled.to_string().green()
            );
        }
    }

    /// Confirm exit
    fn confirm_exit(&self) -> Result<bool> {
        if self.changes_made {
            Confirm::with_theme(&self.theme)
                .with_prompt("Save changes and exit?")
                .default(true)
                .interact()
                .context("Failed to confirm exit")
        } else {
            Ok(true)
        }
    }

    /// Clear the terminal screen
    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();
    }

    /// Show configuration header
    fn show_header(&self) {
        println!("{}", "╔══════════════════════════════════════╗".cyan());
        println!(
            "{}{}{}",
            "║".cyan(),
            "     cmdai Configuration Manager      ".bold(),
            "║".cyan()
        );
        println!("{}", "╚══════════════════════════════════════╝".cyan());
        println!();

        if self.changes_made {
            println!("{}", "● Unsaved changes".yellow());
        }
        println!();
    }
}

impl Default for BackendConfig {
    fn default() -> Self {
        BackendConfig {
            enabled: true,
            endpoint: None,
            model_name: None,
            timeout_seconds: 30,
            max_retries: 2,
            additional_params: HashMap::new(),
        }
    }
}

/// Run interactive configuration with current settings
pub async fn run_interactive_config(
    current_config: ConfigurationState,
) -> Result<ConfigResult> {
    let ui = InteractiveConfigUI::new(current_config);
    ui.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_section_display() {
        assert_eq!(
            ConfigSection::Backends.to_string(),
            "🚀 Backend Configuration"
        );
        assert_eq!(ConfigSection::Safety.to_string(), "🛡️  Safety & Validation");
    }

    #[test]
    fn test_default_backend_config() {
        let config = BackendConfig::default();
        assert!(config.enabled);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 2);
    }
}