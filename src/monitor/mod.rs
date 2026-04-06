//! Kernel-level agent monitoring module
//!
//! Provides real-time syscall interception and policy enforcement for AI agent
//! processes. Uses Apple Endpoint Security on macOS and eBPF on Linux to monitor
//! every operation at the kernel level — no containers, no sandboxes, no overhead.
//!
//! # Architecture
//!
//! - **MonitorBackend trait**: Abstract interface over platform-specific kernel hooks
//! - **PolicyEngine**: Shared policy evaluation (reuses safety patterns)
//! - **IPC protocol**: Unix domain socket communication between CLI and daemon
//! - **MonitorClient**: CLI-side client for preflight checks and process registration
//!
//! # Backends
//!
//! - `es_backend` — Apple Endpoint Security (macOS, requires entitlement)
//! - `ebpf_backend` — eBPF tracepoints + LSM hooks (Linux, requires CAP_BPF)
//!
//! # Feature Flags
//!
//! - `monitor` — Core types, policy engine, IPC protocol
//! - `monitor-es` — Apple Endpoint Security backend
//! - `monitor-ebpf` — eBPF backend with BPF programs

pub mod client;
pub mod events;
pub mod ipc;
pub mod policy;

#[cfg(target_os = "macos")]
pub mod es_backend;

#[cfg(target_os = "linux")]
pub mod ebpf_backend;

use async_trait::async_trait;

use events::{PolicyDecision, SyscallEvent};
use policy::SecurityPolicy;

/// Errors from the monitoring subsystem
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    /// The requested backend is not available on this platform
    #[error("Monitor backend not available on this platform")]
    UnsupportedPlatform,

    /// Insufficient OS-level privileges (entitlement or capability)
    #[error("Insufficient privileges: {0}")]
    InsufficientPrivileges(String),

    /// Failed to attach to the kernel monitoring interface
    #[error("Failed to attach monitor: {0}")]
    AttachError(String),

    /// Policy evaluation encountered an error
    #[error("Policy evaluation failed: {0}")]
    PolicyError(String),

    /// IPC communication error with the daemon
    #[error("IPC error: {0}")]
    IpcError(String),

    /// The caro-monitor daemon is not running
    #[error("Monitor daemon not running")]
    DaemonNotRunning,

    /// IO error during socket or file operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error in IPC protocol
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Abstract backend for kernel-level syscall monitoring
///
/// Implementations intercept syscalls from the operating system's kernel
/// and evaluate them against the shared policy engine. On macOS this uses
/// Apple's Endpoint Security framework; on Linux it uses eBPF.
///
/// AUTH events (exec, open, connect) can be blocked before they execute.
/// NOTIFY events (write, close) are observed for audit purposes.
#[async_trait]
pub trait MonitorBackend: Send + Sync {
    /// Start monitoring with the given security policy
    ///
    /// Attaches to the kernel interface and begins intercepting syscalls.
    /// Returns an error if the backend lacks required privileges.
    async fn start(&mut self, policy: SecurityPolicy) -> Result<(), MonitorError>;

    /// Stop monitoring and detach from the kernel
    ///
    /// Releases all kernel resources. Safe to call multiple times.
    async fn stop(&mut self) -> Result<(), MonitorError>;

    /// Add a process (and its future children) to the watch list
    ///
    /// Only syscalls from watched processes are evaluated against policy.
    /// If no PIDs are watched, all processes are monitored (use with caution).
    async fn watch_pid(&mut self, pid: u32) -> Result<(), MonitorError>;

    /// Remove a process from the watch list
    async fn unwatch_pid(&mut self, pid: u32) -> Result<(), MonitorError>;

    /// Receive the next syscall event with its policy decision
    ///
    /// Blocks until an event is available. For AUTH events, the policy decision
    /// has already been enforced (blocked or allowed) before this returns.
    async fn next_event(&mut self) -> Result<(SyscallEvent, PolicyDecision), MonitorError>;

    /// Check if this backend is available on the current platform
    ///
    /// Returns false if the required kernel interface is not present
    /// or the process lacks necessary privileges.
    fn is_available(&self) -> bool;

    /// Backend name for logging and status display
    fn name(&self) -> &str;
}

/// Status of the monitoring daemon
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DaemonStatus {
    /// Whether the daemon is actively monitoring
    pub running: bool,
    /// Name of the active backend (e.g., "endpoint_security", "ebpf")
    pub backend: String,
    /// PIDs currently being monitored
    pub monitored_pids: Vec<u32>,
    /// Total events processed since daemon start
    pub events_processed: u64,
    /// Total events blocked (denied) since daemon start
    pub events_blocked: u64,
    /// Daemon uptime in seconds
    pub uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_error_display() {
        let err = MonitorError::DaemonNotRunning;
        assert_eq!(err.to_string(), "Monitor daemon not running");

        let err = MonitorError::InsufficientPrivileges("Missing CAP_BPF".into());
        assert_eq!(err.to_string(), "Insufficient privileges: Missing CAP_BPF");
    }

    #[test]
    fn test_daemon_status_serialization() {
        let status = DaemonStatus {
            running: true,
            backend: "endpoint_security".into(),
            monitored_pids: vec![1234, 5678],
            events_processed: 42,
            events_blocked: 3,
            uptime_seconds: 3600,
        };
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: DaemonStatus = serde_json::from_str(&json).unwrap();
        assert!(deserialized.running);
        assert_eq!(deserialized.events_blocked, 3);
    }
}
