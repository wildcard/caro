//! Export local aliases to cheatsheet format
//!
//! Allows users to export their shell aliases and create cheatsheets
//! for community contribution.

use crate::tips::kb::{Cheatsheet, CheatsheetAlias, CheatsheetTip};
use crate::tips::shell::{Alias, AliasSource, ShellIntelligence, TipsShellType};
use std::collections::HashSet;
use std::path::Path;

/// Options for exporting aliases
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Include aliases from shell config files
    pub include_user_config: bool,
    /// Include aliases from plugins
    pub include_plugins: bool,
    /// Filter by alias name pattern
    pub filter_pattern: Option<String>,
    /// Minimum characters saved to include
    pub min_chars_saved: i32,
    /// Generate tips from aliases
    pub generate_tips: bool,
    /// Cheatsheet name
    pub name: Option<String>,
    /// Cheatsheet description
    pub description: Option<String>,
    /// Author attribution
    pub author: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_user_config: true,
            include_plugins: true,
            filter_pattern: None,
            min_chars_saved: 0,
            generate_tips: true,
            name: None,
            description: None,
            author: None,
        }
    }
}

impl ExportOptions {
    /// Create options for exporting only user-defined aliases
    pub fn user_only() -> Self {
        Self {
            include_user_config: true,
            include_plugins: false,
            ..Self::default()
        }
    }

    /// Create options for exporting all aliases
    pub fn all() -> Self {
        Self::default()
    }
}

/// Exports local shell aliases to cheatsheet format
pub struct CheatsheetExporter {
    /// Target shell type
    shell: TipsShellType,
    /// Shell intelligence for alias detection
    intelligence: Option<ShellIntelligence>,
}

impl CheatsheetExporter {
    /// Create a new exporter for the specified shell
    pub fn new(shell: TipsShellType) -> Self {
        Self {
            shell,
            intelligence: ShellIntelligence::detect(),
        }
    }

    /// Create from existing shell intelligence
    pub fn from_intelligence(intelligence: ShellIntelligence) -> Self {
        let shell = intelligence.shell_type();
        Self {
            shell,
            intelligence: Some(intelligence),
        }
    }

    /// Export aliases using the given options
    pub fn export(&self, options: &ExportOptions) -> Result<Cheatsheet, ExportError> {
        let intelligence = self
            .intelligence
            .as_ref()
            .ok_or(ExportError::NoShellDetected)?;

        let aliases = intelligence.aliases();

        // Filter aliases
        let filtered: Vec<_> = aliases
            .values()
            .filter(|a| self.should_include(a, options))
            .collect();

        if filtered.is_empty() {
            return Err(ExportError::NoAliasesFound);
        }

        // Build cheatsheet
        let name = options
            .name
            .clone()
            .unwrap_or_else(|| format!("My {} Aliases", format!("{:?}", self.shell)));

        let description = options.description.clone();
        let authors = options
            .author
            .as_ref()
            .map(|a| vec![a.clone()])
            .unwrap_or_default();

        let mut cheatsheet = Cheatsheet {
            name,
            version: "1.0.0".to_string(),
            authors,
            shells: vec![format!("{:?}", self.shell).to_lowercase()],
            description,
            aliases: Vec::new(),
            tips: Vec::new(),
            plugins: Vec::new(),
        };

        // Convert aliases
        for alias in &filtered {
            cheatsheet.aliases.push(CheatsheetAlias {
                name: alias.name.clone(),
                expansion: alias.expansion.clone(),
                description: None, // Aliases don't have descriptions in the source
                plugin: match &alias.source {
                    AliasSource::Plugin(p) => Some(p.clone()),
                    _ => None,
                },
            });
        }

        // Generate tips if requested
        if options.generate_tips {
            cheatsheet.tips = self.generate_tips_from_aliases(&filtered);
        }

        Ok(cheatsheet)
    }

    /// Export only aliases from user config files
    pub fn export_user_aliases(&self) -> Result<Cheatsheet, ExportError> {
        self.export(&ExportOptions::user_only())
    }

    /// Export all detected aliases
    pub fn export_all_aliases(&self) -> Result<Cheatsheet, ExportError> {
        self.export(&ExportOptions::all())
    }

    /// Check if an alias should be included
    fn should_include(&self, alias: &Alias, options: &ExportOptions) -> bool {
        // Filter by source
        match &alias.source {
            AliasSource::UserConfig(_) if !options.include_user_config => return false,
            AliasSource::Plugin(_) if !options.include_plugins => return false,
            _ => {}
        }

        // Filter by chars saved
        if alias.chars_saved() < options.min_chars_saved {
            return false;
        }

        // Filter by pattern
        if let Some(ref pattern) = options.filter_pattern {
            if !alias.name.contains(pattern) && !alias.expansion.contains(pattern) {
                return false;
            }
        }

        true
    }

