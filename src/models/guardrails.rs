//! Guardrails metadata and visualization types
//!
//! This module extends the safety system with rich metadata for community transparency
//! and contribution. It enables the guardrails browser feature.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{RiskLevel, ShellType};
use crate::safety::DangerPattern;

/// Category of guardrail for organization and filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardrailCategory {
    /// Filesystem destruction (rm -rf /, etc.)
    FilesystemDestruction,
    /// Disk operations (mkfs, dd, etc.)
    DiskOperations,
    /// Privilege escalation (sudo su, setuid, etc.)
    PrivilegeEscalation,
    /// Network backdoors (nc -l -e, reverse shells, etc.)
    NetworkBackdoors,
    /// Process manipulation (kill -9, fork bombs, etc.)
    ProcessManipulation,
    /// System configuration modification (/etc writes, etc.)
    SystemModification,
    /// Environment variable manipulation (PATH override, etc.)
    EnvironmentManipulation,
    /// Package manager operations
    PackageManagement,
    /// Container and virtualization
    Containers,
}

impl GuardrailCategory {
    /// Get human-readable name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FilesystemDestruction => "Filesystem Destruction",
            Self::DiskOperations => "Disk Operations",
            Self::PrivilegeEscalation => "Privilege Escalation",
            Self::NetworkBackdoors => "Network Backdoors",
            Self::ProcessManipulation => "Process Manipulation",
            Self::SystemModification => "System Modification",
            Self::EnvironmentManipulation => "Environment Manipulation",
            Self::PackageManagement => "Package Management",
            Self::Containers => "Containers & Virtualization",
        }
    }

    /// Get icon/emoji for display
    pub fn icon(&self) -> &'static str {
        match self {
            Self::FilesystemDestruction => "🗑️",
            Self::DiskOperations => "💾",
            Self::PrivilegeEscalation => "🔐",
            Self::NetworkBackdoors => "🌐",
            Self::ProcessManipulation => "⚙️",
            Self::SystemModification => "🔧",
            Self::EnvironmentManipulation => "🔤",
            Self::PackageManagement => "📦",
            Self::Containers => "🐳",
        }
    }

    /// Get all categories for listing
    pub fn all() -> Vec<Self> {
        vec![
            Self::FilesystemDestruction,
            Self::DiskOperations,
            Self::PrivilegeEscalation,
            Self::NetworkBackdoors,
            Self::ProcessManipulation,
            Self::SystemModification,
            Self::EnvironmentManipulation,
            Self::PackageManagement,
            Self::Containers,
        ]
    }
}

impl std::fmt::Display for GuardrailCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for GuardrailCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "filesystem_destruction" | "filesystem" => Ok(Self::FilesystemDestruction),
            "disk_operations" | "disk" => Ok(Self::DiskOperations),
            "privilege_escalation" | "privilege" => Ok(Self::PrivilegeEscalation),
            "network_backdoors" | "network" => Ok(Self::NetworkBackdoors),
            "process_manipulation" | "process" => Ok(Self::ProcessManipulation),
            "system_modification" | "system" => Ok(Self::SystemModification),
            "environment_manipulation" | "environment" => Ok(Self::EnvironmentManipulation),
            "package_management" | "packages" => Ok(Self::PackageManagement),
            "containers" | "docker" => Ok(Self::Containers),
            _ => Err(format!("Unknown guardrail category: {}", s)),
        }
    }
}

/// Community note on a guardrail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityNote {
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub note: String,
    pub upvotes: u32,
}

/// Usage statistics for a guardrail
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardrailStats {
    /// Number of times this pattern has triggered
    pub times_triggered: u64,
    /// Number of times users manually overrode this block
    pub times_overridden: u64,
    /// Number of times users reported this as a false positive
    pub false_positive_reports: u64,
    /// Last time this pattern was triggered
    pub last_triggered: Option<DateTime<Utc>>,
}

impl GuardrailStats {
    /// Calculate false positive rate
    pub fn false_positive_rate(&self) -> f32 {
        if self.times_triggered == 0 {
            return 0.0;
        }
        (self.false_positive_reports as f32) / (self.times_triggered as f32)
    }

    /// Calculate override rate
    pub fn override_rate(&self) -> f32 {
        if self.times_triggered == 0 {
            return 0.0;
        }
        (self.times_overridden as f32) / (self.times_triggered as f32)
    }
}

