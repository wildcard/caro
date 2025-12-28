# Research: Karo Distributed Mesh

**Document**: Technology Research and Decisions
**Version**: 1.0.0
**Date**: December 2025

---

## Research Summary

This document captures research findings for implementing Karo's distributed mesh functionality.

---

## 1. Cryptographic Libraries

### Question
Which Rust crates should be used for Ed25519, X25519, and TLS 1.3?

### Research

| Crate | Pros | Cons | Decision |
|-------|------|------|----------|
| **ring** | Audited, fast, maintained by Brian Smith | No WASM, C dependencies | **SELECTED** for crypto |
| **ed25519-dalek** | Pure Rust, WASM-friendly | Less audited than ring | Alternative |
| **rustls** | Pure Rust TLS, no OpenSSL | Slightly larger binary | **SELECTED** for TLS |
| **native-tls** | Uses OS TLS stack | Not consistent across platforms | Rejected |

### Decision
- **ring** for Ed25519 signing, X25519 key exchange, and symmetric crypto
- **rustls** for TLS 1.3 transport with certificate handling
- **blake3** for hashing (faster than SHA-256, no length extension)

### Rationale
- `ring` is audited and maintained by a Google cryptographer
- `rustls` provides modern TLS without OpenSSL complexity
- Both have excellent performance on ARM (Apple Silicon priority)

---

## 2. Local Storage

### Question
What storage solution for local events, summaries, and audit logs?

### Research

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| **SQLite** | Single file, ACID, SQL query | Larger dependency | **SELECTED** |
| **sled** | Pure Rust, embedded | Less mature, no SQL | Alternative |
| **RocksDB** | High performance | Heavy C++ dep | Rejected |
| **JSON files** | Simple | No querying, corruption risk | Rejected |

### Decision
- **rusqlite** crate for SQLite access
- Single database file at `~/.local/share/karo/karo.db`
- WAL mode for concurrent reads
- VACUUM on startup if fragmented

### Rationale
- SQLite is battle-tested for local storage
- SQL enables flexible querying for summaries
- Atomic transactions prevent corruption
- Works well with air-gap requirement (no network)

---

## 3. Shell Integration

### Question
How to capture commands from bash, zsh, and fish shells?

### Research

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| **Shell hooks** (preexec/precmd) | Accurate, real-time | Per-shell config | **SELECTED** |
| **History file polling** | Simple | Delayed, misses failed cmds | Fallback |
| **PTY wrapper** | Complete capture | Complex, intrusive | Rejected |
| **eBPF** | System-wide | Linux-only, privileges | Future |

### Decision
- Primary: Shell hooks for bash (DEBUG trap), zsh (preexec), fish (fish_preexec)
- Fallback: History file polling for unsupported shells
- Installation: `karo shell init bash >> ~/.bashrc`

### Shell Hook Implementations

**Bash:**
```bash
# ~/.bashrc
eval "$(karo shell init bash)"

# Expands to:
__karo_preexec() {
    karo __internal observe "$BASH_COMMAND"
}
trap '__karo_preexec' DEBUG
```

**Zsh:**
```zsh
# ~/.zshrc
eval "$(karo shell init zsh)"

# Expands to:
preexec() {
    karo __internal observe "$1"
}
```

**Fish:**
```fish
# ~/.config/fish/config.fish
karo shell init fish | source

# Expands to:
function __karo_preexec --on-event fish_preexec
    karo __internal observe $argv
end
```

---

## 4. Peer Discovery

### Question
How to discover other Karo nodes on internal networks?

### Research

| Method | Pros | Cons | Decision |
|--------|------|------|----------|
| **Static config** | Reliable, explicit | Manual setup | **DEFAULT** |
| **mDNS/Bonjour** | Zero-config LAN | May not work across subnets | **OPTIONAL** |
| **DNS-SD** | Works with internal DNS | Requires DNS config | Future |
| **Broadcast UDP** | Simple | Not recommended for security | Rejected |

### Decision
- Default: Static peer list in config file
- Optional: mDNS via `mdns` crate for local discovery
- mDNS disabled by default (security stance)
- Service name: `_karo._tcp.local`

### mDNS Record
```
_karo._tcp.local. IN SRV 0 0 9238 hostname.local.
_karo._tcp.local. IN TXT "version=1" "fingerprint=a1b2c3d4"
```

---

## 5. Serialization

### Question
What format for wire messages between nodes?

### Research

| Format | Pros | Cons | Decision |
|--------|------|------|----------|
| **MessagePack** | Compact, schema-less | Binary (not human-readable) | **SELECTED** |
| **JSON** | Human-readable, debug-friendly | Verbose | Dashboard API |
| **Protobuf** | Schema, versioning | Heavy tooling | Rejected |
| **CBOR** | Compact, IETF standard | Less common | Alternative |

### Decision
- **rmp-serde** for inter-node MessagePack
- **serde_json** for dashboard REST API and config files
- Custom framing with magic bytes + length prefix

