//! Audit logging for compliance and command tracking
//!
//! This module provides comprehensive audit logging for all command executions,
//! supporting compliance requirements and forensic analysis.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use tokio::io::AsyncWriteExt;

/// Outcome of command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionOutcome {
    /// Command executed successfully
    Success,
    /// Command failed with error
    Failed,
    /// Command blocked by safety checks
    Blocked,
    /// User declined to execute
    Declined,
    /// Executed in sandbox only
    SandboxOnly,
    /// Sandbox execution rolled back
    RolledBack,
}

impl std::fmt::Display for ExecutionOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "SUCCESS"),
            Self::Failed => write!(f, "FAILED"),
            Self::Blocked => write!(f, "BLOCKED"),
            Self::Declined => write!(f, "DECLINED"),
            Self::SandboxOnly => write!(f, "SANDBOX_ONLY"),
            Self::RolledBack => write!(f, "ROLLED_BACK"),
        }
    }
}

/// File modification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModification {
    pub path: PathBuf,
    pub operation: String,
    pub size_change: i64,
}

/// Structured audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID
    pub id: String,
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
    /// Username who executed command
    pub user: String,
    /// Hostname where command was executed
    pub hostname: String,
    /// Working directory
    pub working_dir: PathBuf,
    /// Original user prompt/intent
    pub prompt: String,
    /// Generated command
    pub command: String,
    /// Risk score assigned
    pub risk_score: f32,
    /// Risk level category
    pub risk_level: String,
    /// Execution outcome
    pub outcome: ExecutionOutcome,
    /// Exit code (if executed)
    pub exit_code: Option<i32>,
    /// File modifications
    pub modifications: Vec<FileModification>,
    /// Duration of execution in milliseconds
    pub duration_ms: Option<u64>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl AuditEntry {
    /// Create new audit entry
    pub fn new(
        user: String,
        hostname: String,
        working_dir: PathBuf,
        prompt: String,
        command: String,
        risk_score: f32,
        risk_level: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user,
            hostname,
            working_dir,
            prompt,
            command,
            risk_score,
            risk_level,
            outcome: ExecutionOutcome::Declined,
            exit_code: None,
            modifications: Vec::new(),
            duration_ms: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Set execution outcome
    pub fn with_outcome(mut self, outcome: ExecutionOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Set exit code
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Add file modifications
    pub fn with_modifications(mut self, modifications: Vec<FileModification>) -> Self {
        self.modifications = modifications;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Filter criteria for querying audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub user: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub outcome: Option<ExecutionOutcome>,
    pub min_risk_score: Option<f32>,
    pub command_pattern: Option<String>,
}

/// Compliance export format
#[derive(Debug, Clone, Copy)]
pub enum ComplianceFormat {
    /// JSON Lines format
    JsonLines,
    /// CSV format
    Csv,
    /// Splunk-compatible format
    Splunk,
    /// Elasticsearch bulk format
    Elasticsearch,
}

/// Audit logger with configurable storage and encryption
pub struct AuditLogger {
    /// Path to audit log file
    log_path: PathBuf,
    /// Whether to encrypt logs at rest
    encrypt: bool,
    /// Rotation policy (days)
    rotation_days: u32,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new(log_path: PathBuf) -> Self {
        Self {
            log_path,
            encrypt: false,
            rotation_days: 90,
        }
    }

    /// Enable encryption at rest
    pub fn with_encryption(mut self) -> Self {
        self.encrypt = true;
        self
    }

    /// Set rotation policy
    pub fn with_rotation_days(mut self, days: u32) -> Self {
        self.rotation_days = days;
        self
    }

    /// Initialize audit log file
    pub async fn init(&self) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.log_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .context("Failed to create audit log directory")?;
        }

        // Create file if it doesn't exist
        if !self.log_path.exists() {
            tokio::fs::File::create(&self.log_path).await
                .context("Failed to create audit log file")?;
        }

        Ok(())
    }

    /// Log audit entry
    pub async fn log(&self, entry: AuditEntry) -> Result<()> {
        // Serialize entry as JSON line
        let mut json = serde_json::to_string(&entry)
            .context("Failed to serialize audit entry")?;
        json.push('\n');

        // Encrypt if enabled
        let data = if self.encrypt {
            self.encrypt_data(json.as_bytes())?
        } else {
            json.into_bytes()
        };

        // Append to log file
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await
            .context("Failed to open audit log file")?;

        file.write_all(&data).await
            .context("Failed to write audit entry")?;

        file.sync_all().await
            .context("Failed to sync audit log")?;

        Ok(())
    }

    /// Query audit logs with filter
    pub async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>> {
        let contents = tokio::fs::read_to_string(&self.log_path).await
            .context("Failed to read audit log")?;

        let mut entries = Vec::new();

        for line in contents.lines() {
            if line.is_empty() {
                continue;
            }

            let entry: AuditEntry = serde_json::from_str(line)
                .context("Failed to parse audit entry")?;

            // Apply filters
            if let Some(ref user) = filter.user {
                if &entry.user != user {
                    continue;
                }
            }

            if let Some(start) = filter.start_time {
                if entry.timestamp < start {
                    continue;
                }
            }

            if let Some(end) = filter.end_time {
                if entry.timestamp > end {
                    continue;
                }
            }

            if let Some(outcome) = filter.outcome {
                if entry.outcome != outcome {
                    continue;
                }
            }

            if let Some(min_risk) = filter.min_risk_score {
                if entry.risk_score < min_risk {
                    continue;
                }
            }

            if let Some(ref pattern) = filter.command_pattern {
                if !entry.command.contains(pattern) {
                    continue;
                }
            }

            entries.push(entry);
        }

        Ok(entries)
    }

    /// Export audit logs in compliance format
    pub async fn export_compliance(&self, format: ComplianceFormat) -> Result<String> {
        let entries = self.query(AuditFilter::default()).await?;

        match format {
            ComplianceFormat::JsonLines => {
                self.export_json_lines(&entries)
            }
            ComplianceFormat::Csv => {
                self.export_csv(&entries)
            }
            ComplianceFormat::Splunk => {
                self.export_splunk(&entries)
            }
            ComplianceFormat::Elasticsearch => {
                self.export_elasticsearch(&entries)
            }
        }
    }

    /// Export as JSON Lines
    fn export_json_lines(&self, entries: &[AuditEntry]) -> Result<String> {
        let mut output = String::new();

        for entry in entries {
            let json = serde_json::to_string(entry)?;
            output.push_str(&json);
            output.push('\n');
        }

        Ok(output)
    }

    /// Export as CSV
    fn export_csv(&self, entries: &[AuditEntry]) -> Result<String> {
        let mut output = String::new();

        // Header
        output.push_str("timestamp,user,hostname,command,risk_score,risk_level,outcome,exit_code\n");

        // Rows
        for entry in entries {
            output.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                entry.timestamp.to_rfc3339(),
                entry.user,
                entry.hostname,
                escape_csv(&entry.command),
                entry.risk_score,
                entry.risk_level,
                entry.outcome,
                entry.exit_code.map(|c| c.to_string()).unwrap_or_default()
            ));
        }

        Ok(output)
    }

    /// Export for Splunk
    fn export_splunk(&self, entries: &[AuditEntry]) -> Result<String> {
        let mut output = String::new();

        for entry in entries {
            // Splunk format: timestamp key=value pairs
            output.push_str(&format!(
                "{} user={} hostname={} command=\"{}\" risk_score={} risk_level={} outcome={}\n",
                entry.timestamp.to_rfc3339(),
                entry.user,
                entry.hostname,
                entry.command,
                entry.risk_score,
                entry.risk_level,
                entry.outcome
            ));
        }

        Ok(output)
    }

    /// Export for Elasticsearch bulk API
    fn export_elasticsearch(&self, entries: &[AuditEntry]) -> Result<String> {
        let mut output = String::new();

        for entry in entries {
            // Elasticsearch bulk format: action line + document line
            let action = serde_json::json!({
                "index": {
                    "_index": "cmdai-audit",
                    "_id": entry.id
                }
            });
            output.push_str(&serde_json::to_string(&action)?);
            output.push('\n');
            output.push_str(&serde_json::to_string(entry)?);
            output.push('\n');
        }

        Ok(output)
    }

    /// Encrypt data (placeholder - would use real encryption)
    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement real encryption using AES-256-GCM
        // For now, just return as-is
        Ok(data.to_vec())
    }

    /// Rotate old log files
    pub async fn rotate(&self) -> Result<Vec<PathBuf>> {
        let cutoff = Utc::now() - chrono::Duration::days(self.rotation_days as i64);
        let mut rotated = Vec::new();

        // Read all entries
        let entries = self.query(AuditFilter::default()).await?;

        // Split into old and new
        let (old, new): (Vec<_>, Vec<_>) = entries.iter()
            .partition(|e| e.timestamp < cutoff);

        if !old.is_empty() {
            // Archive old entries
            let archive_path = self.log_path.with_extension(
                format!("log.{}", Utc::now().format("%Y%m%d"))
            );

            let mut archive_data = String::new();
            for entry in old {
                archive_data.push_str(&serde_json::to_string(entry)?);
                archive_data.push('\n');
            }

            tokio::fs::write(&archive_path, archive_data).await?;
            rotated.push(archive_path);

            // Rewrite main log with only new entries
            let mut new_data = String::new();
            for entry in new {
                new_data.push_str(&serde_json::to_string(entry)?);
                new_data.push('\n');
            }

            tokio::fs::write(&self.log_path, new_data).await?;
        }

        Ok(rotated)
    }
}

