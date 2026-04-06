//! Recipe module - CARO Hub command recipe types
//!
//! Defines the `CommandRecipe` data model for the CARO Hub UGC marketplace.
//! Recipes are the canonical content unit: reproducible, safe terminal actions
//! that wrap commands with metadata for discovery, safety, and social proof.
//!
//! # Evolution Stages
//!
//! The recipe payload supports four stages of complexity:
//! - **Static**: Single command (maps to existing `CommandArtifact`)
//! - **Parameterized**: Template with user inputs (`{{width}}`, `{{input}}`)
//! - **Composable**: Multi-step workflows (maps to existing `Runbook`)
//! - **Conditional**: Branching + approval gates (semi-agent)

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{Platform, RiskLevel, ShellType};

// ---------------------------------------------------------------------------
// Core recipe type
// ---------------------------------------------------------------------------

/// A command recipe: the canonical content unit for CARO Hub.
///
/// Each recipe wraps one or more shell commands with metadata for discovery,
/// safety validation, dependency management, and social proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecipe {
    // -- Identity --
    /// Globally unique, sortable identifier (ULID format recommended)
    pub id: String,
    /// URL-friendly slug, e.g. "convert-video-to-mp4"
    pub slug: String,
    /// Monotonic version counter (incremented on updates)
    pub version: u32,

    // -- Discovery --
    /// Human-readable title shown in search results
    pub title: String,
    /// Rich description, max 500 chars
    pub description: String,
    /// Canonical user intent (primary search anchor)
    pub intent: String,
    /// Consumer-facing category
    pub category: RecipeCategory,
    /// Searchable tags (max 15)
    pub tags: Vec<String>,
    /// Additional SEO keywords (not displayed to users)
    #[serde(default)]
    pub search_keywords: Vec<String>,

    // -- Execution --
    /// The execution payload (static, parameterized, composable, or conditional)
    pub payload: RecipePayload,

    // -- Dependencies --
    /// External tools required to run this recipe
    #[serde(default)]
    pub dependencies: Vec<ToolDependency>,

    // -- Safety & Validation --
    /// Overall confidence level (computed from safety analysis + community data)
    pub confidence_level: ConfidenceLevel,
    /// Detailed safety validation report
    pub safety_validation: SafetyReport,
    /// Whether the recipe can run in a sandbox (Docker/bubblewrap)
    #[serde(default)]
    pub sandboxable: bool,
    /// Whether the recipe produces deterministic output
    #[serde(default)]
    pub deterministic: bool,

    // -- Social Proof --
    /// Aggregated community statistics
    #[serde(default)]
    pub stats: RecipeStats,

    // -- Authorship --
    /// Author's machine fingerprint (CARO identity)
    pub author_id: String,
    /// Display name (populated after account claiming via BetterAuth)
    #[serde(default)]
    pub author_handle: Option<String>,
    /// If this recipe was forked, the ID of the original
    #[serde(default)]
    pub original_recipe_id: Option<String>,

    // -- Moderation --
    /// Current moderation status
    #[serde(default)]
    pub status: RecipeStatus,

    // -- Token tier --
    /// Whether this recipe requires token-gated features
    #[serde(default)]
    pub tier: RecipeTier,

    // -- Timestamps --
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
}

impl CommandRecipe {
    /// Validate that the recipe is well-formed.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Recipe ID cannot be empty".into());
        }
        if self.slug.is_empty() || self.slug.contains(' ') {
            return Err("Slug must be non-empty and contain no spaces".into());
        }
        if self.title.is_empty() {
            return Err("Title cannot be empty".into());
        }
        if self.description.len() > 500 {
            return Err(format!(
                "Description exceeds 500 chars (got {})",
                self.description.len()
            ));
        }
        if self.tags.len() > 15 {
            return Err(format!("Too many tags (max 15, got {})", self.tags.len()));
        }
        if self.author_id.is_empty() {
            return Err("Author ID cannot be empty".into());
        }
        self.payload.validate()?;
        for dep in &self.dependencies {
            dep.validate()?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Categories
// ---------------------------------------------------------------------------

/// Consumer-facing recipe categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeCategory {
    /// File cleanup, disk usage, backups
    PracticalUtility,
    /// Image generation, batch editing, audio/video (FFmpeg, ImageMagick)
    Creative,
    /// Scripts, automation, git workflows
    DevPower,
    /// PDF tools, converters, compressors
    ReplacementTool,
    /// Networking, services, monitoring
    SystemAdmin,
    /// CSV, JSON, text transforms
    DataProcessing,
}

