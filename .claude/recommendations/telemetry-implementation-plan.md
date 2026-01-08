# Telemetry Implementation Plan

**Date**: 2026-01-08
**Milestone**: v1.1.0-beta (Due: Jan 31, 2026)
**Status**: Planning Phase
**Priority**: Critical

---

## Executive Summary

Implement privacy-first telemetry infrastructure for v1.1.0-beta to gather anonymous usage data that will inform product decisions. The telemetry page (website/src/pages/telemetry.astro) already documents our approach - this plan implements that promise.

**Key Philosophy**:
- Beta (v1.1.0-beta): **Opt-out** (ON by default) for rapid feedback
- GA (v1.1.0+): **Opt-in** (OFF by default) respecting user privacy

**Success Criteria**:
- ✅ Telemetry opt-out working with clear user consent
- ✅ Air-gapped export/import workflow functional
- ✅ <5ms startup overhead from telemetry
- ✅ Zero sensitive data in collected events
- ✅ Weekly beta review process operational

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   Caro CLI                          │
│                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌────────────┐ │
│  │   Agent     │  │  Backends   │  │   Safety   │ │
│  │             │  │             │  │            │ │
│  └──────┬──────┘  └──────┬──────┘  └─────┬──────┘ │
│         │                │                │        │
│         └────────────────┴────────────────┘        │
│                          │                         │
│                  Emit Events                       │
│                          │                         │
│              ┌───────────▼────────────┐            │
│              │  Telemetry Collector   │            │
│              │  (async, non-blocking) │            │
│              └───────────┬────────────┘            │
│                          │                         │
│                    Redaction                       │
│                          │                         │
│              ┌───────────▼────────────┐            │
│              │   SQLite Event Queue   │            │
│              │   (~/.caro/telemetry/  │            │
│              │    events.db)          │            │
│              └───────────┬────────────┘            │
└──────────────────────────┼─────────────────────────┘
                           │
              ┌────────────▼────────────┐
              │   Batch Upload Worker   │
              │   (every 1 hour)        │
              └────────────┬────────────┘
                           │
              ┌────────────▼────────────┐
              │  telemetry.caro.sh      │
              │  (PostHog)              │
              └─────────────────────────┘
```

---

## Phase 1: Core Telemetry Module (8 hours)

### 1.1 Create Telemetry Module Structure

**File**: `src/telemetry/mod.rs`

```rust
//! Privacy-first telemetry collection
//!
//! ## Privacy Guarantees
//!
//! - Never collects command content or natural language input
//! - Never collects file paths or environment variables
//! - Anonymous session IDs (rotate daily)
//! - All data redacted before storage
//! - User-controlled opt-out
//!
//! ## Architecture
//!
//! 1. Events emitted from various components
//! 2. Async collector queues to SQLite
//! 3. Batch worker uploads periodically
//! 4. Air-gapped mode: local storage only

pub mod collector;
pub mod events;
pub mod storage;
pub mod uploader;
pub mod config;
pub mod consent;

pub use collector::TelemetryCollector;
pub use events::{Event, EventType, SessionId};
pub use config::TelemetryConfig;
```

### 1.2 Event Types

**File**: `src/telemetry/events.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Anonymous session identifier (rotates daily)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Generate anonymous session ID from machine ID + date
    pub fn generate() -> Self {
        use sha2::{Sha256, Digest};

        let machine_id = machine_uid::get().unwrap_or_default();
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let combined = format!("{}{}", machine_id, date);

        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        SessionId(hash[..16].to_string())
    }
}

/// Telemetry event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventType {
    SessionStart {
        version: String,
        platform: String,
        shell_type: String,
        backend_available: Vec<String>,
    },
    SessionEnd {
        duration_ms: u64,
        commands_generated: u32,
        commands_executed: u32,
    },
    CommandGeneration {
        backend: String,
        duration_ms: u64,
        success: bool,
        error_category: Option<String>,
    },
    SafetyValidation {
        risk_level: String,
        action_taken: String, // allowed, blocked, warned
        pattern_category: Option<String>,
    },
    BackendError {
        backend: String,
        error_category: String,
        recoverable: bool,
    },
}

