use caro::assessment::*;

pub fn mock_apple_silicon() -> SystemProfile {
    SystemProfile {
        cpu: CPUInfo {
            architecture: "arm64".to_string(),
            cores: 8,
            model_name: "Apple M1".to_string(),
            frequency_mhz: Some(3200),
        },
        memory: MemoryInfo {
            total_mb: 16384,
            available_mb: 8192,
        },
        gpu: Some(GPUInfo {
            vendor: GPUVendor::Apple,
            model: "Apple M1".to_string(),
            vram_mb: None, // Unified memory
        }),
        platform: PlatformInfo {
            os: "macos".to_string(),
            arch: "arm64".to_string(),
        },
    }
}

pub fn mock_nvidia_linux() -> SystemProfile {
    SystemProfile {
        cpu: CPUInfo {
            architecture: "x86_64".to_string(),
            cores: 12,
            model_name: "AMD Ryzen 5 5600X".to_string(),
            frequency_mhz: Some(3700),
        },
        memory: MemoryInfo {
            total_mb: 32768,
            available_mb: 24576,
        },
        gpu: Some(GPUInfo {
            vendor: GPUVendor::NVIDIA,
            model: "NVIDIA GeForce RTX 3080".to_string(),
            vram_mb: Some(10240),
        }),
        platform: PlatformInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        },
    }
}

pub fn mock_low_end() -> SystemProfile {
    SystemProfile {
        cpu: CPUInfo {
            architecture: "x86_64".to_string(),
            cores: 4,
            model_name: "Intel Core i5-8250U".to_string(),
            frequency_mhz: Some(1600),
        },
        memory: MemoryInfo {
            total_mb: 4096,
            available_mb: 2048,
        },
        gpu: None,
        platform: PlatformInfo {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
        },
    }
}
