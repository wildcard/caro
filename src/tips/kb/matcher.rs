//! Knowledge Base pattern matching
//!
//! Matches commands against KB tips and aliases, with ranking and scoring.

use super::types::{KbAlias, KbTip, KbTipCategory, KnowledgeBase};
use regex::Regex;
use std::collections::HashMap;

/// Result of a KB match operation
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Matched tips
    pub tips: Vec<KbTip>,
    /// Matched aliases that could shorten the command
    pub aliases: Vec<KbAlias>,
    /// Overall match score (0.0 - 1.0)
    pub score: f32,
}

impl MatchResult {
    /// Create an empty match result
    pub fn empty() -> Self {
        Self {
            tips: Vec::new(),
            aliases: Vec::new(),
            score: 0.0,
        }
    }

    /// Check if there are any matches
    pub fn has_matches(&self) -> bool {
        !self.tips.is_empty() || !self.aliases.is_empty()
    }

    /// Get the best tip if available
    pub fn best_tip(&self) -> Option<&KbTip> {
        self.tips.first()
    }

    /// Get the best alias if available
    pub fn best_alias(&self) -> Option<&KbAlias> {
        self.aliases.first()
    }
}

/// Knowledge Base matcher
pub struct KbMatcher {
    /// The knowledge base to match against
    kb: KnowledgeBase,

    /// Compiled regex patterns for tips (id -> pattern)
    compiled_patterns: HashMap<String, Regex>,

    /// Current shell type for filtering
    shell: Option<String>,

    /// Installed plugins for filtering
    installed_plugins: Vec<String>,
}

impl KbMatcher {
    /// Create a new matcher for a knowledge base
    pub fn new(kb: KnowledgeBase) -> Self {
        let mut matcher = Self {
            kb,
            compiled_patterns: HashMap::new(),
            shell: None,
            installed_plugins: Vec::new(),
        };
        matcher.compile_patterns();
        matcher
    }

    /// Set the current shell for filtering
    pub fn with_shell(mut self, shell: impl Into<String>) -> Self {
        self.shell = Some(shell.into());
        self
    }

    /// Set installed plugins for filtering
    pub fn with_plugins(mut self, plugins: Vec<String>) -> Self {
        self.installed_plugins = plugins;
        self
    }

    /// Compile all regex patterns in the KB
    fn compile_patterns(&mut self) {
        for tip in &self.kb.tips {
            if tip.is_regex {
                if let Ok(re) = Regex::new(&tip.pattern) {
                    self.compiled_patterns.insert(tip.id.clone(), re);
                }
            }
        }
    }

