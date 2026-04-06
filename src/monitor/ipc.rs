//! IPC protocol for communication between caro CLI and caro-monitor daemon
//!
//! Uses a Unix domain socket with a simple length-prefixed JSON wire format.
//! All messages are framed as: 4-byte big-endian length + JSON payload.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::models::RiskLevel;

use super::policy::SecurityPolicy;
use super::{DaemonStatus, MonitorError};

/// Default socket path for the monitor daemon
pub const SOCKET_PATH: &str = "/tmp/caro-monitor.sock";

/// Maximum message size (1MB) to prevent DoS
const MAX_MESSAGE_SIZE: u32 = 1_048_576;

/// Request messages from caro CLI to caro-monitor daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorRequest {
    /// Register a process for kernel-level monitoring
    WatchProcess {
        /// Process ID to monitor
        pid: u32,
        /// Command string for audit logging
        command: String,
    },
    /// Stop monitoring a process
    UnwatchProcess {
        /// Process ID to stop monitoring
        pid: u32,
    },
    /// Pre-execution policy check: would this command be allowed?
    PreflightCheck {
        /// Command to check
        command: String,
        /// Working directory for context
        working_dir: PathBuf,
    },
    /// Hot-reload the security policy
    UpdatePolicy(SecurityPolicy),
    /// Query daemon health and statistics
    Status,
    /// Request graceful shutdown
    Shutdown,
}

/// Response messages from caro-monitor daemon to caro CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorResponse {
    /// Operation succeeded
    Ok,
    /// Command was denied by policy
    Denied {
        /// Reason for denial
        reason: String,
        /// Risk level that triggered denial
        risk_level: RiskLevel,
    },
    /// Daemon status information
    Status(DaemonStatus),
    /// Error occurred processing the request
    Error(String),
}

/// Send a length-prefixed JSON message over a Unix socket
pub async fn send_message<T: Serialize>(
    stream: &mut UnixStream,
    msg: &T,
) -> Result<(), MonitorError> {
    let payload = serde_json::to_vec(msg).map_err(|e| {
        MonitorError::SerializationError(format!("Failed to serialize message: {}", e))
    })?;

    let len = payload.len() as u32;
    if len > MAX_MESSAGE_SIZE {
        return Err(MonitorError::IpcError(format!(
            "Message too large: {} bytes (max {})",
            len, MAX_MESSAGE_SIZE
        )));
    }

    stream
        .write_all(&len.to_be_bytes())
        .await
        .map_err(|e| MonitorError::IpcError(format!("Failed to write length: {}", e)))?;

    stream
        .write_all(&payload)
        .await
        .map_err(|e| MonitorError::IpcError(format!("Failed to write payload: {}", e)))?;

    stream
        .flush()
        .await
        .map_err(|e| MonitorError::IpcError(format!("Failed to flush: {}", e)))?;

    Ok(())
}

/// Receive a length-prefixed JSON message from a Unix socket
pub async fn recv_message<T: for<'de> Deserialize<'de>>(
    stream: &mut UnixStream,
) -> Result<T, MonitorError> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| MonitorError::IpcError(format!("Failed to read length: {}", e)))?;

    let len = u32::from_be_bytes(len_buf);
    if len > MAX_MESSAGE_SIZE {
        return Err(MonitorError::IpcError(format!(
            "Message too large: {} bytes (max {})",
            len, MAX_MESSAGE_SIZE
        )));
    }

    let mut payload = vec![0u8; len as usize];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|e| MonitorError::IpcError(format!("Failed to read payload: {}", e)))?;

    serde_json::from_slice(&payload).map_err(|e| {
        MonitorError::SerializationError(format!("Failed to deserialize message: {}", e))
    })
}

/// Get the socket path, respecting environment override
pub fn socket_path() -> PathBuf {
    std::env::var("CARO_MONITOR_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(SOCKET_PATH))
}

/// Check if the daemon socket exists (quick liveness check)
pub fn socket_exists() -> bool {
    socket_path().exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = MonitorRequest::WatchProcess {
            pid: 1234,
            command: "ls -la".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: MonitorRequest = serde_json::from_str(&json).unwrap();
        match deserialized {
            MonitorRequest::WatchProcess { pid, command } => {
                assert_eq!(pid, 1234);
                assert_eq!(command, "ls -la");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let resp = MonitorResponse::Denied {
            reason: "Blocked by policy".into(),
            risk_level: RiskLevel::Critical,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: MonitorResponse = serde_json::from_str(&json).unwrap();
        match deserialized {
            MonitorResponse::Denied { reason, risk_level } => {
                assert_eq!(reason, "Blocked by policy");
                assert_eq!(risk_level, RiskLevel::Critical);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_preflight_check_serialization() {
        let req = MonitorRequest::PreflightCheck {
            command: "rm -rf /tmp/test".into(),
            working_dir: PathBuf::from("/home/user"),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("rm -rf /tmp/test"));
        assert!(json.contains("/home/user"));
    }

    #[test]
    fn test_status_response() {
        let status = DaemonStatus {
            running: true,
            backend: "ebpf".into(),
            monitored_pids: vec![100, 200],
            events_processed: 1000,
            events_blocked: 5,
            uptime_seconds: 7200,
        };
        let resp = MonitorResponse::Status(status);
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: MonitorResponse = serde_json::from_str(&json).unwrap();
        match deserialized {
            MonitorResponse::Status(s) => {
                assert!(s.running);
                assert_eq!(s.backend, "ebpf");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_socket_path_default() {
        // Clear env var to test default
        std::env::remove_var("CARO_MONITOR_SOCKET");
        assert_eq!(socket_path(), PathBuf::from(SOCKET_PATH));
    }

    #[test]
    fn test_socket_path_override() {
        std::env::set_var("CARO_MONITOR_SOCKET", "/custom/path.sock");
        assert_eq!(socket_path(), PathBuf::from("/custom/path.sock"));
        std::env::remove_var("CARO_MONITOR_SOCKET");
    }

    #[tokio::test]
    async fn test_send_recv_roundtrip() {
        use tokio::net::UnixListener;

        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        let sock_path_clone = sock_path.clone();
        let sender = tokio::spawn(async move {
            let mut stream = UnixStream::connect(&sock_path_clone).await.unwrap();
            let req = MonitorRequest::Status;
            send_message(&mut stream, &req).await.unwrap();
        });

        let (mut stream, _) = listener.accept().await.unwrap();
        let received: MonitorRequest = recv_message(&mut stream).await.unwrap();
        match received {
            MonitorRequest::Status => {}
            _ => panic!("Wrong variant"),
        }

        sender.await.unwrap();
    }
}
