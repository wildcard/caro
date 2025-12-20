use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Execution history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub prompt: String,
    pub generated_command: String,
    pub explanation: Option<String>,
    pub shell_type: String,
    pub risk_level: String,
    pub executed: bool,
    pub blocked_reason: Option<String>,
    pub generation_time_ms: i64,
    pub execution_time_ms: i64,
    pub warnings: Vec<String>,
    pub alternatives: Vec<String>,
    pub backend_used: String,
}

/// Rating for an execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRating {
    pub id: Option<i64>,
    pub execution_id: i64,
    pub rating: i32, // 1-5
    pub feedback: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Vote for an execution (thumbs up/down)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionVote {
    pub id: Option<i64>,
    pub execution_id: i64,
    pub vote_type: VoteType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VoteType {
    Up,
    Down,
}

impl VoteType {
    pub fn as_str(&self) -> &str {
        match self {
            VoteType::Up => "up",
            VoteType::Down => "down",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "up" => Ok(VoteType::Up),
            "down" => Ok(VoteType::Down),
            _ => Err(format!("Invalid vote type: {}", s)),
        }
    }
}

/// Analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analytics {
    pub total_executions: i64,
    pub successful_executions: i64,
    pub blocked_executions: i64,
    pub average_generation_time_ms: f64,
    pub most_used_shell: String,
    pub risk_level_distribution: std::collections::HashMap<String, i64>,
    pub backend_usage: std::collections::HashMap<String, i64>,
}

/// Filter options for execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryFilter {
    pub shell_type: Option<String>,
    pub risk_level: Option<String>,
    pub executed: Option<bool>,
    pub blocked: Option<bool>,
    pub search_query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Command generation request from GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandGenerationRequest {
    pub prompt: String,
    pub shell: Option<String>,
    pub safety_level: Option<String>,
    pub dry_run: bool, // Don't execute, just generate
}

/// Command generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandGenerationResponse {
    pub generated_command: String,
    pub explanation: String,
    pub risk_level: String,
    pub warnings: Vec<String>,
    pub alternatives: Vec<String>,
    pub requires_confirmation: bool,
    pub blocked_reason: Option<String>,
    pub generation_time_ms: u64,
    pub backend_used: String,
}
