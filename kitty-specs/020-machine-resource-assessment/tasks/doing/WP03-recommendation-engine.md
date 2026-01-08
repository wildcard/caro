---
work_package_id: WP03
title: "Recommendation Engine"
priority: P2
phase: "core"
subtasks: [T015, T016, T017, T018, T019]
lane: "doing"
review_status: ""
reviewed_by: ""
assignee: ""
agent: "claude"
shell_pid: "71871"
history:
  - 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated work package prompt
---

# Work Package 03: Recommendation Engine

## Objective

Build the recommendation engine that maps hardware profiles to optimal model configurations. This transforms raw hardware data into actionable guidance for users, suggesting appropriate models, backends, and quantization levels based on detected capabilities.

## Context

**User Story**: P2 - Get Model Recommendations (All 5 acceptance scenarios)

**Why This Matters**: Users need guidance on which models will run well on their hardware. Without recommendations, they must manually research model requirements and may choose poorly-optimized configurations.

## Implementation Guidance

### T015: Define Hardware Tiers

**Location**: `src/assessment/recommender.rs`

Define hardware profile tiers:
```rust
#[derive(Debug, Clone, Copy)]
enum HardwareTier {
    LowEnd,    // < 8GB RAM, no GPU
    MidRange,  // 8-16GB RAM, integrated or no GPU
    HighEnd,   // > 16GB RAM, dedicated GPU
}

impl HardwareTier {
    fn from_profile(profile: &SystemProfile) -> Self {
        let has_dedicated_gpu = profile.gpu
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
```

### T016: Create ModelRecommendation Struct

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub model_name: String,
    pub model_size: String,  // e.g., "2.7B", "7B"
    pub backend: Backend,
    pub quantization: Option<String>,
    pub reasoning: String,
    pub estimated_memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backend {
    MLX,
    CUDA,
    CPU,
}
```

### T017: Implement Recommendation Algorithm

```rust
pub struct Recommender;

impl Recommender {
    pub fn recommend(profile: &SystemProfile) -> Vec<ModelRecommendation> {
        let tier = HardwareTier::from_profile(profile);
        let backend = Self::select_backend(profile);

        match tier {
            HardwareTier::LowEnd => Self::low_end_recommendations(backend),
            HardwareTier::MidRange => Self::mid_range_recommendations(backend),
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
}
```

### T018: Add Model Catalog

Define model configurations:
```rust
impl Recommender {
    fn low_end_recommendations(backend: Backend) -> Vec<ModelRecommendation> {
        vec![
            ModelRecommendation {
                model_name: "Phi-2".to_string(),
                model_size: "2.7B".to_string(),
                backend: backend.clone(),
                quantization: Some("Q4_K_M".to_string()),
                reasoning: "Lightweight model optimized for systems with limited RAM".to_string(),
                estimated_memory_mb: 2048,
            },
            ModelRecommendation {
                model_name: "TinyLlama".to_string(),
                model_size: "1.1B".to_string(),
                backend,
                quantization: Some("Q4_K_M".to_string()),
                reasoning: "Very small model for extremely constrained systems".to_string(),
                estimated_memory_mb: 1024,
            },
        ]
    }

    fn mid_range_recommendations(backend: Backend) -> Vec<ModelRecommendation> {
        vec![
            ModelRecommendation {
                model_name: "Phi-2".to_string(),
                model_size: "2.7B".to_string(),
                backend: backend.clone(),
                quantization: None,
                reasoning: "Efficient model for balanced performance and quality".to_string(),
                estimated_memory_mb: 3072,
            },
            ModelRecommendation {
                model_name: "Mistral 7B".to_string(),
                model_size: "7B".to_string(),
                backend,
                quantization: Some("Q4_K_M".to_string()),
                reasoning: "High-quality model with moderate resource requirements".to_string(),
                estimated_memory_mb: 4096,
            },
        ]
    }

    fn high_end_recommendations(profile: &SystemProfile, backend: Backend) -> Vec<ModelRecommendation> {
        let mut recs = vec![
            ModelRecommendation {
                model_name: "Mistral 7B".to_string(),
                model_size: "7B".to_string(),
                backend: backend.clone(),
                quantization: None,
                reasoning: "Full precision model for optimal quality".to_string(),
                estimated_memory_mb: 14336,
            },
        ];

        // Add larger models if VRAM available
        if let Some(gpu) = &profile.gpu {
            if gpu.vram_mb.map(|v| v >= 16384).unwrap_or(false) {
                recs.push(ModelRecommendation {
                    model_name: "Llama 2 13B".to_string(),
                    model_size: "13B".to_string(),
                    backend,
                    quantization: Some("Q4_K_M".to_string()),
                    reasoning: "Larger model enabled by sufficient VRAM".to_string(),
                    estimated_memory_mb: 8192,
                });
            }
        }

        recs
    }
}
```

### T019: Generate Reasoning Text

Enhance reasoning with profile-specific details:
```rust
impl Recommender {
    fn create_recommendation(
        model: &str,
        size: &str,
        backend: Backend,
        quantization: Option<String>,
        profile: &SystemProfile,
    ) -> ModelRecommendation {
        let reasoning = format!(
            "Based on {} MB RAM, {} backend, and {}",
            profile.memory.total_mb,
            backend.to_string(),
            if profile.gpu.is_some() { "GPU acceleration" } else { "CPU-only" }
        );

        ModelRecommendation {
            model_name: model.to_string(),
            model_size: size.to_string(),
            backend,
            quantization,
            reasoning,
            estimated_memory_mb: Self::estimate_memory(size, quantization.as_deref()),
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
            base / 2  // Rough halving for quantized models
        } else {
            base
        }
    }
}
```

## Definition of Done

- [ ] Hardware tiers defined (low/mid/high-end)
- [ ] Backend selection logic implemented (MLX/CUDA/CPU)
- [ ] Model catalog includes Phi, Llama, Mistral families
- [ ] Recommendations match tier and backend appropriately
- [ ] Reasoning text includes profile-specific details
- [ ] Output shows recommendations with clear formatting
- [ ] Low-end systems recommend lightweight models
- [ ] Apple Silicon systems recommend MLX backend
- [ ] NVIDIA systems recommend CUDA backend

## Reviewer Guidance

Verify:
1. Tier thresholds make sense (8GB, 16GB boundaries)
2. Backend selection handles all GPU vendors
3. Model catalog covers range of hardware profiles
4. Reasoning text is helpful and specific
5. Memory estimates are reasonable

## Activity Log

- 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated initial work package prompt
- 2026-01-08T18:40:45Z – claude – shell_pid=71871 – lane=doing – Starting recommendation engine implementation
