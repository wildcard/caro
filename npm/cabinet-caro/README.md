# @caro/cabinet

TypeScript client for [caro-server](https://github.com/wildcard/caro) — safe shell command generation and execution for [cabinet](https://github.com/hilash/cabinet) agents.

## Installation

```bash
npm install @caro/cabinet
```

## Quick Start

```typescript
import { CaroClient } from '@caro/cabinet';

const caro = new CaroClient({
  url: 'http://localhost:3847',
  token: process.env.CARO_SERVER_TOKEN,
});

// Generate a command
const result = await caro.generate('find all log files older than 7 days');
if (result.status === 'ok' && result.risk_level === 'safe') {
  const execution = await caro.execute(result.command!);
  console.log(execution.stdout);
}

// Search knowledge
const knowledge = await caro.searchKnowledge('docker cleanup');
console.log(knowledge.results);
```

## WebSocket

```typescript
const ws = caro.connectWebSocket();
await ws.connect();

ws.onMessage((msg) => {
  if (msg.type === 'command_result') {
    console.log(`Command: ${msg.command}`);
  }
});

ws.requestCommand('req-1', 'list running containers');
```

## API

### `CaroClient`

| Method | Description |
|--------|-------------|
| `health()` | Check server health |
| `generate(input)` | Generate shell command from natural language |
| `execute(command, options?)` | Execute a generated command |
| `searchKnowledge(query, limit?)` | Search knowledge base |
| `recordKnowledge(entry)` | Record command result |
| `exportKnowledge()` | Export all knowledge |
| `importKnowledge(entries)` | Import knowledge entries |
| `connectWebSocket()` | Open WebSocket connection |

### `CaroWebSocket`

| Method | Description |
|--------|-------------|
| `connect()` | Open connection (returns Promise) |
| `onMessage(handler)` | Register message handler |
| `requestCommand(id, input, agentId?)` | Request command generation |
| `requestExecution(id, command, confirmed?)` | Request command execution |
| `close()` | Close connection |

## License

AGPL-3.0
