use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUInfo {
    pub vendor: GPUVendor,
    pub model: String,
    pub vram_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GPUVendor {
    NVIDIA,
    AMD,
    Intel,
    Apple,
    Unknown,
}

impl fmt::Display for GPUVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GPUVendor::NVIDIA => write!(f, "NVIDIA"),
            GPUVendor::AMD => write!(f, "AMD"),
            GPUVendor::Intel => write!(f, "Intel"),
            GPUVendor::Apple => write!(f, "Apple"),
            GPUVendor::Unknown => write!(f, "Unknown"),
        }
    }
}

impl GPUInfo {
    /// Detect GPU information for the current platform
    pub fn detect() -> Option<Self> {
        #[cfg(target_os = "macos")]
        return Self::detect_macos();

        #[cfg(target_os = "linux")]
        return Self::detect_linux();

        #[cfg(target_os = "windows")]
        return Self::detect_windows();

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return None;
    }

    /// Detect GPU with logging for debugging
    pub fn detect_with_logging() -> Option<Self> {
        match Self::detect() {
            Some(gpu) => {
                tracing::info!("Detected GPU: {} {}", gpu.vendor, gpu.model);
                Some(gpu)
            }
            None => {
                tracing::debug!("No GPU detected or detection failed");
                None
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_macos() -> Option<Self> {
        use std::process::Command;

        let output = Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .arg("-json")
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_str = String::from_utf8(output.stdout).ok()?;

        // Parse JSON to extract GPU info
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if let Some(displays) = json.get("SPDisplaysDataType").and_then(|d| d.as_array()) {
                for display in displays {
                    if let Some(model) = display.get("sppci_model").and_then(|m| m.as_str()) {
                        let vendor = Self::detect_vendor_from_name(model);
                        let vram_mb = display
                            .get("spdisplays_vram")
                            .and_then(|v| v.as_str())
                            .and_then(Self::parse_vram_mb);

                        return Some(GPUInfo {
                            vendor,
                            model: model.to_string(),
                            vram_mb,
                        });
                    }
                }
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    fn detect_linux() -> Option<Self> {
        // Try nvidia-smi first (most reliable)
        if let Some(gpu) = Self::detect_nvidia_smi() {
            return Some(gpu);
        }

        // Fallback: check /proc/driver/nvidia/version
        if let Some(gpu) = Self::detect_nvidia_proc() {
            return Some(gpu);
        }

        // Could add AMD detection here via ROCm
        None
    }

    #[cfg(target_os = "linux")]
    fn detect_nvidia_smi() -> Option<Self> {
        use std::process::Command;

        let output = Command::new("nvidia-smi")
            .arg("--query-gpu=name,memory.total")
            .arg("--format=csv,noheader,nounits")
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let line = String::from_utf8(output.stdout).ok()?;
        let parts: Vec<&str> = line.trim().split(',').collect();

        if parts.len() < 2 {
            return None;
        }

        let model = parts[0].trim().to_string();
        let vram_mb = parts[1].trim().parse::<u64>().ok();

        Some(GPUInfo {
            vendor: GPUVendor::NVIDIA,
            model,
            vram_mb,
        })
    }

    #[cfg(target_os = "linux")]
    fn detect_nvidia_proc() -> Option<Self> {
        use std::fs;

        let version = fs::read_to_string("/proc/driver/nvidia/version").ok()?;

        // File exists, so NVIDIA driver is present
        // We can infer a GPU exists but don't have model/VRAM details
        Some(GPUInfo {
            vendor: GPUVendor::NVIDIA,
            model: "NVIDIA GPU (details unavailable)".to_string(),
            vram_mb: None,
        })
    }

    #[cfg(target_os = "windows")]
    fn detect_windows() -> Option<Self> {
        use std::process::Command;

        let output = Command::new("wmic")
            .arg("path")
            .arg("win32_VideoController")
            .arg("get")
            .arg("name,AdapterRAM")
            .arg("/format:csv")
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let csv = String::from_utf8(output.stdout).ok()?;

        // Parse CSV output (format: Node,AdapterRAM,Name)
        // Skip header row
        for line in csv.lines().skip(1) {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                let name = parts[2].trim();
                if name.is_empty() {
                    continue;
                }

                let vram_bytes: Option<u64> = parts[1].trim().parse().ok();
                let vram_mb = vram_bytes.map(|b| b / (1024 * 1024));

                return Some(GPUInfo {
                    vendor: Self::detect_vendor_from_name(name),
                    model: name.to_string(),
                    vram_mb,
                });
            }
        }

        None
    }

    fn detect_vendor_from_name(name: &str) -> GPUVendor {
        let lower = name.to_lowercase();
        if lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro") {
            GPUVendor::NVIDIA
        } else if lower.contains("amd") || lower.contains("radeon") {
            GPUVendor::AMD
        } else if lower.contains("intel") {
            GPUVendor::Intel
        } else if lower.contains("apple")
            || lower.starts_with("m1")
            || lower.starts_with("m2")
            || lower.starts_with("m3")
            || lower.starts_with("m4")
        {
            GPUVendor::Apple
        } else {
            GPUVendor::Unknown
        }
    }

    /// Parse VRAM size from macOS format (e.g., "8192 MB" or "8 GB")
    #[cfg(target_os = "macos")]
    fn parse_vram_mb(vram_str: &str) -> Option<u64> {
        let parts: Vec<&str> = vram_str.split_whitespace().collect();
        if parts.len() != 2 {
            return None;
        }

        let value: u64 = parts[0].parse().ok()?;
        let unit = parts[1].to_lowercase();

        match unit.as_str() {
            "mb" => Some(value),
            "gb" => Some(value * 1024),
            _ => None,
        }
    }
}
