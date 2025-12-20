//! User profile management for personalized suggestions

use super::{get_caro_dir, CommandPatterns, DetectedTool, EnvironmentInsights, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User profile with analyzed data for personalized suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Profile schema version for migrations
    pub version: String,

    /// When the profile was last analyzed
    pub last_analyzed: DateTime<Utc>,

    /// User experience level (inferred from analysis)
    pub experience_level: ExperienceLevel,

    /// Detected primary workflows
    pub workflows: Vec<Workflow>,

    /// Command frequency patterns from history
    pub command_patterns: CommandPatterns,

    /// Installed tools of interest
    pub detected_tools: Vec<DetectedTool>,

    /// Environment insights
    pub environment_insights: EnvironmentInsights,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_analyzed: Utc::now(),
            experience_level: ExperienceLevel::Beginner,
            workflows: Vec::new(),
            command_patterns: CommandPatterns::default(),
            detected_tools: Vec::new(),
            environment_insights: EnvironmentInsights::default(),
        }
    }
}

impl UserProfile {
    /// Check if the profile is stale and needs refresh
    pub fn is_stale(&self, ttl_secs: u64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.last_analyzed);
        age.num_seconds() > ttl_secs as i64
    }

    /// Get the top N commands from history
    pub fn top_commands(&self, n: usize) -> Vec<&(String, u32)> {
        self.command_patterns
            .top_commands
            .iter()
            .take(n)
            .collect()
    }

    /// Check if user has a specific tool installed
    pub fn has_tool(&self, name: &str) -> bool {
        self.detected_tools
            .iter()
            .any(|t| t.name.eq_ignore_ascii_case(name))
    }

    /// Get tools by category
    pub fn tools_by_category(&self, category: super::tools::ToolCategory) -> Vec<&DetectedTool> {
        self.detected_tools
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }
}

/// User experience level inferred from analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperienceLevel {
    /// New to terminal, limited history, basic tools only
    Beginner,
    /// Some experience, uses common development tools
    Intermediate,
    /// Power user with diverse tool usage and complex patterns
    Advanced,
}

impl ExperienceLevel {
    /// Determine experience level from analysis data
    pub fn infer(
        history_count: usize,
        unique_commands: usize,
        dev_tools_count: usize,
    ) -> Self {
        // Heuristics for experience level
        if history_count < 100 && unique_commands < 20 && dev_tools_count < 3 {
            Self::Beginner
        } else if history_count > 1000 && unique_commands > 50 && dev_tools_count > 5 {
            Self::Advanced
        } else {
            Self::Intermediate
        }
    }
}

impl std::fmt::Display for ExperienceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Beginner => write!(f, "beginner"),
            Self::Intermediate => write!(f, "intermediate"),
            Self::Advanced => write!(f, "advanced"),
        }
    }
}

/// A detected workflow pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Name of the workflow (e.g., "git", "docker", "python-dev")
    pub name: String,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Commands associated with this workflow
    pub related_commands: Vec<String>,
}

impl Workflow {
    /// Create a new workflow
    pub fn new(name: impl Into<String>, confidence: f32, commands: Vec<String>) -> Self {
        Self {
            name: name.into(),
            confidence: confidence.clamp(0.0, 1.0),
            related_commands: commands,
        }
    }
}

/// Manages user profile persistence
pub struct ProfileManager {
    profile_path: PathBuf,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new() -> Result<Self> {
        let caro_dir = get_caro_dir()?;
        let profile_path = caro_dir.join("profile.json");

        Ok(Self { profile_path })
    }

    /// Create with custom path (for testing)
    pub fn with_path(path: PathBuf) -> Self {
        Self { profile_path: path }
    }

    /// Load profile from disk
    pub fn load(&self) -> Result<Option<UserProfile>> {
        if !self.profile_path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&self.profile_path)?;
        let profile: UserProfile = serde_json::from_str(&contents)?;
        Ok(Some(profile))
    }

    /// Save profile to disk
    pub fn save(&self, profile: &UserProfile) -> Result<()> {
        let contents = serde_json::to_string_pretty(profile)?;
        std::fs::write(&self.profile_path, contents)?;
        Ok(())
    }

    /// Load profile or create default
    pub fn load_or_default(&self) -> Result<UserProfile> {
        match self.load()? {
            Some(profile) => Ok(profile),
            None => Ok(UserProfile::default()),
        }
    }

    /// Check if profile exists
    pub fn exists(&self) -> bool {
        self.profile_path.exists()
    }

    /// Get profile path
    pub fn path(&self) -> &PathBuf {
        &self.profile_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_profile_default() {
        let profile = UserProfile::default();
        assert_eq!(profile.version, "1.0.0");
        assert_eq!(profile.experience_level, ExperienceLevel::Beginner);
    }

    #[test]
    fn test_experience_level_inference() {
        assert_eq!(
            ExperienceLevel::infer(50, 10, 1),
            ExperienceLevel::Beginner
        );
        assert_eq!(
            ExperienceLevel::infer(500, 40, 4),
            ExperienceLevel::Intermediate
        );
        assert_eq!(
            ExperienceLevel::infer(2000, 100, 10),
            ExperienceLevel::Advanced
        );
    }

    #[test]
    fn test_profile_manager_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let profile_path = temp_dir.path().join("profile.json");
        let manager = ProfileManager::with_path(profile_path);

        let mut profile = UserProfile::default();
        profile.experience_level = ExperienceLevel::Advanced;

        manager.save(&profile).unwrap();
        let loaded = manager.load().unwrap().unwrap();

        assert_eq!(loaded.experience_level, ExperienceLevel::Advanced);
    }

    #[test]
    fn test_profile_stale_check() {
        let profile = UserProfile::default();
        // Fresh profile is not stale
        assert!(!profile.is_stale(3600));
    }

    #[test]
    fn test_workflow_creation() {
        let workflow = Workflow::new("git", 0.9, vec!["git status".into(), "git commit".into()]);
        assert_eq!(workflow.name, "git");
        assert_eq!(workflow.confidence, 0.9);
    }
}
