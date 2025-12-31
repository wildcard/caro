//! Alias parsing from shell configuration files
//!
//! Extracts alias definitions from .zshrc, .bashrc, and other shell configs.

use super::detector::TipsShellType;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// Source of an alias definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasSource {
    /// Defined in user's config file
    UserConfig(String),
    /// Provided by a plugin
    Plugin(String),
    /// System-level alias
    System,
    /// Unknown source
    Unknown,
}

impl std::fmt::Display for AliasSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserConfig(path) => write!(f, "{}", path),
            Self::Plugin(name) => write!(f, "{} plugin", name),
            Self::System => write!(f, "system"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// A parsed shell alias
#[derive(Debug, Clone)]
pub struct Alias {
    /// The alias name (e.g., "gst")
    pub name: String,

    /// The command it expands to (e.g., "git status")
    pub expansion: String,

    /// Where this alias was defined
    pub source: AliasSource,

    /// Line number in the source file (if applicable)
    pub line_number: Option<usize>,
}

impl Alias {
    /// Create a new alias
    pub fn new(name: impl Into<String>, expansion: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            expansion: expansion.into(),
            source: AliasSource::Unknown,
            line_number: None,
        }
    }

    /// Set the source of this alias
    pub fn with_source(mut self, source: AliasSource) -> Self {
        self.source = source;
        self
    }

    /// Set the line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line_number = Some(line);
        self
    }

    /// Calculate characters saved by using this alias
    pub fn chars_saved(&self) -> i32 {
        self.expansion.len() as i32 - self.name.len() as i32
    }
}

/// Parser for extracting aliases from shell configuration files
pub struct AliasParser {
    shell_type: TipsShellType,
    // Separate regexes for single and double quotes (Rust regex doesn't support backreferences)
    bash_zsh_single_quote: Regex,
    bash_zsh_double_quote: Regex,
    bash_zsh_unquoted: Regex,
    fish_single_quote: Regex,
    fish_double_quote: Regex,
}

impl AliasParser {
    /// Create a new alias parser for the given shell type
    pub fn new(shell_type: TipsShellType) -> Self {
        // Separate regexes for bash/zsh with single and double quotes
        let bash_zsh_single_quote =
            Regex::new(r"^\s*alias\s+([a-zA-Z_][a-zA-Z0-9_!-]*)='([^']*)'\s*$")
                .expect("Invalid bash/zsh single quote regex");

        let bash_zsh_double_quote =
            Regex::new(r#"^\s*alias\s+([a-zA-Z_][a-zA-Z0-9_!-]*)="([^"]*)"\s*$"#)
                .expect("Invalid bash/zsh double quote regex");

        let bash_zsh_unquoted =
            Regex::new(r"^\s*alias\s+([a-zA-Z_][a-zA-Z0-9_!-]*)=(\S+)\s*$")
                .expect("Invalid bash/zsh unquoted regex");

        // Separate regexes for fish with single and double quotes
        let fish_single_quote =
            Regex::new(r"^\s*alias\s+([a-zA-Z_][a-zA-Z0-9_-]*)\s+'([^']*)'\s*$")
                .expect("Invalid fish single quote regex");

        let fish_double_quote =
            Regex::new(r#"^\s*alias\s+([a-zA-Z_][a-zA-Z0-9_-]*)\s+"([^"]*)"\s*$"#)
                .expect("Invalid fish double quote regex");

        Self {
            shell_type,
            bash_zsh_single_quote,
            bash_zsh_double_quote,
            bash_zsh_unquoted,
            fish_single_quote,
            fish_double_quote,
        }
    }

    /// Parse aliases from a config file
    pub fn parse_file(&self, path: &Path) -> Result<Vec<Alias>, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let source = AliasSource::UserConfig(path.display().to_string());
        Ok(self.parse_content(&content, source))
    }

    /// Parse aliases from config content
    pub fn parse_content(&self, content: &str, source: AliasSource) -> Vec<Alias> {
        let mut aliases = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            // Skip comments
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            if let Some(alias) = self.parse_line(line) {
                aliases.push(alias.with_source(source.clone()).with_line(line_num + 1));
            }
        }

        aliases
    }

    /// Parse a single line for an alias definition
    fn parse_line(&self, line: &str) -> Option<Alias> {
        match self.shell_type {
            TipsShellType::Zsh | TipsShellType::Bash | TipsShellType::Sh => {
                self.parse_bash_zsh_alias(line)
            }
            TipsShellType::Fish => self.parse_fish_alias(line),
        }
    }

