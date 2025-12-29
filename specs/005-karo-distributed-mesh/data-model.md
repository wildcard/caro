# Data Model: Caro Distributed Mesh

**Document**: Entity Relationship Model
**Version**: 1.0.0
**Date**: December 2025

---

## Entity Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CARO DATA MODEL                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────┐        ┌───────────────┐        ┌───────────────┐       │
│  │   NodeIdentity │◄──────│     Peer      │───────►│   TrustPolicy │       │
│  └───────┬───────┘        └───────────────┘        └───────────────┘       │
│          │                                                                  │
│          │ owns                                                             │
│          ▼                                                                  │
│  ┌───────────────┐        ┌───────────────┐        ┌───────────────┐       │
│  │ TerminalEvent │───────►│  NodeSummary  │───────►│ MeshResponse  │       │
│  │   (Level 0)   │        │   (Level 1)   │        │   (Level 2)   │       │
│  └───────────────┘        └───────────────┘        └───────────────┘       │
│          │                        │                        ▲               │
│          │                        │                        │               │
│          ▼                        ▼                        │               │
│  ┌───────────────┐        ┌───────────────┐        ┌───────────────┐       │
│  │  AuditEvent   │        │  MeshQuery    │────────┤  Aggregation  │       │
│  └───────────────┘        └───────────────┘        └───────────────┘       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Entities

### 1. NodeIdentity

The cryptographic identity of a Caro node.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `node_id` | String | PK, Unique | Format: `caro:ed25519:<base64-pubkey>` |
| `fingerprint` | String(16) | Unique | BLAKE3 hash, first 8 bytes, hex |
| `display_name` | String? | Optional | Human-readable name |
| `public_key` | Bytes(32) | Required | Ed25519 public key |
| `private_key` | Bytes(32) | Local only | Ed25519 private key (never transmitted) |
| `created_at` | DateTime | Required | Identity creation time |
| `capabilities` | JSON | Required | Node capabilities object |

**Relationships:**
- 1:N → TerminalEvent (owns)
- 1:N → NodeSummary (generates)
- 1:N → Peer (connects to)
- 1:N → AuditEvent (logs)

---

### 2. TerminalEvent (Level 0)

A single observed command execution. **Never transmitted.**

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PK | Unique event identifier |
| `node_id` | String | FK → NodeIdentity | Owning node |
| `timestamp` | DateTime | Required, Indexed | Event time (UTC) |
| `shell` | Enum | Required | bash, zsh, fish, sh, other |
| `command` | Text | Required | Full command text |
| `cwd` | Path | Required | Working directory |
| `exit_code` | Int? | Optional | Exit code (null if running) |
| `duration_ms` | Int? | Optional | Execution duration |
| `caro_generated` | Bool | Default: false | Was command from Caro |
| `category` | Enum | Required | Computed category |
| `risk_level` | Enum | Required | safe, moderate, high, critical |
| `user_confirmed` | Bool | Default: false | User confirmed risky command |

**Indices:**
- `idx_event_timestamp` on (node_id, timestamp)
- `idx_event_category` on (node_id, category, timestamp)
- `idx_event_risk` on (node_id, risk_level, timestamp)

---

### 3. NodeSummary (Level 1)

Aggregated patterns from TerminalEvents. Shareable with consent.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PK | Summary identifier |
| `node_id` | String | FK → NodeIdentity | Source node |
| `period_start` | DateTime | Required | Period start (inclusive) |
| `period_end` | DateTime | Required | Period end (exclusive) |
| `granularity` | Enum | Required | hourly, daily, weekly |
| `category_stats` | JSON | Required | Array of CategoryStat |
| `hourly_activity` | Int[24] | Required | Commands per hour |
| `daily_activity` | Int[7] | Required | Commands per weekday |
| `safety_stats` | JSON | Required | SafetyStats object |
| `total_commands` | Int | Required | Total in period |
| `unique_patterns` | Int | Required | Distinct patterns |
| `generated_at` | DateTime | Required | Summary creation time |
| `signature` | JSON | Required | Ed25519 signature |

**Indices:**
- `idx_summary_period` on (node_id, period_start, period_end)
- `idx_summary_granularity` on (granularity, period_start)

---

### 4. Peer

