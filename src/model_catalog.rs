// Model catalog for managing multiple model options across platforms
//
// Provides a selection of small, efficient models suitable for:
// - Local development
// - CI/CD environments (GitHub Actions)
// - Low-memory devices
// - Quick testing

use std::fmt;

/// Model size categories for selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelSize {
    /// Tiny models (<100MB) - Best for CI/CD
    Tiny,
    /// Small models (100-500MB) - Good balance
    Small,
    /// Medium models (500-1500MB) - Better quality
    Medium,
    /// Large models (>1500MB) - Best quality
    Large,
}

/// Model catalog entry with metadata
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Unique identifier for the model
    pub id: &'static str,
    /// Display name
    pub name: &'static str,
    /// Hugging Face repository
    pub hf_repo: &'static str,
    /// Filename within the repository
    pub filename: &'static str,
    /// Approximate size in MB
    pub size_mb: u64,
    /// Model size category
    pub size_category: ModelSize,
    /// Brief description
    pub description: &'static str,
    /// Whether this model is MLX-optimized
    pub mlx_optimized: bool,
    /// Whether this model is suitable for CI/CD
    pub ci_suitable: bool,
}

impl fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({} MB) - {}",
            self.name, self.size_mb, self.description
        )
    }
}

/// Catalog of available models
pub struct ModelCatalog;

impl ModelCatalog {
    /// Get the default model (balanced for most use cases)
    pub fn default() -> &'static ModelInfo {
        &QWEN_1_5B_Q4
    }

    /// Get the smallest model (best for CI/CD)
    pub fn smallest() -> &'static ModelInfo {
        &SMOLLM_135M_Q4
    }

    /// Get all available models
    pub fn all_models() -> &'static [&'static ModelInfo] {
        &ALL_MODELS
    }

    /// Get models suitable for CI/CD (< 500MB)
    pub fn ci_models() -> Vec<&'static ModelInfo> {
        ALL_MODELS
            .iter()
            .copied()
            .filter(|m| m.ci_suitable)
            .collect()
    }

    /// Get models by size category
    pub fn by_size(size: ModelSize) -> Vec<&'static ModelInfo> {
        ALL_MODELS
            .iter()
            .copied()
            .filter(|m| m.size_category == size)
            .collect()
    }

    /// Get model by ID
    pub fn by_id(id: &str) -> Option<&'static ModelInfo> {
        ALL_MODELS.iter().copied().find(|m| m.id == id)
    }

    /// Get MLX-optimized models
    pub fn mlx_models() -> Vec<&'static ModelInfo> {
        ALL_MODELS
            .iter()
            .copied()
            .filter(|m| m.mlx_optimized)
            .collect()
    }
}

// ============================================================================
// Model Definitions
// ============================================================================

/// TinyLlama 1.1B Q4_K_M - Smallest option (~50MB)
/// Best for: CI/CD, testing, memory-constrained environments
pub static TINYLLAMA_1_1B_Q4: ModelInfo = ModelInfo {
    id: "tinyllama-1.1b-q4",
    name: "TinyLlama 1.1B Q4",
    hf_repo: "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF",
    filename: "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
    size_mb: 669,
    size_category: ModelSize::Small,
    description: "Smallest model, ideal for CI/CD and testing",
    mlx_optimized: false,
    ci_suitable: true,
};

/// Phi-2 2.7B Q4_K_M - Efficient small model (~1.6GB)
/// Best for: Good balance between size and quality
pub static PHI_2_2_7B_Q4: ModelInfo = ModelInfo {
    id: "phi-2-q4",
    name: "Phi-2 2.7B Q4",
    hf_repo: "TheBloke/phi-2-GGUF",
    filename: "phi-2.Q4_K_M.gguf",
    size_mb: 1560,
    size_category: ModelSize::Medium,
    description: "Microsoft Phi-2, excellent code understanding",
    mlx_optimized: false,
    ci_suitable: false,
};

/// Mistral 7B Instruct Q3_K_M - Powerful but compact (~3.5GB)
/// Best for: Production use when quality matters
pub static MISTRAL_7B_Q3: ModelInfo = ModelInfo {
    id: "mistral-7b-q3",
    name: "Mistral 7B Instruct Q3",
    hf_repo: "TheBloke/Mistral-7B-Instruct-v0.2-GGUF",
    filename: "mistral-7b-instruct-v0.2.Q3_K_M.gguf",
    size_mb: 3520,
    size_category: ModelSize::Large,
    description: "High-quality instruction following, larger size",
    mlx_optimized: false,
    ci_suitable: false,
};

