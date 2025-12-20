//! Setup wizard module for first-time configuration
//!
//! Provides an interactive setup wizard similar to Claude Code that runs on first launch
//! or when explicitly invoked via `caro init`.

use crate::config::ConfigManager;
use crate::models::{LogLevel, SafetyLevel, ShellType, UserConfiguration};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use std::io::{self, IsTerminal, Write};

/// Current version of caro (from Cargo.toml)
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Theme configuration for terminal display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    DarkColorblind,
    LightColorblind,
    DarkAnsi,
    LightAnsi,
}

impl Theme {
    /// Get display name for the theme
    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark mode",
            Theme::Light => "Light mode",
            Theme::DarkColorblind => "Dark mode (colorblind-friendly)",
            Theme::LightColorblind => "Light mode (colorblind-friendly)",
            Theme::DarkAnsi => "Dark mode (ANSI colors only)",
            Theme::LightAnsi => "Light mode (ANSI colors only)",
        }
    }

    /// Get all themes as a list
    pub fn all() -> Vec<Theme> {
        vec![
            Theme::Dark,
            Theme::Light,
            Theme::DarkColorblind,
            Theme::LightColorblind,
            Theme::DarkAnsi,
            Theme::LightAnsi,
        ]
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// ASCII art banner for the setup wizard
const BANNER: &str = r#"
     *                                       █████▓▓░
                                 *         ███▓░     ░░
            ░░░░░░                        ███▓░
    ░░░   ░░░░░░░░░░                      ███▓░
   ░░░░░░░░░░░░░░░░░░░    *                ██▓░░      ▓
                                             ░▓▓███▓▓░
 *                                 ░░░░
                                 ░░░░░░░░
                               ░░░░░░░░░░░░░░░░
       ██████████                                       *
      ██▄██████▄██                        *
       ██████████      *
…………………█ █   █ █………………………………………………………………………………………………………………
"#;

/// Alternative minimal banner
const BANNER_MINIMAL: &str = r#"
   ██████╗ █████╗ ██████╗  ██████╗
  ██╔════╝██╔══██╗██╔══██╗██╔═══██╗
  ██║     ███████║██████╔╝██║   ██║
  ██║     ██╔══██║██╔══██╗██║   ██║
  ╚██████╗██║  ██║██║  ██║╚██████╔╝
   ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝
"#;

/// Code diff preview for theme selection
const DIFF_PREVIEW: &str = r#"╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
 1  fn main() {
 2 -    println!("Hello, World!");
 2 +    println!("Hello, Caro!");
 3  }
╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌"#;

/// Setup wizard errors
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),

    #[error("Dialog cancelled by user")]
    Cancelled,

    #[error("Not running in a terminal")]
    NotATty,
}

/// Result of the setup wizard
#[derive(Debug)]
pub struct SetupResult {
    pub configuration: UserConfiguration,
    pub theme: Theme,
    pub completed: bool,
}

/// Interactive setup wizard
pub struct SetupWizard {
    config_manager: ConfigManager,
    use_minimal_banner: bool,
}

impl SetupWizard {
    /// Create a new setup wizard
    pub fn new() -> Result<Self, SetupError> {
        let config_manager = ConfigManager::new()?;
        Ok(Self {
            config_manager,
            use_minimal_banner: false,
        })
    }

    /// Create a setup wizard with a custom config path
    pub fn with_config_path(path: std::path::PathBuf) -> Result<Self, SetupError> {
        let config_manager = ConfigManager::with_config_path(path)?;
        Ok(Self {
            config_manager,
            use_minimal_banner: false,
        })
    }

    /// Use minimal banner (for smaller terminals)
    pub fn use_minimal_banner(mut self, minimal: bool) -> Self {
        self.use_minimal_banner = minimal;
        self
    }

    /// Check if setup is needed (config doesn't exist)
    pub fn needs_setup(&self) -> bool {
        !self.config_manager.config_path().exists()
    }

    /// Run the interactive setup wizard
    pub fn run(&self) -> Result<SetupResult, SetupError> {
        // Check if we're in a terminal
        if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
            return Err(SetupError::NotATty);
        }

        // Clear screen and show welcome header
        self.clear_screen();
        self.print_welcome_header();
        self.print_banner();

        println!();
        println!(" {}", "Let's get started.".bold());
        println!();

        // Step 1: Theme selection
        let theme = self.select_theme()?;

        // Step 2: Shell selection
        let shell = self.select_shell()?;

        // Step 3: Safety level selection
        let safety_level = self.select_safety_level()?;

        // Step 4: Log level selection
        let log_level = self.select_log_level()?;

