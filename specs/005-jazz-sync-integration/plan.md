# Implementation Plan: Jazz Sync Integration

**Branch**: `005-jazz-sync-integration` | **Date**: 2025-12-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-jazz-sync-integration/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   -> Found: 25 functional requirements, 16 non-functional requirements
2. Fill Technical Context
   -> Project Type: single (Rust CLI with Node.js sidecar)
   -> Structure Decision: Hybrid architecture
3. Fill Constitution Check section
   -> Library-First: Sync module as standalone library
   -> TDD: Contract tests before implementation
   -> Safety-First: E2E encryption, no plaintext transmission
4. Evaluate Constitution Check
   -> Complexity justified: Node.js sidecar for Jazz SDK compatibility
5. Execute Phase 0 -> research.md (COMPLETED - already exists)
6. Execute Phase 1 -> contracts, data-model.md (COMPLETED), quickstart.md
7. Re-evaluate Constitution Check
   -> All principles satisfied with documented trade-offs
8. Plan Phase 2 -> Task generation approach described
9. STOP - Ready for /tasks command
```

## Summary

Jazz Sync Integration adds local-first, E2E encrypted synchronization of command history, preferences, and usage patterns across multiple devices using Jazz.tools CRDT infrastructure. The architecture uses a **Node.js sidecar daemon** for Jazz SDK integration while keeping all encryption logic in Rust for security. Users control their data via BIP39 recovery phrases, with zero-knowledge sync through relay servers.

## Technical Context

**Language/Version**: Rust 1.75+ (CLI core), Node.js 20+ (sync daemon), TypeScript 5.x (Jazz SDK)
**Primary Dependencies**:
- Rust: `tokio`, `rusqlite`, `argon2`, `aes-gcm`, `bip39`, `serde_json`
- Node.js: `jazz-tools`, `ws`, `better-sqlite3`
**Storage**: SQLite (local command history), Jazz CoValues (sync layer)
**Testing**: `cargo test` (Rust), `vitest` (Node.js), integration tests via Unix socket
**Target Platform**: Linux (x64, ARM64), macOS (Intel, Apple Silicon), Windows (x64)
**Project Type**: Single (hybrid Rust + Node.js architecture)
**Performance Goals**:
- Sync startup: < 500ms
- Incremental sync: < 5s for typical changes
- Initial sync (10K commands): < 60s
- CLI startup impact: 0ms (daemon runs independently)
**Constraints**:
- Zero plaintext transmission
- Works fully offline
- < 100MB additional disk for sync daemon
- Node.js 20+ runtime required for sync features
**Scale/Scope**:
- 10,000 commands default history
- Unlimited devices per identity
- 90-day default retention (configurable)

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Simplicity
- [x] **PASS**: Minimal abstraction - direct SQLite for local storage, Jazz SDK for sync
- [x] **PASS**: Single data flow: `Command -> SQLite -> Encrypt -> Jazz CoValue -> Relay`
- [x] **JUSTIFIED**: Node.js sidecar adds complexity but avoids reimplementing Jazz protocol

### II. Library-First Architecture
- [x] **PASS**: New `cmdai::sync` module as standalone library
- [x] **PASS**: Clear public API: `SyncManager`, `EncryptionService`, `HistoryStore`
- [x] **PASS**: Daemon is separate process, CLI orchestrates via IPC

### III. Test-First (NON-NEGOTIABLE)
- [x] **PASS**: Contract tests for IPC protocol before daemon implementation
- [x] **PASS**: Encryption tests with known vectors before integration
- [x] **PASS**: Integration tests with mock relay before Jazz Cloud

### IV. Safety-First Development
- [x] **PASS**: E2E encryption mandatory for all sync data
- [x] **PASS**: Recovery phrase validation with checksum
- [x] **PASS**: No unsafe Rust except potentially for crypto primitives (justified)
- [x] **PASS**: Redaction of sync keys from logs

### V. Observability & Versioning
- [x] **PASS**: Structured logging for sync operations via `tracing`
- [x] **PASS**: Schema versioning in CoValues for migration
- [x] **PASS**: Sync status queryable via CLI

## Project Structure

### Documentation (this feature)
```
specs/005-jazz-sync-integration/
├── spec.md              # Feature specification (complete)
├── research.md          # Phase 0 research (complete)
├── data-model.md        # Phase 1 data model (complete)
├── plan.md              # This file
├── quickstart.md        # Phase 1 output (this plan)
├── contracts/           # Phase 1 output (this plan)
│   ├── ipc-protocol.md  # Rust <-> Node.js IPC contract
│   ├── sync-api.md      # Sync daemon API contract
│   └── encryption.md    # Encryption format contract
└── tasks.md             # Phase 2 output (/tasks command)
```

### Source Code (repository root)
```
src/
├── sync/                    # NEW: Sync module (library-first)
│   ├── mod.rs              # Public API exports
│   ├── manager.rs          # SyncManager orchestration
│   ├── identity.rs         # Recovery phrase, key derivation
│   ├── encryption.rs       # AES-256-GCM encryption
│   ├── history.rs          # Command history storage
│   ├── preferences.rs      # Sync preferences
│   ├── ipc.rs              # Unix socket IPC client
│   └── daemon.rs           # Daemon process management
├── cli/
│   └── sync_commands.rs    # NEW: caro sync subcommands
└── lib.rs                  # Updated: export sync module

