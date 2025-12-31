# Caro Telemetry Strategy & Beta Readiness Plan

> **Version:** 1.0.0
> **Status:** Draft
> **Last Updated:** 2025-12-30
> **Target Release:** v1.1.0 (Beta)

## Executive Summary

This document defines Caro's telemetry strategy for the alpha→beta→GA transition. The strategy prioritizes:

1. **Privacy-first design** - Metadata over content, local-first storage
2. **Air-gapped compatibility** - Offline spool with manual export
3. **Minimal performance impact** - <5ms overhead on startup
4. **Actionable insights** - Metrics tied to product decisions

---

## [PHASE PLAN]

### Alpha (Current: v1.0.x)

**Goals:**
- Validate core value proposition with early adopters
- Identify critical bugs and UX friction
- Establish baseline performance metrics

**Feedback Mechanisms:**
- GitHub Issues (manual bug reports)
- Direct user interviews (Discord, email)
- Manual performance profiling

**Minimal Instrumentation:**
- Local-only structured logs (already implemented)
- Timing measurements in `GeneratedCommand` struct
- No network telemetry

**Success Criteria:**
- 50+ active alpha users
- <10 critical bugs reported
- Command success rate >85% (manual tracking)

---

### Beta (v1.1.0 - Target: Feb 15, 2026)

**Goals:**
- Scale to 500+ users across target personas
- Validate safety system effectiveness
- Measure real-world performance across platforms
- Build product intuition through data

**Telemetry Model: Opt-Out (Enabled by Default)**

Users are informed on first run with clear explanation:
```
Caro collects anonymous usage metrics to improve the product.
No commands, paths, or sensitive data are ever collected.

To disable: caro config set telemetry.enabled false
To see what's collected: caro telemetry show

Continue? [Y/n]
```

**What's Collected by Default:**
- Session events (start, end, duration)
- Command generation success/failure (no content)
- Safety validation triggers (pattern category only)
- Backend selection and inference timing
- Platform metadata (OS, arch, shell type)
- Error categories (no stack traces)

**What's NEVER Collected:**
- Command content or natural language input
- File paths or working directories
- Environment variables or secrets
- User identity or machine fingerprints
- Network information or hostnames

**Transmission:**
- Batched every 24 hours (configurable)
- HTTPS to `telemetry.caro.sh` (self-hosted, EU-based)
- Local queue for offline operation
- Manual export for air-gapped: `caro telemetry export`

---

### GA (v1.2.0+ - Target: Mar 31, 2026)

**Telemetry Model: Opt-In (Disabled by Default)**

Post-beta, telemetry becomes strictly opt-in:
```toml
# ~/.config/caro/config.toml
[telemetry]
enabled = false  # Default for GA
```

Users who want to contribute data explicitly enable:
```bash
caro config set telemetry.enabled true
```

**Rationale:**
- Beta users accept data collection as value exchange
- GA users expect polished product without obligations
- Maintains trust with privacy-conscious enterprise users
- Complies with stricter enterprise procurement requirements

---

### Upgrade/Migration Behavior

| From → To | Behavior |
|-----------|----------|
| Alpha → Beta | First-run prompt explains opt-out telemetry |
| Beta → GA | Telemetry auto-disabled with notification |
| Any → Air-gapped | Telemetry auto-disabled, offline spool available |

**Migration Script:**
```bash
# On upgrade to GA, if telemetry was default-enabled
if config.telemetry.enabled && !config.telemetry.explicitly_set:
    config.telemetry.enabled = false
    notify("Telemetry disabled for GA. Enable with: caro config set telemetry.enabled true")
```

---

## [METRICS THAT MATTER]

### North Star Metric

**Command Success Rate (CSR)**

> The percentage of generated commands that users execute without modification.

**Definition:**
```
CSR = (commands_executed_as_generated / total_commands_generated) × 100
```