    /// Parse bash/zsh style alias: alias name='command'
    fn parse_bash_zsh_alias(&self, line: &str) -> Option<Alias> {
        // Try single-quoted version: alias name='value'
        if let Some(caps) = self.bash_zsh_single_quote.captures(line) {
            let name = caps.get(1)?.as_str().to_string();
            let expansion = caps.get(2)?.as_str().to_string();
            return Some(Alias::new(name, expansion));
        }

        // Try double-quoted version: alias name="value"
        if let Some(caps) = self.bash_zsh_double_quote.captures(line) {
            let name = caps.get(1)?.as_str().to_string();
            let expansion = caps.get(2)?.as_str().to_string();
            return Some(Alias::new(name, expansion));
        }

        // Try unquoted version: alias name=value
        if let Some(caps) = self.bash_zsh_unquoted.captures(line) {
            let name = caps.get(1)?.as_str().to_string();
            let expansion = caps.get(2)?.as_str().to_string();
            return Some(Alias::new(name, expansion));
        }

        None
    }

    /// Parse fish style alias: alias name 'command'
    fn parse_fish_alias(&self, line: &str) -> Option<Alias> {
        // Try single-quoted version
        if let Some(caps) = self.fish_single_quote.captures(line) {
            let name = caps.get(1)?.as_str().to_string();
            let expansion = caps.get(2)?.as_str().to_string();
            return Some(Alias::new(name, expansion));
        }

        // Try double-quoted version
        if let Some(caps) = self.fish_double_quote.captures(line) {
            let name = caps.get(1)?.as_str().to_string();
            let expansion = caps.get(2)?.as_str().to_string();
            return Some(Alias::new(name, expansion));
        }

        None
    }

    /// Parse all aliases from multiple config files
    pub fn parse_all_configs(&self, paths: &[&Path]) -> HashMap<String, Alias> {
        let mut aliases = HashMap::new();

        for path in paths {
            if let Ok(file_aliases) = self.parse_file(path) {
                for alias in file_aliases {
                    // Later definitions override earlier ones
                    aliases.insert(alias.name.clone(), alias);
                }
            }
        }

        aliases
    }
}

/// Get aliases from the current shell by running `alias` command
pub fn get_runtime_aliases(shell_type: TipsShellType) -> Result<Vec<Alias>, std::io::Error> {
    let shell_name = shell_type.name();

    let output = std::process::Command::new(shell_name)
        .args(["-i", "-c", "alias"])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let parser = AliasParser::new(shell_type);
    Ok(parser.parse_content(&content, AliasSource::System))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bash_alias_single_quotes() {
        let parser = AliasParser::new(TipsShellType::Bash);
        let alias = parser.parse_line("alias gst='git status'").unwrap();
        assert_eq!(alias.name, "gst");
        assert_eq!(alias.expansion, "git status");
    }

    #[test]
    fn test_parse_bash_alias_double_quotes() {
        let parser = AliasParser::new(TipsShellType::Bash);
        let alias = parser.parse_line(r#"alias ll="ls -la""#).unwrap();
        assert_eq!(alias.name, "ll");
        assert_eq!(alias.expansion, "ls -la");
    }

    #[test]
    fn test_parse_zsh_alias() {
        let parser = AliasParser::new(TipsShellType::Zsh);
        let alias = parser.parse_line("alias gco='git checkout'").unwrap();
        assert_eq!(alias.name, "gco");
        assert_eq!(alias.expansion, "git checkout");
    }

    #[test]
    fn test_parse_fish_alias() {
        let parser = AliasParser::new(TipsShellType::Fish);
        let alias = parser.parse_line("alias gp 'git push'").unwrap();
        assert_eq!(alias.name, "gp");
        assert_eq!(alias.expansion, "git push");
    }

    #[test]
    fn test_parse_content_multiple_aliases() {
        let parser = AliasParser::new(TipsShellType::Bash);
        let content = r#"
# Git aliases
alias gst='git status'
alias gco='git checkout'

# Other aliases
alias ll='ls -la'
"#;
        let aliases = parser.parse_content(content, AliasSource::Unknown);
        assert_eq!(aliases.len(), 3);
    }

    #[test]
    fn test_skip_comments() {
        let parser = AliasParser::new(TipsShellType::Bash);
        let content = r#"
# This is a comment
# alias foo='bar'
alias real='command'
"#;
        let aliases = parser.parse_content(content, AliasSource::Unknown);
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "real");
    }

    #[test]
    fn test_chars_saved() {
        let alias = Alias::new("gst", "git status");
        assert_eq!(alias.chars_saved(), 7); // "git status" (10) - "gst" (3) = 7
    }
}
