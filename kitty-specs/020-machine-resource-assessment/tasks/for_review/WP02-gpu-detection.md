---
work_package_id: WP02
title: "Platform-Specific GPU Detection"
priority: P1
phase: "core"
subtasks: [T009, T010, T011, T012, T013, T014]
lane: "for_review"
review_status: ""
reviewed_by: ""
assignee: ""
agent: "claude"
shell_pid: "71161"
history:
  - 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated work package prompt
---

# Work Package 02: Platform-Specific GPU Detection

## Objective

Implement GPU detection across macOS, Linux, and Windows platforms. This work package adds GPU hardware detection to the system profile, handling platform-specific detection methods and gracefully degrading when GPUs are unavailable or detection fails.

## Context

**User Story**: P1 - View System Resource Assessment (Acceptance Scenarios 3, 4)

**Why This Matters**: GPU presence significantly impacts model recommendations. Users need to know if they have GPU acceleration available (MLX on Apple Silicon, CUDA on NVIDIA) to select appropriate backends and models.

**Technical Approach**:
- macOS: Use `system_profiler SPDisplaysDataType` command or Metal framework
- Linux: Focus on NVIDIA with `nvml-wrapper` crate or `/proc/driver/nvidia/version`
- Windows: Use WMI queries via `wmic` command
- Graceful degradation: Report "No GPU detected" without errors

## Implementation Guidance

### T009: Implement GPUInfo Struct

**Location**: `src/assessment/gpu.rs`

Create GPU information structure:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUInfo {
    pub vendor: GPUVendor,
    pub model: String,
    pub vram_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GPUVendor {
    NVIDIA,
    AMD,
    Intel,
    Apple,
    Unknown,
}

impl GPUInfo {
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
}
```

### T010: macOS GPU Detection

Implement macOS detection using system_profiler:
```rust
impl GPUInfo {
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
        // Look for Apple Silicon (M1/M2), or discrete AMD/NVIDIA

        Some(GPUInfo {
            vendor: GPUVendor::Apple,  // Detect from model name
            model: "Apple M1".to_string(),  // Parse from JSON
            vram_mb: None,  // Unified memory on Apple Silicon
        })
    }
}
```

### T011: Linux GPU Detection (NVIDIA)

Implement Linux NVIDIA detection:
```rust
impl GPUInfo {
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

    fn detect_nvidia_smi() -> Option<Self> {
        use std::process::Command;

        let output = Command::new("nvidia-smi")
            .arg("--query-gpu=name,memory.total")
            .arg("--format=csv,noheader")
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let line = String::from_utf8(output.stdout).ok()?;
        let parts: Vec<&str> = line.trim().split(',').collect();

        Some(GPUInfo {
            vendor: GPUVendor::NVIDIA,
            model: parts.get(0)?.trim().to_string(),
            vram_mb: parts.get(1)?.trim().parse().ok(),
        })
    }
}
```

### T012: Windows GPU Detection

Implement Windows WMI detection:
```rust
impl GPUInfo {
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
        // Parse CSV output to extract GPU name and VRAM

        Some(GPUInfo {
            vendor: Self::detect_vendor_from_name(&"extracted_name"),
            model: "extracted_name".to_string(),
            vram_mb: Some(1024),  // Convert from bytes
        })
    }

    fn detect_vendor_from_name(name: &str) -> GPUVendor {
        let lower = name.to_lowercase();
        if lower.contains("nvidia") {
            GPUVendor::NVIDIA
        } else if lower.contains("amd") || lower.contains("radeon") {
            GPUVendor::AMD
        } else if lower.contains("intel") {
            GPUVendor::Intel
        } else {
            GPUVendor::Unknown
        }
    }
}
```

### T013: Graceful Fallback

Ensure detection never panics:
```rust
// Add error logging
impl GPUInfo {
    pub fn detect_with_logging() -> Option<Self> {
        match Self::detect() {
            Some(gpu) => {
                log::info!("Detected GPU: {} {}", gpu.vendor, gpu.model);
                Some(gpu)
            }
            None => {
                log::debug!("No GPU detected or detection failed");
                None
            }
        }
    }
}
```

### T014: Update SystemProfile

**Location**: `src/assessment/profile.rs`

Add GPU to SystemProfile:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProfile {
    pub cpu: CPUInfo,
    pub memory: MemoryInfo,
    pub gpu: Option<GPUInfo>,  // New field
    pub platform: PlatformInfo,
}

impl SystemProfile {
    pub fn detect() -> Result<Self, AssessmentError> {
        let cpu = CPUInfo::detect()?;
        let memory = MemoryInfo::detect()?;
        let gpu = GPUInfo::detect_with_logging();  // Optional
        let platform = PlatformInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        };

        Ok(SystemProfile { cpu, memory, gpu, platform })
    }
}
```

Update output formatter in `assess.rs`:
```rust
fn print_profile(profile: &SystemProfile) {
    // ... existing code ...

    if let Some(gpu) = &profile.gpu {
        println!("  GPU: {} {} ({:?} MB VRAM)",
            gpu.vendor,
            gpu.model,
            gpu.vram_mb
        );
    } else {
        println!("  GPU: No GPU detected");
    }
}
```

## Definition of Done

- [ ] GPUInfo struct implemented with vendor, model, VRAM fields
- [ ] macOS GPU detection works (Apple Silicon, discrete GPUs)
- [ ] Linux NVIDIA GPU detection works via nvidia-smi
- [ ] Windows GPU detection works via WMI
- [ ] "No GPU detected" message shown gracefully when GPU missing
- [ ] SystemProfile includes optional GPU field
- [ ] Output formatter displays GPU info when available
- [ ] No panics or crashes when GPU detection fails

## Risks & Mitigation

**Risk**: GPU detection commands may not be available
- **Mitigation**: Wrap all commands in Result types, return None on failure

**Risk**: Parsing command output may be brittle
- **Mitigation**: Use conservative parsing, fallback to None on parse errors

## Reviewer Guidance

Verify:
1. Platform-specific code uses correct `#[cfg]` attributes
2. All command executions have error handling
3. GPU detection failure doesn't crash the assessment
4. Output format handles both GPU present and absent cases

## Activity Log

- 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated initial work package prompt
- 2026-01-08T18:37:59Z – claude – shell_pid=68761 – lane=doing – Starting GPU detection implementation
- 2026-01-08T18:40:13Z – claude – shell_pid=71161 – lane=for_review – Completed GPU detection - all platforms supported, build succeeds
