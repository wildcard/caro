//! Submission handling for community contributions
//!
//! Manages the lifecycle of community submissions from creation
//! through moderation and approval.

use super::contributor::ContributorAttribution;
use super::schema::{SchemaValidator, ValidationResult};
use crate::tips::kb::Cheatsheet;
use serde::{Deserialize, Serialize};

/// A community submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    /// Unique submission ID
    pub id: String,

    /// Contributor attribution
    pub attribution: ContributorAttribution,

    /// The cheatsheet content
    pub cheatsheet: Cheatsheet,

    /// Current status
    pub status: SubmissionStatus,

    /// Submission timestamp (Unix)
    pub submitted_at: i64,

    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,

    /// Moderator notes
    #[serde(default)]
    pub notes: Vec<SubmissionNote>,

    /// GitHub issue number (if created)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_issue: Option<u64>,

    /// GitHub PR number (if created)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_pr: Option<u64>,
}

impl Submission {
    /// Create a new submission
    pub fn new(contributor: &str, cheatsheet: Cheatsheet) -> Self {
        let id = format!(
            "sub-{}-{}",
            contributor,
            chrono::Utc::now().timestamp_millis()
        );

        Self {
            id,
            attribution: ContributorAttribution::new(contributor),
            cheatsheet,
            status: SubmissionStatus::Pending,
            submitted_at: chrono::Utc::now().timestamp(),
            updated_at: None,
            notes: Vec::new(),
            github_issue: None,
            github_pr: None,
        }
    }

    /// Validate the submission
    pub fn validate(&self) -> ValidationResult {
        let validator = SchemaValidator::new();
        let yaml = serde_yaml::to_string(&self.cheatsheet).unwrap_or_default();
        validator.validate_yaml(&yaml)
    }

    /// Add a moderator note
    pub fn add_note(&mut self, author: &str, message: &str) {
        self.notes.push(SubmissionNote {
            author: author.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        });
        self.updated_at = Some(chrono::Utc::now().timestamp());
    }

    /// Transition to a new status
    pub fn transition(&mut self, new_status: SubmissionStatus) -> Result<(), StatusTransitionError> {
        // Validate transition
        if !self.status.can_transition_to(&new_status) {
            return Err(StatusTransitionError {
                from: self.status,
                to: new_status,
            });
        }

        self.status = new_status;
        self.updated_at = Some(chrono::Utc::now().timestamp());
        Ok(())
    }

    /// Check if submission is actionable (pending review)
    pub fn is_pending(&self) -> bool {
        matches!(self.status, SubmissionStatus::Pending | SubmissionStatus::InReview)
    }

    /// Get age in seconds
    pub fn age_seconds(&self) -> i64 {
        chrono::Utc::now().timestamp() - self.submitted_at
    }

    /// Format submission summary
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} by @{}: {} aliases, {} tips ({})",
            self.id,
            self.cheatsheet.name,
            self.attribution.contributor,
            self.cheatsheet.aliases.len(),
            self.cheatsheet.tips.len(),
            self.status,
        )
    }
}

/// Submission status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionStatus {
    /// Awaiting review
    #[default]
    Pending,
    /// Under review by moderator
    InReview,
    /// Changes requested
    ChangesRequested,
    /// Approved and awaiting merge
    Approved,
    /// Merged to knowledge base
    Merged,
    /// Rejected
    Rejected,
    /// Withdrawn by contributor
    Withdrawn,
}

impl SubmissionStatus {
    /// Check if transition to new status is valid
    pub fn can_transition_to(&self, new: &Self) -> bool {
        match (self, new) {
            // Pending can go to InReview, Rejected, or Withdrawn
            (Self::Pending, Self::InReview) => true,
            (Self::Pending, Self::Rejected) => true,
            (Self::Pending, Self::Withdrawn) => true,

            // InReview can go to Approved, ChangesRequested, or Rejected
            (Self::InReview, Self::Approved) => true,
            (Self::InReview, Self::ChangesRequested) => true,
            (Self::InReview, Self::Rejected) => true,

            // ChangesRequested can go back to InReview or be Withdrawn
            (Self::ChangesRequested, Self::InReview) => true,
            (Self::ChangesRequested, Self::Withdrawn) => true,

            // Approved can be Merged
            (Self::Approved, Self::Merged) => true,

            // Terminal states
            (Self::Merged, _) => false,
            (Self::Rejected, _) => false,
            (Self::Withdrawn, _) => false,

            _ => false,
        }
    }

    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Merged | Self::Rejected | Self::Withdrawn)
    }

    /// Get all valid next states
    pub fn valid_transitions(&self) -> Vec<Self> {
        let all_states = [
            Self::Pending,
            Self::InReview,
            Self::ChangesRequested,
            Self::Approved,
            Self::Merged,
            Self::Rejected,
            Self::Withdrawn,
        ];

        all_states
            .iter()
            .filter(|s| self.can_transition_to(s))
            .copied()
            .collect()
    }
}

impl std::fmt::Display for SubmissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::InReview => write!(f, "In Review"),
            Self::ChangesRequested => write!(f, "Changes Requested"),
            Self::Approved => write!(f, "Approved"),
            Self::Merged => write!(f, "Merged"),
            Self::Rejected => write!(f, "Rejected"),
            Self::Withdrawn => write!(f, "Withdrawn"),
        }
    }
}

/// A note on a submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionNote {
    /// Author of the note
    pub author: String,
    /// Note content
    pub message: String,
    /// Timestamp
    pub timestamp: i64,
}

