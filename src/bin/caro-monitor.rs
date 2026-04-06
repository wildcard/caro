//! caro-monitor — Kernel-level agent supervision daemon
//!
//! Runs as a background service that intercepts syscalls from monitored
//! AI agent processes and enforces security policy at the kernel level.
//!
//! On macOS: Uses Apple Endpoint Security framework (AUTH + NOTIFY events)
//! On Linux: Uses eBPF tracepoints + LSM hooks
//!
//! # Usage
//!
//! ```bash
//! caro-monitor start [--policy <path>] [--socket <path>]
//! caro-monitor stop
//! caro-monitor status
//! ```

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use clap::{Parser, Subcommand};
use tokio::net::UnixListener;
use tracing::{error, info, warn};

use caro::monitor::events::PolicyAction;
use caro::monitor::ipc::{self, MonitorRequest, MonitorResponse};
use caro::monitor::policy::{PolicyEngine, SecurityPolicy};
use caro::monitor::{DaemonStatus, MonitorBackend};

#[derive(Parser)]
#[command(name = "caro-monitor", about = "Kernel-level agent supervision daemon")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the monitoring daemon
    Start {
        /// Path to security policy file (YAML/TOML)
        #[arg(short, long)]
        policy: Option<PathBuf>,

        /// Unix socket path for IPC
        #[arg(short, long)]
        socket: Option<PathBuf>,

        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },
    /// Stop a running daemon
    Stop {
        /// Unix socket path for IPC
        #[arg(short, long)]
        socket: Option<PathBuf>,
    },
    /// Query daemon status
    Status {
        /// Unix socket path for IPC
        #[arg(short, long)]
        socket: Option<PathBuf>,
    },
}

/// Shared daemon state
struct DaemonState {
    start_time: Instant,
    events_processed: AtomicU64,
    events_blocked: AtomicU64,
    shutdown: AtomicBool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("caro_monitor=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            policy,
            socket,
            foreground,
        } => {
            let socket_path = socket.unwrap_or_else(ipc::socket_path);
            start_daemon(policy, socket_path, foreground).await
        }
        Command::Stop { socket } => {
            let socket_path = socket.unwrap_or_else(ipc::socket_path);
            stop_daemon(socket_path).await
        }
        Command::Status { socket } => {
            let socket_path = socket.unwrap_or_else(ipc::socket_path);
            query_status(socket_path).await
        }
    }
}

async fn start_daemon(
    policy_path: Option<PathBuf>,
    socket_path: PathBuf,
    _foreground: bool,
) -> anyhow::Result<()> {
    // Load security policy
    let policy = if let Some(path) = policy_path {
        let content = tokio::fs::read_to_string(&path).await?;
        if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
            serde_yaml::from_str(&content)?
        } else {
            toml::from_str(&content)?
        }
    } else {
        // Default: derive policy from caro's safety patterns
        info!("No policy file specified, using safety-derived defaults");
        SecurityPolicy::default()
    };

    // Initialize the platform-specific backend
    let mut backend = create_backend()?;
    let backend_name = backend.name().to_string();

    info!(backend = backend_name.as_str(), "Starting monitor daemon");

    // Start the backend
    backend.start(policy.clone()).await.map_err(|e| {
        anyhow::anyhow!("Failed to start {} backend: {}", backend_name, e)
    })?;

    // Remove stale socket file
    if socket_path.exists() {
        tokio::fs::remove_file(&socket_path).await?;
    }

    // Create Unix socket listener
    let listener = UnixListener::bind(&socket_path)?;
    info!(socket = %socket_path.display(), "Listening for IPC connections");

    // Set socket permissions (owner-only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
    }

    // Shared state
    let state = Arc::new(DaemonState {
        start_time: Instant::now(),
        events_processed: AtomicU64::new(0),
        events_blocked: AtomicU64::new(0),
        shutdown: AtomicBool::new(false),
    });

    // Create policy engine for IPC handler
    let policy_engine = Arc::new(std::sync::Mutex::new(
        PolicyEngine::new(policy).map_err(|e| anyhow::anyhow!("Policy error: {}", e))?,
    ));

    info!(backend = backend_name.as_str(), "Monitor daemon running");

    // Main event loop: handle IPC requests
    loop {
        if state.shutdown.load(Ordering::Relaxed) {
            info!("Shutdown requested");
            break;
        }

        tokio::select! {
            // Accept IPC connections
            Ok((mut stream, _)) = listener.accept() => {
                let state = Arc::clone(&state);
                let policy_engine = Arc::clone(&policy_engine);
                let backend_name = backend_name.clone();

                tokio::spawn(async move {
                    match ipc::recv_message::<MonitorRequest>(&mut stream).await {
                        Ok(request) => {
                            let response = handle_request(
                                request,
                                &state,
                                &policy_engine,
                                &backend_name,
                            ).await;

                            if let Err(e) = ipc::send_message(&mut stream, &response).await {
                                error!(error = %e, "Failed to send IPC response");
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to receive IPC request");
                        }
                    }
                });
            }
            // Handle Ctrl+C
            _ = tokio::signal::ctrl_c() => {
                info!("Received SIGINT, shutting down");
                break;
            }
        }
    }

    // Cleanup
    backend.stop().await?;
    if socket_path.exists() {
        tokio::fs::remove_file(&socket_path).await?;
    }
    info!("Monitor daemon stopped");

    Ok(())
}

