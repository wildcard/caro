//! Community guides - curated examples of natural language → command mappings
//!
//! This module provides the data structures for the guides library feature,
//! similar to Warp's Terminus but with executable cmdai prompts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{RiskLevel, ShellType};

/// Category for organizing guides
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuideCategory {
    Git,
    Docker,
    FileManagement,
    Networking,
    SystemAdministration,
    Development,
    Database,
    Kubernetes,
    Cloud,
    Security,
    TextProcessing,
    Monitoring,
}

impl GuideCategory {
    /// Get human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Git => "Git",
            Self::Docker => "Docker",
            Self::FileManagement => "File Management",
            Self::Networking => "Networking",
            Self::SystemAdministration => "System Administration",
            Self::Development => "Development",
            Self::Database => "Database",
            Self::Kubernetes => "Kubernetes",
            Self::Cloud => "Cloud",
            Self::Security => "Security",
            Self::TextProcessing => "Text Processing",
            Self::Monitoring => "Monitoring",
        }
    }

    /// Get icon/emoji for display
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Git => "🔀",
            Self::Docker => "🐳",
            Self::FileManagement => "📁",
            Self::Networking => "🌐",
            Self::SystemAdministration => "⚙️",
            Self::Development => "💻",
            Self::Database => "🗄️",
            Self::Kubernetes => "☸️",
            Self::Cloud => "☁️",
            Self::Security => "🔒",
            Self::TextProcessing => "📝",
            Self::Monitoring => "📊",
        }
    }

    /// Get all categories
    pub fn all() -> Vec<Self> {
        vec![
            Self::Git,
            Self::Docker,
            Self::FileManagement,
            Self::Networking,
            Self::SystemAdministration,
            Self::Development,
            Self::Database,
            Self::Kubernetes,
            Self::Cloud,
            Self::Security,
            Self::TextProcessing,
            Self::Monitoring,
        ]
    }
}

impl std::fmt::Display for GuideCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for GuideCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "git" => Ok(Self::Git),
            "docker" => Ok(Self::Docker),
            "file_management" | "files" => Ok(Self::FileManagement),
            "networking" | "network" => Ok(Self::Networking),
            "system_administration" | "system" | "sysadmin" => Ok(Self::SystemAdministration),
            "development" | "dev" => Ok(Self::Development),
            "database" | "db" => Ok(Self::Database),
            "kubernetes" | "k8s" => Ok(Self::Kubernetes),
            "cloud" => Ok(Self::Cloud),
            "security" | "sec" => Ok(Self::Security),
            "text_processing" | "text" => Ok(Self::TextProcessing),
            "monitoring" | "observability" => Ok(Self::Monitoring),
            _ => Err(format!("Unknown guide category: {}", s)),
        }
    }
}

/// Difficulty level for a guide
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuideDifficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl std::fmt::Display for GuideDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Beginner => write!(f, "Beginner"),
            Self::Intermediate => write!(f, "Intermediate"),
            Self::Advanced => write!(f, "Advanced"),
        }
    }
}

/// Quality metrics for a guide
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuideMetrics {
    /// Number of upvotes from community
    pub upvotes: u32,
    /// Number of downvotes from community
    pub downvotes: u32,
    /// Number of times this guide was executed via "try in cmdai"
    pub execution_count: u64,
    /// Number of successful executions (exit code 0)
    pub success_count: u64,
    /// Number of failed executions
    pub failure_count: u64,
    /// Number of times this guide was viewed
    pub view_count: u64,
    /// Last time this guide was executed
    pub last_executed: Option<DateTime<Utc>>,
}

impl GuideMetrics {
    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f32 {
        if self.execution_count == 0 {
            return 0.0;
        }
        (self.success_count as f32) / (self.execution_count as f32)
    }

