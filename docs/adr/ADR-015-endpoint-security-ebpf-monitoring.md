# ADR-015: Apple Endpoint Security & eBPF Kernel-Level Agent Monitoring

**Status**: Proposed

**Date**: 2026-04-06

**Authors**: @wildcard, Claude Code

**Target**: Hybrid (Community foundation + Enterprise extensions)

**Depends on**:
- ADR-003 (Monitoring and Audit Trail System)
- ADR-010 (Bubblewrap Sandbox Execution)

## Context

Caro generates shell commands from natural language using LLM inference. Our current defense-in-depth stack consists of:

1. **Pattern-based safety validation** (52+ regex patterns in `src/safety/`) — catches known dangerous commands
2. **Bubblewrap sandbox** (ADR-010, proposed) — filesystem/network isolation at process level
3. **Telemetry** — local event logging and optional cloud sync

This stack has fundamental limitations:

- **Pattern evasion**: Attackers can encode dangerous commands using hex escapes, variable expansion, or obfuscation techniques that bypass regex matching
- **Novel attacks**: New dangerous patterns not in our database bypass validation entirely
- **Post-execution blind spots**: Once a command spawns child processes, we have no visibility into what those children do
- **No kernel-level enforcement**: If pattern matching fails, there's no lower-level safety net before operations hit disk or network

With AI agents like Anthropic's models becoming increasingly capable of autonomous operation, the attack surface expands: an agent may chain innocuous-looking commands that individually pass safety checks but collectively perform dangerous operations.

### The Kernel-Level Advantage

Apple's Endpoint Security (ES) framework and Linux's eBPF provide kernel-level syscall interception that operates below the application layer:

- **Every syscall is visible**: exec, open, write, connect, signal — nothing bypasses the kernel
- **AUTH events can block**: On macOS ES, AUTH-type events allow blocking operations before they execute
- **No virtualisation overhead**: Unlike containers or sandboxes, the agent runs natively with full project access
- **Process tree tracking**: Monitor not just the spawned command but all its child processes
- **Real-time enforcement**: Policy decisions happen in microseconds, not milliseconds

### Why Now

Apple has approved caro for the Endpoint Security framework entitlement. Combined with our existing eBPF research on Linux, we can now build a unified kernel-level monitoring layer across both major development platforms.

## Decision

We will implement a cross-platform kernel-level monitoring system with:

1. **A `MonitorBackend` trait** abstracting over platform-specific kernel interfaces
2. **Apple Endpoint Security backend** for macOS using `endpoint-security-sys` FFI bindings
3. **eBPF backend** for Linux using the `aya` crate
4. **A shared `PolicyEngine`** that reuses caro's existing safety patterns at the kernel level
5. **A separate `caro-monitor` daemon** that runs the kernel client and communicates with the caro CLI via Unix domain sockets
6. **Feature-gated compilation** behind `monitor`, `monitor-es`, and `monitor-ebpf` feature flags

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    caro CLI process                       │
│  NL Input → LLM → Safety Validation → User Confirm      │
│                         │                                │
│                    IPC Client ◄──── MonitorClient         │
└─────────────┬───────────────────────────────────────────┘
              │ Unix Domain Socket (length-prefixed JSON)
              ▼
