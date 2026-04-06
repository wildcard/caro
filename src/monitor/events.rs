//! Syscall event types for kernel-level monitoring
//!
//! Defines the event schema shared across all monitor backends (Apple ES, eBPF).
//! Events represent intercepted syscall operations with enough detail for
//! policy evaluation and audit logging.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::models::RiskLevel;

/// Categories of syscall operations the monitor intercepts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SyscallCategory {
    /// Process execution (execve, posix_spawn)
    ProcessExec,
    /// File open/create/write/delete/rename
    FileOperation,
    /// Network connect/bind/listen
    NetworkOperation,
    /// Signal delivery (kill, etc.)
    SignalOperation,
}

impl std::fmt::Display for SyscallCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessExec => write!(f, "process_exec"),
            Self::FileOperation => write!(f, "file_operation"),
            Self::NetworkOperation => write!(f, "network_operation"),
            Self::SignalOperation => write!(f, "signal_operation"),
        }
    }
}

/// A syscall event captured by the kernel-level monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallEvent {
    /// Unique event identifier
    pub id: uuid::Uuid,
    /// When the event was captured
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Process ID that triggered the syscall
    pub pid: u32,
    /// Parent process ID
    pub ppid: u32,
    /// User ID of the process owner
    pub uid: u32,
    /// Path to the executable that triggered the event
    pub process_path: PathBuf,
    /// High-level category for filtering
    pub category: SyscallCategory,
    /// Specific syscall details
    pub detail: SyscallDetail,
}

impl SyscallEvent {
    /// Create a new event with auto-generated ID and timestamp
    pub fn new(pid: u32, ppid: u32, uid: u32, process_path: PathBuf, detail: SyscallDetail) -> Self {
        let category = match &detail {
            SyscallDetail::Exec { .. } => SyscallCategory::ProcessExec,
            SyscallDetail::FileOpen { .. }
            | SyscallDetail::FileWrite { .. }
            | SyscallDetail::FileDelete { .. }
            | SyscallDetail::FileRename { .. } => SyscallCategory::FileOperation,
            SyscallDetail::NetworkConnect { .. } | SyscallDetail::NetworkBind { .. } => {
                SyscallCategory::NetworkOperation
            }
            SyscallDetail::Signal { .. } => SyscallCategory::SignalOperation,
        };

        Self {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            pid,
            ppid,
            uid,
            process_path,
            category,
            detail,
        }
    }
}

/// Specific syscall details — the payload varies by operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyscallDetail {
    /// Process execution (execve, posix_spawn)
    Exec {
        /// Path to the executable being launched
        path: PathBuf,
        /// Command-line arguments
        args: Vec<String>,
        /// Number of environment variables (not values, for privacy)
        env_count: usize,
    },
    /// File open operation
    FileOpen {
        /// Path being opened
        path: PathBuf,
        /// Open flags (O_RDONLY=0, O_WRONLY=1, O_RDWR=2, etc.)
        flags: u32,
    },
    /// File write operation
    FileWrite {
        /// Path being written to
        path: PathBuf,
        /// Number of bytes requested
        bytes_requested: usize,
    },
    /// File deletion (unlink)
    FileDelete {
        /// Path being deleted
        path: PathBuf,
    },
    /// File rename/move
    FileRename {
        /// Original path
        source: PathBuf,
        /// New path
        destination: PathBuf,
    },
    /// Network connect
    NetworkConnect {
        /// Remote address (IP or hostname)
        address: String,
        /// Remote port
        port: u16,
        /// Protocol
        protocol: NetworkProtocol,
    },
    /// Network bind (listening)
    NetworkBind {
        /// Bind address
        address: String,
        /// Bind port
        port: u16,
    },
    /// Signal delivery
    Signal {
        /// Target process ID
        target_pid: u32,
        /// Signal number (e.g., 9 for SIGKILL)
        signal: i32,
    },
}

/// Network protocol for connect/bind events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkProtocol {
    Tcp,
    Udp,
    Other(u8),
}