impl std::fmt::Display for RecipeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PracticalUtility => write!(f, "Practical Utility"),
            Self::Creative => write!(f, "Creative"),
            Self::DevPower => write!(f, "Dev / Power"),
            Self::ReplacementTool => write!(f, "Replacement Tool"),
            Self::SystemAdmin => write!(f, "System Admin"),
            Self::DataProcessing => write!(f, "Data Processing"),
        }
    }
}

// ---------------------------------------------------------------------------
// Confidence levels
// ---------------------------------------------------------------------------

/// Recipe confidence level, mapping to the existing `RiskLevel` enum.
///
/// - `Safe` (green) = `RiskLevel::Safe`
/// - `NeedsReview` (yellow) = `RiskLevel::Moderate`
/// - `Risky` (red) = `RiskLevel::High` or `RiskLevel::Critical`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Safe,
    NeedsReview,
    Risky,
}

impl From<RiskLevel> for ConfidenceLevel {
    fn from(risk: RiskLevel) -> Self {
        match risk {
            RiskLevel::Safe => Self::Safe,
            RiskLevel::Moderate => Self::NeedsReview,
            RiskLevel::High | RiskLevel::Critical => Self::Risky,
        }
    }
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Safe => write!(f, "Safe"),
            Self::NeedsReview => write!(f, "Needs Review"),
            Self::Risky => write!(f, "Risky"),
        }
    }
}

// ---------------------------------------------------------------------------
// Moderation status & tier
// ---------------------------------------------------------------------------

/// Recipe moderation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStatus {
    #[default]
    Draft,
    PendingReview,
    Published,
    Flagged,
    Archived,
}

/// Whether a recipe uses free or token-gated features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeTier {
    #[default]
    Free,
    Enhanced,
}

// ---------------------------------------------------------------------------
// Payload types (the evolution stages)
// ---------------------------------------------------------------------------

/// The execution payload, supporting four evolution stages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecipePayload {
    /// Stage 1: Single command
    Static(StaticPayload),
    /// Stage 2: Template with user-provided parameters
    Parameterized(ParameterizedPayload),
    /// Stage 3: Multi-step workflow
    Composable(ComposablePayload),
    /// Stage 4: Branching + approval gates
    Conditional(ConditionalPayload),
}

impl RecipePayload {
    /// Validate the payload contents.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Static(p) => p.validate(),
            Self::Parameterized(p) => p.validate(),
            Self::Composable(p) => p.validate(),
            Self::Conditional(p) => p.validate(),
        }
    }
}

// -- Stage 1: Static --

/// A single command with explanation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticPayload {
    /// Natural language prompt that generated this command
    pub prompt: String,
    /// The shell command to execute
    pub command: String,
    /// Target shell
    pub shell: ShellType,
    /// Human-readable explanation of what the command does
    pub explanation: String,
    /// What the user should expect to see
    #[serde(default)]
    pub expected_output: Option<String>,
}

impl StaticPayload {
    pub fn validate(&self) -> Result<(), String> {
        if self.command.is_empty() {
            return Err("Static payload command cannot be empty".into());
        }
        if self.prompt.is_empty() {
            return Err("Static payload prompt cannot be empty".into());
        }
        Ok(())
    }
}

// -- Stage 2: Parameterized --

/// A command template with user-provided parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterizedPayload {
    /// Natural language prompt
    pub prompt: String,
    /// Command template with `{{param}}` placeholders
    pub command_template: String,
    /// Parameter definitions
    pub parameters: Vec<RecipeParameter>,
    /// Target shell
    pub shell: ShellType,
    /// Explanation of what the command does
    pub explanation: String,
    /// Expected output description
    #[serde(default)]
    pub expected_output: Option<String>,
}

impl ParameterizedPayload {
    pub fn validate(&self) -> Result<(), String> {
        if self.command_template.is_empty() {
            return Err("Command template cannot be empty".into());
        }
        if self.parameters.is_empty() {
            return Err("Parameterized payload must have at least one parameter".into());
        }
        for p in &self.parameters {
            p.validate()?;
        }
        Ok(())
    }
}