**Why This Metric:**
- Directly measures product value ("Caro gave me the right command")
- Captures both accuracy AND safety (unsafe commands aren't executed)
- Comparable across platforms, backends, and user segments
- Actionable: Low CSR → improve model, prompts, or safety calibration

**Target:** 80%+ CSR for Beta, 90%+ for GA

---

### Supporting Metrics (Tied to JTBD)

| # | Metric | JTBD Connection | Computation | Target |
|---|--------|-----------------|-------------|--------|
| 1 | **Time to First Command (TTFC)** | "Fast command generation" | `first_command_timestamp - session_start` | <3s |
| 2 | **Safety Block Rate** | "Validate commands are safe" | `safety_blocks / total_generations` | 2-5% |
| 3 | **False Positive Rate** | "Don't over-block safe commands" | `user_overrides / safety_blocks` | <10% |
| 4 | **Retry Rate** | "Get the right command" | `sessions_with_retries / total_sessions` | <20% |
| 5 | **Backend Success Rate** | "Reliable inference" | `successful_inferences / total_attempts` per backend | >99% |
| 6 | **Inference Latency P95** | "Fast response" | 95th percentile of `inference_time_ms` | <2s (MLX), <5s (remote) |
| 7 | **Session Completion Rate** | "Complete my task" | `sessions_with_execution / total_sessions` | >70% |
| 8 | **Daily Active Users (DAU)** | "Habitual usage" | Unique anonymous IDs per day | Growth metric |
| 9 | **Error Rate by Category** | "Reliable tool" | `errors_by_category / total_sessions` | <1% per category |
| 10 | **Platform Distribution** | "Cross-platform support" | Sessions by OS/arch/shell | Coverage metric |

---

### Metric Definitions

#### 1. Time to First Command (TTFC)
- **When measured:** From `session_start` to first `command_generated`
- **Excludes:** Model download time (separate metric)
- **Segments:** By backend, platform, first-run vs returning

#### 2. Safety Block Rate
- **Numerator:** Commands blocked by safety validation
- **Denominator:** Total commands generated
- **Segments:** By risk level (Critical, High, Moderate)

#### 3. False Positive Rate
- **Numerator:** Safety blocks where user proceeded with `-y` override
- **Denominator:** Total safety blocks
- **Signal:** High rate → patterns too aggressive

#### 4. Retry Rate
- **Definition:** Sessions where user ran >1 generation before execution
- **Signal:** High rate → model accuracy issues or UX friction

#### 5. Backend Success Rate
- **Per backend:** embedded-mlx, embedded-cpu, ollama, vllm
- **Failure modes:** timeout, connection_error, parse_error, model_error

#### 6. Inference Latency P95
- **Measured at:** `GeneratedCommand.generation_time_ms`
- **Segments:** By backend, model size, platform
- **Excludes:** Network latency for remote backends (measured separately)

---

## [EVENT TAXONOMY]

### Event Design Principles

1. **Structured metadata only** - Never raw content
2. **Categorical values** - Enums over free text
3. **Timing data** - Latency, duration, timestamps
4. **Counts** - Success/failure/retry counts
5. **Hashed identifiers** - Anonymous session correlation

---

### Core Events

#### 1. `session.start`

**When emitted:** CLI invocation begins

**Required fields:**
```json
{
  "event": "session.start",
  "timestamp": "2025-12-30T10:15:30.123Z",
  "session_id": "sha256(machine_id + date)[:16]",
  "caro_version": "1.1.0",
  "platform": {
    "os": "macos",
    "arch": "aarch64",
    "shell": "zsh"
  },
  "backend_config": {
    "primary": "embedded-mlx",
    "fallback_enabled": true
  }
}
```

**Optional fields:**
```json
{
  "safety_level": "moderate",
  "output_format": "plain",
  "is_first_run": false,
  "config_source": "file"
}
```

**Redaction rules:**
- `session_id`: Hashed, rotates daily
- No hostname, username, or IP

---

#### 2. `command.generated`

**When emitted:** After successful command generation

**Required fields:**
```json
{
  "event": "command.generated",
  "timestamp": "2025-12-30T10:15:32.456Z",
  "session_id": "abc123def456",
  "generation_id": "uuid-v4",
  "backend_used": "embedded-mlx",
  "inference_time_ms": 1823,
  "safety_result": {
    "risk_level": "safe",
    "patterns_matched": 0
  }
}
```

**Optional fields:**
```json
{
  "model_name": "Qwen2.5-Coder-1.5B",
  "input_token_count": 45,
  "output_token_count": 23,
  "confidence_score": 0.92,
  "retry_count": 0
}
```

**Redaction rules:**
- NO command content
- NO natural language input
- Token counts only, not tokens themselves

---

#### 3. `command.executed`

**When emitted:** User executes generated command

**Required fields:**
```json
{
  "event": "command.executed",
  "timestamp": "2025-12-30T10:15:35.789Z",
  "session_id": "abc123def456",
  "generation_id": "uuid-v4",
  "execution_mode": "confirmed",
  "modified_before_execution": false
}
```

**Optional fields:**
```json
{
  "exit_code_category": "success",
  "execution_time_ms": 234
}
```

**Redaction rules:**
- `exit_code_category`: Only "success", "error", "timeout" - not actual codes
- NO command output captured

---

#### 4. `safety.triggered`

**When emitted:** Safety validation blocks or warns

**Required fields:**
```json
{
  "event": "safety.triggered",
  "timestamp": "2025-12-30T10:15:31.234Z",
  "session_id": "abc123def456",
  "generation_id": "uuid-v4",
  "risk_level": "critical",
  "pattern_category": "filesystem_destruction",
  "action_taken": "blocked"
}
```

**Optional fields:**
```json
{
  "user_override": false,
  "safety_level_config": "strict"
}
```

**Redaction rules:**
- `pattern_category`: High-level category only (not specific pattern)
- NO matched command content

---

#### 5. `error.occurred`

**When emitted:** Any error during operation

**Required fields:**
```json
{
  "event": "error.occurred",
  "timestamp": "2025-12-30T10:15:33.567Z",
  "session_id": "abc123def456",
  "error_category": "backend_timeout",
  "error_code": "E1001",
  "component": "inference"
}
```

**Optional fields:**
```json
{
  "backend": "ollama",
  "recoverable": true,
  "fallback_attempted": true
}
```

**Redaction rules:**
- NO stack traces
- NO file paths in errors
- Categorized error codes only

---

#### 6. `session.end`

**When emitted:** CLI exits

**Required fields:**
```json
{
  "event": "session.end",
  "timestamp": "2025-12-30T10:16:00.000Z",
  "session_id": "abc123def456",
  "session_duration_ms": 29877,
  "commands_generated": 2,
  "commands_executed": 1,
  "errors_encountered": 0
}
```

**Optional fields:**
```json
{
  "exit_reason": "user_complete",
  "safety_blocks": 0,
  "retries": 1
}
```

---

### Event Categories Summary

| Category | Events | Purpose |
|----------|--------|---------|
| **Session** | `session.start`, `session.end` | User journey, retention |
| **Command** | `command.generated`, `command.executed` | Core value delivery |
| **Safety** | `safety.triggered` | Guardrail effectiveness |
| **Error** | `error.occurred` | Reliability, debugging |
| **Performance** | (embedded in other events) | Latency, optimization |

---

## [PRIVACY & SECURITY GUARANTEES]

### What We NEVER Collect

| Data Type | Why Excluded |
|-----------|--------------|
| Natural language input | Could contain sensitive context |
| Generated command content | Could contain paths, secrets, credentials |
| File paths or directories | Reveals file system structure |
| Environment variables | Often contain secrets, tokens |
| Hostnames or IP addresses | Network topology disclosure |
| Usernames | PII |
| Full error stack traces | Could contain paths, secrets |
| Command output | Arbitrary sensitive content |
| Shell history | Privacy violation |

### How We Avoid Sensitive Data Capture

**1. Allowlist-Only Approach**
```rust
// Only these fields can be transmitted
const ALLOWED_FIELDS: &[&str] = &[
    "event", "timestamp", "session_id", "caro_version",
    "os", "arch", "shell", "backend_used", "inference_time_ms",
    "risk_level", "pattern_category", "error_category", "error_code"
];
```

**2. Enum-Based Values**
```rust
enum RiskLevel { Safe, Moderate, High, Critical }
enum ErrorCategory { BackendTimeout, ParseError, ConfigError, ... }
enum PatternCategory { FilesystemDestruction, PrivilegeEscalation, ... }
```

**3. Hashing for Identifiers**
```rust
// Session ID: anonymous, rotates daily
let session_id = sha256(machine_id + current_date)[..16];

// No persistent user ID across days
```

**4. Truncation Rules**
```rust
// Model names: truncate to 50 chars
// Version strings: truncate to 20 chars
// All string fields: max 100 chars
```

**5. Pre-Transmission Validation**
```rust
fn validate_event(event: &TelemetryEvent) -> Result<(), ValidationError> {
    // Reject if any field matches sensitive patterns
    for field in event.all_string_fields() {
        if looks_like_path(field) { return Err(PathDetected); }
        if looks_like_secret(field) { return Err(SecretDetected); }
        if looks_like_command(field) { return Err(CommandDetected); }
    }
    Ok(())
}
```

### Data Retention

| Data Type | Retention | Justification |
|-----------|-----------|---------------|
| Raw events | 90 days | Short-term debugging |
| Aggregated metrics | 2 years | Trend analysis |
| Session data | 30 days | User journey analysis |
| Error data | 180 days | Bug pattern detection |

**User Rights:**
- Request deletion: `caro telemetry delete-remote`
- Local data: Automatically purged after 7 days

### User-Facing Transparency Text

**First Run Prompt:**
```
Caro collects anonymous usage metrics to improve the product.

What we collect:
  - Session timing and duration
  - Backend performance (inference speed)
  - Error categories (not details)
  - Platform info (OS, architecture, shell)

What we NEVER collect:
  - Your commands or inputs
  - File paths or directories
  - Any identifying information

View details: caro telemetry show
Disable: caro config set telemetry.enabled false
```

**Privacy Policy Link:** https://caro.sh/privacy

---

## [USER CONTROLS UX]

### Configuration

**Config File:** `~/.config/caro/config.toml`
```toml
[telemetry]
# Master switch (default: true for beta, false for GA)
enabled = true

# Collection level: "minimal" | "standard" | "detailed"
# minimal: session start/end, errors only
# standard: + command events, safety events (default)
# detailed: + performance percentiles, retry patterns
level = "standard"

# Transmission settings
batch_interval_hours = 24
offline_spool_max_mb = 10

# Air-gapped mode: collect locally but never transmit
air_gapped = false
```

### CLI Flags

```bash
# Disable telemetry for single session
caro --no-telemetry "list files"

# Permanently disable
caro config set telemetry.enabled false

# Set collection level
caro config set telemetry.level minimal

# Enable air-gapped mode
caro config set telemetry.air_gapped true
```

### Environment Variables

```bash
# Disable via environment (overrides config)
export CARO_TELEMETRY_ENABLED=false

# Set level via environment
export CARO_TELEMETRY_LEVEL=minimal

# CI/CD recommended setting
export CARO_TELEMETRY_ENABLED=false
```

### "Show Me What You Collect" Command

```bash
$ caro telemetry show

Telemetry Status: ENABLED (opt-out)
Collection Level: standard
Air-Gapped Mode: disabled

Pending Events (not yet sent):
┌─────────────────────┬───────────────────────┬─────────────┐
│ Event               │ Timestamp             │ Size        │
├─────────────────────┼───────────────────────┼─────────────┤
│ session.start       │ 2025-12-30 10:15:30   │ 245 bytes   │
│ command.generated   │ 2025-12-30 10:15:32   │ 312 bytes   │
│ command.executed    │ 2025-12-30 10:15:35   │ 189 bytes   │
│ session.end         │ 2025-12-30 10:16:00   │ 201 bytes   │
└─────────────────────┴───────────────────────┴─────────────┘

Total: 4 events, 947 bytes

View event details: caro telemetry show --verbose
View as JSON: caro telemetry show --json
```

### "Show Verbose" Output

```bash
$ caro telemetry show --verbose

Event: session.start
Timestamp: 2025-12-30T10:15:30.123Z
Fields:
  session_id: "a1b2c3d4e5f6g7h8" (anonymous, rotates daily)
  caro_version: "1.1.0"
  platform.os: "macos"
  platform.arch: "aarch64"
  platform.shell: "zsh"
  backend_config.primary: "embedded-mlx"
---
(... more events ...)
```

### Export for Air-Gapped Environments

```bash
# Export pending telemetry to file
$ caro telemetry export

Exported 47 events to: ~/.cache/caro/telemetry-export-2025-12-30.json.gz
Size: 12.3 KB (compressed)

To submit: Upload to https://caro.sh/telemetry/upload
Or email to: telemetry@caro.sh

# Export with custom path
$ caro telemetry export --output /path/to/export.json

# Clear local spool after export
$ caro telemetry export --clear
```

### Delete Local Telemetry

```bash
# Clear pending events
$ caro telemetry clear

Cleared 47 pending events.

# Request remote data deletion (if any was sent)
$ caro telemetry delete-remote

This will request deletion of all data associated with your anonymous ID.
Your current ID: a1b2c3d4e5f6g7h8

Proceed? [y/N] y
Deletion request submitted. Data will be purged within 72 hours.
```

---

## [FEEDBACK LOOP]

### Weekly Beta Review Template

```markdown
# Caro Beta Review - Week of [DATE]

## What Changed This Week
- [ ] Features shipped
- [ ] Bugs fixed
- [ ] Documentation updated

## What Telemetry Says

### North Star: Command Success Rate
- This week: XX% (target: 80%)
- Trend: ↑/↓ X% from last week
- Breakdown by platform: macOS XX%, Linux XX%, Windows XX%

### Key Metrics Dashboard

| Metric | This Week | Last Week | Target | Status |
|--------|-----------|-----------|--------|--------|
| DAU | 000 | 000 | Growth | 🟢/🟡/🔴 |
| TTFC (P50) | X.Xs | X.Xs | <3s | 🟢/🟡/🔴 |
| Inference P95 | X.Xs | X.Xs | <2s | 🟢/🟡/🔴 |
| Safety Block Rate | X% | X% | 2-5% | 🟢/🟡/🔴 |
| Error Rate | X% | X% | <1% | 🟢/🟡/🔴 |
| Retry Rate | X% | X% | <20% | 🟢/🟡/🔴 |

### Insights & Anomalies
1. [Observation]: [Data point]
   - Hypothesis: [Why this might be happening]
   - Action: [What to do about it]

2. [Observation]: [Data point]
   - Hypothesis: [Why this might be happening]
   - Action: [What to do about it]

### Platform & Backend Distribution
- macOS (MLX): XX% of sessions
- macOS (CPU): XX% of sessions
- Linux: XX% of sessions
- Windows: XX% of sessions

- Ollama backend: XX% of remote backend sessions
- vLLM backend: XX% of remote backend sessions

## Qualitative Feedback

### GitHub Issues This Week
- [#XXX]: [Title] - [Theme]
- [#XXX]: [Title] - [Theme]

### Discord/Email Highlights
- [Quote or paraphrase from user]
- [Quote or paraphrase from user]

### User Interview Insights (if any)
- [Key learning from interview]

## Synthesis: What to Build Next

### High Priority (This Week)
1. [Task] - Rationale: [Telemetry insight + qualitative feedback]
2. [Task] - Rationale: [Telemetry insight + qualitative feedback]

### Medium Priority (Next 2 Weeks)
1. [Task]
2. [Task]

### Deprioritized/Stopped
1. [Task] - Reason: [Data showing low impact/usage]

## Open Questions for User Research
1. [Question to investigate via interviews]
2. [Question to investigate via interviews]
```

### Combining Data Sources

```
┌─────────────────────────────────────────────────────────────────┐
│                     INSIGHT GENERATION                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐          │
│  │  Telemetry  │   │   GitHub    │   │  Interviews │          │
│  │   (What)    │   │   (Why)     │   │   (Deep)    │          │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘          │
│         │                 │                 │                  │
│         ▼                 ▼                 ▼                  │
│  ┌─────────────────────────────────────────────────────┐      │
│  │              Weekly Synthesis Meeting               │      │
│  │                                                     │      │
│  │  1. Review telemetry dashboards (15 min)           │      │
│  │  2. Triage new GitHub issues (15 min)              │      │
│  │  3. Discuss user interview insights (15 min)       │      │
│  │  4. Prioritize next actions (15 min)               │      │
│  └─────────────────────────────────────────────────────┘      │
│                           │                                    │
│                           ▼                                    │
│  ┌─────────────────────────────────────────────────────┐      │
│  │              Prioritized Backlog                    │      │
│  │                                                     │      │
│  │  - Data-backed feature requests                    │      │
│  │  - Bug fixes with impact metrics                   │      │
│  │  - UX improvements with usage patterns             │      │
│  └─────────────────────────────────────────────────────┘      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Telemetry-Driven Decision Framework

| Signal | Telemetry Indicator | Qualitative Validation | Decision |
|--------|---------------------|------------------------|----------|
| **Model accuracy issues** | CSR <80%, high retry rate | Users report "wrong commands" | Improve prompts or switch model |
| **Safety too aggressive** | Block rate >5%, FP rate >10% | "Caro blocks safe commands" | Tune pattern sensitivity |
| **Safety too permissive** | Block rate <1% | Security concerns in issues | Add more patterns |
| **Performance regression** | P95 latency spike | "Caro got slow" complaints | Profile and optimize |
| **Backend issues** | Backend success <99% | "Ollama doesn't work" reports | Fix integration or docs |
| **Feature unused** | <1% sessions use feature | No mentions in feedback | Consider deprecation |
| **Platform underserved** | Error rate high on platform | Platform-specific issues | Prioritize platform fixes |

---

## [IMPLEMENTATION NOTES]

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Caro CLI                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │   Command   │───▶│  Telemetry  │───▶│   Local     │        │
│  │   Logic     │    │   Collector │    │   Queue     │        │
│  └─────────────┘    └─────────────┘    └──────┬──────┘        │
│                                               │                │
│                                               ▼                │
│                                        ┌─────────────┐        │
│                                        │   Batcher   │        │
│                                        │  (24h/cfg)  │        │
│                                        └──────┬──────┘        │
│                                               │                │
└───────────────────────────────────────────────┼────────────────┘
                                                │
                    ┌───────────────────────────┴───────────────┐
                    │                                           │
                    ▼                                           ▼
             ┌─────────────┐                           ┌─────────────┐
             │   HTTPS     │                           │   Export    │
             │   Upload    │                           │   File      │
             │ (if online) │                           │(air-gapped) │
             └─────────────┘                           └─────────────┘
                    │
                    ▼
             ┌─────────────┐
             │  telemetry  │
             │  .caro.sh   │
             └─────────────┘
```

### Module Structure

```
src/
├── telemetry/
│   ├── mod.rs           # Public API
│   ├── collector.rs     # Event collection
│   ├── events.rs        # Event type definitions
│   ├── queue.rs         # Local SQLite queue
│   ├── batcher.rs       # Batch transmission logic
│   ├── redaction.rs     # Sensitive data filtering
│   ├── export.rs        # Air-gapped export
│   └── config.rs        # Telemetry configuration
```

### Key Implementation Details

#### 1. Local Queue (SQLite)

```rust
// ~/.cache/caro/telemetry.db
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,  -- JSON
    created_at TEXT NOT NULL,
    transmitted_at TEXT,
    export_id TEXT
);

CREATE INDEX idx_pending ON events(transmitted_at) WHERE transmitted_at IS NULL;
```

**Why SQLite:**
- Already a dependency (many Rust projects use it)
- Reliable, crash-safe storage
- Easy querying for `caro telemetry show`
- Supports offline operation naturally

#### 2. Collector API

```rust
pub struct TelemetryCollector {
    config: TelemetryConfig,
    queue: EventQueue,
}

impl TelemetryCollector {
    /// Record an event (non-blocking, never fails visibly)
    pub fn record(&self, event: impl Into<TelemetryEvent>) {
        if !self.config.enabled {
            return;
        }

        let event = event.into();

        // Validate: reject if sensitive data detected
        if let Err(e) = self.validate(&event) {
            // Log locally, don't record
            tracing::warn!("Telemetry event rejected: {:?}", e);
            return;
        }

        // Queue for later transmission (non-blocking)
        let _ = self.queue.enqueue(event);
    }

    /// Called on graceful shutdown or periodically
    pub async fn flush(&self) -> Result<FlushResult> {
        if self.config.air_gapped {
            return Ok(FlushResult::Spooled);
        }

        let batch = self.queue.pending_batch(MAX_BATCH_SIZE)?;
        if batch.is_empty() {
            return Ok(FlushResult::Empty);
        }

        match self.transmit(&batch).await {
            Ok(_) => {
                self.queue.mark_transmitted(&batch)?;
                Ok(FlushResult::Transmitted(batch.len()))
            }
            Err(e) => {
                // Keep in queue for retry
                tracing::debug!("Telemetry transmission failed, will retry: {:?}", e);
                Ok(FlushResult::Deferred)
            }
        }
    }
}
```

#### 3. Performance Considerations

```rust
// Telemetry collection must not impact CLI performance
impl TelemetryCollector {
    pub fn record(&self, event: impl Into<TelemetryEvent>) {
        // 1. Check enabled flag (single atomic read)
        if !self.config.enabled.load(Ordering::Relaxed) {
            return;  // <1μs path
        }

        // 2. Spawn blocking task for queue write
        //    CLI doesn't wait for this
        let queue = self.queue.clone();
        let event = event.into();
        tokio::task::spawn_blocking(move || {
            let _ = queue.enqueue(event);
        });
    }
}
```

**Performance Budget:**
- Event recording: <100μs (async, non-blocking)
- Startup overhead: <5ms (config read, queue check)
- Flush operation: Background task, no CLI impact

#### 4. Failure Modes

| Failure | Behavior | User Impact |
|---------|----------|-------------|
| SQLite write fails | Log warning, continue | None |
| Network unavailable | Keep in queue, retry next flush | None |
| Queue full (10MB) | Drop oldest events | None |
| Config read fails | Disable telemetry | None |
| Transmission timeout | Retry next flush | None |
| Server error (5xx) | Retry with backoff | None |
| Server reject (4xx) | Drop batch, log error | None |

**Principle:** Telemetry failures NEVER impact CLI functionality.

#### 5. Air-Gapped Export Format

```json
{
  "export_version": "1.0",
  "exported_at": "2025-12-30T10:30:00Z",
  "caro_version": "1.1.0",
  "event_count": 47,
  "events": [
    { "event": "session.start", "timestamp": "...", ... },
    { "event": "command.generated", "timestamp": "...", ... }
  ],
  "checksum": "sha256:abc123..."
}
```

Compressed with gzip for transfer efficiency.

### Test Plan

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_event_validation_rejects_paths() {
        let event = TelemetryEvent::error("Failed at /home/user/.ssh/id_rsa");
        assert!(validate_event(&event).is_err());
    }

    #[test]
    fn test_event_validation_rejects_commands() {
        let event = TelemetryEvent::custom("cmd", "rm -rf /");
        assert!(validate_event(&event).is_err());
    }

    #[test]
    fn test_session_id_rotates_daily() {
        let id1 = generate_session_id("2025-12-30");
        let id2 = generate_session_id("2025-12-31");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_queue_respects_size_limit() {
        let queue = EventQueue::new(Config { max_size_mb: 1 });
        // Fill to limit
        for _ in 0..10000 {
            queue.enqueue(large_event());
        }
        assert!(queue.size_bytes() <= 1_000_000);
    }

    #[test]
    fn test_disabled_telemetry_is_noop() {
        let collector = TelemetryCollector::new(Config { enabled: false });
        collector.record(TelemetryEvent::session_start());
        assert_eq!(collector.queue.len(), 0);
    }
}
```

#### Integration Tests

```rust
#[tokio::test]
async fn test_full_telemetry_flow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = TelemetryConfig {
        enabled: true,
        db_path: temp_dir.path().join("telemetry.db"),
        ..Default::default()
    };

    let collector = TelemetryCollector::new(config);

    // Simulate session
    collector.record(TelemetryEvent::session_start());
    collector.record(TelemetryEvent::command_generated(...));
    collector.record(TelemetryEvent::session_end());

    // Verify queue
    let pending = collector.queue.pending_batch(100).unwrap();
    assert_eq!(pending.len(), 3);

    // Verify export
    let export_path = temp_dir.path().join("export.json");
    collector.export(&export_path).unwrap();
    assert!(export_path.exists());
}

