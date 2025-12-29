# Data Model: Jazz Sync Integration

**Feature**: 005-jazz-sync-integration
**Date**: 2025-12-20
**Status**: Draft

---

## Overview

This document defines the data model for Caro's Jazz sync integration, covering local storage, sync schemas, and the encryption layer.

---

## Local Storage Schema (SQLite)

### Command History Table

```sql
CREATE TABLE command_history (
    id TEXT PRIMARY KEY,           -- UUID v4
    prompt TEXT NOT NULL,          -- User's natural language input (encrypted)
    command TEXT NOT NULL,         -- Generated shell command (encrypted)
    explanation TEXT,              -- Command explanation (encrypted)

    -- Execution metadata
    executed BOOLEAN DEFAULT FALSE,
    success BOOLEAN,
    exit_code INTEGER,
    execution_time_ms INTEGER,

    -- Context
    working_dir TEXT,              -- Where command was run
    shell_type TEXT,               -- bash, zsh, fish, etc.
    platform TEXT,                 -- linux, macos, windows

    -- Safety
    safety_level TEXT NOT NULL,    -- safe, moderate, high, critical
    safety_warnings TEXT,          -- JSON array of warnings

    -- Sync metadata
    device_id TEXT NOT NULL,       -- Origin device
    created_at INTEGER NOT NULL,   -- Unix timestamp (ms)
    updated_at INTEGER NOT NULL,   -- Unix timestamp (ms)
    synced_at INTEGER,             -- Last sync timestamp
    deleted_at INTEGER,            -- Soft delete for sync

    -- Versioning
    schema_version INTEGER DEFAULT 1
);

CREATE INDEX idx_command_history_created ON command_history(created_at DESC);
CREATE INDEX idx_command_history_synced ON command_history(synced_at);
CREATE INDEX idx_command_history_deleted ON command_history(deleted_at);
```

### Sync Identity Table

```sql
CREATE TABLE sync_identity (
    id TEXT PRIMARY KEY,           -- Single row: 'primary'

    -- Identity
    account_id TEXT UNIQUE,        -- Jazz account ID
    device_id TEXT NOT NULL,       -- This device's ID
    device_name TEXT,              -- User-friendly device name

    -- Encryption (derived from recovery phrase)
    key_id TEXT NOT NULL,          -- Key identifier
    salt BLOB NOT NULL,            -- Argon2 salt
    encrypted_key BLOB NOT NULL,   -- Encrypted master key (for local storage)

    -- State
    created_at INTEGER NOT NULL,
    last_sync_at INTEGER,
    sync_enabled BOOLEAN DEFAULT FALSE,

    -- Recovery
    recovery_phrase_hash TEXT      -- For verification only, NOT the phrase
);
```

### Device Registry Table

```sql
CREATE TABLE devices (
    device_id TEXT PRIMARY KEY,
    device_name TEXT NOT NULL,
    platform TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    first_seen_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    is_current BOOLEAN DEFAULT FALSE
);
```

### Sync Queue Table

```sql
CREATE TABLE sync_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation TEXT NOT NULL,       -- 'create', 'update', 'delete'
    entity_type TEXT NOT NULL,     -- 'command', 'preference', 'device'
    entity_id TEXT NOT NULL,
    payload BLOB,                  -- Encrypted payload
    created_at INTEGER NOT NULL,
    attempts INTEGER DEFAULT 0,
    last_attempt_at INTEGER,
    error TEXT
);

CREATE INDEX idx_sync_queue_pending ON sync_queue(attempts, created_at);
```

### Preferences Table

```sql
CREATE TABLE sync_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,           -- JSON value
    updated_at INTEGER NOT NULL,
    synced_at INTEGER
);
```

---

## Jazz CoValue Schema

### TypeScript Definitions