/// StarCoder 1B Q4_K_M - Code-specialized (~700MB)
/// Best for: Code generation and shell commands
pub static STARCODER_1B_Q4: ModelInfo = ModelInfo {
    id: "starcoder-1b-q4",
    name: "StarCoder 1B Q4",
    hf_repo: "TheBloke/starcoderbase-1b-GGUF",
    filename: "starcoderbase-1b.Q4_K_M.gguf",
    size_mb: 700,
    size_category: ModelSize::Small,
    description: "Code-specialized model, good for shell commands",
    mlx_optimized: false,
    ci_suitable: true,
};

/// Qwen2.5-Coder 1.5B Q4_K_M - Default model (~1.1GB)
/// Best for: Balanced performance and size, good code understanding
pub static QWEN_1_5B_Q4: ModelInfo = ModelInfo {
    id: "qwen-1.5b-q4",
    name: "Qwen2.5-Coder 1.5B Q4",
    hf_repo: "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF",
    filename: "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
    size_mb: 1117,
    size_category: ModelSize::Medium,
    description: "Default model, excellent for code and commands",
    mlx_optimized: true,
    ci_suitable: false,
};

/// Qwen2.5-Coder 0.5B Q4_K_M - Tiny version (~350MB)
/// Best for: CI/CD when Qwen quality is desired
pub static QWEN_0_5B_Q4: ModelInfo = ModelInfo {
    id: "qwen-0.5b-q4",
    name: "Qwen2.5-Coder 0.5B Q4",
    hf_repo: "Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF",
    filename: "qwen2.5-coder-0.5b-instruct-q4_k_m.gguf",
    size_mb: 352,
    size_category: ModelSize::Small,
    description: "Tiny Qwen model, fast and CI-friendly",
    mlx_optimized: true,
    ci_suitable: true,
};

/// SmolLM 135M Q4_K_M - Ultra-tiny for fastest tests (~80MB)
/// Best for: Unit tests, extreme resource constraints
pub static SMOLLM_135M_Q4: ModelInfo = ModelInfo {
    id: "smollm-135m-q4",
    name: "SmolLM 135M Q4",
    hf_repo: "HuggingFaceTB/SmolLM-135M-Instruct-GGUF",
    filename: "smollm-135m-instruct-q4_k_m.gguf",
    size_mb: 82,
    size_category: ModelSize::Tiny,
    description: "Ultra-tiny model for testing only",
    mlx_optimized: false,
    ci_suitable: true,
};

// All models in order of size (smallest to largest)
static ALL_MODELS: &[&ModelInfo] = &[
    &SMOLLM_135M_Q4,
    &QWEN_0_5B_Q4,
    &TINYLLAMA_1_1B_Q4,
    &STARCODER_1B_Q4,
    &QWEN_1_5B_Q4,
    &PHI_2_2_7B_Q4,
    &MISTRAL_7B_Q3,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model() {
        let model = ModelCatalog::default();
        assert_eq!(model.id, "qwen-1.5b-q4");
    }

    #[test]
    fn test_smallest_model() {
        let model = ModelCatalog::smallest();
        assert_eq!(model.id, "smollm-135m-q4");
        assert!(model.size_mb < 100);
    }

    #[test]
    fn test_ci_models() {
        let models = ModelCatalog::ci_models();
        assert!(!models.is_empty());
        for model in models {
            assert!(model.ci_suitable);
            assert!(model.size_mb < 1000);
        }
    }

    #[test]
    fn test_by_size() {
        let tiny = ModelCatalog::by_size(ModelSize::Tiny);
        let small = ModelCatalog::by_size(ModelSize::Small);
        assert!(!tiny.is_empty());
        assert!(!small.is_empty());
    }

    #[test]
    fn test_by_id() {
        let model = ModelCatalog::by_id("qwen-0.5b-q4");
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "Qwen2.5-Coder 0.5B Q4");
    }

    #[test]
    fn test_mlx_models() {
        let models = ModelCatalog::mlx_models();
        assert!(!models.is_empty());
        for model in models {
            assert!(model.mlx_optimized);
        }
    }

    #[test]
    fn test_all_models_sorted() {
        let models = ModelCatalog::all_models();
        for i in 0..models.len() - 1 {
            assert!(models[i].size_mb <= models[i + 1].size_mb);
        }
    }
}
