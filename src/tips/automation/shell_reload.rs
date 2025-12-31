//! Shell reload functionality
//!
//! Provides instructions and utilities for reloading shell configuration
//! after installation changes.

use crate::tips::shell::TipsShellType;
use std::path::Path;

/// Helper for shell reload operations
#[derive(Debug, Clone)]
pub struct ShellReload {
    /// The shell type
    shell: TipsShellType,
}

impl ShellReload {
    /// Create a new shell reload helper
    pub fn new(shell: TipsShellType) -> Self {
        Self { shell }
    }

    /// Get the command to reload the shell configuration
    pub fn reload_command(&self) -> &'static str {
        match self.shell {
            TipsShellType::Zsh => "source ~/.zshrc",
            TipsShellType::Bash => "source ~/.bashrc",
            TipsShellType::Fish => "source ~/.config/fish/config.fish",
            TipsShellType::Sh => ". ~/.profile",
        }
    }

    /// Get the command to reload a specific config file
    pub fn reload_file_command(&self, path: &Path) -> String {
        let path_str = path.display();
        match self.shell {
            TipsShellType::Fish => format!("source {}", path_str),
            TipsShellType::Sh => format!(". {}", path_str),
            _ => format!("source {}", path_str),
        }
    }

    /// Get a user-friendly message about reloading
    pub fn reload_message(&self) -> String {
        format!(
            "To apply changes, either:\n  1. Run: {}\n  2. Open a new terminal window",
            self.reload_command()
        )
    }

    /// Get the exec replacement command (completely replace current shell)
    pub fn exec_command(&self) -> String {
        match self.shell {
            TipsShellType::Zsh => "exec zsh".to_string(),
            TipsShellType::Bash => "exec bash".to_string(),
            TipsShellType::Fish => "exec fish".to_string(),
            TipsShellType::Sh => "exec sh".to_string(),
        }
    }

    /// Get the shell's main config file path
    pub fn config_file(&self) -> &'static str {
        match self.shell {
            TipsShellType::Zsh => "~/.zshrc",
            TipsShellType::Bash => "~/.bashrc",
            TipsShellType::Fish => "~/.config/fish/config.fish",
            TipsShellType::Sh => "~/.profile",
        }
    }

    /// Get all config files for this shell (in load order)
    pub fn config_files(&self) -> Vec<&'static str> {
        match self.shell {
            TipsShellType::Zsh => vec!["~/.zshenv", "~/.zprofile", "~/.zshrc", "~/.zlogin"],
            TipsShellType::Bash => vec![
                "~/.bash_profile",
                "~/.bashrc",
                "~/.bash_login",
                "~/.profile",
            ],
            TipsShellType::Fish => {
                vec!["~/.config/fish/config.fish", "~/.config/fish/conf.d/*.fish"]
            }
            TipsShellType::Sh => vec!["~/.profile"],
        }
    }

    /// Check if a file is a config file for this shell
    pub fn is_config_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        match self.shell {
            TipsShellType::Zsh => {
                filename.starts_with(".zsh")
                    || filename == ".zshrc"
                    || filename == ".zprofile"
                    || filename == ".zlogin"
                    || filename == ".zlogout"
                    || path_str.contains(".oh-my-zsh")
            }
            TipsShellType::Bash => {
                filename.starts_with(".bash") || filename == ".bashrc" || filename == ".profile"
            }
            TipsShellType::Fish => {
                path_str.contains("fish/config.fish")
                    || path_str.contains("fish/conf.d/")
                    || path_str.contains("fish/functions/")
            }
            TipsShellType::Sh => filename == ".profile" || filename == ".shrc",
        }
    }

    /// Get instructions for permanently applying an alias
    pub fn alias_instruction(&self, name: &str, expansion: &str) -> String {
        match self.shell {
            TipsShellType::Zsh | TipsShellType::Bash | TipsShellType::Sh => {
                format!(
                    "Add this line to {}:\n  alias {}='{}'",
                    self.config_file(),
                    name,
                    expansion.replace('\'', "'\\''")
                )
            }
            TipsShellType::Fish => {
                format!(
                    "Run:\n  alias --save {}='{}'",
                    name,
                    expansion.replace('\'', "\\'")
                )
            }
        }
    }

    /// Get instructions for adding to PATH
    pub fn path_instruction(&self, dir: &str) -> String {
        match self.shell {
            TipsShellType::Zsh | TipsShellType::Bash => {
                format!(
                    "Add this line to {}:\n  export PATH=\"{}:$PATH\"",
                    self.config_file(),
                    dir
                )
            }
            TipsShellType::Fish => {
                format!("Run:\n  fish_add_path {}", dir)
            }
            TipsShellType::Sh => {
                format!(
                    "Add this line to ~/.profile:\n  export PATH=\"{}:$PATH\"",
                    dir
                )
            }
        }
    }

    /// Generate a script snippet to reload configuration
    pub fn generate_reload_script(&self) -> String {
        match self.shell {
            TipsShellType::Zsh => r#"
# Reload zsh configuration
if [[ -f ~/.zshrc ]]; then
    source ~/.zshrc
    echo "Configuration reloaded!"
fi
"#
            .to_string(),
            TipsShellType::Bash => r#"
# Reload bash configuration
if [[ -f ~/.bashrc ]]; then
    source ~/.bashrc
    echo "Configuration reloaded!"
fi
"#
            .to_string(),
            TipsShellType::Fish => r#"
# Reload fish configuration
if test -f ~/.config/fish/config.fish
    source ~/.config/fish/config.fish
    echo "Configuration reloaded!"
end
"#
            .to_string(),
            TipsShellType::Sh => r#"
# Reload shell configuration
if [ -f ~/.profile ]; then
    . ~/.profile
    echo "Configuration reloaded!"
fi
"#
            .to_string(),
        }
    }
}

