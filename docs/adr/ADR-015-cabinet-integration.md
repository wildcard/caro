# ADR-015: Cabinet Integration for AI Company Orchestration

| **Status**     | Proposed                            |
|----------------|-------------------------------------|
| **Date**       | April 2026                          |
| **Authors**    | Caro Maintainers                    |
| **Target**     | Community                           |

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [Decision](#decision)
4. [Rationale](#rationale)
5. [Consequences](#consequences)
6. [Alternatives Considered](#alternatives-considered)
7. [Implementation Notes](#implementation-notes)
8. [Success Metrics](#success-metrics)
9. [References](#references)

---

## Executive Summary

This ADR proposes integrating caro with [cabinet](https://github.com/hilash/cabinet), an AI-first knowledge base and agent orchestration platform, to enable caro as the **safe command execution layer** for AI-driven company workflows.

The integration adds a new `caro-server` binary target that exposes caro's command generation and safety validation pipeline as an HTTP/WebSocket API. Cabinet agents connect to this API to safely generate and execute shell commands, while caro's knowledge index and cabinet's markdown knowledge base sync bidirectionally.

**Key Decision**: Expose caro as an HTTP API server (separate binary, feature-gated), not as a new `CommandGenerator` backend.

---

## Context and Problem Statement

### The Problem

Caro excels at converting natural language to safe shell commands, but it operates as a single-user CLI tool. There is no way for external systems to programmatically invoke caro's pipeline (command generation, safety validation, execution) without shelling out to the binary.

### What Cabinet Provides

Cabinet is a TypeScript/Next.js platform that acts as a "startup OS":
- **Markdown-based, git-backed knowledge base** (no database)
- **20+ AI agent templates** (CEO, product managers, engineers, marketers)
- **WebSocket daemon** for real-time agent communication
- **Cron-based job scheduling** via node-cron
- **Web terminal** with Claude Code integration

### The Opportunity

Together, caro + cabinet create a system where:
1. Cabinet orchestrates **what** needs to happen (agent decisions, scheduled tasks)
2. Caro handles **how** it happens safely (command generation, safety validation, execution)
3. Knowledge flows bidirectionally (caro learns from executions, cabinet retains organizational context)

### Forces at Play

- Caro is Rust; cabinet is TypeScript/Node.js -- need a clean cross-language boundary
- Safety validation must never be byppassable, even via API
- Caro's existing `CommandGenerator` trait is designed for LLM backends, not API consumers
- Cabinet stores everything as markdown files with git history -- no database integration needed
- Both tools run locally -- network latency is negligible

---

## Decision

**Expose caro's command pipeline as an HTTP/WebSocket API server** via a new `caro-server` binary target, feature-gated under a `server` Cargo feature.

Specifically:

1. **New binary**: `caro-server` (not baked into the main `caro` CLI)
2. **Transport**: HTTP REST endpoints + WebSocket for streaming/real-time
3. **Auth**: Bearer token (shared secret in config)
4. **Safety**: All API requests pass through the same `SafetyValidator` as CLI usage
5. **Feature gate**: `server` feature flag in `Cargo.toml` (optional dependency on `axum`)
6. **Knowledge sync**: Export/import endpoints for bidirectional knowledge sharing

The API does NOT bypass the `CommandGenerator` trait -- it wraps the existing `AgentLoop` which already orchestrates backend selection, command generation, and safety validation.

---

## Rationale

### Why HTTP/JSON (not FFI, WASM, or CLI wrapping)

| Approach | Pros | Cons |
|----------|------|------|
| **HTTP/JSON** | Language-agnostic, debuggable, well-understood | Network overhead (negligible on localhost) |
| FFI (C ABI) | Zero overhead | Brittle, unsafe, complex build | 
| WASM | Portable | No filesystem/network access without WASI |
| CLI subprocess | No new code | Parsing stdout is fragile, no streaming |

HTTP/JSON is the simplest approach that works. On localhost, latency is sub-millisecond for the transport layer. The real latency is LLM inference (100-2000ms), making transport overhead irrelevant.

### Why a separate binary (not embedded in `caro`)

- **Dependency isolation**: Users who only want the CLI never pay for `axum`, `tower`, `tokio-tungstenite`
- **Follows existing pattern**: Caro already has multiple binary targets (`caro`, `caro-eval`, `generate-schema`)
- **Single responsibility**: CLI handles interactive terminal UX; server handles API concerns
- **Independent lifecycle**: Server can be deployed as a daemon without CLI baggage

### Why NOT a new CommandGenerator backend

The `CommandGenerator` trait (`src/backends/mod.rs:17-32`) is designed for **sources of LLM inference** -- it takes a `CommandRequest` and returns a `GeneratedCommand`. Cabinet is a **consumer** of the pipeline, not a source. Making cabinet a backend would invert the abstraction.

### Safety is non-negotiable

The API exposes the same safety guarantees as the CLI:
- All generated commands pass through `SafetyValidator` (52+ patterns)
- Risk levels (Safe, Moderate, High, Critical) are returned in every response
- The `/execute` endpoint requires explicit confirmation for risky commands
- There is no "bypass safety" flag in the API

---

## Consequences

### Benefits

- **Programmatic access**: Any HTTP client can use caro's pipeline (not just cabinet)
- **AI company enablement**: Cabinet agents gain safe command execution capabilities
- **Knowledge compounding**: Successful commands feed both caro's vector DB and cabinet's markdown KB
- **No core changes**: The existing CLI, backends, and safety modules are untouched
- **Deployable**: `caro-server` can run as a systemd service or Docker container

### Trade-offs

- **New binary to maintain**: `caro-server` adds surface area (HTTP routes, auth, CORS)
- **Feature flag complexity**: `server` feature adds another build configuration
- **Auth is simple**: Bearer token is sufficient for trusted networks but not for public exposure
- **Two-process model**: Users run both `caro-server` and cabinet's daemon

### Risks

- **Security**: Exposing command execution over HTTP requires careful auth and CORS configuration. **Mitigation**: Default to `127.0.0.1` binding, require explicit config to listen on other interfaces.
- **API stability**: Once cabinet depends on the API, breaking changes are costly. **Mitigation**: Version the API (`/api/v1/...`), follow semver for the `server` feature.
- **Scope creep**: The API could grow to replicate the entire CLI. **Mitigation**: Limit to generation, execution, and knowledge -- no interactive features.

---

## Alternatives Considered

### Alternative 1: MCP (Model Context Protocol) Server

- **Description**: Expose caro as an MCP tool server that cabinet agents can invoke
- **Pros**: Standard protocol for AI tool use; growing ecosystem
- **Cons**: MCP is designed for AI model tool calls, not general API access; adds protocol complexity; not well-suited for streaming execution output
- **Verdict**: Worth considering for Phase 2, but HTTP is more universal for Phase 1

### Alternative 2: gRPC

- **Description**: Use gRPC with protobuf for the API layer
- **Pros**: Strongly typed, efficient binary protocol, built-in streaming
- **Cons**: Protobuf adds build complexity; TypeScript gRPC clients are heavier than `fetch()`; overkill for localhost communication
- **Verdict**: Over-engineered for the use case

### Alternative 3: Unix Domain Socket

- **Description**: Use a Unix socket instead of TCP for IPC
- **Pros**: No port allocation, slightly lower overhead, inherits file permissions
- **Cons**: Not cross-platform (no Windows); harder to debug (no curl); TypeScript support is less mature
- **Verdict**: Could be offered as an alternative transport in Phase 2

### Alternative 4: Embed cabinet in Rust

- **Description**: Rewrite cabinet's agent orchestration in Rust as a caro module
- **Pros**: Single binary, no IPC overhead
- **Cons**: Massive scope; duplicates cabinet's existing TypeScript ecosystem; loses cabinet's community and templates
- **Verdict**: Not practical

---

## Implementation Notes

See [Cabinet Integration Specification](../cabinet-integration-spec.md) for the full API specification.

### Phase 1: HTTP API Server (MVP)

**New files:**
- `src/server/mod.rs` -- server module root
- `src/server/types.rs` -- API request/response types
- `src/server/routes.rs` -- HTTP route handlers
- `src/server/ws.rs` -- WebSocket protocol
- `src/bin/caro_server.rs` -- server binary entry point

**Modified files:**
- `Cargo.toml` -- add `server` feature, axum dependencies, binary target
- `src/lib.rs` -- add `#[cfg(feature = "server")] pub mod server;`
- `src/models/mod.rs` -- add `ServerConfig` struct to `UserConfiguration`

**New dependencies (feature-gated):**
- `axum` (HTTP framework)
- `tower` (middleware)
- `tokio-tungstenite` (WebSocket)

### Phase 2: Cabinet Agent Template

- npm package `@caro/cabinet` with TypeScript client
- Cabinet agent template that wraps the API
- Documentation for cabinet users

### Phase 3: Knowledge Sync

- Export/import endpoints for bidirectional knowledge sharing
- Cron job template for scheduled sync
- Markdown format for knowledge entries

### Testing Strategy

- Unit tests for API types serialization/deserialization
- Integration tests using `axum::test` for route handlers
- End-to-end test with a mock cabinet client
- Safety validation tests through the API (ensure no bypass)

---

## Success Metrics

| Metric | Target |
|--------|--------|
| API response time (generate) | < 50ms overhead over backend inference time |
| Safety validation coverage | 100% parity with CLI (same validator, same patterns) |
| Knowledge sync latency | < 5s for export/import cycle |
| Auth bypass attempts blocked | 100% (no unauthenticated access when token configured) |

---

## References

- [Cabinet repository](https://github.com/hilash/cabinet) -- AI-first knowledge base and agent orchestration
- `src/backends/mod.rs` -- `CommandGenerator` trait definition
- `src/models/mod.rs` -- Core data types (`CommandRequest`, `GeneratedCommand`, `BackendType`)
- `src/safety/patterns.rs` -- 52+ dangerous command patterns
- `src/agent/mod.rs` -- `AgentLoop` for iterative command refinement
- `src/execution/executor.rs` -- `CommandExecutor` for safe command execution
- `src/config/mod.rs` -- Configuration management
- [ADR-010](./ADR-010-bubblewrap-sandbox-execution.md) -- Bubblewrap sandbox (complementary security layer)

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-04-03 | Caro Maintainers | Initial draft |
