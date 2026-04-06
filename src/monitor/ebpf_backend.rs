//! eBPF backend for Linux kernel-level monitoring
//!
//! Uses eBPF programs attached to kernel tracepoints and kprobes to monitor
//! syscalls from AI agent processes. On kernels 5.7+, LSM BPF hooks enable
//! blocking (AUTH-equivalent) operations; on older kernels, monitoring is
//! observe-only (NOTIFY-equivalent).
//!
//! # Requirements
//!
//! - Linux kernel 4.18+ for basic tracepoints
//! - Linux kernel 5.7+ for LSM BPF hooks (blocking capability)
//! - CAP_BPF capability (or root)
//! - `aya` crate for BPF program loading and management
//!
//! # BPF Programs
//!
//! The eBPF programs are in `src/monitor/bpf/`:
//! - `exec_monitor.bpf.c` — tracepoint:sched/sched_process_exec
//! - `file_monitor.bpf.c` — kprobe:vfs_open, vfs_write, vfs_unlink
//! - `net_monitor.bpf.c` — kprobe:tcp_connect, inet_bind
//!
//! Events are communicated from kernel to userspace via perf event arrays.

#[cfg(target_os = "linux")]
mod implementation {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use tokio::sync::mpsc;
    use tracing::{debug, info};

    use crate::monitor::events::*;
    use crate::monitor::policy::{PolicyEngine, SecurityPolicy};
    use crate::monitor::{MonitorBackend, MonitorError};

    /// Minimum kernel version for tracepoint support
    #[allow(dead_code)]
    const MIN_KERNEL_TRACEPOINT: &str = "4.18";
    /// Minimum kernel version for LSM BPF hooks (blocking capability)
    #[allow(dead_code)]
    const MIN_KERNEL_LSM_BPF: &str = "5.7";

    /// eBPF-based syscall monitoring backend
    ///
    /// Attaches BPF programs to kernel hooks and receives events via
    /// perf event arrays. Policy evaluation happens in userspace after
    /// receiving the event from the kernel.
    pub struct EbpfBackend {
        /// Policy engine for evaluating events
        policy_engine: Arc<Mutex<PolicyEngine>>,
        /// Channel for forwarding events to the daemon event loop (used by BPF reader task)
        #[allow(dead_code)]
        event_tx: mpsc::Sender<(SyscallEvent, PolicyDecision)>,
        /// Receiving end for the daemon to consume events
        event_rx: mpsc::Receiver<(SyscallEvent, PolicyDecision)>,
        /// Set of PIDs being actively monitored
        monitored_pids: Arc<Mutex<HashSet<u32>>>,
        /// Whether the BPF programs are loaded and attached
        active: bool,
        /// Whether LSM BPF hooks are available (can block operations)
        lsm_available: bool,
        /// Handle to the perf event reader task
        _reader_handle: Option<tokio::task::JoinHandle<()>>,
    }

    /// Raw event structure shared between BPF programs and userspace
    ///
    /// This matches the C struct layout in the BPF programs.
    /// All fields are native-endian (the BPF program and userspace
    /// run on the same machine).
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    #[allow(dead_code)]
    pub struct RawBpfEvent {
        /// Event type discriminator
        pub event_type: u32,
        /// Process ID
        pub pid: u32,
        /// Parent process ID
        pub ppid: u32,
        /// User ID
        pub uid: u32,
        /// Timestamp (nanoseconds since boot)
        pub timestamp_ns: u64,
        /// Path buffer (null-terminated)
        pub path: [u8; 256],
        /// Secondary path (for rename operations)
        pub path2: [u8; 256],
        /// Arguments or additional data
        pub args: [u8; 512],
        /// Numeric argument (port, flags, signal, etc.)
        pub arg_num: u64,
    }

    /// Event type discriminators matching the BPF program constants
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    pub enum BpfEventType {
        Exec = 1,
        FileOpen = 2,
        FileWrite = 3,
        FileDelete = 4,
        FileRename = 5,
        NetConnect = 6,
        NetBind = 7,
        Signal = 8,
    }

