use serde::{Deserialize, Serialize};
use std::fmt;

use crate::assessment::{GPUVendor, SystemProfile};

/// Hardware capability tiers based on RAM and GPU
#[derive(Debug, Clone, Copy, PartialEq)]
enum HardwareTier {
    /// < 8GB RAM, no dedicated GPU
    LowEnd,
    /// 8-16GB RAM, integrated or no GPU
    MidRange,
    /// > 16GB RAM, dedicated GPU
    HighEnd,
}

impl HardwareTier {
    fn from_profile(profile: &SystemProfile) -> Self {
        let has_dedicated_gpu = profile
            .gpu
            .as_ref()
            .map(|g| matches!(g.vendor, GPUVendor::NVIDIA | GPUVendor::AMD))
            .unwrap_or(false);

        if profile.memory.total_mb < 8192 {
            HardwareTier::LowEnd
        } else if profile.memory.total_mb < 16384 || !has_dedicated_gpu {
            HardwareTier::MidRange
        } else {
            HardwareTier::HighEnd
        }
    }
}

/// Backend type for model execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Backend {
    MLX,
    CUDA,
    CPU,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Backend::MLX => write!(f, "MLX"),
            Backend::CUDA => write!(f, "CUDA"),
            Backend::CPU => write!(f, "CPU"),
        }
    }
}

/// Model recommendation with configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub model_name: String,
    pub model_size: String,
    pub backend: Backend,
    pub quantization: Option<String>,
    pub reasoning: String,
    pub estimated_memory_mb: u64,
}

/// Recommendation engine that maps hardware to optimal models
pub struct Recommender;

impl Recommender {
    /// Generate model recommendations based on system profile
    pub fn recommend(profile: &SystemProfile) -> Vec<ModelRecommendation> {
        let tier = HardwareTier::from_profile(profile);
        let backend = Self::select_backend(profile);

        match tier {
            HardwareTier::LowEnd => Self::low_end_recommendations(profile, backend),
            HardwareTier::MidRange => Self::mid_range_recommendations(profile, backend),
            HardwareTier::HighEnd => Self::high_end_recommendations(profile, backend),
        }
    }

    fn select_backend(profile: &SystemProfile) -> Backend {
        if let Some(gpu) = &profile.gpu {
            match gpu.vendor {
                GPUVendor::Apple => Backend::MLX,
                GPUVendor::NVIDIA => Backend::CUDA,
                _ => Backend::CPU,
            }
        } else {
            Backend::CPU
        }
    }

    fn low_end_recommendations(
        profile: &SystemProfile,
        backend: Backend,
    ) -> Vec<ModelRecommendation> {
        vec![
            Self::create_recommendation(
                "Phi-2",
                "2.7B",
                backend.clone(),
                Some("Q4_K_M".to_string()),
                profile,
                "Lightweight model optimized for systems with limited RAM",
            ),
            Self::create_recommendation(
                "TinyLlama",
                "1.1B",
                backend,
                Some("Q4_K_M".to_string()),
                profile,
                "Very small model for extremely constrained systems",
            ),
        ]
    }

    fn mid_range_recommendations(
        profile: &SystemProfile,
        backend: Backend,
    ) -> Vec<ModelRecommendation> {
        vec![
            Self::create_recommendation(
                "Phi-2",
                "2.7B",
                backend.clone(),
                None,
                profile,
                "Efficient model for balanced performance and quality",
            ),
            Self::create_recommendation(
                "Mistral 7B",
                "7B",
                backend,
                Some("Q4_K_M".to_string()),
                profile,
                "High-quality model with moderate resource requirements",
            ),
        ]
    }

    fn high_end_recommendations(
        profile: &SystemProfile,
        backend: Backend,
    ) -> Vec<ModelRecommendation> {
        let mut recs = vec![Self::create_recommendation(
            "Mistral 7B",
            "7B",
            backend.clone(),
            None,
            profile,
            "Full precision model for optimal quality",
        )];

        // Add larger models if sufficient VRAM available
        if let Some(gpu) = &profile.gpu {
            if gpu.vram_mb.map(|v| v >= 16384).unwrap_or(false) {
                recs.push(Self::create_recommendation(
                    "Llama 2 13B",
                    "13B",
                    backend,
                    Some("Q4_K_M".to_string()),
                    profile,
                    "Larger model enabled by sufficient VRAM",
                ));
            }
        }

        recs
    }

    fn create_recommendation(
        model: &str,
        size: &str,
        backend: Backend,
        quantization: Option<String>,
        profile: &SystemProfile,
        base_reasoning: &str,
    ) -> ModelRecommendation {
        let gpu_desc = if let Some(gpu) = &profile.gpu {
            format!("{} GPU acceleration", gpu.vendor)
        } else {
            "CPU-only".to_string()
        };

        let reasoning = format!(
            "{}. Based on {} MB RAM, {} backend, and {}",
            base_reasoning, profile.memory.total_mb, backend, gpu_desc
        );

        let estimated_memory_mb = Self::estimate_memory(size, quantization.as_deref());

        ModelRecommendation {
            model_name: model.to_string(),
            model_size: size.to_string(),
            backend,
            quantization,
            reasoning,
            estimated_memory_mb,
        }
    }

