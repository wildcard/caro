//! Search functionality for command history
//! 
//! Provides full-text and semantic search capabilities with
//! performance optimizations for constitutional compliance.

use super::models::CommandHistoryEntry;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Search query for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search text
    pub text: String,
    
    /// Optional filters to apply
    pub filters: Option<SearchFilters>,
    
    /// Similarity threshold for semantic search
    pub similarity_threshold: Option<f64>,
    
    /// Maximum results to return
    pub max_results: Option<usize>,
}

impl SearchQuery {
    /// Create a new search query
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            filters: None,
            similarity_threshold: None,
            max_results: None,
        }
    }
    
    /// Add filters to the query
    pub fn with_filters(mut self, filters: SearchFilters) -> Self {
        self.filters = Some(filters);
        self
    }
    
    /// Set similarity threshold for semantic search
    pub fn with_similarity_threshold(mut self, threshold: f64) -> Self {
        self.similarity_threshold = Some(threshold);
        self
    }
    
    /// Set maximum results
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }
}

/// Search filters for refining results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    
    /// Filter by working directory
    pub working_directory: Option<String>,
    
    /// Filter by shell type
    pub shell_type: Option<String>,
    
    /// Filter by date range
    pub date_range: Option<DateRange>,
    
    /// Filter by risk level
    pub risk_level: Option<String>,
}

impl SearchFilters {
    /// Create new search filters
    pub fn new() -> Self {
        Self {
            tags: None,
            working_directory: None,
            shell_type: None,
            date_range: None,
            risk_level: None,
        }
    }
    
    /// Filter by tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
    
    /// Filter by working directory
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }
    
    /// Filter by shell type
    pub fn with_shell_type(mut self, shell: impl Into<String>) -> Self {
        self.shell_type = Some(shell.into());
        self
    }
    
    /// Filter by date range
    pub fn with_date_range(mut self, range: DateRange) -> Self {
        self.date_range = Some(range);
        self
    }
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self::new()
    }
}

/// Date range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date (inclusive)
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    
    /// End date (inclusive)
    pub to: Option<chrono::DateTime<chrono::Utc>>,
}

impl DateRange {
    /// Create a date range
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
        }
    }
    
    /// Set start date
    pub fn from(mut self, date: chrono::DateTime<chrono::Utc>) -> Self {
        self.from = Some(date);
        self
    }
    
    /// Set end date
    pub fn to(mut self, date: chrono::DateTime<chrono::Utc>) -> Self {
        self.to = Some(date);
        self
    }
    
    /// Create a range for today
    pub fn today() -> Self {
        let today = chrono::Utc::now().date_naive();
        let start = today.and_hms_opt(0, 0, 0).unwrap();
        let end = today.and_hms_opt(23, 59, 59).unwrap();
        
        Self {
            from: Some(start.and_utc()),
            to: Some(end.and_utc()),
        }
    }
    
    /// Create a range for last N days
    pub fn last_days(days: i64) -> Self {
        let now = chrono::Utc::now();
        let start = now - chrono::Duration::days(days);
        
        Self {
            from: Some(start),
            to: Some(now),
        }
    }
}

impl Default for DateRange {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching history entry
    pub entry: CommandHistoryEntry,
    
    /// Similarity score (0.0 to 1.0)
    pub similarity_score: f64,
    
    /// Context of the match
    pub matching_context: String,
    
    /// Explanation of why this matched
    pub explanation: String,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(entry: CommandHistoryEntry, score: f64) -> Self {
        Self {
            entry,
            similarity_score: score,
            matching_context: String::new(),
            explanation: String::new(),
        }
    }
    
    /// Add matching context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.matching_context = context.into();
        self
    }
    
    /// Add explanation
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (score: {:.2}): {}",
            self.entry.timestamp.format("%Y-%m-%d %H:%M"),
            self.similarity_score,
            self.entry.command
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("find rust files")
            .with_similarity_threshold(0.8)
            .with_max_results(10);
        
        assert_eq!(query.text, "find rust files");
        assert_eq!(query.similarity_threshold, Some(0.8));
        assert_eq!(query.max_results, Some(10));
    }
    
    #[test]
    fn test_search_filters() {
        let filters = SearchFilters::new()
            .with_tags(vec!["rust".to_string(), "search".to_string()])
            .with_working_directory("/home/user/project");
        
        assert_eq!(filters.tags.unwrap().len(), 2);
        assert_eq!(filters.working_directory.unwrap(), "/home/user/project");
    }
    
    #[test]
    fn test_date_range() {
        let range = DateRange::last_days(7);
        
        assert!(range.from.is_some());
        assert!(range.to.is_some());
        assert!(range.from.unwrap() < range.to.unwrap());
    }
}