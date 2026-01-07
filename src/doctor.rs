//! Diagnostic utilities for troubleshooting caro installation and configuration
//!
//! The `caro doctor` command provides comprehensive system diagnostics to help users
//! identify and resolve issues with their caro installation.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::time::Duration;

use crate::model_loader::ModelLoader;

/// Diagnostic report containing system information and health checks
#[derive(Debug)]
pub struct DiagnosticReport {
    pub system_info: SystemInfo,
    pub network_status: NetworkStatus,
    pub cache_status: CacheStatus,
    pub backend_status: BackendStatus,
}

/// System information (OS, architecture, shell)
#[derive(Debug)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub shell: String,
    pub shell_version: String,
}

/// Network connectivity status
#[derive(Debug)]
pub struct NetworkStatus {
    pub huggingface_reachable: bool,
    pub proxy_detected: bool,
    pub proxy_settings: Option<String>,
}

/// Cache directory and model status
#[derive(Debug)]
pub struct CacheStatus {
    pub cache_dir: PathBuf,
    pub cache_exists: bool,
    pub model_present: bool,
    pub model_path: Option<PathBuf>,
    pub model_size_mb: Option<u64>,
}

/// Backend availability status
#[derive(Debug)]
pub struct BackendStatus {
    pub embedded_available: bool,
    pub embedded_needs_model: bool,
    pub ollama_available: bool,
    pub ollama_running: bool,
}

impl DiagnosticReport {
    /// Run comprehensive diagnostics and generate a report
    pub async fn generate() -> Result<Self> {
        let system_info = SystemInfo::detect()?;
        let network_status = NetworkStatus::check().await;
        let cache_status = CacheStatus::check()?;
        let backend_status = BackendStatus::check().await;

        Ok(Self {
            system_info,
            network_status,
            cache_status,
            backend_status,
        })
    }

    /// Print the diagnostic report to stdout
    pub fn print(&self) {
        println!("{}", "Caro Diagnostics".bold().cyan());
        println!("{}", "================".cyan());
        println!();

        // System Information
        println!("{}", "System:".bold());
        println!(
            "  {} {} ({}), {} {}",
            self.system_info.os_name,
            self.system_info.os_version,
            self.system_info.architecture,
            self.system_info.shell,
            self.system_info.shell_version
        );
        println!();

        // Network Status
        println!("{}", "Network:".bold());
        if self.network_status.huggingface_reachable {
            println!("  {} huggingface.co reachable", "✓".green());
        } else {
            println!(
                "  {} huggingface.co not reachable (model downloads may fail)",
                "✗".red()
            );
        }

        if self.network_status.proxy_detected {
            if let Some(proxy) = &self.network_status.proxy_settings {
                println!("  {} Proxy detected: {}", "ℹ".blue(), proxy);
            }
        }
        println!();

        // Cache Status
        println!("{}", "Cache:".bold());
        println!("  Directory: {}", self.cache_status.cache_dir.display());

        if !self.cache_status.cache_exists {
            println!("  {} Cache directory does not exist", "⚠".yellow());
        }

        if self.cache_status.model_present {
            if let Some(path) = &self.cache_status.model_path {
                println!("  {} Model found: {}", "✓".green(), path.display());
                if let Some(size_mb) = self.cache_status.model_size_mb {
                    println!("    Size: {} MB", size_mb);
                }
            }
        } else {
            println!("  {} No model downloaded yet", "ℹ".blue());
            println!("    Models will be downloaded automatically on first use");
        }
        println!();

        // Backend Status
        println!("{}", "Backends:".bold());

        if self.backend_status.embedded_available {
            if self.backend_status.embedded_needs_model {
                println!("  {} Embedded (needs model download)", "⚠".yellow());
            } else {
                println!("  {} Embedded (ready)", "✓".green());
            }
        } else {
            println!("  {} Embedded (not available)", "✗".red());
        }

        if self.backend_status.ollama_available {
            if self.backend_status.ollama_running {
                println!("  {} Ollama (running)", "✓".green());
            } else {
                println!("  {} Ollama (installed but not running)", "⚠".yellow());
                println!("    Start with: ollama serve");
            }
        } else {
            println!("  {} Ollama (not installed)", "ℹ".blue());
            println!("    Install from: https://ollama.ai");
        }
        println!();

        // Overall Health Summary
        self.print_health_summary();
    }

    /// Print overall health summary and recommendations
    fn print_health_summary(&self) {
        let has_issues = !self.network_status.huggingface_reachable
            || (self.backend_status.embedded_needs_model && !self.backend_status.ollama_running);

        if has_issues {
            println!("{}", "Issues Detected:".bold().yellow());

            if !self.network_status.huggingface_reachable {
                println!("  {} Network connectivity issue", "•".yellow());
                println!("    Check your internet connection and proxy settings");
                println!("    Try: ping huggingface.co");
            }

            if self.backend_status.embedded_needs_model && !self.backend_status.ollama_running {
                println!("  {} No ready backend available", "•".yellow());
                println!("    Either download a model or start Ollama");
                println!("    Try running: caro \"list files\" (will download model)");
                println!("    Or install and start Ollama: https://ollama.ai");
            }

            println!();
        } else {
            println!("{} All systems operational", "✓".green().bold());
            println!();
        }

        // Helpful commands
        println!("{}", "Helpful Commands:".bold());
        println!("  caro --version          Show version information");
        println!("  caro --show-config      Display current configuration");
        println!("  caro --help             Show all available options");
        println!();
    }
}