        // Step 5: Confirm and save
        let configuration = UserConfiguration {
            default_shell: shell,
            safety_level,
            default_model: None,
            log_level,
            cache_max_size_gb: 10,
            log_rotation_days: 7,
        };

        self.print_summary(&configuration, theme);

        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Save this configuration?")
            .default(true)
            .interact()
            .map_err(|_| SetupError::Cancelled)?;

        if !confirmed {
            println!();
            println!("{}", "Setup cancelled. You can run 'caro init' to try again.".yellow());
            return Ok(SetupResult {
                configuration: UserConfiguration::default(),
                theme: Theme::default(),
                completed: false,
            });
        }

        // Save configuration
        self.config_manager.save(&configuration)?;

        println!();
        println!("{}", "✓ Configuration saved!".green().bold());
        println!(
            "  Config file: {}",
            self.config_manager.config_path().display().to_string().dimmed()
        );

        // Print security notes
        self.print_security_notes();

        println!();
        println!("{}", "You're all set! Try running:".bold());
        println!("  {} \"list all files in current directory\"", "caro".bright_cyan());
        println!();
        println!(
            "{}",
            "To change settings later, run 'caro init' again.".dimmed()
        );

        Ok(SetupResult {
            configuration,
            theme,
            completed: true,
        })
    }

    /// Clear the terminal screen
    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();
    }

    /// Print the welcome header with version
    fn print_welcome_header(&self) {
        println!(
            "{} {}",
            "Welcome to Caro".bold(),
            format!("v{}", VERSION).dimmed()
        );
        println!("{}", "…………………………………………………………………………………………………………………………………………………………".dimmed());
        println!();
    }

    /// Print the welcome banner
    fn print_banner(&self) {
        let banner = if self.use_minimal_banner {
            BANNER_MINIMAL
        } else {
            BANNER
        };

        for line in banner.lines() {
            println!("{}", line.bright_cyan());
        }
    }

    /// Print security notes after setup completion
    fn print_security_notes(&self) {
        println!();
        println!("{}", "━".repeat(60).dimmed());
        println!();
        println!(" {}", "Security notes:".bold().yellow());
        println!();
        println!("  {}", "Caro uses AI to generate shell commands".bold());
        println!(
            "  {}",
            "You should always review commands before executing them,"
        );
        println!(
            "  {}",
            "especially those that modify files or system settings."
        );
        println!();
        println!(
            "  {}",
            "Due to prompt injection risks, only use it with code you trust."
        );
        println!("  For more details see:");
        println!(
            "  {}",
            "https://caro.sh/docs/security".bright_cyan().underline()
        );
        println!();
        println!("{}", "━".repeat(60).dimmed());
    }

    /// Select terminal theme
    fn select_theme(&self) -> Result<Theme, SetupError> {
        println!(
            " {}",
            "Choose the text style that looks best with your terminal".bold()
        );
        println!(" {}", "To change this later, run 'caro init'".dimmed());
        println!();

        let themes = Theme::all();
        let theme_names: Vec<String> = themes
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, t.display_name()))
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&theme_names)
            .default(0)
            .interact()
            .map_err(|_| SetupError::Cancelled)?;

        let selected_theme = themes[selection];

        // Show diff preview with selected theme
        println!();
        self.show_diff_preview(selected_theme);
        println!(
            " {} {}",
            "Selected theme:".dimmed(),
            selected_theme.display_name().bright_cyan()
        );
        println!();

        Ok(selected_theme)
    }

    /// Show diff preview with theme colors
    fn show_diff_preview(&self, _theme: Theme) {
        // Print the diff preview with appropriate colors
        for line in DIFF_PREVIEW.lines() {
            if line.starts_with(" 2 -") {
                println!("{}", line.red());
            } else if line.starts_with(" 2 +") {
                println!("{}", line.green());
            } else if line.starts_with("╌") {
                println!("{}", line.dimmed());
            } else {
                println!("{}", line);
            }
        }
    }

    /// Select default shell
    fn select_shell(&self) -> Result<Option<ShellType>, SetupError> {
        let detected = ShellType::detect();
        let detected_name = match detected {
            ShellType::Unknown => "Unknown",
            _ => match detected {
                ShellType::Bash => "Bash",
                ShellType::Zsh => "Zsh",
                ShellType::Fish => "Fish",
                ShellType::Sh => "POSIX sh",
                ShellType::PowerShell => "PowerShell",
                ShellType::Cmd => "cmd.exe",
                ShellType::Unknown => "Unknown",
            },
        };

        println!(
            " {}",
            "Choose your default shell".bold()
        );
        println!(
            " {} {}",
            "Detected:".dimmed(),
            detected_name.bright_cyan()
        );
        println!();

        let shells = vec![
            format!("Auto-detect (currently: {})", detected_name),
            "Bash".to_string(),
            "Zsh".to_string(),
            "Fish".to_string(),
            "POSIX sh".to_string(),
            "PowerShell".to_string(),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&shells)
            .default(0)
            .interact()
            .map_err(|_| SetupError::Cancelled)?;

        let shell = match selection {
            0 => None, // Auto-detect
            1 => Some(ShellType::Bash),
            2 => Some(ShellType::Zsh),
            3 => Some(ShellType::Fish),
            4 => Some(ShellType::Sh),
            5 => Some(ShellType::PowerShell),
            _ => None,
        };

        println!();
        Ok(shell)
    }

    /// Select safety level
    fn select_safety_level(&self) -> Result<SafetyLevel, SetupError> {
        println!(
            " {}",
            "Choose your safety level for command validation".bold()
        );
        println!(
            " {}",
            "This determines how cautious Caro is with potentially dangerous commands.".dimmed()
        );
        println!();

        let levels = vec![
            format!(
                "{} - {} (recommended)",
                "Moderate".yellow(),
                "Blocks critical commands, confirms high-risk ones"
            ),
            format!(
                "{} - {}",
                "Strict".red(),
                "Blocks high/critical, confirms moderate-risk commands"
            ),
            format!(
                "{} - {}",
                "Permissive".green(),
                "Warns about dangerous commands but allows with confirmation"
            ),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&levels)
            .default(0)
            .interact()
            .map_err(|_| SetupError::Cancelled)?;

        let safety = match selection {
            0 => SafetyLevel::Moderate,
            1 => SafetyLevel::Strict,
            2 => SafetyLevel::Permissive,
            _ => SafetyLevel::Moderate,
        };

        println!();
        Ok(safety)
    }

    /// Select log level
    fn select_log_level(&self) -> Result<LogLevel, SetupError> {
        println!(
            " {}",
            "Choose your preferred log level".bold()
        );
        println!();

        let levels = vec![
            format!("{} - Normal operation (recommended)", "Info".bright_cyan()),
            format!("{} - Only warnings and errors", "Warn".yellow()),
            format!("{} - Detailed debugging information", "Debug".dimmed()),
            format!("{} - Only critical errors", "Error".red()),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&levels)
            .default(0)
            .interact()
            .map_err(|_| SetupError::Cancelled)?;

        let level = match selection {
            0 => LogLevel::Info,
            1 => LogLevel::Warn,
            2 => LogLevel::Debug,
            3 => LogLevel::Error,
            _ => LogLevel::Info,
        };

        println!();
        Ok(level)
    }

    /// Print configuration summary
    fn print_summary(&self, config: &UserConfiguration, theme: Theme) {
        println!();
        println!("{}", "━".repeat(60).dimmed());
        println!();
        println!(" {}", "Configuration Summary".bold());
        println!();

        println!(
            "   {} {}",
            "Theme:".dimmed(),
            theme.display_name().bright_cyan()
        );

        let shell_display = match &config.default_shell {
            Some(s) => format!("{}", s),
            None => "Auto-detect".to_string(),
        };
        println!(
            "   {} {}",
            "Default shell:".dimmed(),
            shell_display.bright_cyan()
        );

        println!(
            "   {} {}",
            "Safety level:".dimmed(),
            format!("{}", config.safety_level).bright_cyan()
        );

        println!(
            "   {} {}",
            "Log level:".dimmed(),
            format!("{}", config.log_level).bright_cyan()
        );

        println!(
            "   {} {} GB",
            "Cache size limit:".dimmed(),
            config.cache_max_size_gb.to_string().bright_cyan()
        );

        println!();
        println!("{}", "━".repeat(60).dimmed());
        println!();
    }
}

/// Run the setup wizard directly (convenience function)
pub fn run_setup() -> Result<SetupResult, SetupError> {
    SetupWizard::new()?.run()
}

/// Check if first-time setup is needed
pub fn needs_setup() -> bool {
    ConfigManager::new()
        .map(|cm| !cm.config_path().exists())
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_display_names() {
        assert_eq!(Theme::Dark.display_name(), "Dark mode");
        assert_eq!(Theme::Light.display_name(), "Light mode");
        assert_eq!(
            Theme::DarkColorblind.display_name(),
            "Dark mode (colorblind-friendly)"
        );
    }

    #[test]
    fn test_theme_all() {
        let themes = Theme::all();
        assert_eq!(themes.len(), 6);
    }

    #[test]
    fn test_needs_setup_with_missing_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        let wizard = SetupWizard::with_config_path(config_path).unwrap();
        assert!(wizard.needs_setup());
    }
}