    /// Calculate quality score (weighted combination of metrics)
    pub fn quality_score(&self) -> f32 {
        let upvote_ratio = if self.upvotes + self.downvotes > 0 {
            (self.upvotes as f32) / ((self.upvotes + self.downvotes) as f32)
        } else {
            0.5 // Neutral if no votes
        };

        let success_rate = self.success_rate();
        let execution_score = (self.execution_count as f32).log10().max(0.0) / 4.0; // Log scale, capped at 10k

        // Weighted average: 40% upvotes, 40% success rate, 20% usage
        (upvote_ratio * 0.4) + (success_rate * 0.4) + (execution_score.min(1.0) * 0.2)
    }

    /// Check if this guide is popular (high usage)
    pub fn is_popular(&self) -> bool {
        self.execution_count > 100 && self.success_rate() > 0.8
    }

    /// Check if this guide needs review (low quality)
    pub fn needs_review(&self) -> bool {
        (self.execution_count > 20 && self.success_rate() < 0.5)
            || (self.upvotes + self.downvotes > 10 && self.upvotes < self.downvotes)
    }
}

/// A community-contributed guide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityGuide {
    /// Unique identifier
    pub id: String,

    /// Short title
    pub title: String,

    /// Longer description
    pub description: String,

    /// Category for organization
    pub category: GuideCategory,

    /// Difficulty level
    pub difficulty: GuideDifficulty,

    /// Tags for searchability
    pub tags: Vec<String>,

    // === The Core Content ===
    /// The natural language prompt to give to cmdai
    pub natural_language_prompt: String,

    /// The expected generated command
    pub generated_command: String,

    /// Target shell type
    pub shell_type: ShellType,

    // === Context and Safety ===
    /// Detailed explanation of what this command does
    pub explanation: String,

    /// Safety notes and warnings
    pub safety_notes: String,

    /// Assessed risk level
    pub risk_level: RiskLevel,

    /// Prerequisites (e.g., "git repository must exist", "docker must be running")
    pub prerequisites: Vec<String>,

    /// Expected outcomes after running the command
    pub expected_outcomes: Vec<String>,

    // === Community Engagement ===
    /// Original author
    pub author: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Quality and usage metrics
    pub metrics: GuideMetrics,

    // === Relationships ===
    /// Related guide IDs
    pub related_guides: Vec<String>,

    /// Related guardrail IDs
    pub related_guardrails: Vec<String>,

    /// Alternative commands that achieve similar results
    pub alternatives: Vec<String>,
}