sync-daemon/                 # NEW: Node.js sync daemon
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts            # Daemon entry point
│   ├── jazz-sync.ts        # Jazz SDK integration
│   ├── ipc-server.ts       # Unix socket server
│   ├── covalue-schemas.ts  # Jazz CoValue definitions
│   └── types.ts            # Shared TypeScript types
└── tests/
    └── integration/

tests/
├── contract/
│   └── sync_ipc_test.rs    # IPC protocol contract tests
├── integration/
│   ├── sync_flow_test.rs   # End-to-end sync tests
│   └── encryption_test.rs  # Encryption verification
└── unit/
    ├── identity_test.rs    # Key derivation tests
    └── history_test.rs     # Local storage tests
```

**Structure Decision**: Hybrid single-project with embedded Node.js daemon directory. The Rust CLI remains the primary binary; the sync-daemon is an optional companion process that only runs when sync is enabled.

## Phase 0: Outline & Research

**Status**: COMPLETE (see [research.md](./research.md))

### Key Research Decisions

| Topic | Decision | Rationale |
|-------|----------|-----------|
| Integration Architecture | Node.js Sidecar Daemon | Full Jazz SDK compatibility, lower maintenance than protocol reimplementation |
| Local Storage | SQLite via `rusqlite` | Consistent with existing cache architecture, proven reliability |
| Key Derivation | Argon2id | Memory-hard, resistant to GPU attacks, recommended by OWASP |
| Encryption | AES-256-GCM | AEAD encryption, authenticated, widely supported |
| Recovery Phrase | BIP39 (24 words) | 256-bit entropy, human-readable, well-understood standard |
| IPC Protocol | Unix Domain Socket + JSON-RPC | Fast local communication, structured messages, cross-platform |
| Relay Strategy | Jazz Cloud MVP, self-host later | Faster development, reduces initial complexity |

## Phase 1: Design & Contracts

### 1. Data Model

**Status**: COMPLETE (see [data-model.md](./data-model.md))

Key entities:
- `SyncIdentity`: Recovery phrase, derived keys, device registry
- `CommandMemory`: Encrypted command history entries
- `SyncPreferences`: User settings that sync across devices
- `DeviceRecord`: Device metadata for multi-device management

### 2. API Contracts

#### IPC Protocol Contract (`contracts/ipc-protocol.md`)

Communication between Rust CLI and Node.js daemon via Unix socket:

```
Socket Path: ~/.config/cmdai/sync.sock (Unix) or \\.\pipe\cmdai-sync (Windows)
Protocol: JSON-RPC 2.0 over newline-delimited JSON
```

**Methods**:

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `sync.status` | `{}` | `{connected: bool, lastSync: timestamp, pending: number}` | Get sync status |
| `sync.push` | `{entries: EncryptedEntry[]}` | `{synced: number, errors: string[]}` | Push encrypted entries |
| `sync.pull` | `{since: timestamp}` | `{entries: EncryptedEntry[], hasMore: bool}` | Pull new entries |
| `sync.init` | `{accountId: string, authKey: string}` | `{success: bool}` | Initialize Jazz account |
| `device.register` | `{name: string, platform: string}` | `{deviceId: string}` | Register this device |
| `device.list` | `{}` | `{devices: DeviceRecord[]}` | List all devices |

#### Encryption Contract (`contracts/encryption.md`)

```
Encrypted Entry Format:
┌─────────────┬─────────────┬──────────────────────────┐
│ Version (1B)│ Nonce (12B) │ Ciphertext + Tag (var)   │
└─────────────┴─────────────┴──────────────────────────┘

Version: 0x01 (AES-256-GCM)
Nonce: Random 96-bit IV
Ciphertext: AES-256-GCM encrypted payload
Tag: 128-bit authentication tag (included in ciphertext)

Key Derivation:
  recovery_phrase (24 BIP39 words)
       │
       ▼
  Argon2id(phrase, salt=SHA256(phrase), m=64MB, t=3, p=4)
       │
       ▼
  master_key (256-bit)
       │
       ├──► HKDF-SHA256(master_key, "command-encryption") → command_key
       ├──► HKDF-SHA256(master_key, "preference-encryption") → preference_key
       └──► HKDF-SHA256(master_key, "jazz-auth") → jazz_auth_key
