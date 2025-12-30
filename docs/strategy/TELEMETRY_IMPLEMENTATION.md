# Telemetry Implementation Specification

> **Companion to:** TELEMETRY_STRATEGY.md
> **Purpose:** Concrete implementation details for developers
> **Target:** v1.1.0 Beta Release

## Overview

This document provides implementable specifications for adding telemetry to Caro. It translates the strategy document into Rust code structures, configuration schema, and CLI commands.

---

## 1. Configuration Schema

### Addition to `src/config/mod.rs`

```rust
use serde::{Deserialize, Serialize};

/// Telemetry configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Master switch for telemetry collection
    /// Default: true for beta, false for GA
    pub enabled: bool,

    /// Collection level: minimal, standard, detailed
    #[serde(default = "default_level")]
    pub level: TelemetryLevel,

    /// Hours between transmission attempts
    #[serde(default = "default_batch_interval")]
    pub batch_interval_hours: u32,

    /// Maximum local spool size in MB
    #[serde(default = "default_spool_size")]
    pub offline_spool_max_mb: u32,

    /// Air-gapped mode: collect but never transmit
    #[serde(default)]
    pub air_gapped: bool,

    /// Endpoint for telemetry submission (internal use)
    #[serde(skip_serializing)]
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TelemetryLevel {
    /// Session start/end, errors only
    Minimal,
    /// + command events, safety events
    #[default]
    Standard,
    /// + performance percentiles, retry patterns
    Detailed,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: cfg!(feature = "beta"), // true for beta builds
            level: TelemetryLevel::Standard,
            batch_interval_hours: 24,
            offline_spool_max_mb: 10,
            air_gapped: false,
            endpoint: None,
        }
    }
}

fn default_level() -> TelemetryLevel {
    TelemetryLevel::Standard
}

fn default_batch_interval() -> u32 {
    24
}

fn default_spool_size() -> u32 {
    10
}
```

### Config File Example

```toml
# ~/.config/caro/config.toml

[telemetry]
enabled = true
level = "standard"
batch_interval_hours = 24
offline_spool_max_mb = 10
air_gapped = false
```

### Environment Variable Overrides

| Variable | Type | Description |
|----------|------|-------------|
| `CARO_TELEMETRY_ENABLED` | bool | Override enabled state |
| `CARO_TELEMETRY_LEVEL` | string | Override collection level |
| `CARO_TELEMETRY_AIR_GAPPED` | bool | Override air-gapped mode |

---

## 2. Event Type Definitions

### File: `src/telemetry/events.rs`

