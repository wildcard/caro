mod fixtures;

#[cfg(test)]
mod cpu_tests {
    use caro::assessment::CPUInfo;

    #[test]
    fn test_cpu_detection() {
        let cpu = CPUInfo::detect().expect("CPU detection should succeed");

        assert!(!cpu.architecture.is_empty(), "Architecture should be detected");
        assert!(cpu.cores > 0, "Core count should be positive");
        assert!(!cpu.model_name.is_empty(), "Model name should be detected");
    }

    #[test]
    fn test_cpu_architecture_valid() {
        let cpu = CPUInfo::detect().unwrap();

        let valid_archs = ["x86_64", "aarch64", "arm64"];
        assert!(
            valid_archs.contains(&cpu.architecture.as_str()),
            "Architecture should be one of: {:?}",
            valid_archs
        );
    }
}

#[cfg(test)]
mod memory_tests {
    use caro::assessment::MemoryInfo;

    #[test]
    fn test_memory_detection() {
        let memory = MemoryInfo::detect().expect("Memory detection should succeed");

        assert!(memory.total_mb > 0, "Total memory should be positive");
        assert!(memory.available_mb > 0, "Available memory should be positive");
        assert!(
            memory.available_mb <= memory.total_mb,
            "Available memory should not exceed total"
        );
    }

    #[test]
    fn test_memory_reasonable_values() {
        let memory = MemoryInfo::detect().unwrap();

        // Most systems should have at least 1GB total
        assert!(memory.total_mb >= 1024, "Total memory seems too low");
        // Available memory should be reported (even if system is heavily loaded)
        // We don't enforce a minimum percentage as systems can be legitimately low on memory
        assert!(
            memory.available_mb < memory.total_mb,
            "Available memory should not exceed or equal total"
        );
    }
}

#[cfg(test)]
mod recommendation_tests {
    use caro::assessment::{Backend, Recommender};
    use crate::fixtures::mock_profiles::*;

    #[test]
    fn test_apple_silicon_recommendations() {
        let profile = mock_apple_silicon();
        let recommendations = Recommender::recommend(&profile);

        assert!(!recommendations.is_empty(), "Should have recommendations");

        // Apple Silicon should recommend MLX backend
        let mlx_count = recommendations
            .iter()
            .filter(|r| matches!(r.backend, Backend::MLX))
            .count();
        assert!(mlx_count > 0, "Apple Silicon should recommend MLX backend");
    }

    #[test]
    fn test_nvidia_recommendations() {
        let profile = mock_nvidia_linux();
        let recommendations = Recommender::recommend(&profile);

        // NVIDIA GPU should recommend CUDA backend
        let cuda_count = recommendations
            .iter()
            .filter(|r| matches!(r.backend, Backend::CUDA))
            .count();
        assert!(cuda_count > 0, "NVIDIA GPU should recommend CUDA backend");
    }

    #[test]
    fn test_low_end_recommendations() {
        let profile = mock_low_end();
        let recommendations = Recommender::recommend(&profile);

        // Low-end system should recommend lightweight models
        let lightweight_count = recommendations
            .iter()
            .filter(|r| r.model_name.contains("Phi-2") || r.model_name.contains("TinyLlama"))
            .count();
        assert!(
            lightweight_count > 0,
            "Low-end system should recommend lightweight models"
        );

        // Should not recommend large models
        let has_large_model = recommendations
            .iter()
            .any(|r| r.model_name.contains("70B") || r.model_name.contains("65B"));
        assert!(
            !has_large_model,
            "Low-end system should not recommend large models"
        );
    }
}