impl CommunityGuide {
    /// Create a new guide
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        category: GuideCategory,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            category,
            difficulty: GuideDifficulty::Beginner,
            tags: Vec::new(),
            natural_language_prompt: String::new(),
            generated_command: String::new(),
            shell_type: ShellType::Bash,
            explanation: String::new(),
            safety_notes: String::new(),
            risk_level: RiskLevel::Safe,
            prerequisites: Vec::new(),
            expected_outcomes: Vec::new(),
            author: "community".to_string(),
            created_at: now,
            updated_at: now,
            metrics: GuideMetrics::default(),
            related_guides: Vec::new(),
            related_guardrails: Vec::new(),
            alternatives: Vec::new(),
        }
    }

    /// Builder: set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Builder: set prompt
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.natural_language_prompt = prompt.into();
        self
    }

    /// Builder: set command
    pub fn with_command(mut self, cmd: impl Into<String>) -> Self {
        self.generated_command = cmd.into();
        self
    }

    /// Builder: set explanation
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }

    /// Builder: set difficulty
    pub fn with_difficulty(mut self, difficulty: GuideDifficulty) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// Builder: set risk level
    pub fn with_risk_level(mut self, risk: RiskLevel) -> Self {
        self.risk_level = risk;
        self
    }

    /// Builder: add tag
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Builder: add prerequisite
    pub fn add_prerequisite(mut self, prereq: impl Into<String>) -> Self {
        self.prerequisites.push(prereq.into());
        self
    }

    /// Builder: add expected outcome
    pub fn add_expected_outcome(mut self, outcome: impl Into<String>) -> Self {
        self.expected_outcomes.push(outcome.into());
        self
    }

    /// Check if this guide matches a search query
    pub fn matches_query(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Search in title
        if self.title.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Search in description
        if self.description.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Search in prompt
        if self.natural_language_prompt.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Search in command
        if self.generated_command.to_lowercase().contains(&query_lower) {
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

    /// Validate guide completeness
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Guide ID cannot be empty".to_string());
        }

        if self.title.is_empty() {
            return Err("Guide title cannot be empty".to_string());
        }

        if self.natural_language_prompt.is_empty() {
            return Err("Natural language prompt cannot be empty".to_string());
        }

        if self.generated_command.is_empty() {
            return Err("Generated command cannot be empty".to_string());
        }

        if self.explanation.is_empty() {
            return Err("Explanation cannot be empty".to_string());
        }

        Ok(())
    }

    /// Get a short preview of the guide (for list views)
    pub fn preview(&self) -> String {
        let max_len = 100;
        if self.description.len() <= max_len {
            self.description.clone()
        } else {
            format!("{}...", &self.description[..max_len])
        }
    }

    /// Record an execution attempt
    pub fn record_execution(&mut self, success: bool) {
        self.metrics.execution_count += 1;
        if success {
            self.metrics.success_count += 1;
        } else {
            self.metrics.failure_count += 1;
        }
        self.metrics.last_executed = Some(Utc::now());
    }

    /// Record a view
    pub fn record_view(&mut self) {
        self.metrics.view_count += 1;
    }

    /// Record a vote
    pub fn record_vote(&mut self, is_upvote: bool) {
        if is_upvote {
            self.metrics.upvotes += 1;
        } else {
            self.metrics.downvotes += 1;
        }
    }
}

/// Filter options for listing guides
#[derive(Debug, Clone, Default)]
pub struct GuideFilter {
    pub category: Option<GuideCategory>,
    pub difficulty: Option<GuideDifficulty>,
    pub risk_level: Option<RiskLevel>,
    pub shell_type: Option<ShellType>,
    pub search_query: Option<String>,
    pub min_quality_score: Option<f32>,
    pub popular_only: bool,
    pub needs_review_only: bool,
    pub tags: Vec<String>,
}

