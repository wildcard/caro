use serde::{Deserialize, Serialize};
use sysinfo::System;

use crate::assessment::profile::AssessmentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub available_mb: u64,
}

impl MemoryInfo {
    pub fn detect() -> Result<Self, AssessmentError> {
        let mut sys = System::new_all();
        sys.refresh_memory();

        let total_mb = sys.total_memory() / 1024 / 1024;
        let available_mb = sys.available_memory() / 1024 / 1024;

        if total_mb == 0 {
            return Err(AssessmentError::DetectionFailed(
                "Could not detect system memory".to_string(),
            ));
        }

        Ok(MemoryInfo {
            total_mb,
            available_mb,
        })
    }
}