/// Handle a single IPC request
async fn handle_request(
    request: MonitorRequest,
    state: &DaemonState,
    policy_engine: &std::sync::Mutex<PolicyEngine>,
    backend_name: &str,
) -> MonitorResponse {
    match request {
        MonitorRequest::PreflightCheck {
            command,
            working_dir: _,
        } => {
            state.events_processed.fetch_add(1, Ordering::Relaxed);

            // Create a synthetic exec event for policy evaluation
            let event = caro::monitor::events::SyscallEvent::new(
                0, // PID unknown at preflight
                0,
                0,
                PathBuf::from("/bin/sh"),
                caro::monitor::events::SyscallDetail::Exec {
                    path: PathBuf::from("/bin/sh"),
                    args: vec![
                        "sh".to_string(),
                        "-c".to_string(),
                        command.clone(),
                    ],
                    env_count: 0,
                },
            );

            let decision = policy_engine.lock().unwrap().evaluate(&event);

            match decision.action {
                PolicyAction::Deny => {
                    state.events_blocked.fetch_add(1, Ordering::Relaxed);
                    info!(
                        command = command.as_str(),
                        reason = decision.reason.as_str(),
                        "Preflight check DENIED"
                    );
                    MonitorResponse::Denied {
                        reason: decision.reason,
                        risk_level: decision.risk_level,
                    }
                }
                _ => MonitorResponse::Ok,
            }
        }
        MonitorRequest::WatchProcess { pid, command } => {
            info!(pid = pid, command = command.as_str(), "Watching process");
            policy_engine.lock().unwrap().add_monitored_pid(pid);
            MonitorResponse::Ok
        }
        MonitorRequest::UnwatchProcess { pid } => {
            info!(pid = pid, "Unwatching process");
            policy_engine.lock().unwrap().remove_monitored_pid(pid);
            MonitorResponse::Ok
        }
        MonitorRequest::UpdatePolicy(new_policy) => {
            match PolicyEngine::new(new_policy) {
                Ok(engine) => {
                    *policy_engine.lock().unwrap() = engine;
                    info!("Security policy updated");
                    MonitorResponse::Ok
                }
                Err(e) => {
                    error!(error = %e, "Failed to update policy");
                    MonitorResponse::Error(format!("Invalid policy: {}", e))
                }
            }
        }
        MonitorRequest::Status => {
            let pids: Vec<u32> = policy_engine
                .lock()
                .unwrap()
                .monitored_pids()
                .iter()
                .copied()
                .collect();

            MonitorResponse::Status(DaemonStatus {
                running: true,
                backend: backend_name.to_string(),
                monitored_pids: pids,
                events_processed: state.events_processed.load(Ordering::Relaxed),
                events_blocked: state.events_blocked.load(Ordering::Relaxed),
                uptime_seconds: state.start_time.elapsed().as_secs(),
            })
        }
        MonitorRequest::Shutdown => {
            state.shutdown.store(true, Ordering::Relaxed);
            info!("Shutdown requested via IPC");
            MonitorResponse::Ok
        }
    }
}

/// Create the platform-appropriate monitoring backend
fn create_backend() -> anyhow::Result<Box<dyn MonitorBackend>> {
    #[cfg(target_os = "macos")]
    {
        info!("Detected macOS — using Endpoint Security backend");
        Ok(Box::new(
            caro::monitor::es_backend::EndpointSecurityBackend::new(),
        ))
    }

    #[cfg(target_os = "linux")]
    {
        info!("Detected Linux — using eBPF backend");
        Ok(Box::new(
            caro::monitor::ebpf_backend::EbpfBackend::new(),
        ))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(anyhow::anyhow!(
            "Kernel-level monitoring is not supported on this platform. \
             Supported: macOS (Endpoint Security), Linux (eBPF)"
        ))
    }
}

async fn stop_daemon(socket_path: PathBuf) -> anyhow::Result<()> {
    if !socket_path.exists() {
        println!("Monitor daemon is not running (no socket at {})", socket_path.display());
        return Ok(());
    }

    let mut stream = tokio::net::UnixStream::connect(&socket_path).await?;
    ipc::send_message(&mut stream, &MonitorRequest::Shutdown).await?;
    let response: MonitorResponse = ipc::recv_message(&mut stream).await?;

    match response {
        MonitorResponse::Ok => println!("Monitor daemon shutting down"),
        MonitorResponse::Error(e) => println!("Shutdown error: {}", e),
        _ => println!("Unexpected response"),
    }

    Ok(())
}

async fn query_status(socket_path: PathBuf) -> anyhow::Result<()> {
    if !socket_path.exists() {
        println!("Monitor daemon is not running (no socket at {})", socket_path.display());
        return Ok(());
    }

    let mut stream = tokio::net::UnixStream::connect(&socket_path).await?;
    ipc::send_message(&mut stream, &MonitorRequest::Status).await?;
    let response: MonitorResponse = ipc::recv_message(&mut stream).await?;

    match response {
        MonitorResponse::Status(status) => {
            println!("caro-monitor status:");
            println!("  Running:          {}", status.running);
            println!("  Backend:          {}", status.backend);
            println!("  Monitored PIDs:   {:?}", status.monitored_pids);
            println!("  Events processed: {}", status.events_processed);
            println!("  Events blocked:   {}", status.events_blocked);
            println!("  Uptime:           {}s", status.uptime_seconds);
        }
        MonitorResponse::Error(e) => println!("Status error: {}", e),
        _ => println!("Unexpected response"),
    }

    Ok(())
}