#[tokio::test]
async fn test_cli_flags_disable_telemetry() {
    let output = Command::new("cargo")
        .args(["run", "--", "--no-telemetry", "list files"])
        .output()
        .await
        .unwrap();

    // Verify no telemetry recorded
    let db_path = dirs::cache_dir().unwrap().join("caro/telemetry.db");
    // Check no new events were added
}

#[tokio::test]
async fn test_air_gapped_mode() {
    std::env::set_var("CARO_TELEMETRY_AIR_GAPPED", "true");

    // Run caro
    // Verify events queued locally
    // Verify no network calls attempted
    // Verify export command works
}
```

#### Privacy Tests

```rust
#[test]
fn test_no_sensitive_data_in_events() {
    // Run caro with various inputs containing sensitive data
    let inputs = [
        "delete /etc/passwd",
        "curl https://api.example.com?token=secret123",
        "export AWS_SECRET_KEY=abc123",
    ];

    for input in inputs {
        // Generate command
        // Capture telemetry events
        // Assert no input content in events
        // Assert no paths in events
        // Assert no secrets in events
    }
}
```

---

## Appendix A: Backend Architecture

### Server-Side Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    telemetry.caro.sh                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │   Ingest    │───▶│   Validate  │───▶│   Store     │        │
│  │   (HTTPS)   │    │   + Redact  │    │(TimescaleDB)│        │
│  └─────────────┘    └─────────────┘    └──────┬──────┘        │
│                                               │                │
│                                               ▼                │
│                                        ┌─────────────┐        │
│                                        │  Aggregate  │        │
│                                        │  (hourly)   │        │
│                                        └──────┬──────┘        │
│                                               │                │
│                                               ▼                │
│                                        ┌─────────────┐        │
│                                        │  Dashboard  │        │
│                                        │  (Grafana)  │        │
│                                        └─────────────┘        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Tech Stack:**
- Ingest: Rust (axum) for consistency with CLI
- Storage: TimescaleDB for time-series efficiency
- Dashboard: Grafana for visualization
- Hosting: EU-based for GDPR compliance

### Endpoint Specification

```
POST https://telemetry.caro.sh/v1/events

