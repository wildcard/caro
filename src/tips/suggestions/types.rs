//! Tip types and data structures
//!
//! Core types for representing tips and suggestions.

use crate::tips::shell::{Alias, AliasSource};

/// Category of a tip
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipCategory {
    /// Suggest using an alias instead of full command
    AliasShortcut,
    /// Recommend installing a plugin
    PluginRecommendation,
    /// General best practice advice
    BestPractice,
    /// Safety-related tip
    SafetyTip,
}

impl std::fmt::Display for TipCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AliasShortcut => write!(f, "Alias"),
            Self::PluginRecommendation => write!(f, "Plugin"),
            Self::BestPractice => write!(f, "Tip"),
            Self::SafetyTip => write!(f, "Safety"),
        }
    }
}

/// A tip or suggestion to show to the user
#[derive(Debug, Clone)]
pub struct Tip {
    /// Unique identifier for this tip
    pub id: String,

    /// Category of the tip
    pub category: TipCategory,

    /// The main message to display
    pub message: String,

    /// Optional action the user can take
    pub action: Option<TipAction>,

    /// Source of the tip (for alias suggestions)
    pub source: Option<AliasSource>,

    /// Keystroke savings (for alias suggestions)
    pub chars_saved: Option<i32>,
}

impl Tip {
    /// Create a new tip
    pub fn new(id: impl Into<String>, category: TipCategory, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            category,
            message: message.into(),
            action: None,
            source: None,
            chars_saved: None,
        }
    }

    /// Create an alias suggestion tip
    pub fn alias_suggestion(alias: &Alias, original_command: &str) -> Self {
        let chars_saved = alias.chars_saved();
        let message = format!(
            "Use `{}` instead of `{}`",
            alias.name,
            original_command
        );

        Self {
            id: format!("alias:{}", alias.name),
            category: TipCategory::AliasShortcut,
            message,
            action: Some(TipAction::UseAlias {
                alias: alias.name.clone(),
                expansion: alias.expansion.clone(),
            }),
            source: Some(alias.source.clone()),
            chars_saved: Some(chars_saved),
        }
    }

    /// Create a plugin recommendation tip
    pub fn plugin_recommendation(
        plugin_name: &str,
        plugin_manager: &str,
        reason: &str,
    ) -> Self {
        let message = format!(
            "Install the {} plugin for {} - {}",
            plugin_name, plugin_manager, reason
        );

        Self {
            id: format!("plugin:{}:{}", plugin_manager, plugin_name),
            category: TipCategory::PluginRecommendation,
            message,
            action: Some(TipAction::InstallPlugin {
                manager: plugin_manager.to_string(),
                plugin: plugin_name.to_string(),
            }),
            source: None,
            chars_saved: None,
        }
    }

    /// Set the action for this tip
    pub fn with_action(mut self, action: TipAction) -> Self {
        self.action = Some(action);
        self
    }
}

/// An action that can be taken in response to a tip
#[derive(Debug, Clone)]
pub enum TipAction {
    /// Suggest using an alias
    UseAlias {
        alias: String,
        expansion: String,
    },
    /// Suggest installing a plugin
    InstallPlugin {
        manager: String,
        plugin: String,
    },
    /// Open a URL for more information
    OpenUrl {
        url: String,
    },
    /// Run a command
    RunCommand {
        command: String,
        description: String,
    },
}

/// Result of a suggestion attempt
#[derive(Debug)]
pub enum SuggestionResult {
    /// A tip was found
    Found(Tip),
    /// No tip applicable for this command
    NoMatch,
    /// Tips are disabled
    Disabled,
    /// Already shown this tip recently
    Cooldown,
    /// Max tips per session reached
    SessionLimitReached,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tip_creation() {
        let tip = Tip::new("test-tip", TipCategory::BestPractice, "This is a tip");
        assert_eq!(tip.id, "test-tip");
        assert_eq!(tip.category, TipCategory::BestPractice);
        assert_eq!(tip.message, "This is a tip");
    }

    #[test]
    fn test_alias_suggestion_tip() {
        let alias = Alias::new("gst", "git status");
        let tip = Tip::alias_suggestion(&alias, "git status");

        assert!(tip.id.contains("gst"));
        assert_eq!(tip.category, TipCategory::AliasShortcut);
        assert!(tip.message.contains("gst"));
        assert!(tip.chars_saved.is_some());
    }

    #[test]
    fn test_plugin_recommendation_tip() {
        let tip = Tip::plugin_recommendation("git", "Oh My Zsh", "provides useful git aliases");

        assert!(tip.id.contains("git"));
        assert_eq!(tip.category, TipCategory::PluginRecommendation);
        assert!(tip.message.contains("git"));
    }
}