### Wire Format
```
[MAGIC:4][VERSION:2][TYPE:2][LENGTH:4][PAYLOAD:N][SIGNATURE:64]
```

---

## 6. Background Service Architecture

### Question
How to run Karo as a persistent background service?

### Research

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| **Systemd/launchd** | Native, reliable | Platform-specific | **SELECTED** |
| **Self-daemonize** | Cross-platform | Complex, error-prone | Rejected |
| **Always-running CLI** | Simple | Requires terminal | Fallback |

### Decision
- macOS: launchd plist in `~/Library/LaunchAgents/`
- Linux: systemd user service in `~/.config/systemd/user/`
- Windows: Future (Windows Service API)
- CLI fallback: `karo service run` for foreground mode

### Service Installation
```bash
# macOS
karo service install  # Creates launchd plist
launchctl load ~/Library/LaunchAgents/com.karo.agent.plist

# Linux
karo service install  # Creates systemd unit
systemctl --user enable --now karo
```

---

## 7. Web Dashboard

### Question
What framework for the local web dashboard?

### Research

| Framework | Pros | Cons | Decision |
|-----------|------|------|----------|
| **Axum** | Async, tower ecosystem | Newer | **SELECTED** |
| **Actix-web** | Fast, mature | Complex actors | Alternative |
| **Warp** | Simple, filters | Less flexible | Rejected |
| **Rocket** | Ergonomic | Slower compile | Rejected |

### Decision
- **axum** for HTTP server (integrates with tokio)
- **tower** middleware for auth, logging
- Static files embedded via `rust-embed`
- Frontend: Vanilla JS + HTMX (minimal bundle size)

### API Design
```
GET  /api/v1/health           # Health check
GET  /api/v1/identity         # Node identity
GET  /api/v1/events           # Recent events (paginated)
GET  /api/v1/summaries        # Summaries (paginated)
GET  /api/v1/peers            # Connected peers
POST /api/v1/query            # Execute mesh query
GET  /api/v1/audit            # Audit log
```

---

## 8. Message Ordering and Delivery

### Question
How to ensure reliable message delivery between nodes?

### Research

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| **TCP only** | Simple, ordered | Head-of-line blocking | **SELECTED** |
| **QUIC** | Multiplexed, fast | Complex, UDP may be blocked | Future |
| **Custom UDP** | Low latency | Reliability is complex | Rejected |

### Decision
- TCP with persistent connections (keep-alive)
- Reconnect with exponential backoff (1s, 2s, 4s, 8s, max 60s)
- Heartbeat every 30 seconds
- Connection timeout: 10 seconds

### Connection State Machine
```
DISCONNECTED → CONNECTING → AUTHENTICATING → CONNECTED → DISCONNECTING → DISCONNECTED
                   ↓              ↓               ↓
              FAILED         REJECTED        ERROR/TIMEOUT
```

---

## 9. Time Synchronization

### Question
How to handle clock skew between nodes?

### Research

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| **Reject old messages** | Simple | May reject valid msgs | **SELECTED** |
| **NTP sync requirement** | Accurate | Breaks air-gap | Rejected |
| **Logical clocks** | No wall-clock | Complex ordering | Future |

### Decision
- Accept messages with timestamp within ±5 minutes of local time
- Log clock skew warnings when > 1 minute
- Pong messages include RTT for clock offset estimation
- No hard requirement for NTP (air-gap compatible)

---

## 10. Privacy-Preserving Aggregation

### Question
How to aggregate data without exposing individual nodes?

### Research

| Technique | Pros | Cons | Decision |
|-----------|------|------|----------|
| **K-anonymity** | Simple threshold | May lose detail | **SELECTED** |
| **Differential privacy** | Mathematically rigorous | Noisy results | Future |
| **Homomorphic** | Compute on encrypted | Very slow | Rejected |
| **Secure aggregation** | Cryptographic | Complex protocol | Future |

### Decision
- Minimum 5 contributors required for any aggregate
- No per-node breakdowns in aggregate responses
- Category/risk distributions only (no raw counts if < 5)
- Future: Differential privacy for sensitive metrics

---

## Dependencies Summary

```toml
[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# Cryptography
ring = "0.17"
rustls = "0.23"
blake3 = "1.5"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rmp-serde = "1"

# Storage
rusqlite = { version = "0.32", features = ["bundled"] }

# Web
axum = "0.7"
tower = "0.5"
rust-embed = "8"

# Discovery (optional)
mdns = { version = "3", optional = true }

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## Open Questions

1. **Windows support**: What service framework for Windows? (Task Scheduler, Windows Service)
2. **Distributed queries**: How to handle partial results when some nodes are offline?
3. **Key backup**: Should we support encrypted key export/import?
4. **Rate limiting**: How to prevent query flooding in large meshes?

---

*Research completed December 2025. See plan.md for implementation approach.*
