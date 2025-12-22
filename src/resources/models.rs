//! Model Tier Definitions
//!
//! Defines the available model tiers (micro, small, medium, large, custom)
//! with their specifications and requirements.

use serde::{Deserialize, Serialize};

/// Model tier categories based on system resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelTier {
    /// Micro: Fastest, simplest setup (Qwen 2.5 Coder 1.5B)
    /// Best for: Limited resources, very fast responses
    Micro,

    /// Small: Slightly more capable (Qwen3 1.7B)
    /// Best for: Basic tasks with slightly better quality
    Small,

    /// Medium: Thinking model with better reasoning (Qwen3 4B Thinking)
    /// Best for: Complex tasks requiring reasoning
    Medium,

    /// Large: Full-featured model (Qwen3 8B or DeepSeek R1 Qwen3 8B)
    /// Best for: Maximum quality, complex reasoning, tool use
    Large,

    /// Custom: User-specified model from Hugging Face
    Custom,
}

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelTier::Micro => write!(f, "Micro (1.5B)"),
            ModelTier::Small => write!(f, "Small (1.7B)"),
            ModelTier::Medium => write!(f, "Medium (4B)"),
            ModelTier::Large => write!(f, "Large (8B)"),
            ModelTier::Custom => write!(f, "Custom"),
        }
    }
}

impl ModelTier {
    /// Get all available tiers
    pub fn all() -> &'static [ModelTier] {
        &[
            ModelTier::Micro,
            ModelTier::Small,
            ModelTier::Medium,
            ModelTier::Large,
            ModelTier::Custom,
        ]
    }

    /// Get preset tiers (excluding custom)
    pub fn presets() -> &'static [ModelTier] {
        &[
            ModelTier::Micro,
            ModelTier::Small,
            ModelTier::Medium,
            ModelTier::Large,
        ]
    }
}

/// Detailed model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierModelInfo {
    /// Model tier
    pub tier: ModelTier,

    /// Human-readable name
    pub name: String,

    /// Full model identifier (HuggingFace repo/file)
    pub model_id: String,

    /// Hugging Face repository
    pub hf_repo: String,

    /// GGUF filename
    pub gguf_file: String,

    /// Model size in MB
    pub size_mb: u64,

    /// Parameter count (in billions)
    pub parameters_b: f32,

    /// Whether this model supports thinking/reasoning
    pub supports_thinking: bool,

    /// Whether this model supports tool calling
    pub supports_tool_calling: bool,

    /// Minimum RAM required in GB
    pub min_ram_gb: u64,

    /// Minimum GPU memory required in GB (0 for CPU-only capable)
    pub min_gpu_memory_gb: u64,

    /// Minimum storage required in GB
    pub min_storage_gb: u64,

    /// Typical inference latency in seconds
    pub typical_latency_s: f32,

    /// Description of the model's capabilities
    pub description: String,

    /// LMStudio URL for reference
    pub lmstudio_url: Option<String>,
}

impl TierModelInfo {
    /// Get the default micro model (Qwen 2.5 Coder 1.5B)
    pub fn micro() -> Self {
        Self {
            tier: ModelTier::Micro,
            name: "Qwen 2.5 Coder 1.5B".to_string(),
            model_id: "qwen2.5-coder-1.5b-instruct".to_string(),
            hf_repo: "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF".to_string(),
            gguf_file: "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf".to_string(),
            size_mb: 1100,
            parameters_b: 1.5,
            supports_thinking: false,
            supports_tool_calling: false,
            min_ram_gb: 4,
            min_gpu_memory_gb: 0, // Can run on CPU
            min_storage_gb: 2,
            typical_latency_s: 1.5,
            description: "Fast, lightweight coder model. Best for simple command generation with quick responses.".to_string(),
            lmstudio_url: None,
        }
    }

    /// Get the small model (Qwen3 1.7B)
    pub fn small() -> Self {
        Self {
            tier: ModelTier::Small,
            name: "Qwen3 1.7B".to_string(),
            model_id: "qwen3-1.7b".to_string(),
            hf_repo: "Qwen/Qwen3-1.7B-GGUF".to_string(),
            gguf_file: "qwen3-1.7b-q4_k_m.gguf".to_string(),
            size_mb: 1200,
            parameters_b: 1.7,
            supports_thinking: true,
            supports_tool_calling: false,
            min_ram_gb: 4,
            min_gpu_memory_gb: 0,
            min_storage_gb: 2,
            typical_latency_s: 2.0,
            description: "Improved reasoning with basic thinking capabilities. Good balance of speed and quality.".to_string(),
            lmstudio_url: Some("https://lmstudio.ai/models/qwen/qwen3-1.7b".to_string()),
        }
    }

