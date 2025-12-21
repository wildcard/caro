//! Shell profile parsing module
//!
//! This module parses user shell configuration files to extract aliases,
//! environment variables, and PATH modifications. This helps Caro generate
//! commands that match the user's shell customizations.
//!
//! # Supported Shells
//!
//! - **Bash**: `~/.bashrc`, `~/.bash_profile`, `~/.profile`
//! - **Zsh**: `~/.zshrc`, `~/.zprofile`
//! - **Fish**: `~/.config/fish/config.fish`
//!
//! # Safety
//!
//! This module only reads and parses files - it never executes any shell code.
//! All parsing is done with simple regex patterns to avoid security risks.

use crate::models::ShellType;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, trace, warn};

/// Shell profile data extracted from configuration files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShellProfile {
    /// Extracted aliases (name → command)
    pub aliases: HashMap<String, String>,

    /// Exported environment variables (filtered for non-sensitive)
    pub exports: HashMap<String, String>,

    /// PATH additions
    pub path_additions: Vec<PathBuf>,

    /// Shell type
    pub shell_type: ShellType,

    /// Profile files that were parsed
    pub profile_files: Vec<PathBuf>,
}

impl ShellProfile {
    /// Parse shell profile files for the given shell type
    ///
    /// This reads and parses all relevant profile files for the shell,
    /// extracting aliases, exports, and PATH modifications.
    ///
    /// # Arguments
    ///
    /// * `shell` - The shell type to parse profiles for
    ///
    /// # Returns
    ///
    /// A ShellProfile with extracted data, or an error if parsing fails
    pub fn parse(shell: ShellType) -> Result<Self, std::io::Error> {
        debug!("Parsing shell profile for {:?}", shell);

        let profile_paths = Self::get_profile_paths(shell);
        let mut profile = Self {
            shell_type: shell,
            ..Default::default()
        };

        for path in &profile_paths {
            if path.exists() {
                debug!("Reading profile: {:?}", path);
                profile.profile_files.push(path.clone());

                match fs::read_to_string(path) {
                    Ok(content) => {
                        profile.parse_content(&content, shell);
                    }
                    Err(e) => {
                        warn!("Failed to read {:?}: {}", path, e);
                    }
                }
            }
        }

        // Also check for common plugin directories
        profile.parse_common_plugins(shell);

        debug!(
            "Parsed {} aliases, {} exports",
            profile.aliases.len(),
            profile.exports.len()
        );

        Ok(profile)
    }

    /// Create an empty profile
    pub fn empty(shell: ShellType) -> Self {
        Self {
            shell_type: shell,
            ..Default::default()
        }
    }

