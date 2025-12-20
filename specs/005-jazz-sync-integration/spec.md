# Feature Specification: Jazz Sync Integration

**Feature Branch**: `005-jazz-sync-integration`
**Created**: 2025-12-20
**Status**: Draft
**Input**: User description: "Deep integration with Jazz.tools for syncing Caro memories, data, usage history, and preferences between machine installations. Local-first, privacy-aware, CRDT-based E2E encrypted sync similar to Atuin. User data belongs only to the user. Optional anonymous data sharing with maintainers only when explicitly opted-in."

## Execution Flow (main)
```
1. Parse user description from Input
   -> Identified: Multi-device sync system for CLI tool using Jazz.tools
   -> Core values: Privacy-first, user data ownership, local-first, optional sharing
2. Extract key concepts from description
   -> Actors: CLI user, multiple devices, sync relay, (optional) maintainers
   -> Actions: sync memories, sync preferences, sync history, encrypt/decrypt
   -> Data: command history, user preferences, model usage, encrypted blobs
   -> Constraints: E2E encryption, privacy-first, offline-first, user consent
3. For each unclear aspect:
   -> Sync conflict resolution: Use CRDT (automatic merge)
   -> Identity management: Device-based keypairs with optional account
   -> Data schema versioning: Included in CoValue structure
4. Fill User Scenarios & Testing section
   -> Primary: Sync history between machines
   -> Secondary: Share anonymized insights with community
5. Generate Functional Requirements
   -> 25 testable requirements across 5 modules
6. Identify Key Entities
   -> SyncIdentity, CommandMemory, SyncPreferences, UsageMetrics, EncryptedBlob
7. Run Review Checklist
   -> No implementation details (Jazz SDK specifics avoided)
   -> Focus on user value and privacy guarantees
8. Return: SUCCESS (spec ready for planning)
```

---

## Quick Guidelines
- Privacy-first: User data belongs ONLY to the user
- Local-first: Works offline, syncs when convenient
- Opt-in everything: No sync without explicit user consent
- E2E encryption: Data unreadable by relay servers or maintainers
- Simple UX: Complex crypto invisible to users

---

## Vision Statement

**Caro Sync** enables users to seamlessly access their command history, preferences, and learned patterns across all their machines while maintaining absolute ownership and privacy of their data. Like Atuin for shell history, but for AI-assisted command generation with even stronger privacy guarantees.

### Core Philosophy

1. **Your Data, Your Control**: Data never leaves your devices unencrypted. Even sync relays see only encrypted blobs.

2. **Local-First, Cloud-Optional**: Everything works offline. Sync is a convenience layer, not a requirement.

3. **Zero-Knowledge Sync**: Relay servers facilitate sync without knowing what they're syncing. No user tracking, no analytics, no metadata harvesting.

4. **Opt-In Sharing**: If users choose to help improve Caro, they can share anonymized, aggregated insights. Never individual commands, never identifiable data.

---

## User Scenarios & Testing

### Primary User Story: Multi-Device Sync
**As a** developer using Caro on multiple machines (laptop, desktop, work computer)
**I want** my command history and preferences to sync automatically
**So that** I have the same personalized experience everywhere without manual setup