impl SystemInfo {
    /// Detect system information
    fn detect() -> Result<Self> {
        let os_info = os_info::get();
        let os_name = os_info.os_type().to_string();
        let os_version = os_info.version().to_string();

        let architecture = std::env::consts::ARCH.to_string();

        // Detect shell
        let shell = std::env::var("SHELL")
            .unwrap_or_else(|_| "unknown".to_string())
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string();

        // Get shell version
        let shell_version =
            Self::get_shell_version(&shell).unwrap_or_else(|| "unknown".to_string());

        Ok(Self {
            os_name,
            os_version,
            architecture,
            shell,
            shell_version,
        })
    }

    /// Get shell version by running shell --version
    fn get_shell_version(shell: &str) -> Option<String> {
        use std::process::Command;

        let output = Command::new(shell).arg("--version").output().ok()?;

        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            // Extract version number from first line
            version_str.lines().next().and_then(|line| {
                // Try to extract version number (e.g., "5.9" from "zsh 5.9 (x86_64-apple-darwin23.0)")
                line.split_whitespace()
                    .find(|word| word.chars().next().map(|c| c.is_numeric()).unwrap_or(false))
                    .map(|v| v.to_string())
            })
        } else {
            None
        }
    }
}

impl NetworkStatus {
    /// Check network connectivity to Hugging Face Hub
    async fn check() -> Self {
        let huggingface_reachable = Self::check_huggingface_connectivity().await;

        let http_proxy = std::env::var("HTTP_PROXY")
            .ok()
            .or_else(|| std::env::var("http_proxy").ok());
        let https_proxy = std::env::var("HTTPS_PROXY")
            .ok()
            .or_else(|| std::env::var("https_proxy").ok());

        let proxy_detected = http_proxy.is_some() || https_proxy.is_some();
        let proxy_settings = https_proxy.or(http_proxy);

        Self {
            huggingface_reachable,
            proxy_detected,
            proxy_settings,
        }
    }

    /// Check if huggingface.co is reachable
    async fn check_huggingface_connectivity() -> bool {
        // Try to resolve huggingface.co DNS
        use tokio::net::TcpStream;
        use tokio::time::timeout;

        match timeout(
            Duration::from_secs(5),
            TcpStream::connect("huggingface.co:443"),
        )
        .await
        {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }
}

impl CacheStatus {
    /// Check cache directory and model status
    fn check() -> Result<Self> {
        let loader = ModelLoader::new()?;
        let cache_dir = ModelLoader::default_cache_dir()?;
        let cache_exists = cache_dir.exists();

        // Try to get model path
        let model_path = loader.get_embedded_model_path().ok();
        let model_present = model_path.as_ref().map(|p| p.exists()).unwrap_or(false);

        // Get model size if present
        let model_size_mb = if model_present {
            model_path.as_ref().and_then(|path| {
                std::fs::metadata(path)
                    .ok()
                    .map(|m| m.len() / (1024 * 1024))
            })
        } else {
            None
        };

        Ok(Self {
            cache_dir,
            cache_exists,
            model_present,
            model_path,
            model_size_mb,
        })
    }
}

impl BackendStatus {
    /// Check backend availability
    async fn check() -> Self {
        let embedded_available = true; // Embedded backend is always compiled in
        let model_present = CacheStatus::check()
            .map(|c| c.model_present)
            .unwrap_or(false);
        let embedded_needs_model = !model_present;

        // Check if Ollama is installed and running
        let (ollama_available, ollama_running) = Self::check_ollama().await;

        Self {
            embedded_available,
            embedded_needs_model,
            ollama_available,
            ollama_running,
        }
    }

    /// Check if Ollama is installed and running
    async fn check_ollama() -> (bool, bool) {
        use std::process::Command;

        // Check if ollama binary exists
        let ollama_installed = Command::new("which")
            .arg("ollama")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !ollama_installed {
            return (false, false);
        }

        // Check if Ollama API is responding
        #[cfg(feature = "remote-backends")]
        {
            let ollama_running = reqwest::Client::new()
                .get("http://localhost:11434/api/version")
                .timeout(Duration::from_secs(2))
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            (ollama_installed, ollama_running)
        }

        #[cfg(not(feature = "remote-backends"))]
        {
            // Without reqwest, we can't check if it's running
            (ollama_installed, false)
        }
    }
}

/// Run diagnostics and print the report
pub async fn run_diagnostics() -> Result<()> {
    let report = DiagnosticReport::generate().await?;
    report.print();
    Ok(())
}
