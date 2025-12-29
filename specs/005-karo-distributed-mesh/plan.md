# Implementation Plan: Caro Distributed Mesh

**Branch**: `005-caro-distributed-mesh` | **Date**: December 2025 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-caro-distributed-mesh/spec.md`

---

## Summary

Evolve Caro from a single-machine CLI tool into a distributed terminal intelligence system. Each node operates independently while optionally participating in an encrypted peer-to-peer mesh for organization-wide visibility. Privacy is preserved through data classification (L0/L1/L2), cryptographic identity (Ed25519), and policy-based access control.

**Primary requirements:**
- Local observation of shell commands with safety assessment
- Background service with embedded web dashboard
- Encrypted peer-to-peer mesh communication (TLS 1.3)
- Privacy-preserving summary exchange (never share raw commands)
- Role-based aggregate queries for security teams

---

## Technical Context

| Aspect | Decision |
|--------|----------|
| **Language/Version** | Rust 1.75+ (edition 2021) |
| **Primary Dependencies** | tokio, axum, ring, rustls, rusqlite, serde |
| **Storage** | SQLite (local events, summaries, audit) |
| **Testing** | cargo test, integration tests with mock nodes |
| **Target Platform** | macOS (arm64, x86_64), Linux (x86_64, arm64) |
| **Project Type** | Single binary with library architecture |
| **Performance Goals** | <50MB RAM idle, <10ms observation latency, <500ms query RTT |
| **Constraints** | Zero internet dependency, air-gap compatible |

---

## Constitution Check

*Validated against cmdai Constitution v1.0.0*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Simplicity** | PASS | Flat module structure; no unnecessary abstractions |
| **II. Library-First** | PASS | All mesh logic in `caro::mesh` library, CLI orchestrates |
| **III. Test-First** | ENFORCED | Contract tests for all message types before implementation |
| **IV. Safety-First** | PASS | All crypto via audited crates; no unsafe blocks |
| **V. Observability** | PASS | Structured logging with tracing; audit trail |

---

## Project Structure

### Documentation (this feature)
```
specs/005-caro-distributed-mesh/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Technology research
├── data-model.md        # Entity definitions
├── quickstart.md        # Validation scenarios
├── tasks.md             # Generated tasks
└── contracts/
    ├── node-identity.md    # Identity schema contract
    ├── message-protocol.md # Wire protocol contract
    ├── query-api.md        # Query/response contract
    └── sharing-policy.md   # Policy configuration contract
```

### Source Code Structure
```
src/
├── main.rs                  # CLI entry point (existing)
├── lib.rs                   # Library exports (existing)
├── backends/                # Inference backends (existing, ADR-001)
├── safety/                  # Command validation (existing)
├── config/                  # Configuration (existing)
├── cache/                   # Model caching (existing)
│
├── mesh/                    # NEW: Distributed mesh system
│   ├── mod.rs              # Mesh module exports
│   ├── identity.rs         # Ed25519 key management
│   ├── peer.rs             # Peer connection handling
│   ├── discovery.rs        # Peer discovery (static, mDNS)
│   ├── transport.rs        # TLS 1.3 transport layer
│   ├── message.rs          # Message types and serialization
│   ├── query.rs            # Query routing and aggregation
│   └── policy.rs           # Sharing policy enforcement
│
├── observation/             # NEW: Terminal observation system
│   ├── mod.rs              # Observation module exports
│   ├── shell.rs            # Shell integration (bash, zsh, fish)
│   ├── event.rs            # Terminal event types
│   ├── watcher.rs          # Background event watcher
│   └── classifier.rs       # Command pattern classification
│
├── storage/                 # NEW: Local storage layer
│   ├── mod.rs              # Storage module exports
│   ├── events.rs           # Event store (SQLite)
│   ├── summaries.rs        # Summary generation and storage
│   ├── audit.rs            # Audit log (append-only)
│   └── schema.sql          # Database schema
│
├── dashboard/               # NEW: Web dashboard
│   ├── mod.rs              # Dashboard module exports
│   ├── server.rs           # Axum HTTP server
│   ├── api.rs              # REST API endpoints
│   ├── auth.rs             # Local authentication
│   └── static/             # Frontend assets
│
└── service/                 # NEW: Background service
    ├── mod.rs              # Service module exports
    ├── daemon.rs           # Daemonization logic
    └── health.rs           # Health check endpoints

