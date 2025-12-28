# ADR-002: Karo System Definition — Distributed Terminal Intelligence

| **Status**     | Proposed                            |
|----------------|-------------------------------------|
| **Date**       | December 2025                       |
| **Authors**    | Caro Maintainers                    |
| **Supersedes** | N/A                                 |
| **Related**    | ADR-001 (LLM Inference Architecture)|

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [Decision Drivers](#decision-drivers)
4. [System Mental Model](#system-mental-model)
5. [Architecture Overview](#architecture-overview)
6. [Node Architecture](#node-architecture)
7. [Distributed Mesh Architecture](#distributed-mesh-architecture)
8. [Data Flow and Schemas](#data-flow-and-schemas)
9. [Access and Role Model](#access-and-role-model)
10. [Trust and Cryptography](#trust-and-cryptography)
11. [Security Considerations](#security-considerations)
12. [Future Direction](#future-direction)
13. [Consequences](#consequences)

---

## Executive Summary

This document defines **Karo** as a distributed terminal intelligence system designed for air-gapped and closed internal networks. Karo evolves from a single-machine CLI tool into a cooperative node network that provides:

- **Individual value**: Personal terminal copilot with inference, safety checks, and usage insights
- **Organizational value**: Aggregate visibility into terminal behavior, security posture, and operational patterns
- **Zero-egress architecture**: No external network communication; all data stays within the trusted network

**Core Tenets:**
- **Local-first, mesh-optional**: Each node is fully functional standalone
- **Air-gap compatible**: Zero internet dependencies after deployment
- **Privacy-preserving aggregation**: Derived insights, not raw surveillance
- **Cryptographic trust**: End-to-end encrypted peer communication
- **Role-aware visibility**: Different views for individuals, admins, and security teams

---

## Context and Problem Statement

### The Evolution

ADR-001 established Karo as a local-first CLI tool for command generation. This ADR extends that vision to address organizational needs:

1. **Individual developers** want terminal intelligence without data leaving their machine
2. **Security teams** need visibility into terminal behavior patterns across the organization
3. **SRE/Ops teams** want to understand operational workflows and detect anomalies
4. **Regulated environments** require air-gap compatibility and data sovereignty

### The Challenge

Design a system that:
1. Provides immediate value on a single machine (no network required)
2. Scales to organization-wide visibility when nodes are connected
3. Operates entirely within closed networks (no external dependencies)
4. Preserves individual privacy while enabling aggregate insights
5. Requires no central infrastructure (no servers, databases, or cloud services)

### Why Not Traditional Approaches?

| Approach | Why Not for Karo? |
|----------|-------------------|
| **Centralized logging** (Splunk, ELK) | Requires infrastructure, not air-gap friendly |
| **Agent-based monitoring** (Datadog) | Phones home, requires internet |
| **SIEM systems** | Heavy infrastructure, not terminal-focused |
| **Shell history sync** | Raw data, no intelligence, privacy concerns |

---

## Decision Drivers

### Primary Drivers

1. **Air-Gap First**: Must work in networks with zero internet connectivity
2. **No Central Infrastructure**: No servers, databases, or coordination points required
3. **Privacy Gradient**: Individual data stays local; only consented summaries are shared
4. **Standalone Value**: Single node must be fully useful without mesh
5. **Cryptographic Security**: All inter-node communication encrypted

### Secondary Drivers

- Minimal resource footprint on individual machines
- Graceful degradation when nodes are unreachable
- Support for heterogeneous environments (macOS, Linux, various shells)
- Auditability of what data is shared

---

## System Mental Model

Karo operates as four simultaneous identities:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         KARO NODE IDENTITY                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │    TERMINAL     │  │     LOCAL       │  │    DISTRIBUTED      │ │
│  │     COPILOT     │  │  OBSERVABILITY  │  │   INTELLIGENCE      │ │
│  │                 │  │     AGENT       │  │       NODE          │ │
│  │  • NL→Command   │  │                 │  │                     │ │
│  │  • Safety check │  │  • Shell history│  │  • Mesh participant │ │
│  │  • Context help │  │  • Process watch│  │  • Encrypted relay  │ │
│  │                 │  │  • Usage patterns│  │  • Aggregate views  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘ │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    ZERO-EGRESS SECURITY SYSTEM                  ││
│  │                                                                 ││
│  │  • Never communicates outside internal network                  ││
│  │  • All external model inference is local                        ││
│  │  • Cryptographic identity per node                              ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

---

## Architecture Overview

### Layered System Design

```
┌─────────────────────────────────────────────────────────────────────┐
│                        PRESENTATION LAYER                           │
├──────────────────┬──────────────────┬───────────────────────────────┤
│   CLI Interface  │  Local Dashboard │    Mesh Dashboard             │
│   (Terminal)     │  (localhost:9237)│    (Role-Based Views)         │
└──────────────────┴──────────────────┴───────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                       APPLICATION LAYER                             │
├──────────────────┬──────────────────┬───────────────────────────────┤
│  Inference       │  Observation     │    Aggregation                │
│  Engine          │  Engine          │    Engine                     │
│  (ADR-001)       │                  │                               │
│                  │  • Shell watcher │    • Local summaries          │
│  • Command gen   │  • Process mon   │    • Cross-node queries       │
│  • Safety check  │  • Context track │    • Pattern detection        │
│  • Risk assess   │  • Event log     │    • Anomaly alerts           │
└──────────────────┴──────────────────┴───────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                          DATA LAYER                                 │
├──────────────────────────────────────────────────────────────────────┤
│  Local Storage                      │  Mesh Communication           │
│  • SQLite event store               │  • Peer discovery             │
│  • Command history                  │  • Encrypted channels         │
│  • Inference cache                  │  • Summary exchange           │
│  • Configuration                    │  • Query routing              │
└─────────────────────────────────────┴───────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                         PLATFORM LAYER                              │
├──────────────────────────────────────────────────────────────────────┤
│  • Shell integration (bash, zsh, fish)                              │
│  • Process observation (procfs, sysctl)                             │
│  • Network stack (TCP/TLS internal only)                            │
│  • Cryptographic primitives (ring, rustls)                          │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Node Architecture

### Single Node Components

```
┌─────────────────────────────────────────────────────────────────────┐
│                           KARO NODE                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    CLI AGENT (Terminal)                      │   │
│  │  caro "list all files modified today"                        │   │
│  │  caro --explain "what does this awk command do?"             │   │
│  │  caro --history                                              │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                  BACKGROUND SERVICE                          │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐ │   │
│  │  │Shell Watcher │ │Process Mon   │ │ Event Processor      │ │   │
│  │  │              │ │              │ │                      │ │   │
│  │  │• Hook into   │ │• Track child │ │• Categorize events   │ │   │
│  │  │  shell       │ │  processes   │ │• Extract patterns    │ │   │
│  │  │• Capture     │ │• Monitor     │ │• Generate summaries  │ │   │
│  │  │  commands    │ │  resources   │ │• Detect anomalies    │ │   │
│  │  └──────────────┘ └──────────────┘ └──────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    LOCAL WEB SERVER                          │   │
│  │  http://localhost:9237                                       │   │
│  │  • Personal dashboard                                        │   │
│  │  • Usage analytics                                           │   │
│  │  • Mesh status (if connected)                                │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    LOCAL DATA STORE                          │   │
│  │  ~/.local/share/karo/                                        │   │
│  │  ├── events.db          # SQLite event store                 │   │
│  │  ├── config.toml        # Node configuration                 │   │
│  │  ├── identity.key       # Node cryptographic identity        │   │
│  │  └── cache/             # Inference & model cache            │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Observation Scope

What a Karo node observes on its machine:

| Category | Data Collected | Purpose |
|----------|---------------|---------|
| **Shell Commands** | Command text, exit codes, duration | Usage patterns, failure analysis |
| **Working Context** | cwd, shell type, user, privileges | Context-aware assistance |
| **Process Tree** | Child processes of terminal | Understanding command effects |
| **Karo Interactions** | Generated commands, user prompts | Quality improvement, usage stats |
| **Timestamps** | When commands executed | Temporal patterns |

What a Karo node **never** collects:
- File contents (only paths if part of command)
- Network traffic or connections
- Keystrokes outside of commands
- Screen contents or clipboard
- Other applications' data

---

## Distributed Mesh Architecture

### Peer-to-Peer Topology

```
┌─────────────────────────────────────────────────────────────────────┐
│                    KARO MESH (Internal Network)                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│     ┌───────────┐         ┌───────────┐         ┌───────────┐      │
│     │  Node A   │◄───────►│  Node B   │◄───────►│  Node C   │      │
│     │ (Dev 1)   │         │ (Dev 2)   │         │ (SRE 1)   │      │
│     └─────┬─────┘         └─────┬─────┘         └─────┬─────┘      │
│           │                     │                     │             │
│           │    ┌───────────┐    │                     │             │
│           └───►│  Node D   │◄───┘                     │             │
│                │ (Admin)   │◄─────────────────────────┘             │
│                └─────┬─────┘                                        │
│                      │                                              │
│                      ▼                                              │
│              ┌───────────────┐                                      │
│              │   Node E      │                                      │
│              │   (CISO)      │                                      │
│              │               │                                      │
│              │ Aggregate     │                                      │
│              │ Dashboard     │                                      │
│              └───────────────┘                                      │
│                                                                     │
│  Legend:                                                            │
│  ◄──────► Encrypted peer connection                                 │
│  All connections are bidirectional, E2E encrypted                   │
│  No central server - any node can query the mesh                    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Mesh Properties

1. **Decentralized**: No coordinator, leader, or central server
2. **Eventually Consistent**: Summaries propagate through gossip
3. **Partition Tolerant**: Nodes work independently if isolated
4. **Encrypted**: All inter-node traffic uses TLS 1.3 with mutual auth
5. **Opt-In**: Nodes choose what to share via sharing policies

### Node Discovery

Within closed networks, nodes discover each other via:

| Method | How It Works | Configuration |
|--------|-------------|---------------|
| **Static Config** | Explicit list of peer addresses | `peers = ["10.0.0.5:9238", "10.0.0.6:9238"]` |
| **Subnet Scan** | Probe known port on subnet | `discovery.subnet = "10.0.0.0/24"` |
| **mDNS/Bonjour** | Multicast DNS service discovery | `discovery.mdns = true` |
| **DNS-SD** | DNS service records in internal DNS | `discovery.dns_sd = "_karo._tcp.internal.corp"` |

**Default**: Static config + optional mDNS (zero external dependencies).

---

## Data Flow and Schemas

### Data Categories

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DATA CLASSIFICATION                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  LEVEL 0: RAW (Never leaves node)                                   │
│  ├── Full command text with arguments                               │
│  ├── File paths and contents                                        │
│  ├── Environment variables                                          │
│  └── User prompts to Karo                                           │
│                                                                     │
│  LEVEL 1: SUMMARIZED (Shared with explicit consent)                 │
│  ├── Command patterns (e.g., "git operations: 45/day")              │
│  ├── Tool usage frequencies                                         │
│  ├── Temporal patterns (e.g., "peak activity: 10-11am")             │
│  └── Risk event counts (e.g., "3 high-risk commands blocked")       │
│                                                                     │
│  LEVEL 2: AGGREGATED (Mesh-wide visibility)                         │
│  ├── Organization-wide tool adoption                                │
│  ├── Cross-team workflow patterns                                   │
│  ├── Anomaly detection signals                                      │
│  └── Security posture metrics                                       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Core Data Schemas

#### Node Identity

```rust
/// Cryptographic identity of a Karo node
struct NodeIdentity {
    /// Ed25519 public key (32 bytes, base64 encoded)
    public_key: String,

    /// Human-readable node name (optional)
    display_name: Option<String>,

    /// Node capabilities and version
    capabilities: NodeCapabilities,

    /// First seen timestamp (by this node)
    first_seen: DateTime<Utc>,

    /// Trust level assigned by local policy
    trust_level: TrustLevel,
}

struct NodeCapabilities {
    /// Protocol version
    protocol_version: u32,

    /// Karo version
    karo_version: String,

    /// Supported sharing levels
    supports_level1: bool,
    supports_level2: bool,

    /// Whether node can serve aggregate queries
    can_aggregate: bool,
}

enum TrustLevel {
    /// Not trusted, no data exchange
    Untrusted,
    /// Can receive our summaries
    ShareTo,
    /// Can query our summaries
    QueryFrom,
    /// Full bidirectional trust
    Peer,
    /// Can see all mesh data (admin/CISO)
    Supervisor,
}
```

#### Terminal Event

```rust
/// A single terminal event (Level 0 - never shared)
struct TerminalEvent {
    /// Unique event ID
    id: Uuid,

    /// When the command was executed
    timestamp: DateTime<Utc>,

    /// The shell type
    shell: ShellType,

    /// Full command text
    command: String,

    /// Working directory
    cwd: PathBuf,

    /// Exit code (if completed)
    exit_code: Option<i32>,

    /// Duration in milliseconds
    duration_ms: Option<u64>,

    /// Was this command generated by Karo?
    karo_generated: bool,

    /// Safety assessment
    risk_level: RiskLevel,

    /// Was user confirmation required?
    required_confirmation: bool,
}

enum ShellType {
    Bash,
    Zsh,
    Fish,
    Sh,
    Other(String),
}

enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}
```

#### Node Summary (Level 1)

```rust
/// Summarized data that can be shared with peers
struct NodeSummary {
    /// Summary period
    period: SummaryPeriod,

    /// Node identity
    node_id: String,  // Public key fingerprint

    /// Command pattern statistics
    command_patterns: Vec<PatternStat>,

    /// Tool usage frequencies
    tool_usage: HashMap<String, u32>,

    /// Temporal activity pattern
    activity_pattern: ActivityPattern,

    /// Safety statistics
    safety_stats: SafetyStats,

    /// Generated at timestamp
    generated_at: DateTime<Utc>,

    /// Cryptographic signature
    signature: String,
}

struct SummaryPeriod {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    granularity: Granularity,
}

enum Granularity {
    Hourly,
    Daily,
    Weekly,
}

struct PatternStat {
    /// Pattern category (e.g., "git", "docker", "file-ops")
    category: String,

    /// Count in period
    count: u32,

    /// Average duration (ms)
    avg_duration_ms: u32,

    /// Failure rate (0.0 - 1.0)
    failure_rate: f32,
}

struct ActivityPattern {
    /// Commands per hour bucket (24 entries)
    hourly_distribution: [u32; 24],

    /// Commands per day of week (7 entries)
    daily_distribution: [u32; 7],

    /// Total commands in period
    total_commands: u32,

    /// Unique command count
    unique_commands: u32,
}

struct SafetyStats {
    /// Commands by risk level
    by_risk_level: HashMap<RiskLevel, u32>,

    /// Blocked commands count
    blocked_count: u32,

    /// User-confirmed risky commands
    confirmed_risky: u32,

    /// Karo-generated commands
    karo_generated: u32,
}
```

#### Mesh Query

```rust
/// Query sent to mesh for aggregate data
struct MeshQuery {
    /// Query ID for correlation
    query_id: Uuid,

    /// Requesting node identity
    requester: String,

    /// Query type
    query_type: QueryType,

    /// Time range
    time_range: TimeRange,

    /// Optional filters
    filters: Vec<QueryFilter>,

    /// Signature proving identity
    signature: String,
}

enum QueryType {
    /// Get aggregated tool usage across mesh
    ToolUsage,

    /// Get safety posture metrics
    SafetyPosture,

    /// Get activity patterns
    ActivityPatterns,

    /// Get anomaly signals
    Anomalies,

    /// Get node health status
    NodeHealth,
}

struct QueryFilter {
    field: String,
    operator: FilterOp,
    value: String,
}

enum FilterOp {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
}
```

#### Mesh Response

```rust
/// Response to a mesh query
struct MeshResponse {
    /// Correlation ID
    query_id: Uuid,

    /// Responding node
    responder: String,

    /// Response data (varies by query type)
    data: ResponseData,

    /// Nodes that contributed to this response
    contributing_nodes: Vec<String>,

    /// Response timestamp
    timestamp: DateTime<Utc>,

    /// Signature
    signature: String,
}

enum ResponseData {
    ToolUsage(AggregatedToolUsage),
    SafetyPosture(AggregatedSafetyPosture),
    ActivityPatterns(AggregatedActivityPatterns),
    Anomalies(Vec<AnomalySignal>),
    NodeHealth(Vec<NodeHealthStatus>),
}
```

---

## Access and Role Model

### Role Hierarchy

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ROLE HIERARCHY                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  INDIVIDUAL CONTRIBUTOR                                     │   │
│  │  • Full access to own node data (Level 0-2)                 │   │
│  │  • Personal dashboard                                        │   │
│  │  • Own usage analytics                                       │   │
│  │  • Controls what is shared                                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  TEAM LEAD / SYSADMIN                                       │   │
│  │  • Level 1-2 data from team nodes (consented)               │   │
│  │  • Team aggregate dashboard                                  │   │
│  │  • Tool adoption metrics                                     │   │
│  │  • Cannot see raw commands                                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  SECURITY TEAM / CISO                                       │   │
│  │  • Level 2 data from all nodes (by policy)                  │   │
│  │  • Organization-wide security posture                        │   │
│  │  • Anomaly detection dashboard                               │   │
│  │  • Risk trend analysis                                       │   │
│  │  • Cannot see raw commands (only patterns)                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Access Without Central Server

In a serverless mesh, access is granted by:

1. **Direct Connection**: User installs Karo on their machine, connects to mesh
2. **Query Routing**: Their node routes queries through the mesh
3. **Policy Enforcement**: Each responding node enforces its sharing policy
4. **Result Aggregation**: Requesting node aggregates responses

```
┌──────────────┐      Query      ┌──────────────┐
│   CISO       │ ───────────────►│   Node A     │
│   Node       │                 │              │
│              │                 │ (checks      │
│              │                 │  policy:     │
│              │                 │  CISO=allow) │
│              │◄─────────────── │              │
│              │   Level 2 data  └──────────────┘
│              │
│              │      Query      ┌──────────────┐
│              │ ───────────────►│   Node B     │
│              │                 │ (policy: OK) │
│              │◄─────────────── │              │
│              │   Level 2 data  └──────────────┘
│              │
│              │      Query      ┌──────────────┐
│              │ ───────────────►│   Node C     │
│              │                 │ (policy:     │
│              │                 │  CISO=deny)  │
│              │◄─────────────── │              │
│              │   ACCESS DENIED └──────────────┘
│              │
│ Aggregate    │
│ A + B        │
└──────────────┘
```

### Sharing Policies

Each node defines its sharing policy:

```toml
# ~/.local/share/karo/config.toml

[sharing]
# What level of data to share
max_level = 2  # 0=none, 1=summaries, 2=aggregated

# Who can query this node (by public key or role)
[sharing.allow]
peers = ["*"]  # Allow all trusted peers
supervisors = ["fingerprint:abc123..."]  # Specific CISO key

# What categories to share
[sharing.categories]
tool_usage = true
activity_patterns = true
safety_stats = true
anomalies = true

# Explicit denials override allows
[sharing.deny]
# Don't share with untrusted nodes
untrusted = true
```

---

## Trust and Cryptography

### Cryptographic Primitives

| Purpose | Algorithm | Implementation |
|---------|-----------|----------------|
| **Node Identity** | Ed25519 | `ring` crate |
| **Key Exchange** | X25519 | `ring` crate |
| **Transport** | TLS 1.3 | `rustls` |
| **Symmetric Encryption** | ChaCha20-Poly1305 | `ring` crate |
| **Hashing** | BLAKE3 | `blake3` crate |
| **Key Derivation** | HKDF-SHA256 | `ring` crate |

### Identity Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                      NODE IDENTITY LIFECYCLE                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. GENERATION (First Run)                                          │
│     ┌─────────────────────────────────────────────────────────┐    │
│     │  Ed25519 keypair generated                              │    │
│     │  Private key stored: ~/.local/share/karo/identity.key   │    │
│     │  Public key = Node ID (base64: "karo:ed25519:Abc123...") │   │
│     │  Fingerprint = BLAKE3(public_key)[0:8] (for display)    │    │
│     └─────────────────────────────────────────────────────────┘    │
│                                                                     │
│  2. PEER INTRODUCTION                                               │
│     ┌─────────────────────────────────────────────────────────┐    │
│     │  Node A ──► "Hello, I am karo:ed25519:Abc123"           │    │
│     │  Node B ──► "Hello, I am karo:ed25519:Def456"           │    │
│     │  Both perform X25519 key agreement for session key      │    │
│     │  TLS 1.3 channel established with mutual authentication │    │
│     └─────────────────────────────────────────────────────────┘    │
│                                                                     │
│  3. TRUST ESTABLISHMENT                                             │
│     ┌─────────────────────────────────────────────────────────┐    │
│     │  Option A: Pre-shared trust (config file)               │    │
│     │    [peers.trusted]                                      │    │
│     │    "karo:ed25519:Def456" = { name = "Bob", role = "dev" }│   │
│     │                                                          │    │
│     │  Option B: TOFU (Trust On First Use) with confirmation   │    │
│     │    "New peer detected: Def456. Trust? [y/N]"            │    │
│     │                                                          │    │
│     │  Option C: Certificate chain (enterprise deployment)     │    │
│     │    Organization root CA signs node certificates         │    │
│     └─────────────────────────────────────────────────────────┘    │
│                                                                     │
│  4. KEY ROTATION                                                    │
│     ┌─────────────────────────────────────────────────────────┐    │
│     │  Nodes can rotate keys while maintaining identity        │    │
│     │  Old key signs endorsement of new key                   │    │
│     │  Grace period for peers to learn new key                │    │
│     └─────────────────────────────────────────────────────────┘    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Trust Domains

For enterprise deployment, trust can be scoped by domain:

```toml
[trust.domains]
# Engineering team
engineering = { subnet = "10.0.1.0/24", role = "peer" }

# Security team (supervisor access)
security = { subnet = "10.0.2.0/24", role = "supervisor" }

# External contractors (no mesh access)
contractors = { subnet = "10.0.3.0/24", role = "untrusted" }
```

### Message Authentication

All inter-node messages are signed:

```rust
struct SignedMessage<T> {
    /// The payload
    payload: T,

    /// Sender's node ID
    sender: String,

    /// Timestamp (prevents replay)
    timestamp: DateTime<Utc>,

    /// Nonce (prevents replay)
    nonce: [u8; 16],

    /// Ed25519 signature over (payload || sender || timestamp || nonce)
    signature: [u8; 64],
}
```

### Replay Protection

1. **Timestamps**: Messages older than 5 minutes are rejected
2. **Nonces**: Recent nonces are cached; duplicates rejected
3. **Sequence Numbers**: Long-lived connections use monotonic sequence numbers

---

## Security Considerations

### Threat Model

| Threat | Mitigation |
|--------|-----------|
| **Network eavesdropping** | TLS 1.3 encryption on all channels |
| **Node impersonation** | Ed25519 signatures on all messages |
| **Replay attacks** | Timestamps + nonces + sequence numbers |
| **Unauthorized access** | Role-based policies, cryptographic identity |
| **Data exfiltration** | No external network access, Level 0 never shared |
| **Compromised node** | Can only share its own data; cannot forge others' |
| **Key compromise** | Key rotation supported; revocation via trust removal |

### Defense in Depth

```
┌─────────────────────────────────────────────────────────────────────┐
│                       SECURITY LAYERS                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Layer 1: Network Isolation                                         │
│  └── Karo only binds to internal interfaces                         │
│  └── Firewall rules can further restrict mesh ports                 │
│                                                                     │
│  Layer 2: Transport Security                                        │
│  └── TLS 1.3 with mutual authentication                            │
│  └── Certificate pinning for known peers                            │
│                                                                     │
│  Layer 3: Message Security                                          │
│  └── All messages signed by sender                                  │
│  └── Replay protection via timestamp/nonce                          │
│                                                                     │
│  Layer 4: Access Control                                            │
│  └── Role-based query permissions                                   │
│  └── Per-node sharing policies                                      │
│                                                                     │
│  Layer 5: Data Classification                                       │
│  └── Level 0 data never leaves node                                 │
│  └── Only derived/summarized data shared                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Audit Trail

Every mesh operation is logged locally:

```rust
struct AuditEvent {
    timestamp: DateTime<Utc>,
    event_type: AuditEventType,
    peer: Option<String>,
    details: String,
    success: bool,
}

enum AuditEventType {
    PeerConnected,
    PeerDisconnected,
    QueryReceived,
    QueryResponded,
    PolicyViolation,
    TrustChange,
    KeyRotation,
}
```

---

## Future Direction

### Short-Term (6 months)

1. **Core Implementation**: Background service, shell integration, local dashboard
2. **Peer Discovery**: Static config, mDNS support
3. **Basic Mesh**: Summary exchange between trusted peers
4. **CLI Dashboard**: `caro dashboard` opens local web UI

### Medium-Term (12 months)

1. **Aggregate Views**: Cross-node query routing and aggregation
2. **Anomaly Detection**: Pattern-based unusual activity detection
3. **Policy Engine**: Fine-grained sharing controls
4. **Enterprise Deployment**: Configuration management, certificate chain trust

### Long-Term Vision

1. **Reactive Agents**: Real-time intervention for risky commands
2. **Continuous Learning**: Organization-specific pattern learning
3. **Policy-Aware Inference**: Commands aligned with internal standards
4. **Compliance Reporting**: Automated security posture reports

---

## Consequences

### Positive

1. **Air-Gap Compatible**: Works in most secure environments
2. **No Infrastructure**: No servers to deploy or maintain
3. **Privacy Preserving**: Raw data never leaves the machine
4. **Individually Useful**: Full value even without mesh
5. **Cryptographically Secure**: Strong authentication and encryption
6. **Auditable**: Complete local audit trail

### Negative

1. **Complexity**: Significant increase over single-node CLI
2. **Resource Usage**: Background service consumes memory
3. **Network Configuration**: Mesh requires network access between nodes
4. **Trust Management**: Peer trust needs initial configuration
5. **Eventual Consistency**: No real-time global view

### Risks

1. **Discovery Reliability**: mDNS may not work in all network environments
2. **Key Management**: Lost identity keys require re-establishing trust
3. **Policy Drift**: Nodes may have inconsistent sharing policies
4. **Query Performance**: Large meshes may have slow aggregate queries

### Mitigations

1. **Multiple Discovery Methods**: Static config as reliable fallback
2. **Key Backup**: Optional encrypted key backup
3. **Policy Templates**: Organization-wide policy distribution
4. **Query Caching**: Cache aggregate results with TTL

---

## Appendix A: Protocol Wire Format

Messages between nodes use a simple framed format:

```
┌─────────────────────────────────────────────────────────────────┐
│  Magic (4 bytes)  │  Version (2)  │  Length (4)  │  Type (2)   │
├─────────────────────────────────────────────────────────────────┤
│                        Payload (variable)                       │
├─────────────────────────────────────────────────────────────────┤
│                      Signature (64 bytes)                       │
└─────────────────────────────────────────────────────────────────┘

Magic: 0x4B41524F ("KARO")
Version: Protocol version (currently 1)
Length: Payload length in bytes
Type: Message type enum
Payload: MessagePack-encoded message body
Signature: Ed25519 signature over (Version || Length || Type || Payload)
```

## Appendix B: Dashboard Mockups

### Individual Dashboard

```
┌─────────────────────────────────────────────────────────────────┐
│  KARO - Personal Terminal Intelligence        localhost:9237   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Today's Activity                    Command Categories         │
│  ─────────────────                   ──────────────────         │
│  Commands: 127                       ████████ git (45)          │
│  Karo-generated: 23                  ██████ docker (32)         │
│  Risky (blocked): 2                  █████ kubectl (28)         │
│  Avg duration: 1.2s                  ███ npm (15)               │
│                                      ██ other (7)               │
│                                                                 │
│  Recent Commands (sanitized)                                    │
│  ───────────────────────────                                    │
│  10:32 | git commit -m "..."        | ✓ Safe                   │
│  10:31 | docker build .             | ✓ Safe                   │
│  10:28 | rm -rf node_modules/       | ⚠ Moderate (confirmed)   │
│  10:25 | kubectl get pods           | ✓ Safe                   │
│                                                                 │
│  Mesh Status: Connected (4 peers)                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Security Dashboard (CISO View)

```
┌─────────────────────────────────────────────────────────────────┐
│  KARO - Organization Security Posture                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Mesh Health                         Risk Distribution          │
│  ───────────                         ─────────────────          │
│  Nodes Online: 42/45                 ████████████ Safe (89%)   │
│  Last 24h Queries: 1,247             ███ Moderate (8%)          │
│  Avg Response: 45ms                  █ High (2%)                │
│                                      ░ Critical (1%)            │
│                                                                 │
│  Anomaly Signals (Last 7 Days)                                  │
│  ─────────────────────────────                                  │
│  ⚠ Node eng-042: Unusual rm patterns (3 incidents)             │
│  ⚠ Subnet 10.0.3.x: High failure rate (15% vs 2% baseline)     │
│  ✓ No privilege escalation attempts detected                   │
│                                                                 │
│  Tool Adoption Trends                                           │
│  ────────────────────                                           │
│  kubectl: ▲ 23% (security training impact?)                    │
│  docker: ─ stable                                               │
│  legacy-script.sh: ▼ 45% (migration successful)                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Appendix C: Related Documents

- [ADR-001: LLM Inference Architecture](./001-llm-inference-architecture.md)
- [Security Settings Guide](../SECURITY_SETTINGS.md)
- [Release Process](../RELEASE_PROCESS.md)

---

*This ADR was authored in December 2025 and represents the target architecture for Karo as a distributed terminal intelligence system. Implementation will proceed in phases as defined in the Future Direction section.*
