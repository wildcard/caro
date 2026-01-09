---
work_package_id: "WP10"
subtasks:
  - "T079"
  - "T080"
  - "T081"
  - "T082"
  - "T083"
  - "T084"
  - "T085"
  - "T086"
  - "T087"
  - "T088"
title: "Multi-Backend Consistency"
phase: "Phase 3 - P2 Features"
lane: "planned"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-09T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP10 – Multi-Backend Consistency

## ⚠️ Priority Notice

**This is a P2 feature** (User Story 2) and is **optional for v1.1.0 MVP**.

The v1.1.0 milestone can be completed with WP01-WP09 alone. WP10 adds multi-backend consistency testing, which is valuable for validating LLM quality across different inference engines but not critical for the initial release.

**Defer this work package** if time is constrained for v1.1.0 deadline (Feb 15, 2026).

## Objectives & Success Criteria

**Goal**: Add vLLM and Ollama backend support; implement consistency testing.

**Success Criteria**:
- Evaluation harness supports MLX, vLLM, and Ollama backends
- Backend selection configurable (env var or CLI argument)
- Consistency test generates commands across all available backends
- Command similarity score calculated (≥95% target)
- Divergence reported when backends disagree
- Unavailable backends handled gracefully (skip with warning)
- Integration test validates multi-backend consistency

## Context & Constraints

**References**:
- [spec.md](../../spec.md) - User Story 2 (lines 73-86) for multi-backend validation requirements
- [plan.md](../../plan.md) - Backend integration strategy (lines 277-301)
- Existing backends: `caro::backends::{mlx, vllm, ollama}`

**User Story 2** (from spec.md):
> As a caro maintainer, I need to validate that command generation quality is consistent across MLX, vLLM, and Ollama backends so I can confidently recommend any backend to users.
>
> **Acceptance**:
> - Run same prompt through MLX, vLLM, Ollama
> - Calculate command similarity (semantic equivalence or edit distance)
> - Report divergence when similarity < 95%
> - Document backend-specific quirks discovered

**Constraints**:
- Backend availability varies (not all users have all backends installed)
- Make consistency testing optional (skip unavailable backends)
- Default to MLX-only for MVP (WP05 implementation)

## Subtasks & Detailed Guidance

### T079-T080 – Backend Integration

Extend `harness.rs` to support vLLM and Ollama:

```rust
use caro::backends::{BackendTrait, mlx::MlxBackend, vllm::VllmBackend, ollama::OllamaBackend};

#[derive(Debug, Clone)]
pub enum Backend {
    Mlx,
    Vllm,
    Ollama,
}

impl Backend {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "mlx" => Ok(Backend::Mlx),
            "vllm" => Ok(Backend::Vllm),
            "ollama" => Ok(Backend::Ollama),
            _ => Err(format!("Unknown backend: {}", s)),
        }
    }

    pub async fn initialize(&self) -> Result<Box<dyn BackendTrait>, String> {
        match self {
            Backend::Mlx => Ok(Box::new(
                MlxBackend::new()
                    .map_err(|e| format!("Failed to initialize MLX: {}", e))?
            )),
            Backend::Vllm => Ok(Box::new(
                VllmBackend::new()
                    .map_err(|e| format!("Failed to initialize vLLM: {}", e))?
            )),
            Backend::Ollama => Ok(Box::new(
                OllamaBackend::new()
                    .map_err(|e| format!("Failed to initialize Ollama: {}", e))?
            )),
        }
    }
}
```

### T081 – Backend Selector Logic

Add configuration support in `harness.rs`:

```rust
use std::env;

/// Get backend from environment variable or default to MLX
pub fn get_selected_backend() -> Backend {
    env::var("CARO_EVAL_BACKEND")
        .ok()
        .and_then(|s| Backend::from_str(&s).ok())
        .unwrap_or(Backend::Mlx)
}
```

Update `run_evaluation()` to use configurable backend:

```rust
pub async fn run_evaluation(dataset_path: &Path) -> Result<Vec<TestResult>, String> {
    let dataset = TestDataset::from_toml(dataset_path)?;

    // Use configurable backend instead of hardcoded MLX
    let selected_backend = get_selected_backend();
    let backend = selected_backend.initialize().await?;

    let mut results = Vec::new();

    for test_case in &dataset.test_cases {
        let result = execute_test_case(backend.as_ref(), test_case).await;
        results.push(result);
    }

    Ok(results)
}
```

### T082-T085 – Consistency Testing

Add consistency test function in `harness.rs`:

