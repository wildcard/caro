# Research: Jazz Sync Integration

**Feature**: 005-jazz-sync-integration
**Date**: 2025-12-20
**Status**: Complete

---

## Executive Summary

This research evaluates Jazz.tools as a sync backend for Caro, comparing it with alternatives like Atuin's approach. Jazz provides a powerful local-first CRDT framework with E2E encryption, but requires a hybrid architecture since it's TypeScript-only.

---

## Jazz.tools Deep Dive

### What Is Jazz?

Jazz is a local-first database framework that:
- Syncs structured data as "Collaborative Values" (CoValues)
- Provides automatic conflict resolution via CRDTs
- Includes E2E encryption out of the box
- Supports real-time multiplayer collaboration
- Offers both cloud (Jazz Cloud) and self-hosted options

### Core Concepts

#### CoValues (Collaborative Values)
- **CoMap**: Key-value objects that sync
- **CoList**: Ordered arrays with CRDT merge
- **CoFeed**: Append-only logs (ideal for command history)
- **CoText**: Collaborative text editing
- **FileStream**: Binary file sync

#### Accounts & Groups
- **Account**: User identity with profile data
- **Group**: Permission scope for sharing CoValues
- Roles: reader, writer, admin
- Every CoValue has an owner (Account or Group)

#### Sync Architecture
- WebSocket-based sync protocol
- Global Mesh (Jazz Cloud) for storage and CDN
- SQLite-based local persistence
- Self-hostable sync servers

### SDK Availability

| Platform | SDK | Maturity |
|----------|-----|----------|
| React | jazz-react | Stable |
| React Native | jazz-react-native | Stable |
| Svelte | jazz-svelte | Stable |
| Vue | jazz-vue | Beta |
| Node.js | jazz-nodejs | Stable |
| **Rust** | **None** | N/A |

**Critical Finding**: No Rust SDK exists. Integration requires hybrid approach.

### Self-Hosting

Jazz sync server can be self-hosted:
```bash
npx jazz-run sync --port 4200 --db sync-db/storage.db
```

Options:
- `--host`: Bind address (default: 127.0.0.1)
- `--port`: Port (default: 4200)
- `--in-memory`: No persistence, sync only
- `--db`: SQLite path for persistence

### Security Model

- E2E encryption using WASM crypto
- Optional native crypto for Node.js (better performance)
- Keys managed per-account
- Groups define access control
- Server never sees plaintext

---

## Local-First Principles Analysis

### Ink & Switch Definition

From the foundational paper, local-first software provides:
1. **Fast** - No round-trip for operations
2. **Works offline** - Full functionality without network
3. **Multi-device** - Sync across devices
4. **Collaboration** - Real-time with others
5. **Longevity** - Data survives service shutdown
6. **Security** - E2E encryption
7. **User ownership** - Export, delete, control

### How Jazz Meets These Principles

| Principle | Jazz Support | Notes |
|-----------|--------------|-------|
| Fast | Yes | Local-first, instant UI updates |
| Offline | Yes | SQLite local storage |
| Multi-device | Yes | WebSocket sync |
| Collaboration | Yes | Real-time CRDT merge |
| Longevity | Partial | Self-host option helps |
| Security | Yes | E2E encryption |
| Ownership | Yes | Export via CoValue access |

### Gaps

- **Longevity**: Depends on self-hosting or backup strategy
- **Simple Recovery**: No built-in BIP39-style recovery phrases

---

## Atuin Reference Architecture

### How Atuin Works

Atuin is a shell history sync tool that:
1. Replaces shell history with SQLite database
2. Encrypts all data client-side
3. Syncs encrypted blobs to server
4. Uses symmetric encryption (256-bit keys)
5. Displays keys as BIP39 haiku for memorability

### Atuin Security Model

1. **Key Generation**: Random 256-bit key on first setup
2. **Key Display**: BIP39 words for human readability
3. **Encryption**: Per-entry encryption with random payload keys
4. **Storage**: Server stores only encrypted blobs
5. **Sync**: Client decrypts locally after download

### What We Can Learn

