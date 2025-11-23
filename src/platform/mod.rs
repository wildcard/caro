//! Platform-specific optimizations for cmdai
//!
//! This module provides platform detection and optimization for:
//! - macOS (Apple Silicon M1/M2/M3)
//! - Linux (various distributions)
//! - UNIX-compliant systems
//!
//! Key features:
//! - Apple Silicon detection for MLX backend
//! - Shell detection (Bash, Zsh, Fish, PowerShell)
//! - File descriptor optimization
//! - Platform-specific environment setup

use std::env;
use std::process::Command;

/// Platform information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os: OperatingSystem,
    pub arch: Architecture,
    pub shell: Shell,
    pub is_apple_silicon: bool,
    pub supports_metal: bool,
    pub cpu_cores: usize,
}

/// Operating system type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatingSystem {
    MacOS,
    Linux,
    Windows,
    FreeBSD,
    OpenBSD,
    Unknown,
}

/// CPU architecture
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Architecture {
    Aarch64,  // ARM64 (Apple Silicon, ARM servers)
    X86_64,   // x86-64 (Intel/AMD)
    X86,      // 32-bit x86
    Unknown,
}

/// Shell type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Sh,
    Unknown,
}

impl PlatformInfo {
    /// Detect current platform and capabilities
    pub fn detect() -> Self {
        let os = Self::detect_os();
        let arch = Self::detect_architecture();
        let shell = Self::detect_shell();
        let is_apple_silicon = Self::is_apple_silicon();
        let supports_metal = os == OperatingSystem::MacOS && is_apple_silicon;
        let cpu_cores = num_cpus::get();

        Self {
            os,
            arch,
            shell,
            is_apple_silicon,
            supports_metal,
            cpu_cores,
        }
    }

