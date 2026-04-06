//! Monitor client for caro CLI
//!
//! Provides the CLI-side interface to communicate with the caro-monitor daemon.
//! Used for preflight checks before command execution and process registration
//! after spawning.

use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::net::UnixStream;
use tracing::{debug, trace, warn};

use super::ipc::{self, MonitorRequest, MonitorResponse};
use super::{DaemonStatus, MonitorError};

/// Client for the caro CLI to communicate with the caro-monitor daemon
///
/// The client connects to the daemon's Unix domain socket to perform
/// preflight policy checks and register processes for monitoring.
///
/// If the daemon is not running, all operations return gracefully
/// (the monitor is an optional defense-in-depth layer, not a hard dependency).
pub struct MonitorClient {
    socket_path: PathBuf,
    connect_timeout: Duration,
}

impl MonitorClient {
    /// Create a new monitor client with default socket path
    pub fn new() -> Self {
        Self {
            socket_path: ipc::socket_path(),
            connect_timeout: Duration::from_millis(500),
        }
    }

    /// Create a client connecting to a custom socket path
    pub fn with_socket_path(path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: path.into(),
            connect_timeout: Duration::from_millis(500),
        }
    }

    /// Set the connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Check if the daemon is likely running (socket file exists)
    pub fn is_daemon_available(&self) -> bool {
        self.socket_path.exists()
    }

    /// Connect to the daemon and send a request
    async fn send_request(
        &self,
        request: &MonitorRequest,
    ) -> Result<MonitorResponse, MonitorError> {
        if !self.is_daemon_available() {
            return Err(MonitorError::DaemonNotRunning);
        }

        let mut stream = tokio::time::timeout(
            self.connect_timeout,
            UnixStream::connect(&self.socket_path),
        )
        .await
        .map_err(|_| {
            MonitorError::IpcError(format!(
                "Connection timeout after {}ms",
                self.connect_timeout.as_millis()
            ))
        })?
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::ConnectionRefused {
                MonitorError::DaemonNotRunning
            } else {
                MonitorError::IpcError(format!("Failed to connect: {}", e))
            }
        })?;

        ipc::send_message(&mut stream, request).await?;
        ipc::recv_message(&mut stream).await
    }

    /// Pre-execution policy check: asks the daemon if a command would be allowed
    ///
    /// Returns `Ok(MonitorResponse::Ok)` if the command passes policy.
    /// Returns `Ok(MonitorResponse::Denied { .. })` if the command would be blocked.
    /// Returns `Err(MonitorError::DaemonNotRunning)` if daemon is not available.
    pub async fn preflight_check(
        &self,
        command: &str,
        working_dir: &Path,
    ) -> Result<MonitorResponse, MonitorError> {
        debug!(command = command, "Performing monitor preflight check");

        let response = self
            .send_request(&MonitorRequest::PreflightCheck {
                command: command.to_string(),
                working_dir: working_dir.to_path_buf(),
            })
            .await?;

        match &response {
            MonitorResponse::Denied { reason, risk_level } => {
                warn!(
                    reason = reason.as_str(),
                    risk_level = ?risk_level,
                    "Monitor daemon denied command"
                );
            }
            MonitorResponse::Ok => {
                trace!("Monitor daemon approved command");
            }
            _ => {}
        }

        Ok(response)
    }

    /// Register a spawned process for kernel-level monitoring
    ///
    /// Call this after spawning a child process to have the daemon
    /// monitor all syscalls from that process and its children.
    pub async fn watch_process(
        &self,
        pid: u32,
        command: &str,
    ) -> Result<MonitorResponse, MonitorError> {
        debug!(pid = pid, command = command, "Registering process for monitoring");

        self.send_request(&MonitorRequest::WatchProcess {
            pid,
            command: command.to_string(),
        })
        .await
    }

    /// Stop monitoring a process
    pub async fn unwatch_process(&self, pid: u32) -> Result<MonitorResponse, MonitorError> {
        debug!(pid = pid, "Unregistering process from monitoring");
        self.send_request(&MonitorRequest::UnwatchProcess { pid })
            .await
    }

    /// Query the daemon's current status
    pub async fn status(&self) -> Result<DaemonStatus, MonitorError> {
        let response = self.send_request(&MonitorRequest::Status).await?;
        match response {
            MonitorResponse::Status(status) => Ok(status),
            MonitorResponse::Error(e) => Err(MonitorError::IpcError(e)),
            _ => Err(MonitorError::IpcError("Unexpected response type".into())),
        }
    }

    /// Request graceful daemon shutdown
    pub async fn shutdown(&self) -> Result<(), MonitorError> {
        let response = self.send_request(&MonitorRequest::Shutdown).await?;
        match response {
            MonitorResponse::Ok => Ok(()),
            MonitorResponse::Error(e) => Err(MonitorError::IpcError(e)),
            _ => Err(MonitorError::IpcError("Unexpected response type".into())),
        }
    }
}

impl Default for MonitorClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = MonitorClient::new();
        assert_eq!(client.connect_timeout, Duration::from_millis(500));
    }

    #[test]
    fn test_client_custom_socket() {
        let client = MonitorClient::with_socket_path("/custom/socket.sock");
        assert_eq!(client.socket_path, PathBuf::from("/custom/socket.sock"));
    }

    #[test]
    fn test_client_timeout_config() {
        let client = MonitorClient::new().with_timeout(Duration::from_secs(2));
        assert_eq!(client.connect_timeout, Duration::from_secs(2));
    }

    #[test]
    fn test_daemon_not_available() {
        let client = MonitorClient::with_socket_path("/nonexistent/socket.sock");
        assert!(!client.is_daemon_available());
    }

    #[tokio::test]
    async fn test_preflight_without_daemon() {
        let client = MonitorClient::with_socket_path("/nonexistent/socket.sock");
        let result = client
            .preflight_check("ls -la", Path::new("/tmp"))
            .await;
        assert!(matches!(result, Err(MonitorError::DaemonNotRunning)));
    }

    #[tokio::test]
    async fn test_status_without_daemon() {
        let client = MonitorClient::with_socket_path("/nonexistent/socket.sock");
        let result = client.status().await;
        assert!(matches!(result, Err(MonitorError::DaemonNotRunning)));
    }

    #[tokio::test]
    async fn test_watch_without_daemon() {
        let client = MonitorClient::with_socket_path("/nonexistent/socket.sock");
        let result = client.watch_process(1234, "ls -la").await;
        assert!(matches!(result, Err(MonitorError::DaemonNotRunning)));
    }

    #[tokio::test]
    async fn test_client_with_mock_daemon() {
        use tokio::net::UnixListener;

        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        // Spawn a mock daemon that always responds Ok
        let daemon = tokio::spawn({
            let _sock_path = sock_path.clone();
            async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let _req: MonitorRequest = ipc::recv_message(&mut stream).await.unwrap();
                ipc::send_message(&mut stream, &MonitorResponse::Ok)
                    .await
                    .unwrap();
            }
        });

        let client = MonitorClient::with_socket_path(&sock_path);
        let result = client
            .preflight_check("ls -la", Path::new("/tmp"))
            .await
            .unwrap();
        assert!(matches!(result, MonitorResponse::Ok));

        daemon.await.unwrap();
    }
}
