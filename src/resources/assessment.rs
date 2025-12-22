//! System Resource Assessment
//!
//! Detects GPU, memory, storage, and other system resources to determine
//! optimal model configuration.

use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use tracing::{debug, info, warn};

use super::ResourceError;

/// GPU information for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU name/model
    pub name: String,
    /// GPU vendor (Apple, NVIDIA, AMD, Intel)
    pub vendor: GpuVendor,
    /// Available VRAM in MB (0 for unified memory systems)
    pub vram_mb: u64,
    /// Whether Metal is available (macOS)
    pub metal_available: bool,
    /// Whether CUDA is available
    pub cuda_available: bool,
    /// Compute capability (for CUDA devices)
    pub compute_capability: Option<String>,
}

/// GPU vendor categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Apple,
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

impl std::fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuVendor::Apple => write!(f, "Apple"),
            GpuVendor::Nvidia => write!(f, "NVIDIA"),
            GpuVendor::Amd => write!(f, "AMD"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Unknown => write!(f, "Unknown"),
        }
    }
}

/// System resources detected during assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// Total system RAM in MB
    pub total_ram_mb: u64,
    /// Available system RAM in MB
    pub available_ram_mb: u64,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// CPU brand/model name
    pub cpu_brand: String,
    /// GPU information (if available)
    pub gpu: Option<GpuInfo>,
    /// Available storage in MB
    pub available_storage_mb: u64,
    /// Total storage in MB
    pub total_storage_mb: u64,
    /// Cache directory path
    pub cache_dir: std::path::PathBuf,
    /// Operating system
    pub os: String,
    /// Architecture (x86_64, aarch64, etc.)
    pub arch: String,
    /// Whether this is Apple Silicon
    pub is_apple_silicon: bool,
}

impl SystemResources {
    /// Get RAM in GB (rounded)
    pub fn ram_gb(&self) -> u64 {
        self.total_ram_mb / 1024
    }

    /// Get available RAM in GB (rounded)
    pub fn available_ram_gb(&self) -> u64 {
        self.available_ram_mb / 1024
    }

    /// Get available storage in GB (rounded)
    pub fn available_storage_gb(&self) -> u64 {
        self.available_storage_mb / 1024
    }

    /// Check if system has GPU acceleration
    pub fn has_gpu_acceleration(&self) -> bool {
        if let Some(gpu) = &self.gpu {
            gpu.metal_available || gpu.cuda_available
        } else {
            false
        }
    }

    /// Get effective GPU memory (VRAM or unified memory for Apple Silicon)
    pub fn effective_gpu_memory_mb(&self) -> u64 {
        if self.is_apple_silicon {
            // Apple Silicon uses unified memory - can use most of system RAM
            // Reserve ~4GB for system and other apps
            self.available_ram_mb.saturating_sub(4096)
        } else if let Some(gpu) = &self.gpu {
            gpu.vram_mb
        } else {
            0
        }
    }

    /// Get effective GPU memory in GB
    pub fn effective_gpu_memory_gb(&self) -> u64 {
        self.effective_gpu_memory_mb() / 1024
    }

    /// Determine if this is a "heavy" machine (suitable for large models)
    pub fn is_heavy_machine(&self) -> bool {
        let gpu_mem_gb = self.effective_gpu_memory_gb();
        let ram_gb = self.ram_gb();

        // Heavy: 16+ GB GPU memory or 32+ GB RAM with GPU acceleration
        gpu_mem_gb >= 16 || (ram_gb >= 32 && self.has_gpu_acceleration())
    }

    /// Determine if this is a "medium" machine
    pub fn is_medium_machine(&self) -> bool {
        let gpu_mem_gb = self.effective_gpu_memory_gb();
        let ram_gb = self.ram_gb();

        // Medium: 8-16 GB GPU memory or 16-32 GB RAM with GPU
        (gpu_mem_gb >= 8 && gpu_mem_gb < 16)
            || (ram_gb >= 16 && ram_gb < 32 && self.has_gpu_acceleration())
    }

    /// Determine if this is a "light" machine (basic GPU or CPU-only)
    pub fn is_light_machine(&self) -> bool {
        !self.is_heavy_machine() && !self.is_medium_machine()
    }
}

impl std::fmt::Display for SystemResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "System Resources:")?;
        writeln!(f, "  OS: {} ({})", self.os, self.arch)?;
        writeln!(f, "  CPU: {} ({} cores)", self.cpu_brand, self.cpu_cores)?;
        writeln!(
            f,
            "  RAM: {} GB total, {} GB available",
            self.ram_gb(),
            self.available_ram_gb()
        )?;

        if let Some(gpu) = &self.gpu {
            writeln!(f, "  GPU: {} ({})", gpu.name, gpu.vendor)?;
            if gpu.vram_mb > 0 {
                writeln!(f, "  VRAM: {} GB", gpu.vram_mb / 1024)?;
            }
            if gpu.metal_available {
                writeln!(f, "  Metal: Available")?;
            }
            if gpu.cuda_available {
                writeln!(f, "  CUDA: Available")?;
            }
        } else {
            writeln!(f, "  GPU: Not detected")?;
        }

        if self.is_apple_silicon {
            writeln!(
                f,
                "  Unified Memory: {} GB effective",
                self.effective_gpu_memory_gb()
            )?;
        }

        writeln!(
            f,
            "  Storage: {} GB available of {} GB",
            self.available_storage_gb(),
            self.total_storage_mb / 1024
        )?;

        Ok(())
    }
}

