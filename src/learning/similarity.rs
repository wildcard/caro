//! Similarity search for finding related past commands
//!
//! Phase 1: Keyword-based matching
//! Phase 2 (future): Embedding-based semantic search

use crate::learning::pattern_db::{CommandPattern, PatternDB};
use anyhow::Result;
use std::collections::HashMap;

/// Similarity search engine
pub struct SimilaritySearch {
    db: PatternDB,
}

impl SimilaritySearch {
    /// Create a new similarity search engine
    pub fn new(db: PatternDB) -> Self {
        Self { db }
    }

    /// Find similar commands using keyword matching
    pub async fn find_similar(&self, prompt: &str, k: usize) -> Result<Vec<CommandPattern>> {
        // Get all patterns
        let all_patterns = self.db.get_all_patterns().await?;

        if all_patterns.is_empty() {
            return Ok(vec![]);
        }

        // Calculate similarity scores
        let mut scores: Vec<(CommandPattern, f32)> = all_patterns
            .into_iter()
            .map(|pattern| {
                let score = self.calculate_similarity(prompt, &pattern.user_prompt);
                (pattern, score)
            })
            .collect();

        // Sort by score (highest first)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top-k
        Ok(scores
            .into_iter()
            .take(k)
            .filter(|(_, score)| *score > 0.0) // Only return patterns with positive similarity
            .map(|(pattern, _)| pattern)
            .collect())
    }

    /// Calculate similarity between two prompts using keyword overlap
    fn calculate_similarity(&self, prompt1: &str, prompt2: &str) -> f32 {
        let words1 = self.extract_keywords(prompt1);
        let words2 = self.extract_keywords(prompt2);

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        // Calculate Jaccard similarity
        let intersection: usize = words1.iter().filter(|w| words2.contains(w)).count();
        let union_size = words1.len() + words2.len() - intersection;

        if union_size == 0 {
            return 0.0;
        }

        intersection as f32 / union_size as f32
    }

    /// Extract keywords from prompt
    fn extract_keywords(&self, prompt: &str) -> Vec<String> {
        let stop_words = self.get_stop_words();

        prompt
            .to_lowercase()
            .split_whitespace()
            .filter(|word| !stop_words.contains(word))
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|word| !word.is_empty())
            .collect()
    }

    /// Get common stop words to ignore
    fn get_stop_words(&self) -> Vec<&'static str> {
        vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "as", "is", "are", "was", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "should", "could", "may", "might",
            "must", "can", "all", "my", "me", "i",
        ]
    }

    /// Find patterns by exact command match
    pub async fn find_by_command(&self, command: &str) -> Result<Vec<CommandPattern>> {
        let all_patterns = self.db.get_all_patterns().await?;

        Ok(all_patterns
            .into_iter()
            .filter(|p| {
                p.generated_command == command
                    || p.final_command.as_ref() == Some(&command.to_string())
            })
            .collect())
    }

    /// Get popular commands (most frequently generated)
    pub async fn get_popular_commands(&self, limit: usize) -> Result<Vec<(String, usize)>> {
        let all_patterns = self.db.get_all_patterns().await?;

        let mut command_counts: HashMap<String, usize> = HashMap::new();

        for pattern in all_patterns {
            let cmd = pattern.final_command.unwrap_or(pattern.generated_command);
            *command_counts.entry(cmd).or_insert(0) += 1;
        }

        let mut popular: Vec<(String, usize)> = command_counts.into_iter().collect();
        popular.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(popular.into_iter().take(limit).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_extract_keywords() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        let search = SimilaritySearch::new(db);

        let keywords = search.extract_keywords("find all log files in the directory");

        assert!(keywords.contains(&"find".to_string()));
        assert!(keywords.contains(&"log".to_string()));
        assert!(keywords.contains(&"files".to_string()));
        assert!(!keywords.contains(&"the".to_string())); // Stop word
    }

    #[tokio::test]
    async fn test_calculate_similarity() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        let search = SimilaritySearch::new(db);

        let sim1 = search.calculate_similarity(
            "find all log files",
            "find log files",
        );
        assert!(sim1 > 0.5);

        let sim2 = search.calculate_similarity(
            "find log files",
            "delete temporary files",
        );
        assert!(sim2 < sim1);
    }

    #[tokio::test]
    async fn test_find_similar() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();

        // Add some patterns
        let patterns = vec![
            ("find log files", "find . -name '*.log'"),
            ("find all logs", "find . -type f -name '*.log'"),
            ("delete temporary files", "rm -rf /tmp/*.tmp"),
        ];

        for (prompt, cmd) in patterns {
            let pattern = CommandPattern {
                id: Uuid::new_v4(),
                user_prompt: prompt.to_string(),
                generated_command: cmd.to_string(),
                final_command: None,
                context_snapshot: serde_json::json!({}),
                execution_success: None,
                user_rating: None,
                timestamp: Utc::now(),
            };
            db.record_interaction(pattern).await.unwrap();
        }

        let search = SimilaritySearch::new(db);
        let similar = search.find_similar("find log files", 2).await.unwrap();

        assert!(!similar.is_empty());
        assert!(similar[0].user_prompt.contains("log"));
    }
}