A remote node that this node has connected to.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PK | Local peer record ID |
| `node_id` | String | Unique | Remote node's identity |
| `fingerprint` | String(16) | Required | For display |
| `display_name` | String? | Optional | User-assigned name |
| `address` | String | Required | Last known address:port |
| `trust_level` | Enum | Required | untrusted, share_to, query_from, peer, supervisor |
| `first_seen` | DateTime | Required | First connection time |
| `last_seen` | DateTime | Required | Last successful connection |
| `last_summary_at` | DateTime? | Optional | Last summary received |
| `connection_count` | Int | Default: 0 | Successful connections |
| `failure_count` | Int | Default: 0 | Failed connections |

**Indices:**
- `idx_peer_trust` on (trust_level)
- `idx_peer_last_seen` on (last_seen)

---

### 5. TrustPolicy

Configuration for data sharing and access control.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | Int | PK | Always 1 (singleton) |
| `max_level` | Enum | Required | Max data level to share |
| `default_trust` | Enum | Required | Trust for unknown nodes |
| `node_trust` | JSON | Required | Map<node_id, TrustLevel> |
| `subnet_trust` | JSON | Required | Map<CIDR, TrustLevel> |
| `allowed_categories` | JSON | Required | Categories to share (empty = all) |
| `denied_categories` | JSON | Required | Categories to never share |
| `enable_mdns` | Bool | Default: false | Enable mDNS discovery |
| `static_peers` | JSON | Required | Array of peer addresses |
| `updated_at` | DateTime | Required | Last modification |

---

### 6. MeshQuery

A query sent to peers for aggregate data.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `query_id` | UUID | PK | Correlation ID |
| `requester` | String | FK → NodeIdentity | Requesting node |
| `query_type` | Enum | Required | Type of aggregation |
| `time_start` | DateTime | Required | Query range start |
| `time_end` | DateTime | Required | Query range end |
| `filters` | JSON | Required | Array of QueryFilter |
| `signature` | JSON | Required | Requester's signature |
| `created_at` | DateTime | Required | Query creation time |
| `status` | Enum | Required | pending, in_progress, complete, failed |

---

### 7. MeshResponse

A response to a MeshQuery.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PK | Response ID |
| `query_id` | UUID | FK → MeshQuery | Correlation |
| `responder` | String | Required | Responding node ID |
| `status` | Enum | Required | success, access_denied, not_supported, error, no_data |
| `data` | JSON? | Optional | Response payload |
| `contributors` | JSON | Required | Array of contributing node IDs |
| `timestamp` | DateTime | Required | Response time |
| `signature` | JSON | Required | Responder's signature |

---

### 8. AuditEvent

Tamper-evident log of security-relevant actions.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PK | Event ID |
| `sequence` | BigInt | Unique, Auto | Monotonic sequence |
| `timestamp` | DateTime | Required | Event time |
| `event_type` | Enum | Required | Type of audit event |
| `peer_id` | String? | Optional | Related peer |
| `details` | Text | Required | Event details |
| `success` | Bool | Required | Operation success |
| `prev_hash` | String(64)? | Optional | Previous entry hash |
| `hash` | String(64) | Required | BLAKE3(sequence || timestamp || event_type || details || prev_hash) |

**Indices:**
- `idx_audit_sequence` on (sequence) - enforces ordering
- `idx_audit_type` on (event_type, timestamp)

---

## Enum Definitions

### ShellType
```
bash | zsh | fish | sh | other
```

### CommandCategory
```
git | docker | kubernetes | npm | cargo | file_ops | network | system | editor | other
```

### RiskLevel
```
safe (0) | moderate (1) | high (2) | critical (3)
```

### DataLevel
```
raw (0) | summarized (1) | aggregated (2)
```

### TrustLevel
```
untrusted (0) | share_to (1) | query_from (2) | peer (3) | supervisor (4)
```

### Granularity
```
hourly | daily | weekly
```

### QueryType
```
tool_usage | safety_posture | activity_patterns | anomalies | node_health
```

### ResponseStatus
```
success | access_denied | not_supported | error | no_data
```

### AuditEventType
```
identity_created | peer_connected | peer_disconnected | query_received |
response_sent | policy_violation | trust_changed | key_rotated |
summary_generated | summary_shared
```

---

## Embedded Objects

### CategoryStat
```json
{
  "category": "git",
  "count": 45,
  "avg_duration_ms": 120,
  "failure_rate": 0.02
}
```

### SafetyStats
```json
{
  "by_risk": {
    "safe": 980,
    "moderate": 15,
    "high": 4,
    "critical": 1
  },
  "blocked_count": 2,
  "user_confirmed_risky": 3,
  "caro_generated": 50,
  "caro_executed": 48
}
```

### Signature
```json
{
  "signer": "a1b2c3d4e5f6g7h8",
  "value": "base64-encoded-64-bytes",
  "timestamp": "2025-12-28T10:30:00Z"
}
```