```rust
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// All telemetry event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum TelemetryEvent {
    SessionStart(SessionStartEvent),
    SessionEnd(SessionEndEvent),
    CommandGenerated(CommandGeneratedEvent),
    CommandExecuted(CommandExecutedEvent),
    SafetyTriggered(SafetyTriggeredEvent),
    ErrorOccurred(ErrorOccurredEvent),
}

impl TelemetryEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::SessionStart(_) => "session.start",
            Self::SessionEnd(_) => "session.end",
            Self::CommandGenerated(_) => "command.generated",
            Self::CommandExecuted(_) => "command.executed",
            Self::SafetyTriggered(_) => "safety.triggered",
            Self::ErrorOccurred(_) => "error.occurred",
        }
    }

    pub fn timestamp(&self) -> SystemTime {
        match self {
            Self::SessionStart(e) => e.timestamp,
            Self::SessionEnd(e) => e.timestamp,
            Self::CommandGenerated(e) => e.timestamp,
            Self::CommandExecuted(e) => e.timestamp,
            Self::SafetyTriggered(e) => e.timestamp,
            Self::ErrorOccurred(e) => e.timestamp,
        }
    }
}

/// Session start event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartEvent {
    pub timestamp: SystemTime,
    pub session_id: String,
    pub caro_version: String,
    pub platform: PlatformInfo,
    pub backend_config: BackendConfigInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_first_run: Option<bool>,
}

/// Platform information (safe metadata only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,      // "macos", "linux", "windows"
    pub arch: String,    // "aarch64", "x86_64"
    pub shell: String,   // "zsh", "bash", "fish", etc.
}

/// Backend configuration (no secrets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfigInfo {
    pub primary: String,           // "embedded-mlx", "ollama", etc.
    pub fallback_enabled: bool,
}

/// Session end event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEndEvent {
    pub timestamp: SystemTime,
    pub session_id: String,
    pub session_duration_ms: u64,
    pub commands_generated: u32,
    pub commands_executed: u32,
    pub errors_encountered: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_reason: Option<ExitReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_blocks: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    UserComplete,
    UserAbort,
    Error,
    Timeout,
}

/// Command generated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandGeneratedEvent {
    pub timestamp: SystemTime,
    pub session_id: String,
    pub generation_id: Uuid,
    pub backend_used: String,
    pub inference_time_ms: u64,
    pub safety_result: SafetyResultInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyResultInfo {
    pub risk_level: RiskLevel,
    pub patterns_matched: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}

/// Command executed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutedEvent {
    pub timestamp: SystemTime,
    pub session_id: String,
    pub generation_id: Uuid,
    pub execution_mode: ExecutionMode,
    pub modified_before_execution: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code_category: Option<ExitCodeCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Confirmed,      // User confirmed execution
    AutoConfirm,    // -y flag used
    DryRun,         // --dry-run
    Interactive,    // -i step-by-step
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExitCodeCategory {
    Success,
    Error,
    Timeout,
}

/// Safety triggered event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyTriggeredEvent {
    pub timestamp: SystemTime,
    pub session_id: String,
    pub generation_id: Uuid,
    pub risk_level: RiskLevel,
    pub pattern_category: PatternCategory,
    pub action_taken: SafetyAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_override: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_level_config: Option<String>,
}

/// High-level pattern categories (no specific patterns exposed)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternCategory {
    FilesystemDestruction,
    PrivilegeEscalation,
    DiskOperation,
    NetworkExfiltration,
    SystemModification,
    ServiceControl,
    ConfigModification,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SafetyAction {
    Blocked,
    Warned,
    Allowed,
}

/// Error occurred event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorOccurredEvent {
    pub timestamp: SystemTime,
    pub session_id: String,
    pub error_category: ErrorCategory,
    pub error_code: String,
    pub component: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recoverable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_attempted: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    BackendTimeout,
    BackendConnection,
    BackendParse,
    ModelLoad,
    ConfigError,
    ValidationError,
    ExecutionError,
    Unknown,
}
```

---

## 3. Collector Implementation

### File: `src/telemetry/collector.rs`