**Flow**:
1. User installs Caro on Device A and uses it for a week
2. User installs Caro on Device B
3. User runs `caro sync init` on Device B
4. System generates a sync key (displayed as memorable words like Atuin's BIP39 haiku)
5. User enters sync key from Device A (or scans QR code)
6. Devices establish encrypted sync relationship
7. Command history, preferences, and patterns sync automatically
8. User works on either device with identical experience

### Secondary User Story: Fresh Start with Cloud Backup
**As a** user setting up a new machine
**I want** to restore my Caro configuration from a secure backup
**So that** I don't lose my command history when changing devices

**Flow**:
1. User's old device is lost/replaced
2. User installs Caro on new device
3. User runs `caro sync restore`
4. User enters their recovery phrase (12-24 words)
5. System connects to relay and downloads encrypted data
6. System decrypts data locally using recovery phrase
7. User has full history restored

### Tertiary User Story: Privacy-Respecting Community Contribution
**As a** Caro user who wants to help improve the product
**I want** to optionally share anonymized usage patterns
**So that** maintainers can understand common use cases without accessing my commands

**Flow**:
1. User runs `caro community contribute`
2. System explains what will be shared (counts, categories, never commands)
3. User reviews and approves contribution
4. System generates anonymized report (e.g., "100 file operations, 50 git commands")
5. Report is signed but not linked to user identity
6. User can revoke consent at any time

### Acceptance Scenarios

#### Sync Initialization
1. **Given** user has never synced, **When** user runs `caro sync init`, **Then** system generates new identity with memorable recovery phrase
2. **Given** user has existing sync identity, **When** user runs `caro sync init` on new device, **Then** system prompts for recovery phrase
3. **Given** invalid recovery phrase entered, **When** system attempts to decrypt, **Then** system provides clear error without leaking information

#### Cross-Device Sync
1. **Given** user runs command on Device A, **When** Device B comes online, **Then** command appears in Device B history within 30 seconds
2. **Given** user modifies preference on Device A, **When** Device B syncs, **Then** preference is updated on Device B
3. **Given** both devices are offline, **When** user runs commands on both, **Then** histories merge without conflict when online
4. **Given** user deletes command from history on Device A, **When** Device B syncs, **Then** command is removed from Device B

#### Privacy Guarantees
1. **Given** user has synced commands, **When** relay server is compromised, **Then** attacker sees only encrypted blobs
2. **Given** user has never opted into community sharing, **When** maintainers query, **Then** zero data about user is accessible
3. **Given** user disables sync, **When** system operates, **Then** no network requests are made for sync purposes

#### Offline Operation
1. **Given** user is offline, **When** user runs commands, **Then** commands are stored locally for later sync
2. **Given** user is offline, **When** user queries history, **Then** full local history is available
3. **Given** user was offline for 30 days, **When** user comes online, **Then** sync completes without data loss

### Edge Cases

#### Identity Management
- What happens when recovery phrase is lost? (Cannot recover - by design)
- What happens when user wants to rotate keys? (Generate new identity, migrate data)
- What happens when user wants to sync with untrusted device? (Not supported - all devices are equal)
- How are device additions authorized? (Recovery phrase grants full access)

#### Sync Conflicts
- What happens when same preference is modified on two devices offline? (CRDT merge with last-write-wins for simple values)
- What happens when command is deleted on one device while being run on another? (Deletion wins after sync)
- What happens when schema versions differ between devices? (Older device upgrades or warns)

#### Data Management
- How much data is synced by default? (Last 90 days, configurable)
- What is the storage limit? (10,000 commands by default)
- Can user selectively exclude commands from sync? (Yes, via patterns or manual removal)
- How are large commands handled? (Truncated or excluded from sync)

#### Privacy Edge Cases
- What happens if user accidentally syncs sensitive command? (Can delete from all devices)
- What if relay is operated by malicious actor? (E2E encryption prevents access)
- What metadata is visible to relay? (Device ID, timestamp, blob size - NOT content)

---

## Requirements

### Functional Requirements

#### Sync Identity Module (FR-SI001 to FR-SI005)
- **FR-SI001**: System MUST generate cryptographically secure sync identities using standard algorithms
- **FR-SI002**: System MUST display recovery phrase in human-readable format (BIP39-style words)
- **FR-SI003**: System MUST derive encryption keys deterministically from recovery phrase
- **FR-SI004**: System MUST support adding new devices using only the recovery phrase
- **FR-SI005**: System MUST allow identity revocation with new identity generation and data migration

#### Data Encryption Module (FR-DE001 to FR-DE005)
- **FR-DE001**: System MUST encrypt all sync data client-side before transmission
- **FR-DE002**: System MUST use authenticated encryption to prevent tampering
- **FR-DE003**: System MUST never transmit plaintext data, keys, or recovery phrases
- **FR-DE004**: System MUST support key rotation without losing access to historical data
- **FR-DE005**: System MUST securely store local keys in platform-appropriate secure storage

#### Sync Transport Module (FR-ST001 to FR-ST005)
- **FR-ST001**: System MUST support syncing through Jazz-compatible relay infrastructure
- **FR-ST002**: System MUST support self-hosted relay servers for maximum privacy
- **FR-ST003**: System MUST handle network failures gracefully with automatic retry
- **FR-ST004**: System MUST minimize metadata exposure to relay servers
- **FR-ST005**: System MUST support sync through firewalls and NAT (WebSocket-based)

#### Data Synchronization Module (FR-DS001 to FR-DS005)
- **FR-DS001**: System MUST sync command history across devices with CRDT-based conflict resolution
- **FR-DS002**: System MUST sync user preferences with automatic merge semantics
- **FR-DS003**: System MUST sync model usage statistics for consistent experience
- **FR-DS004**: System MUST support selective sync (include/exclude patterns)
- **FR-DS005**: System MUST provide sync status visibility to users

#### Privacy & Consent Module (FR-PC001 to FR-PC005)
- **FR-PC001**: System MUST be fully functional without any sync features enabled
- **FR-PC002**: System MUST require explicit opt-in for any network communication
- **FR-PC003**: System MUST allow users to delete synced data from all devices
- **FR-PC004**: System MUST support optional anonymized usage sharing with clear consent
- **FR-PC005**: System MUST allow users to export all their data in readable format

### Non-Functional Requirements

#### Security (NFR-S001 to NFR-S004)
- **NFR-S001**: Encryption MUST use AES-256-GCM or equivalent strength algorithm
- **NFR-S002**: Key derivation MUST use Argon2 or equivalent memory-hard function
- **NFR-S003**: No sync data MUST be accessible without recovery phrase, even to Caro maintainers
- **NFR-S004**: Recovery phrase MUST provide 128+ bits of entropy

#### Performance (NFR-P001 to NFR-P004)
- **NFR-P001**: Sync operations MUST NOT impact CLI startup time (async/background)
- **NFR-P002**: Initial sync of 10,000 commands MUST complete within 60 seconds
- **NFR-P003**: Incremental sync MUST complete within 5 seconds for typical usage
- **NFR-P004**: Local operations MUST function identically with sync enabled or disabled

#### Reliability (NFR-R001 to NFR-R004)
- **NFR-R001**: System MUST handle extended offline periods (30+ days) without data loss
- **NFR-R002**: System MUST recover gracefully from interrupted sync operations
- **NFR-R003**: System MUST maintain data integrity across schema migrations
- **NFR-R004**: System MUST provide clear diagnostics for sync failures

#### Usability (NFR-U001 to NFR-U004)
- **NFR-U001**: Sync setup MUST be completable in under 2 minutes
- **NFR-U002**: Recovery phrase MUST be memorable (12-24 common English words)
- **NFR-U003**: Sync status MUST be queryable via simple CLI command
- **NFR-U004**: Error messages MUST provide actionable guidance without technical jargon

### Key Entities

- **SyncIdentity**: Represents a user's cryptographic identity for sync (recovery phrase, derived keys, device list, created timestamp)

- **EncryptedBlob**: A piece of synced data (encrypted payload, nonce, associated device ID, schema version)

- **CommandMemory**: A command execution record for sync (command hash, prompt text, generated command, timestamp, execution result, safety level, tags)

- **SyncPreferences**: User preferences that sync (safety level, default shell, model preferences, exclusion patterns, sync frequency)

- **UsageMetrics**: Anonymizable usage data (command category counts, success rates, model usage distribution, time patterns - NO actual commands)

- **DeviceRecord**: A synced device (device ID, device name, last sync timestamp, schema version, platform)

- **SyncRelay**: A relay server configuration (URL, public key, is self-hosted flag, trust level)

---

## Architecture Considerations

### Jazz Integration Strategy

Since Jazz.tools is a TypeScript-first framework without native Rust bindings, the integration requires a hybrid approach:

#### Option A: Sidecar Daemon (Recommended)
- Node.js process runs alongside Rust CLI
- Rust CLI communicates with daemon via Unix socket or HTTP
- Daemon handles Jazz sync, Rust handles encryption
- Pros: Full Jazz features, native CRDT support
- Cons: Additional process, Node.js dependency

#### Option B: Protocol-Level Integration
- Implement Jazz sync protocol directly in Rust
- Use Jazz relay servers without Jazz SDK
- Pros: Single binary, no additional dependencies
- Cons: Must maintain protocol compatibility, more work

#### Option C: Embedded JavaScript Runtime
- Embed Deno or QuickJS for Jazz SDK
- Run Jazz in embedded runtime within Rust binary
- Pros: Single binary with full Jazz support
- Cons: Binary size increase, runtime overhead

### Data Flow

```
User Command → Caro CLI (Rust)
    ↓
Local Storage (SQLite) ← → Sync Manager
    ↓                           ↓
Encrypted Blob ← ← ← ← ← ← Encryption Layer
    ↓
Jazz Sync Layer (Node.js or Protocol)
    ↓
Relay Server (Jazz Cloud or Self-Hosted)
    ↓
Other Devices (Reverse Flow)
```

### Privacy Architecture

1. **At Rest**: All sync data encrypted with user-derived keys
2. **In Transit**: E2E encrypted, relay sees only blobs
3. **On Relay**: Encrypted storage, no access to content
4. **With Maintainers**: Zero access unless explicit opt-in for anonymized metrics

---

## Review & Acceptance Checklist

### Content Quality
- [x] No implementation details (languages, frameworks avoided in requirements)
- [x] Focused on user value and privacy guarantees
- [x] Written for understanding sync capabilities
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

### Privacy & Security Review
- [x] E2E encryption specified for all sync data
- [x] No plaintext transmission of sensitive data
- [x] User consent required for all sharing
- [x] Data deletion capability specified
- [x] Self-hosting option available

### Assumptions
- Users have access to Jazz-compatible sync relay (cloud or self-hosted)
- Users can securely store their recovery phrase
- Devices have internet access for sync (but work offline)
- SQLite is available for local storage

### Dependencies
- Depends on existing `config` module for preference storage format
- Depends on existing `logging` module for structured logs
- Integrates with `cli` module for sync subcommands
- Requires new persistent storage for command history (currently not stored)

---

## Execution Status

- [x] User description parsed
- [x] Key concepts extracted (privacy-first sync with Jazz)
- [x] Ambiguities resolved (architecture options documented)
- [x] User scenarios defined (3 primary stories + acceptance scenarios)
- [x] Requirements generated (25 functional + 16 non-functional)
- [x] Entities identified (7 key entities)
- [x] Review checklist passed

---

## References

### Local-First Principles
- [Local-first software: You own your data](https://www.inkandswitch.com/essay/local-first/) - Ink & Switch foundational paper
- [Why Local-First Software Is the Future](https://rxdb.info/articles/local-first-future.html) - RxDB practical guidance
- [Martin Kleppmann's Local-First Paper](https://martin.kleppmann.com/papers/local-first.pdf) - Academic foundations

### Jazz.tools Resources
- [Jazz.tools Documentation](https://jazz.tools/docs/react) - Official docs
- [Jazz Groups & Permissions](https://jazz.tools/docs/react/permissions-and-sharing/overview) - Permission model
- [Jazz Server Workers](https://jazz.tools/docs/svelte/project-setup/server-side) - Backend integration

### Reference Implementations
- [Atuin Shell History](https://atuin.sh/) - Similar sync approach for shell history
- [Atuin Encryption Scheme](https://blog.atuin.sh/new-encryption/) - E2E encryption design

---

**Ready for next phase**: `/clarify` (optional for architecture decision) -> `/plan` (implementation design)