/// A single recipe parameter definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeParameter {
    /// Machine name used in template (e.g. "width")
    pub name: String,
    /// Human-readable label (e.g. "Output Width")
    pub label: String,
    /// Value type
    pub param_type: ParameterType,
    /// Default value (as string)
    #[serde(default)]
    pub default: Option<String>,
    /// Whether this parameter is required
    #[serde(default = "default_true")]
    pub required: bool,
    /// Validation regex or range (e.g. "1-8192")
    #[serde(default)]
    pub validation: Option<String>,
    /// Allowed values for `Enum` type
    #[serde(default)]
    pub enum_values: Vec<String>,
    /// Help text
    #[serde(default)]
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

impl RecipeParameter {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Parameter name cannot be empty".into());
        }
        if self.label.is_empty() {
            return Err("Parameter label cannot be empty".into());
        }
        if self.param_type == ParameterType::Enum && self.enum_values.is_empty() {
            return Err(format!(
                "Parameter '{}' is Enum type but has no enum_values",
                self.name
            ));
        }
        Ok(())
    }
}

/// Parameter value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Number,
    File,
    Enum,
    Boolean,
}

// -- Stage 3: Composable --

/// Multi-step workflow with ordered steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposablePayload {
    /// Ordered list of steps
    pub steps: Vec<RecipeStep>,
    /// Prerequisites (natural language descriptions)
    #[serde(default)]
    pub prerequisites: Vec<String>,
    /// Estimated time (e.g. "~5 minutes")
    #[serde(default)]
    pub estimated_time: Option<String>,
    /// Difficulty level
    #[serde(default)]
    pub difficulty: Option<Difficulty>,
}

impl ComposablePayload {
    pub fn validate(&self) -> Result<(), String> {
        if self.steps.is_empty() {
            return Err("Composable payload must have at least one step".into());
        }
        for (i, step) in self.steps.iter().enumerate() {
            if step.order != (i as u32 + 1) {
                return Err(format!(
                    "Step order mismatch: expected {}, got {}",
                    i + 1,
                    step.order
                ));
            }
            step.validate()?;
        }
        Ok(())
    }
}

/// A single step in a composable workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    /// 1-based step order
    pub order: u32,
    /// Step title
    pub title: String,
    /// Natural language prompt
    pub prompt: String,
    /// Command or command template
    pub command_template: String,
    /// Optional parameters for this step
    #[serde(default)]
    pub parameters: Vec<RecipeParameter>,
    /// Target shell
    pub shell: ShellType,
    /// Safety assessment for this step
    pub safety_level: ConfidenceLevel,
    /// Additional notes
    #[serde(default)]
    pub notes: Option<String>,
    /// Expected output
    #[serde(default)]
    pub expected_output: Option<String>,
    /// Whether to continue if this step fails
    #[serde(default)]
    pub continue_on_error: bool,
}

impl RecipeStep {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.is_empty() {
            return Err(format!("Step {} title cannot be empty", self.order));
        }
        if self.command_template.is_empty() {
            return Err(format!("Step {} command cannot be empty", self.order));
        }
        Ok(())
    }
}

/// Difficulty level for multi-step recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

// -- Stage 4: Conditional --

/// Branching workflow with approval gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalPayload {
    /// Steps with optional conditions and branching
    pub steps: Vec<ConditionalStep>,
    /// Points where the user must approve before continuing
    #[serde(default)]
    pub approval_gates: Vec<ApprovalGate>,
}

impl ConditionalPayload {
    pub fn validate(&self) -> Result<(), String> {
        if self.steps.is_empty() {
            return Err("Conditional payload must have at least one step".into());
        }
        for step in &self.steps {
            step.base.validate()?;
        }
        Ok(())
    }
}

/// A step that can branch based on the previous step's result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalStep {
    /// The underlying step definition
    #[serde(flatten)]
    pub base: RecipeStep,
    /// Condition to evaluate (e.g. "exit_code == 0")
    #[serde(default)]
    pub condition: Option<String>,
    /// What to do if this step fails
    #[serde(default)]
    pub on_failure: Option<FailureAction>,
    /// Step order to jump to on branch
    #[serde(default)]
    pub branch_to: Option<u32>,
}

/// Action to take when a conditional step fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureAction {
    Abort,
    Skip,
    Retry,
    Branch,
}

/// A point in the workflow where the user must approve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalGate {
    /// Execute this gate after step N
    pub after_step: u32,
    /// Message shown to the user
    pub message: String,
    /// Whether token holders can auto-approve
    #[serde(default)]
    pub auto_approve: bool,
}

// ---------------------------------------------------------------------------
// Dependencies
// ---------------------------------------------------------------------------

