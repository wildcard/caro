//! Tips Engine - Main orchestrator for the tips system
//!
//! Coordinates shell intelligence, suggestion generation, and display.

use super::config::TipsConfig;
use super::shell::ShellIntelligence;
use super::suggestions::{AliasSuggester, SuggestionResult, Tip, TipDisplay};
use std::collections::HashSet;

/// Session state for tracking shown tips
#[derive(Debug, Default)]
pub struct TipsSession {
    /// Tips shown in this session
    shown_tips: HashSet<String>,

    /// Commands since last tip for each tip ID
    commands_since_tip: std::collections::HashMap<String, usize>,

    /// Total tips shown this session
    total_shown: usize,
}

impl TipsSession {
    /// Create a new session
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that a tip was shown
    pub fn record_shown(&mut self, tip_id: &str) {
        self.shown_tips.insert(tip_id.to_string());
        self.commands_since_tip.insert(tip_id.to_string(), 0);
        self.total_shown += 1;
    }

    /// Check if a tip was shown recently (within cooldown period)
    pub fn is_in_cooldown(&self, tip_id: &str, cooldown: usize) -> bool {
        self.commands_since_tip
            .get(tip_id)
            .map(|&count| count < cooldown)
            .unwrap_or(false)
    }

    /// Increment command count for all tips (called after each command)
    pub fn tick(&mut self) {
        for count in self.commands_since_tip.values_mut() {
            *count += 1;
        }
    }

    /// Get total tips shown this session
    pub fn total_shown(&self) -> usize {
        self.total_shown
    }

    /// Check if session limit reached
    pub fn limit_reached(&self, max: usize) -> bool {
        self.total_shown >= max
    }
}

/// Main tips engine
pub struct TipsEngine {
    config: TipsConfig,
    intelligence: Option<ShellIntelligence>,
    session: TipsSession,
    display: TipDisplay,
}

impl TipsEngine {
    /// Create a new tips engine with default configuration
    pub fn new() -> Self {
        Self::with_config(TipsConfig::default())
    }

    /// Create a tips engine with custom configuration
    pub fn with_config(config: TipsConfig) -> Self {
        let intelligence = if config.enabled {
            ShellIntelligence::detect()
        } else {
            None
        };

        Self {
            config,
            intelligence,
            session: TipsSession::new(),
            display: TipDisplay::new(),
        }
    }

    /// Check if tips are enabled and ready
    pub fn is_ready(&self) -> bool {
        self.config.enabled && self.intelligence.is_some()
    }

    /// Get the shell intelligence (if available)
    pub fn intelligence(&self) -> Option<&ShellIntelligence> {
        self.intelligence.as_ref()
    }

    /// Suggest a tip for a command
    pub fn suggest(&mut self, command: &str) -> SuggestionResult {
        // Check if tips are enabled
        if !self.config.enabled {
            return SuggestionResult::Disabled;
        }

        // Check session limit
        if self.session.limit_reached(self.config.max_per_session) {
            return SuggestionResult::SessionLimitReached;
        }

        // Get shell intelligence
        let Some(ref intelligence) = self.intelligence else {
            return SuggestionResult::NoMatch;
        };

        // Check frequency
        if !self.config.frequency.should_show() {
            self.session.tick();
            return SuggestionResult::NoMatch;
        }

        // Try alias suggestion
        if self.config.categories.alias_shortcuts {
            let suggester = AliasSuggester::new(intelligence);
            if let Some(tip) = suggester.suggest(command) {
                // Check cooldown
                if self
                    .session
                    .is_in_cooldown(&tip.id, self.config.tip_cooldown)
                {
                    self.session.tick();
                    return SuggestionResult::Cooldown;
                }

                self.session.record_shown(&tip.id);
                return SuggestionResult::Found(tip);
            }

            // Try plugin suggestion
            if self.config.categories.plugin_recommendations {
                if let Some(tip) = suggester.suggest_plugin(command) {
                    if !self
                        .session
                        .is_in_cooldown(&tip.id, self.config.tip_cooldown * 2)
                    {
                        self.session.record_shown(&tip.id);
                        return SuggestionResult::Found(tip);
                    }
                }
            }
        }

        self.session.tick();
        SuggestionResult::NoMatch
    }

    /// Display a tip to the user
    pub fn display(&self, tip: &Tip) {
        println!("{}", self.display.format(tip));
    }

    /// Process a command and optionally show a tip
    pub fn process_command(&mut self, command: &str) -> Option<&Tip> {
        match self.suggest(command) {
            SuggestionResult::Found(tip) => {
                self.display(&tip);
                // Note: We can't return a reference to the tip because it's moved
                // In a real implementation, we'd store it in the engine
                None
            }
            _ => None,
        }
    }

    /// Get all applicable tips for a command (without rate limiting)
    pub fn all_tips(&self, command: &str) -> Vec<Tip> {
        let Some(ref intelligence) = self.intelligence else {
            return Vec::new();
        };

        let suggester = AliasSuggester::new(intelligence);
        let mut tips = suggester.all_suggestions(command);

        if let Some(plugin_tip) = suggester.suggest_plugin(command) {
            tips.push(plugin_tip);
        }

        tips
    }

    /// Get statistics about the current session
    pub fn session_stats(&self) -> SessionStats {
        SessionStats {
            tips_shown: self.session.total_shown(),
            max_tips: self.config.max_per_session,
            aliases_available: self
                .intelligence
                .as_ref()
                .map(|i| i.aliases().len())
                .unwrap_or(0),
            plugins_detected: self
                .intelligence
                .as_ref()
                .map(|i| i.plugin_managers().len())
                .unwrap_or(0),
        }
    }

    /// Reset the session state
    pub fn reset_session(&mut self) {
        self.session = TipsSession::new();
    }
}

impl Default for TipsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the tips session
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub tips_shown: usize,
    pub max_tips: usize,
    pub aliases_available: usize,
    pub plugins_detected: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tips_engine_creation() {
        let engine = TipsEngine::new();
        // Engine should be created even if shell detection fails
        assert!(engine.config.enabled);
    }

    #[test]
    fn test_disabled_engine() {
        let config = TipsConfig::disabled();
        let mut engine = TipsEngine::with_config(config);

        let result = engine.suggest("git status");
        assert!(matches!(result, SuggestionResult::Disabled));
    }

    #[test]
    fn test_session_limit() {
        let config = TipsConfig::default().with_max_per_session(0);
        let mut engine = TipsEngine::with_config(config);

        let result = engine.suggest("git status");
        assert!(matches!(result, SuggestionResult::SessionLimitReached));
    }

    #[test]
    fn test_session_stats() {
        let engine = TipsEngine::new();
        let stats = engine.session_stats();

        assert_eq!(stats.tips_shown, 0);
        assert_eq!(stats.max_tips, 5);
    }

    #[test]
    fn test_reset_session() {
        let mut engine = TipsEngine::new();
        engine.session.record_shown("test-tip");
        assert_eq!(engine.session.total_shown(), 1);

        engine.reset_session();
        assert_eq!(engine.session.total_shown(), 0);
    }
}