/// Escape CSV field
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_audit_logger_init() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(temp_file.path().to_path_buf());

        logger.init().await.unwrap();
        assert!(temp_file.path().exists());
    }

    #[tokio::test]
    async fn test_log_entry() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(temp_file.path().to_path_buf());
        logger.init().await.unwrap();

        let entry = AuditEntry::new(
            "testuser".to_string(),
            "testhost".to_string(),
            PathBuf::from("/tmp"),
            "list files".to_string(),
            "ls -la".to_string(),
            0.0,
            "Safe".to_string(),
        );

        logger.log(entry).await.unwrap();

        let contents = tokio::fs::read_to_string(temp_file.path()).await.unwrap();
        assert!(contents.contains("testuser"));
        assert!(contents.contains("ls -la"));
    }

    #[tokio::test]
    async fn test_query_filter() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(temp_file.path().to_path_buf());
        logger.init().await.unwrap();

        let entry1 = AuditEntry::new(
            "user1".to_string(),
            "host1".to_string(),
            PathBuf::from("/tmp"),
            "test".to_string(),
            "ls".to_string(),
            0.0,
            "Safe".to_string(),
        );

        let entry2 = AuditEntry::new(
            "user2".to_string(),
            "host1".to_string(),
            PathBuf::from("/tmp"),
            "test".to_string(),
            "rm -rf".to_string(),
            9.0,
            "Critical".to_string(),
        );

        logger.log(entry1).await.unwrap();
        logger.log(entry2).await.unwrap();

        // Filter by user
        let filter = AuditFilter {
            user: Some("user1".to_string()),
            ..Default::default()
        };

        let results = logger.query(filter).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user, "user1");
    }

    #[tokio::test]
    async fn test_export_csv() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(temp_file.path().to_path_buf());
        logger.init().await.unwrap();

        let entry = AuditEntry::new(
            "user".to_string(),
            "host".to_string(),
            PathBuf::from("/tmp"),
            "test".to_string(),
            "ls".to_string(),
            0.0,
            "Safe".to_string(),
        );

        logger.log(entry).await.unwrap();

        let csv = logger.export_compliance(ComplianceFormat::Csv).await.unwrap();
        assert!(csv.contains("timestamp,user,hostname"));
        assert!(csv.contains("user,host,ls"));
    }
}