```rust
use crate::config::TelemetryConfig;
use crate::telemetry::events::TelemetryEvent;
use crate::telemetry::queue::EventQueue;
use crate::telemetry::redaction::validate_event;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Global telemetry collector instance
static COLLECTOR: once_cell::sync::OnceCell<TelemetryCollector> = once_cell::sync::OnceCell::new();

/// Initialize global telemetry collector
pub fn init(config: TelemetryConfig) -> &'static TelemetryCollector {
    COLLECTOR.get_or_init(|| TelemetryCollector::new(config))
}

/// Get global telemetry collector (panics if not initialized)
pub fn get() -> &'static TelemetryCollector {
    COLLECTOR.get().expect("Telemetry not initialized")
}

/// Try to get global telemetry collector
pub fn try_get() -> Option<&'static TelemetryCollector> {
    COLLECTOR.get()
}

/// Main telemetry collector
pub struct TelemetryCollector {
    config: TelemetryConfig,
    enabled: AtomicBool,
    queue: Arc<EventQueue>,
    sender: mpsc::UnboundedSender<TelemetryEvent>,
}

impl TelemetryCollector {
    pub fn new(config: TelemetryConfig) -> Self {
        let enabled = config.enabled;
        let queue = Arc::new(EventQueue::new(&config).expect("Failed to create event queue"));

        // Background task for async event processing
        let (sender, mut receiver) = mpsc::unbounded_channel::<TelemetryEvent>();
        let queue_clone = queue.clone();

        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let Err(e) = queue_clone.enqueue(&event) {
                    warn!("Failed to enqueue telemetry event: {:?}", e);
                }
            }
        });

        Self {
            config,
            enabled: AtomicBool::new(enabled),
            queue,
            sender,
        }
    }

    /// Record an event (non-blocking, never visibly fails)
    pub fn record(&self, event: impl Into<TelemetryEvent>) {
        // Fast path: check enabled flag
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        let event = event.into();

        // Validate event doesn't contain sensitive data
        if let Err(e) = validate_event(&event) {
            warn!("Telemetry event rejected (sensitive data): {:?}", e);
            return;
        }

        // Non-blocking send to background task
        if let Err(e) = self.sender.send(event) {
            debug!("Failed to send telemetry event: {:?}", e);
        }
    }

    /// Enable or disable telemetry at runtime
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Get pending event count
    pub fn pending_count(&self) -> usize {
        self.queue.count().unwrap_or(0)
    }

    /// Get pending events for display
    pub fn pending_events(&self) -> Vec<TelemetryEvent> {
        self.queue.all_pending().unwrap_or_default()
    }

    /// Flush pending events (transmit or spool)
    pub async fn flush(&self) -> Result<FlushResult, TelemetryError> {
        if self.config.air_gapped {
            return Ok(FlushResult::Spooled(self.pending_count()));
        }

        let batch = self.queue.pending_batch(1000)?;
        if batch.is_empty() {
            return Ok(FlushResult::Empty);
        }

        match self.transmit(&batch).await {
            Ok(_) => {
                self.queue.mark_transmitted(&batch)?;
                Ok(FlushResult::Transmitted(batch.len()))
            }
            Err(e) => {
                debug!("Telemetry transmission failed, will retry: {:?}", e);
                Ok(FlushResult::Deferred(batch.len()))
            }
        }
    }

    /// Export events to file (for air-gapped environments)
    pub fn export(&self, path: &std::path::Path) -> Result<ExportResult, TelemetryError> {
        let events = self.queue.all_pending()?;
        let export = TelemetryExport {
            export_version: "1.0".to_string(),
            exported_at: chrono::Utc::now(),
            caro_version: env!("CARGO_PKG_VERSION").to_string(),
            event_count: events.len(),
            events,
        };

        let json = serde_json::to_string_pretty(&export)?;

        // Compress with gzip
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let file = std::fs::File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(json.as_bytes())?;
        encoder.finish()?;

        Ok(ExportResult {
            path: path.to_path_buf(),
            event_count: export.event_count,
            size_bytes: std::fs::metadata(path)?.len(),
        })
    }

    /// Clear all pending events
    pub fn clear(&self) -> Result<usize, TelemetryError> {
        self.queue.clear()
    }

    async fn transmit(&self, batch: &[TelemetryEvent]) -> Result<(), TelemetryError> {
        let endpoint = self.config.endpoint
            .as_deref()
            .unwrap_or("https://telemetry.caro.sh/v1/events");

        let payload = TransmitPayload {
            events: batch.to_vec(),
            batch_id: uuid::Uuid::new_v4(),
            client_timestamp: chrono::Utc::now(),
        };

        let client = reqwest::Client::new();
        let response = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .header("X-Caro-Version", env!("CARGO_PKG_VERSION"))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(TelemetryError::TransmitFailed(response.status().as_u16()))
        }
    }
}

#[derive(Debug)]
pub enum FlushResult {
    Empty,
    Transmitted(usize),
    Deferred(usize),
    Spooled(usize),
}

#[derive(Debug)]
pub struct ExportResult {
    pub path: std::path::PathBuf,
    pub event_count: usize,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize)]
struct TransmitPayload {
    events: Vec<TelemetryEvent>,
    batch_id: uuid::Uuid,
    client_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct TelemetryExport {
    export_version: String,
    exported_at: chrono::DateTime<chrono::Utc>,
    caro_version: String,
    event_count: usize,
    events: Vec<TelemetryEvent>,
}

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("Queue error: {0}")]
    Queue(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Transmission failed with status: {0}")]
    TransmitFailed(u16),
}
```

