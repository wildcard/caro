//! Model Recommendation Engine
//!
//! Recommends the optimal model tier based on system resources
//! and user preferences.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::assessment::SystemResources;
use super::models::{ModelInfo, ModelTier, ModelTierConfig};
use super::ResourceError;

/// A model recommendation with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    /// Recommended tier
    pub tier: ModelTier,

    /// Model information
    pub model: ModelInfo,

    /// Reasoning for the recommendation
    pub reasoning: String,

    /// Resource requirements summary
    pub resource_summary: String,

    /// Whether this recommendation fits the system
    pub is_compatible: bool,

    /// Warnings about this choice
    pub warnings: Vec<String>,

    /// Alternative recommendations
    pub alternatives: Vec<ModelTier>,
}

impl ModelRecommendation {
    /// Create a new recommendation
    pub fn new(tier: ModelTier, resources: &SystemResources) -> Self {
        let model = ModelInfo::for_tier(tier);
        let (is_compatible, warnings) = Self::check_compatibility(&model, resources);
        let reasoning = Self::generate_reasoning(tier, resources);
        let resource_summary = Self::generate_resource_summary(&model, resources);
        let alternatives = Self::find_alternatives(tier, resources);

        Self {
            tier,
            model,
            reasoning,
            resource_summary,
            is_compatible,
            warnings,
            alternatives,
        }
    }

    /// Check if model is compatible with resources
    fn check_compatibility(model: &ModelInfo, resources: &SystemResources) -> (bool, Vec<String>) {
        let mut warnings = Vec::new();
        let mut compatible = true;

        // Check storage
        if resources.available_storage_gb() < model.min_storage_gb {
            warnings.push(format!(
                "Storage: {} GB available, {} GB required. Consider freeing disk space.",
                resources.available_storage_gb(),
                model.min_storage_gb
            ));
            compatible = false;
        }

        // Check RAM
        if resources.ram_gb() < model.min_ram_gb {
            warnings.push(format!(
                "RAM: {} GB available, {} GB minimum. Performance may be degraded.",
                resources.ram_gb(),
                model.min_ram_gb
            ));
            compatible = false;
        }

        // Check GPU memory
        if model.min_gpu_memory_gb > 0 {
            let gpu_mem = resources.effective_gpu_memory_gb();
            if gpu_mem < model.min_gpu_memory_gb {
                warnings.push(format!(
                    "GPU Memory: {} GB available, {} GB recommended. Will use CPU fallback.",
                    gpu_mem,
                    model.min_gpu_memory_gb
                ));
            }
        }

        // Check GPU availability for larger models
        if model.tier != ModelTier::Micro && !resources.has_gpu_acceleration() {
            warnings.push(
                "No GPU acceleration detected. Inference will be slower on CPU.".to_string(),
            );
        }

        (compatible, warnings)
    }

    /// Generate reasoning for the recommendation
    fn generate_reasoning(tier: ModelTier, resources: &SystemResources) -> String {
        match tier {
            ModelTier::Micro => {
                if resources.is_light_machine() {
                    "Recommended for your system's resources. Fast inference with good code generation quality.".to_string()
                } else {
                    "Selected for fastest possible responses. Your system could run larger models if needed.".to_string()
                }
            }
            ModelTier::Small => {
                "Good balance of speed and capability. Supports basic thinking for better responses.".to_string()
            }
            ModelTier::Medium => {
                if resources.is_medium_machine() {
                    "Optimal for your system. Thinking model provides better reasoning for complex requests.".to_string()
                } else {
                    "Advanced thinking model with explicit reasoning. Handles ambiguous requests well.".to_string()
                }
            }
            ModelTier::Large => {
                if resources.is_heavy_machine() {
                    "Your system has excellent resources. This model provides the best quality for complex tasks.".to_string()
                } else {
                    "Full-featured model with comprehensive capabilities. May be slower on your system.".to_string()
                }
            }
            ModelTier::Custom => {
                "User-specified model. Ensure it's compatible with your system resources.".to_string()
            }
        }
    }

    /// Generate resource summary
    fn generate_resource_summary(model: &ModelInfo, resources: &SystemResources) -> String {
        let storage_status = if resources.available_storage_gb() >= model.min_storage_gb {
            format!("Storage: OK ({} GB available)", resources.available_storage_gb())
        } else {
            format!(
                "Storage: LOW ({} GB available, {} GB needed)",
                resources.available_storage_gb(),
                model.min_storage_gb
            )
        };

        let ram_status = if resources.ram_gb() >= model.min_ram_gb {
            format!("RAM: OK ({} GB available)", resources.ram_gb())
        } else {
            format!(
                "RAM: LOW ({} GB available, {} GB needed)",
                resources.ram_gb(),
                model.min_ram_gb
            )
        };

        let gpu_status = if resources.has_gpu_acceleration() {
            format!(
                "GPU: {} ({} GB effective memory)",
                resources.gpu.as_ref().map(|g| g.name.as_str()).unwrap_or("Available"),
                resources.effective_gpu_memory_gb()
            )
        } else {
            "GPU: Not available (CPU-only mode)".to_string()
        };

        format!("{}\n{}\n{}", storage_status, ram_status, gpu_status)
    }

