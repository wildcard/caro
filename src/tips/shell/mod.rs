//! Shell intelligence module for tips functionality
//!
//! Provides shell detection, alias parsing, and plugin manager detection.
//!
//! # Components
//!
//! - [`detector`] - Shell type detection and config path discovery
//! - [`alias_parser`] - Extract aliases from shell configuration files
//! - [`plugin_detector`] - Detect installed plugin managers and their plugins
//!
//! # Example
//!
//! ```no_run
//! use caro::tips::shell::{ShellIntelligence, TipsShellType};
//!
//! let intel = ShellIntelligence::detect().unwrap();
//! println!("Shell: {}", intel.shell_type());
//! println!("Aliases: {:?}", intel.aliases().len());
//! println!("Plugin managers: {:?}", intel.plugin_managers().len());
//! ```

pub mod alias_parser;
pub mod detector;
pub mod plugin_detector;

pub use alias_parser::{Alias, AliasParser, AliasSource};
pub use detector::{ShellEnvironment, TipsShellType};
pub use plugin_detector::{PluginDetector, PluginManager};

use std::collections::HashMap;

/// Comprehensive shell intelligence aggregating all shell information
#[derive(Debug, Clone)]
pub struct ShellIntelligence {
    /// The detected shell environment
    environment: ShellEnvironment,

    /// All parsed aliases (name -> Alias)
    aliases: HashMap<String, Alias>,

    /// Detected plugin managers
    plugin_managers: Vec<PluginManager>,
}

impl ShellIntelligence {
    /// Detect and gather all shell intelligence
    pub fn detect() -> Option<Self> {
        let environment = ShellEnvironment::detect()?;

        // Parse aliases from config files
        let parser = AliasParser::new(environment.shell_type);
        let mut aliases = HashMap::new();

        for path in &environment.config_paths {
            if let Ok(file_aliases) = parser.parse_file(path) {
                for alias in file_aliases {
                    aliases.insert(alias.name.clone(), alias);
                }
            }
        }

        // Detect plugin managers
        let plugin_detector = PluginDetector::new()?;
        let plugin_managers = plugin_detector.detect_all();

        // Add plugin-provided aliases
        for manager in &plugin_managers {
            if let PluginManager::OhMyZsh { plugins, .. } = manager {
                for plugin in plugins {
                    let plugin_aliases = plugin_detector.get_omz_plugin_aliases(plugin);
                    for (name, expansion) in plugin_aliases {
                        if !aliases.contains_key(name) {
                            let alias = Alias::new(name, expansion)
                                .with_source(AliasSource::Plugin(plugin.clone()));
                            aliases.insert(name.to_string(), alias);
                        }
                    }
                }
            }
        }

        Some(Self {
            environment,
            aliases,
            plugin_managers,
        })
    }

    /// Get the detected shell type
    pub fn shell_type(&self) -> TipsShellType {
        self.environment.shell_type
    }

    /// Get the shell environment
    pub fn environment(&self) -> &ShellEnvironment {
        &self.environment
    }

    /// Get all aliases
    pub fn aliases(&self) -> &HashMap<String, Alias> {
        &self.aliases
    }

    /// Get a specific alias by name
    pub fn get_alias(&self, name: &str) -> Option<&Alias> {
        self.aliases.get(name)
    }

    /// Find an alias that matches a command
    pub fn find_alias_for_command(&self, command: &str) -> Option<&Alias> {
        // Exact match on expansion
        self.aliases
            .values()
            .find(|a| a.expansion == command || a.expansion.starts_with(&format!("{} ", command)))
    }

    /// Find all aliases that could shorten a command
    pub fn find_shorter_aliases(&self, command: &str) -> Vec<&Alias> {
        self.aliases
            .values()
            .filter(|a| {
                // The alias expansion matches or starts the command
                command.starts_with(&a.expansion) && a.chars_saved() > 0
            })
            .collect()
    }

    /// Get all detected plugin managers
    pub fn plugin_managers(&self) -> &[PluginManager] {
        &self.plugin_managers
    }

    /// Check if a specific plugin is installed
    pub fn has_plugin(&self, plugin_name: &str) -> bool {
        self.plugin_managers
            .iter()
            .any(|m| m.has_plugin(plugin_name))
    }

    /// Get the Oh My Zsh plugin manager if installed
    pub fn ohmyzsh(&self) -> Option<&PluginManager> {
        self.plugin_managers
            .iter()
            .find(|m| matches!(m, PluginManager::OhMyZsh { .. }))
    }

    /// Check if Oh My Zsh is installed
    pub fn has_ohmyzsh(&self) -> bool {
        self.ohmyzsh().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_shorter_aliases() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "gst".to_string(),
            Alias::new("gst", "git status").with_source(AliasSource::Plugin("git".to_string())),
        );
        aliases.insert(
            "ll".to_string(),
            Alias::new("ll", "ls -la").with_source(AliasSource::Unknown),
        );

        // Create a mock ShellIntelligence
        let intel = ShellIntelligence {
            environment: ShellEnvironment {
                shell_type: TipsShellType::Zsh,
                shell_path: "/bin/zsh".into(),
                config_paths: vec![],
                is_interactive: true,
                is_login_shell: false,
            },
            aliases,
            plugin_managers: vec![],
        };

        let matches = intel.find_shorter_aliases("git status");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].name, "gst");
    }
}
