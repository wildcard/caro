//! GitHub API Client for Feedback System
//!
//! This module provides a client for creating GitHub issues from feedback
//! submissions. It handles:
//!
//! - Authentication with GitHub personal access tokens
//! - Issue creation with proper formatting
//! - Rate limiting and error handling
//! - Issue template formatting

use crate::feedback::types::*;
use crate::feedback::FeedbackError;
use reqwest::Client;
use serde::Deserialize;

// =============================================================================
// GitHub API Client
// =============================================================================

/// GitHub API client for creating issues
pub struct GitHubClient {
    client: Client,
    token: String,
    repo_owner: String,
    repo_name: String,
    base_url: String,
}

impl std::fmt::Debug for GitHubClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitHubClient")
            .field("repo_owner", &self.repo_owner)
            .field("repo_name", &self.repo_name)
            .field("base_url", &self.base_url)
            .field("token", &"[REDACTED]")
            .finish()
    }
}

/// GitHub API response for issue creation
#[derive(Debug, Deserialize)]
struct GitHubIssueResponse {
    html_url: String,
    #[allow(dead_code)]
    number: u64,
}

impl GitHubClient {
    /// Create a new GitHub client
    ///
    /// # Arguments
    /// * `token` - GitHub personal access token with `repo` or `public_repo` scope
    /// * `repo_owner` - Repository owner (username or organization)
    /// * `repo_name` - Repository name
    ///
    /// # Returns
    /// Result containing the client or an error
    pub fn new(token: String, repo_owner: String, repo_name: String) -> Result<Self, FeedbackError> {
        if token.is_empty() {
            return Err(FeedbackError::ConfigError(
                "GitHub token cannot be empty".to_string(),
            ));
        }

        let client = Client::builder()
            .user_agent("cmdai-feedback/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| FeedbackError::NetworkError(e.to_string()))?;

        Ok(Self {
            client,
            token,
            repo_owner,
            repo_name,
            base_url: "https://api.github.com".to_string(),
        })
    }

    /// Create a new client with a custom base URL (for testing)
    #[cfg(test)]
    pub fn with_base_url(
        token: String,
        repo_owner: String,
        repo_name: String,
        base_url: String,
    ) -> Result<Self, FeedbackError> {
        let mut client = Self::new(token, repo_owner, repo_name)?;
        client.base_url = base_url;
        Ok(client)
    }

    /// Create a new GitHub issue from feedback
    ///
    /// # Arguments
    /// * `feedback` - The feedback to create an issue for
    ///
    /// # Returns
    /// Result containing the issue URL or an error
    pub async fn create_issue(&self, feedback: &Feedback) -> Result<String, FeedbackError> {
        let url = format!(
            "{}/repos/{}/{}/issues",
            self.base_url, self.repo_owner, self.repo_name
        );

        let issue = GitHubIssueRequest {
            title: format_issue_title(feedback),
            body: format_issue_body(feedback),
            labels: get_issue_labels(feedback),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&issue)
            .send()
            .await
            .map_err(|e| FeedbackError::NetworkError(e.to_string()))?;

        let status = response.status();

        if status.is_success() {
            let issue_response: GitHubIssueResponse = response
                .json()
                .await
                .map_err(|e| FeedbackError::GitHubError(format!("Failed to parse response: {}", e)))?;

            Ok(issue_response.html_url)
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            Err(FeedbackError::GitHubError(
                "GitHub authentication failed. Please check your token.".to_string(),
            ))
        } else if status == reqwest::StatusCode::FORBIDDEN {
            let body = response.text().await.unwrap_or_default();
            if body.contains("rate limit") {
                Err(FeedbackError::GitHubError(
                    "GitHub API rate limit exceeded. Please try again later.".to_string(),
                ))
            } else {
                Err(FeedbackError::GitHubError(format!(
                    "GitHub API access forbidden: {}",
                    body
                )))
            }
        } else if status == reqwest::StatusCode::NOT_FOUND {
            Err(FeedbackError::GitHubError(format!(
                "Repository {}/{} not found or not accessible",
                self.repo_owner, self.repo_name
            )))
        } else {
            let error_body = response.text().await.unwrap_or_default();
            Err(FeedbackError::GitHubError(format!(
                "GitHub API error ({}): {}",
                status, error_body
            )))
        }
    }

    /// Check if the client can access the repository
    pub async fn check_access(&self) -> Result<bool, FeedbackError> {
        let url = format!(
            "{}/repos/{}/{}",
            self.base_url, self.repo_owner, self.repo_name
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| FeedbackError::NetworkError(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

// =============================================================================
// Issue Formatting
// =============================================================================

/// Format the issue title from feedback
///
/// Creates a title like: `[Bug] Command failed: <first line of description>`
pub fn format_issue_title(feedback: &Feedback) -> String {
    // Get the first line of the description
    let first_line = feedback
        .user_description
        .lines()
        .next()
        .unwrap_or(&feedback.user_description);

    // Determine issue type
    let issue_type = if feedback.context.error_info.is_some() {
        "Bug"
    } else {
        "Feedback"
    };

    // Create title and truncate if needed
    let title = format!("[{}] {}", issue_type, first_line);

    // GitHub titles should be max 256 chars, but 80 is more readable
    if title.len() > 80 {
        format!("{}...", &title[..77])
    } else {
        title
    }
}

/// Format the issue body from feedback
///
/// Creates a markdown body with all relevant information
pub fn format_issue_body(feedback: &Feedback) -> String {
    let mut body = String::new();

    // Header with feedback ID
    body.push_str(&format!("**Feedback ID:** `{}`\n\n", feedback.id));

    // User description
    body.push_str("## Description\n\n");
    body.push_str(&feedback.user_description);
    body.push_str("\n\n");

    // Reproduction steps if provided
    if let Some(ref steps) = feedback.reproduction_steps {
        body.push_str("## Steps to Reproduce\n\n");
        body.push_str(steps);
        body.push_str("\n\n");
    }

    // Command context
    body.push_str("## Command Context\n\n");
    body.push_str(&format!(
        "- **Prompt:** `{}`\n",
        feedback.context.command_info.user_prompt
    ));
    body.push_str(&format!(
        "- **Generated Command:** `{}`\n",
        feedback.context.command_info.generated_command
    ));
    body.push_str(&format!(
        "- **Backend:** `{}`\n",
        feedback.context.command_info.backend
    ));
    if let Some(ref model) = feedback.context.command_info.model {
        body.push_str(&format!("- **Model:** `{}`\n", model));
    }
    body.push_str("\n");

    // Error information
    if let Some(ref error) = feedback.context.error_info {
        body.push_str("## Error Details\n\n");
        if let Some(code) = error.exit_code {
            body.push_str(&format!("- **Exit Code:** `{}`\n", code));
        }
        body.push_str(&format!("- **Error Message:** {}\n", error.error_message));

        if !error.stderr.is_empty() {
            body.push_str("\n**Standard Error:**\n```\n");
            // Limit stderr length
            let stderr = if error.stderr.len() > 1000 {
                format!("{}...(truncated)", &error.stderr[..1000])
            } else {
                error.stderr.clone()
            };
            body.push_str(&stderr);
            body.push_str("\n```\n");
        }
        body.push_str("\n");
    }

    // Environment
    body.push_str("## Environment\n\n");
    body.push_str(&format!(
        "- **OS:** {} {}\n",
        feedback.context.environment.os, feedback.context.environment.os_version
    ));
    body.push_str(&format!(
        "- **Arch:** `{}`\n",
        feedback.context.environment.arch
    ));
    body.push_str(&format!(
        "- **Shell:** `{}`\n",
        feedback.context.environment.shell
    ));
    body.push_str(&format!(
        "- **Terminal:** `{}`\n",
        feedback.context.environment.terminal
    ));
    body.push_str(&format!(
        "- **cmdai Version:** `{}`\n",
        feedback.context.cmdai_version
    ));
    if let Some(ref rust_ver) = feedback.context.environment.rust_version {
        body.push_str(&format!("- **Rust Version:** `{}`\n", rust_ver));
    }
    body.push_str("\n");

    // Git context if available
    if let Some(ref git) = feedback.context.git_context {
        body.push_str("## Git Context\n\n");
        if let Some(ref url) = git.repo_url {
            body.push_str(&format!("- **Repository:** {}\n", url));
        }
        body.push_str(&format!("- **Branch:** `{}`\n", git.current_branch));
        body.push_str(&format!(
            "- **Uncommitted Changes:** {}\n",
            if git.has_uncommitted_changes {
                "Yes"
            } else {
                "No"
            }
        ));
        if let Some(ref hash) = git.last_commit_hash {
            body.push_str(&format!("- **Last Commit:** `{}`\n", hash));
        }
        body.push_str("\n");
    }

    // Full context JSON in collapsible section
    body.push_str("<details>\n<summary>Full Context (JSON)</summary>\n\n```json\n");
    if let Ok(json) = serde_json::to_string_pretty(&feedback.context) {
        body.push_str(&json);
    } else {
        body.push_str("Error serializing context");
    }
    body.push_str("\n```\n</details>\n\n");

    // Footer
    body.push_str("---\n");
    body.push_str("*This issue was automatically created by cmdai feedback system*\n");

    body
}

/// Get labels for the issue based on feedback content
fn get_issue_labels(feedback: &Feedback) -> Vec<String> {
    let mut labels = vec!["user-feedback".to_string()];

    // Add bug label if there's an error
    if feedback.context.error_info.is_some() {
        labels.push("bug".to_string());
    }

    // Add needs-triage for new issues
    labels.push("needs-triage".to_string());

    // Add backend-specific label
    match feedback.context.command_info.backend.as_str() {
        "mlx" => labels.push("backend:mlx".to_string()),
        "embedded" => labels.push("backend:embedded".to_string()),
        "static" => labels.push("backend:static".to_string()),
        "ollama" => labels.push("backend:ollama".to_string()),
        _ => {}
    }

    // Add platform label
    match feedback.context.environment.os.as_str() {
        "macos" => labels.push("platform:macos".to_string()),
        "linux" => labels.push("platform:linux".to_string()),
        "windows" => labels.push("platform:windows".to_string()),
        _ => {}
    }

    labels
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // =========================================================================
    // Client Creation Tests
    // =========================================================================

    #[test]
    fn test_client_creation() {
        let client =
            GitHubClient::new("token123".to_string(), "owner".to_string(), "repo".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_empty_token() {
        let client =
            GitHubClient::new("".to_string(), "owner".to_string(), "repo".to_string());
        assert!(client.is_err());
        assert!(matches!(client.unwrap_err(), FeedbackError::ConfigError(_)));
    }

    // =========================================================================
    // Issue Title Formatting Tests
    // =========================================================================

    #[test]
    fn test_format_issue_title_basic() {
        let feedback = create_test_feedback();
        let title = format_issue_title(&feedback);

        assert!(title.starts_with("["));
        assert!(title.len() <= 80);
    }

    #[test]
    fn test_format_issue_title_with_error() {
        let mut feedback = create_test_feedback();
        feedback.context.error_info = Some(ErrorInfo {
            exit_code: Some(1),
            stderr: "error".to_string(),
            stdout: "".to_string(),
            error_message: "Failed".to_string(),
            error_type: None,
        });

        let title = format_issue_title(&feedback);
        assert!(title.starts_with("[Bug]"));
    }

    #[test]
    fn test_format_issue_title_without_error() {
        let feedback = create_test_feedback();
        let title = format_issue_title(&feedback);
        assert!(title.starts_with("[Feedback]"));
    }

    #[test]
    fn test_format_issue_title_truncation() {
        let mut feedback = create_test_feedback();
        feedback.user_description = "x".repeat(100);

        let title = format_issue_title(&feedback);
        assert!(title.len() <= 80);
        assert!(title.ends_with("..."));
    }

    // =========================================================================
    // Issue Body Formatting Tests
    // =========================================================================

    #[test]
    fn test_format_issue_body_contains_required_sections() {
        let feedback = create_test_feedback();
        let body = format_issue_body(&feedback);

        assert!(body.contains("Feedback ID:"));
        assert!(body.contains("## Description"));
        assert!(body.contains("## Command Context"));
        assert!(body.contains("## Environment"));
        assert!(body.contains("Full Context (JSON)"));
    }

    #[test]
    fn test_format_issue_body_includes_command_info() {
        let feedback = create_test_feedback();
        let body = format_issue_body(&feedback);

        assert!(body.contains(&feedback.context.command_info.user_prompt));
        assert!(body.contains(&feedback.context.command_info.generated_command));
        assert!(body.contains(&feedback.context.command_info.backend));
    }

    #[test]
    fn test_format_issue_body_includes_environment() {
        let feedback = create_test_feedback();
        let body = format_issue_body(&feedback);

        assert!(body.contains(&feedback.context.environment.os));
        assert!(body.contains(&feedback.context.environment.arch));
        assert!(body.contains(&feedback.context.environment.shell));
    }

    #[test]
    fn test_format_issue_body_with_reproduction_steps() {
        let mut feedback = create_test_feedback();
        feedback.reproduction_steps = Some("1. Run command\n2. See error".to_string());

        let body = format_issue_body(&feedback);
        assert!(body.contains("## Steps to Reproduce"));
        assert!(body.contains("1. Run command"));
    }

    #[test]
    fn test_format_issue_body_with_error() {
        let mut feedback = create_test_feedback();
        feedback.context.error_info = Some(ErrorInfo {
            exit_code: Some(1),
            stderr: "Something went wrong".to_string(),
            stdout: "".to_string(),
            error_message: "Command failed with exit code 1".to_string(),
            error_type: Some("ExecutionError".to_string()),
        });

        let body = format_issue_body(&feedback);
        assert!(body.contains("## Error Details"));
        assert!(body.contains("Exit Code:"));
        assert!(body.contains("Something went wrong"));
    }

    #[test]
    fn test_format_issue_body_with_git_context() {
        let mut feedback = create_test_feedback();
        feedback.context.git_context = Some(GitContext {
            repo_url: Some("https://github.com/test/repo".to_string()),
            current_branch: "main".to_string(),
            has_uncommitted_changes: true,
            last_commit_hash: Some("abc1234".to_string()),
        });

        let body = format_issue_body(&feedback);
        assert!(body.contains("## Git Context"));
        assert!(body.contains("main"));
        assert!(body.contains("abc1234"));
    }

    // =========================================================================
    // Label Generation Tests
    // =========================================================================

    #[test]
    fn test_get_issue_labels_basic() {
        let feedback = create_test_feedback();
        let labels = get_issue_labels(&feedback);

        assert!(labels.contains(&"user-feedback".to_string()));
        assert!(labels.contains(&"needs-triage".to_string()));
    }

    #[test]
    fn test_get_issue_labels_with_error() {
        let mut feedback = create_test_feedback();
        feedback.context.error_info = Some(ErrorInfo {
            exit_code: Some(1),
            stderr: "".to_string(),
            stdout: "".to_string(),
            error_message: "Error".to_string(),
            error_type: None,
        });

        let labels = get_issue_labels(&feedback);
        assert!(labels.contains(&"bug".to_string()));
    }

    #[test]
    fn test_get_issue_labels_backend() {
        let mut feedback = create_test_feedback();
        feedback.context.command_info.backend = "mlx".to_string();

        let labels = get_issue_labels(&feedback);
        assert!(labels.contains(&"backend:mlx".to_string()));
    }

    #[test]
    fn test_get_issue_labels_platform() {
        let mut feedback = create_test_feedback();
        feedback.context.environment.os = "macos".to_string();

        let labels = get_issue_labels(&feedback);
        assert!(labels.contains(&"platform:macos".to_string()));
    }

    // =========================================================================
    // API Integration Tests (using wiremock)
    // =========================================================================

    #[tokio::test]
    async fn test_create_issue_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/repos/owner/repo/issues"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "html_url": "https://github.com/owner/repo/issues/123",
                "number": 123
            })))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_url(
            "test-token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            mock_server.uri(),
        )
        .unwrap();

        let feedback = create_test_feedback();
        let result = client.create_issue(&feedback).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "https://github.com/owner/repo/issues/123"
        );
    }

    #[tokio::test]
    async fn test_create_issue_authentication_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/repos/owner/repo/issues"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "message": "Bad credentials"
            })))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_url(
            "bad-token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            mock_server.uri(),
        )
        .unwrap();

        let feedback = create_test_feedback();
        let result = client.create_issue(&feedback).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FeedbackError::GitHubError(_)));
    }

    #[tokio::test]
    async fn test_create_issue_rate_limit() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/repos/owner/repo/issues"))
            .respond_with(ResponseTemplate::new(403).set_body_string(
                "API rate limit exceeded for user ID",
            ))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_url(
            "test-token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            mock_server.uri(),
        )
        .unwrap();

        let feedback = create_test_feedback();
        let result = client.create_issue(&feedback).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, FeedbackError::GitHubError(_)));
        if let FeedbackError::GitHubError(msg) = err {
            assert!(msg.contains("rate limit"));
        }
    }

    #[tokio::test]
    async fn test_create_issue_repo_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/repos/owner/repo/issues"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found"
            })))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_url(
            "test-token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            mock_server.uri(),
        )
        .unwrap();

        let feedback = create_test_feedback();
        let result = client.create_issue(&feedback).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        if let FeedbackError::GitHubError(msg) = err {
            assert!(msg.contains("not found"));
        }
    }

    #[tokio::test]
    async fn test_check_access_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/owner/repo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "full_name": "owner/repo"
            })))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_url(
            "test-token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            mock_server.uri(),
        )
        .unwrap();

        let result = client.check_access().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // =========================================================================
    // Helper Functions
    // =========================================================================

    fn create_test_feedback() -> Feedback {
        Feedback {
            id: FeedbackId::parse("fb-abc123").unwrap(),
            timestamp: Utc::now(),
            user_description: "The command didn't work as expected".to_string(),
            reproduction_steps: None,
            context: FeedbackContext {
                timestamp: Utc::now(),
                cmdai_version: "1.0.0".to_string(),
                environment: EnvironmentInfo {
                    os: "macos".to_string(),
                    os_version: "14.0".to_string(),
                    arch: "arm64".to_string(),
                    shell: "zsh".to_string(),
                    terminal: "Terminal.app".to_string(),
                    rust_version: Some("1.75.0".to_string()),
                },
                command_info: CommandInfo {
                    user_prompt: "list files".to_string(),
                    generated_command: "ls -la".to_string(),
                    backend: "static".to_string(),
                    model: None,
                    command_history: vec![],
                },
                error_info: None,
                system_state: SystemState {
                    available_backends: vec!["static".to_string()],
                    cache_dir: PathBuf::from("$HOME/.cache/caro"),
                    config_file: None,
                    is_ci: false,
                    is_interactive: true,
                },
                git_context: None,
            },
            github_issue_url: None,
            status: FeedbackStatus::Submitted,
        }
    }
}
