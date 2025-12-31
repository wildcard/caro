//! Alias suggestion engine
//!
//! Matches commands against known aliases and suggests shorter alternatives.

use super::types::{Tip, TipCategory};
use crate::tips::shell::{Alias, ShellIntelligence};

/// Suggests aliases for commands
pub struct AliasSuggester<'a> {
    intelligence: &'a ShellIntelligence,
}

impl<'a> AliasSuggester<'a> {
    /// Create a new alias suggester
    pub fn new(intelligence: &'a ShellIntelligence) -> Self {
        Self { intelligence }
    }

    /// Suggest an alias for a command
    pub fn suggest(&self, command: &str) -> Option<Tip> {
        let command = command.trim();

        // First, try exact match
        if let Some(alias) = self.find_exact_match(command) {
            return Some(Tip::alias_suggestion(alias, command));
        }

        // Then try prefix match
        if let Some((alias, matched_part)) = self.find_prefix_match(command) {
            return Some(Tip::alias_suggestion(alias, matched_part));
        }

        None
    }

    /// Find an alias whose expansion exactly matches the command
    fn find_exact_match(&self, command: &str) -> Option<&Alias> {
        self.intelligence
            .aliases()
            .values()
            .find(|a| a.expansion == command && a.chars_saved() > 0)
    }

    /// Find an alias whose expansion is a prefix of the command
    fn find_prefix_match(&self, command: &str) -> Option<(&Alias, &str)> {
        // Sort by expansion length (longest first) to prefer more specific matches
        let mut aliases: Vec<_> = self.intelligence.aliases().values().collect();
        aliases.sort_by(|a, b| b.expansion.len().cmp(&a.expansion.len()));

        for alias in aliases {
            // Check if command starts with the alias expansion followed by space or end
            if alias.chars_saved() > 0 {
                if command == alias.expansion {
                    return Some((alias, &alias.expansion));
                }
                if command.starts_with(&format!("{} ", alias.expansion)) {
                    return Some((alias, &alias.expansion));
                }
            }
        }

        None
    }

    /// Get all applicable aliases for a command with their savings
    pub fn all_suggestions(&self, command: &str) -> Vec<Tip> {
        let command = command.trim();
        let mut tips = Vec::new();

        for alias in self.intelligence.aliases().values() {
            if alias.chars_saved() > 0 {
                // Exact match
                if alias.expansion == command {
                    tips.push(Tip::alias_suggestion(alias, command));
                }
                // Prefix match
                else if command.starts_with(&format!("{} ", alias.expansion)) {
                    tips.push(Tip::alias_suggestion(alias, &alias.expansion));
                }
            }
        }

        // Sort by chars saved (most savings first)
        tips.sort_by(|a, b| b.chars_saved.unwrap_or(0).cmp(&a.chars_saved.unwrap_or(0)));

        tips
    }

    /// Suggest a plugin if the user frequently uses commands it would help with
    pub fn suggest_plugin(&self, command: &str) -> Option<Tip> {
        let command = command.trim();

        // If using git commands but don't have git plugin
        if command.starts_with("git ") && !self.intelligence.has_plugin("git") {
            if self.intelligence.has_ohmyzsh() {
                return Some(Tip::new(
                    "plugin:ohmyzsh:git",
                    TipCategory::PluginRecommendation,
                    "Enable the 'git' plugin in Oh My Zsh for useful git aliases like `gst`, `gco`, `gp`",
                ));
            } else {
                return Some(Tip::new(
                    "plugin:ohmyzsh:install",
                    TipCategory::PluginRecommendation,
                    "Install Oh My Zsh for 200+ productivity aliases. Run: sh -c \"$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\"",
                ));
            }
        }

        // If using docker commands but don't have docker plugin
        if command.starts_with("docker ") && !self.intelligence.has_plugin("docker") {
            if self.intelligence.has_ohmyzsh() {
                return Some(Tip::new(
                    "plugin:ohmyzsh:docker",
                    TipCategory::PluginRecommendation,
                    "Enable the 'docker' plugin in Oh My Zsh for useful docker aliases like `dps`, `dex`",
                ));
            }
        }

        // If using kubectl but don't have kubectl plugin
        if command.starts_with("kubectl ") && !self.intelligence.has_plugin("kubectl") {
            if self.intelligence.has_ohmyzsh() {
                return Some(Tip::new(
                    "plugin:ohmyzsh:kubectl",
                    TipCategory::PluginRecommendation,
                    "Enable the 'kubectl' plugin in Oh My Zsh for useful k8s aliases like `k`, `kgp`, `kgs`",
                ));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tips::shell::{AliasSource, ShellEnvironment, TipsShellType};
    use std::collections::HashMap;

    fn create_test_intelligence() -> ShellIntelligence {
        let mut aliases = HashMap::new();
        aliases.insert(
            "gst".to_string(),
            Alias::new("gst", "git status").with_source(AliasSource::Plugin("git".to_string())),
        );
        aliases.insert(
            "gco".to_string(),
            Alias::new("gco", "git checkout").with_source(AliasSource::Plugin("git".to_string())),
        );
        aliases.insert(
            "ll".to_string(),
            Alias::new("ll", "ls -la").with_source(AliasSource::Unknown),
        );

        ShellIntelligence {
            environment: ShellEnvironment {
                shell_type: TipsShellType::Zsh,
                shell_path: "/bin/zsh".into(),
                config_paths: vec![],
                is_interactive: true,
                is_login_shell: false,
            },
            aliases,
            plugin_managers: vec![],
        }
    }

    #[test]
    fn test_exact_match() {
        let intel = create_test_intelligence();
        let suggester = AliasSuggester::new(&intel);

        let tip = suggester.suggest("git status").unwrap();
        assert!(tip.message.contains("gst"));
        assert_eq!(tip.chars_saved, Some(7));
    }

    #[test]
    fn test_prefix_match() {
        let intel = create_test_intelligence();
        let suggester = AliasSuggester::new(&intel);

        let tip = suggester.suggest("git checkout main").unwrap();
        assert!(tip.message.contains("gco"));
    }

    #[test]
    fn test_no_match() {
        let intel = create_test_intelligence();
        let suggester = AliasSuggester::new(&intel);

        let tip = suggester.suggest("npm install");
        assert!(tip.is_none());
    }

    #[test]
    fn test_all_suggestions() {
        let intel = create_test_intelligence();
        let suggester = AliasSuggester::new(&intel);

        let tips = suggester.all_suggestions("git status");
        assert!(!tips.is_empty());
    }
}