    /// Get the medium model (Qwen3 4B Thinking)
    pub fn medium() -> Self {
        Self {
            tier: ModelTier::Medium,
            name: "Qwen3 4B Thinking".to_string(),
            model_id: "qwen3-4b-thinking".to_string(),
            hf_repo: "Qwen/Qwen3-4B-Thinking-2507-GGUF".to_string(),
            gguf_file: "qwen3-4b-thinking-2507-q4_k_m.gguf".to_string(),
            size_mb: 2800,
            parameters_b: 4.0,
            supports_thinking: true,
            supports_tool_calling: true,
            min_ram_gb: 8,
            min_gpu_memory_gb: 4,
            min_storage_gb: 4,
            typical_latency_s: 3.5,
            description: "Advanced thinking model with explicit reasoning. Handles complex and ambiguous requests well.".to_string(),
            lmstudio_url: Some("https://lmstudio.ai/models/qwen/qwen3-4b-thinking-2507".to_string()),
        }
    }

    /// Get the large model (Qwen3 8B)
    pub fn large() -> Self {
        Self {
            tier: ModelTier::Large,
            name: "Qwen3 8B".to_string(),
            model_id: "qwen3-8b".to_string(),
            hf_repo: "Qwen/Qwen3-8B-GGUF".to_string(),
            gguf_file: "qwen3-8b-q4_k_m.gguf".to_string(),
            size_mb: 5000,
            parameters_b: 8.0,
            supports_thinking: true,
            supports_tool_calling: true,
            min_ram_gb: 16,
            min_gpu_memory_gb: 8,
            min_storage_gb: 6,
            typical_latency_s: 5.0,
            description: "Full-featured large model with comprehensive reasoning and tool use. Best quality for complex tasks.".to_string(),
            lmstudio_url: Some("https://lmstudio.ai/models/qwen/qwen3-8b".to_string()),
        }
    }

    /// Get the large alternative model (DeepSeek R1 Qwen3 8B)
    pub fn large_deepseek() -> Self {
        Self {
            tier: ModelTier::Large,
            name: "DeepSeek R1 Qwen3 8B".to_string(),
            model_id: "deepseek-r1-qwen3-8b".to_string(),
            hf_repo: "deepseek-ai/DeepSeek-R1-0528-Qwen3-8B-GGUF".to_string(),
            gguf_file: "deepseek-r1-0528-qwen3-8b-q4_k_m.gguf".to_string(),
            size_mb: 5200,
            parameters_b: 8.0,
            supports_thinking: true,
            supports_tool_calling: true,
            min_ram_gb: 16,
            min_gpu_memory_gb: 8,
            min_storage_gb: 6,
            typical_latency_s: 5.5,
            description: "DeepSeek R1 variant with enhanced reasoning and coding abilities.".to_string(),
            lmstudio_url: None,
        }
    }

    /// Create a custom model info
    pub fn custom(
        name: String,
        hf_repo: String,
        gguf_file: String,
        size_mb: u64,
        parameters_b: f32,
    ) -> Self {
        Self {
            tier: ModelTier::Custom,
            name,
            model_id: format!("custom-{}", hf_repo.replace('/', "-")),
            hf_repo,
            gguf_file,
            size_mb,
            parameters_b,
            supports_thinking: false, // Unknown
            supports_tool_calling: false,
            min_ram_gb: (size_mb / 1024) + 2, // Model size + 2GB overhead
            min_gpu_memory_gb: 0,
            min_storage_gb: (size_mb / 1024) + 1,
            typical_latency_s: 3.0,
            description: "User-specified custom model from Hugging Face.".to_string(),
            lmstudio_url: None,
        }
    }

    /// Get model by tier
    pub fn for_tier(tier: ModelTier) -> Self {
        match tier {
            ModelTier::Micro => Self::micro(),
            ModelTier::Small => Self::small(),
            ModelTier::Medium => Self::medium(),
            ModelTier::Large => Self::large(),
            ModelTier::Custom => panic!("Use TierModelInfo::custom() for custom models"),
        }
    }

    /// Get full Hugging Face URL
    pub fn hf_url(&self) -> String {
        format!("https://huggingface.co/{}/resolve/main/{}", self.hf_repo, self.gguf_file)
    }

    /// Get size in GB
    pub fn size_gb(&self) -> f32 {
        self.size_mb as f32 / 1024.0
    }
}

impl std::fmt::Display for TierModelInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} ({})", self.name, self.tier)?;
        writeln!(f, "  Parameters: {:.1}B", self.parameters_b)?;
        writeln!(f, "  Size: {:.1} GB", self.size_gb())?;
        writeln!(f, "  Thinking: {}", if self.supports_thinking { "Yes" } else { "No" })?;
        writeln!(f, "  Tool Calling: {}", if self.supports_tool_calling { "Yes" } else { "No" })?;
        writeln!(f, "  Min RAM: {} GB", self.min_ram_gb)?;
        if self.min_gpu_memory_gb > 0 {
            writeln!(f, "  Min GPU Memory: {} GB", self.min_gpu_memory_gb)?;
        }
        writeln!(f, "  Typical Latency: {:.1}s", self.typical_latency_s)?;
        writeln!(f, "  {}", self.description)?;
        Ok(())
    }
}

