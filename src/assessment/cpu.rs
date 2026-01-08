use serde::{Deserialize, Serialize};
use sysinfo::System;

use crate::assessment::profile::AssessmentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUInfo {
    pub architecture: String,
    pub cores: usize,
    pub model_name: String,
    pub frequency_mhz: Option<u64>,
}

impl CPUInfo {
    pub fn detect() -> Result<Self, AssessmentError> {
        let mut sys = System::new_all();
        sys.refresh_cpu_all();

        // Get CPU info from sysinfo
        let cpus = sys.cpus();
        if cpus.is_empty() {
            return Err(AssessmentError::DetectionFailed(
                "No CPUs detected".to_string(),
            ));
        }

        let cpu = &cpus[0];

        Ok(CPUInfo {
            architecture: std::env::consts::ARCH.to_string(),
            cores: sys.cpus().len(),
            model_name: cpu.brand().to_string(),
            frequency_mhz: Some(cpu.frequency()),
        })
    }
}