    /// Generate tips from aliases
    fn generate_tips_from_aliases(&self, aliases: &[&Alias]) -> Vec<CheatsheetTip> {
        let mut tips = Vec::new();
        let mut seen_commands = HashSet::new();

        for alias in aliases {
            // Skip if we've already generated a tip for this base command
            let base_cmd = alias.expansion.split_whitespace().next().unwrap_or("");
            if seen_commands.contains(base_cmd) {
                continue;
            }

            // Generate tip for significant aliases (save at least 3 chars)
            if alias.chars_saved() >= 3 {
                let tip_id = format!("use-{}-alias", alias.name.to_lowercase().replace('_', "-"));

                tips.push(CheatsheetTip {
                    id: tip_id,
                    pattern: alias.expansion.clone(),
                    is_regex: false,
                    message: format!(
                        "Use '{}' instead! Saves {} characters.",
                        alias.name,
                        alias.chars_saved()
                    ),
                    category: crate::tips::kb::KbTipCategory::AliasShortcut,
                    shells: vec![],
                    requires_plugin: match &alias.source {
                        AliasSource::Plugin(p) => Some(p.clone()),
                        _ => None,
                    },
                    tags: vec!["auto-generated".to_string()],
                });

                seen_commands.insert(base_cmd);
            }
        }

        tips
    }

    /// Export to YAML string
    pub fn to_yaml(&self, options: &ExportOptions) -> Result<String, ExportError> {
        let cheatsheet = self.export(options)?;
        serde_yaml::to_string(&cheatsheet)
            .map_err(|e| ExportError::SerializationError(e.to_string()))
    }

    /// Export to file
    pub fn to_file(&self, path: &Path, options: &ExportOptions) -> Result<(), ExportError> {
        let yaml = self.to_yaml(options)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Get export statistics
    pub fn get_stats(&self, options: &ExportOptions) -> Option<ExportStats> {
        let intelligence = self.intelligence.as_ref()?;
        let aliases = intelligence.aliases();

        let filtered: Vec<_> = aliases
            .values()
            .filter(|a| self.should_include(a, options))
            .collect();

        let from_plugins = filtered
            .iter()
            .filter(|a| matches!(a.source, AliasSource::Plugin(_)))
            .count();

        let from_config = filtered
            .iter()
            .filter(|a| matches!(a.source, AliasSource::UserConfig(_)))
            .count();

        let total_chars_saved: i32 = filtered.iter().map(|a| a.chars_saved().max(0)).sum();

        Some(ExportStats {
            total_aliases: filtered.len(),
            from_plugins,
            from_config,
            total_chars_saved,
        })
    }
}

/// Export statistics
#[derive(Debug, Clone)]
pub struct ExportStats {
    /// Total number of aliases to export
    pub total_aliases: usize,
    /// Number from plugins
    pub from_plugins: usize,
    /// Number from user config
    pub from_config: usize,
    /// Total characters saved by using all aliases
    pub total_chars_saved: i32,
}

impl std::fmt::Display for ExportStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Export Statistics:")?;
        writeln!(f, "  Total aliases: {}", self.total_aliases)?;
        writeln!(f, "  From plugins: {}", self.from_plugins)?;
        writeln!(f, "  From config: {}", self.from_config)?;
        writeln!(f, "  Characters saved: {}", self.total_chars_saved)?;
        Ok(())
    }
}

/// Errors that can occur during export
#[derive(Debug)]
pub enum ExportError {
    /// No shell detected
    NoShellDetected,
    /// No aliases found to export
    NoAliasesFound,
    /// IO error
    IoError(std::io::Error),
    /// Serialization error
    SerializationError(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoShellDetected => write!(f, "Could not detect shell configuration"),
            Self::NoAliasesFound => write!(f, "No aliases found to export"),
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<std::io::Error> for ExportError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_options_default() {
        let opts = ExportOptions::default();
        assert!(opts.include_user_config);
        assert!(opts.include_plugins);
        assert!(opts.generate_tips);
    }

    #[test]
    fn test_export_options_user_only() {
        let opts = ExportOptions::user_only();
        assert!(opts.include_user_config);
        assert!(!opts.include_plugins);
    }

    #[test]
    fn test_export_stats_display() {
        let stats = ExportStats {
            total_aliases: 10,
            from_plugins: 7,
            from_config: 3,
            total_chars_saved: 50,
        };

        let display = format!("{}", stats);
        assert!(display.contains("10"));
        assert!(display.contains("50"));
    }
}
