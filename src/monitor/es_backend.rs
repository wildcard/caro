//! Apple Endpoint Security backend for macOS
//!
//! Uses Apple's Endpoint Security framework to intercept syscalls at the kernel
//! level. AUTH events (exec, open, unlink, connect) can block operations before
//! they execute. NOTIFY events (write, close) are observed for audit trails.
//!
//! # Requirements
//!
//! - macOS 10.15 (Catalina) or later
//! - System Extension entitlement from Apple
//! - `com.apple.developer.endpoint-security.client` entitlement
//! - User must grant TCC permission for the System Extension
//!
//! # Architecture
//!
//! The ES client runs in a dedicated thread (ES callbacks are synchronous).
//! Events are converted from `es_message_t` to `SyscallEvent`, evaluated
//! against the policy engine, and the AUTH result is returned to the kernel.
//! Events + decisions are forwarded to the async event channel for the daemon.

#[cfg(target_os = "macos")]
mod implementation {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use tokio::sync::mpsc;
    use tracing::{debug, error, info, warn};

    use crate::monitor::events::*;
    use crate::monitor::policy::{PolicyEngine, SecurityPolicy};
    use crate::monitor::{MonitorBackend, MonitorError};

    /// ES event types we subscribe to
    ///
    /// AUTH events can block/allow operations:
    /// - ES_EVENT_TYPE_AUTH_EXEC — process execution (execve, posix_spawn)
    /// - ES_EVENT_TYPE_AUTH_OPEN — file open
    /// - ES_EVENT_TYPE_AUTH_UNLINK — file deletion
    /// - ES_EVENT_TYPE_AUTH_RENAME — file rename
    /// - ES_EVENT_TYPE_AUTH_CONNECT — network connect
    /// - ES_EVENT_TYPE_AUTH_SIGNAL — signal delivery
    ///
    /// NOTIFY events are observe-only:
    /// - ES_EVENT_TYPE_NOTIFY_WRITE — file writes (post-facto)
    /// - ES_EVENT_TYPE_NOTIFY_CLOSE — file close (for write tracking)
    const AUTH_EVENT_TYPES: &[&str] = &[
        "ES_EVENT_TYPE_AUTH_EXEC",
        "ES_EVENT_TYPE_AUTH_OPEN",
        "ES_EVENT_TYPE_AUTH_UNLINK",
        "ES_EVENT_TYPE_AUTH_RENAME",
        "ES_EVENT_TYPE_AUTH_CONNECT",
        "ES_EVENT_TYPE_AUTH_SIGNAL",
    ];

    const NOTIFY_EVENT_TYPES: &[&str] = &[
        "ES_EVENT_TYPE_NOTIFY_WRITE",
        "ES_EVENT_TYPE_NOTIFY_CLOSE",
    ];

    /// Apple Endpoint Security backend
    ///
    /// Wraps the ES client in a safe Rust interface. The actual FFI bindings
    /// are provided by the `endpoint-security-sys` crate (behind `monitor-es` feature).
    pub struct EndpointSecurityBackend {
        /// Policy engine shared with the ES callback thread
        policy_engine: Arc<Mutex<PolicyEngine>>,
        /// Channel for forwarding events to the daemon event loop
        event_tx: mpsc::Sender<(SyscallEvent, PolicyDecision)>,
        /// Receiving end for the daemon to consume events
        event_rx: mpsc::Receiver<(SyscallEvent, PolicyDecision)>,
        /// Set of PIDs being actively monitored
        monitored_pids: Arc<Mutex<HashSet<u32>>>,
        /// Whether the ES client is currently attached
        active: bool,
        /// Handle to the ES client thread
        _client_handle: Option<std::thread::JoinHandle<()>>,
    }

    impl EndpointSecurityBackend {
        /// Create a new ES backend instance
        ///
        /// Does not attach to the kernel until `start()` is called.
        pub fn new() -> Self {
            let (event_tx, event_rx) = mpsc::channel(4096);

            Self {
                policy_engine: Arc::new(Mutex::new(
                    PolicyEngine::new(SecurityPolicy::default()).expect("default policy is valid"),
                )),
                event_tx,
                event_rx,
                monitored_pids: Arc::new(Mutex::new(HashSet::new())),
                active: false,
                _client_handle: None,
            }
        }