    fn estimate_memory(size: &str, quantization: Option<&str>) -> u64 {
        // Rough estimates based on model size and quantization
        let base = match size {
            "1.1B" => 1024,
            "2.7B" => 3072,
            "7B" => 14336,
            "13B" => 26624,
            _ => 4096,
        };

        if quantization.is_some() {
            base / 2 // Rough halving for quantized models
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::{CPUInfo, GPUInfo, MemoryInfo, PlatformInfo};

    fn make_profile(ram_mb: u64, gpu: Option<GPUInfo>) -> SystemProfile {
        SystemProfile {
            cpu: CPUInfo {
                architecture: "x86_64".to_string(),
                cores: 8,
                model_name: "Test CPU".to_string(),
                frequency_mhz: Some(3000),
            },
            memory: MemoryInfo {
                total_mb: ram_mb,
                available_mb: ram_mb / 2,
            },
            gpu,
            platform: PlatformInfo {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        }
    }

    fn apple_gpu() -> GPUInfo {
        GPUInfo {
            vendor: GPUVendor::Apple,
            model: "M1".to_string(),
            vram_mb: Some(16384),
        }
    }

    fn nvidia_gpu(vram_mb: u64) -> GPUInfo {
        GPUInfo {
            vendor: GPUVendor::NVIDIA,
            model: "RTX 3090".to_string(),
            vram_mb: Some(vram_mb),
        }
    }

    // --- HardwareTier tests ---

    #[test]
    fn test_tier_low_end() {
        let profile = make_profile(4096, None);
        assert_eq!(HardwareTier::from_profile(&profile), HardwareTier::LowEnd);
    }

    #[test]
    fn test_tier_mid_range_by_ram() {
        let profile = make_profile(16384, None);
        assert_eq!(
            HardwareTier::from_profile(&profile),
            HardwareTier::MidRange
        );
    }

    #[test]
    fn test_tier_high_end() {
        let profile = make_profile(32768, Some(nvidia_gpu(24576)));
        assert_eq!(HardwareTier::from_profile(&profile), HardwareTier::HighEnd);
    }

    #[test]
    fn test_tier_high_ram_no_gpu_is_mid() {
        // 32GB RAM but no dedicated GPU -> MidRange
        let profile = make_profile(32768, None);
        assert_eq!(
            HardwareTier::from_profile(&profile),
            HardwareTier::MidRange
        );
    }

    #[test]
    fn test_tier_high_ram_apple_gpu_is_mid() {
        // Apple GPU isn't NVIDIA/AMD, so doesn't count as "dedicated" for tier
        let profile = make_profile(32768, Some(apple_gpu()));
        assert_eq!(
            HardwareTier::from_profile(&profile),
            HardwareTier::MidRange
        );
    }

    // --- Backend selection tests ---

    #[test]
    fn test_backend_apple_selects_mlx() {
        let profile = make_profile(16384, Some(apple_gpu()));
        assert_eq!(Recommender::select_backend(&profile), Backend::MLX);
    }

    #[test]
    fn test_backend_nvidia_selects_cuda() {
        let profile = make_profile(16384, Some(nvidia_gpu(8192)));
        assert_eq!(Recommender::select_backend(&profile), Backend::CUDA);
    }

    #[test]
    fn test_backend_no_gpu_selects_cpu() {
        let profile = make_profile(16384, None);
        assert_eq!(Recommender::select_backend(&profile), Backend::CPU);
    }

    // --- Recommendation output tests ---

    #[test]
    fn test_recommend_returns_non_empty() {
        let profile = make_profile(8192, None);
        let recs = Recommender::recommend(&profile);
        assert!(!recs.is_empty());
    }

    #[test]
    fn test_recommend_low_end_includes_small_models() {
        let profile = make_profile(4096, None);
        let recs = Recommender::recommend(&profile);
        assert!(recs.iter().any(|r| r.model_name == "TinyLlama"));
    }

    #[test]
    fn test_recommend_mid_range_includes_mistral() {
        let profile = make_profile(12288, None);
        let recs = Recommender::recommend(&profile);
        assert!(recs.iter().any(|r| r.model_name == "Mistral 7B"));
    }

    #[test]
    fn test_recommend_high_end_with_large_vram_includes_13b() {
        let profile = make_profile(32768, Some(nvidia_gpu(24576)));
        let recs = Recommender::recommend(&profile);
        assert!(recs.iter().any(|r| r.model_name == "Llama 2 13B"));
    }

    #[test]
    fn test_recommend_high_end_small_vram_no_13b() {
        let profile = make_profile(32768, Some(nvidia_gpu(8192)));
        let recs = Recommender::recommend(&profile);
        assert!(!recs.iter().any(|r| r.model_name == "Llama 2 13B"));
    }

    #[test]
    fn test_quantized_uses_less_memory() {
        let full = Recommender::estimate_memory("7B", None);
        let quantized = Recommender::estimate_memory("7B", Some("Q4_K_M"));
        assert!(quantized < full);
    }

    #[test]
    fn test_recommendation_has_reasoning() {
        let profile = make_profile(8192, None);
        let recs = Recommender::recommend(&profile);
        for rec in &recs {
            assert!(!rec.reasoning.is_empty());
            assert!(rec.reasoning.contains("RAM"));
        }
    }
}