    /// Find alternative tiers that would work
    fn find_alternatives(selected: ModelTier, resources: &SystemResources) -> Vec<ModelTier> {
        let mut alternatives = Vec::new();

        for tier in ModelTier::presets() {
            if *tier != selected {
                let model = ModelInfo::for_tier(*tier);
                if resources.available_storage_gb() >= model.min_storage_gb
                    && resources.ram_gb() >= model.min_ram_gb
                {
                    alternatives.push(*tier);
                }
            }
        }

        alternatives
    }
}

impl std::fmt::Display for ModelRecommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Recommended Model: {} ({})", self.model.name, self.tier)?;
        writeln!(f)?;
        writeln!(f, "Reasoning: {}", self.reasoning)?;
        writeln!(f)?;
        writeln!(f, "Resource Check:")?;
        for line in self.resource_summary.lines() {
            writeln!(f, "  {}", line)?;
        }

        if !self.warnings.is_empty() {
            writeln!(f)?;
            writeln!(f, "Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  - {}", warning)?;
            }
        }

        if !self.alternatives.is_empty() {
            writeln!(f)?;
            writeln!(f, "Alternatives: {:?}", self.alternatives)?;
        }

        Ok(())
    }
}

/// Engine for generating model recommendations
pub struct RecommendationEngine {
    resources: SystemResources,
}

impl RecommendationEngine {
    /// Create a new recommendation engine
    pub fn new(resources: SystemResources) -> Self {
        Self { resources }
    }

    /// Get the optimal recommendation based on resources
    pub fn recommend(&self) -> ModelRecommendation {
        let tier = self.determine_optimal_tier();
        info!("Optimal tier determined: {:?}", tier);
        ModelRecommendation::new(tier, &self.resources)
    }

    /// Get recommendations for all tiers
    pub fn all_recommendations(&self) -> Vec<ModelRecommendation> {
        ModelTier::presets()
            .iter()
            .map(|tier| ModelRecommendation::new(*tier, &self.resources))
            .collect()
    }

    /// Get compatible recommendations only
    pub fn compatible_recommendations(&self) -> Vec<ModelRecommendation> {
        self.all_recommendations()
            .into_iter()
            .filter(|r| r.is_compatible)
            .collect()
    }

    /// Determine the optimal tier based on resources
    fn determine_optimal_tier(&self) -> ModelTier {
        debug!("Determining optimal tier for resources: {:?}", self.resources);

        // Check if we have heavy resources (large model)
        if self.resources.is_heavy_machine() {
            let large = ModelInfo::large();
            if self.can_run_model(&large) {
                debug!("Heavy machine detected, recommending Large tier");
                return ModelTier::Large;
            }
        }

        // Check if we have medium resources (medium model)
        if self.resources.is_medium_machine() || self.resources.is_heavy_machine() {
            let medium = ModelInfo::medium();
            if self.can_run_model(&medium) {
                debug!("Medium machine detected, recommending Medium tier");
                return ModelTier::Medium;
            }
        }

        // Check if we can run small model
        let small = ModelInfo::small();
        if self.can_run_model(&small) {
            debug!("Recommending Small tier");
            return ModelTier::Small;
        }

        // Default to micro
        debug!("Defaulting to Micro tier");
        ModelTier::Micro
    }

    /// Check if a model can be run on current resources
    fn can_run_model(&self, model: &ModelInfo) -> bool {
        // Check storage
        if self.resources.available_storage_gb() < model.min_storage_gb {
            debug!(
                "Insufficient storage for {}: {} < {}",
                model.name,
                self.resources.available_storage_gb(),
                model.min_storage_gb
            );
            return false;
        }

        // Check RAM
        if self.resources.ram_gb() < model.min_ram_gb {
            debug!(
                "Insufficient RAM for {}: {} < {}",
                model.name,
                self.resources.ram_gb(),
                model.min_ram_gb
            );
            return false;
        }

        true
    }

    /// Validate a user's tier selection
    pub fn validate_selection(&self, tier: ModelTier) -> Result<ModelRecommendation, ResourceError> {
        let recommendation = ModelRecommendation::new(tier, &self.resources);

        if !recommendation.is_compatible {
            let warnings_str = recommendation.warnings.join("; ");
            return Err(ResourceError::InsufficientResources(format!(
                "Selected tier {} has resource issues: {}",
                tier, warnings_str
            )));
        }

        Ok(recommendation)
    }

