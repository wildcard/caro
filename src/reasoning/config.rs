//! Configuration for the reasoning mode
//!
//! Controls how the reasoning engine behaves, including:
//! - Whether reasoning is enabled
//! - Context fetching policies
//! - Clarification strategies

use super::clarification::ClarificationStrategy;
use serde::{Deserialize, Serialize};

/// Main reasoning mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// Is reasoning mode enabled?
    pub enabled: bool,

    /// The reasoning mode to use
    pub mode: ReasoningMode,

    /// Policy for fetching context
    pub context_fetch_policy: ContextFetchPolicy,

    /// Strategy for handling clarifications
    pub clarification_strategy: ClarificationStrategy,

    /// Maximum depth for file tree exploration
    pub max_tree_depth: usize,

    /// Confidence threshold below which to trigger reasoning
    pub confidence_threshold: f64,

    /// Whether to show reasoning chain in output
    pub show_reasoning: bool,

    /// Timeout for context enrichment in seconds
    pub enrichment_timeout_secs: u64,

    /// Commands that are safe to run for context (allowlist)
    pub safe_context_commands: Vec<String>,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: ReasoningMode::Auto,
            context_fetch_policy: ContextFetchPolicy::SafeOnly,
            clarification_strategy: ClarificationStrategy::AskUser,
            max_tree_depth: 3,
            confidence_threshold: 0.7,
            show_reasoning: false,
            enrichment_timeout_secs: 5,
            safe_context_commands: vec![
                "ls".to_string(),
                "pwd".to_string(),
                "git status".to_string(),
                "whoami".to_string(),
                "uname".to_string(),
            ],
        }
    }
}

impl ReasoningConfig {
    /// Create a new config with reasoning disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create a config for fast/minimal reasoning
    pub fn fast() -> Self {
        Self {
            enabled: true,
            mode: ReasoningMode::Fast,
            context_fetch_policy: ContextFetchPolicy::PassiveOnly,
            clarification_strategy: ClarificationStrategy::BestGuess,
            max_tree_depth: 1,
            enrichment_timeout_secs: 2,
            ..Default::default()
        }
    }

    /// Create a config for thorough reasoning
    pub fn thorough() -> Self {
        Self {
            enabled: true,
            mode: ReasoningMode::Thorough,
            context_fetch_policy: ContextFetchPolicy::AutoFetch,
            clarification_strategy: ClarificationStrategy::AskUser,
            max_tree_depth: 5,
            confidence_threshold: 0.8,
            enrichment_timeout_secs: 10,
            ..Default::default()
        }
    }

    /// Create a config that shows reasoning steps
    pub fn with_reasoning_output() -> Self {
        Self {
            show_reasoning: true,
            ..Default::default()
        }
    }

    /// Builder: set enabled state
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builder: set mode
    pub fn set_mode(mut self, mode: ReasoningMode) -> Self {
        self.mode = mode;
        self
    }

    /// Builder: set context fetch policy
    pub fn set_context_policy(mut self, policy: ContextFetchPolicy) -> Self {
        self.context_fetch_policy = policy;
        self
    }

    /// Builder: set clarification strategy
    pub fn set_clarification(mut self, strategy: ClarificationStrategy) -> Self {
        self.clarification_strategy = strategy;
        self
    }

    /// Builder: set show reasoning
    pub fn set_show_reasoning(mut self, show: bool) -> Self {
        self.show_reasoning = show;
        self
    }
}

/// The mode of reasoning to apply
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningMode {
    /// Disable reasoning entirely (fastest)
    Off,

    /// Minimal reasoning - just classify query type
    Fast,

    /// Automatic - apply reasoning when beneficial
    Auto,

    /// Always apply thorough reasoning
    Thorough,

    /// Interactive mode with user feedback
    Interactive,
}

impl Default for ReasoningMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl ReasoningMode {
    /// Whether this mode allows clarification questions
    pub fn allows_clarification(&self) -> bool {
        matches!(self, Self::Auto | Self::Thorough | Self::Interactive)
    }

    /// Whether this mode allows context enrichment
    pub fn allows_enrichment(&self) -> bool {
        !matches!(self, Self::Off | Self::Fast)
    }

    /// Whether this mode is interactive
    pub fn is_interactive(&self) -> bool {
        matches!(self, Self::Interactive)
    }
}

impl std::str::FromStr for ReasoningMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" | "disabled" | "none" => Ok(Self::Off),
            "fast" | "minimal" | "quick" => Ok(Self::Fast),
            "auto" | "automatic" | "default" => Ok(Self::Auto),
            "thorough" | "full" | "complete" => Ok(Self::Thorough),
            "interactive" | "ask" => Ok(Self::Interactive),
            _ => Err(format!(
                "Invalid reasoning mode '{}'. Valid options: off, fast, auto, thorough, interactive",
                s
            )),
        }
    }
}

