use serde::{Deserialize, Serialize};

use crate::assessment::{CPUInfo, MemoryInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProfile {
    pub cpu: CPUInfo,
    pub memory: MemoryInfo,
    pub platform: PlatformInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

impl SystemProfile {
    pub fn detect() -> Result<Self, AssessmentError> {
        let cpu = CPUInfo::detect()?;
        let memory = MemoryInfo::detect()?;
        let platform = PlatformInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        };

        Ok(SystemProfile {
            cpu,
            memory,
            platform,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssessmentError {
    #[error("Failed to detect system information: {0}")]
    DetectionFailed(String),
}
