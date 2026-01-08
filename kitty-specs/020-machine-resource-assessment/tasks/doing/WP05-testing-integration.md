---
work_package_id: WP05
title: "Testing & Integration"
priority: P3
phase: "polish"
subtasks: [T027, T028, T029, T030, T031, T032]
lane: "doing"
review_status: ""
reviewed_by: ""
assignee: ""
agent: "claude"
shell_pid: "80573"
history:
  - 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated initial work package prompt
---

# Work Package 05: Testing & Integration

## Objective

Add comprehensive test coverage and integration validation. This work package ensures the feature works reliably across platforms and provides documentation for users.

## Context

**User Story**: Quality assurance across all P1-P3 stories

**Why This Matters**: Testing prevents regressions and ensures the feature works as expected on different hardware configurations. Documentation helps users understand and use the feature effectively.

## Implementation Guidance

### T027: Create Mock System Profiles

**Location**: `tests/fixtures/mock_profiles.rs`

```rust
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
            vram_mb: None,  // Unified memory
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
```

### T028: Write CPU Detection Tests [P]

**Location**: `tests/assessment_tests.rs`

```rust
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
```

### T029: Write Memory Detection Tests [P]

```rust
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

        // Most systems should have at least 1GB
        assert!(memory.total_mb >= 1024, "Total memory seems too low");
        // Available should be at least some fraction of total
        assert!(
            memory.available_mb >= memory.total_mb / 10,
            "Available memory seems suspiciously low"
        );
    }
}
```

### T030: Write Recommendation Algorithm Tests [P]

```rust
#[cfg(test)]
mod recommendation_tests {
    use caro::assessment::Recommender;
    use tests::fixtures::mock_profiles::*;

    #[test]
    fn test_apple_silicon_recommendations() {
        let profile = mock_apple_silicon();
        let recommendations = Recommender::recommend(&profile);

        assert!(!recommendations.is_empty(), "Should have recommendations");

        // Apple Silicon should recommend MLX backend
        let mlx_count = recommendations.iter()
            .filter(|r| matches!(r.backend, Backend::MLX))
            .count();
        assert!(mlx_count > 0, "Apple Silicon should recommend MLX backend");
    }

    #[test]
    fn test_nvidia_recommendations() {
        let profile = mock_nvidia_linux();
        let recommendations = Recommender::recommend(&profile);

        // NVIDIA GPU should recommend CUDA backend
        let cuda_count = recommendations.iter()
            .filter(|r| matches!(r.backend, Backend::CUDA))
            .count();
        assert!(cuda_count > 0, "NVIDIA GPU should recommend CUDA backend");
    }

    #[test]
    fn test_low_end_recommendations() {
        let profile = mock_low_end();
        let recommendations = Recommender::recommend(&profile);

        // Low-end system should recommend lightweight models
        let lightweight_count = recommendations.iter()
            .filter(|r| {
                r.model_name.contains("Phi-2") || r.model_name.contains("TinyLlama")
            })
            .count();
        assert!(
            lightweight_count > 0,
            "Low-end system should recommend lightweight models"
        );

        // Should not recommend large models
        let has_large_model = recommendations.iter()
            .any(|r| r.model_name.contains("70B") || r.model_name.contains("65B"));
        assert!(
            !has_large_model,
            "Low-end system should not recommend large models"
        );
    }
}
```

### T031: Write Integration Test

**Location**: `tests/integration/assess_command_test.rs`

```rust
use assert_cmd::Command;

#[test]
fn test_assess_command_runs() {
    let mut cmd = Command::cargo_bin("caro").unwrap();

    cmd.arg("assess")
        .assert()
        .success();
}

#[test]
fn test_assess_command_output() {
    let mut cmd = Command::cargo_bin("caro").unwrap();

    let output = cmd.arg("assess")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify output contains expected sections
    assert!(stdout.contains("Caro System Assessment"));
    assert!(stdout.contains("System Information:"));
    assert!(stdout.contains("CPU:"));
    assert!(stdout.contains("Memory:"));
}

#[test]
fn test_assess_json_export() {
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut cmd = Command::cargo_bin("caro").unwrap();

    cmd.arg("assess")
        .arg("--export")
        .arg("json")
        .arg("--output")
        .arg(path)
        .assert()
        .success();

    // Verify JSON file was created and is valid
    let content = std::fs::read_to_string(path).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content)
        .expect("Output should be valid JSON");
}

#[test]
fn test_assess_completes_quickly() {
    use std::time::Instant;

    let mut cmd = Command::cargo_bin("caro").unwrap();

    let start = Instant::now();
    cmd.arg("assess")
        .assert()
        .success();
    let duration = start.elapsed();

    // Should complete in < 5 seconds (SC-001)
    assert!(
        duration.as_secs() < 5,
        "Assessment took {:?}, expected < 5s",
        duration
    );
}
```

### T032: Add Documentation

**Location**: `README.md`

Add section to README:
```markdown
### System Assessment

Caro can assess your system's hardware capabilities and recommend optimal model configurations:

```bash
# Basic assessment
caro assess

# Export to JSON
caro assess --export json --output assessment.json

# Export to Markdown
caro assess --export markdown --output assessment.md
```

The assessment command detects:
- CPU architecture, cores, and model
- Total and available memory (RAM)
- GPU vendor, model, and VRAM (if available)
- Recommended models and backends based on your hardware

**Supported Platforms**: macOS, Linux, Windows

**Backends**: MLX (Apple Silicon), CUDA (NVIDIA), CPU-only

For more details, run `caro assess --help`.
```

## Definition of Done

- [ ] Mock profiles created for Apple Silicon, NVIDIA, low-end systems
- [ ] CPU detection tests pass
- [ ] Memory detection tests pass
- [ ] Recommendation algorithm tests pass
- [ ] Integration test validates end-to-end workflow
- [ ] Integration test verifies < 5 second completion (SC-001)
- [ ] JSON export test validates output is valid JSON
- [ ] README.md documents `caro assess` command
- [ ] All tests pass on CI/CD (macOS at minimum)

## Test Execution

Run tests:
```bash
cargo test --test assessment_tests
cargo test --test assess_command_test
```

Expected output:
```
running 10 tests
test cpu_tests::test_cpu_detection ... ok
test memory_tests::test_memory_detection ... ok
test recommendation_tests::test_apple_silicon_recommendations ... ok
...
test result: ok. 10 passed; 0 failed
```

## Risks & Mitigation

**Risk**: Platform-specific tests may fail on other OSes
- **Mitigation**: Use mock profiles for unit tests, mark integration tests with platform attributes

**Risk**: Integration tests may be flaky on CI
- **Mitigation**: Add retries, increase timeout thresholds

## Reviewer Guidance

Verify:
1. All tests are hermetic (don't depend on specific hardware)
2. Mock profiles cover realistic hardware configurations
3. Integration tests validate user-facing behavior
4. Documentation is clear and accurate
5. Test coverage includes edge cases (no GPU, low memory)

## Activity Log

- 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated initial work package prompt
- 2026-01-08T18:51:37Z – claude – shell_pid=80573 – lane=doing – Started implementation