/// Format for displaying reload instructions
#[derive(Debug, Clone, Copy, Default)]
pub enum ReloadFormat {
    /// Simple one-liner
    #[default]
    Simple,
    /// Detailed with explanation
    Detailed,
    /// Copyable command only
    CommandOnly,
}

/// Generate reload instructions in the specified format
pub fn format_reload_instructions(reload: &ShellReload, format: ReloadFormat) -> String {
    match format {
        ReloadFormat::Simple => reload.reload_command().to_string(),
        ReloadFormat::CommandOnly => reload.reload_command().to_string(),
        ReloadFormat::Detailed => {
            let mut output = String::new();
            output.push_str("━━━ Shell Reload Required ━━━\n\n");
            output.push_str("Your shell configuration has been modified.\n");
            output.push_str("To apply changes, choose one option:\n\n");
            output.push_str(&format!("  Option 1: {}\n", reload.reload_command()));
            output.push_str(&format!("  Option 2: {}\n", reload.exec_command()));
            output.push_str("  Option 3: Open a new terminal window\n");
            output
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zsh_reload() {
        let reload = ShellReload::new(TipsShellType::Zsh);
        assert_eq!(reload.reload_command(), "source ~/.zshrc");
        assert_eq!(reload.exec_command(), "exec zsh");
        assert_eq!(reload.config_file(), "~/.zshrc");
    }

    #[test]
    fn test_bash_reload() {
        let reload = ShellReload::new(TipsShellType::Bash);
        assert_eq!(reload.reload_command(), "source ~/.bashrc");
        assert_eq!(reload.exec_command(), "exec bash");
        assert_eq!(reload.config_file(), "~/.bashrc");
    }

    #[test]
    fn test_fish_reload() {
        let reload = ShellReload::new(TipsShellType::Fish);
        assert!(reload.reload_command().contains("fish"));
        assert_eq!(reload.exec_command(), "exec fish");
    }

    #[test]
    fn test_reload_file_command() {
        let reload = ShellReload::new(TipsShellType::Zsh);
        let cmd = reload.reload_file_command(Path::new("/path/to/config"));
        assert_eq!(cmd, "source /path/to/config");

        let reload = ShellReload::new(TipsShellType::Sh);
        let cmd = reload.reload_file_command(Path::new("/path/to/config"));
        assert_eq!(cmd, ". /path/to/config");
    }

    #[test]
    fn test_is_config_file() {
        let zsh = ShellReload::new(TipsShellType::Zsh);
        assert!(zsh.is_config_file(Path::new("/home/user/.zshrc")));
        assert!(zsh.is_config_file(Path::new("/home/user/.zprofile")));
        assert!(!zsh.is_config_file(Path::new("/home/user/.bashrc")));

        let bash = ShellReload::new(TipsShellType::Bash);
        assert!(bash.is_config_file(Path::new("/home/user/.bashrc")));
        assert!(bash.is_config_file(Path::new("/home/user/.bash_profile")));
        assert!(!bash.is_config_file(Path::new("/home/user/.zshrc")));
    }

    #[test]
    fn test_alias_instruction() {
        let zsh = ShellReload::new(TipsShellType::Zsh);
        let instr = zsh.alias_instruction("ll", "ls -la");
        assert!(instr.contains("~/.zshrc"));
        assert!(instr.contains("alias ll="));

        let fish = ShellReload::new(TipsShellType::Fish);
        let instr = fish.alias_instruction("ll", "ls -la");
        assert!(instr.contains("alias --save"));
    }

    #[test]
    fn test_path_instruction() {
        let zsh = ShellReload::new(TipsShellType::Zsh);
        let instr = zsh.path_instruction("/usr/local/bin");
        assert!(instr.contains("export PATH="));
        assert!(instr.contains("/usr/local/bin"));

        let fish = ShellReload::new(TipsShellType::Fish);
        let instr = fish.path_instruction("/usr/local/bin");
        assert!(instr.contains("fish_add_path"));
    }

    #[test]
    fn test_config_files() {
        let zsh = ShellReload::new(TipsShellType::Zsh);
        let files = zsh.config_files();
        assert!(files.contains(&"~/.zshrc"));
        assert!(files.contains(&"~/.zshenv"));

        let bash = ShellReload::new(TipsShellType::Bash);
        let files = bash.config_files();
        assert!(files.contains(&"~/.bashrc"));
    }

    #[test]
    fn test_format_reload_instructions() {
        let reload = ShellReload::new(TipsShellType::Zsh);

        let simple = format_reload_instructions(&reload, ReloadFormat::Simple);
        assert_eq!(simple, "source ~/.zshrc");

        let detailed = format_reload_instructions(&reload, ReloadFormat::Detailed);
        assert!(detailed.contains("Shell Reload Required"));
        assert!(detailed.contains("source ~/.zshrc"));
        assert!(detailed.contains("exec zsh"));
    }
}
