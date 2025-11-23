//! Achievement system for gamifying learning
//!
//! Tracks user progress and unlocks achievements to encourage exploration
//! and learning of shell commands.

use crate::learning::pattern_db::PatternDB;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

/// Achievement that can be unlocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub unlock_condition: UnlockCondition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlocked_at: Option<DateTime<Utc>>,
}

/// Conditions for unlocking achievements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnlockCondition {
    CommandsGenerated { count: u32 },
    TutorialsCompleted { count: u32 },
    DaysStreak { days: u32 },
    SafetyScoreAverage { score: f32 },
    SpecificCommand { command: String },
    PatternsEdited { count: u32 },
}

/// Achievement tracker
pub struct AchievementTracker {
    db: PatternDB,
    achievements: Vec<Achievement>,
}

impl AchievementTracker {
    /// Create a new achievement tracker
    pub async fn new(db: PatternDB) -> Result<Self> {
        let achievements = Self::create_built_in_achievements();

        // Initialize achievements table in database
        Self::initialize_schema(&db.pool).await?;

        Ok(Self { db, achievements })
    }

    /// Initialize achievements table
    async fn initialize_schema(pool: &Arc<SqlitePool>) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS unlocked_achievements (
                id TEXT PRIMARY KEY,
                achievement_id TEXT NOT NULL,
                unlocked_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool.as_ref())
        .await?;