```typescript
import { co, CoMap, CoList, CoFeed, Account, Group } from "jazz-tools";

/**
 * Caro user profile - public information
 */
export class CaroProfile extends CoMap {
  displayName = co.string;
  deviceCount = co.number;
  createdAt = co.number;
}

/**
 * Root sync object for a Caro user
 * Owned by user's account, synced across all their devices
 */
export class CaroSyncRoot extends CoMap {
  // User profile
  profile = co.ref(CaroProfile);

  // Command history (append-only feed for CRDT-friendly sync)
  commandHistory = co.ref(CommandHistoryFeed);

  // User preferences
  preferences = co.ref(SyncPreferences);

  // Known devices
  devices = co.ref(DeviceList);

  // Optional: anonymized usage metrics (only if opted in)
  metrics = co.optional.ref(UsageMetrics);

  // Schema version for migrations
  schemaVersion = co.number;
}

/**
 * Command history as append-only feed
 * Using CoFeed for efficient append-only semantics
 */
export class CommandHistoryFeed extends CoFeed<CommandEntry> {}

/**
 * Individual command entry
 * Sensitive fields are additionally encrypted client-side
 */
export class CommandEntry extends CoMap {
  // Identifiers
  id = co.string;
  deviceId = co.string;

  // Encrypted content (Caro-layer encryption, not just Jazz)
  encryptedPrompt = co.string;     // Base64 of encrypted prompt
  encryptedCommand = co.string;    // Base64 of encrypted command
  encryptedExplanation = co.optional.string;

  // Metadata (visible for CRDT merge, not sensitive)
  executed = co.boolean;
  success = co.optional.boolean;
  safetyLevel = co.string;

  // Timestamps
  createdAt = co.number;
  executedAt = co.optional.number;

  // Tags (user-defined, encrypted)
  encryptedTags = co.optional.string; // Base64 of encrypted JSON array

  // Soft delete marker
  deletedAt = co.optional.number;
}

/**
 * User preferences that sync
 */
export class SyncPreferences extends CoMap {
  // Caro settings
  safetyLevel = co.string;         // 'strict' | 'moderate' | 'permissive'
  defaultShell = co.optional.string;
  defaultModel = co.optional.string;

  // Sync settings
  syncEnabled = co.boolean;
  syncFrequency = co.string;       // 'realtime' | 'hourly' | 'daily'
  retentionDays = co.number;
  maxCommands = co.number;

  // Privacy settings
  excludePatterns = co.ref(ExcludePatternList);
  shareMetrics = co.boolean;       // Opt-in for anonymous metrics

  // Last update
  updatedAt = co.number;
}

/**
 * List of exclusion patterns for sync
 */
export class ExcludePatternList extends CoList<string> {}

/**
 * Device registry
 */
export class DeviceList extends CoList<DeviceRecord> {}

/**
 * Device information
 */
export class DeviceRecord extends CoMap {
  deviceId = co.string;
  deviceName = co.string;
  platform = co.string;            // 'linux' | 'macos' | 'windows'
  schemaVersion = co.number;
  firstSeen = co.number;
  lastSeen = co.number;
  isActive = co.boolean;
}

/**
 * Optional anonymized usage metrics
 * Only populated if user opts in
 */
export class UsageMetrics extends CoMap {
  // Aggregate counts (no individual commands)
  totalCommands = co.number;
  executedCommands = co.number;
  successfulCommands = co.number;

  // Category distribution (counts only)
  categoryDistribution = co.ref(CategoryCounts);

  // Safety level distribution
  safetyDistribution = co.ref(SafetyCounts);

  // Time-based patterns (bucketed, not exact)
  hourlyDistribution = co.ref(HourlyCounts);

  // Metadata
  periodStart = co.number;
  periodEnd = co.number;
  lastUpdated = co.number;
}

export class CategoryCounts extends CoMap {
  [category: string]: co.number;
}

export class SafetyCounts extends CoMap {
  safe = co.number;
  moderate = co.number;
  high = co.number;
  critical = co.number;
}

export class HourlyCounts extends CoList<number> {} // 24 hourly buckets
```

---

## Encryption Schema

### Key Derivation

```
Recovery Phrase (24 words, BIP39)
        |
        v
    [PBKDF2 or Argon2id]
        |
        +---> Master Key (256-bit)
        |           |
        |           +---> Command Encryption Key
        |           +---> Preference Encryption Key
        |           +---> Metrics Encryption Key
        |
        +---> Authentication Key (for Jazz account)
```

### Encryption Format

```
Encrypted Payload Structure:
+----------------+------------------+----------------------+
| Version (1B)   | Nonce (12B)      | Ciphertext (variable)|
+----------------+------------------+----------------------+
        |               |                    |
        v               v                    v
    Schema ID    Random nonce       AES-256-GCM output
                                    (includes auth tag)
```

### Per-Field Encryption

| Field | Encrypted | Algorithm | Notes |
|-------|-----------|-----------|-------|
| prompt | Yes | AES-256-GCM | User's natural language |
| command | Yes | AES-256-GCM | Generated shell command |
| explanation | Yes | AES-256-GCM | Optional field |
| tags | Yes | AES-256-GCM | JSON array |
| safetyLevel | No | - | Needed for CRDT merge |
| timestamps | No | - | Needed for ordering |
| deviceId | No | - | Pseudonymous identifier |

---