### QueryFilter
```json
{
  "field": "category",
  "operator": "equals",
  "value": "git"
}
```

---

## Database Schema (SQLite)

```sql
-- Node identity (local only)
CREATE TABLE node_identity (
    node_id TEXT PRIMARY KEY,
    fingerprint TEXT UNIQUE NOT NULL,
    display_name TEXT,
    public_key BLOB NOT NULL,
    private_key BLOB NOT NULL,
    capabilities TEXT NOT NULL,  -- JSON
    created_at TEXT NOT NULL
);

-- Terminal events (Level 0, never shared)
CREATE TABLE terminal_events (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES node_identity(node_id),
    timestamp TEXT NOT NULL,
    shell TEXT NOT NULL,
    command TEXT NOT NULL,
    cwd TEXT NOT NULL,
    exit_code INTEGER,
    duration_ms INTEGER,
    caro_generated INTEGER NOT NULL DEFAULT 0,
    category TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    user_confirmed INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_event_timestamp ON terminal_events(node_id, timestamp);
CREATE INDEX idx_event_category ON terminal_events(node_id, category, timestamp);
CREATE INDEX idx_event_risk ON terminal_events(node_id, risk_level, timestamp);

-- Node summaries (Level 1)
CREATE TABLE node_summaries (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES node_identity(node_id),
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    granularity TEXT NOT NULL,
    category_stats TEXT NOT NULL,  -- JSON
    hourly_activity TEXT NOT NULL,  -- JSON array
    daily_activity TEXT NOT NULL,   -- JSON array
    safety_stats TEXT NOT NULL,     -- JSON
    total_commands INTEGER NOT NULL,
    unique_patterns INTEGER NOT NULL,
    generated_at TEXT NOT NULL,
    signature TEXT NOT NULL  -- JSON
);

CREATE INDEX idx_summary_period ON node_summaries(node_id, period_start, period_end);

-- Peers
CREATE TABLE peers (
    id TEXT PRIMARY KEY,
    node_id TEXT UNIQUE NOT NULL,
    fingerprint TEXT NOT NULL,
    display_name TEXT,
    address TEXT NOT NULL,
    trust_level TEXT NOT NULL,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    last_summary_at TEXT,
    connection_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_peer_trust ON peers(trust_level);

-- Trust policy (singleton)
CREATE TABLE trust_policy (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    max_level TEXT NOT NULL,
    default_trust TEXT NOT NULL,
    node_trust TEXT NOT NULL,     -- JSON
    subnet_trust TEXT NOT NULL,   -- JSON
    allowed_categories TEXT NOT NULL,  -- JSON
    denied_categories TEXT NOT NULL,   -- JSON
    enable_mdns INTEGER NOT NULL DEFAULT 0,
    static_peers TEXT NOT NULL,   -- JSON
    updated_at TEXT NOT NULL
);

-- Audit log (append-only)
CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,
    sequence INTEGER UNIQUE NOT NULL,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL,
    peer_id TEXT,
    details TEXT NOT NULL,
    success INTEGER NOT NULL,
    prev_hash TEXT,
    hash TEXT NOT NULL
);

CREATE INDEX idx_audit_sequence ON audit_log(sequence);
CREATE INDEX idx_audit_type ON audit_log(event_type, timestamp);

-- Received summaries from peers
CREATE TABLE received_summaries (
    id TEXT PRIMARY KEY,
    peer_id TEXT NOT NULL REFERENCES peers(id),
    summary TEXT NOT NULL,  -- JSON (NodeSummary)
    received_at TEXT NOT NULL,
    verified INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_received_peer ON received_summaries(peer_id, received_at);
```

---

## Data Flow Diagrams

### Event Collection Flow
```
Terminal → Shell Hook → Watcher → Classifier → Storage
                                      ↓
                              TerminalEvent (L0)
                                      ↓
                              SummaryGenerator
                                      ↓
                              NodeSummary (L1)
```

### Query Flow
```
CISO Dashboard → MeshQuery → Peer Routing → Policy Check → Response
                     ↓              ↓              ↓
              Query signed    Each peer     Access granted/denied
                     ↓         checks          ↓
              Broadcast      own policy   Aggregate responses
```

### Summary Exchange Flow
```
NodeA (Summary) → Sign → Encrypt → TLS → NodeB → Verify → Store
                                            ↓
                                    ReceivedSummary
```

---

*This data model supports the Caro distributed mesh architecture defined in ADR-002.*
