//! Learning Engine for cmdai V2
//!
//! The learning engine makes cmdai learn from every interaction, explain commands,
//! and improve over time. It provides:
//!
//! - **Pattern Database**: Stores command interaction history locally
//! - **Improvement Learning**: Learns from user edits to generated commands
//! - **Command Explainer**: Explains shell commands in plain English
//! - **Similarity Search**: Finds similar past commands for better suggestions
//! - **Tutorial System**: Interactive learning mode for terminal commands
//! - **Achievement System**: Gamified learning to encourage exploration
//!
//! All data is stored locally by default with optional encrypted cloud sync.
//!
//! # Privacy
//!
//! The learning engine is privacy-first:
//! - All data stored locally in `~/.cmdai/patterns.db`
//! - No telemetry without explicit opt-in
//! - Users can delete history anytime with `cmdai --clear-history`
//! - Optional encryption at rest
//!
//! # Example
//!
//! ```no_run
//! use cmdai::learning::{PatternDB, CommandPattern};
//! use uuid::Uuid;
//! use chrono::Utc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let db = PatternDB::new("~/.cmdai/patterns.db".into()).await?;
//!
//! let pattern = CommandPattern {
//!     id: Uuid::new_v4(),
//!     user_prompt: "find all log files".to_string(),
//!     generated_command: "find . -name '*.log'".to_string(),
//!     final_command: None,
//!     context_snapshot: serde_json::json!({}),
//!     execution_success: None,
//!     user_rating: None,
//!     timestamp: Utc::now(),
//! };
//!
//! db.record_interaction(pattern).await?;
//! # Ok(())
//! # }
//! ```

pub mod achievements;
pub mod explainer;
pub mod improvement_learner;
pub mod migration;
pub mod pattern_db;
pub mod similarity;
pub mod tutorials;

// Re-export commonly used types
pub use achievements::{Achievement, AchievementTracker, UnlockCondition};
pub use explainer::{
    Alternative, CommandExplainer, CommandInfo, Example, Explanation, ExplanationPart,
};
pub use improvement_learner::{ImprovementLearner, ImprovementPattern};
pub use migration::migrate_v1_to_v2;
pub use pattern_db::{CommandPattern, PatternDB};
pub use similarity::SimilaritySearch;
pub use tutorials::{Difficulty, Lesson, Quiz, Tutorial, TutorialResult};

use anyhow::Result;
use std::path::PathBuf;

/// Learning engine configuration
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// Database file path
    pub db_path: PathBuf,
    /// Enable learning from user edits
    pub learn_from_edits: bool,
    /// Enable similarity search
    pub enable_similarity: bool,
    /// Enable achievements
    pub enable_achievements: bool,
    /// Maximum patterns to store (for disk space management)
    pub max_patterns: Option<usize>,
}

impl Default for LearningConfig {
    fn default() -> Self {
        let db_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cmdai")
            .join("patterns.db");

        Self {
            db_path,
            learn_from_edits: true,
            enable_similarity: true,
            enable_achievements: true,
            max_patterns: Some(100_000), // 100K pattern limit
        }
    }
}

/// Main learning engine interface
pub struct LearningEngine {
    config: LearningConfig,
    pattern_db: PatternDB,
    improvement_learner: ImprovementLearner,
    explainer: CommandExplainer,
    similarity_search: SimilaritySearch,
    achievement_tracker: AchievementTracker,
}

impl LearningEngine {
    /// Create a new learning engine with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(LearningConfig::default()).await
    }

    /// Create a new learning engine with custom configuration
    pub async fn with_config(config: LearningConfig) -> Result<Self> {
        // Ensure .cmdai directory exists
        if let Some(parent) = config.db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let pattern_db = PatternDB::new(config.db_path.clone()).await?;
        let improvement_learner = ImprovementLearner::new(pattern_db.clone());
        let explainer = CommandExplainer::new()?;
        let similarity_search = SimilaritySearch::new(pattern_db.clone());
        let achievement_tracker = AchievementTracker::new(pattern_db.clone()).await?;

        Ok(Self {
            config,
            pattern_db,
            improvement_learner,
            explainer,
            similarity_search,
            achievement_tracker,
        })
    }

    /// Record a command interaction
    pub async fn record_interaction(&self, pattern: CommandPattern) -> Result<()> {
        self.pattern_db.record_interaction(pattern).await?;

        // Check for new achievements
        if self.config.enable_achievements {
            let _ = self.achievement_tracker.check_achievements().await;
        }

        Ok(())
    }

    /// Learn from a user edit
    pub async fn learn_from_edit(&self, pattern_id: uuid::Uuid, edited_command: &str) -> Result<()> {
        if !self.config.learn_from_edits {
            return Ok(());
        }

        self.pattern_db
            .learn_from_edit(pattern_id, edited_command)
            .await?;

        // Analyze the edit for improvement patterns
        if let Some(original) = self.pattern_db.get_pattern_by_id(pattern_id).await? {
            let _ = self
                .improvement_learner
                .analyze_edit(&original.generated_command, edited_command)
                .await;
        }

        Ok(())
    }

    /// Find similar past commands
    pub async fn find_similar(&self, prompt: &str, limit: usize) -> Result<Vec<CommandPattern>> {
        if !self.config.enable_similarity {
            return Ok(vec![]);
        }

        self.similarity_search.find_similar(prompt, limit).await
    }

    /// Explain a command
    pub fn explain(&self, command: &str) -> Result<Explanation> {
        self.explainer.explain(command)
    }

    /// Get improvement suggestions for a command
    pub async fn suggest_improvements(&self, command: &str) -> Result<Vec<ImprovementPattern>> {
        self.improvement_learner.suggest_improvements(command).await
    }

    /// Get user's achievement progress
    pub async fn get_achievements(&self) -> Result<Vec<Achievement>> {
        if !self.config.enable_achievements {
            return Ok(vec![]);
        }

        self.achievement_tracker.get_unlocked().await
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<LearningStats> {
        Ok(LearningStats {
            total_patterns: self.pattern_db.count_patterns().await?,
            patterns_with_edits: self.pattern_db.count_edited_patterns().await?,
            total_achievements: self.achievement_tracker.count_unlocked().await?,
            db_size_bytes: self.get_db_size().await?,
        })
    }

    /// Clear all learning data (privacy feature)
    pub async fn clear_history(&self) -> Result<()> {
        self.pattern_db.clear_all().await
    }

    /// Get the underlying pattern database
    pub fn pattern_db(&self) -> &PatternDB {
        &self.pattern_db
    }

    /// Get the command explainer
    pub fn explainer(&self) -> &CommandExplainer {
        &self.explainer
    }

    /// Get the achievement tracker
    pub fn achievement_tracker(&self) -> &AchievementTracker {
        &self.achievement_tracker
    }

    async fn get_db_size(&self) -> Result<u64> {
        let metadata = tokio::fs::metadata(&self.config.db_path).await?;
        Ok(metadata.len())
    }
}

/// Learning engine statistics
#[derive(Debug, Clone)]
pub struct LearningStats {
    pub total_patterns: usize,
    pub patterns_with_edits: usize,
    pub total_achievements: usize,
    pub db_size_bytes: u64,
}