/// Telemetry event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub session_id: SessionId,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
}

impl Event {
    pub fn new(session_id: SessionId, event_type: EventType) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            timestamp: Utc::now(),
            event_type,
        }
    }
}
```

### 1.3 Async Collector

**File**: `src/telemetry/collector.rs`

```rust
use super::{Event, SessionId, storage::TelemetryStorage};
use tokio::sync::mpsc;
use std::sync::Arc;

/// Non-blocking telemetry collector
pub struct TelemetryCollector {
    tx: mpsc::UnboundedSender<Event>,
    session_id: SessionId,
    enabled: bool,
}

impl TelemetryCollector {
    pub fn new(storage: Arc<TelemetryStorage>, enabled: bool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let session_id = SessionId::generate();

        if enabled {
            tokio::spawn(Self::event_loop(storage, rx));
        }

        Self {
            tx,
            session_id,
            enabled,
        }
    }

    /// Emit event (non-blocking)
    pub fn emit(&self, event_type: super::EventType) {
        if !self.enabled {
            return;
        }

        let event = Event::new(self.session_id.clone(), event_type);

        // Fire and forget - never block main thread
        let _ = self.tx.send(event);
    }

    async fn event_loop(
        storage: Arc<TelemetryStorage>,
        mut rx: mpsc::UnboundedReceiver<Event>,
    ) {
        while let Some(event) = rx.recv().await {
            // Store event asynchronously
            if let Err(e) = storage.store_event(&event).await {
                tracing::warn!("Failed to store telemetry event: {}", e);
            }
        }
    }
}
```

---

## Phase 2: SQLite Storage (4 hours)

### 2.1 Storage Layer

**File**: `src/telemetry/storage.rs`

```rust
use super::Event;
use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use tokio::sync::Mutex;

pub struct TelemetryStorage {
    conn: Mutex<Connection>,
}

impl TelemetryStorage {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                data TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON events(timestamp)",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub async fn store_event(&self, event: &Event) -> Result<()> {
        let conn = self.conn.lock().await;

        let data = serde_json::to_string(event)?;

        conn.execute(
            "INSERT INTO events (id, session_id, timestamp, event_type, data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.id.to_string(),
                serde_json::to_string(&event.session_id)?,
                event.timestamp.to_rfc3339(),
                serde_json::to_string(&event.event_type)?,
                data,
            ],
        )?;

        Ok(())
    }

    pub async fn get_pending_events(&self, limit: usize) -> Result<Vec<Event>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT data FROM events
             ORDER BY timestamp ASC
             LIMIT ?1"
        )?;

        let events = stmt.query_map([limit], |row| {
            let data: String = row.get(0)?;
            Ok(serde_json::from_str(&data).unwrap())
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }

    pub async fn delete_events(&self, event_ids: &[String]) -> Result<()> {
        let conn = self.conn.lock().await;

        for id in event_ids {
            conn.execute("DELETE FROM events WHERE id = ?1", params![id])?;
        }

        Ok(())
    }
}
```

---

## Phase 3: Privacy & Redaction (4 hours)

### 3.1 Sensitive Data Redaction

**File**: `src/telemetry/redaction.rs`

```rust
use super::Event;
use regex::Regex;
use once_cell::sync::Lazy;

static PATH_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"/[\w/.-]+").unwrap()
});

static EMAIL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap()
});

