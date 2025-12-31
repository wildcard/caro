//! Tips configuration management
//!
//! Configuration for the tips system including frequency, categories, and display options.

use serde::{Deserialize, Serialize};

/// Frequency of showing tips
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TipFrequency {
    /// Always show tips when available
    Always,
    /// Show tips sometimes (default ~30% of the time)
    #[default]
    Sometimes,
    /// Rarely show tips (~10% of the time)
    Rarely,
    /// Never show tips
    Never,
}

impl TipFrequency {
    /// Get the probability of showing a tip (0.0 to 1.0)
    pub fn probability(&self) -> f32 {
        match self {
            Self::Always => 1.0,
            Self::Sometimes => 0.3,
            Self::Rarely => 0.1,
            Self::Never => 0.0,
        }
    }

    /// Check if a tip should be shown based on this frequency
    pub fn should_show(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            _ => rand_simple() < self.probability(),
        }
    }
}

impl std::str::FromStr for TipFrequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "always" => Ok(Self::Always),
            "sometimes" => Ok(Self::Sometimes),
            "rarely" => Ok(Self::Rarely),
            "never" => Ok(Self::Never),
            _ => Err(format!(
                "Invalid tip frequency '{}'. Valid: always, sometimes, rarely, never",
                s
            )),
        }
    }
}

impl std::fmt::Display for TipFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Sometimes => write!(f, "sometimes"),
            Self::Rarely => write!(f, "rarely"),
            Self::Never => write!(f, "never"),
        }
    }
}

/// Categories of tips that can be enabled/disabled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TipCategories {
    /// Suggest shorter aliases for commands
    pub alias_shortcuts: bool,
    /// Recommend productivity plugins
    pub plugin_recommendations: bool,
    /// General shell best practices
    pub best_practices: bool,
    /// Safety-related tips
    pub safety_tips: bool,
}

impl Default for TipCategories {
    fn default() -> Self {
        Self {
            alias_shortcuts: true,
            plugin_recommendations: true,
            best_practices: true,
            safety_tips: true,
        }
    }
}

/// Configuration for the tips system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TipsConfig {
    /// Whether tips are enabled
    pub enabled: bool,

    /// How frequently to show tips
    pub frequency: TipFrequency,

    /// Maximum number of tips to show per session
    pub max_per_session: usize,

    /// Categories of tips to show
    pub categories: TipCategories,

    /// Show keystroke/time savings
    pub show_savings: bool,

    /// Cooldown between showing the same tip (in commands)
    pub tip_cooldown: usize,
}

impl Default for TipsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency: TipFrequency::Sometimes,
            max_per_session: 5,
            categories: TipCategories::default(),
            show_savings: true,
            tip_cooldown: 10,
        }
    }
}

impl TipsConfig {
    /// Create a disabled tips configuration
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Builder pattern: set enabled
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builder pattern: set frequency
    pub fn with_frequency(mut self, frequency: TipFrequency) -> Self {
        self.frequency = frequency;
        self
    }

    /// Builder pattern: set max per session
    pub fn with_max_per_session(mut self, max: usize) -> Self {
        self.max_per_session = max;
        self
    }
}

/// Simple random number generator (0.0 to 1.0) without external dependencies
fn rand_simple() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);

    // Simple LCG-style mixing
    let mixed = nanos.wrapping_mul(1103515245).wrapping_add(12345);
    (mixed as f32) / (u32::MAX as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tip_frequency_probability() {
        assert_eq!(TipFrequency::Always.probability(), 1.0);
        assert_eq!(TipFrequency::Never.probability(), 0.0);
        assert!(TipFrequency::Sometimes.probability() > 0.0);
        assert!(TipFrequency::Sometimes.probability() < 1.0);
    }

    #[test]
    fn test_tip_frequency_from_str() {
        assert_eq!(
            "always".parse::<TipFrequency>().unwrap(),
            TipFrequency::Always
        );
        assert_eq!(
            "never".parse::<TipFrequency>().unwrap(),
            TipFrequency::Never
        );
        assert!("invalid".parse::<TipFrequency>().is_err());
    }

    #[test]
    fn test_default_config() {
        let config = TipsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.frequency, TipFrequency::Sometimes);
        assert_eq!(config.max_per_session, 5);
    }

    #[test]
    fn test_disabled_config() {
        let config = TipsConfig::disabled();
        assert!(!config.enabled);
    }
}