/// Extended metadata for a danger pattern with community features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailMeta {
    /// Unique identifier for this guardrail
    pub id: String,

    /// The underlying safety pattern
    pub pattern: DangerPattern,

    /// Category for organization
    pub category: GuardrailCategory,

    /// Examples of commands this pattern blocks
    pub examples_blocked: Vec<String>,

    /// Examples of similar commands that are safe
    pub examples_safe: Vec<String>,

    /// Detailed explanation of why this is dangerous
    pub explanation: String,

    /// URL to learn more (if available)
    pub learn_more_url: Option<String>,

    /// Community-contributed notes and insights
    pub community_notes: Vec<CommunityNote>,

    /// Usage statistics (triggers, overrides, false positives)
    pub stats: GuardrailStats,

    /// When this guardrail was added
    pub created_at: DateTime<Utc>,

    /// Last time metadata was updated
    pub updated_at: DateTime<Utc>,

    /// Tags for additional searchability
    pub tags: Vec<String>,
}

impl GuardrailMeta {
    /// Create a new guardrail metadata entry
    pub fn new(
        id: impl Into<String>,
        pattern: DangerPattern,
        category: GuardrailCategory,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            pattern,
            category,
            examples_blocked: Vec::new(),
            examples_safe: Vec::new(),
            explanation: String::new(),
            learn_more_url: None,
            community_notes: Vec::new(),
            stats: GuardrailStats::default(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }

    /// Builder: set explanation
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }

    /// Builder: add blocked example
    pub fn add_blocked_example(mut self, example: impl Into<String>) -> Self {
        self.examples_blocked.push(example.into());
        self
    }

    /// Builder: add safe example
    pub fn add_safe_example(mut self, example: impl Into<String>) -> Self {
        self.examples_safe.push(example.into());
        self
    }

    /// Builder: add tag
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Builder: set learn more URL
    pub fn with_learn_more(mut self, url: impl Into<String>) -> Self {
        self.learn_more_url = Some(url.into());
        self
    }

    /// Check if this guardrail matches a search query
    pub fn matches_query(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Search in description
        if self.pattern.description.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Search in pattern regex
        if self.pattern.pattern.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Search in examples
        if self.examples_blocked.iter().any(|ex| ex.to_lowercase().contains(&query_lower)) {
            return true;
        }

        // Search in tags
        if self.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
            return true;
        }

        // Search in category
        if self.category.display_name().to_lowercase().contains(&query_lower) {
            return true;
        }

        false
    }

    /// Get shell compatibility as human-readable string
    pub fn shell_compatibility(&self) -> String {
        match self.pattern.shell_specific {
            Some(shell) => format!("{} only", shell),
            None => "All shells".to_string(),
        }
    }

    /// Validate guardrail metadata
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Guardrail ID cannot be empty".to_string());
        }

        if self.pattern.pattern.is_empty() {
            return Err("Pattern regex cannot be empty".to_string());
        }

        if self.pattern.description.is_empty() {
            return Err("Pattern description cannot be empty".to_string());
        }

        if self.examples_blocked.is_empty() {
            return Err("Must provide at least one blocked example".to_string());
        }

        // Validate regex compiles
        if let Err(e) = regex::Regex::new(&self.pattern.pattern) {
            return Err(format!("Invalid regex pattern: {}", e));
        }

        Ok(())
    }
}

/// Filter options for listing guardrails
#[derive(Debug, Clone, Default)]
pub struct GuardrailFilter {
    pub category: Option<GuardrailCategory>,
    pub risk_level: Option<RiskLevel>,
    pub shell_type: Option<ShellType>,
    pub search_query: Option<String>,
    pub min_triggers: Option<u64>,
    pub show_high_false_positive: bool,
}

