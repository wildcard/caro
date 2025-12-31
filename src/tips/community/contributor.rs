//! Contributor attribution for community contributions
//!
//! Tracks contributor information and provides attribution in tips.

use serde::{Deserialize, Serialize};

/// Contributor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    /// GitHub username
    pub github_username: String,

    /// Display name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Number of contributions
    #[serde(default)]
    pub contribution_count: u32,

    /// First contribution date (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_contribution: Option<i64>,

    /// Whether this contributor is a maintainer
    #[serde(default)]
    pub is_maintainer: bool,
}

impl Contributor {
    /// Create a new contributor
    pub fn new(github_username: impl Into<String>) -> Self {
        Self {
            github_username: github_username.into(),
            display_name: None,
            contribution_count: 0,
            first_contribution: None,
            is_maintainer: false,
        }
    }

    /// Set the display name
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Mark as maintainer
    pub fn as_maintainer(mut self) -> Self {
        self.is_maintainer = true;
        self
    }

    /// Get the display name or github username
    pub fn name(&self) -> &str {
        self.display_name
            .as_deref()
            .unwrap_or(&self.github_username)
    }

    /// Get GitHub profile URL
    pub fn github_url(&self) -> String {
        format!("https://github.com/{}", self.github_username)
    }

    /// Increment contribution count
    pub fn add_contribution(&mut self) {
        self.contribution_count += 1;
        if self.first_contribution.is_none() {
            self.first_contribution = Some(chrono::Utc::now().timestamp());
        }
    }
}

/// Attribution information for a tip or alias
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorAttribution {
    /// Contributor GitHub username
    pub contributor: String,

    /// Source cheatsheet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_cheatsheet: Option<String>,

    /// Date added (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_added: Option<i64>,

    /// License information
    #[serde(default = "default_license")]
    pub license: String,
}

fn default_license() -> String {
    "MIT".to_string()
}

impl ContributorAttribution {
    /// Create a new attribution
    pub fn new(contributor: impl Into<String>) -> Self {
        Self {
            contributor: contributor.into(),
            source_cheatsheet: None,
            date_added: Some(chrono::Utc::now().timestamp()),
            license: default_license(),
        }
    }

    /// Set the source cheatsheet
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source_cheatsheet = Some(source.into());
        self
    }

    /// Set the license
    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = license.into();
        self
    }

    /// Format attribution for display
    pub fn format(&self) -> String {
        let mut parts = vec![format!("Contributed by @{}", self.contributor)];

        if let Some(ref source) = self.source_cheatsheet {
            parts.push(format!("from {}", source));
        }

        parts.join(" ")
    }

    /// Format as compact attribution line
    pub fn format_compact(&self) -> String {
        format!("by @{}", self.contributor)
    }
}

/// Contributor leaderboard
#[derive(Debug, Clone, Default)]
pub struct ContributorLeaderboard {
    /// Contributors sorted by contribution count
    contributors: Vec<Contributor>,
}

impl ContributorLeaderboard {
    /// Create a new empty leaderboard
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a contributor
    pub fn add_contribution(&mut self, github_username: &str) {
        if let Some(contributor) = self
            .contributors
            .iter_mut()
            .find(|c| c.github_username == github_username)
        {
            contributor.add_contribution();
        } else {
            let mut contributor = Contributor::new(github_username);
            contributor.add_contribution();
            self.contributors.push(contributor);
        }

        // Re-sort by contribution count
        self.contributors
            .sort_by(|a, b| b.contribution_count.cmp(&a.contribution_count));
    }

    /// Get top contributors
    pub fn top(&self, n: usize) -> &[Contributor] {
        &self.contributors[..n.min(self.contributors.len())]
    }

    /// Get total contributor count
    pub fn count(&self) -> usize {
        self.contributors.len()
    }

    /// Get total contribution count
    pub fn total_contributions(&self) -> u32 {
        self.contributors.iter().map(|c| c.contribution_count).sum()
    }

    /// Find a specific contributor
    pub fn find(&self, github_username: &str) -> Option<&Contributor> {
        self.contributors
            .iter()
            .find(|c| c.github_username == github_username)
    }

    /// Get contributor's rank (1-indexed)
    pub fn rank(&self, github_username: &str) -> Option<usize> {
        self.contributors
            .iter()
            .position(|c| c.github_username == github_username)
            .map(|i| i + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contributor_creation() {
        let contributor = Contributor::new("testuser")
            .with_display_name("Test User")
            .as_maintainer();

        assert_eq!(contributor.github_username, "testuser");
        assert_eq!(contributor.name(), "Test User");
        assert!(contributor.is_maintainer);
    }

    #[test]
    fn test_contributor_github_url() {
        let contributor = Contributor::new("testuser");
        assert_eq!(contributor.github_url(), "https://github.com/testuser");
    }

    #[test]
    fn test_add_contribution() {
        let mut contributor = Contributor::new("testuser");
        assert_eq!(contributor.contribution_count, 0);
        assert!(contributor.first_contribution.is_none());

        contributor.add_contribution();
        assert_eq!(contributor.contribution_count, 1);
        assert!(contributor.first_contribution.is_some());

        contributor.add_contribution();
        assert_eq!(contributor.contribution_count, 2);
    }

    #[test]
    fn test_attribution_format() {
        let attribution = ContributorAttribution::new("testuser").with_source("git-cheatsheet");

        let formatted = attribution.format();
        assert!(formatted.contains("@testuser"));
        assert!(formatted.contains("git-cheatsheet"));
    }

    #[test]
    fn test_leaderboard() {
        let mut leaderboard = ContributorLeaderboard::new();

        leaderboard.add_contribution("user1");
        leaderboard.add_contribution("user2");
        leaderboard.add_contribution("user1");
        leaderboard.add_contribution("user1");

        assert_eq!(leaderboard.count(), 2);
        assert_eq!(leaderboard.total_contributions(), 4);

        // user1 should be first (3 contributions)
        let top = leaderboard.top(2);
        assert_eq!(top[0].github_username, "user1");
        assert_eq!(top[0].contribution_count, 3);
        assert_eq!(top[1].github_username, "user2");
        assert_eq!(top[1].contribution_count, 1);

        // Check rank
        assert_eq!(leaderboard.rank("user1"), Some(1));
        assert_eq!(leaderboard.rank("user2"), Some(2));
    }
}