```rust
/// Run consistency test across multiple backends
///
/// Generates commands for the same prompts using MLX, vLLM, and Ollama,
/// then calculates similarity scores to detect divergence.
pub async fn run_consistency_test(
    dataset_path: &Path,
) -> Result<ConsistencyReport, String> {
    let dataset = TestDataset::from_toml(dataset_path)?;

    // Check which backends are available
    let available_backends = check_backend_availability().await;

    if available_backends.len() < 2 {
        return Err(format!(
            "Consistency testing requires at least 2 backends. Available: {:?}",
            available_backends
        ));
    }

    let mut divergences = Vec::new();

    for test_case in &dataset.test_cases {
        let mut commands = HashMap::new();

        // Generate command with each backend
        for backend in &available_backends {
            let backend_impl = backend.initialize().await?;

            match timeout(
                Duration::from_secs(30),
                backend_impl.generate_command(&test_case.prompt)
            ).await {
                Ok(Ok(cmd)) => {
                    commands.insert(format!("{:?}", backend), cmd);
                }
                Ok(Err(e)) => {
                    eprintln!("Backend {:?} error for {}: {}", backend, test_case.id, e);
                }
                Err(_) => {
                    eprintln!("Backend {:?} timeout for {}", backend, test_case.id);
                }
            }
        }

        // Calculate similarity across backends
        if commands.len() >= 2 {
            let similarity = calculate_command_similarity(&commands);

            if similarity < 0.95 {
                divergences.push(Divergence {
                    test_id: test_case.id.clone(),
                    prompt: test_case.prompt.clone(),
                    commands,
                    similarity,
                });
            }
        }
    }

    Ok(ConsistencyReport {
        available_backends,
        total_tests: dataset.test_cases.len(),
        divergences,
    })
}

async fn check_backend_availability() -> Vec<Backend> {
    let mut available = Vec::new();

    for backend in [Backend::Mlx, Backend::Vllm, Backend::Ollama] {
        if backend.initialize().await.is_ok() {
            available.push(backend);
        }
    }

    available
}

fn calculate_command_similarity(commands: &HashMap<String, String>) -> f64 {
    // Use semantic equivalence (via normalization) instead of edit distance
    use crate::validators::normalize_command;

    let normalized: Vec<_> = commands
        .values()
        .map(|cmd| normalize_command(cmd))
        .collect();

    if normalized.is_empty() {
        return 0.0;
    }

    // Check if all normalized commands are identical
    let first = &normalized[0];
    let all_identical = normalized.iter().all(|cmd| cmd == first);

    if all_identical {
        1.0  // Perfect similarity
    } else {
        // Calculate pairwise similarity using Levenshtein distance
        // For simplicity, return 0.0 if any divergence detected
        // (Proper implementation would calculate average edit distance)
        0.0
    }
}

#[derive(Debug)]
pub struct ConsistencyReport {
    pub available_backends: Vec<Backend>,
    pub total_tests: usize,
    pub divergences: Vec<Divergence>,
}

#[derive(Debug)]
pub struct Divergence {
    pub test_id: String,
    pub prompt: String,
    pub commands: HashMap<String, String>,  // backend → command
    pub similarity: f64,
}
```

### T086 – Graceful Backend Handling (already implemented above)

The `check_backend_availability()` function already handles unavailable backends gracefully by skipping initialization errors.

### T087 – Integration Test

Add to `tests/evaluation.rs`:

```rust
#[tokio::test]
#[ignore]  // Requires all backends installed
async fn test_multi_backend_consistency() {
    let dataset_path = std::path::Path::new("tests/evaluation/test_cases.toml");

    let report = evaluation::harness::run_consistency_test(dataset_path)
        .await
        .expect("Consistency test should complete");

    println!("Available backends: {:?}", report.available_backends);
    println!("Total tests: {}", report.total_tests);
    println!("Divergences: {}", report.divergences.len());

    for divergence in &report.divergences {
        println!("\nDivergence in test: {}", divergence.test_id);
        println!("Prompt: {}", divergence.prompt);
        println!("Similarity: {:.2}%", divergence.similarity * 100.0);
        for (backend, command) in &divergence.commands {
            println!("  {}: {}", backend, command);
        }
    }

    // For CI, require ≥95% similarity across all tests
    let divergence_rate = report.divergences.len() as f64 / report.total_tests as f64;
    assert!(
        divergence_rate < 0.05,
        "Too many divergences: {:.1}%",
        divergence_rate * 100.0
    );
}
```

### T088 – Add Consistency Metrics to EvaluationResult

Extend `EvaluationResult` struct in `harness.rs`:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluationResult {
    // ... existing fields ...

    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<ConsistencyMetrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsistencyMetrics {
    pub backends_tested: Vec<String>,
    pub total_comparisons: usize,
    pub divergences: usize,
    pub similarity_rate: f64,  // (total - divergences) / total
}
```

## Definition of Done Checklist

- [ ] vLLM backend integration works
- [ ] Ollama backend integration works
- [ ] Backend selection via CARO_EVAL_BACKEND env var
- [ ] `run_consistency_test()` generates commands across backends
- [ ] Similarity calculation detects divergence (< 95%)
- [ ] Unavailable backends skipped with warning
- [ ] Integration test validates consistency (marked #[ignore] for CI)
- [ ] ConsistencyMetrics added to EvaluationResult (optional field)

## Usage

**Run evaluation with specific backend**:
```bash
CARO_EVAL_BACKEND=vllm cargo test --test evaluation
CARO_EVAL_BACKEND=ollama cargo test --test evaluation
```

**Run consistency test** (requires all backends):
```bash
cargo test test_multi_backend_consistency -- --ignored --nocapture
```

## Activity Log

- 2026-01-09T00:00:00Z – system – shell_pid= – lane=planned – Prompt created