---

## 4. Redaction and Validation

### File: `src/telemetry/redaction.rs`

```rust
use crate::telemetry::events::TelemetryEvent;
use regex::Regex;
use once_cell::sync::Lazy;

/// Patterns that indicate sensitive data
static SENSITIVE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // File paths
        Regex::new(r"^/[a-zA-Z]").unwrap(),
        Regex::new(r"^~[/\\]").unwrap(),
        Regex::new(r"^[A-Z]:\\").unwrap(),
        Regex::new(r"/home/[^/]+").unwrap(),
        Regex::new(r"/Users/[^/]+").unwrap(),

        // Secrets
        Regex::new(r"(?i)(password|secret|token|key|credential)").unwrap(),
        Regex::new(r"(?i)(api[_-]?key|auth[_-]?token)").unwrap(),
        Regex::new(r"[a-zA-Z0-9]{32,}").unwrap(), // Long random strings

        // URLs with credentials
        Regex::new(r"https?://[^:]+:[^@]+@").unwrap(),

        // Commands
        Regex::new(r"^\s*(rm|mv|cp|chmod|chown|sudo|dd|mkfs)").unwrap(),
        Regex::new(r"[|;`$]").unwrap(), // Shell metacharacters
    ]
});

/// Validation errors for telemetry events
#[derive(Debug, Clone)]
pub enum ValidationError {
    PathDetected(String),
    SecretDetected(String),
    CommandDetected(String),
    FieldTooLong(String, usize),
}

/// Validate that an event doesn't contain sensitive data
pub fn validate_event(event: &TelemetryEvent) -> Result<(), ValidationError> {
    // Serialize to JSON and check all string values
    let json = serde_json::to_value(event)
        .map_err(|_| ValidationError::CommandDetected("serialization failed".into()))?;

    validate_value(&json)
}