        Ok(())
    }

    /// Create built-in achievements
    fn create_built_in_achievements() -> Vec<Achievement> {
        vec![
            Achievement {
                id: "first_command".to_string(),
                title: "🎉 First Command".to_string(),
                description: "Generated your first command with cmdai".to_string(),
                icon: "🎉".to_string(),
                unlock_condition: UnlockCondition::CommandsGenerated { count: 1 },
                unlocked_at: None,
            },
            Achievement {
                id: "getting_started".to_string(),
                title: "🚀 Getting Started".to_string(),
                description: "Generated 10 commands".to_string(),
                icon: "🚀".to_string(),
                unlock_condition: UnlockCondition::CommandsGenerated { count: 10 },
                unlocked_at: None,
            },
            Achievement {
                id: "power_user".to_string(),
                title: "⚡ Power User".to_string(),
                description: "Generated 100 commands".to_string(),
                icon: "⚡".to_string(),
                unlock_condition: UnlockCondition::CommandsGenerated { count: 100 },
                unlocked_at: None,
            },
            Achievement {
                id: "expert".to_string(),
                title: "🏆 Expert".to_string(),
                description: "Generated 1000 commands".to_string(),
                icon: "🏆".to_string(),
                unlock_condition: UnlockCondition::CommandsGenerated { count: 1000 },
                unlocked_at: None,
            },
            Achievement {
                id: "editor".to_string(),
                title: "✏️ Editor".to_string(),
                description: "Edited 10 generated commands".to_string(),
                icon: "✏️".to_string(),
                unlock_condition: UnlockCondition::PatternsEdited { count: 10 },
                unlocked_at: None,
            },
            Achievement {
                id: "perfectionist".to_string(),
                title: "💎 Perfectionist".to_string(),
                description: "Edited 50 commands to perfection".to_string(),
                icon: "💎".to_string(),
                unlock_condition: UnlockCondition::PatternsEdited { count: 50 },
                unlocked_at: None,
            },
            Achievement {
                id: "student".to_string(),
                title: "📚 Student".to_string(),
                description: "Completed 1 tutorial".to_string(),
                icon: "📚".to_string(),
                unlock_condition: UnlockCondition::TutorialsCompleted { count: 1 },
                unlocked_at: None,
            },
            Achievement {
                id: "scholar".to_string(),
                title: "🎓 Scholar".to_string(),
                description: "Completed 5 tutorials".to_string(),
                icon: "🎓".to_string(),
                unlock_condition: UnlockCondition::TutorialsCompleted { count: 5 },
                unlocked_at: None,
            },
            Achievement {
                id: "find_master".to_string(),
                title: "🔍 Find Master".to_string(),
                description: "Used find command successfully".to_string(),
                icon: "🔍".to_string(),
                unlock_condition: UnlockCondition::SpecificCommand {
                    command: "find".to_string(),
                },
                unlocked_at: None,
            },
            Achievement {
                id: "grep_guru".to_string(),
                title: "🔎 Grep Guru".to_string(),
                description: "Used grep command successfully".to_string(),
                icon: "🔎".to_string(),
                unlock_condition: UnlockCondition::SpecificCommand {
                    command: "grep".to_string(),
                },
                unlocked_at: None,
            },
            Achievement {
                id: "docker_captain".to_string(),
                title: "🐳 Docker Captain".to_string(),
                description: "Used docker command successfully".to_string(),
                icon: "🐳".to_string(),
                unlock_condition: UnlockCondition::SpecificCommand {
                    command: "docker".to_string(),
                },
                unlocked_at: None,
            },
        ]
    }

    /// Check for newly unlocked achievements
    pub async fn check_achievements(&self) -> Result<Vec<Achievement>> {
        let mut newly_unlocked = Vec::new();

        for achievement in &self.achievements {
            // Skip if already unlocked
            if self.is_unlocked(&achievement.id).await? {
                continue;
            }

            // Check if condition is met
            if self.check_condition(&achievement.unlock_condition).await? {
                self.unlock(&achievement.id).await?;
                let mut unlocked = achievement.clone();
                unlocked.unlocked_at = Some(Utc::now());
                newly_unlocked.push(unlocked);
            }
        }

        Ok(newly_unlocked)
    }

    /// Check if a specific condition is met
    async fn check_condition(&self, condition: &UnlockCondition) -> Result<bool> {
        match condition {
            UnlockCondition::CommandsGenerated { count } => {
                let total = self.db.count_patterns().await?;
                Ok(total >= *count as usize)
            }
            UnlockCondition::PatternsEdited { count } => {
                let edited = self.db.count_edited_patterns().await?;
                Ok(edited >= *count as usize)
            }
            UnlockCondition::SpecificCommand { command } => {
                let patterns = self.db.get_all_patterns().await?;
                Ok(patterns
                    .iter()
                    .any(|p| p.generated_command.starts_with(command) ||
                           p.final_command.as_ref().map_or(false, |c| c.starts_with(command))))
            }
            UnlockCondition::TutorialsCompleted { .. } => {
                // TODO: Implement tutorial completion tracking
                Ok(false)
            }
            UnlockCondition::DaysStreak { .. } => {
                // TODO: Implement streak tracking
                Ok(false)
            }
            UnlockCondition::SafetyScoreAverage { .. } => {
                // TODO: Implement safety score tracking
                Ok(false)
            }
        }
    }

    /// Unlock an achievement
    pub async fn unlock(&self, achievement_id: &str) -> Result<()> {
        let pool = self.db.pool.clone();
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO unlocked_achievements (id, achievement_id, unlocked_at)
            VALUES (?, ?, ?)
            ON CONFLICT(id) DO NOTHING
            "#,
        )
        .bind(id.to_string())
        .bind(achievement_id)
        .bind(now.to_rfc3339())
        .execute(pool.as_ref())
        .await?;

        Ok(())
    }

    /// Check if achievement is unlocked
    pub async fn is_unlocked(&self, achievement_id: &str) -> Result<bool> {
        let pool = self.db.pool.clone();

        let result: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT achievement_id FROM unlocked_achievements
            WHERE achievement_id = ?
            "#,
        )
        .bind(achievement_id)
        .fetch_optional(pool.as_ref())
        .await?;

        Ok(result.is_some())
    }

    /// Get all unlocked achievements
    pub async fn get_unlocked(&self) -> Result<Vec<Achievement>> {
        let pool = self.db.pool.clone();

        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT achievement_id, unlocked_at FROM unlocked_achievements
            ORDER BY unlocked_at DESC
            "#,
        )
        .fetch_all(pool.as_ref())
        .await?;

        let mut unlocked = Vec::new();
        for (achievement_id, unlocked_at) in rows {
            if let Some(achievement) = self
                .achievements
                .iter()
                .find(|a| a.id == achievement_id)
                .cloned()
            {
                let mut ach = achievement;
                ach.unlocked_at = DateTime::parse_from_rfc3339(&unlocked_at)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc));
                unlocked.push(ach);
            }
        }

        Ok(unlocked)
    }

    /// Get all achievements (including locked ones)
    pub fn get_all(&self) -> Vec<Achievement> {
        self.achievements.clone()
    }

    /// Count unlocked achievements
    pub async fn count_unlocked(&self) -> Result<usize> {
        let pool = self.db.pool.clone();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM unlocked_achievements")
            .fetch_one(pool.as_ref())
            .await?;

        Ok(count.0 as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_achievement_creation() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        let tracker = AchievementTracker::new(db).await.unwrap();

        let all = tracker.get_all();
        assert!(!all.is_empty());
        assert!(all.iter().any(|a| a.id == "first_command"));
    }

    #[tokio::test]
    async fn test_unlock_achievement() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        let tracker = AchievementTracker::new(db).await.unwrap();

        assert!(!tracker.is_unlocked("first_command").await.unwrap());

        tracker.unlock("first_command").await.unwrap();

        assert!(tracker.is_unlocked("first_command").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_unlocked() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        let tracker = AchievementTracker::new(db).await.unwrap();

        tracker.unlock("first_command").await.unwrap();
        tracker.unlock("getting_started").await.unwrap();

        let unlocked = tracker.get_unlocked().await.unwrap();
        assert_eq!(unlocked.len(), 2);
    }
}