┌─────────────────────────────────────────────────────────┐
│               caro-monitor daemon                        │
│  ┌───────────────────────────────────────────────────┐  │
│  │              PolicyEngine (shared)                 │  │
│  │  SecurityPolicy rules derived from safety patterns │  │
│  └───────────┬───────────────────────┬───────────────┘  │
│              │                       │                   │
│  ┌───────────▼──────┐   ┌───────────▼──────────────┐   │
│  │  ES Backend      │   │  eBPF Backend             │   │
│  │  (macOS)         │   │  (Linux)                  │   │
│  │  AUTH + NOTIFY   │   │  tracepoint + LSM BPF     │   │
│  └──────────────────┘   └──────────────────────────-┘   │
└─────────────────────────────────────────────────────────┘
```

### IPC Protocol

The caro CLI and caro-monitor daemon communicate over a Unix domain socket at `/tmp/caro-monitor.sock`. Messages use a 4-byte big-endian length prefix followed by a JSON payload.

Request types:
- `WatchProcess { pid, command }` — Start monitoring a process tree
- `UnwatchProcess { pid }` — Stop monitoring
- `PreflightCheck { command, working_dir }` — Ask if a command would be allowed
- `UpdatePolicy(SecurityPolicy)` — Hot-reload security policy
- `Status` — Query daemon health
- `Shutdown` — Graceful shutdown

### Execution Pipeline Integration

The `MonitorClient` integrates into the existing execution pipeline as an optional defense-in-depth layer:

```
Safety Validation → [Monitor Preflight] → User Confirm → Execute → [Monitor Watch]
```

If the daemon is not running, execution proceeds normally (graceful degradation).

## Rationale

### Why a Separate Daemon?

1. **Privilege separation**: The ES/eBPF client requires elevated privileges; the caro CLI does not
2. **Lifecycle independence**: The daemon persists across CLI invocations
3. **Process tree visibility**: The daemon can monitor child processes spawned by commands
4. **Resource efficiency**: One daemon serves multiple concurrent caro sessions

### Why Not Containers/Sandboxes?

- Containers add 200-500MB overhead and break native filesystem access
- Sandbox profiles (macOS sandbox-exec) are deprecated and coarse-grained
- Bubblewrap (ADR-010) provides process isolation but not syscall visibility
- Kernel-level monitoring provides the same (or better) protection with zero overhead

### Why Reuse Safety Patterns?

The `PolicyEngine::from_safety_config()` conversion ensures consistency: the same patterns that block `rm -rf /` at the CLI level also block it at the kernel level if pattern matching is somehow bypassed. This avoids policy drift between layers.

### Platform Considerations

| Capability | macOS ES | Linux eBPF |
|-----------|----------|------------|
| Block operations | AUTH events (yes) | LSM BPF hooks (kernel 5.7+) |
| Observe operations | NOTIFY events (yes) | tracepoints/kprobes (yes) |
| Process tree tracking | Yes (via audit token) | Yes (via pid/ppid) |
| No root required | Requires entitlement | Requires CAP_BPF |
| Overhead | ~10μs per event | ~5μs per event |

## Consequences

### Positive

- **Defense-in-depth**: Pattern evasion no longer bypasses all safety — the kernel catches it
- **Process tree visibility**: Child processes of AI agents are monitored, not just the initial command
- **Zero overhead for normal operation**: If daemon is not running, no performance impact
- **Native execution**: No containers, no restricted filesystem, full project access
- **Audit trail**: Every syscall from monitored processes is logged

### Negative

- **Platform-specific code**: ES and eBPF require separate implementations behind feature flags
- **Elevated privileges**: The daemon requires entitlements (macOS) or capabilities (Linux)
- **Additional binary**: Users must run `caro-monitor start` separately (or via launchd/systemd)
- **Apple entitlement dependency**: ES requires Apple's approval for distribution

### Risks

- **ES API stability**: Apple may change the ES framework between macOS versions
- **eBPF kernel version requirements**: LSM BPF hooks require kernel 5.7+; older kernels get observe-only
- **False positives**: Overly aggressive kernel policies could break legitimate workflows
- **IPC latency**: Preflight checks add ~1-2ms per command execution

## Alternatives Considered

1. **Container-based isolation** — Rejected: too much overhead, breaks native workflows
2. **macOS sandbox-exec** — Rejected: deprecated, Seatbelt profiles are undocumented
3. **seccomp-bpf only** — Rejected: Linux-only, no macOS equivalent
4. **ptrace-based monitoring** — Rejected: high overhead (~100x slower), platform-specific
5. **DTrace** — Rejected: observe-only on macOS (no blocking), requires SIP disabled

## Implementation Plan

| Phase | Description | Timeframe |
|-------|-------------|-----------|
| 1 | Core types, policy engine, IPC protocol | Week 1 |
| 2 | MonitorBackend trait, MonitorClient | Week 1 |
| 3 | Apple ES backend (macOS) | Week 2 |
| 4 | eBPF backend + BPF programs (Linux) | Week 2-3 |
| 5 | Daemon binary, execution integration | Week 3 |
| 6 | Testing, documentation, CI | Week 4 |