impl GuardrailFilter {
    /// Create new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by category
    pub fn with_category(mut self, category: GuardrailCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Filter by risk level
    pub fn with_risk_level(mut self, risk: RiskLevel) -> Self {
        self.risk_level = Some(risk);
        self
    }

    /// Filter by shell type
    pub fn with_shell_type(mut self, shell: ShellType) -> Self {
        self.shell_type = Some(shell);
        self
    }

    /// Filter by search query
    pub fn with_search(mut self, query: impl Into<String>) -> Self {
        self.search_query = Some(query.into());
        self
    }

    /// Check if a guardrail passes this filter
    pub fn matches(&self, guardrail: &GuardrailMeta) -> bool {
        // Category filter
        if let Some(cat) = self.category {
            if guardrail.category != cat {
                return false;
            }
        }

        // Risk level filter
        if let Some(risk) = self.risk_level {
            if guardrail.pattern.risk_level != risk {
                return false;
            }
        }

        // Shell type filter
        if let Some(shell) = self.shell_type {
            if let Some(pattern_shell) = guardrail.pattern.shell_specific {
                if pattern_shell != shell {
                    return false;
                }
            }
        }

        // Search query filter
        if let Some(ref query) = self.search_query {
            if !guardrail.matches_query(query) {
                return false;
            }
        }

        // Min triggers filter
        if let Some(min) = self.min_triggers {
            if guardrail.stats.times_triggered < min {
                return false;
            }
        }

        // High false positive filter
        if self.show_high_false_positive {
            if guardrail.stats.false_positive_rate() < 0.1 {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardrail_category_display() {
        assert_eq!(
            GuardrailCategory::FilesystemDestruction.display_name(),
            "Filesystem Destruction"
        );
        assert_eq!(GuardrailCategory::DiskOperations.icon(), "💾");
    }

    #[test]
    fn test_guardrail_category_from_str() {
        assert_eq!(
            "filesystem".parse::<GuardrailCategory>().unwrap(),
            GuardrailCategory::FilesystemDestruction
        );
        assert_eq!(
            "network_backdoors".parse::<GuardrailCategory>().unwrap(),
            GuardrailCategory::NetworkBackdoors
        );
    }

    #[test]
    fn test_guardrail_stats_rates() {
        let mut stats = GuardrailStats::default();
        stats.times_triggered = 100;
        stats.times_overridden = 10;
        stats.false_positive_reports = 5;

        assert_eq!(stats.override_rate(), 0.1);
        assert_eq!(stats.false_positive_rate(), 0.05);
    }

    #[test]
    fn test_guardrail_meta_validation() {
        let pattern = DangerPattern {
            pattern: r"rm\s+-rf\s+/".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Delete root".to_string(),
            shell_specific: None,
        };

        let mut meta = GuardrailMeta::new("grd-001", pattern, GuardrailCategory::FilesystemDestruction);

        // Should fail without examples
        assert!(meta.validate().is_err());

        // Add examples and try again
        meta = meta.add_blocked_example("rm -rf /");
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_guardrail_search() {
        let pattern = DangerPattern {
            pattern: r"rm\s+-rf\s+/".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of root directory".to_string(),
            shell_specific: None,
        };

        let meta = GuardrailMeta::new("grd-001", pattern, GuardrailCategory::FilesystemDestruction)
            .add_blocked_example("rm -rf /")
            .add_tag("deletion");

        assert!(meta.matches_query("rm"));
        assert!(meta.matches_query("root"));
        assert!(meta.matches_query("deletion"));
        assert!(meta.matches_query("filesystem"));
        assert!(!meta.matches_query("network"));
    }

    #[test]
    fn test_filter_matches() {
        let pattern = DangerPattern {
            pattern: r"rm\s+-rf\s+/".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Delete root".to_string(),
            shell_specific: Some(ShellType::Bash),
        };

        let meta = GuardrailMeta::new("grd-001", pattern, GuardrailCategory::FilesystemDestruction)
            .add_blocked_example("rm -rf /");

        // Test category filter
        let filter = GuardrailFilter::new()
            .with_category(GuardrailCategory::FilesystemDestruction);
        assert!(filter.matches(&meta));

        let filter = GuardrailFilter::new()
            .with_category(GuardrailCategory::NetworkBackdoors);
        assert!(!filter.matches(&meta));

        // Test risk level filter
        let filter = GuardrailFilter::new()
            .with_risk_level(RiskLevel::Critical);
        assert!(filter.matches(&meta));

        // Test shell type filter
        let filter = GuardrailFilter::new()
            .with_shell_type(ShellType::Bash);
        assert!(filter.matches(&meta));

        let filter = GuardrailFilter::new()
            .with_shell_type(ShellType::Zsh);
        assert!(!filter.matches(&meta));
    }
}