impl GuideFilter {
    /// Create new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by category
    pub fn with_category(mut self, category: GuideCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Filter by difficulty
    pub fn with_difficulty(mut self, difficulty: GuideDifficulty) -> Self {
        self.difficulty = Some(difficulty);
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

    /// Filter by minimum quality score
    pub fn with_min_quality(mut self, score: f32) -> Self {
        self.min_quality_score = Some(score);
        self
    }

    /// Show only popular guides
    pub fn popular_only(mut self) -> Self {
        self.popular_only = true;
        self
    }

    /// Check if a guide passes this filter
    pub fn matches(&self, guide: &CommunityGuide) -> bool {
        // Category filter
        if let Some(cat) = self.category {
            if guide.category != cat {
                return false;
            }
        }

        // Difficulty filter
        if let Some(diff) = self.difficulty {
            if guide.difficulty != diff {
                return false;
            }
        }

        // Risk level filter
        if let Some(risk) = self.risk_level {
            if guide.risk_level != risk {
                return false;
            }
        }

        // Shell type filter
        if let Some(shell) = self.shell_type {
            if guide.shell_type != shell {
                return false;
            }
        }

        // Search query filter
        if let Some(ref query) = self.search_query {
            if !guide.matches_query(query) {
                return false;
            }
        }

        // Quality score filter
        if let Some(min_quality) = self.min_quality_score {
            if guide.metrics.quality_score() < min_quality {
                return false;
            }
        }

        // Popular filter
        if self.popular_only && !guide.metrics.is_popular() {
            return false;
        }

        // Needs review filter
        if self.needs_review_only && !guide.metrics.needs_review() {
            return false;
        }

        // Tags filter (guide must have ALL specified tags)
        if !self.tags.is_empty() {
            for required_tag in &self.tags {
                if !guide.tags.contains(required_tag) {
                    return false;
                }
            }
        }

        true
    }
}

/// Sort order for guide listings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideSortOrder {
    /// Most recently created first
    Newest,
    /// Oldest first
    Oldest,
    /// Highest quality score first
    BestQuality,
    /// Most executions first
    MostPopular,
    /// Alphabetical by title
    Alphabetical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guide_category_display() {
        assert_eq!(GuideCategory::Git.display_name(), "Git");
        assert_eq!(GuideCategory::Docker.icon(), "🐳");
    }

    #[test]
    fn test_guide_metrics_success_rate() {
        let mut metrics = GuideMetrics::default();
        metrics.execution_count = 100;
        metrics.success_count = 90;
        metrics.failure_count = 10;

        assert_eq!(metrics.success_rate(), 0.9);
    }

    #[test]
    fn test_guide_metrics_quality_score() {
        let mut metrics = GuideMetrics::default();
        metrics.upvotes = 80;
        metrics.downvotes = 20;
        metrics.execution_count = 100;
        metrics.success_count = 90;

        let score = metrics.quality_score();
        assert!(score > 0.7); // Should be high quality
    }

    #[test]
    fn test_guide_validation() {
        let mut guide = CommunityGuide::new("guide-001", "Test Guide", GuideCategory::Git);

        // Should fail without required fields
        assert!(guide.validate().is_err());

        // Add required fields
        guide = guide
            .with_prompt("undo my last commit")
            .with_command("git reset --soft HEAD~1")
            .with_explanation("This resets the last commit");

        assert!(guide.validate().is_ok());
    }

    #[test]
    fn test_guide_search() {
        let guide = CommunityGuide::new("guide-001", "Undo Git Commit", GuideCategory::Git)
            .with_description("Learn how to undo your last commit")
            .with_prompt("undo my last git commit")
            .with_command("git reset --soft HEAD~1")
            .add_tag("git")
            .add_tag("undo");

        assert!(guide.matches_query("git"));
        assert!(guide.matches_query("undo"));
        assert!(guide.matches_query("commit"));
        assert!(!guide.matches_query("docker"));
    }

    #[test]
    fn test_guide_filter() {
        let guide = CommunityGuide::new("guide-001", "Undo Git Commit", GuideCategory::Git)
            .with_prompt("undo commit")
            .with_command("git reset HEAD~1")
            .with_explanation("Undoes commit")
            .with_difficulty(GuideDifficulty::Beginner)
            .with_risk_level(RiskLevel::Safe);

        // Category filter
        let filter = GuideFilter::new().with_category(GuideCategory::Git);
        assert!(filter.matches(&guide));

        let filter = GuideFilter::new().with_category(GuideCategory::Docker);
        assert!(!filter.matches(&guide));

        // Difficulty filter
        let filter = GuideFilter::new().with_difficulty(GuideDifficulty::Beginner);
        assert!(filter.matches(&guide));

        let filter = GuideFilter::new().with_difficulty(GuideDifficulty::Advanced);
        assert!(!filter.matches(&guide));
    }

    #[test]
    fn test_guide_record_execution() {
        let mut guide = CommunityGuide::new("guide-001", "Test", GuideCategory::Git);

        assert_eq!(guide.metrics.execution_count, 0);
        assert_eq!(guide.metrics.success_count, 0);

        guide.record_execution(true);
        assert_eq!(guide.metrics.execution_count, 1);
        assert_eq!(guide.metrics.success_count, 1);
        assert_eq!(guide.metrics.success_rate(), 1.0);

        guide.record_execution(false);
        assert_eq!(guide.metrics.execution_count, 2);
        assert_eq!(guide.metrics.success_count, 1);
        assert_eq!(guide.metrics.success_rate(), 0.5);
    }
}