/// Error for invalid status transitions
#[derive(Debug, Clone)]
pub struct StatusTransitionError {
    pub from: SubmissionStatus,
    pub to: SubmissionStatus,
}

impl std::fmt::Display for StatusTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid status transition: {} -> {}",
            self.from, self.to
        )
    }
}

impl std::error::Error for StatusTransitionError {}

/// Format for submission data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SubmissionFormat {
    /// YAML format
    #[default]
    Yaml,
    /// JSON format
    Json,
    /// MessagePack format (binary)
    MessagePack,
}

impl SubmissionFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::MessagePack => "msgpack",
        }
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Yaml => "application/x-yaml",
            Self::Json => "application/json",
            Self::MessagePack => "application/x-msgpack",
        }
    }
}

/// Queue for managing pending submissions
#[derive(Debug, Clone, Default)]
pub struct SubmissionQueue {
    submissions: Vec<Submission>,
}

impl SubmissionQueue {
    /// Create a new empty queue
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a submission to the queue
    pub fn add(&mut self, submission: Submission) {
        self.submissions.push(submission);
    }

    /// Get all pending submissions
    pub fn pending(&self) -> Vec<&Submission> {
        self.submissions
            .iter()
            .filter(|s| s.is_pending())
            .collect()
    }

    /// Get submissions by status
    pub fn by_status(&self, status: SubmissionStatus) -> Vec<&Submission> {
        self.submissions
            .iter()
            .filter(|s| s.status == status)
            .collect()
    }

    /// Get submission by ID
    pub fn get(&self, id: &str) -> Option<&Submission> {
        self.submissions.iter().find(|s| s.id == id)
    }

    /// Get mutable submission by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Submission> {
        self.submissions.iter_mut().find(|s| s.id == id)
    }

    /// Get queue statistics
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            total: self.submissions.len(),
            pending: self.submissions.iter().filter(|s| s.status == SubmissionStatus::Pending).count(),
            in_review: self.submissions.iter().filter(|s| s.status == SubmissionStatus::InReview).count(),
            approved: self.submissions.iter().filter(|s| s.status == SubmissionStatus::Approved).count(),
            merged: self.submissions.iter().filter(|s| s.status == SubmissionStatus::Merged).count(),
            rejected: self.submissions.iter().filter(|s| s.status == SubmissionStatus::Rejected).count(),
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total: usize,
    pub pending: usize,
    pub in_review: usize,
    pub approved: usize,
    pub merged: usize,
    pub rejected: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tips::kb::Cheatsheet;

    fn test_cheatsheet() -> Cheatsheet {
        Cheatsheet {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_submission_creation() {
        let submission = Submission::new("testuser", test_cheatsheet());

        assert!(submission.id.starts_with("sub-testuser-"));
        assert_eq!(submission.status, SubmissionStatus::Pending);
        assert_eq!(submission.attribution.contributor, "testuser");
    }

    #[test]
    fn test_status_transitions() {
        let mut submission = Submission::new("testuser", test_cheatsheet());

        // Pending -> InReview
        assert!(submission.transition(SubmissionStatus::InReview).is_ok());
        assert_eq!(submission.status, SubmissionStatus::InReview);

        // InReview -> Approved
        assert!(submission.transition(SubmissionStatus::Approved).is_ok());
        assert_eq!(submission.status, SubmissionStatus::Approved);

        // Approved -> Merged
        assert!(submission.transition(SubmissionStatus::Merged).is_ok());
        assert_eq!(submission.status, SubmissionStatus::Merged);

        // Merged is terminal
        assert!(submission.transition(SubmissionStatus::Pending).is_err());
    }

    #[test]
    fn test_invalid_transition() {
        let mut submission = Submission::new("testuser", test_cheatsheet());

        // Pending -> Merged is invalid (skips steps)
        assert!(submission.transition(SubmissionStatus::Merged).is_err());
    }

    #[test]
    fn test_add_note() {
        let mut submission = Submission::new("testuser", test_cheatsheet());
        submission.add_note("moderator", "Looks good!");

        assert_eq!(submission.notes.len(), 1);
        assert_eq!(submission.notes[0].author, "moderator");
        assert!(submission.updated_at.is_some());
    }

    #[test]
    fn test_submission_queue() {
        let mut queue = SubmissionQueue::new();

        queue.add(Submission::new("user1", test_cheatsheet()));
        queue.add(Submission::new("user2", test_cheatsheet()));

        assert_eq!(queue.pending().len(), 2);

        // Approve one
        if let Some(sub) = queue.get_mut("sub-user1-") {
            // Note: ID won't match exactly, so let's use stats
        }

        let stats = queue.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.pending, 2);
    }

    #[test]
    fn test_submission_format() {
        assert_eq!(SubmissionFormat::Yaml.extension(), "yaml");
        assert_eq!(SubmissionFormat::Json.extension(), "json");
        assert_eq!(SubmissionFormat::MessagePack.extension(), "msgpack");
    }

    #[test]
    fn test_valid_transitions() {
        let pending = SubmissionStatus::Pending;
        let transitions = pending.valid_transitions();

        assert!(transitions.contains(&SubmissionStatus::InReview));
        assert!(transitions.contains(&SubmissionStatus::Rejected));
        assert!(transitions.contains(&SubmissionStatus::Withdrawn));
        assert!(!transitions.contains(&SubmissionStatus::Merged));
    }
}