/// An external tool required by a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDependency {
    /// Tool name (e.g. "ffmpeg")
    pub name: String,
    /// Command to check if installed (e.g. "ffmpeg -version")
    pub check_command: String,
    /// Platform-specific install instructions
    #[serde(default)]
    pub install_hint: HashMap<String, String>,
    /// Whether the tool is optional
    #[serde(default)]
    pub optional: bool,
    /// Minimum required version
    #[serde(default)]
    pub min_version: Option<String>,
}

impl ToolDependency {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Dependency name cannot be empty".into());
        }
        if self.check_command.is_empty() {
            return Err(format!(
                "Dependency '{}' check_command cannot be empty",
                self.name
            ));
        }
        Ok(())
    }

    /// Get the install hint for the current platform.
    pub fn install_hint_for_current(&self) -> Option<&str> {
        let key = match Platform::detect() {
            Platform::MacOS => "macos",
            Platform::Linux => "ubuntu", // default to ubuntu for Linux
            Platform::Windows => "windows",
        };
        self.install_hint.get(key).map(|s| s.as_str())
    }
}

// ---------------------------------------------------------------------------
// Safety report
// ---------------------------------------------------------------------------

/// Detailed safety validation report for a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReport {
    /// Overall risk level from CARO's SafetyValidator
    pub overall_risk: RiskLevel,
    /// Which safety patterns matched (pattern names)
    #[serde(default)]
    pub pattern_matches: Vec<String>,
    /// When the validation was performed
    pub validated_at: DateTime<Utc>,
    /// CARO CLI version that performed the validation
    pub validator_version: String,
    /// Sandbox test result (if tested)
    #[serde(default)]
    pub sandbox_result: Option<SandboxResult>,
}

/// Result of running a recipe in a sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxResult {
    Pass,
    Fail,
    NotTested,
}

// ---------------------------------------------------------------------------
// Social proof / trust signals
// ---------------------------------------------------------------------------

/// Aggregated community statistics for a recipe.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStats {
    /// How many times this recipe has been run
    pub run_count: u64,
    /// Fraction of successful runs (0.0 to 1.0)
    pub success_rate: f64,
    /// Thumbs up / thumbs down
    pub ratings: Ratings,
    /// How many forks exist
    pub fork_count: u64,
    /// Comment count
    pub comment_count: u64,
    /// When the recipe was last run
    #[serde(default)]
    pub last_run_at: Option<DateTime<Utc>>,
}

/// Simple up/down rating counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ratings {
    pub up: u64,
    pub down: u64,
}

impl Ratings {
    /// Net score (up - down).
    pub fn net(&self) -> i64 {
        self.up as i64 - self.down as i64
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_static_recipe() -> CommandRecipe {
        CommandRecipe {
            id: "01HXYZ1234567890ABCDEF".into(),
            slug: "convert-video-to-mp4".into(),
            version: 1,
            title: "Convert any video to MP4".into(),
            description: "Convert a video file to MP4 format using FFmpeg.".into(),
            intent: "convert video to mp4".into(),
            category: RecipeCategory::Creative,
            tags: vec!["ffmpeg".into(), "video".into(), "conversion".into()],
            search_keywords: vec![],
            payload: RecipePayload::Static(StaticPayload {
                prompt: "convert my video to mp4".into(),
                command: "ffmpeg -i input.avi -c:v libx264 -c:a aac output.mp4".into(),
                shell: ShellType::Bash,
                explanation: "Uses FFmpeg to transcode the input video to H.264 video and AAC audio in an MP4 container.".into(),
                expected_output: Some("output.mp4 file created".into()),
            }),
            dependencies: vec![ToolDependency {
                name: "ffmpeg".into(),
                check_command: "ffmpeg -version".into(),
                install_hint: HashMap::from([
                    ("macos".into(), "brew install ffmpeg".into()),
                    ("ubuntu".into(), "apt install ffmpeg".into()),
                ]),
                optional: false,
                min_version: Some("5.0".into()),
            }],
            confidence_level: ConfidenceLevel::Safe,
            safety_validation: SafetyReport {
                overall_risk: RiskLevel::Safe,
                pattern_matches: vec![],
                validated_at: Utc::now(),
                validator_version: "1.1.0".into(),
                sandbox_result: None,
            },
            sandboxable: true,
            deterministic: true,
            stats: RecipeStats::default(),
            author_id: "machine_abc123".into(),
            author_handle: None,
            original_recipe_id: None,
            status: RecipeStatus::Draft,
            tier: RecipeTier::Free,
            created_at: Utc::now(),
            updated_at: None,
            published_at: None,
        }
    }

    #[test]
    fn test_static_recipe_validates() {
        let recipe = sample_static_recipe();
        assert!(recipe.validate().is_ok());
    }

    #[test]
    fn test_recipe_json_roundtrip() {
        let recipe = sample_static_recipe();
        let json = serde_json::to_string_pretty(&recipe).expect("serialize");
        let deserialized: CommandRecipe = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.slug, recipe.slug);
        assert_eq!(deserialized.title, recipe.title);
        assert_eq!(deserialized.category, recipe.category);
    }