/// Validate event contains no sensitive data
pub fn validate_event(event: &Event) -> Result<(), String> {
    let json = serde_json::to_string(event).unwrap();

    // Check for paths
    if PATH_PATTERN.is_match(&json) {
        return Err("Event contains file path".to_string());
    }

    // Check for emails
    if EMAIL_PATTERN.is_match(&json) {
        return Err("Event contains email address".to_string());
    }

    // Check for environment variables
    if json.contains("PATH=") || json.contains("HOME=") {
        return Err("Event contains environment variable".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_paths() {
        let json = r#"{"path": "/Users/test/file.txt"}"#;
        assert!(PATH_PATTERN.is_match(json));
    }

    #[test]
    fn test_detects_emails() {
        let json = r#"{"email": "user@example.com"}"#;
        assert!(EMAIL_PATTERN.is_match(json));
    }
}
```

---

## Phase 4: User Controls (6 hours)

### 4.1 Configuration

**File**: `src/telemetry/config.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Telemetry enabled (opt-out in beta, opt-in in GA)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Telemetry level (minimal, normal, verbose)
    #[serde(default)]
    pub level: TelemetryLevel,

    /// Air-gapped mode (no uploads, local storage only)
    #[serde(default)]
    pub air_gapped: bool,

    /// Upload endpoint
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
}

fn default_enabled() -> bool {
    // Beta: opt-out (true), GA: opt-in (false)
    let version = env!("CARGO_PKG_VERSION");
    version.contains("beta")
}

fn default_endpoint() -> String {
    "https://telemetry.caro.sh/api/events".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TelemetryLevel {
    Minimal,  // Only critical events
    Normal,   // Standard events
    Verbose,  // Detailed debug events
}

impl Default for TelemetryLevel {
    fn default() -> Self {
        Self::Normal
    }
}
```

### 4.2 First-Run Consent

**File**: `src/telemetry/consent.rs`

```rust
use dialoguer::{Confirm, theme::ColorfulTheme};
use colored::*;

pub fn prompt_consent() -> bool {
    println!("\n{}", "━".repeat(60).bright_blue());
    println!("{}", "📊  Telemetry & Privacy".bright_white().bold());
    println!("{}", "━".repeat(60).bright_blue());

    println!("\n{}", "Caro is in beta and collects anonymous usage data to improve the product.".bright_white());
    println!("\n{}", "We collect:".bright_white());
    println!("  {} Session timing and performance metrics", "✓".green());
    println!("  {} Platform info (OS, shell type)", "✓".green());
    println!("  {} Error categories and safety events", "✓".green());

    println!("\n{}", "We NEVER collect:".bright_white());
    println!("  {} Your commands or natural language input", "✗".red());
    println!("  {} File paths or environment variables", "✗".red());
    println!("  {} Any personally identifiable information", "✗".red());

    println!("\n{}", format!("Learn more: {}", "https://caro.sh/telemetry".cyan()));
    println!("{}", "You can disable telemetry anytime with:".bright_black());
    println!("{}", "  caro config set telemetry.enabled false".bright_black());

    println!("\n{}", "━".repeat(60).bright_blue());

    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Enable telemetry to help improve Caro?")
        .default(true)
        .interact()
        .unwrap_or(false)
}
```

### 4.3 CLI Commands

**File**: `src/cli/telemetry.rs`

```rust
use clap::Subcommand;
use anyhow::Result;

#[derive(Debug, Subcommand)]
pub enum TelemetryCommands {
    /// Show queued telemetry events
    Show {
        /// Show only last N events
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Export telemetry data (for air-gapped environments)
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Clear all queued events
    Clear {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show telemetry status and configuration
    Status,
}

pub async fn handle_telemetry(cmd: TelemetryCommands) -> Result<()> {
    match cmd {
        TelemetryCommands::Show { limit } => show_events(limit).await,
        TelemetryCommands::Export { output } => export_events(output).await,
        TelemetryCommands::Clear { force } => clear_events(force).await,
        TelemetryCommands::Status => show_status().await,
    }
}
```

---

## Phase 5: Batch Upload Worker (4 hours)

### 5.1 Uploader

**File**: `src/telemetry/uploader.rs`

```rust
use super::{Event, storage::TelemetryStorage, config::TelemetryConfig};
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct TelemetryUploader {
    storage: Arc<TelemetryStorage>,
    config: TelemetryConfig,
}

impl TelemetryUploader {
    pub fn new(storage: Arc<TelemetryStorage>, config: TelemetryConfig) -> Self {
        Self { storage, config }
    }

    /// Start background upload worker
    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(3600)); // 1 hour

            loop {
                ticker.tick().await;

                if !self.config.air_gapped {
                    if let Err(e) = self.upload_batch().await {
                        tracing::warn!("Telemetry upload failed: {}", e);
                    }
                }
            }
        });
    }

    async fn upload_batch(&self) -> Result<()> {
        let events = self.storage.get_pending_events(100).await?;

        if events.is_empty() {
            return Ok(());
        }

        let client = reqwest::Client::new();
        let response = client
            .post(&self.config.endpoint)
            .json(&events)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        if response.status().is_success() {
            let event_ids: Vec<String> = events.iter()
                .map(|e| e.id.to_string())
                .collect();

            self.storage.delete_events(&event_ids).await?;

            tracing::debug!("Uploaded {} telemetry events", events.len());
        }

        Ok(())
    }
}
```

---

## Phase 6: Integration (4 hours)

### 6.1 Update lib.rs

```rust
pub mod telemetry;