impl std::fmt::Display for ReasoningMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "off"),
            Self::Fast => write!(f, "fast"),
            Self::Auto => write!(f, "auto"),
            Self::Thorough => write!(f, "thorough"),
            Self::Interactive => write!(f, "interactive"),
        }
    }
}

/// Policy for fetching context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextFetchPolicy {
    /// Automatically fetch all context (including running commands)
    AutoFetch,

    /// Only fetch safe context (no command execution)
    SafeOnly,

    /// Only use passively available information (no I/O)
    PassiveOnly,

    /// Ask permission before each fetch
    AskPermission,

    /// Don't fetch any context
    None,
}

impl Default for ContextFetchPolicy {
    fn default() -> Self {
        Self::SafeOnly
    }
}

impl ContextFetchPolicy {
    /// Whether this policy allows any I/O operations
    pub fn allows_io(&self) -> bool {
        !matches!(self, Self::PassiveOnly | Self::None)
    }

    /// Whether this policy allows running commands
    pub fn allows_commands(&self) -> bool {
        matches!(self, Self::AutoFetch)
    }

    /// Whether this policy requires user permission
    pub fn requires_permission(&self) -> bool {
        matches!(self, Self::AskPermission)
    }
}

impl std::str::FromStr for ContextFetchPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" | "autofetch" | "automatic" => Ok(Self::AutoFetch),
            "safe" | "safeonly" | "safe_only" => Ok(Self::SafeOnly),
            "passive" | "passiveonly" | "passive_only" => Ok(Self::PassiveOnly),
            "ask" | "askpermission" | "ask_permission" | "permission" => Ok(Self::AskPermission),
            "none" | "off" | "disabled" => Ok(Self::None),
            _ => Err(format!(
                "Invalid context fetch policy '{}'. Valid options: auto, safe, passive, ask, none",
                s
            )),
        }
    }
}

impl std::fmt::Display for ContextFetchPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AutoFetch => write!(f, "auto"),
            Self::SafeOnly => write!(f, "safe"),
            Self::PassiveOnly => write!(f, "passive"),
            Self::AskPermission => write!(f, "ask"),
            Self::None => write!(f, "none"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ReasoningConfig::default();
        assert!(config.enabled);
        assert_eq!(config.mode, ReasoningMode::Auto);
        assert_eq!(config.context_fetch_policy, ContextFetchPolicy::SafeOnly);
    }

    #[test]
    fn test_disabled_config() {
        let config = ReasoningConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_fast_config() {
        let config = ReasoningConfig::fast();
        assert!(config.enabled);
        assert_eq!(config.mode, ReasoningMode::Fast);
        assert_eq!(config.max_tree_depth, 1);
    }

    #[test]
    fn test_thorough_config() {
        let config = ReasoningConfig::thorough();
        assert_eq!(config.mode, ReasoningMode::Thorough);
        assert_eq!(config.max_tree_depth, 5);
    }

    #[test]
    fn test_reasoning_mode_parse() {
        assert_eq!("auto".parse::<ReasoningMode>().unwrap(), ReasoningMode::Auto);
        assert_eq!("fast".parse::<ReasoningMode>().unwrap(), ReasoningMode::Fast);
        assert_eq!("off".parse::<ReasoningMode>().unwrap(), ReasoningMode::Off);
        assert!("invalid".parse::<ReasoningMode>().is_err());
    }

    #[test]
    fn test_context_policy_parse() {
        assert_eq!("auto".parse::<ContextFetchPolicy>().unwrap(), ContextFetchPolicy::AutoFetch);
        assert_eq!("safe".parse::<ContextFetchPolicy>().unwrap(), ContextFetchPolicy::SafeOnly);
        assert_eq!("none".parse::<ContextFetchPolicy>().unwrap(), ContextFetchPolicy::None);
    }

    #[test]
    fn test_mode_capabilities() {
        assert!(!ReasoningMode::Off.allows_clarification());
        assert!(!ReasoningMode::Fast.allows_clarification());
        assert!(ReasoningMode::Auto.allows_clarification());
        assert!(ReasoningMode::Thorough.allows_clarification());
        assert!(ReasoningMode::Interactive.is_interactive());
    }

    #[test]
    fn test_policy_capabilities() {
        assert!(ContextFetchPolicy::AutoFetch.allows_commands());
        assert!(!ContextFetchPolicy::SafeOnly.allows_commands());
        assert!(!ContextFetchPolicy::PassiveOnly.allows_io());
        assert!(ContextFetchPolicy::AskPermission.requires_permission());
    }

    #[test]
    fn test_builder_pattern() {
        let config = ReasoningConfig::default()
            .set_enabled(true)
            .set_mode(ReasoningMode::Thorough)
            .set_context_policy(ContextFetchPolicy::AutoFetch)
            .set_show_reasoning(true);

        assert!(config.enabled);
        assert_eq!(config.mode, ReasoningMode::Thorough);
        assert_eq!(config.context_fetch_policy, ContextFetchPolicy::AutoFetch);
        assert!(config.show_reasoning);
    }
}