/// Resource assessment service
pub struct ResourceAssessment;

impl ResourceAssessment {
    /// Perform a complete system resource assessment
    pub fn assess() -> Result<SystemResources, ResourceError> {
        info!("Starting system resource assessment...");

        let mut sys = System::new_all();
        sys.refresh_all();

        // Get CPU information
        let cpu_cores = sys.cpus().len();
        let cpu_brand = sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        debug!("CPU: {} ({} cores)", cpu_brand, cpu_cores);

        // Get memory information
        let total_ram_mb = sys.total_memory() / (1024 * 1024);
        let available_ram_mb = sys.available_memory() / (1024 * 1024);

        debug!(
            "RAM: {} MB total, {} MB available",
            total_ram_mb, available_ram_mb
        );

        // Get storage information (refresh system to get disk info)
        let (total_storage_mb, available_storage_mb) = Self::calculate_storage(&sys);

        debug!(
            "Storage: {} MB available of {} MB",
            available_storage_mb, total_storage_mb
        );

        // Detect GPU
        let gpu = Self::detect_gpu();
        debug!("GPU detection result: {:?}", gpu);

        // Detect architecture
        let arch = std::env::consts::ARCH.to_string();
        let os = std::env::consts::OS.to_string();
        let is_apple_silicon = cfg!(all(target_os = "macos", target_arch = "aarch64"));

        // Get cache directory
        let cache_dir = directories::BaseDirs::new()
            .map(|dirs| dirs.cache_dir().join("caro").join("models"))
            .unwrap_or_else(|| std::path::PathBuf::from(".cache/caro/models"));

        let resources = SystemResources {
            total_ram_mb,
            available_ram_mb,
            cpu_cores,
            cpu_brand,
            gpu,
            available_storage_mb,
            total_storage_mb,
            cache_dir,
            os,
            arch,
            is_apple_silicon,
        };

        info!(
            "Assessment complete: {} GB RAM, {} GB storage available",
            resources.ram_gb(),
            resources.available_storage_gb()
        );

        Ok(resources)
    }

    /// Calculate storage from available disks
    fn calculate_storage(sys: &System) -> (u64, u64) {
        let mut total = 0u64;
        let mut available = 0u64;

        for disk in sys.disks() {
            // Only count the root filesystem or primary drive
            let mount = disk.mount_point();
            if mount.to_string_lossy() == "/"
                || mount.starts_with("/home")
                || mount.to_string_lossy().contains(":")
            {
                total += disk.total_space() / (1024 * 1024);
                available += disk.available_space() / (1024 * 1024);
            }
        }

        // Fallback to first disk if nothing matched
        if total == 0 {
            if let Some(disk) = sys.disks().first() {
                total = disk.total_space() / (1024 * 1024);
                available = disk.available_space() / (1024 * 1024);
            }
        }

        (total, available)
    }

    /// Detect GPU capabilities
    fn detect_gpu() -> Option<GpuInfo> {
        // Apple Silicon detection
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            return Some(Self::detect_apple_silicon_gpu());
        }