pub use telemetry::{TelemetryCollector, TelemetryConfig};
```

### 6.2 Update main.rs

```rust
use caro::telemetry::{TelemetryCollector, TelemetryStorage, TelemetryConfig, consent};

async fn run() -> Result<()> {
    // Load config
    let config = Config::load()?;

    // First-run consent
    if config.telemetry.first_run {
        let enabled = consent::prompt_consent();
        config.set("telemetry.enabled", enabled)?;
        config.set("telemetry.first_run", false)?;
    }

    // Initialize telemetry
    let storage = Arc::new(TelemetryStorage::new(
        dirs::data_dir()
            .unwrap()
            .join("caro")
            .join("telemetry")
            .join("events.db")
    )?);

    let collector = TelemetryCollector::new(storage.clone(), config.telemetry.enabled);

    // Emit session start
    collector.emit(EventType::SessionStart {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        shell_type: detect_shell().to_string(),
        backend_available: available_backends(),
    });

    // Run CLI
    let result = run_cli(collector.clone()).await;

    // Emit session end
    collector.emit(EventType::SessionEnd {
        duration_ms: session_duration(),
        commands_generated: stats.generated,
        commands_executed: stats.executed,
    });

    result
}
```

### 6.3 Emit Events from Components

```rust
// In agent/mod.rs
collector.emit(EventType::CommandGeneration {
    backend: "embedded".to_string(),
    duration_ms: elapsed.as_millis() as u64,
    success: true,
    error_category: None,
});

// In safety/mod.rs
collector.emit(EventType::SafetyValidation {
    risk_level: "critical".to_string(),
    action_taken: "blocked".to_string(),
    pattern_category: Some("destructive".to_string()),
});
```

---

## Dependencies to Add

**Cargo.toml**:

```toml
# Telemetry dependencies
uuid = { version = "1", features = ["v4", "serde"] }
rusqlite = { version = "0.34", features = ["bundled"] }
machine-uid = "0.5"
```

---

## Testing Plan

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_rotates_daily() {
        let id1 = SessionId::generate();
        // Mock date change
        let id2 = SessionId::generate();
        assert_ne!(id1.0, id2.0);
    }

    #[test]
    fn test_event_redaction() {
        let event = Event::new(
            SessionId::generate(),
            EventType::SessionStart { ... }
        );

        assert!(validate_event(&event).is_ok());
    }

    #[tokio::test]
    async fn test_storage_roundtrip() {
        let storage = TelemetryStorage::new_temp()?;
        let event = Event::new(...);

        storage.store_event(&event).await?;
        let events = storage.get_pending_events(10).await?;

        assert_eq!(events.len(), 1);
    }
}
```