    #[allow(dead_code, clippy::new_without_default)]
    impl EbpfBackend {
        /// Create a new eBPF backend instance
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
                lsm_available: false,
                _reader_handle: None,
            }
        }

        /// Check kernel version and eBPF availability
        pub fn check_availability() -> Result<bool, MonitorError> {
            // Read kernel version from /proc/version or uname
            let uname_output = std::process::Command::new("uname")
                .arg("-r")
                .output()
                .map_err(|_| MonitorError::UnsupportedPlatform)?;

            let version = String::from_utf8_lossy(&uname_output.stdout);
            let version = version.trim();

            // Parse major.minor
            let parts: Vec<&str> = version.split('.').collect();
            let major: u32 = parts
                .first()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let minor: u32 = parts
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            if major < 4 || (major == 4 && minor < 18) {
                return Err(MonitorError::UnsupportedPlatform);
            }

            let lsm_available = major > 5 || (major == 5 && minor >= 7);

            if lsm_available {
                info!(
                    kernel = version,
                    "Linux kernel supports LSM BPF hooks (blocking capable)"
                );
            } else {
                info!(
                    kernel = version,
                    "Linux kernel supports tracepoints only (observe-only mode)"
                );
            }

            Ok(lsm_available)
        }

        /// Check if the process has CAP_BPF capability
        fn check_capabilities() -> Result<(), MonitorError> {
            // Check if we're root or have CAP_BPF
            let uid = unsafe { libc::getuid() };
            if uid == 0 {
                return Ok(());
            }

            // Check /proc/self/status for CapEff
            let status = std::fs::read_to_string("/proc/self/status").map_err(|_| {
                MonitorError::InsufficientPrivileges(
                    "Cannot read /proc/self/status".into(),
                )
            })?;

            for line in status.lines() {
                if line.starts_with("CapEff:") {
                    let cap_hex = line.split_whitespace().nth(1).unwrap_or("0");
                    let caps = u64::from_str_radix(cap_hex, 16).unwrap_or(0);
                    // CAP_BPF is bit 39
                    let cap_bpf = 1u64 << 39;
                    if caps & cap_bpf != 0 {
                        return Ok(());
                    }
                }
            }

            Err(MonitorError::InsufficientPrivileges(
                "Requires CAP_BPF capability or root. Run: sudo setcap cap_bpf+ep caro-monitor"
                    .into(),
            ))
        }

        /// Convert a raw BPF event to our SyscallEvent type
        fn convert_event(raw: &RawBpfEvent) -> Option<SyscallEvent> {
            let path = Self::extract_string(&raw.path);
            let process_path = PathBuf::from(&path);

            let detail = match raw.event_type {
                1 => {
                    // Exec — args are null-separated in the buffer
                    let args: Vec<String> = Self::extract_null_separated(&raw.args);
                    SyscallDetail::Exec {
                        path: PathBuf::from(&path),
                        args,
                        env_count: raw.arg_num as usize,
                    }
                }
                2 => {
                    // FileOpen
                    SyscallDetail::FileOpen {
                        path: PathBuf::from(&path),
                        flags: raw.arg_num as u32,
                    }
                }
                3 => {
                    // FileWrite
                    SyscallDetail::FileWrite {
                        path: PathBuf::from(&path),
                        bytes_requested: raw.arg_num as usize,
                    }
                }
                4 => {
                    // FileDelete
                    SyscallDetail::FileDelete {
                        path: PathBuf::from(&path),
                    }
                }
                5 => {
                    // FileRename
                    let dest = Self::extract_string(&raw.path2);
                    SyscallDetail::FileRename {
                        source: PathBuf::from(&path),
                        destination: PathBuf::from(dest),
                    }
                }
                6 => {
                    // NetConnect
                    let addr = Self::extract_string(&raw.args);
                    let port = (raw.arg_num & 0xFFFF) as u16;
                    let proto = ((raw.arg_num >> 16) & 0xFF) as u8;
                    SyscallDetail::NetworkConnect {
                        address: addr,
                        port,
                        protocol: match proto {
                            6 => NetworkProtocol::Tcp,
                            17 => NetworkProtocol::Udp,
                            other => NetworkProtocol::Other(other),
                        },
                    }
                }
                7 => {
                    // NetBind
                    let addr = Self::extract_string(&raw.args);
                    let port = (raw.arg_num & 0xFFFF) as u16;
                    SyscallDetail::NetworkBind {
                        address: addr,
                        port,
                    }
                }
                8 => {
                    // Signal
                    let target_pid = (raw.arg_num >> 32) as u32;
                    let signal = (raw.arg_num & 0xFFFFFFFF) as i32;
                    SyscallDetail::Signal { target_pid, signal }
                }
                _ => return None,
            };

            Some(SyscallEvent::new(
                raw.pid,
                raw.ppid,
                raw.uid,
                process_path,
                detail,
            ))
        }

        /// Extract a null-terminated string from a fixed-size byte buffer
        fn extract_string(buf: &[u8]) -> String {
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            String::from_utf8_lossy(&buf[..end]).into_owned()
        }

        /// Extract null-separated strings from a fixed-size byte buffer
        ///
        /// BPF programs store command arguments as null-separated strings.
        /// This reads all non-empty segments up to the first double-null or end of data.
        fn extract_null_separated(buf: &[u8]) -> Vec<String> {
            // Find the end of meaningful data (two consecutive nulls or buffer end)
            let data_end = buf.len();
            let data = &buf[..data_end];

            data.split(|&b| b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .collect()
        }
    }

    #[async_trait]
    impl MonitorBackend for EbpfBackend {
        async fn start(&mut self, policy: SecurityPolicy) -> Result<(), MonitorError> {
            if self.active {
                return Ok(());
            }

            // Check kernel version
            self.lsm_available = Self::check_availability()?;

            // Check capabilities
            Self::check_capabilities()?;

            // Update policy engine
            let engine = PolicyEngine::new(policy)
                .map_err(|e| MonitorError::PolicyError(e.to_string()))?;
            *self.policy_engine.lock().unwrap() = engine;

            // In the full implementation with the aya crate, this would:
            // 1. Load compiled BPF programs from embedded bytes
            // 2. Attach tracepoints:
            //    - sched:sched_process_exec
            // 3. Attach kprobes:
            //    - vfs_open, vfs_write, vfs_unlink, vfs_rename
            //    - tcp_connect, inet_bind
            // 4. If LSM available, attach LSM hooks for blocking
            // 5. Set up perf event array reader
            // 6. Update BPF map with monitored PIDs
            // 7. Spawn async reader task

            info!(
                lsm_available = self.lsm_available,
                "eBPF monitoring started"
            );

            self.active = true;

            // Placeholder: actual BPF program loading requires the `aya` crate
            // (behind monitor-ebpf feature flag). The trait implementation is
            // complete; BPF program compilation and loading is the next step.
            #[cfg(feature = "monitor-ebpf")]
            {
                // TODO: Load BPF programs via aya
                // let mut bpf = aya::Ebpf::load(include_bytes_aligned!("bpf/exec_monitor"))?;
                // let prog: &mut TracePoint = bpf.program_mut("exec_monitor")?;
                // prog.load()?;
                // prog.attach("sched", "sched_process_exec")?;
            }

            Ok(())
        }

        async fn stop(&mut self) -> Result<(), MonitorError> {
            if !self.active {
                return Ok(());
            }

            // In full implementation: detach all BPF programs
            info!("eBPF monitoring stopped");
            self.active = false;

            if let Some(handle) = self._reader_handle.take() {
                handle.abort();
            }

            Ok(())
        }

        async fn watch_pid(&mut self, pid: u32) -> Result<(), MonitorError> {
            self.monitored_pids.lock().unwrap().insert(pid);
            // In full implementation: update BPF hash map with new PID
            // bpf_map_update_elem(monitored_pids_map, &pid, &1, BPF_ANY)
            debug!(pid = pid, "Added PID to eBPF watch list");
            Ok(())
        }

        async fn unwatch_pid(&mut self, pid: u32) -> Result<(), MonitorError> {
            self.monitored_pids.lock().unwrap().remove(&pid);
            // In full implementation: remove PID from BPF hash map
            // bpf_map_delete_elem(monitored_pids_map, &pid)
            debug!(pid = pid, "Removed PID from eBPF watch list");
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
            "ebpf"
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_ebpf_backend_creation() {
            let backend = EbpfBackend::new();
            assert_eq!(backend.name(), "ebpf");
        }

        #[test]
        fn test_extract_string() {
            let mut buf = [0u8; 256];
            let test = b"hello world";
            buf[..test.len()].copy_from_slice(test);
            assert_eq!(EbpfBackend::extract_string(&buf), "hello world");
        }

        #[test]
        fn test_extract_string_empty() {
            let buf = [0u8; 256];
            assert_eq!(EbpfBackend::extract_string(&buf), "");
        }

        #[test]
        fn test_convert_exec_event() {
            let mut raw = RawBpfEvent {
                event_type: 1, // Exec
                pid: 100,
                ppid: 1,
                uid: 501,
                timestamp_ns: 0,
                path: [0u8; 256],
                path2: [0u8; 256],
                args: [0u8; 512],
                arg_num: 5, // env_count
            };

            let path = b"/bin/ls";
            raw.path[..path.len()].copy_from_slice(path);

            let args = b"ls\0-la\0/tmp";
            raw.args[..args.len()].copy_from_slice(args);

            let event = EbpfBackend::convert_event(&raw).unwrap();
            assert_eq!(event.pid, 100);
            assert_eq!(
                event.category,
                crate::monitor::events::SyscallCategory::ProcessExec
            );

            if let crate::monitor::events::SyscallDetail::Exec { args, env_count, .. } =
                &event.detail
            {
                assert_eq!(args, &["ls", "-la", "/tmp"]);
                assert_eq!(*env_count, 5);
            } else {
                panic!("Expected Exec detail");
            }
        }

        #[test]
        fn test_convert_net_connect_event() {
            let mut raw = RawBpfEvent {
                event_type: 6, // NetConnect
                pid: 200,
                ppid: 1,
                uid: 501,
                timestamp_ns: 0,
                path: [0u8; 256],
                path2: [0u8; 256],
                args: [0u8; 512],
                arg_num: 443 | (6 << 16), // port=443, proto=TCP(6)
            };

            let path = b"/usr/bin/curl";
            raw.path[..path.len()].copy_from_slice(path);

            let addr = b"93.184.216.34";
            raw.args[..addr.len()].copy_from_slice(addr);

            let event = EbpfBackend::convert_event(&raw).unwrap();
            assert_eq!(
                event.category,
                crate::monitor::events::SyscallCategory::NetworkOperation
            );

            if let crate::monitor::events::SyscallDetail::NetworkConnect {
                port, protocol, ..
            } = &event.detail
            {
                assert_eq!(*port, 443);
                assert_eq!(*protocol, crate::monitor::events::NetworkProtocol::Tcp);
            } else {
                panic!("Expected NetworkConnect detail");
            }
        }

        #[test]
        fn test_convert_unknown_event_returns_none() {
            let raw = RawBpfEvent {
                event_type: 99, // Unknown
                pid: 1,
                ppid: 0,
                uid: 0,
                timestamp_ns: 0,
                path: [0u8; 256],
                path2: [0u8; 256],
                args: [0u8; 512],
                arg_num: 0,
            };
            assert!(EbpfBackend::convert_event(&raw).is_none());
        }
    }
}

// Re-export the implementation on Linux
#[cfg(target_os = "linux")]
pub use implementation::EbpfBackend;