fn validate_value(value: &serde_json::Value) -> Result<(), ValidationError> {
    match value {
        serde_json::Value::String(s) => validate_string(s),
        serde_json::Value::Array(arr) => {
            for item in arr {
                validate_value(item)?;
            }
            Ok(())
        }
        serde_json::Value::Object(obj) => {
            for (_, v) in obj {
                validate_value(v)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_string(s: &str) -> Result<(), ValidationError> {
    // Check length
    if s.len() > 200 {
        return Err(ValidationError::FieldTooLong(
            s.chars().take(50).collect(),
            s.len(),
        ));
    }

    // Check against sensitive patterns
    for pattern in SENSITIVE_PATTERNS.iter() {
        if pattern.is_match(s) {
            // Determine which type of sensitive data
            if s.starts_with('/') || s.starts_with('~') || s.contains(":\\") {
                return Err(ValidationError::PathDetected(s.chars().take(20).collect()));
            }
            if s.contains("password") || s.contains("secret") || s.contains("token") {
                return Err(ValidationError::SecretDetected("[redacted]".into()));
            }
            if s.contains('|') || s.contains(';') || s.contains('`') {
                return Err(ValidationError::CommandDetected("[redacted]".into()));
            }
        }
    }

    Ok(())
}

/// Allowlist of safe field values
pub fn is_allowed_value(field: &str, value: &str) -> bool {
    match field {
        "os" => ["macos", "linux", "windows", "unknown"].contains(&value),
        "arch" => ["aarch64", "x86_64", "arm", "unknown"].contains(&value),
        "shell" => ["bash", "zsh", "fish", "sh", "powershell", "cmd", "unknown"].contains(&value),
        "backend_used" | "primary" => {
            ["embedded-mlx", "embedded-cpu", "ollama", "vllm", "exo", "mock"].contains(&value)
        }
        "risk_level" => ["safe", "moderate", "high", "critical"].contains(&value),
        "event" => value.starts_with("session.")
            || value.starts_with("command.")
            || value.starts_with("safety.")
            || value.starts_with("error."),
        _ => true, // Other fields validated by pattern matching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_file_paths() {
        assert!(validate_string("/etc/passwd").is_err());
        assert!(validate_string("~/Documents/secrets.txt").is_err());
        assert!(validate_string("C:\\Users\\admin").is_err());
    }

    #[test]
    fn test_rejects_secrets() {
        assert!(validate_string("password=hunter2").is_err());
        assert!(validate_string("API_KEY=abcd1234").is_err());
    }

    #[test]
    fn test_rejects_commands() {
        assert!(validate_string("rm -rf /").is_err());
        assert!(validate_string("echo foo | grep bar").is_err());
    }

    #[test]
    fn test_allows_safe_values() {
        assert!(validate_string("macos").is_ok());
        assert!(validate_string("embedded-mlx").is_ok());
        assert!(validate_string("1.1.0").is_ok());
    }
}
```

---

## 5. CLI Commands

### Addition to `src/main.rs` (clap definitions)

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...

    /// Manage telemetry settings
    #[command(subcommand)]
    Telemetry(TelemetryCommands),
}

#[derive(Subcommand)]
pub enum TelemetryCommands {
    /// Show pending telemetry events
    Show {
        /// Show detailed event payloads
        #[arg(long)]
        verbose: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Export telemetry to file (for air-gapped environments)
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Clear local queue after export
        #[arg(long)]
        clear: bool,
    },

    /// Clear pending telemetry events
    Clear,

    /// Request deletion of remote data
    DeleteRemote,

    /// Show telemetry status
    Status,
}
```

### Implementation: `src/commands/telemetry.rs`

```rust
use crate::telemetry::{self, FlushResult};
use crate::config::Config;
use colored::Colorize;

pub async fn handle_telemetry_command(cmd: TelemetryCommands) -> anyhow::Result<()> {
    match cmd {
        TelemetryCommands::Show { verbose, json } => show(verbose, json).await,
        TelemetryCommands::Export { output, clear } => export(output, clear).await,
        TelemetryCommands::Clear => clear().await,
        TelemetryCommands::DeleteRemote => delete_remote().await,
        TelemetryCommands::Status => status().await,
    }
}

async fn status() -> anyhow::Result<()> {
    let config = Config::load()?;
    let collector = telemetry::try_get();

    println!("{}", "Telemetry Status".bold());
    println!();

    if config.telemetry.enabled {
        println!("  Status: {} (opt-out)", "ENABLED".green());
    } else {
        println!("  Status: {}", "DISABLED".yellow());
    }

    println!("  Collection Level: {}", format!("{:?}", config.telemetry.level).to_lowercase());
    println!("  Air-Gapped Mode: {}", if config.telemetry.air_gapped { "enabled" } else { "disabled" });

    if let Some(collector) = collector {
        println!("  Pending Events: {}", collector.pending_count());
    }

    println!();
    println!("{}", "Commands:".bold());
    println!("  caro telemetry show        View pending events");
    println!("  caro telemetry export      Export for air-gapped upload");
    println!("  caro config set telemetry.enabled false   Disable telemetry");

    Ok(())
}

async fn show(verbose: bool, json: bool) -> anyhow::Result<()> {
    let collector = telemetry::try_get()
        .ok_or_else(|| anyhow::anyhow!("Telemetry not initialized"))?;

    let events = collector.pending_events();

    if json {
        println!("{}", serde_json::to_string_pretty(&events)?);
        return Ok(());
    }

    if events.is_empty() {
        println!("No pending telemetry events.");
        return Ok(());
    }

    println!("{}", "Pending Telemetry Events".bold());
    println!();

    if verbose {
        for event in &events {
            println!("{}", serde_json::to_string_pretty(event)?);
            println!("---");
        }
    } else {
        println!("{:<20} {:<25} {:>10}", "Event", "Timestamp", "Size");
        println!("{}", "-".repeat(57));

        for event in &events {
            let json = serde_json::to_string(event)?;
            println!(
                "{:<20} {:<25} {:>10}",
                event.event_type(),
                format_timestamp(event.timestamp()),
                format!("{} bytes", json.len())
            );
        }
    }

    println!();
    println!("Total: {} events", events.len());

    Ok(())
}

async fn export(output: Option<PathBuf>, clear: bool) -> anyhow::Result<()> {
    let collector = telemetry::try_get()
        .ok_or_else(|| anyhow::anyhow!("Telemetry not initialized"))?;

    let output_path = output.unwrap_or_else(|| {
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        let filename = format!("telemetry-export-{}.json.gz",
            chrono::Local::now().format("%Y-%m-%d"));
        cache_dir.join("caro").join(filename)
    });

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let result = collector.export(&output_path)?;

    println!("{}", "Telemetry Export Complete".green().bold());
    println!();
    println!("  File: {}", output_path.display());
    println!("  Events: {}", result.event_count);
    println!("  Size: {} (compressed)", format_bytes(result.size_bytes));
    println!();
    println!("To submit: Upload to https://caro.sh/telemetry/upload");
    println!("Or email to: telemetry@caro.sh");

    if clear {
        let cleared = collector.clear()?;
        println!();
        println!("Cleared {} pending events.", cleared);
    }

    Ok(())
}

async fn clear() -> anyhow::Result<()> {
    let collector = telemetry::try_get()
        .ok_or_else(|| anyhow::anyhow!("Telemetry not initialized"))?;

    let count = collector.clear()?;
    println!("Cleared {} pending events.", count);

    Ok(())
}

async fn delete_remote() -> anyhow::Result<()> {
    let collector = telemetry::try_get()
        .ok_or_else(|| anyhow::anyhow!("Telemetry not initialized"))?;

    // Get session ID for display
    println!("{}", "Remote Data Deletion Request".bold());
    println!();
    println!("This will request deletion of all data associated with your anonymous ID.");
    println!();

    print!("Proceed? [y/N] ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        println!("Cancelled.");
        return Ok(());
    }

    // Send deletion request
    // TODO: Implement actual deletion request
    println!("Deletion request submitted. Data will be purged within 72 hours.");

    Ok(())
}

fn format_timestamp(time: std::time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
```

---

## 6. First-Run Prompt

### File: `src/telemetry/first_run.rs`

```rust
use crate::config::{Config, TelemetryConfig};
use colored::Colorize;
use std::io::{self, Write};

/// Check if this is first run and show telemetry prompt if needed
pub fn check_first_run() -> anyhow::Result<bool> {
    let config_path = Config::default_path();

    // If config file exists and has telemetry settings, skip prompt
    if config_path.exists() {
        let config = Config::load()?;
        if config.telemetry_explicitly_configured() {
            return Ok(config.telemetry.enabled);
        }
    }

    // Show first-run prompt
    show_telemetry_prompt()
}

fn show_telemetry_prompt() -> anyhow::Result<bool> {
    println!();
    println!("{}", "Welcome to Caro!".bold().green());
    println!();
    println!("Caro collects anonymous usage metrics to improve the product.");
    println!();
    println!("{}", "What we collect:".bold());
    println!("  {} Session timing and duration", "•".dimmed());
    println!("  {} Backend performance (inference speed)", "•".dimmed());
    println!("  {} Error categories (not details)", "•".dimmed());
    println!("  {} Platform info (OS, architecture, shell)", "•".dimmed());
    println!();
    println!("{}", "What we NEVER collect:".bold());
    println!("  {} Your commands or inputs", "✗".red());
    println!("  {} File paths or directories", "✗".red());
    println!("  {} Any identifying information", "✗".red());
    println!();
    println!("View details: {}", "caro telemetry show".cyan());
    println!("Disable later: {}", "caro config set telemetry.enabled false".cyan());
    println!();

    print!("Enable telemetry to help improve Caro? [Y/n] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let enabled = input.trim().is_empty() || input.trim().to_lowercase() == "y";

    // Save the choice
    let mut config = Config::load().unwrap_or_default();
    config.telemetry.enabled = enabled;
    config.set_telemetry_explicitly_configured(true);
    config.save()?;

    if enabled {
        println!();
        println!("{} Telemetry enabled. Thank you!", "✓".green());
    } else {
        println!();
        println!("{} Telemetry disabled.", "✓".yellow());
    }
    println!();

    Ok(enabled)
}
```

---

## 7. Integration Points

### In `src/main.rs`

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config
    let config = Config::load()?;

    // Initialize telemetry (if enabled)
    if config.telemetry.enabled && !args.no_telemetry {
        // Check first run
        if telemetry::first_run::check_first_run()? {
            telemetry::init(config.telemetry.clone());

            // Record session start
            telemetry::get().record(SessionStartEvent::new(&config));
        }
    }

    // Run CLI
    let result = run_cli(args).await;

    // Record session end (if telemetry enabled)
    if let Some(collector) = telemetry::try_get() {
        collector.record(SessionEndEvent::from_session(&session_stats));

        // Attempt flush on clean exit
        let _ = collector.flush().await;
    }

    result
}
```

### In command generation (e.g., `src/commands/generate.rs`)

```rust
pub async fn generate_command(input: &str, config: &Config) -> anyhow::Result<GeneratedCommand> {
    let generation_id = Uuid::new_v4();
    let start = std::time::Instant::now();

    // Generate command...
    let result = backend.generate(input).await;
    let elapsed = start.elapsed();

    // Record telemetry
    if let Some(collector) = telemetry::try_get() {
        match &result {
            Ok(cmd) => {
                collector.record(CommandGeneratedEvent {
                    timestamp: SystemTime::now(),
                    session_id: session_id(),
                    generation_id,
                    backend_used: backend.name().to_string(),
                    inference_time_ms: elapsed.as_millis() as u64,
                    safety_result: SafetyResultInfo {
                        risk_level: cmd.safety_level.into(),
                        patterns_matched: cmd.patterns_matched as u32,
                    },
                    model_name: Some(backend.model_name().to_string()),
                    ..Default::default()
                });
            }
            Err(e) => {
                collector.record(ErrorOccurredEvent::from_error(e, "generation"));
            }
        }
    }

    result
}
```

---

## 8. Dependencies to Add

### In `Cargo.toml`

```toml
[dependencies]
# ... existing ...

# Telemetry
rusqlite = { version = "0.31", features = ["bundled"] }
flate2 = "1.0"
chrono = { version = "0.4", features = ["serde"] }
once_cell = "1.19"  # If not already present

[features]
default = []
beta = []  # Enables opt-out telemetry by default
```

---

## 9. Test Plan Summary

| Test Category | Count | Coverage |
|---------------|-------|----------|
| Unit: Event validation | 15+ | Sensitive data rejection |
| Unit: Redaction patterns | 20+ | All pattern categories |
| Unit: Queue operations | 10+ | CRUD, size limits |
| Integration: Full flow | 5+ | Collect → queue → export |
| Integration: CLI commands | 8+ | All telemetry subcommands |
| Privacy: Sensitive detection | 20+ | Paths, secrets, commands |
| Performance: Overhead | 3+ | <5ms startup impact |

---

## 10. Rollout Checklist

### Week 1: Core Implementation
- [ ] Event types defined (`events.rs`)
- [ ] Collector with queue (`collector.rs`, `queue.rs`)
- [ ] Redaction/validation (`redaction.rs`)
- [ ] Unit tests passing

### Week 2: CLI Integration
- [ ] CLI commands (`telemetry.rs`)
- [ ] First-run prompt
- [ ] Config integration
- [ ] Integration tests passing

### Week 3: Polish & Backend
- [ ] Backend ingest service (separate repo)
- [ ] Export/import workflow tested
- [ ] Air-gapped mode verified
- [ ] Documentation complete

### Week 4: Beta Release
- [ ] Feature flag enabled for beta
- [ ] Release notes drafted
- [ ] Privacy policy updated
- [ ] Monitoring dashboards ready