### Integration Tests

1. **Test telemetry can be disabled**:
   ```bash
   caro config set telemetry.enabled false
   caro "list files" --no-telemetry
   # Verify no events in DB
   ```

2. **Test air-gapped mode**:
   ```bash
   caro config set telemetry.air_gapped true
   caro "list files"
   caro telemetry export -o data.json
   # Verify JSON contains events
   ```

3. **Test startup overhead**:
   ```bash
   time caro "list files" --no-telemetry
   time caro "list files" # with telemetry
   # Difference should be <5ms
   ```

---

## Metrics to Track (PostHog)

### North Star Metrics

1. **Command Success Rate (CSR)**: Target 80%+
   - `command_generation.success / total_commands`

2. **Time to First Command (TTFC)**: Target <3s
   - `session_start → first_command_generation`

3. **Safety Block Rate**: Target 2-5%
   - `safety_validation.blocked / total_commands`

4. **Backend Success Rate**: Target >99%
   - `backend_errors / total_commands`

5. **Inference Latency P95**: Target <2s (MLX)
   - `p95(command_generation.duration_ms)`

### Secondary Metrics

- Platform distribution (macOS vs Linux)
- Shell type usage (bash, zsh, fish)
- Backend usage (embedded vs remote)
- Error categories distribution
- Session duration distribution

---

## Release Checklist

### Before Beta Release

- [ ] All code implemented and tested
- [ ] Website telemetry page matches implementation
- [ ] First-run consent prompt tested
- [ ] Air-gapped export/import working
- [ ] PostHog ingest service deployed
- [ ] Grafana dashboards created
- [ ] Privacy audit completed
- [ ] Legal review passed
- [ ] Documentation updated

### Beta Release Day

- [ ] Deploy telemetry.caro.sh
- [ ] Monitor first 24 hours closely
- [ ] Verify no sensitive data in events
- [ ] Check startup overhead <5ms
- [ ] Test opt-out flow

### Post-Beta

- [ ] Weekly review of metrics
- [ ] Identify top failure modes
- [ ] Plan improvements for GA
- [ ] Switch to opt-in for v1.1.0 GA

---

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 1: Core Module | 8h | Events, Collector, async queue |
| Phase 2: Storage | 4h | SQLite storage layer |
| Phase 3: Privacy | 4h | Redaction and validation |
| Phase 4: User Controls | 6h | CLI commands, consent prompt |
| Phase 5: Upload Worker | 4h | Batch uploader |
| Phase 6: Integration | 4h | Integrate into main app |
| **Total** | **30h** | Full telemetry system |

---

## Success Metrics

**Must-Have**:
- ✅ Telemetry collection working
- ✅ Opt-out functional
- ✅ No sensitive data collected
- ✅ <5ms startup overhead
- ✅ Air-gapped mode working

**Quality Gates**:
- ✅ 100% test coverage on privacy/redaction
- ✅ Zero PII in sample of 1000 events
- ✅ Grafana dashboards showing metrics
- ✅ Documentation complete

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Privacy violation | Multi-layer validation, open source audit |
| Performance impact | Async collection, <5ms SLA, benchmarks |
| User backlash | Clear communication, easy opt-out |
| Infrastructure cost | Start small, scale as needed |
| Data retention | 90-day policy, automatic deletion |

---

## Open Questions

1. Should we support manual upload for air-gapped users via web portal?
2. What's the retention policy for raw events? (Proposal: 90 days)
3. Should we allow users to request data deletion? (Proposal: Yes)
4. Do we need GDPR compliance given we collect no PII? (Consult legal)

---

*This plan implements the telemetry approach documented at https://caro.sh/telemetry*