| Atuin Feature | Caro Application |
|---------------|------------------|
| BIP39 recovery phrase | Same - user-friendly key backup |
| Per-entry encryption | Same - granular encryption |
| SQLite local storage | Same - reliable local store |
| Self-host option | Same - via Jazz self-hosted server |
| History filtering | Same - exclude sensitive patterns |

### Atuin Limitations We Can Improve

1. **No CRDT**: Atuin uses timestamp-based merge. Jazz CRDTs are superior
2. **Shell-specific**: Atuin is shell history only. We sync more data
3. **Single-purpose**: We can build on Jazz's richer data model

---

## Integration Architecture Options

### Option A: Node.js Sidecar Daemon (Recommended)

```
[Caro CLI (Rust)] <--Unix Socket--> [Sync Daemon (Node.js/Jazz)]
       |                                      |
       v                                      v
[Local SQLite] <----------------------> [Jazz CoValues]
       |                                      |
       +------ Encryption (Rust) ------+      v
                                        [Jazz Relay]
```

**Implementation**:
- Rust CLI manages local SQLite for command history
- Node.js daemon handles Jazz sync protocol
- Communication via Unix socket (fast, local)
- Encryption/decryption in Rust for security
- Daemon auto-starts on first sync, runs as background service

**Pros**:
- Full Jazz feature set
- Native CRDT support
- Well-maintained SDK
- Easy to update Jazz version

**Cons**:
- Node.js runtime dependency (~50MB)
- Two processes to manage
- IPC overhead (minimal)

### Option B: Protocol-Level Rust Implementation

```
[Caro CLI (Rust)]
       |
       v
[Jazz Protocol (Rust reimplementation)]
       |
       v
[WebSocket to Jazz Relay]
```

**Implementation**:
- Reimplement Jazz sync protocol in Rust
- Use existing WebSocket crates (tungstenite)
- Implement CRDT merge logic in Rust
- Connect to standard Jazz relays

**Pros**:
- Single binary
- No runtime dependencies
- Full control

**Cons**:
- Significant development effort
- Must track Jazz protocol changes
- CRDT implementation complexity
- Testing burden

### Option C: Embedded JavaScript Runtime

```
[Caro CLI (Rust)]
       |
       v
[Deno/QuickJS Embedded]
       |
       v
[Jazz SDK (TypeScript)]
       |
       v
[Jazz Relay]
```

**Implementation**:
- Embed Deno Core or QuickJS in Rust binary
- Bundle Jazz SDK as JavaScript
- Call JavaScript from Rust for sync operations

**Pros**:
- Single binary
- Full Jazz SDK compatibility
- No separate process

**Cons**:
- Binary size increase (~20-30MB)
- Runtime overhead
- Debugging complexity
- Dependency on embedded JS runtime

### Recommendation: Option A (Sidecar Daemon)

For Caro's use case, the sidecar daemon approach offers:
1. **Best compatibility**: Full Jazz SDK with all features
2. **Simplest maintenance**: Update npm packages, not protocol code
3. **Clean separation**: Rust handles CLI + encryption, Node handles sync
4. **Proven pattern**: Similar to how VSCode extensions work

The Node.js dependency is acceptable because:
- Many developers already have Node installed
- We can bundle a minimal Node runtime if needed
- The daemon only runs when sync is enabled

---

## Data Model for Jazz Integration

### CoValue Schema

```typescript
// User's sync root
interface CaroSyncRoot extends CoMap {
  profile: CaroProfile;
  commandHistory: CoFeed<CommandMemory>;
  preferences: SyncPreferences;
  devices: CoList<DeviceRecord>;
  metrics: UsageMetrics; // Only if opted in
}

// Command history entry (in CoFeed for append-only semantics)
interface CommandMemory extends CoMap {
  id: string;           // UUID
  prompt: string;       // User's natural language input
  command: string;      // Generated shell command
  executed: boolean;    // Was it run?
  succeeded: boolean;   // Did it work?
  timestamp: number;    // Unix timestamp
  deviceId: string;     // Origin device
  safetyLevel: string;  // Risk assessment
  tags: CoList<string>; // User tags
}

// User preferences that sync
interface SyncPreferences extends CoMap {
  safetyLevel: string;
  defaultShell: string;
  defaultModel: string;
  excludePatterns: CoList<string>;
  syncFrequency: string;
  retentionDays: number;
}

// Device tracking
interface DeviceRecord extends CoMap {
  id: string;
  name: string;
  platform: string;
  lastSync: number;
  schemaVersion: string;
}
```