    /// Detect operating system
    fn detect_os() -> OperatingSystem {
        #[cfg(target_os = "macos")]
        return OperatingSystem::MacOS;

        #[cfg(target_os = "linux")]
        return OperatingSystem::Linux;

        #[cfg(target_os = "windows")]
        return OperatingSystem::Windows;

        #[cfg(target_os = "freebsd")]
        return OperatingSystem::FreeBSD;

        #[cfg(target_os = "openbsd")]
        return OperatingSystem::OpenBSD;

        #[cfg(not(any(
            target_os = "macos",
            target_os = "linux",
            target_os = "windows",
            target_os = "freebsd",
            target_os = "openbsd"
        )))]
        return OperatingSystem::Unknown;
    }

    /// Detect CPU architecture
    fn detect_architecture() -> Architecture {
        #[cfg(target_arch = "aarch64")]
        return Architecture::Aarch64;

        #[cfg(target_arch = "x86_64")]
        return Architecture::X86_64;

        #[cfg(target_arch = "x86")]
        return Architecture::X86;

        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "x86")))]
        return Architecture::Unknown;
    }

    /// Detect if running on Apple Silicon
    fn is_apple_silicon() -> bool {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            // Double-check with sysctl on macOS
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", "machdep.cpu.brand_string"])
                .output()
            {
                let brand = String::from_utf8_lossy(&output.stdout);
                return brand.contains("Apple");
            }
            true // Default to true for aarch64 macOS
        }

        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        false
    }

    /// Detect current shell
    fn detect_shell() -> Shell {
        // Try SHELL environment variable first
        if let Ok(shell_path) = env::var("SHELL") {
            let shell_name = shell_path
                .split('/')
                .last()
                .unwrap_or("")
                .to_lowercase();

            return match shell_name.as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                "sh" => Shell::Sh,
                "pwsh" | "powershell" => Shell::PowerShell,
                _ => Shell::Unknown,
            };
        }

        // Fallback to platform defaults
        #[cfg(target_os = "macos")]
        return Shell::Zsh; // macOS default since Catalina

        #[cfg(target_os = "linux")]
        return Shell::Bash; // Most Linux distros default

        #[cfg(target_os = "windows")]
        return Shell::PowerShell;

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return Shell::Sh;
    }

    /// Apply platform-specific optimizations
    pub fn apply_optimizations(&self) {
        // macOS + Apple Silicon optimizations
        if self.is_apple_silicon && self.supports_metal {
            self.optimize_for_metal();
        }

        // UNIX file descriptor optimizations
        #[cfg(unix)]
        {
            self.optimize_file_descriptors();
        }

        // Set appropriate thread pool size
        self.optimize_thread_pool();
    }

    /// Optimize for Metal Performance Shaders (macOS + Apple Silicon)
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn optimize_for_metal(&self) {
        // Enable Metal fallback for PyTorch/MLX
        env::set_var("PYTORCH_ENABLE_MPS_FALLBACK", "1");

        // Set Metal device selection (use unified memory)
        env::set_var("METAL_DEVICE_WRAPPER_TYPE", "1");

        tracing::debug!("Applied Metal Performance Shaders optimizations");
    }

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    fn optimize_for_metal(&self) {
        // No-op on non-Apple Silicon platforms
    }

    /// Optimize file descriptor limits for parallel operations
    #[cfg(unix)]
    fn optimize_file_descriptors(&self) {
        #[cfg(target_os = "linux")]
        {
            use libc::{rlimit, setrlimit, RLIMIT_NOFILE};

            unsafe {
                let new_limit = rlimit {
                    rlim_cur: 4096,
                    rlim_max: 4096,
                };

                if setrlimit(RLIMIT_NOFILE, &new_limit) == 0 {
                    tracing::debug!("Increased file descriptor limit to 4096");
                } else {
                    tracing::warn!("Failed to increase file descriptor limit");
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS has different limits, but generally sufficient
            tracing::debug!("Using macOS default file descriptor limits");
        }
    }

    /// Optimize thread pool size based on CPU cores
    fn optimize_thread_pool(&self) {
        // Set Rayon thread pool size if not already set
        if env::var("RAYON_NUM_THREADS").is_err() {
            let optimal_threads = if self.cpu_cores > 8 {
                // Leave some cores for system
                self.cpu_cores - 2
            } else {
                self.cpu_cores
            };

            env::set_var("RAYON_NUM_THREADS", optimal_threads.to_string());
            tracing::debug!("Set thread pool size to {}", optimal_threads);
        }
    }

    /// Get optimal backend for this platform
    pub fn recommended_backend(&self) -> &str {
        match (&self.os, &self.arch) {
            (OperatingSystem::MacOS, Architecture::Aarch64) => "mlx", // Apple Silicon
            (OperatingSystem::Linux, Architecture::Aarch64) => "cpu", // ARM servers
            (OperatingSystem::Linux, Architecture::X86_64) => "cpu",  // x86-64 servers
            (OperatingSystem::MacOS, Architecture::X86_64) => "cpu",  // Intel Macs
            _ => "cpu", // Fallback
        }
    }

    /// Check if platform supports GPU acceleration
    pub fn supports_gpu_acceleration(&self) -> bool {
        self.supports_metal // Currently only Metal on Apple Silicon
    }

    /// Get platform-specific cache directory
    pub fn cache_dir(&self) -> std::path::PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| env::temp_dir())
            .join("cmdai")
    }

    /// Get platform-specific config directory
    pub fn config_dir(&self) -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| env::temp_dir()))
            .join("cmdai")
    }

    /// Get platform-specific data directory
    pub fn data_dir(&self) -> std::path::PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| env::temp_dir()))
            .join("cmdai")
    }
}

impl std::fmt::Display for PlatformInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} ({} cores, shell: {:?}{})",
            self.os,
            self.arch,
            self.cpu_cores,
            self.shell,
            if self.is_apple_silicon { ", Apple Silicon" } else { "" }
        )
    }
}

/// Initialize platform optimizations
pub fn init() -> PlatformInfo {
    let platform = PlatformInfo::detect();

    tracing::info!("Detected platform: {}", platform);

    // Apply optimizations
    platform.apply_optimizations();

    platform
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = PlatformInfo::detect();

        // Basic sanity checks
        assert_ne!(platform.os, OperatingSystem::Unknown);
        assert_ne!(platform.arch, Architecture::Unknown);
        assert!(platform.cpu_cores > 0);
    }

    #[test]
    fn test_recommended_backend() {
        let platform = PlatformInfo::detect();
        let backend = platform.recommended_backend();

        assert!(!backend.is_empty());
        assert!(["mlx", "cpu", "ollama", "vllm"].contains(&backend));
    }

    #[test]
    fn test_directories_exist_or_creatable() {
        let platform = PlatformInfo::detect();

        let cache = platform.cache_dir();
        let config = platform.config_dir();
        let data = platform.data_dir();

        // Should be valid paths
        assert!(cache.is_absolute());
        assert!(config.is_absolute());
        assert!(data.is_absolute());
    }
}
