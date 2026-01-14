//! Generation Profile System
//!
//! This module provides different behavioral profiles for command generation.
//! Each profile configures how caro generates and explains commands.
//!
//! # Profiles
//!
//! - **Generator** (default): Optimized for quick command generation with brief explanations
//! - **Explainer**: Educational mode with detailed explanations of commands and options
//!
//! # Usage
//!
//! ```rust
//! use caro::prompts::profiles::{GenerationProfile, ProfileConfig};
//!
//! // Use default generator profile
//! let config = ProfileConfig::default();
//!
//! // Use explainer profile
//! let config = ProfileConfig::new(GenerationProfile::Explainer);
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Generation profile that controls command generation behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GenerationProfile {
    /// Default profile optimized for quick command generation
    /// - Fast inference
    /// - Brief explanations
    /// - Minimal output
    #[default]
    Generator,

    /// Educational profile with detailed explanations
    /// - Identifies relevant Unix tools
    /// - Explains command syntax and options
    /// - Provides usage examples and alternatives
    /// - Ideal for learning and understanding commands
    Explainer,
}

impl GenerationProfile {
    /// Get the display name of the profile
    pub fn name(&self) -> &'static str {
        match self {
            GenerationProfile::Generator => "generator",
            GenerationProfile::Explainer => "explainer",
        }
    }

    /// Get a description of what this profile does
    pub fn description(&self) -> &'static str {
        match self {
            GenerationProfile::Generator => {
                "Quick command generation with minimal output"
            }
            GenerationProfile::Explainer => {
                "Educational mode with detailed explanations of commands and options"
            }
        }
    }

    /// Check if this profile should include detailed explanations
    pub fn should_explain(&self) -> bool {
        matches!(self, GenerationProfile::Explainer)
    }

    /// Check if this profile is the default
    pub fn is_default(&self) -> bool {
        matches!(self, GenerationProfile::Generator)
    }
}

impl fmt::Display for GenerationProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for GenerationProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "generator" | "gen" | "default" => Ok(GenerationProfile::Generator),
            "explainer" | "explain" | "educational" | "learn" => Ok(GenerationProfile::Explainer),
            _ => Err(format!(
                "Unknown profile '{}'. Valid profiles: generator, explainer",
                s
            )),
        }
    }
}

/// Configuration for a generation profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// The active profile
    pub profile: GenerationProfile,

    /// Whether to show alternatives in output
    pub show_alternatives: bool,

    /// Whether to show examples in explanations
    pub show_examples: bool,

    /// Whether to show option breakdowns
    pub show_option_breakdown: bool,

    /// Maximum number of examples to show
    pub max_examples: usize,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            profile: GenerationProfile::default(),
            show_alternatives: false,
            show_examples: false,
            show_option_breakdown: false,
            max_examples: 3,
        }
    }
}

impl ProfileConfig {
    /// Create a new profile config with the specified profile
    pub fn new(profile: GenerationProfile) -> Self {
        match profile {
            GenerationProfile::Generator => Self::default(),
            GenerationProfile::Explainer => Self {
                profile,
                show_alternatives: true,
                show_examples: true,
                show_option_breakdown: true,
                max_examples: 5,
            },
        }
    }

    /// Create a generator profile config
    pub fn generator() -> Self {
        Self::new(GenerationProfile::Generator)
    }

    /// Create an explainer profile config
    pub fn explainer() -> Self {
        Self::new(GenerationProfile::Explainer)
    }
}

/// Explanation output for a generated command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExplanation {
    /// The generated command
    pub command: String,

    /// Brief one-line summary
    pub summary: String,

    /// Detailed explanation of what the command does
    pub detailed_explanation: String,

    /// Breakdown of individual options/flags
    pub option_breakdown: Vec<OptionExplanation>,

    /// Usage examples with variations
    pub examples: Vec<UsageExample>,

    /// Alternative approaches to achieve the same goal
    pub alternatives: Vec<AlternativeCommand>,

    /// Relevant Unix tool identified
    pub tool_used: String,

    /// When to use this command vs alternatives
    pub use_cases: Vec<String>,
}

impl Default for CommandExplanation {
    fn default() -> Self {
        Self {
            command: String::new(),
            summary: String::new(),
            detailed_explanation: String::new(),
            option_breakdown: Vec::new(),
            examples: Vec::new(),
            alternatives: Vec::new(),
            tool_used: String::new(),
            use_cases: Vec::new(),
        }
    }
}

/// Explanation of a single option/flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionExplanation {
    /// The option/flag (e.g., "-type f")
    pub option: String,

    /// What this option does
    pub description: String,

    /// Example value if applicable
    pub example_value: Option<String>,
}

/// A usage example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageExample {
    /// Description of this example
    pub description: String,

    /// The example command
    pub command: String,
}

/// An alternative command approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeCommand {
    /// The alternative command
    pub command: String,

    /// Why you might use this alternative
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_from_str() {
        assert_eq!(
            "generator".parse::<GenerationProfile>().unwrap(),
            GenerationProfile::Generator
        );
        assert_eq!(
            "explainer".parse::<GenerationProfile>().unwrap(),
            GenerationProfile::Explainer
        );
        assert_eq!(
            "explain".parse::<GenerationProfile>().unwrap(),
            GenerationProfile::Explainer
        );
        assert_eq!(
            "default".parse::<GenerationProfile>().unwrap(),
            GenerationProfile::Generator
        );
        assert!("unknown".parse::<GenerationProfile>().is_err());
    }

    #[test]
    fn test_profile_display() {
        assert_eq!(GenerationProfile::Generator.to_string(), "generator");
        assert_eq!(GenerationProfile::Explainer.to_string(), "explainer");
    }

    #[test]
    fn test_profile_config() {
        let gen_config = ProfileConfig::generator();
        assert_eq!(gen_config.profile, GenerationProfile::Generator);
        assert!(!gen_config.show_examples);

        let exp_config = ProfileConfig::explainer();
        assert_eq!(exp_config.profile, GenerationProfile::Explainer);
        assert!(exp_config.show_examples);
        assert!(exp_config.show_alternatives);
    }

    #[test]
    fn test_profile_should_explain() {
        assert!(!GenerationProfile::Generator.should_explain());
        assert!(GenerationProfile::Explainer.should_explain());
    }
}
