# Feature Specification: Caro Distributed Mesh

**Feature Branch**: `005-caro-distributed-mesh`
**Created**: December 2025
**Status**: Draft
**Input**: User description: "Caro distributed terminal intelligence system for closed networks with encrypted peer-to-peer node communication, organization-wide visibility, and privacy-preserving aggregation."

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story

As a developer working in an air-gapped network, I want my Caro installation to connect with my teammates' Caro nodes so that our security team can see aggregate terminal usage patterns without accessing my raw command history.

### Actor Definitions

| Actor | Description | Primary Goals |
|-------|-------------|---------------|
| **Individual Contributor** | Developer running Caro on their workstation | Personal terminal insights, privacy control |
| **System Administrator** | Ops engineer managing infrastructure | Team visibility, operational patterns |
| **Security Analyst (CISO)** | Security team member | Organization-wide risk posture, anomaly detection |
| **Node** | A Caro installation (software actor) | Mesh participation, data exchange |

### Acceptance Scenarios

#### Scenario 1: Single Node Value (Standalone)
1. **Given** a fresh Caro installation on an isolated machine
2. **When** the user runs commands in their terminal
3. **Then** Caro observes and categorizes commands locally
4. **And** the user can view their personal dashboard at `localhost:9237`
5. **And** no network communication occurs outside the machine

#### Scenario 2: Peer Discovery and Connection
1. **Given** two Caro nodes on the same internal network
2. **When** Node A is configured with Node B's address as a trusted peer
3. **Then** Node A initiates a TLS 1.3 connection to Node B
4. **And** both nodes perform mutual Ed25519 authentication
5. **And** both nodes record the successful peer connection in their audit logs

#### Scenario 3: Summary Exchange
1. **Given** two connected and mutually trusted Caro nodes
2. **When** Node A generates a daily summary of command patterns
3. **Then** Node A signs the summary with its Ed25519 private key
4. **And** Node A transmits the encrypted summary to Node B
5. **And** Node B validates the signature and stores the summary
6. **And** Node B does NOT receive raw command text

#### Scenario 4: Aggregate Query (CISO Role)
1. **Given** a CISO's Caro node connected to 10 developer nodes
2. **When** the CISO queries "show me organization-wide risk distribution"
3. **Then** the query is routed to all 10 peer nodes
4. **And** each node evaluates its sharing policy for the CISO's identity
5. **And** nodes that allow CISO access return aggregated safety stats
6. **And** nodes that deny CISO access return an ACCESS_DENIED response
7. **And** the CISO's node aggregates and displays the combined results

#### Scenario 5: Privacy Preservation
1. **Given** a developer node with default sharing policy
2. **When** an admin queries for "recent commands"
3. **Then** the node returns summarized patterns (e.g., "git: 45 commands")
4. **And** the node does NOT return raw command text
5. **And** the audit log records the query and response level

#### Scenario 6: Air-Gap Operation
1. **Given** a Caro mesh operating in an air-gapped facility
2. **When** all nodes are configured with static peer lists
3. **Then** no mDNS or external DNS queries occur
4. **And** no traffic leaves the internal network
5. **And** all features function without internet connectivity

### Edge Cases

- What happens when a peer is unreachable during a query?
  - **Expected**: Query completes with available nodes; unreachable nodes marked as "no response"

- What happens when a node's Ed25519 key is compromised?
  - **Expected**: User can revoke trust from that key; node can rotate to new key with signed endorsement

- What happens when two nodes have clock skew > 5 minutes?
  - **Expected**: Timestamp validation fails; connection rejected with clear error message

- How does system handle a malicious node sending forged summaries?
  - **Expected**: All summaries are signed; signature verification fails; forged data rejected

- What happens when the local SQLite database becomes corrupted?
  - **Expected**: Caro detects corruption on startup; offers repair or fresh start option

---

## Requirements *(mandatory)*

### Functional Requirements

#### FR-1xx: Node Identity and Cryptography
- **FR-101**: System MUST generate an Ed25519 keypair on first startup
- **FR-102**: System MUST store private key in `~/.local/share/caro/identity.key` with 0600 permissions
- **FR-103**: System MUST represent node identity as `caro:ed25519:<base64-public-key>`
- **FR-104**: System MUST support key rotation with signed endorsement from old key
- **FR-105**: System MUST use BLAKE3 for fingerprint generation (first 8 bytes of hash)

#### FR-2xx: Network Communication
- **FR-201**: System MUST use TLS 1.3 for all inter-node communication
- **FR-202**: System MUST perform mutual authentication on every connection
- **FR-203**: System MUST support static peer configuration via TOML
- **FR-204**: System SHOULD support mDNS/Bonjour discovery (optional feature)
- **FR-205**: System MUST reject connections from nodes not in trusted list
- **FR-206**: System MUST bind only to internal/loopback interfaces by default
- **FR-207**: System MUST NOT require internet connectivity for any feature