### Encryption Layer

Jazz provides E2E encryption, but we add an additional layer:

1. **Jazz Encryption**: Protects data in transit and at rest on relay
2. **Caro Encryption**: Additional encryption for command content
   - Prompt and command fields encrypted with user key
   - Metadata (timestamps, device) visible to Jazz for CRDT merge
   - Double encryption ensures even Jazz infrastructure can't read commands

### Sync Granularity

| Data Type | Sync Frequency | Conflict Resolution |
|-----------|----------------|---------------------|
| Commands | Real-time | Append-only (CoFeed) |
| Preferences | On change | Last-write-wins |
| Devices | On connection | Merge by device ID |
| Metrics | Daily | Sum/merge counters |

---

## Privacy Analysis

### Data Flow Security

```
User Command
    |
    v
[Encrypt command content with user key] -- Key never leaves device
    |
    v
[Wrap in Jazz CoValue] -- Jazz encrypts for sync
    |
    v
[Send to Relay] -- Double encrypted
    |
    v
[Store on Relay] -- Opaque blobs only
```

### Threat Model

| Threat | Mitigation |
|--------|------------|
| Relay compromise | E2E encryption, no plaintext |
| Network interception | TLS + E2E encryption |
| Device theft | Local encryption + key derivation |
| Key compromise | Key rotation capability |
| Maintainer access | Zero access by default, opt-in only |

### Metadata Exposure

What the relay CAN see:
- Device IDs (pseudonymous)
- Timestamps
- Blob sizes
- Sync frequency

What the relay CANNOT see:
- Command content
- User prompts
- File paths
- Environment variables

### GDPR/Privacy Compliance

- **Right to access**: Export command via CLI
- **Right to deletion**: Delete from all devices
- **Data portability**: Standard JSON export
- **Consent**: Explicit opt-in for any sync

---

## Implementation Roadmap

### Phase 1: Local Foundation (No Sync)
- Add command history storage (SQLite)
- Implement history querying
- Add preference persistence
- No network features

### Phase 2: Encryption Layer
- Implement key derivation from recovery phrase
- Add command encryption/decryption
- Secure local key storage
- BIP39 phrase generation

### Phase 3: Sync Daemon
- Create Node.js sync service
- Implement Unix socket IPC
- Add Jazz SDK integration
- Basic sync protocol

### Phase 4: Full Sync
- Multi-device sync
- Conflict resolution
- Device management
- Status reporting

### Phase 5: Privacy Features
- Optional anonymized metrics
- Pattern-based exclusions
- Full data export
- Key rotation

---

## Open Questions for Clarification

1. **Self-Hosting Priority**: Should self-hosted relay be MVP or later phase?
   - Recommendation: MVP includes Jazz Cloud, self-host in Phase 5

2. **Command History Depth**: How many commands to store/sync?
   - Recommendation: 10,000 by default, configurable

3. **Sync Granularity**: Real-time vs batch sync?
   - Recommendation: Real-time with debouncing (1-second delay)

4. **Recovery Phrase Length**: 12 or 24 words?
   - Recommendation: 24 words (256 bits) for maximum security

5. **Desktop App**: Future desktop GUI for sync management?
   - Out of scope for this spec, but architecture should support it

---

## Conclusion

Jazz.tools is a strong choice for Caro sync integration because:
1. E2E encryption is built-in
2. CRDT conflict resolution is automatic
3. Self-hosting is supported
4. Local-first architecture aligns with Caro's privacy values

The recommended approach is a Node.js sidecar daemon that handles Jazz sync while the Rust CLI manages local operations and additional encryption. This provides the best balance of compatibility, maintainability, and security.
