use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::assessment::{ModelRecommendation, SystemProfile};

/// Complete assessment result with timestamp and warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    pub timestamp: DateTime<Utc>,
    pub system_profile: SystemProfile,
    pub recommendations: Vec<ModelRecommendation>,
    pub warnings: Vec<String>,
}

impl AssessmentResult {
    pub fn new(
        profile: SystemProfile,
        recommendations: Vec<ModelRecommendation>,
        warnings: Vec<String>,
    ) -> Self {
        AssessmentResult {
            timestamp: Utc::now(),
            system_profile: profile,
            recommendations,
            warnings,
        }
    }
}