    #[test]
    fn test_recipe_yaml_roundtrip() {
        let recipe = sample_static_recipe();
        let yaml = serde_yaml::to_string(&recipe).expect("serialize");
        let deserialized: CommandRecipe = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(deserialized.slug, recipe.slug);
        assert_eq!(deserialized.version, recipe.version);
    }

    #[test]
    fn test_empty_slug_fails_validation() {
        let mut recipe = sample_static_recipe();
        recipe.slug = "".into();
        assert!(recipe.validate().is_err());
    }

    #[test]
    fn test_slug_with_spaces_fails() {
        let mut recipe = sample_static_recipe();
        recipe.slug = "has spaces".into();
        assert!(recipe.validate().is_err());
    }

    #[test]
    fn test_too_many_tags_fails() {
        let mut recipe = sample_static_recipe();
        recipe.tags = (0..16).map(|i| format!("tag{}", i)).collect();
        assert!(recipe.validate().is_err());
    }

    #[test]
    fn test_confidence_from_risk_level() {
        assert_eq!(ConfidenceLevel::from(RiskLevel::Safe), ConfidenceLevel::Safe);
        assert_eq!(
            ConfidenceLevel::from(RiskLevel::Moderate),
            ConfidenceLevel::NeedsReview
        );
        assert_eq!(
            ConfidenceLevel::from(RiskLevel::High),
            ConfidenceLevel::Risky
        );
        assert_eq!(
            ConfidenceLevel::from(RiskLevel::Critical),
            ConfidenceLevel::Risky
        );
    }

    #[test]
    fn test_parameterized_payload() {
        let payload = ParameterizedPayload {
            prompt: "resize images".into(),
            command_template: "mogrify -resize {{width}}x{{height}} *.jpg".into(),
            parameters: vec![
                RecipeParameter {
                    name: "width".into(),
                    label: "Width".into(),
                    param_type: ParameterType::Number,
                    default: Some("800".into()),
                    required: true,
                    validation: Some("1-8192".into()),
                    enum_values: vec![],
                    description: Some("Output width in pixels".into()),
                },
                RecipeParameter {
                    name: "height".into(),
                    label: "Height".into(),
                    param_type: ParameterType::Number,
                    default: Some("600".into()),
                    required: true,
                    validation: None,
                    enum_values: vec![],
                    description: None,
                },
            ],
            shell: ShellType::Bash,
            explanation: "Batch resize all JPG images using ImageMagick.".into(),
            expected_output: None,
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_composable_payload_order_mismatch() {
        let payload = ComposablePayload {
            steps: vec![RecipeStep {
                order: 5, // should be 1
                title: "Step".into(),
                prompt: "do thing".into(),
                command_template: "echo hi".into(),
                parameters: vec![],
                shell: ShellType::Bash,
                safety_level: ConfidenceLevel::Safe,
                notes: None,
                expected_output: None,
                continue_on_error: false,
            }],
            prerequisites: vec![],
            estimated_time: None,
            difficulty: None,
        };
        assert!(payload.validate().is_err());
    }

    #[test]
    fn test_ratings_net_score() {
        let r = Ratings { up: 42, down: 7 };
        assert_eq!(r.net(), 35);
    }

    #[test]
    fn test_tool_dependency_validates() {
        let dep = ToolDependency {
            name: "ffmpeg".into(),
            check_command: "ffmpeg -version".into(),
            install_hint: HashMap::new(),
            optional: false,
            min_version: None,
        };
        assert!(dep.validate().is_ok());

        let bad_dep = ToolDependency {
            name: "".into(),
            check_command: "".into(),
            install_hint: HashMap::new(),
            optional: false,
            min_version: None,
        };
        assert!(bad_dep.validate().is_err());
    }
}