```

#### Sync Daemon API Contract (`contracts/sync-api.md`)

Internal daemon architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    Sync Daemon (Node.js)                 │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌────────────┐ │
│  │ IPC Server  │◄──►│ Sync Manager │◄──►│ Jazz SDK   │ │
│  │ (Unix Sock) │    │              │    │ (CoValues) │ │
│  └─────────────┘    └──────────────┘    └────────────┘ │
│         ▲                  │                   │        │
│         │                  ▼                   ▼        │
│  ┌──────┴──────┐    ┌──────────────┐    ┌────────────┐ │
│  │ Rust CLI    │    │ Local Cache  │    │ Jazz Relay │ │
│  │ (External)  │    │ (SQLite)     │    │ (Remote)   │ │
│  └─────────────┘    └──────────────┘    └────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 3. Contract Tests (to be generated)

```rust
// tests/contract/sync_ipc_test.rs
#[tokio::test]
async fn test_sync_status_returns_valid_response() {
    // Given: daemon is running
    // When: send sync.status request
    // Then: receive valid SyncStatus response
}

#[tokio::test]
async fn test_push_encrypted_entries_succeeds() {
    // Given: valid encrypted entries
    // When: send sync.push request
    // Then: entries are acknowledged
}

#[tokio::test]
async fn test_invalid_encryption_rejected() {
    // Given: malformed encrypted entry
    // When: send sync.push request
    // Then: receive error response
}
```

### 4. Quickstart (see `quickstart.md`)

Step-by-step guide for:
1. Installing sync daemon
2. Initializing sync identity
3. Adding a second device
4. Verifying sync works

### 5. Agent Context Update

Add to `CLAUDE.md`:
```markdown
## Sync Module
- `src/sync/`: Local-first sync with Jazz.tools
- `sync-daemon/`: Node.js companion for Jazz SDK
- IPC: Unix socket at `~/.config/cmdai/sync.sock`
- Encryption: AES-256-GCM with Argon2id key derivation
```

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:

1. **Foundation Tasks** (Models & Storage):
   - Task 001: Create `src/sync/mod.rs` with module structure
   - Task 002: Implement `SyncIdentity` with BIP39 generation
   - Task 003: Implement `EncryptionService` with AES-256-GCM
   - Task 004: Implement `HistoryStore` with SQLite

2. **IPC Tasks** (Communication Layer):
   - Task 005: Define IPC protocol types
   - Task 006: Implement Unix socket client in Rust
   - Task 007: Create sync-daemon package.json and structure
   - Task 008: Implement IPC server in Node.js

3. **Jazz Integration Tasks**:
   - Task 009: Define Jazz CoValue schemas
   - Task 010: Implement Jazz sync manager
   - Task 011: Connect IPC to Jazz operations

4. **CLI Tasks**:
   - Task 012: Add `caro sync` subcommand group
   - Task 013: Implement `caro sync init`
   - Task 014: Implement `caro sync status`
   - Task 015: Implement `caro sync restore`

5. **Integration Tasks**:
   - Task 016: Hook command execution to history storage
   - Task 017: Implement automatic sync on command completion
   - Task 018: Add daemon lifecycle management

6. **Test Tasks** (TDD - tests first):
   - Task T01: Write encryption contract tests [BEFORE Task 003]
   - Task T02: Write IPC protocol tests [BEFORE Task 006]
   - Task T03: Write integration tests [BEFORE Task 016]

**Ordering Strategy**:
- TDD order: Test tasks (T0x) MUST precede corresponding implementation
- Dependency order: Identity -> Encryption -> Storage -> IPC -> Jazz -> CLI
- Parallelizable: Tasks 002-004 can run in parallel after Task 001
- Parallelizable: Tasks 007-008 can run in parallel with Rust IPC

**Estimated Output**: 20-25 numbered tasks in tasks.md

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)
**Phase 4**: Implementation following TDD cycle
**Phase 5**: Validation
- Run full test suite
- Execute quickstart.md end-to-end
- Performance benchmarks (sync latency, startup impact)
- Security review of encryption implementation

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Node.js sidecar | Jazz SDK is TypeScript-only | Protocol reimplementation would take 3x longer and require ongoing maintenance |
| Two-process architecture | Separation of concerns: Rust for crypto, Node for Jazz | Embedded JS runtime adds binary bloat and debugging complexity |
| BIP39 + Argon2id | Security best practices | Simpler key derivation doesn't meet security requirements |

## Progress Tracking

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented

---
*Based on Constitution v1.0.0 - See `.specify/memory/constitution.md`*