impl std::fmt::Display for NetworkProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tcp => write!(f, "TCP"),
            Self::Udp => write!(f, "UDP"),
            Self::Other(n) => write!(f, "proto({})", n),
        }
    }
}

/// Result of policy evaluation for a syscall event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// ID of the event this decision applies to
    pub event_id: uuid::Uuid,
    /// What action to take
    pub action: PolicyAction,
    /// Human-readable explanation
    pub reason: String,
    /// Names of policy rules that matched
    pub matched_rules: Vec<String>,
    /// Assessed risk level
    pub risk_level: RiskLevel,
}

/// Actions the policy engine can take on a syscall event
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyAction {
    /// Allow the operation to proceed
    Allow,
    /// Block the operation (AUTH events only — ES can block, eBPF LSM can block)
    Deny,
    /// Allow but log for audit trail
    AuditAllow,
}

impl std::fmt::Display for PolicyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => write!(f, "ALLOW"),
            Self::Deny => write!(f, "DENY"),
            Self::AuditAllow => write!(f, "AUDIT_ALLOW"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_event_creation() {
        let event = SyscallEvent::new(
            1234,
            1000,
            501,
            PathBuf::from("/bin/rm"),
            SyscallDetail::Exec {
                path: PathBuf::from("/bin/rm"),
                args: vec!["rm".into(), "-rf".into(), "/".into()],
                env_count: 10,
            },
        );

        assert_eq!(event.pid, 1234);
        assert_eq!(event.ppid, 1000);
        assert_eq!(event.category, SyscallCategory::ProcessExec);
    }

    #[test]
    fn test_category_from_detail() {
        let exec_event = SyscallEvent::new(
            1, 0, 0, PathBuf::from("/bin/sh"),
            SyscallDetail::Exec { path: "/bin/sh".into(), args: vec![], env_count: 0 },
        );
        assert_eq!(exec_event.category, SyscallCategory::ProcessExec);

        let file_event = SyscallEvent::new(
            1, 0, 0, PathBuf::from("/bin/cat"),
            SyscallDetail::FileOpen { path: "/etc/passwd".into(), flags: 0 },
        );
        assert_eq!(file_event.category, SyscallCategory::FileOperation);

        let net_event = SyscallEvent::new(
            1, 0, 0, PathBuf::from("/usr/bin/curl"),
            SyscallDetail::NetworkConnect {
                address: "10.0.0.1".into(),
                port: 443,
                protocol: NetworkProtocol::Tcp,
            },
        );
        assert_eq!(net_event.category, SyscallCategory::NetworkOperation);

        let sig_event = SyscallEvent::new(
            1, 0, 0, PathBuf::from("/bin/kill"),
            SyscallDetail::Signal { target_pid: 999, signal: 9 },
        );
        assert_eq!(sig_event.category, SyscallCategory::SignalOperation);
    }

    #[test]
    fn test_policy_action_display() {
        assert_eq!(PolicyAction::Allow.to_string(), "ALLOW");
        assert_eq!(PolicyAction::Deny.to_string(), "DENY");
        assert_eq!(PolicyAction::AuditAllow.to_string(), "AUDIT_ALLOW");
    }

    #[test]
    fn test_event_serialization() {
        let event = SyscallEvent::new(
            42, 1, 501, PathBuf::from("/bin/ls"),
            SyscallDetail::FileOpen { path: "/tmp/test".into(), flags: 2 },
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: SyscallEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pid, 42);
        assert_eq!(deserialized.category, SyscallCategory::FileOperation);
    }

    #[test]
    fn test_policy_decision_serialization() {
        let decision = PolicyDecision {
            event_id: uuid::Uuid::new_v4(),
            action: PolicyAction::Deny,
            reason: "Blocked rm -rf /".into(),
            matched_rules: vec!["recursive_delete".into()],
            risk_level: RiskLevel::Critical,
        };
        let json = serde_json::to_string(&decision).unwrap();
        let deserialized: PolicyDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.action, PolicyAction::Deny);
        assert_eq!(deserialized.risk_level, RiskLevel::Critical);
    }
}
