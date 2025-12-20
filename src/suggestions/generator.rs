//! Suggestion generator - combines analysis to produce relevant queries

use super::{
    defaults, profile::UserProfile, QueryCategory, Result, SuggestedQuery,
    SuggestionsConfig,
};

/// Generates suggestions based on user profile and context
pub struct SuggestionGenerator {
    config: SuggestionsConfig,
}

impl Default for SuggestionGenerator {
    fn default() -> Self {
        Self::new(SuggestionsConfig::default())
    }
}

impl SuggestionGenerator {
    /// Create a new suggestion generator
    pub fn new(config: SuggestionsConfig) -> Self {
        Self { config }
    }

    /// Generate suggestions based on user profile
    pub fn generate(&self, profile: &UserProfile) -> Result<Vec<SuggestedQuery>> {
        let mut all_suggestions = Vec::new();

        // Check experience level and add appropriate suggestions
        match profile.experience_level {
            super::profile::ExperienceLevel::Beginner => {
                // Prioritize learning suggestions
                all_suggestions.extend(defaults::get_beginner_suggestions());
            }
            _ => {
                // For intermediate/advanced users, focus on context

                // Git suggestions (Caro loves Git!)
                if let Some(git_state) = &profile.environment_insights.git_state {
                    all_suggestions.extend(defaults::get_git_suggestions(git_state));
                }

                // Project-based suggestions
                if let Some(project_type) = &profile.environment_insights.project_type {
                    all_suggestions.extend(defaults::get_project_suggestions(project_type));
                }

                // History-based suggestions
                let top_cmds: Vec<(String, u32)> = profile
                    .command_patterns
                    .top_commands
                    .iter()
                    .cloned()
                    .collect();
                all_suggestions.extend(defaults::get_history_based_suggestions(&top_cmds));

                // Tool-based suggestions
                if profile.has_tool("docker") {
                    all_suggestions.extend(defaults::get_docker_suggestions());
                }
            }
        }

        // Filter by minimum relevance
        let mut suggestions: Vec<SuggestedQuery> = all_suggestions
            .into_iter()
            .filter(|s| s.relevance >= self.config.min_relevance)
            .collect();

        // Sort by relevance (descending)
        suggestions.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());

        // Deduplicate similar suggestions
        suggestions = self.deduplicate(suggestions);

        // Ensure variety in categories
        suggestions = self.ensure_variety(suggestions);

        // Limit to max suggestions
        suggestions.truncate(self.config.max_suggestions);

        Ok(suggestions)
    }

    /// Generate default suggestions without any profile (cold start)
    pub fn generate_defaults(&self) -> Vec<SuggestedQuery> {
        let mut suggestions = defaults::get_beginner_suggestions();

        // Add some general git suggestions if in a git repo
        if std::path::Path::new(".git").exists() {
            suggestions.push(
                SuggestedQuery::new(
                    "show git status",
                    "See current branch and changes",
                    super::SuggestionReason::GitWorkflow {
                        state: "git repository".to_string(),
                    },
                    QueryCategory::Git,
                )
                .with_relevance(0.95),
            );
        }

        suggestions.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        suggestions.truncate(self.config.max_suggestions);
        suggestions
    }

    /// Remove similar/duplicate suggestions
    fn deduplicate(&self, suggestions: Vec<SuggestedQuery>) -> Vec<SuggestedQuery> {
        let mut seen_queries: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut result = Vec::new();

        for suggestion in suggestions {
            // Normalize query for comparison
            let normalized = suggestion.query.to_lowercase();

            // Check for similar queries (simple word-based similarity)
            let words: std::collections::HashSet<&str> = normalized.split_whitespace().collect();
            let is_similar = seen_queries.iter().any(|seen| {
                let seen_words: std::collections::HashSet<&str> =
                    seen.split_whitespace().collect();
                let intersection: usize = words.intersection(&seen_words).count();
                let union: usize = words.union(&seen_words).count();
                if union == 0 {
                    return false;
                }
                (intersection as f32 / union as f32) > 0.7
            });

            if !is_similar {
                seen_queries.insert(normalized);
                result.push(suggestion);
            }
        }

        result
    }

    /// Ensure variety in suggestion categories
    fn ensure_variety(&self, suggestions: Vec<SuggestedQuery>) -> Vec<SuggestedQuery> {
        use std::collections::HashMap;

        let max_per_category = 3;
        let mut category_counts: HashMap<QueryCategory, usize> = HashMap::new();
        let mut result = Vec::new();

        for suggestion in suggestions {
            let count = category_counts.entry(suggestion.category).or_insert(0);
            if *count < max_per_category {
                *count += 1;
                result.push(suggestion);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggestions::{
        environment::EnvironmentInsights, history::CommandPatterns, profile::ExperienceLevel,
    };

    fn make_test_profile(experience: ExperienceLevel) -> UserProfile {
        UserProfile {
            version: "1.0.0".to_string(),
            last_analyzed: chrono::Utc::now(),
            experience_level: experience,
            workflows: Vec::new(),
            command_patterns: CommandPatterns::default(),
            detected_tools: Vec::new(),
            environment_insights: EnvironmentInsights::default(),
        }
    }

    #[test]
    fn test_generator_beginner() {
        let generator = SuggestionGenerator::default();
        let profile = make_test_profile(ExperienceLevel::Beginner);

        let suggestions = generator.generate(&profile).unwrap();

        assert!(!suggestions.is_empty());
        // Beginners should get learning suggestions
        assert!(suggestions
            .iter()
            .any(|s| s.category == QueryCategory::Learning
                || s.category == QueryCategory::FileOperations));
    }

    #[test]
    fn test_generator_defaults() {
        let generator = SuggestionGenerator::default();
        let suggestions = generator.generate_defaults();

        assert!(!suggestions.is_empty());
        assert!(suggestions.len() <= 5);
    }

    #[test]
    fn test_deduplication() {
        let generator = SuggestionGenerator::default();

        let suggestions = vec![
            SuggestedQuery::new(
                "list files in directory",
                "desc",
                super::super::SuggestionReason::NewUserOnboarding,
                QueryCategory::FileOperations,
            ),
            SuggestedQuery::new(
                "list files in this directory",
                "desc",
                super::super::SuggestionReason::NewUserOnboarding,
                QueryCategory::FileOperations,
            ),
            SuggestedQuery::new(
                "show git status",
                "desc",
                super::super::SuggestionReason::NewUserOnboarding,
                QueryCategory::Git,
            ),
        ];

        let deduped = generator.deduplicate(suggestions);

        // Similar "list files" suggestions should be merged
        assert!(deduped.len() < 3);
    }

    #[test]
    fn test_variety() {
        let generator = SuggestionGenerator::default();

        // Create many suggestions in the same category
        let suggestions: Vec<SuggestedQuery> = (0..10)
            .map(|i| {
                SuggestedQuery::new(
                    format!("git command {}", i),
                    "desc",
                    super::super::SuggestionReason::NewUserOnboarding,
                    QueryCategory::Git,
                )
            })
            .collect();

        let varied = generator.ensure_variety(suggestions);

        // Should limit to max per category (3)
        assert!(varied.len() <= 3);
    }
}
