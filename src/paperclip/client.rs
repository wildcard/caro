//! HTTP client for the Paperclip AI orchestration API.
//!
//! Handles authentication, task retrieval, progress reporting, and budget
//! checks against the Paperclip control plane.

use super::config::PaperclipConfig;
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Data types returned by / sent to the Paperclip API
// ---------------------------------------------------------------------------

/// A task assigned to this agent by Paperclip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Status of a task in progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    InProgress,
    Completed,
    Failed,
}

/// Result payload sent back to Paperclip on task completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub command: String,
    pub safety_level: String,
    #[serde(default)]
    pub explanation: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Budget information for this agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub remaining: f64,
    pub limit: f64,
    pub currency: String,
    #[serde(default)]
    pub exceeded: bool,
}

/// Payload for progress/status updates.
#[derive(Debug, Serialize)]
struct ProgressPayload {
    status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<TaskResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// HTTP client for communicating with the Paperclip orchestration API.
pub struct PaperclipClient {
    http: reqwest::Client,
    config: PaperclipConfig,
}

impl PaperclipClient {
    /// Create a new client from a Paperclip configuration.
    pub fn new(config: PaperclipConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                .context("invalid API key for header")?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self { http, config })
    }

    /// Fetch tasks assigned to this agent for the current run.
    pub async fn get_tasks(&self) -> Result<Vec<Task>> {
        let url = format!(
            "{}/api/agents/{}/tasks?run_id={}",
            self.config.api_url, self.config.agent_id, self.config.run_id
        );
        debug!("Fetching tasks from {}", url);

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("failed to reach Paperclip API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Paperclip API returned {}: {}", status, body);
        }

        let tasks: Vec<Task> = resp.json().await.context("failed to parse tasks response")?;
        debug!("Received {} task(s)", tasks.len());
        Ok(tasks)
    }

    /// Report that work on a task has started.
    pub async fn report_progress(&self, task_id: &str) -> Result<()> {
        self.send_status_update(
            task_id,
            ProgressPayload {
                status: TaskStatus::InProgress,
                result: None,
                error: None,
            },
        )
        .await
    }

    /// Report successful completion of a task with a result.
    pub async fn report_completion(&self, task_id: &str, result: TaskResult) -> Result<()> {
        self.send_status_update(
            task_id,
            ProgressPayload {
                status: TaskStatus::Completed,
                result: Some(result),
                error: None,
            },
        )
        .await
    }

    /// Report that a task failed.
    pub async fn report_error(&self, task_id: &str, error: &str) -> Result<()> {
        self.send_status_update(
            task_id,
            ProgressPayload {
                status: TaskStatus::Failed,
                result: None,
                error: Some(error.to_string()),
            },
        )
        .await
    }

    /// Check the remaining budget for this agent.
    pub async fn check_budget(&self) -> Result<BudgetStatus> {
        let url = format!(
            "{}/api/agents/{}/budget",
            self.config.api_url, self.config.agent_id
        );
        debug!("Checking budget at {}", url);

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("failed to reach Paperclip API for budget check")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!("Budget check failed ({}): {}", status, body);
            // Return a permissive default so budget failures don't block work
            return Ok(BudgetStatus {
                remaining: f64::MAX,
                limit: f64::MAX,
                currency: "USD".to_string(),
                exceeded: false,
            });
        }

        resp.json()
            .await
            .context("failed to parse budget response")
    }

    /// Check connectivity to the Paperclip API.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/health", self.config.api_url);
        match self.http.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(e) => {
                warn!("Paperclip health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Send a status update for a specific task.
    async fn send_status_update(&self, task_id: &str, payload: ProgressPayload) -> Result<()> {
        let url = format!(
            "{}/api/agents/{}/tasks/{}/status",
            self.config.api_url, self.config.agent_id, task_id
        );
        debug!("Sending {:?} update for task {}", payload.status, task_id);

        let resp = self
            .http
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("failed to send status update")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Status update failed ({}): {}", status, body);
        }

        Ok(())
    }

    /// Get the underlying config (for display / diagnostics).
    pub fn config(&self) -> &PaperclipConfig {
        &self.config
    }
}
