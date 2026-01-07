//! Static Pattern Matcher Backend
//!
//! Provides deterministic command generation for known website-advertised examples.
//! This backend matches natural language patterns to exact shell commands,
//! ensuring consistent output for documented use cases.
//!
//! The static matcher runs BEFORE LLM backends, providing instant, predictable
//! results for common queries advertised on the website.

use async_trait::async_trait;
use regex::Regex;
use std::sync::Arc;

use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel, SafetyLevel, ShellType};
use crate::prompts::CapabilityProfile;

/// Static pattern matcher for deterministic command generation
#[derive(Clone)]
pub struct StaticMatcher {
    patterns: Arc<Vec<PatternEntry>>,
    profile: CapabilityProfile,
}

/// A single pattern entry mapping natural language to shell command
#[derive(Debug, Clone)]
struct PatternEntry {
    /// Keywords that must be present (all required)
    required_keywords: Vec<String>,
    /// Keywords that boost match confidence
    optional_keywords: Vec<String>,
    /// Regex pattern for more precise matching (optional)
    regex_pattern: Option<Regex>,
    /// Command for GNU/Linux systems
    gnu_command: String,
    /// Command for BSD systems (macOS)
    bsd_command: Option<String>,
    /// Description for debugging
    description: String,
}

impl StaticMatcher {
    /// Create a new static matcher with detected capabilities
    pub fn new(profile: CapabilityProfile) -> Self {
        Self {
            patterns: Arc::new(Self::build_patterns()),
            profile,
        }
    }

    /// Build the pattern library from website-advertised examples
    fn build_patterns() -> Vec<PatternEntry> {
        vec![
            // Pattern 1: "list all files modified today"
            PatternEntry {
                required_keywords: vec!["file".to_string(), "modified".to_string(), "today".to_string()],
                optional_keywords: vec!["list".to_string(), "all".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(list|show|find|get).*(files?|file).*(modified|changed|updated).*(today|last 24 hours?)").unwrap()),
                gnu_command: "find . -type f -mtime 0".to_string(),
                bsd_command: Some("find . -type f -mtime 0".to_string()),
                description: "List files modified today".to_string(),
            },

            // Pattern 2: "find large files over 100MB"
            PatternEntry {
                required_keywords: vec!["large".to_string(), "file".to_string(), "100".to_string()],
                optional_keywords: vec!["find".to_string(), "over".to_string(), "mb".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|show|list).*(large|big).*(files?).*(over|above|bigger|greater).*(100|100mb|100m)").unwrap()),
                gnu_command: "find . -type f -size +100M".to_string(),
                bsd_command: Some("find . -type f -size +100M".to_string()),
                description: "Find large files over 100MB".to_string(),
            },

            // Pattern 3: "show disk usage by folder"
            PatternEntry {
                required_keywords: vec!["disk".to_string(), "usage".to_string(), "folder".to_string()],
                optional_keywords: vec!["show".to_string(), "display".to_string(), "by".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|display|list|get).*(disk|space).*(usage|size).*(by |per )?(folder|director)").unwrap()),
                gnu_command: "du -sh */ | sort -rh | head -10".to_string(),
                bsd_command: Some("du -sh */ | sort -rh | head -10".to_string()),
                description: "Show disk usage by folder".to_string(),
            },

            // Pattern 4: "find python files modified last week"
            PatternEntry {
                required_keywords: vec!["python".to_string(), "file".to_string(), "modified".to_string(), "week".to_string()],
                optional_keywords: vec!["find".to_string(), "last".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(python|\.py).*(files?).*(modified|changed|updated).*(last week|past week)").unwrap()),
                gnu_command: "find . -name \"*.py\" -type f -mtime -7".to_string(),
                bsd_command: Some("find . -name \"*.py\" -type f -mtime -7".to_string()),
                description: "Find Python files modified last week".to_string(),
            },
        ]
    }

    /// Try to match the query against known patterns
    fn try_match(&self, query: &str) -> Option<&PatternEntry> {
        let query_lower = query.to_lowercase();

        for pattern in self.patterns.iter() {
            // Check regex pattern first (most precise)
            if let Some(ref regex) = pattern.regex_pattern {
                if regex.is_match(&query_lower) {
                    return Some(pattern);
                }
            }

            // Fallback to keyword matching
            let all_required = pattern.required_keywords.iter()
                .all(|kw| query_lower.contains(kw));

            if all_required {
                // Count optional keywords for confidence boost
                let optional_count = pattern.optional_keywords.iter()
                    .filter(|kw| query_lower.contains(*kw))
                    .count();

                // Require at least some optional keywords for keyword-only match
                if optional_count > 0 || pattern.regex_pattern.is_none() {
                    return Some(pattern);
                }
            }
        }

        None
    }

    /// Select the appropriate command based on platform
    fn select_command(&self, pattern: &PatternEntry) -> String {
        // For now, use GNU commands as default
        // In the future, we can use self.profile to detect platform
        pattern.gnu_command.clone()
    }
}

#[async_trait]
impl CommandGenerator for StaticMatcher {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Try to match the query
        if let Some(pattern) = self.try_match(&request.input) {
            let command = self.select_command(pattern);

            Ok(GeneratedCommand {
                command: command.clone(),
                explanation: format!("Matched pattern: {}", pattern.description),
                safety_level: RiskLevel::Safe,
                estimated_impact: "Read-only query - safe to execute".to_string(),
                alternatives: vec![],
                backend_used: "static-matcher".to_string(),
                generation_time_ms: 0, // Instant - no LLM call
                confidence_score: 1.0,  // Deterministic match
            })
        } else {
            // No match - return error so we can fall through to LLM
            Err(GeneratorError::BackendUnavailable {
                reason: "No static pattern match found".to_string(),
            })
        }
    }

    async fn is_available(&self) -> bool {
        // Static matcher is always available
        true
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Embedded,
            model_name: "static-matcher".to_string(),
            supports_streaming: false,
            max_tokens: 0,
            typical_latency_ms: 0,
            memory_usage_mb: 1,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        // Nothing to clean up
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_website_example_1() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("list all files modified today", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.command, "find . -type f -mtime 0");
        assert_eq!(cmd.safety_level, RiskLevel::Safe);
    }

    #[tokio::test]
    async fn test_website_example_2() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("find large files over 100MB", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.command, "find . -type f -size +100M");
    }

    #[tokio::test]
    async fn test_website_example_3() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("show disk usage by folder", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.command, "du -sh */ | sort -rh | head -10");
    }

    #[tokio::test]
    async fn test_website_example_4() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("find python files modified last week", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.command, "find . -name \"*.py\" -type f -mtime -7");
    }

    #[tokio::test]
    async fn test_variant_phrasing() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        // Should still match with different phrasing
        let request = CommandRequest::new("show me all files that were modified today", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_match() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("compile my rust project", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_err());
    }
}