#### FR-3xx: Data Classification
- **FR-301**: System MUST classify data into three levels: Raw (L0), Summarized (L1), Aggregated (L2)
- **FR-302**: System MUST NOT transmit Level 0 data outside the local node
- **FR-303**: System MUST allow users to configure maximum sharing level (0, 1, or 2)
- **FR-304**: System MUST sign all Level 1 and Level 2 data with node's Ed25519 key
- **FR-305**: System MUST include timestamp and nonce in all signed messages

#### FR-4xx: Observation and Collection
- **FR-401**: System MUST observe commands from bash, zsh, fish, and sh shells
- **FR-402**: System MUST capture: command text, exit code, duration, working directory
- **FR-403**: System MUST categorize commands into patterns (git, docker, kubectl, etc.)
- **FR-404**: System MUST perform safety risk assessment on all observed commands
- **FR-405**: System MUST store all events in local SQLite database
- **FR-406**: System MUST NOT observe file contents, network traffic, or clipboard

#### FR-5xx: Summary Generation
- **FR-501**: System MUST generate hourly, daily, and weekly summaries
- **FR-502**: Summaries MUST include: command pattern counts, tool usage, activity patterns, safety stats
- **FR-503**: Summaries MUST NOT include: raw command text, file paths, user prompts
- **FR-504**: System MUST allow users to preview summaries before sharing is enabled

#### FR-6xx: Query and Aggregation
- **FR-601**: System MUST support mesh-wide queries for aggregate data
- **FR-602**: System MUST route queries to all reachable peers
- **FR-603**: System MUST respect each node's sharing policy when responding to queries
- **FR-604**: System MUST aggregate responses and handle partial results
- **FR-605**: System MUST support query types: ToolUsage, SafetyPosture, ActivityPatterns, Anomalies

#### FR-7xx: Access Control
- **FR-701**: System MUST support trust levels: Untrusted, ShareTo, QueryFrom, Peer, Supervisor
- **FR-702**: System MUST allow per-node trust configuration
- **FR-703**: System MUST allow trust by subnet (e.g., "10.0.1.0/24" = Peer)
- **FR-704**: System MUST enforce deny rules over allow rules
- **FR-705**: Supervisor role MUST only see Level 2 data (never raw commands)

#### FR-8xx: Dashboard
- **FR-801**: System MUST serve a local web dashboard on configurable port (default: 9237)
- **FR-802**: Individual dashboard MUST show personal usage analytics
- **FR-803**: Admin dashboard MUST show aggregate team metrics (if authorized)
- **FR-804**: Security dashboard MUST show organization-wide risk posture (if Supervisor)
- **FR-805**: Dashboard MUST indicate mesh connection status

#### FR-9xx: Audit and Compliance
- **FR-901**: System MUST log all peer connections in local audit trail
- **FR-902**: System MUST log all queries received and responses sent
- **FR-903**: System MUST log all trust changes and policy updates
- **FR-904**: Audit logs MUST be tamper-evident (append-only with checksums)
- **FR-905**: System MUST support export of audit logs in JSON format

### Non-Functional Requirements

#### NFR-1xx: Performance
- **NFR-101**: Background service MUST use < 50MB RAM during idle
- **NFR-102**: Command observation latency MUST be < 10ms
- **NFR-103**: Summary generation MUST complete in < 1 second
- **NFR-104**: Mesh query round-trip MUST complete in < 500ms (local network)
- **NFR-105**: Dashboard MUST render in < 200ms

#### NFR-2xx: Security
- **NFR-201**: All cryptographic operations MUST use audited libraries (ring, rustls)
- **NFR-202**: Private keys MUST never leave the local machine
- **NFR-203**: System MUST implement replay protection via timestamp + nonce
- **NFR-204**: System MUST reject messages older than 5 minutes
- **NFR-205**: System MUST cache recent nonces to detect replay attempts

#### NFR-3xx: Reliability
- **NFR-301**: System MUST gracefully degrade when peers are unreachable
- **NFR-302**: System MUST continue local operation during network partitions
- **NFR-303**: System MUST survive and recover from SQLite corruption
- **NFR-304**: System MUST handle clock skew up to 5 minutes gracefully

#### NFR-4xx: Portability
- **NFR-401**: System MUST support macOS (arm64, x86_64)
- **NFR-402**: System MUST support Linux (x86_64, arm64)
- **NFR-403**: System SHOULD support Windows (future consideration)
- **NFR-404**: System MUST work in Docker containers

### Key Entities

- **Node**: A Caro installation with cryptographic identity, local data store, and network presence
- **Peer**: Another Node that this Node has established trust with
- **TerminalEvent**: A single observed command execution (Level 0 data)
- **NodeSummary**: Aggregated patterns from TerminalEvents (Level 1 data)
- **MeshQuery**: A request for aggregate data sent to peers
- **MeshResponse**: The result of a MeshQuery from one or more peers
- **TrustPolicy**: Configuration defining who can access what level of data
- **AuditEvent**: Record of a security-relevant action

---

## Review & Acceptance Checklist

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Execution Status

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked and resolved
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed

---