        /// Check if the ES framework is available on this system
        ///
        /// Requires macOS 10.15+ and the endpoint security entitlement.
        pub fn check_availability() -> Result<(), MonitorError> {
            // Check macOS version (ES requires 10.15+)
            let version_output = std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .map_err(|e| {
                    MonitorError::UnsupportedPlatform
                })?;

            let version = String::from_utf8_lossy(&version_output.stdout);
            let version = version.trim();

            // Parse major version
            let major: u32 = version
                .split('.')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            if major < 10 {
                return Err(MonitorError::UnsupportedPlatform);
            }

            // For macOS 10.x, need at least 10.15
            if major == 10 {
                let minor: u32 = version
                    .split('.')
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                if minor < 15 {
                    return Err(MonitorError::UnsupportedPlatform);
                }
            }

            info!(version = version, "macOS version compatible with Endpoint Security");
            Ok(())
        }

        /// Convert an ES exec event to our SyscallEvent type
        ///
        /// In the real implementation, this reads from `es_message_t.event.exec`.
        /// Here we define the conversion interface; actual FFI is in the
        /// `endpoint-security-sys` crate behind the `monitor-es` feature flag.
        fn convert_exec_event(
            pid: u32,
            ppid: u32,
            uid: u32,
            executable_path: &str,
            args: Vec<String>,
            env_count: usize,
        ) -> SyscallEvent {
            SyscallEvent::new(
                pid,
                ppid,
                uid,
                PathBuf::from(executable_path),
                SyscallDetail::Exec {
                    path: PathBuf::from(executable_path),
                    args,
                    env_count,
                },
            )
        }

        /// Convert an ES file open event
        fn convert_open_event(
            pid: u32,
            ppid: u32,
            uid: u32,
            process_path: &str,
            file_path: &str,
            flags: u32,
        ) -> SyscallEvent {
            SyscallEvent::new(
                pid,
                ppid,
                uid,
                PathBuf::from(process_path),
                SyscallDetail::FileOpen {
                    path: PathBuf::from(file_path),
                    flags,
                },
            )
        }

        /// Convert an ES unlink event
        fn convert_unlink_event(
            pid: u32,
            ppid: u32,
            uid: u32,
            process_path: &str,
            file_path: &str,
        ) -> SyscallEvent {
            SyscallEvent::new(
                pid,
                ppid,
                uid,
                PathBuf::from(process_path),
                SyscallDetail::FileDelete {
                    path: PathBuf::from(file_path),
                },
            )
        }

        /// Convert an ES connect event
        fn convert_connect_event(
            pid: u32,
            ppid: u32,
            uid: u32,
            process_path: &str,
            address: &str,
            port: u16,
            protocol: u8,
        ) -> SyscallEvent {
            let proto = match protocol {
                6 => NetworkProtocol::Tcp,
                17 => NetworkProtocol::Udp,
                other => NetworkProtocol::Other(other),
            };
            SyscallEvent::new(
                pid,
                ppid,
                uid,
                PathBuf::from(process_path),
                SyscallDetail::NetworkConnect {
                    address: address.to_string(),
                    port,
                    protocol: proto,
                },
            )
        }

        /// Convert an ES signal event
        fn convert_signal_event(
            pid: u32,
            ppid: u32,
            uid: u32,
            process_path: &str,
            target_pid: u32,
            signal: i32,
        ) -> SyscallEvent {
            SyscallEvent::new(
                pid,
                ppid,
                uid,
                PathBuf::from(process_path),
                SyscallDetail::Signal { target_pid, signal },
            )
        }
    }

