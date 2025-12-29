# IPC Protocol Contract

**Version**: 1.0.0
**Transport**: Unix Domain Socket (POSIX) / Named Pipe (Windows)
**Protocol**: JSON-RPC 2.0 over newline-delimited JSON

---

## Connection Details

### Socket Paths
- **Linux/macOS**: `~/.config/caro/sync.sock`
- **Windows**: `\\.\pipe\caro-sync`

### Connection Lifecycle
1. Rust CLI connects to socket
2. If connection fails, CLI starts daemon process
3. CLI waits up to 5 seconds for daemon to be ready
4. All messages are newline-delimited JSON-RPC 2.0

---

## Message Format

### Request
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "sync.status",
  "params": {}
}
```

### Response (Success)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "connected": true,
    "lastSync": 1703062800000
  }
}
```

### Response (Error)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Not initialized",
    "data": {"hint": "Run 'caro sync init' first"}
  }
}
```

---

## Methods

### sync.status

Get current sync status.

**Request**:
```json
{
  "method": "sync.status",
  "params": {}
}
```

**Response**:
```typescript
interface SyncStatusResponse {
  connected: boolean;        // Connected to Jazz relay
  lastSync: number | null;   // Unix timestamp (ms) of last successful sync
  pendingUp: number;         // Entries waiting to be pushed
  pendingDown: number;       // Entries waiting to be pulled
  deviceId: string;          // This device's ID
  accountId: string | null;  // Jazz account ID (null if not initialized)
}
```

**Errors**:
- `-32001`: Daemon not initialized

---

### sync.init

Initialize sync with Jazz account credentials.

**Request**:
```json
{
  "method": "sync.init",
  "params": {
    "accountId": "co_zABC123...",
    "authKey": "base64-encoded-key",
    "deviceName": "MacBook Pro",
    "platform": "macos"
  }
}
```

**Response**:
```typescript
interface InitResponse {
  success: boolean;
  deviceId: string;
  syncRootId: string;  // CaroSyncRoot CoValue ID
}
```

**Errors**:
- `-32002`: Invalid credentials
- `-32003`: Network unreachable
- `-32004`: Already initialized (use sync.reset first)

---

### sync.push

Push encrypted entries to sync.

**Request**:
```json
{
  "method": "sync.push",
  "params": {
    "entries": [
      {
        "id": "uuid-v4",
        "type": "command",
        "encrypted": "base64-encoded-encrypted-blob",
        "metadata": {
          "deviceId": "device-uuid",
          "createdAt": 1703062800000,
          "safetyLevel": "safe"
        }
      }
    ]
  }
}
```

**Response**:
```typescript
interface PushResponse {
  synced: number;           // Number of entries successfully synced
  failed: string[];         // IDs of entries that failed
  errors: {
    id: string;
    code: number;
    message: string;
  }[];
}
```

**Errors**:
- `-32001`: Daemon not initialized
- `-32005`: Rate limited
- `-32006`: Payload too large (max 1MB per request)

---

### sync.pull

Pull new entries from sync.

**Request**:
```json
{
  "method": "sync.pull",
  "params": {
    "since": 1703062800000,  // Unix timestamp (ms), null for full sync
    "limit": 100             // Max entries to return (default 100, max 1000)
  }
}
```

**Response**:
```typescript
interface PullResponse {
  entries: {
    id: string;
    type: "command" | "preference" | "device";
    encrypted: string;       // Base64-encoded encrypted blob
    metadata: {
      deviceId: string;
      createdAt: number;
      updatedAt: number;
      deletedAt?: number;    // Present if soft-deleted
    };
  }[];
  hasMore: boolean;          // More entries available
  nextCursor: number;        // Timestamp to use for next pull
  syncTimestamp: number;     // Server timestamp for consistency
}
```

**Errors**:
- `-32001`: Daemon not initialized
- `-32003`: Network unreachable

---

### device.register

Register a new device.

**Request**:
```json
{
  "method": "device.register",
  "params": {
    "name": "MacBook Pro",
    "platform": "macos"
  }
}
```

**Response**:
```typescript
interface RegisterResponse {
  deviceId: string;
  registeredAt: number;
}
```

---

### device.list

List all registered devices.

**Request**:
```json
{
  "method": "device.list",
  "params": {}
}
```

**Response**:
```typescript
interface DeviceListResponse {
  devices: {
    deviceId: string;
    name: string;
    platform: string;
    lastSeen: number;
    isCurrentDevice: boolean;
    schemaVersion: number;
  }[];
}
```

---

### daemon.ping

Health check for daemon.

**Request**:
```json
{
  "method": "daemon.ping",
  "params": {}
}
```

**Response**:
```typescript
interface PingResponse {
  pong: true;
  version: string;       // Daemon version
  uptime: number;        // Seconds since daemon started
}
```

---

### daemon.shutdown

Gracefully shutdown the daemon.

**Request**:
```json
{
  "method": "daemon.shutdown",
  "params": {
    "reason": "user-requested"
  }
}
```

**Response**:
```typescript
interface ShutdownResponse {
  acknowledged: true;
  pendingOperations: number;  // Operations that will complete before shutdown
}
```

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse Error | Invalid JSON |
| -32600 | Invalid Request | Not a valid JSON-RPC request |
| -32601 | Method Not Found | Unknown method |
| -32602 | Invalid Params | Invalid method parameters |
| -32603 | Internal Error | Internal daemon error |
| -32001 | Not Initialized | Sync not initialized |
| -32002 | Invalid Credentials | Auth failed |
| -32003 | Network Unreachable | Cannot reach Jazz relay |
| -32004 | Already Initialized | Sync already set up |
| -32005 | Rate Limited | Too many requests |
| -32006 | Payload Too Large | Request exceeds size limit |
| -32007 | Encryption Error | Failed to process encrypted data |

---

## Timeouts

| Operation | Timeout | Notes |
|-----------|---------|-------|
| Connection | 5s | Time to establish socket connection |
| Request | 30s | Time for single request/response |
| Push (batch) | 60s | Large batch operations |
| Pull (full sync) | 300s | Initial full sync |

---

## Versioning

Protocol version is negotiated on first message:

```json
{
  "method": "daemon.handshake",
  "params": {
    "clientVersion": "0.1.0",
    "protocolVersion": "1.0.0"
  }
}
```

Response includes supported version range:
```json
{
  "result": {
    "protocolVersion": "1.0.0",
    "minSupportedVersion": "1.0.0"
  }
}
```
