//! Resource Assessment and Model Recommendation Module
//!
//! This module provides system resource detection and model recommendations
//! for optimal LLM configuration based on available hardware.

mod assessment;
mod models;
mod onboarding;
mod recommendation;

pub use assessment::{ResourceAssessment, SystemResources};
pub use models::{ModelTier, ModelTierConfig, TierModelInfo};
pub use onboarding::{OnboardingFlow, OnboardingResult, UserPreferences};
pub use recommendation::{ModelRecommendation, RecommendationEngine};

use thiserror::Error;

/// Errors that can occur during resource assessment
#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Failed to detect system resources: {0}")]
    DetectionError(String),

    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),

    #[error("Model download failed: {0}")]
    DownloadError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