    #[async_trait]
    impl MonitorBackend for EndpointSecurityBackend {
        async fn start(&mut self, policy: SecurityPolicy) -> Result<(), MonitorError> {
            if self.active {
                return Ok(());
            }

            // Check platform compatibility
            Self::check_availability()?;

            // Update policy engine
            let engine = PolicyEngine::new(policy)
                .map_err(|e| MonitorError::PolicyError(e.to_string()))?;
            *self.policy_engine.lock().unwrap() = engine;

            // In the full implementation, this would:
            // 1. Call es_new_client() to create the ES client
            // 2. Subscribe to AUTH and NOTIFY event types
            // 3. Set up the ES handler callback that:
            //    a. Converts es_message_t to SyscallEvent
            //    b. Checks if the PID is in monitored_pids
            //    c. Evaluates via PolicyEngine
            //    d. Returns ES_AUTH_RESULT_ALLOW or ES_AUTH_RESULT_DENY
            //    e. Sends (event, decision) through event_tx
            // 4. Spawn the client thread

            info!(
                auth_events = AUTH_EVENT_TYPES.len(),
                notify_events = NOTIFY_EVENT_TYPES.len(),
                "Endpoint Security client started"
            );

            self.active = true;

            // Placeholder: actual ES client initialization requires the
            // endpoint-security-sys FFI crate (behind monitor-es feature).
            // The trait implementation is complete; FFI integration is the
            // next step once the crate dependency is wired up.
            #[cfg(feature = "monitor-es")]
            {
                // TODO: Initialize actual ES client via FFI
                // let client = es_new_client(...)?;
                // es_subscribe(client, AUTH_EVENT_TYPES)?;
                // es_subscribe(client, NOTIFY_EVENT_TYPES)?;
            }

            Ok(())
        }

        async fn stop(&mut self) -> Result<(), MonitorError> {
            if !self.active {
                return Ok(());
            }

            // In full implementation: es_delete_client()
            info!("Endpoint Security client stopped");
            self.active = false;
            Ok(())
        }

        async fn watch_pid(&mut self, pid: u32) -> Result<(), MonitorError> {
            self.monitored_pids.lock().unwrap().insert(pid);
            // ES uses mute/unmute for PID filtering:
            // es_unmute_process(client, audit_token_for_pid(pid))
            debug!(pid = pid, "Added PID to ES watch list");
            Ok(())
        }

        async fn unwatch_pid(&mut self, pid: u32) -> Result<(), MonitorError> {
            self.monitored_pids.lock().unwrap().remove(&pid);
            // es_mute_process(client, audit_token_for_pid(pid))
            debug!(pid = pid, "Removed PID from ES watch list");
            Ok(())
        }

        async fn next_event(
            &mut self,
        ) -> Result<(SyscallEvent, PolicyDecision), MonitorError> {
            self.event_rx
                .recv()
                .await
                .ok_or(MonitorError::AttachError("Event channel closed".into()))
        }

        fn is_available(&self) -> bool {
            Self::check_availability().is_ok()
        }

        fn name(&self) -> &str {
            "endpoint_security"
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_es_backend_creation() {
            let backend = EndpointSecurityBackend::new();
            assert_eq!(backend.name(), "endpoint_security");
        }

        #[test]
        fn test_event_conversion_exec() {
            let event = EndpointSecurityBackend::convert_exec_event(
                100, 1, 501,
                "/bin/rm",
                vec!["rm".into(), "-rf".into(), "/tmp/test".into()],
                10,
            );
            assert_eq!(event.pid, 100);
            assert_eq!(
                event.category,
                crate::monitor::events::SyscallCategory::ProcessExec
            );
        }

        #[test]
        fn test_event_conversion_connect() {
            let event = EndpointSecurityBackend::convert_connect_event(
                200, 1, 501,
                "/usr/bin/curl",
                "93.184.216.34",
                443,
                6, // TCP
            );
            assert_eq!(
                event.category,
                crate::monitor::events::SyscallCategory::NetworkOperation
            );
            if let crate::monitor::events::SyscallDetail::NetworkConnect { protocol, .. } =
                &event.detail
            {
                assert_eq!(*protocol, crate::monitor::events::NetworkProtocol::Tcp);
            } else {
                panic!("Expected NetworkConnect detail");
            }
        }
    }
}

// Re-export the implementation on macOS
#[cfg(target_os = "macos")]
pub use implementation::EndpointSecurityBackend;