    /// Create configuration for a selected tier
    pub fn create_config(&self, tier: ModelTier) -> ModelTierConfig {
        let config = ModelTierConfig::for_tier(tier);

        // Adjust based on resources
        let config = if !self.resources.has_gpu_acceleration() {
            // Disable advanced features on CPU-only systems for speed
            config.with_fast_mode(true)
        } else {
            config
        };

        config
    }

    /// Get resource summary for display
    pub fn resource_summary(&self) -> String {
        format!("{}", self.resources)
    }

    /// Get machine classification
    pub fn machine_class(&self) -> &'static str {
        if self.resources.is_heavy_machine() {
            "Heavy (suitable for 8B+ models)"
        } else if self.resources.is_medium_machine() {
            "Medium (suitable for 4B models)"
        } else {
            "Light (suitable for 1-2B models)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::assessment::GpuInfo;
    use crate::resources::assessment::GpuVendor;

    fn create_test_resources(ram_gb: u64, storage_gb: u64, has_gpu: bool) -> SystemResources {
        SystemResources {
            total_ram_mb: ram_gb * 1024,
            available_ram_mb: (ram_gb * 1024) - 2048, // Assume 2GB in use
            cpu_cores: 8,
            cpu_brand: "Test CPU".to_string(),
            gpu: if has_gpu {
                Some(GpuInfo {
                    name: "Test GPU".to_string(),
                    vendor: GpuVendor::Apple,
                    vram_mb: ram_gb * 512, // Half of RAM as GPU memory
                    metal_available: true,
                    cuda_available: false,
                    compute_capability: None,
                })
            } else {
                None
            },
            available_storage_mb: storage_gb * 1024,
            total_storage_mb: storage_gb * 1024 * 2,
            cache_dir: std::path::PathBuf::from("/tmp"),
            os: "test".to_string(),
            arch: "aarch64".to_string(),
            is_apple_silicon: has_gpu,
        }
    }

    #[test]
    fn test_recommend_small_or_micro_for_limited_resources() {
        // Limited resources - 4GB RAM without GPU, can run Micro and Small
        let resources = create_test_resources(4, 10, false);
        let engine = RecommendationEngine::new(resources);
        let recommendation = engine.recommend();

        // Should recommend Small (it's more capable) or Micro
        assert!(
            recommendation.tier == ModelTier::Micro || recommendation.tier == ModelTier::Small,
            "Expected Micro or Small, got {:?}",
            recommendation.tier
        );
        assert!(recommendation.is_compatible);
    }

    #[test]
    fn test_recommend_large_for_heavy_machine() {
        let resources = create_test_resources(32, 100, true);
        let engine = RecommendationEngine::new(resources);
        let recommendation = engine.recommend();

        assert_eq!(recommendation.tier, ModelTier::Large);
        assert!(recommendation.is_compatible);
    }

    #[test]
    fn test_recommend_medium_for_medium_machine() {
        let resources = create_test_resources(16, 50, true);
        let engine = RecommendationEngine::new(resources);
        let recommendation = engine.recommend();

        // Should recommend medium or higher
        assert!(recommendation.tier == ModelTier::Medium || recommendation.tier == ModelTier::Large);
    }

    #[test]
    fn test_all_recommendations() {
        let resources = create_test_resources(16, 50, true);
        let engine = RecommendationEngine::new(resources);
        let all = engine.all_recommendations();

        assert_eq!(all.len(), 4); // All preset tiers
    }

    #[test]
    fn test_compatible_recommendations() {
        let resources = create_test_resources(8, 20, true);
        let engine = RecommendationEngine::new(resources);
        let compatible = engine.compatible_recommendations();

        // Should have at least micro and small
        assert!(!compatible.is_empty());
        assert!(compatible.iter().any(|r| r.tier == ModelTier::Micro));
    }

    #[test]
    fn test_validate_selection() {
        let resources = create_test_resources(4, 5, false);
        let engine = RecommendationEngine::new(resources);

        // Micro should work
        assert!(engine.validate_selection(ModelTier::Micro).is_ok());

        // Large should fail due to resources
        let result = engine.validate_selection(ModelTier::Large);
        assert!(result.is_err() || !result.unwrap().is_compatible);
    }

    #[test]
    fn test_machine_class() {
        let heavy = create_test_resources(32, 100, true);
        let engine = RecommendationEngine::new(heavy);
        assert!(engine.machine_class().contains("Heavy"));

        let light = create_test_resources(4, 10, false);
        let engine = RecommendationEngine::new(light);
        assert!(engine.machine_class().contains("Light"));
    }
}