    /// Match a command against the knowledge base
    pub fn match_command(&self, command: &str) -> MatchResult {
        let mut tips = Vec::new();
        let mut aliases = Vec::new();

        // Match tips
        for tip in &self.kb.tips {
            if self.tip_matches(tip, command) {
                tips.push(tip.clone());
            }
        }

        // Match aliases that could shorten the command
        for alias in &self.kb.aliases {
            if self.alias_applicable(alias, command) {
                aliases.push(alias.clone());
            }
        }

        // Sort tips by relevance
        tips.sort_by(|a, b| {
            // Prefer alias shortcuts over general tips
            let a_score = self.tip_relevance_score(a, command);
            let b_score = self.tip_relevance_score(b, command);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Sort aliases by chars saved (most savings first)
        aliases.sort_by(|a, b| b.chars_saved().cmp(&a.chars_saved()));

        // Calculate overall score
        let score = if tips.is_empty() && aliases.is_empty() {
            0.0
        } else {
            let tip_score = tips.first().map(|t| self.tip_relevance_score(t, command)).unwrap_or(0.0);
            let alias_score = if let Some(a) = aliases.first() {
                (a.chars_saved() as f32 / command.len() as f32).min(1.0)
            } else {
                0.0
            };
            (tip_score + alias_score) / 2.0
        };

        MatchResult { tips, aliases, score }
    }

    /// Check if a tip matches a command
    fn tip_matches(&self, tip: &KbTip, command: &str) -> bool {
        // Check shell filter
        if let Some(ref shell) = self.shell {
            if !tip.applies_to_shell(shell) {
                return false;
            }
        }

        // Check plugin requirement
        if let Some(ref required_plugin) = tip.requires_plugin {
            if !self.installed_plugins.iter().any(|p| p.eq_ignore_ascii_case(required_plugin)) {
                return false;
            }
        }

        // Check pattern match
        if tip.is_regex {
            if let Some(re) = self.compiled_patterns.get(&tip.id) {
                return re.is_match(command);
            }
            false
        } else {
            // Literal match
            command == tip.pattern
                || command.starts_with(&format!("{} ", tip.pattern))
        }
    }

    /// Check if an alias is applicable for a command
    fn alias_applicable(&self, alias: &KbAlias, command: &str) -> bool {
        // Check shell filter
        if let Some(ref shell) = self.shell {
            if !alias.shells.is_empty()
                && !alias.shells.iter().any(|s| s.eq_ignore_ascii_case(shell))
            {
                return false;
            }
        }

        // Check if the command starts with the alias expansion
        command.starts_with(&alias.expansion)
            || command == alias.expansion
    }

    /// Calculate relevance score for a tip (0.0 - 1.0)
    fn tip_relevance_score(&self, tip: &KbTip, command: &str) -> f32 {
        let mut score: f32 = 0.5; // Base score

        // Exact match gets higher score
        if tip.pattern == command {
            score += 0.3;
        }

        // Category-based scoring
        match tip.category {
            KbTipCategory::AliasShortcut => score += 0.2,
            KbTipCategory::SafetyTip => score += 0.15,
            KbTipCategory::BestPractice => score += 0.1,
            KbTipCategory::PluginRecommendation => score += 0.05,
            _ => {}
        }

        // Plugin match bonus
        if tip.requires_plugin.is_some() {
            score += 0.1;
        }

        score.min(1.0)
    }

    /// Find all aliases that match an expansion prefix
    pub fn find_aliases_for_expansion(&self, expansion: &str) -> Vec<&KbAlias> {
        self.kb
            .aliases
            .iter()
            .filter(|a| a.expansion == expansion || expansion.starts_with(&format!("{} ", a.expansion)))
            .collect()
    }

    /// Get the underlying knowledge base
    pub fn knowledge_base(&self) -> &KnowledgeBase {
        &self.kb
    }

    /// Get statistics about the matcher
    pub fn stats(&self) -> MatcherStats {
        MatcherStats {
            total_tips: self.kb.tips.len(),
            regex_tips: self.compiled_patterns.len(),
            total_aliases: self.kb.aliases.len(),
            total_plugins: self.kb.plugins.len(),
        }
    }
}

/// Statistics about the matcher
#[derive(Debug, Clone)]
pub struct MatcherStats {
    pub total_tips: usize,
    pub regex_tips: usize,
    pub total_aliases: usize,
    pub total_plugins: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_kb() -> KnowledgeBase {
        let mut kb = KnowledgeBase::with_version("1.0.0");

        // Add tips
        kb.add_tip(
            KbTip::new("git-status", "git status", "Use `gst` for git status")
                .with_category(KbTipCategory::AliasShortcut)
                .with_required_plugin("git".to_string()),
        );
        kb.add_tip(
            KbTip::new("git-push-force", "^git push -f", "Consider using --force-with-lease instead")
                .with_regex()
                .with_category(KbTipCategory::SafetyTip),
        );
        kb.add_tip(
            KbTip::new("zsh-only", "zsh-cmd", "ZSH tip")
                .with_shells(vec!["zsh".to_string()]),
        );

        // Add aliases
        kb.add_alias(
            KbAlias::new("gst", "git status")
                .with_plugin("git".to_string()),
        );
        kb.add_alias(
            KbAlias::new("gp", "git push")
                .with_plugin("git".to_string()),
        );
        kb.add_alias(KbAlias::new("ll", "ls -la"));

        kb
    }

    #[test]
    fn test_match_literal_tip() {
        let kb = create_test_kb();
        let matcher = KbMatcher::new(kb).with_plugins(vec!["git".to_string()]);

        let result = matcher.match_command("git status");
        assert!(result.has_matches());
        assert!(!result.tips.is_empty());
        assert_eq!(result.tips[0].id, "git-status");
    }

    #[test]
    fn test_match_regex_tip() {
        let kb = create_test_kb();
        let matcher = KbMatcher::new(kb);

        let result = matcher.match_command("git push -f origin main");
        assert!(result.has_matches());
        assert!(result.tips.iter().any(|t| t.id == "git-push-force"));
    }

    #[test]
    fn test_match_alias() {
        let kb = create_test_kb();
        let matcher = KbMatcher::new(kb);

        let result = matcher.match_command("git status");
        assert!(result.aliases.iter().any(|a| a.name == "gst"));
    }

    #[test]
    fn test_shell_filter() {
        let kb = create_test_kb();
        let matcher = KbMatcher::new(kb.clone()).with_shell("zsh");
        let result = matcher.match_command("zsh-cmd");
        assert!(result.tips.iter().any(|t| t.id == "zsh-only"));

        // Bash should not match zsh-only tip
        let matcher_bash = KbMatcher::new(kb).with_shell("bash");
        let result_bash = matcher_bash.match_command("zsh-cmd");
        assert!(!result_bash.tips.iter().any(|t| t.id == "zsh-only"));
    }

    #[test]
    fn test_plugin_filter() {
        let kb = create_test_kb();

        // Without git plugin, git-status tip should not match
        let matcher_no_plugin = KbMatcher::new(kb.clone());
        let result = matcher_no_plugin.match_command("git status");
        assert!(!result.tips.iter().any(|t| t.id == "git-status"));

        // With git plugin, it should match
        let matcher_with_plugin = KbMatcher::new(kb).with_plugins(vec!["git".to_string()]);
        let result = matcher_with_plugin.match_command("git status");
        assert!(result.tips.iter().any(|t| t.id == "git-status"));
    }

    #[test]
    fn test_alias_sorting() {
        let kb = create_test_kb();
        let matcher = KbMatcher::new(kb);

        let result = matcher.match_command("git status");
        // Aliases should be sorted by chars saved
        assert!(!result.aliases.is_empty());
    }

    #[test]
    fn test_empty_match() {
        let kb = create_test_kb();
        let matcher = KbMatcher::new(kb);

        let result = matcher.match_command("unknown command");
        assert!(!result.has_matches());
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_find_aliases_for_expansion() {
        let kb = create_test_kb();
        let matcher = KbMatcher::new(kb);

        let aliases = matcher.find_aliases_for_expansion("git status");
        assert!(aliases.iter().any(|a| a.name == "gst"));
    }

    #[test]
    fn test_stats() {
        let kb = create_test_kb();
        let matcher = KbMatcher::new(kb);

        let stats = matcher.stats();
        assert_eq!(stats.total_tips, 3);
        assert_eq!(stats.regex_tips, 1);
        assert_eq!(stats.total_aliases, 3);
    }
}
