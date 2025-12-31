//! Shell type detection and configuration path discovery
//!
//! Detects the user's shell type from environment and maps to config file paths.

use std::path::PathBuf;

/// Supported shell types for tips functionality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipsShellType {
    Zsh,
    Bash,
    Fish,
    Sh,
}

impl TipsShellType {
    /// Detect shell type from environment
    pub fn detect() -> Option<Self> {
        // Check SHELL environment variable
        if let Ok(shell) = std::env::var("SHELL") {
            return Self::from_shell_path(&shell);
        }

        // Fallback: check common environment hints
        if std::env::var("ZSH_VERSION").is_ok() {
            return Some(Self::Zsh);
        }
        if std::env::var("BASH_VERSION").is_ok() {
            return Some(Self::Bash);
        }
        if std::env::var("FISH_VERSION").is_ok() {
            return Some(Self::Fish);
        }

        None
    }

    /// Parse shell type from shell binary path
    pub fn from_shell_path(path: &str) -> Option<Self> {
        let shell_name = path.rsplit('/').next().unwrap_or(path);
        match shell_name {
            "zsh" => Some(Self::Zsh),
            "bash" => Some(Self::Bash),
            "fish" => Some(Self::Fish),
            "sh" | "dash" | "ash" => Some(Self::Sh),
            _ => None,
        }
    }

    /// Get the primary config file path for this shell
    pub fn primary_config_path(&self) -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        match self {
            Self::Zsh => Some(home.join(".zshrc")),
            Self::Bash => Some(home.join(".bashrc")),
            Self::Fish => Some(home.join(".config/fish/config.fish")),
            Self::Sh => Some(home.join(".profile")),
        }
    }

    /// Get all config file paths for this shell (in load order)
    pub fn config_paths(&self) -> Vec<PathBuf> {
        let Some(home) = dirs::home_dir() else {
            return Vec::new();
        };

        match self {
            Self::Zsh => vec![
                home.join(".zshenv"),
                home.join(".zprofile"),
                home.join(".zshrc"),
                home.join(".zlogin"),
            ],
            Self::Bash => vec![
                home.join(".bash_profile"),
                home.join(".bashrc"),
                home.join(".bash_login"),
                home.join(".profile"),
            ],
            Self::Fish => vec![
                home.join(".config/fish/config.fish"),
                home.join(".config/fish/conf.d"),
            ],
            Self::Sh => vec![home.join(".profile")],
        }
    }

    /// Get the shell name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Self::Zsh => "zsh",
            Self::Bash => "bash",
            Self::Fish => "fish",
            Self::Sh => "sh",
        }
    }

    /// Check if this shell supports the `alias` command syntax
    pub fn supports_alias_command(&self) -> bool {
        matches!(self, Self::Zsh | Self::Bash | Self::Sh)
    }
}

impl std::fmt::Display for TipsShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Comprehensive shell environment detection
#[derive(Debug, Clone)]
pub struct ShellEnvironment {
    /// Detected shell type
    pub shell_type: TipsShellType,

    /// Path to the shell binary
    pub shell_path: PathBuf,

    /// All relevant config file paths
    pub config_paths: Vec<PathBuf>,

    /// Whether this is an interactive shell
    pub is_interactive: bool,

    /// Whether this is a login shell
    pub is_login_shell: bool,
}

impl ShellEnvironment {
    /// Detect the current shell environment
    pub fn detect() -> Option<Self> {
        let shell_type = TipsShellType::detect()?;
        let shell_path = std::env::var("SHELL")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("/bin/{}", shell_type.name())));

        let config_paths = shell_type
            .config_paths()
            .into_iter()
            .filter(|p| p.exists())
            .collect();

        // Detect shell mode from environment hints
        let is_interactive = std::env::var("PS1").is_ok() || std::env::var("PROMPT").is_ok();
        let is_login_shell = std::env::var("LOGIN_SHELL").is_ok()
            || std::env::args()
                .next()
                .map(|arg| arg.starts_with('-'))
                .unwrap_or(false);

        Some(Self {
            shell_type,
            shell_path,
            config_paths,
            is_interactive,
            is_login_shell,
        })
    }

    /// Get the primary config file path
    pub fn primary_config(&self) -> Option<&PathBuf> {
        self.config_paths.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type_from_path() {
        assert_eq!(
            TipsShellType::from_shell_path("/bin/zsh"),
            Some(TipsShellType::Zsh)
        );
        assert_eq!(
            TipsShellType::from_shell_path("/usr/bin/bash"),
            Some(TipsShellType::Bash)
        );
        assert_eq!(
            TipsShellType::from_shell_path("/usr/local/bin/fish"),
            Some(TipsShellType::Fish)
        );
        assert_eq!(
            TipsShellType::from_shell_path("/bin/sh"),
            Some(TipsShellType::Sh)
        );
        assert_eq!(TipsShellType::from_shell_path("/bin/unknown"), None);
    }

    #[test]
    fn test_shell_type_name() {
        assert_eq!(TipsShellType::Zsh.name(), "zsh");
        assert_eq!(TipsShellType::Bash.name(), "bash");
        assert_eq!(TipsShellType::Fish.name(), "fish");
        assert_eq!(TipsShellType::Sh.name(), "sh");
    }

    #[test]
    fn test_config_paths_not_empty() {
        // Should return paths even if they don't exist
        let paths = TipsShellType::Zsh.config_paths();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_supports_alias() {
        assert!(TipsShellType::Zsh.supports_alias_command());
        assert!(TipsShellType::Bash.supports_alias_command());
        assert!(!TipsShellType::Fish.supports_alias_command());
    }
}
