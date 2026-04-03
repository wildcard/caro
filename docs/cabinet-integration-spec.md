# Cabinet Integration Specification

> **Status**: Draft  
> **ADR**: [ADR-015](./adr/ADR-015-cabinet-integration.md)  
> **Date**: April 2026

This document specifies the HTTP/WebSocket API that `caro-server` exposes for integration with [cabinet](https://github.com/hilash/cabinet) and other external systems.

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Configuration](#configuration)
4. [REST API Endpoints](#rest-api-endpoints)
5. [WebSocket Protocol](#websocket-protocol)
6. [Safety Enforcement](#safety-enforcement)
7. [Knowledge Sync Protocol](#knowledge-sync-protocol)
8. [Cabinet Agent Template](#cabinet-agent-template)
9. [Deployment](#deployment)
10. [Error Handling](#error-handling)

---

## Overview

`caro-server` wraps caro's existing command pipeline (`AgentLoop` + `SafetyValidator` + `CommandExecutor`) in an HTTP API. It is a separate binary target, feature-gated under `server` in `Cargo.toml`.

```
Cabinet Agent  ──HTTP──▶  caro-server  ──▶  AgentLoop
                                             ├── StaticMatcher (0ms)
                                             ├── EmbeddedBackend (100-2000ms)
                                             └── RemoteBackend (200-2000ms)
                                        ──▶  SafetyValidator (52+ patterns)
                                        ──▶  CommandExecutor
```

---

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────┐
│                caro-server                   │
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐ │
│  │  Routes   │  │   Auth   │  │   CORS    │ │
│  │ (axum)   │  │ (tower)  │  │  (tower)  │ │
│  └────┬─────┘  └────┬─────┘  └─────┬─────┘ │
│       │              │              │        │
│  ┌────▼──────────────▼──────────────▼─────┐ │
│  │              AppState (Arc)             │ │
│  │                                         │ │
│  │  ┌───────────┐  ┌──────────────────┐   │ │
│  │  │ AgentLoop │  │ SafetyValidator  │   │ │
│  │  └───────────┘  └──────────────────┘   │ │
│  │  ┌───────────┐  ┌──────────────────┐   │ │
│  │  │ Executor  │  │ KnowledgeIndex?  │   │ │
│  │  └───────────┘  └──────────────────┘   │ │
│  └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

### Shared State

```rust
// src/server/mod.rs
struct AppState {
    agent_loop: AgentLoop,
    validator: SafetyValidator,
    executor: CommandExecutor,
    config: ServerConfig,
    #[cfg(feature = "knowledge")]
    knowledge: Option<Arc<KnowledgeIndex>>,
}
```

---

## Configuration

### TOML Configuration

Added to `~/.config/caro/config.toml`:

```toml
[server]
host = "127.0.0.1"       # Bind address (default: localhost only)
port = 3847               # Port number (default: 3847)
auth_token = "secret"     # Bearer token for authentication (optional)
allowed_origins = []      # CORS origins (empty = same-origin only)
```

### Rust Type

```rust
// Added to src/models/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,              // "127.0.0.1"
    #[serde(default = "default_port")]
    pub port: u16,                 // 3847
    pub auth_token: Option<String>,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}
```

### Environment Variable Overrides

| Variable | Purpose |
|----------|---------|
| `CARO_SERVER_HOST` | Override bind address |
| `CARO_SERVER_PORT` | Override port |
| `CARO_SERVER_TOKEN` | Override auth token |

---

## REST API Endpoints

All endpoints are prefixed with `/api/v1/`. Responses use `Content-Type: application/json`.

### Authentication

When `auth_token` is configured, all requests must include:

```
Authorization: Bearer <token>
```

Unauthenticated requests receive `401 Unauthorized`. If no token is configured, all requests are accepted (localhost-only deployments).

---

### `GET /api/v1/health`

Health check and backend status.

**Response** `200 OK`:

```json
{
  "status": "ok",
  "version": "1.2.0",
  "backends": {
    "static_matcher": true,
    "embedded": true,
    "ollama": false,
    "claude": false
  },
  "safety_patterns": 52,
  "uptime_seconds": 3600
}
```

---

### `POST /api/v1/generate`

Generate a shell command from natural language. Does NOT execute the command.

**Request:**

```json
{
  "input": "find all Python files larger than 1MB",
  "shell": "bash",
  "safety_level": "moderate",
  "context": "/home/user/project",
  "request_id": "req-abc-123",
  "agent_id": "devops-agent"
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `input` | string | yes | -- | Natural language query |
| `shell` | string | no | `"bash"` | Target shell (`bash`, `zsh`, `sh`, `fish`) |
| `safety_level` | string | no | `"moderate"` | `strict`, `moderate`, or `permissive` |
| `context` | string | no | `null` | Working directory or additional context |
| `request_id` | string | no | auto-generated | Client-provided request ID for correlation |
| `agent_id` | string | no | `null` | Cabinet agent identifier (for logging) |

**Response** `200 OK`:

```json
{
  "request_id": "req-abc-123",
  "status": "ok",
  "command": "find . -name '*.py' -type f -size +1M",
  "explanation": "Recursively finds Python files larger than 1MB in the current directory",
  "risk_level": "safe",
  "estimated_impact": "Read-only filesystem traversal",
  "alternatives": [
    "find . -name '*.py' -size +1048576c"
  ],
  "backend_used": "static_matcher",
  "generation_time_ms": 2,
  "confidence_score": 0.95,
  "warnings": []
}
```

**Response** `200 OK` (blocked by safety):

```json
{
  "request_id": "req-abc-456",
  "status": "blocked",
  "command": "rm -rf /",
  "explanation": "Recursively removes all files from root",
  "risk_level": "critical",
  "warnings": [
    "Matches pattern: recursive deletion of root filesystem"
  ],
  "reason": "Command blocked by safety validation at 'moderate' level"
}
```

**Response** `400 Bad Request`:

```json
{
  "request_id": "req-abc-789",
  "status": "error",
  "error": "Input cannot be empty"
}
```

---

### `POST /api/v1/execute`

Execute a previously generated command. Requires explicit confirmation for risky commands.

**Request:**

```json
{
  "command": "find . -name '*.py' -type f -size +1M",
  "confirmed": true,
  "request_id": "req-abc-123",
  "timeout_ms": 30000
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `command` | string | yes | -- | The command to execute |
| `confirmed` | bool | yes | -- | Explicit safety confirmation |
| `request_id` | string | no | auto-generated | Correlation ID |
| `timeout_ms` | u64 | no | `30000` | Execution timeout in milliseconds |

**Response** `200 OK`:

```json
{
  "request_id": "req-abc-123",
  "status": "ok",
  "exit_code": 0,
  "stdout": "./src/heavy_model.py\n./data/processor.py\n",
  "stderr": "",
  "execution_time_ms": 45
}
```

**Response** `403 Forbidden` (unconfirmed risky command):

```json
{
  "request_id": "req-abc-456",
  "status": "blocked",
  "error": "Command has risk level 'high' and requires confirmed=true",
  "risk_level": "high",
  "warnings": ["Matches pattern: recursive file deletion"]
}
```

**Important**: The execute endpoint re-validates the command through `SafetyValidator` before execution. A command that was `"status": "ok"` from `/generate` will still be validated again, ensuring safety even if the caller fabricates a command.

---

### `GET /api/v1/knowledge/search`

Search caro's knowledge index for past commands. Requires `knowledge` feature.

**Query parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `q` | string | yes | Search query |
| `limit` | u32 | no | Max results (default: 10) |

**Response** `200 OK`:

```json
{
  "results": [
    {
      "input": "find large python files",
      "command": "find . -name '*.py' -size +1M",
      "confidence": 0.92,
      "times_used": 5,
      "last_used": "2026-04-01T10:30:00Z"
    }
  ],
  "total": 1
}
```

**Response** `501 Not Implemented` (knowledge feature not enabled):

```json
{
  "status": "error",
  "error": "Knowledge index not available (compile with --features knowledge)"
}
```

---

### `POST /api/v1/knowledge/record`

Record a successful command execution to the knowledge index.

**Request:**

```json
{
  "input": "find all Python files larger than 1MB",
  "command": "find . -name '*.py' -type f -size +1M",
  "context": "/home/user/project",
  "success": true,
  "agent_id": "devops-agent"
}
```

**Response** `201 Created`:

```json
{
  "status": "ok",
  "message": "Knowledge entry recorded"
}
```

---

### `GET /api/v1/knowledge/export`

Export knowledge entries as markdown (for cabinet's git-backed KB).

**Query parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `since` | string (ISO 8601) | no | Only export entries after this date |
| `format` | string | no | `markdown` (default) or `json` |

**Response** `200 OK` (markdown format):

```markdown
## Command Knowledge Export

### find large python files
- **Command**: `find . -name '*.py' -type f -size +1M`
- **Context**: /home/user/project
- **Confidence**: 0.92
- **Times Used**: 5
- **Last Used**: 2026-04-01

### list running docker containers
- **Command**: `docker ps --format '{{.Names}}\t{{.Status}}'`
- **Confidence**: 0.88
- **Times Used**: 12
- **Last Used**: 2026-04-03
```

---

### `POST /api/v1/knowledge/import`

Import knowledge from cabinet's markdown knowledge base into caro's vector index.

**Request** (`Content-Type: text/markdown`):

```markdown
## DevOps Runbook Commands

### check disk usage
- **Command**: `df -h | sort -k5 -rn`
- **Context**: server maintenance
```

**Response** `200 OK`:

```json
{
  "status": "ok",
  "imported": 1,
  "skipped": 0
}
```

---

## WebSocket Protocol

### Connection

```
ws://localhost:3847/api/v1/ws
```

Authentication via query parameter when bearer token is configured:

```
ws://localhost:3847/api/v1/ws?token=<bearer-token>
```

### Message Format

All messages are JSON with a `type` field:

```typescript
type WsMessage =
  | { type: "command_request"; id: string; input: string; agent_id?: string }
  | { type: "command_result"; id: string; status: "ok" | "blocked" | "error"; command?: string; explanation?: string; risk_level?: string; warnings?: string[] }
  | { type: "execution_request"; id: string; command: string; confirmed: boolean }
  | { type: "execution_result"; id: string; exit_code: number; stdout: string; stderr: string; execution_time_ms: number }
  | { type: "knowledge_update"; entries: KnowledgeEntry[] }
  | { type: "heartbeat" }
  | { type: "error"; message: string }
```

### Flow Example

```
Client                          Server
  │                                │
  │──command_request──────────────▶│
  │  {id: "1", input: "list py"}  │
  │                                │  (runs AgentLoop + SafetyValidator)
  │◀──command_result───────────────│
  │  {id: "1", status: "ok",      │
  │   command: "ls *.py"}          │
  │                                │
  │──execution_request────────────▶│
  │  {id: "1", command: "ls *.py", │
  │   confirmed: true}             │
  │                                │  (runs CommandExecutor)
  │◀──execution_result─────────────│
  │  {id: "1", exit_code: 0,      │
  │   stdout: "main.py\ntest.py"} │
  │                                │
  │◀──heartbeat────────────────────│  (every 30s)
  │                                │
```

---

## Safety Enforcement

### Invariants

These properties hold for ALL API access:

1. **Every command is validated**: Both `/generate` and `/execute` run the full `SafetyValidator` pipeline from `src/safety/patterns.rs` (52+ compiled regex patterns).

2. **Risk levels are always returned**: Clients always see `risk_level` in responses, matching the `RiskLevel` enum (`safe`, `moderate`, `high`, `critical`).

3. **Blocking follows safety config**: The same `SafetyLevel` logic from `src/models/mod.rs:159-174` applies:
   - `strict`: Blocks High + Critical
   - `moderate`: Blocks Critical
   - `permissive`: Warns only

4. **No bypass endpoint**: There is no API flag to skip safety validation.

5. **Double validation on execute**: `/execute` re-validates even if the command came from `/generate`, preventing injection via fabricated requests.

### Audit Trail

All API requests are logged via caro's telemetry system (`src/telemetry/`) with:
- Request ID, agent ID, timestamp
- Command generated, risk level assessed
- Whether execution was attempted and its result
- Auth token hash (not the token itself)

---

## Knowledge Sync Protocol

### Sync Strategy

Knowledge sync between caro and cabinet is **async and pull-based**:

```
Cabinet cron job (every N hours)
  │
  ├── GET /api/v1/knowledge/export?since=<last_sync>
  │   └── Write markdown to cabinet's knowledge base
  │   └── git commit + push
  │
  └── POST /api/v1/knowledge/import
      └── Send relevant cabinet KB entries to caro
```

### Cabinet-Side Integration

Cabinet stores exported knowledge as markdown files in its git-backed KB:

```
knowledge/
├── caro-commands/
│   ├── 2026-04-01-export.md
│   ├── 2026-04-02-export.md
│   └── 2026-04-03-export.md
└── devops-runbooks/
    └── common-commands.md  (imported into caro)
```

---

## Cabinet Agent Template

### Agent Definition

A cabinet agent template (`agents/caro-executor/`) that other cabinet agents can delegate to:

```yaml
# agents/caro-executor/config.yaml
name: "Caro Command Executor"
description: "Safely generates and executes shell commands using caro"
type: "tool"

connection:
  url: "http://localhost:3847"
  auth_token: "${CARO_SERVER_TOKEN}"
  timeout_ms: 30000

safety:
  level: "moderate"
  require_confirmation_above: "moderate"
  auto_execute_safe: true

knowledge:
  sync_enabled: true
  sync_interval: "0 */6 * * *"  # Every 6 hours
```

### Agent Personality (agent.md)

```markdown
You are the Caro Command Executor, the team's safe shell command expert.

When other agents need to interact with the operating system:
1. Accept their natural language description of what needs to happen
2. Use caro to generate a safe command
3. Review the risk level and warnings
4. Execute only if the risk is acceptable
5. Report results back to the requesting agent

NEVER bypass safety validation. If a command is blocked, explain why
and suggest a safer alternative.
```

### TypeScript Client

The `@caro/cabinet` npm package provides a typed client:

```typescript
import { CaroClient } from '@caro/cabinet';

const caro = new CaroClient({
  url: 'http://localhost:3847',
  token: process.env.CARO_SERVER_TOKEN,
});

// Generate a command
const result = await caro.generate('find all log files older than 7 days');
if (result.status === 'ok' && result.risk_level === 'safe') {
  const execution = await caro.execute(result.command);
  console.log(execution.stdout);
}

// Search knowledge
const knowledge = await caro.searchKnowledge('docker cleanup');
```

---

## Deployment

### Single Machine (Default)

```bash
# Terminal 1: Start caro-server
cargo run --bin caro-server --features server

# Terminal 2: Start cabinet
npx create-cabinet@latest
# Configure caro-executor agent with localhost URL
```

### Systemd Service

```ini
[Unit]
Description=Caro Command Server
After=network.target

[Service]
ExecStart=/usr/local/bin/caro-server
Environment=CARO_SERVER_TOKEN=your-secret-token
Restart=always
User=caro

[Install]
WantedBy=multi-user.target
```

### Docker Compose (Multi-Service)

```yaml
services:
  caro-server:
    build:
      context: .
      args:
        FEATURES: "server,embedded-cpu"
    ports:
      - "3847:3847"
    environment:
      - CARO_SERVER_TOKEN=${CARO_SERVER_TOKEN}
      - CARO_SERVER_HOST=0.0.0.0
    volumes:
      - caro-cache:/root/.cache/caro
      - caro-config:/root/.config/caro

  cabinet:
    image: node:20
    working_dir: /app
    command: npm start
    depends_on:
      - caro-server
    environment:
      - CARO_SERVER_TOKEN=${CARO_SERVER_TOKEN}
```

---

## Error Handling

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| `200` | Success |
| `201` | Created (knowledge record) |
| `400` | Invalid request (bad input, missing fields) |
| `401` | Unauthorized (missing/invalid bearer token) |
| `403` | Forbidden (safety blocked execution) |
| `404` | Not found (unknown endpoint) |
| `408` | Timeout (command execution exceeded timeout) |
| `500` | Internal server error |
| `501` | Not implemented (feature not compiled) |
| `503` | Service unavailable (backend not ready) |

### Error Response Format

All errors follow a consistent format:

```json
{
  "status": "error",
  "error": "Human-readable error message",
  "request_id": "req-abc-123",
  "details": {}
}
```

---

## Appendix: Cargo.toml Changes

```toml
[features]
server = ["axum", "tower", "tower-http", "tokio-tungstenite"]

[dependencies]
axum = { version = "0.7", optional = true }
tower = { version = "0.4", optional = true }
tower-http = { version = "0.5", features = ["cors", "auth"], optional = true }
tokio-tungstenite = { version = "0.24", optional = true }

[[bin]]
name = "caro-server"
path = "src/bin/caro_server.rs"
required-features = ["server"]
```

## Appendix: File Map

| New File | Purpose |
|----------|---------|
| `src/server/mod.rs` | Server module root, `AppState` struct |
| `src/server/types.rs` | `ApiCommandRequest`, `ApiCommandResponse`, etc. |
| `src/server/routes.rs` | Axum route handlers for all endpoints |
| `src/server/ws.rs` | WebSocket message types and handler |
| `src/bin/caro_server.rs` | Binary entry point (CLI args, server startup) |
| `npm/cabinet-caro/package.json` | TypeScript client npm package |
| `npm/cabinet-caro/src/index.ts` | `CaroClient` class |
| `docs/cabinet-integration-spec.md` | This document |
| `docs/adr/ADR-015-cabinet-integration.md` | Architecture decision record |