tests/
├── contract/
│   ├── message_test.rs     # Message serialization contracts
│   ├── query_test.rs       # Query/response contracts
│   └── policy_test.rs      # Policy enforcement contracts
├── integration/
│   ├── mesh_test.rs        # Multi-node mesh tests
│   ├── observation_test.rs # Shell observation tests
│   └── dashboard_test.rs   # Dashboard API tests
└── unit/
    └── [module-specific tests]
```

---

## Data Model: Core Schemas

### Node Identity Schema

```rust
/// Cryptographic identity of a Caro node (FR-101 through FR-105)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeIdentity {
    /// Ed25519 public key, base64-encoded (32 bytes)
    /// Format: "caro:ed25519:<base64>"
    pub node_id: String,

    /// BLAKE3 fingerprint of public key (first 8 bytes, hex)
    /// For human-readable display: "a1b2c3d4e5f6g7h8"
    pub fingerprint: String,

    /// Optional human-readable name
    pub display_name: Option<String>,

    /// Node capabilities
    pub capabilities: NodeCapabilities,

    /// Identity creation timestamp
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeCapabilities {
    /// Wire protocol version (currently 1)
    pub protocol_version: u32,

    /// Caro software version
    pub caro_version: String,

    /// Maximum data level this node will share
    pub max_share_level: DataLevel,

    /// Whether this node can route aggregate queries
    pub can_aggregate: bool,

    /// Supported query types
    pub supported_queries: Vec<QueryType>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum DataLevel {
    /// Raw data - never shared
    Raw = 0,
    /// Summarized patterns - shared with consent
    Summarized = 1,
    /// Aggregated metrics - shared widely
    Aggregated = 2,
}
```

### Terminal Event Schema (Level 0 - Never Shared)

```rust
/// A single terminal event (FR-401 through FR-405)
/// This data NEVER leaves the local node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalEvent {
    /// Unique event identifier
    pub id: Uuid,

    /// Event timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Shell type
    pub shell: ShellType,

    /// Full command text (NEVER shared)
    pub command: String,

    /// Working directory (NEVER shared)
    pub cwd: PathBuf,

    /// Exit code (0 = success)
    pub exit_code: Option<i32>,

    /// Execution duration in milliseconds
    pub duration_ms: Option<u64>,

    /// Was this command generated by Caro?
    pub caro_generated: bool,

    /// Computed command category
    pub category: CommandCategory,

    /// Safety risk assessment
    pub risk_level: RiskLevel,

    /// Did user confirm execution?
    pub user_confirmed: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Sh,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommandCategory {
    Git,
    Docker,
    Kubernetes,
    Npm,
    Cargo,
    FileOps,
    Network,
    System,
    Editor,
    Other(String),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum RiskLevel {
    Safe = 0,
    Moderate = 1,
    High = 2,
    Critical = 3,
}
```

### Node Summary Schema (Level 1 - Shared with Consent)

```rust
/// Summarized node activity (FR-501 through FR-504)
/// This is the primary unit of data exchange between nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeSummary {
    /// Summary identifier
    pub id: Uuid,

    /// Node that generated this summary
    pub node_id: String,

    /// Time period covered
    pub period: SummaryPeriod,

    /// Command category statistics
    pub category_stats: Vec<CategoryStat>,

    /// Hourly activity distribution (24 buckets)
    pub hourly_activity: [u32; 24],

    /// Daily activity distribution (7 buckets, Mon=0)
    pub daily_activity: [u32; 7],

    /// Safety statistics
    pub safety_stats: SafetyStats,

    /// Total commands in period
    pub total_commands: u32,

    /// Unique command patterns
    pub unique_patterns: u32,

    /// Summary generation timestamp
    pub generated_at: DateTime<Utc>,

    /// Ed25519 signature over summary content
    pub signature: Signature,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SummaryPeriod {
    /// Period start (inclusive)
    pub start: DateTime<Utc>,

    /// Period end (exclusive)
    pub end: DateTime<Utc>,

    /// Period granularity
    pub granularity: Granularity,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Granularity {
    Hourly,
    Daily,
    Weekly,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryStat {
    /// Command category
    pub category: CommandCategory,

    /// Count of commands in this category
    pub count: u32,

    /// Average duration (ms)
    pub avg_duration_ms: u32,

    /// Failure rate (0.0 - 1.0)
    pub failure_rate: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafetyStats {
    /// Commands by risk level
    pub by_risk: HashMap<RiskLevel, u32>,

    /// Commands blocked by safety system
    pub blocked_count: u32,

    /// Risky commands that user confirmed
    pub user_confirmed_risky: u32,

    /// Commands generated by Caro
    pub caro_generated: u32,

    /// Caro commands that user executed
    pub caro_executed: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature {
    /// Signing node's public key fingerprint
    pub signer: String,

    /// Ed25519 signature (64 bytes, base64)
    pub value: String,

    /// Timestamp of signature
    pub timestamp: DateTime<Utc>,
}
```

### Mesh Query Schema (FR-601 through FR-605)

```rust
/// Query sent to mesh for aggregate data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshQuery {
    /// Query identifier for correlation
    pub query_id: Uuid,

    /// Requesting node identity
    pub requester: String,

    /// Query type
    pub query_type: QueryType,

    /// Time range for query
    pub time_range: TimeRange,

    /// Optional filters
    pub filters: Vec<QueryFilter>,

    /// Requester's signature
    pub signature: Signature,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryType {
    /// Aggregate tool/category usage
    ToolUsage,

    /// Safety posture metrics
    SafetyPosture,

    /// Activity pattern analysis
    ActivityPatterns,

    /// Anomaly detection signals
    Anomalies,

    /// Node health status
    NodeHealth,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start of range (inclusive)
    pub start: DateTime<Utc>,

    /// End of range (exclusive)
    pub end: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryFilter {
    /// Field to filter on
    pub field: String,

    /// Filter operator
    pub operator: FilterOperator,

    /// Filter value
    pub value: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    InList,
}
```

### Mesh Response Schema

```rust
/// Response to a mesh query
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshResponse {
    /// Correlation with query
    pub query_id: Uuid,

    /// Responding node identity
    pub responder: String,

    /// Response status
    pub status: ResponseStatus,

    /// Response payload (varies by query type)
    pub data: Option<ResponseData>,

    /// Nodes that contributed to this response
    pub contributors: Vec<String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Responder's signature
    pub signature: Signature,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseStatus {
    /// Query successful
    Success,

    /// Access denied by policy
    AccessDenied,

    /// Query type not supported
    NotSupported,

    /// Internal error
    Error,

    /// No data for requested time range
    NoData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResponseData {
    /// Tool usage aggregation
    ToolUsage(AggregatedToolUsage),

    /// Safety posture metrics
    SafetyPosture(AggregatedSafetyPosture),

    /// Activity patterns
    ActivityPatterns(AggregatedActivityPatterns),

    /// Anomaly signals
    Anomalies(Vec<AnomalySignal>),

    /// Node health status
    NodeHealth(NodeHealthStatus),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregatedToolUsage {
    /// Tool usage by category
    pub by_category: HashMap<CommandCategory, CategoryAggregate>,

    /// Total commands across all contributors
    pub total_commands: u64,

    /// Time range covered
    pub time_range: TimeRange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryAggregate {
    /// Total count
    pub count: u64,

    /// Number of nodes using this category
    pub node_count: u32,

    /// Average duration (ms)
    pub avg_duration_ms: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregatedSafetyPosture {
    /// Distribution by risk level (percentages)
    pub risk_distribution: HashMap<RiskLevel, f32>,

    /// Total blocked commands
    pub blocked_count: u64,

    /// User-confirmed risky commands
    pub confirmed_risky: u64,

    /// Safety coverage (% of commands assessed)
    pub coverage: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregatedActivityPatterns {
    /// Aggregated hourly distribution (normalized 0.0-1.0)
    pub hourly_pattern: [f32; 24],

    /// Aggregated daily distribution (normalized 0.0-1.0)
    pub daily_pattern: [f32; 7],

    /// Peak activity hours
    pub peak_hours: Vec<u8>,

    /// Total active nodes
    pub active_nodes: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnomalySignal {
    /// Anomaly type
    pub anomaly_type: AnomalyType,

    /// Severity (1-10)
    pub severity: u8,

    /// Affected node (fingerprint only)
    pub node_fingerprint: String,

    /// Detection timestamp
    pub detected_at: DateTime<Utc>,

    /// Brief description (no PII)
    pub description: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnomalyType {
    /// Unusual command patterns
    UnusualPattern,

    /// High failure rate
    HighFailureRate,

    /// Unusual activity timing
    UnusualTiming,

    /// Repeated risky commands
    RepeatedRisky,

    /// Deviation from baseline
    BaselineDeviation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeHealthStatus {
    /// Node fingerprint
    pub fingerprint: String,

    /// Is node online
    pub online: bool,

    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,

    /// Protocol version
    pub protocol_version: u32,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Event count (last 24h)
    pub event_count_24h: u32,
}
```

### Trust and Policy Schema (FR-701 through FR-705)

```rust
/// Trust level for a peer node
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TrustLevel {
    /// Not trusted - no data exchange
    Untrusted = 0,

    /// Can receive our summaries
    ShareTo = 1,

    /// Can query our summaries
    QueryFrom = 2,

    /// Full bidirectional trust
    Peer = 3,

    /// Administrative access (Level 2 only)
    Supervisor = 4,
}

/// Sharing policy configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharingPolicy {
    /// Maximum data level to share
    pub max_level: DataLevel,

    /// Default trust for unknown nodes
    pub default_trust: TrustLevel,

    /// Explicit trust assignments by node ID
    pub node_trust: HashMap<String, TrustLevel>,

    /// Trust by subnet (e.g., "10.0.1.0/24" => Peer)
    pub subnet_trust: HashMap<String, TrustLevel>,

    /// Categories to share (if empty, share all)
    pub allowed_categories: Vec<String>,

    /// Categories to never share
    pub denied_categories: Vec<String>,

    /// Enable mDNS discovery
    pub enable_mdns: bool,

    /// Static peer addresses
    pub static_peers: Vec<String>,
}

/// Audit event for compliance (FR-901 through FR-905)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event ID
    pub id: Uuid,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event type
    pub event_type: AuditEventType,

    /// Related peer (if applicable)
    pub peer: Option<String>,

    /// Event details
    pub details: String,

    /// Operation success
    pub success: bool,

    /// Previous entry hash (for tamper evidence)
    pub prev_hash: Option<String>,

    /// This entry's hash
    pub hash: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    /// Node identity created
    IdentityCreated,

    /// Peer connected
    PeerConnected,

    /// Peer disconnected
    PeerDisconnected,

    /// Query received from peer
    QueryReceived,

    /// Response sent to peer
    ResponseSent,

    /// Policy violation (access denied)
    PolicyViolation,

    /// Trust level changed
    TrustChanged,

    /// Key rotation
    KeyRotated,

    /// Summary generated
    SummaryGenerated,

    /// Summary shared
    SummaryShared,
}
```

### Wire Protocol Schema

```rust
/// Message envelope for all inter-node communication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireMessage {
    /// Magic bytes: "CARO" (0x4B41524F)
    pub magic: [u8; 4],

    /// Protocol version
    pub version: u16,

    /// Message type
    pub msg_type: MessageType,

    /// Sender node ID
    pub sender: String,

    /// Message timestamp (for replay protection)
    pub timestamp: DateTime<Utc>,

    /// Random nonce (for replay protection)
    pub nonce: [u8; 16],

    /// Sequence number (for ordered delivery)
    pub sequence: u64,

    /// Message payload (MessagePack encoded)
    pub payload: Vec<u8>,

    /// Ed25519 signature over (version || msg_type || sender || timestamp || nonce || sequence || payload)
    pub signature: [u8; 64],
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u16)]
pub enum MessageType {
    /// Handshake initiation
    Hello = 0x0001,

    /// Handshake response
    HelloAck = 0x0002,

    /// Peer capability exchange
    Capabilities = 0x0003,

    /// Heartbeat/keepalive
    Ping = 0x0010,

    /// Heartbeat response
    Pong = 0x0011,

    /// Push summary to peer
    SummaryPush = 0x0100,

    /// Acknowledge summary receipt
    SummaryAck = 0x0101,

    /// Query request
    Query = 0x0200,

    /// Query response
    QueryResponse = 0x0201,

    /// Key rotation announcement
    KeyRotation = 0x0300,

    /// Key rotation acknowledgment
    KeyRotationAck = 0x0301,

    /// Disconnect notification
    Goodbye = 0xFFFF,
}

/// Handshake message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HelloMessage {
    /// Sender's node identity
    pub identity: NodeIdentity,

    /// Proposed session key material (X25519)
    pub key_exchange: [u8; 32],

    /// Supported protocol versions
    pub supported_versions: Vec<u16>,
}

/// Heartbeat message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PingMessage {
    /// Ping ID for correlation
    pub ping_id: u64,

    /// Local timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PongMessage {
    /// Correlation with ping
    pub ping_id: u64,

    /// Remote timestamp (for RTT calculation)
    pub remote_timestamp: DateTime<Utc>,

    /// Local timestamp
    pub local_timestamp: DateTime<Utc>,
}
```

---

## Phase 0: Research (Completed)

See [research.md](./research.md) for detailed research findings on:

- Ed25519 key management patterns in Rust
- TLS 1.3 implementation with rustls
- SQLite integration patterns for Rust
- Shell integration mechanisms (bash, zsh, fish)
- mDNS/Bonjour discovery in closed networks
- MessagePack serialization performance

---

## Phase 1: Design & Contracts

### Contract Files

1. **contracts/node-identity.md** - Node identity schema, key generation, fingerprinting
2. **contracts/message-protocol.md** - Wire format, message types, serialization
3. **contracts/query-api.md** - Query types, filters, response formats
4. **contracts/sharing-policy.md** - Policy configuration, trust levels, enforcement

### Data Model

See **data-model.md** for complete entity definitions.

### Contract Tests (to be created)

```
tests/contract/
├── identity_test.rs     # NodeIdentity serialization, fingerprint generation
├── message_test.rs      # WireMessage format, signature verification
├── query_test.rs        # MeshQuery/MeshResponse serialization
├── policy_test.rs       # SharingPolicy evaluation logic
└── summary_test.rs      # NodeSummary generation, signature
```

---

## Phase 2: Task Planning Approach

**Task Generation Strategy:**
- Each schema type → serialization contract test
- Each message type → wire format test
- Each module → unit tests
- Each integration point → integration test
- Each scenario → end-to-end test

**Ordering (TDD-compliant):**
1. Contract tests (schemas, signatures) [P]
2. Core libraries (identity, message, policy) [P]
3. Storage layer (SQLite, audit log)
4. Observation layer (shell integration)
5. Mesh layer (peer discovery, transport)
6. Query layer (routing, aggregation)
7. Dashboard layer (HTTP server, API)
8. Service layer (background daemon)
9. Integration tests (multi-node scenarios)
10. End-to-end tests (acceptance scenarios)

**Estimated Output:** 35-40 tasks in tasks.md

---

## Complexity Tracking

| Aspect | Justification |
|--------|---------------|
| Multiple crates | NOT USED - single crate with modules |
| Repository pattern | NOT USED - direct SQLite access |
| Custom crypto | NOT USED - ring/rustls only |
| Complex abstractions | NOT USED - flat module structure |

No constitutional violations identified.

---

## Progress Tracking

**Phase Status:**
- [x] Phase 0: Research complete
- [x] Phase 1: Design complete (this plan)
- [ ] Phase 2: Task planning (tasks.md)
- [ ] Phase 3: Implementation
- [ ] Phase 4: Integration testing
- [ ] Phase 5: Validation

**Gate Status:**
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented (none)

---

*Based on Constitution v1.0.0 and ADR-002 (Caro System Definition)*