        // Linux/Windows GPU detection
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            Self::detect_discrete_gpu()
        }
    }

    /// Detect Apple Silicon GPU
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn detect_apple_silicon_gpu() -> GpuInfo {
        // Try to get more specific chip info
        let chip_name = Self::get_apple_chip_name();

        GpuInfo {
            name: chip_name,
            vendor: GpuVendor::Apple,
            vram_mb: 0, // Unified memory - calculated from system RAM
            metal_available: true,
            cuda_available: false,
            compute_capability: None,
        }
    }

    /// Get Apple chip name from system profiler
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn get_apple_chip_name() -> String {
        use std::process::Command;

        let output = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let brand = String::from_utf8_lossy(&output.stdout);
                return brand.trim().to_string();
            }
        }

        // Fallback
        "Apple Silicon GPU".to_string()
    }

    /// Detect discrete GPU on Linux/Windows
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    fn detect_discrete_gpu() -> Option<GpuInfo> {
        // Try NVIDIA detection first
        if let Some(nvidia_gpu) = Self::detect_nvidia_gpu() {
            return Some(nvidia_gpu);
        }

        // Try AMD detection
        if let Some(amd_gpu) = Self::detect_amd_gpu() {
            return Some(amd_gpu);
        }

        // Try Intel integrated
        if let Some(intel_gpu) = Self::detect_intel_gpu() {
            return Some(intel_gpu);
        }

        None
    }

    /// Detect NVIDIA GPU using nvidia-smi
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    fn detect_nvidia_gpu() -> Option<GpuInfo> {
        use std::process::Command;

        // Try nvidia-smi
        let output = Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = stdout.trim().split(',').collect();

                if parts.len() >= 2 {
                    let name = parts[0].trim().to_string();
                    let vram_mb: u64 = parts[1].trim().parse().unwrap_or(0);

                    return Some(GpuInfo {
                        name,
                        vendor: GpuVendor::Nvidia,
                        vram_mb,
                        metal_available: false,
                        cuda_available: true,
                        compute_capability: Self::get_nvidia_compute_capability(),
                    });
                }
            }
        }

        None
    }

    /// Get NVIDIA compute capability
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    fn get_nvidia_compute_capability() -> Option<String> {
        use std::process::Command;

        let output = Command::new("nvidia-smi")
            .args([
                "--query-gpu=compute_cap",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let cap = String::from_utf8_lossy(&output.stdout);
                let cap = cap.trim();
                if !cap.is_empty() && cap != "[N/A]" {
                    return Some(cap.to_string());
                }
            }
        }

        None
    }

    /// Detect AMD GPU
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    fn detect_amd_gpu() -> Option<GpuInfo> {
        use std::process::Command;

        // Try rocm-smi
        let output = Command::new("rocm-smi").args(["--showproductname"]).output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("GPU") || line.contains("Card") {
                        let name = line.trim().to_string();
                        let vram = Self::get_amd_vram();

                        return Some(GpuInfo {
                            name,
                            vendor: GpuVendor::Amd,
                            vram_mb: vram,
                            metal_available: false,
                            cuda_available: false,
                            compute_capability: None,
                        });
                    }
                }
            }
        }

        None
    }

    /// Get AMD VRAM
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    fn get_amd_vram() -> u64 {
        use std::process::Command;

        let output = Command::new("rocm-smi").args(["--showmeminfo", "vram"]).output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("Total") {
                        // Parse total VRAM from line like "Total Memory (B): 17163091968"
                        if let Some(bytes_str) = line.split(':').nth(1) {
                            if let Ok(bytes) = bytes_str.trim().parse::<u64>() {
                                return bytes / (1024 * 1024);
                            }
                        }
                    }
                }
            }
        }

        0
    }

    /// Detect Intel integrated GPU
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    fn detect_intel_gpu() -> Option<GpuInfo> {
        use std::process::Command;

        // Check for Intel GPU via lspci on Linux
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("lspci").output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if line.contains("VGA") && line.to_lowercase().contains("intel") {
                            let name = line
                                .split(':')
                                .nth(2)
                                .map(|s| s.trim().to_string())
                                .unwrap_or_else(|| "Intel Integrated Graphics".to_string());

                            return Some(GpuInfo {
                                name,
                                vendor: GpuVendor::Intel,
                                vram_mb: 0, // Shared memory
                                metal_available: false,
                                cuda_available: false,
                                compute_capability: None,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Quick check if sufficient resources are available for a model size
    pub fn check_sufficient_resources(
        resources: &SystemResources,
        model_size_mb: u64,
    ) -> Result<(), ResourceError> {
        // Check storage
        if resources.available_storage_mb < model_size_mb {
            return Err(ResourceError::InsufficientResources(format!(
                "Insufficient storage: {} GB available, {} GB required",
                resources.available_storage_gb(),
                model_size_mb / 1024
            )));
        }

        // Check RAM (need at least model size + 2GB overhead)
        let required_ram = model_size_mb + 2048;
        if resources.available_ram_mb < required_ram {
            warn!(
                "Low RAM: {} MB available, {} MB recommended",
                resources.available_ram_mb, required_ram
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_assessment() {
        let resources = ResourceAssessment::assess();
        assert!(resources.is_ok());

        let resources = resources.unwrap();
        assert!(resources.total_ram_mb > 0);
        assert!(resources.cpu_cores > 0);
        assert!(!resources.cpu_brand.is_empty());
    }

    #[test]
    fn test_system_resources_display() {
        let resources = ResourceAssessment::assess().unwrap();
        let display = format!("{}", resources);
        assert!(display.contains("System Resources:"));
        assert!(display.contains("CPU:"));
        assert!(display.contains("RAM:"));
    }

    #[test]
    fn test_machine_classification() {
        // Create a test resource set for a heavy machine
        let heavy = SystemResources {
            total_ram_mb: 32768, // 32 GB
            available_ram_mb: 24576,
            cpu_cores: 10,
            cpu_brand: "Test CPU".to_string(),
            gpu: Some(GpuInfo {
                name: "Test GPU".to_string(),
                vendor: GpuVendor::Apple,
                vram_mb: 0,
                metal_available: true,
                cuda_available: false,
                compute_capability: None,
            }),
            available_storage_mb: 102400,
            total_storage_mb: 512000,
            cache_dir: std::path::PathBuf::from("/tmp"),
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            is_apple_silicon: true,
        };

        assert!(heavy.is_heavy_machine());
        assert!(!heavy.is_medium_machine());
        assert!(!heavy.is_light_machine());
    }
}
