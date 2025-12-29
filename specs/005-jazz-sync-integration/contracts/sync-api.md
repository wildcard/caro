# Sync Daemon API Contract

**Version**: 1.0.0
**Component**: Node.js Sync Daemon
**Purpose**: Jazz SDK integration and sync orchestration

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      Sync Daemon (Node.js)                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐      ┌────────────────┐      ┌─────────────┐ │
│  │  IPC Server  │◄────►│  Sync Manager  │◄────►│  Jazz SDK   │ │
│  │ (Unix Socket)│      │                │      │  (jazz-tools)│ │
│  └──────────────┘      └────────────────┘      └─────────────┘ │
│         ▲                     │                       │         │
│         │                     ▼                       ▼         │
│  ┌──────┴───────┐      ┌──────────────┐      ┌─────────────┐   │
│  │   Rust CLI   │      │ Local Cache  │      │ Jazz Relay  │   │
│  │  (External)  │      │  (SQLite)    │      │  (Remote)   │   │
│  └──────────────┘      └──────────────┘      └─────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Responsibilities

### IPC Server
- Listen on Unix socket for Rust CLI connections
- Parse JSON-RPC 2.0 messages
- Route requests to Sync Manager
- Handle connection lifecycle

### Sync Manager
- Coordinate between IPC and Jazz SDK
- Manage sync state machine
- Handle retry logic and error recovery
- Maintain local cache for offline resilience

### Jazz SDK Integration
- Initialize Jazz account and worker
- Create and manage CoValues
- Subscribe to real-time updates
- Push/pull encrypted entries

---

## State Machine

```
                    ┌───────────────┐
                    │ UNINITIALIZED │
                    └───────┬───────┘
                            │ sync.init()
                            ▼
                    ┌───────────────┐
          ┌────────│  INITIALIZING │────────┐
          │        └───────────────┘        │
          │ success                    fail │
          ▼                                 ▼
   ┌─────────────┐                  ┌───────────────┐
   │   ONLINE    │◄────────────────►│    OFFLINE    │
   └─────────────┘  network change  └───────────────┘
          │                                 │
          │ sync.reset()                    │
          ▼                                 ▼
   ┌─────────────────────────────────────────────┐
   │              UNINITIALIZED                   │
   └─────────────────────────────────────────────┘
```

### States

| State | Description | Allowed Operations |
|-------|-------------|-------------------|
| UNINITIALIZED | No sync identity | `sync.init`, `daemon.ping` |
| INITIALIZING | Connecting to Jazz | `daemon.ping` |
| ONLINE | Connected and syncing | All operations |
| OFFLINE | Cached mode, queuing changes | All except push (queued) |

---

## Internal APIs

### SyncManager Class

```typescript
class SyncManager {
  // Lifecycle
  async initialize(config: InitConfig): Promise<void>;
  async shutdown(): Promise<void>;

  // Sync operations
  async pushEntries(entries: EncryptedEntry[]): Promise<PushResult>;
  async pullEntries(since: number): Promise<PullResult>;

  // State
  getStatus(): SyncStatus;
  isOnline(): boolean;

  // Events
  on(event: 'stateChange', handler: (state: SyncState) => void): void;
  on(event: 'syncComplete', handler: (result: SyncResult) => void): void;
  on(event: 'error', handler: (error: SyncError) => void): void;
}
```

### JazzSyncService Class

```typescript
class JazzSyncService {
  // Account management
  async createAccount(authKey: Buffer): Promise<Account>;
  async loginAccount(accountId: string, authKey: Buffer): Promise<Account>;

  // CoValue operations
  async getSyncRoot(): Promise<CaroSyncRoot>;
  async appendCommand(entry: EncryptedCommandEntry): Promise<void>;
  async updatePreference(key: string, encrypted: string): Promise<void>;

  // Device management
  async registerDevice(info: DeviceInfo): Promise<DeviceRecord>;
  async listDevices(): Promise<DeviceRecord[]>;

  // Subscriptions
  subscribeToUpdates(callback: (update: SyncUpdate) => void): Unsubscribe;
}
```

---

## Jazz CoValue Schema

### CaroSyncRoot

```typescript
import { co, CoMap, CoList, CoFeed, Account } from "jazz-tools";

export class CaroSyncRoot extends CoMap {
  // Version for schema migrations
  schemaVersion = co.number;

  // Command history (append-only for CRDT-friendly sync)
  commandHistory = co.ref(CommandHistoryFeed);

  // User preferences
  preferences = co.ref(SyncPreferences);

  // Device registry
  devices = co.ref(DeviceList);

  // Metadata
  createdAt = co.number;
  updatedAt = co.number;
}
```