Headers:
  Content-Type: application/json
  Content-Encoding: gzip
  X-Caro-Version: 1.1.0

Body (JSON, gzipped):
{
  "events": [ ... ],
  "batch_id": "uuid",
  "client_timestamp": "ISO8601"
}

Response:
  200 OK: { "accepted": 47, "rejected": 0 }
  400 Bad Request: { "error": "validation_failed", "details": [...] }
  429 Too Many Requests: { "retry_after": 3600 }
  500 Internal Server Error: { "error": "internal" }
```

---

## Appendix B: Compliance Considerations

### GDPR Compliance

| Requirement | Implementation |
|-------------|----------------|
| Lawful basis | Legitimate interest (product improvement) + Consent (opt-out) |
| Data minimization | Only collect necessary metadata |
| Purpose limitation | Only for product improvement |
| Storage limitation | 90 days raw, 2 years aggregated |
| Accuracy | Automated, no user-provided PII |
| Integrity & confidentiality | HTTPS, encrypted at rest |
| Accountability | Privacy policy, DPA available |

### SOC2 Alignment

For enterprise customers, telemetry design supports:
- **Security:** No sensitive data collection
- **Availability:** Graceful degradation if telemetry fails
- **Confidentiality:** No command content captured
- **Privacy:** Opt-in for GA, user controls

---

## Appendix C: Rollout Checklist

### Pre-Beta Launch

- [ ] Telemetry module implemented and tested
- [ ] Privacy policy updated on caro.sh
- [ ] First-run prompt implemented
- [ ] `caro telemetry show` command working
- [ ] `caro telemetry export` command working
- [ ] Air-gapped mode tested
- [ ] Backend ingest service deployed
- [ ] Grafana dashboards configured
- [ ] Weekly review process documented

### Beta Launch

- [ ] Announce telemetry in release notes
- [ ] Update README with telemetry section
- [ ] Monitor opt-out rate (target: <20%)
- [ ] First weekly review completed
- [ ] Iterate based on initial data

### GA Transition

- [ ] Switch default to opt-in
- [ ] Migration script tested
- [ ] Notify beta users of change
- [ ] Update documentation
- [ ] Archive beta-specific dashboards

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-30 | Initial draft |