    /// Get profile file paths for a shell
    fn get_profile_paths(shell: ShellType) -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        match shell {
            ShellType::Bash => vec![
                home.join(".bashrc"),
                home.join(".bash_profile"),
                home.join(".profile"),
                home.join(".bash_aliases"),
            ],
            ShellType::Zsh => vec![
                home.join(".zshrc"),
                home.join(".zprofile"),
                home.join(".zshenv"),
                home.join(".zsh_aliases"),
            ],
            ShellType::Fish => vec![
                home.join(".config/fish/config.fish"),
                home.join(".config/fish/functions"),
            ],
            _ => vec![home.join(".profile")],
        }
    }

    /// Parse content from a shell profile
    fn parse_content(&mut self, content: &str, shell: ShellType) {
        match shell {
            ShellType::Fish => self.parse_fish_content(content),
            _ => self.parse_posix_content(content),
        }
    }

    /// Parse POSIX-style shell content (bash, zsh)
    fn parse_posix_content(&mut self, content: &str) {
        // Match alias definitions: alias name='command' or alias name="command"
        let alias_re =
            Regex::new(r#"^\s*alias\s+([a-zA-Z_][a-zA-Z0-9_-]*)=['"]([^'"]+)['"]"#).unwrap();

        // Match export statements: export VAR=value or export VAR="value"
        let export_re =
            Regex::new(r#"^\s*export\s+([A-Z_][A-Z0-9_]*)=['"]?([^'"]+)['"]?"#).unwrap();

        // Match PATH additions: export PATH="$PATH:..." or PATH="...${PATH}..."
        let path_re = Regex::new(r#"(?:export\s+)?PATH=.*?([/~][^:$"']+)"#).unwrap();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            // Extract aliases
            if let Some(caps) = alias_re.captures(line) {
                let name = caps.get(1).unwrap().as_str().to_string();
                let command = caps.get(2).unwrap().as_str().to_string();
                trace!("Found alias: {} = {}", name, command);
                self.aliases.insert(name, command);
            }

            // Extract exports (filter sensitive ones)
            if let Some(caps) = export_re.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let value = caps.get(2).unwrap().as_str().to_string();

                if !Self::is_sensitive_variable(name) {
                    self.exports.insert(name.to_string(), value);
                }
            }

            // Extract PATH additions
            if line.contains("PATH") {
                for caps in path_re.captures_iter(line) {
                    if let Some(path) = caps.get(1) {
                        let path_str = path.as_str();
                        // Expand ~ to home directory
                        let expanded = if path_str.starts_with('~') {
                            dirs::home_dir()
                                .map(|h| h.join(&path_str[2..]))
                                .unwrap_or_else(|| PathBuf::from(path_str))
                        } else {
                            PathBuf::from(path_str)
                        };
                        if !self.path_additions.contains(&expanded) {
                            self.path_additions.push(expanded);
                        }
                    }
                }
            }
        }
    }

    /// Parse Fish shell content
    fn parse_fish_content(&mut self, content: &str) {
        // Fish alias: alias name='command' or abbr -a name command
        let alias_re = Regex::new(r#"^\s*alias\s+([a-zA-Z_]\w*)(?:=|\s+)['"]?([^'"]+)['"]?"#).unwrap();
        let abbr_re = Regex::new(r#"^\s*abbr\s+(?:-a\s+)?([a-zA-Z_]\w*)\s+(.+)"#).unwrap();

        // Fish export: set -gx VAR value
        let export_re = Regex::new(r#"^\s*set\s+(?:-[gx]+\s+)?([A-Z_][A-Z0-9_]*)\s+(.+)"#).unwrap();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            // Extract aliases
            if let Some(caps) = alias_re.captures(line) {
                let name = caps.get(1).unwrap().as_str().to_string();
                let command = caps.get(2).unwrap().as_str().trim().to_string();
                self.aliases.insert(name, command);
            }

            // Extract abbreviations (treated as aliases)
            if let Some(caps) = abbr_re.captures(line) {
                let name = caps.get(1).unwrap().as_str().to_string();
                let command = caps.get(2).unwrap().as_str().trim().to_string();
                self.aliases.insert(name, command);
            }

            // Extract exports
            if let Some(caps) = export_re.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let value = caps.get(2).unwrap().as_str().to_string();

                if !Self::is_sensitive_variable(name) {
                    self.exports.insert(name.to_string(), value);
                }
            }
        }
    }

    /// Parse common shell plugins for additional aliases
    fn parse_common_plugins(&mut self, shell: ShellType) {
        if shell == ShellType::Zsh {
            self.add_oh_my_zsh_git_aliases();
        }
    }

    /// Add common oh-my-zsh git plugin aliases
    ///
    /// These are so common that we include them by default if the user
    /// has oh-my-zsh with the git plugin enabled.
    fn add_oh_my_zsh_git_aliases(&mut self) {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let omz_path = home.join(".oh-my-zsh");

        // Check if oh-my-zsh is installed
        if !omz_path.exists() {
            return;
        }

        // Check if git plugin is enabled (by checking .zshrc for plugins=(... git ...))
        let zshrc = home.join(".zshrc");
        if let Ok(content) = fs::read_to_string(&zshrc) {
            if !content.contains("plugins=") || !content.contains("git") {
                return;
            }
        }

        debug!("Detected oh-my-zsh with git plugin, adding common aliases");

        // Add the most commonly used oh-my-zsh git aliases
        let git_aliases = [
            ("g", "git"),
            ("ga", "git add"),
            ("gaa", "git add --all"),
            ("gst", "git status"),
            ("gss", "git status -s"),
            ("gd", "git diff"),
            ("gco", "git checkout"),
            ("gcb", "git checkout -b"),
            ("gcm", "git checkout $(git_main_branch)"),
            ("gc", "git commit -v"),
            ("gc!", "git commit -v --amend"),
            ("gca", "git commit -v -a"),
            ("gcam", "git commit -a -m"),
            ("gcmsg", "git commit -m"),
            ("gp", "git push"),
            ("gpf", "git push --force-with-lease"),
            ("gpsup", "git push --set-upstream origin $(git_current_branch)"),
            ("gl", "git pull"),
            ("gf", "git fetch"),
            ("gfa", "git fetch --all --prune"),
            ("gb", "git branch"),
            ("gba", "git branch -a"),
            ("gbd", "git branch -d"),
            ("gbD", "git branch -D"),
            ("gm", "git merge"),
            ("grb", "git rebase"),
            ("grba", "git rebase --abort"),
            ("grbc", "git rebase --continue"),
            ("grbi", "git rebase -i"),
            ("gsta", "git stash push"),
            ("gstp", "git stash pop"),
            ("gstl", "git stash list"),
            ("glg", "git log --stat"),
            ("glgg", "git log --graph"),
            ("glgp", "git log --stat -p"),
            ("glog", "git log --oneline --decorate --graph"),
        ];

        for (alias, command) in git_aliases {
            // Don't override user's custom aliases
            self.aliases
                .entry(alias.to_string())
                .or_insert_with(|| command.to_string());
        }
    }

    /// Check if a variable name is sensitive
    fn is_sensitive_variable(name: &str) -> bool {
        let sensitive_patterns = [
            "API_KEY",
            "TOKEN",
            "SECRET",
            "PASSWORD",
            "PASSWD",
            "CREDENTIAL",
            "AUTH",
            "PRIVATE",
            "KEY",
            "AWS_",
            "GITHUB_",
            "NPM_TOKEN",
            "DOCKER_",
        ];

        let upper = name.to_uppercase();
        sensitive_patterns
            .iter()
            .any(|pattern| upper.contains(pattern))
    }

    /// Get an alias command by name
    pub fn get_alias(&self, name: &str) -> Option<&str> {
        self.aliases.get(name).map(|s| s.as_str())
    }

    /// Check if a command has a shorter alias available
    ///
    /// Returns the alias name and what it expands to if found.
    pub fn has_shorter_alias(&self, command: &str) -> Option<(&str, &str)> {
        // Normalize command for comparison
        let command_normalized = command.trim();

        for (alias, expansion) in &self.aliases {
            // Check if the alias expansion matches the start of the command
            if command_normalized.starts_with(expansion.as_str())
                || command_normalized == expansion.as_str()
            {
                // Only suggest if alias is actually shorter than what it replaces
                // Compare alias length with the expansion length (what would be replaced)
                if alias.len() < expansion.len() {
                    return Some((alias.as_str(), expansion.as_str()));
                }
            }
        }

        None
    }

    /// Find the best alias for a command
    pub fn find_alias_for_command(&self, command: &str) -> Option<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        // Try to find exact match first
        for (alias, expansion) in &self.aliases {
            if expansion == command {
                return Some(alias.clone());
            }
        }

        // Try to find prefix match and append remaining args
        for (alias, expansion) in &self.aliases {
            if command.starts_with(expansion.as_str()) {
                let remainder = command[expansion.len()..].trim();
                if remainder.is_empty() {
                    return Some(alias.clone());
                } else {
                    return Some(format!("{} {}", alias, remainder));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bash_aliases() {
        let content = r#"
# Comment
alias ll='ls -la'
alias gs='git status'
alias gp="git push"

export PATH="$PATH:/usr/local/bin"
export EDITOR=vim
"#;

        let mut profile = ShellProfile::empty(ShellType::Bash);
        profile.parse_posix_content(content);

        assert_eq!(profile.aliases.get("ll"), Some(&"ls -la".to_string()));
        assert_eq!(profile.aliases.get("gs"), Some(&"git status".to_string()));
        assert_eq!(profile.aliases.get("gp"), Some(&"git push".to_string()));
        assert_eq!(profile.exports.get("EDITOR"), Some(&"vim".to_string()));
    }

    #[test]
    fn test_parse_zsh_aliases() {
        let content = r#"
alias gst='git status'
alias gco='git checkout'
alias -g G='| grep'

export GOPATH="$HOME/go"
"#;

        let mut profile = ShellProfile::empty(ShellType::Zsh);
        profile.parse_posix_content(content);

        assert_eq!(profile.aliases.get("gst"), Some(&"git status".to_string()));
        assert_eq!(
            profile.aliases.get("gco"),
            Some(&"git checkout".to_string())
        );
    }

    #[test]
    fn test_filter_sensitive_variables() {
        let content = r#"
export EDITOR=vim
export API_KEY="secret123"
export GITHUB_TOKEN="ghp_xxx"
export MY_VAR="safe"
"#;

        let mut profile = ShellProfile::empty(ShellType::Bash);
        profile.parse_posix_content(content);

        assert!(profile.exports.contains_key("EDITOR"));
        assert!(profile.exports.contains_key("MY_VAR"));
        assert!(!profile.exports.contains_key("API_KEY"));
        assert!(!profile.exports.contains_key("GITHUB_TOKEN"));
    }

    #[test]
    fn test_has_shorter_alias() {
        let mut profile = ShellProfile::empty(ShellType::Zsh);
        profile
            .aliases
            .insert("gst".to_string(), "git status".to_string());
        profile
            .aliases
            .insert("gco".to_string(), "git checkout".to_string());

        assert!(profile.has_shorter_alias("git status").is_some());
        assert_eq!(
            profile.has_shorter_alias("git status"),
            Some(("gst", "git status"))
        );

        assert!(profile.has_shorter_alias("git checkout main").is_some());
    }

    #[test]
    fn test_find_alias_for_command() {
        let mut profile = ShellProfile::empty(ShellType::Zsh);
        profile
            .aliases
            .insert("gst".to_string(), "git status".to_string());
        profile
            .aliases
            .insert("gco".to_string(), "git checkout".to_string());

        assert_eq!(
            profile.find_alias_for_command("git status"),
            Some("gst".to_string())
        );
        assert_eq!(
            profile.find_alias_for_command("git checkout main"),
            Some("gco main".to_string())
        );
    }

    #[test]
    fn test_parse_fish_content() {
        let content = r#"
alias gs='git status'
abbr -a gco git checkout
set -gx EDITOR vim
"#;

        let mut profile = ShellProfile::empty(ShellType::Fish);
        profile.parse_fish_content(content);

        assert_eq!(profile.aliases.get("gs"), Some(&"git status".to_string()));
        assert_eq!(
            profile.aliases.get("gco"),
            Some(&"git checkout".to_string())
        );
        assert_eq!(profile.exports.get("EDITOR"), Some(&"vim".to_string()));
    }

    #[test]
    fn test_sensitive_variable_detection() {
        assert!(ShellProfile::is_sensitive_variable("API_KEY"));
        assert!(ShellProfile::is_sensitive_variable("GITHUB_TOKEN"));
        assert!(ShellProfile::is_sensitive_variable("AWS_SECRET_ACCESS_KEY"));
        assert!(!ShellProfile::is_sensitive_variable("EDITOR"));
        assert!(!ShellProfile::is_sensitive_variable("PATH"));
    }
}