### CommandHistoryFeed

```typescript
export class CommandHistoryFeed extends CoFeed.Of(EncryptedCommandEntry) {}

export class EncryptedCommandEntry extends CoMap {
  // Identifier
  id = co.string;              // UUID v4
  deviceId = co.string;        // Origin device

  // Encrypted content (base64)
  encryptedPayload = co.string;

  // Unencrypted metadata (needed for CRDT merge)
  safetyLevel = co.string;     // "safe" | "moderate" | "high" | "critical"
  executed = co.boolean;
  success = co.optional.boolean;

  // Timestamps
  createdAt = co.number;
  executedAt = co.optional.number;
  deletedAt = co.optional.number;
}
```

### SyncPreferences

```typescript
export class SyncPreferences extends CoMap {
  // Each preference is key -> encrypted value
  entries = co.ref(PreferenceEntries);

  // Last update for conflict resolution
  updatedAt = co.number;
}

export class PreferenceEntries extends CoMap {
  [key: string]: co.string;  // key -> base64 encrypted value
}
```

### DeviceList

```typescript
export class DeviceList extends CoList.Of(DeviceRecord) {}

export class DeviceRecord extends CoMap {
  deviceId = co.string;
  name = co.string;
  platform = co.string;        // "linux" | "macos" | "windows"
  schemaVersion = co.number;
  firstSeen = co.number;
  lastSeen = co.number;
  isActive = co.boolean;
}
```

---

## Sync Strategies

### Push Strategy

1. Rust CLI stores command locally
2. CLI sends encrypted entry via IPC
3. Daemon appends to `CommandHistoryFeed`
4. Jazz syncs to relay automatically
5. Daemon confirms to CLI

### Pull Strategy

1. Jazz receives update from relay
2. Daemon's subscription callback fires
3. Daemon stores in local cache
4. Next CLI query returns new entries

### Conflict Resolution

- **Command History**: Append-only (CoFeed), no conflicts
- **Preferences**: Last-write-wins based on `updatedAt`
- **Device List**: Merge by `deviceId`, update `lastSeen`

---

## Error Handling

### Retry Policy

| Error Type | Retry | Backoff | Max Attempts |
|------------|-------|---------|--------------|
| Network timeout | Yes | Exponential (1s, 2s, 4s, 8s) | 5 |
| Rate limit | Yes | Use Retry-After header | 3 |
| Auth failure | No | - | - |
| Invalid data | No | - | - |
| Server error (5xx) | Yes | Exponential | 3 |

### Offline Queue

When offline:
1. Push operations are queued to SQLite
2. Queue is replayed when online
3. Queue entries have TTL (24 hours default)
4. Conflicts resolved on replay

---

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CARO_SYNC_SOCKET` | `~/.config/cmdai/sync.sock` | IPC socket path |
| `CARO_SYNC_RELAY` | `wss://mesh.jazz.tools` | Jazz relay URL |
| `CARO_SYNC_LOG_LEVEL` | `info` | Logging level |
| `CARO_SYNC_CACHE_PATH` | `~/.cache/cmdai/sync.db` | Local cache DB |

### Config File

`~/.config/cmdai/sync-daemon.json`:

```json
{
  "relay": "wss://mesh.jazz.tools",
  "cacheMaxSize": 104857600,
  "syncIntervalMs": 5000,
  "offlineQueueTtlMs": 86400000,
  "logLevel": "info"
}
```

---

## Monitoring

### Health Endpoint

Daemon exposes health via IPC:

```json
{
  "method": "daemon.health",
  "params": {}
}
```

Response:
```json
{
  "healthy": true,
  "uptime": 3600,
  "state": "ONLINE",
  "jazz": {
    "connected": true,
    "relay": "wss://mesh.jazz.tools",
    "latency": 45
  },
  "cache": {
    "entries": 1234,
    "sizeBytes": 5242880
  },
  "queue": {
    "pending": 0,
    "failed": 0
  }
}
```

### Metrics (Future)

- `caro_sync_push_total`: Counter of push operations
- `caro_sync_pull_total`: Counter of pull operations
- `caro_sync_latency_ms`: Histogram of sync latency
- `caro_sync_queue_size`: Gauge of offline queue size