/// Configuration for model tiers with runtime preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTierConfig {
    /// Selected model tier
    pub tier: ModelTier,

    /// Selected model info
    pub model: TierModelInfo,

    /// Whether to enable thinking/reasoning mode
    pub enable_thinking: bool,

    /// Whether to enable tool calling
    pub enable_tool_calling: bool,

    /// Custom HuggingFace repo (for Custom tier)
    pub custom_hf_repo: Option<String>,

    /// Custom GGUF file (for Custom tier)
    pub custom_gguf_file: Option<String>,

    /// Fast mode (disable extra features for speed)
    pub fast_mode: bool,
}

impl Default for ModelTierConfig {
    fn default() -> Self {
        let model = TierModelInfo::micro();
        Self {
            tier: ModelTier::Micro,
            model,
            enable_thinking: false,
            enable_tool_calling: false,
            custom_hf_repo: None,
            custom_gguf_file: None,
            fast_mode: true,
        }
    }
}

impl ModelTierConfig {
    /// Create config for a specific tier
    pub fn for_tier(tier: ModelTier) -> Self {
        let model = TierModelInfo::for_tier(tier);
        let enable_thinking = model.supports_thinking;
        let enable_tool_calling = model.supports_tool_calling;
        let fast_mode = tier == ModelTier::Micro;

        Self {
            tier,
            model,
            enable_thinking,
            enable_tool_calling,
            custom_hf_repo: None,
            custom_gguf_file: None,
            fast_mode,
        }
    }

    /// Create config for a custom model
    pub fn custom(hf_repo: String, gguf_file: String, size_mb: u64) -> Self {
        let model = TierModelInfo::custom(
            format!("Custom: {}", gguf_file),
            hf_repo.clone(),
            gguf_file.clone(),
            size_mb,
            0.0, // Unknown parameter count
        );

        Self {
            tier: ModelTier::Custom,
            model,
            enable_thinking: false,
            enable_tool_calling: false,
            custom_hf_repo: Some(hf_repo),
            custom_gguf_file: Some(gguf_file),
            fast_mode: false,
        }
    }

    /// Enable fast mode (simpler processing)
    pub fn with_fast_mode(mut self, fast: bool) -> Self {
        self.fast_mode = fast;
        self
    }

    /// Enable thinking/reasoning
    pub fn with_thinking(mut self, thinking: bool) -> Self {
        self.enable_thinking = thinking && self.model.supports_thinking;
        self
    }

    /// Enable tool calling
    pub fn with_tool_calling(mut self, tool_calling: bool) -> Self {
        self.enable_tool_calling = tool_calling && self.model.supports_tool_calling;
        self
    }

    /// Get model download path
    pub fn model_path(&self) -> String {
        if self.tier == ModelTier::Custom {
            if let (Some(repo), Some(file)) = (&self.custom_hf_repo, &self.custom_gguf_file) {
                return format!("{}/{}", repo, file);
            }
        }
        format!("{}/{}", self.model.hf_repo, self.model.gguf_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_tiers() {
        let tiers = ModelTier::presets();
        assert_eq!(tiers.len(), 4);
        assert_eq!(tiers[0], ModelTier::Micro);
        assert_eq!(tiers[3], ModelTier::Large);
    }

    #[test]
    fn test_model_info_micro() {
        let model = TierModelInfo::micro();
        assert_eq!(model.tier, ModelTier::Micro);
        assert!(model.size_mb > 0);
        assert!(model.parameters_b > 0.0);
        assert!(!model.supports_thinking);
    }

    #[test]
    fn test_model_info_medium() {
        let model = TierModelInfo::medium();
        assert_eq!(model.tier, ModelTier::Medium);
        assert!(model.supports_thinking);
        assert!(model.size_mb > TierModelInfo::micro().size_mb);
    }

    #[test]
    fn test_model_tier_config() {
        let config = ModelTierConfig::for_tier(ModelTier::Medium);
        assert_eq!(config.tier, ModelTier::Medium);
        assert!(config.enable_thinking);
    }

    #[test]
    fn test_model_info_display() {
        let model = TierModelInfo::micro();
        let display = format!("{}", model);
        assert!(display.contains("Qwen"));
        assert!(display.contains("1.5B"));
    }

    #[test]
    fn test_custom_model() {
        let custom = TierModelInfo::custom(
            "my-org/my-model".to_string(),
            "my-org/my-model-GGUF".to_string(),
            "my-model-q4.gguf".to_string(),
            2000,
            3.0,
        );
        assert_eq!(custom.tier, ModelTier::Custom);
        assert_eq!(custom.size_mb, 2000);
    }

    #[test]
    fn test_hf_url() {
        let model = TierModelInfo::micro();
        let url = model.hf_url();
        assert!(url.contains("huggingface.co"));
        assert!(url.contains("Qwen"));
    }
}