## Sync Protocol

### Initialization Flow

```
1. User runs `caro sync init`
        |
        v
2. Generate recovery phrase (24 BIP39 words)
        |
        v
3. Derive master key from phrase (Argon2id)
        |
        v
4. Create Jazz account with derived auth key
        |
        v
5. Create CaroSyncRoot CoValue
        |
        v
6. Store encrypted master key locally
        |
        v
7. Display recovery phrase to user (ONCE)
```

### Device Addition Flow

```
1. User runs `caro sync join` on new device
        |
        v
2. User enters recovery phrase
        |
        v
3. Derive master key (same as original device)
        |
        v
4. Authenticate with Jazz using derived key
        |
        v
5. Fetch existing CaroSyncRoot
        |
        v
6. Add device to DeviceList
        |
        v
7. Sync existing command history
```

### Real-Time Sync Flow

```
[Device A: New Command]
        |
        v
1. Store in local SQLite
        |
        v
2. Encrypt command content with master key
        |
        v
3. Create CommandEntry CoValue
        |
        v
4. Append to CommandHistoryFeed
        |
        v
5. Jazz syncs to relay (encrypted)
        |
        v
[Relay: Store encrypted blob]
        |
        v
[Device B: Receive update]
        |
        v
6. Jazz delivers new CommandEntry
        |
        v
7. Decrypt command content with master key
        |
        v
8. Store in local SQLite
```

---

## Migration Strategy

### Schema Versioning

Each CoValue includes a `schemaVersion` field:
- Version 1: Initial schema
- Version 2+: Future migrations

### Migration Rules

1. **Additive changes**: New optional fields are safe
2. **Structural changes**: Require migration script
3. **Breaking changes**: Require new sync identity

### Backward Compatibility

- Older devices skip unknown fields
- Newer devices provide defaults for missing fields
- Schema version mismatch triggers user notification

---

## Data Lifecycle

### Retention

| Data Type | Default Retention | Configurable |
|-----------|-------------------|--------------|
| Commands | 90 days | Yes (30-365 days) |
| Preferences | Forever | No |
| Devices | Until removed | No |
| Metrics | 30 days | No |

### Deletion

1. **Soft delete**: Set `deletedAt` timestamp
2. **Sync deletion**: Propagates to all devices
3. **Hard delete**: After retention period expires
4. **Full wipe**: `caro sync reset` removes all data

### Export Format

```json
{
  "exportVersion": "1.0",
  "exportedAt": "2025-12-20T10:00:00Z",
  "profile": {
    "displayName": "User's Caro",
    "deviceCount": 3
  },
  "commands": [
    {
      "id": "uuid-here",
      "prompt": "list all files in current directory",
      "command": "ls -la",
      "executed": true,
      "success": true,
      "createdAt": "2025-12-19T15:30:00Z"
    }
  ],
  "preferences": {
    "safetyLevel": "moderate",
    "defaultShell": "zsh"
  },
  "devices": [
    {
      "name": "MacBook Pro",
      "platform": "macos",
      "lastSeen": "2025-12-20T09:00:00Z"
    }
  ]
}
```

---

## Privacy Guarantees

### What IS Encrypted

- Command prompts (user's natural language)
- Generated shell commands
- Command explanations
- User-defined tags
- Exclusion patterns

### What is NOT Encrypted (but pseudonymous)

- Device identifiers (random UUIDs)
- Timestamps
- Safety levels
- Execution success/failure
- Schema versions

### Zero-Knowledge Properties

1. **Relay cannot read commands**: Double encrypted (Caro + Jazz)
2. **Relay cannot identify user**: No PII, only pseudonymous device IDs
3. **Maintainers cannot access**: No backdoors, no master keys
4. **Pattern analysis limited**: Timestamps visible but content hidden

---

## Performance Considerations

### Local Operations

| Operation | Target Latency | Notes |
|-----------|----------------|-------|
| Store command | < 10ms | SQLite insert |
| Query history | < 50ms | Indexed query |
| Encrypt field | < 1ms | Per-field |
| Decrypt field | < 1ms | Per-field |

### Sync Operations

| Operation | Target Latency | Notes |
|-----------|----------------|-------|
| Initial sync (10K commands) | < 60s | Batched |
| Incremental sync | < 5s | Real-time |
| Device addition | < 30s | Full history |

### Storage Limits

| Resource | Default Limit | Configurable |
|----------|---------------|--------------|
| Local history | 10,000 commands | Yes |
| Sync history | 10,000 commands | Yes |
| Command size | 10 KB | No |
| Total local DB | 100 MB | No |
