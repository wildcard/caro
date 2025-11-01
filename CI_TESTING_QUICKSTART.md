# CI Inference Testing - Quick Start Guide

**See full plan**: [CI_INFERENCE_TESTING_PLAN.md](CI_INFERENCE_TESTING_PLAN.md)

---

## Overview

**Goal**: Test cmdai with real local LLM inference in CI environments
**Strategy**: Hybrid approach - keep fast mocked tests, add optional real inference tests
**Effort**: 3-4 weeks implementation
**Status**: Planning complete, ready for implementation (post-v1.0)

---

## Key Points

### Current State (v1.0)
- ✅ All tests use **mocked/simulated inference** (fast, deterministic)
- ✅ No actual Candle or MLX integration yet
- ❌ Cannot validate real command generation quality
- ❌ No performance validation with real models

### Proposed Architecture

```
Fast CI (Existing - Keep)          Slow CI (NEW - Add after v1.0)
├── Mocked inference               ├── Real CPU inference (nightly)
├── 15-30 min runtime              ├── Real MLX inference (weekly)
├── Runs on every PR               ├── 45-90 min runtime
└── Tests: CLI, safety, config     └── Tests: Quality, performance
```

### Testing Tiers

| Tier | When | Duration | What |
|------|------|----------|------|
| T1: Unit | Every commit | 5-10 min | Fast mocked tests |
| T2: Integration | Every PR | 15-30 min | E2E with mocks |
| T3: CPU Inference | Nightly | 45-60 min | 0.5B model quality checks |
| T4: MLX Inference | Weekly | 30-45 min | Apple Silicon performance |
| T5: Full Stack | Pre-release | 90-120 min | All backends + stress tests |

---

## Model Selection for CI

**For fast CI testing** (0.5B model):
```
Model: Qwen2.5-Coder-0.5B-Instruct-Q4_K_S
Size: ~300MB
Inference: 2-3s on 2 CPUs
Use: PR validation, quick feedback
```

**For quality validation** (1.5B model):
```
Model: Qwen2.5-Coder-1.5B-Instruct-Q4_K_M
Size: ~1.1GB
Inference: 5-8s on 2 CPUs, <2s on Apple Silicon
Use: Nightly quality checks, production validation
```

---

## Implementation Phases

### Phase 1: Foundation (Week 1) - 16-24 hours
**Goal**: Real CPU inference working

- [ ] Implement real Candle backend integration
- [ ] Add model download utility
- [ ] Create test fixtures framework
- [ ] Add `slow-tests` feature flag
- [ ] Write first real inference test

**Test**: `cargo test --features slow-tests -- --ignored`

---

### Phase 2: CI Integration (Week 2) - 12-16 hours
**Goal**: Inference tests running in CI with caching

- [ ] Create nightly inference workflow
- [ ] Add GitHub Actions model caching
- [ ] Add model download script
- [ ] Configure test execution
- [ ] Add quality report generation

**Workflow**: `.github/workflows/inference-tests.yml`

---

### Phase 3: MLX & Platform Testing (Week 3) - 16-20 hours
**Goal**: GPU inference on macOS

- [ ] Implement real MLX backend
- [ ] Add macOS-specific inference tests
- [ ] Create MLX workflow for Apple Silicon
- [ ] Add performance benchmarks
- [ ] Configure conditional execution

**Test**: `cargo test --features slow-tests,embedded-mlx -- test_mlx`

---

### Phase 4: Remote Backend Testing (Week 4) - 12-16 hours
**Goal**: Ollama and vLLM integration with containers

- [ ] Create Docker Compose for Ollama
- [ ] Add remote backend integration tests
- [ ] Add remote backend workflow
- [ ] Test fallback scenarios
- [ ] Add network simulation tests

**Test**: `cargo test --features remote-tests -- --ignored`

---

## Key CI Features

### Model Caching
```yaml
- uses: actions/cache@v3
  with:
    path: ~/.cache/cmdai/models
    key: v1-models-${{ hashFiles('src/backends/embedded/config.rs') }}
```

**Impact**:
- First run: 2-3 min download + inference
- Cached runs: 10-20s restore + inference

### Quality Validation
```rust
// Validate command quality against fixtures
#[test]
#[ignore]
#[cfg(feature = "slow-tests")]
async fn test_command_quality_against_fixtures() {
    // Load test cases from YAML
    // Run real inference
    // Validate patterns match expected output
    // Generate quality report
}
```

### Performance Budgets
```
CPU Inference (0.5B): Target 2-3s, Fail >8s
CPU Inference (1.5B): Target 5-8s, Fail >15s
MLX Inference (0.5B): Target 0.5-1s, Fail >3s
MLX Inference (1.5B): Target 1-2s, Fail >5s
```

---

## Cost Analysis

### GitHub Actions (Free Tier)
```
Fast CI:          750 min/month Linux (existing)
CPU Inference:    900 min/month Linux (nightly)
MLX Inference:     80 min/month macOS (weekly)
Remote Backends:   80 min/month Linux (weekly)
────────────────────────────────────────────────
Total:          1,810 min Linux, 80 min macOS
Status:         Within free tier ✅
```

**All within free tier for public repos!**

Paid usage only needed if:
- Exceeding 2,000 min/month Linux
- Exceeding 100 actual min/month macOS (1,000 with 10x multiplier)

---

## Quick Commands

### Run Inference Tests Locally
```bash
# Setup (first time)
./scripts/setup-inference-model.sh

# Run all inference tests
cargo test --features slow-tests -- --ignored

# Run CPU tests only
cargo test --features slow-tests -- --ignored test_cpu

# Run with verbose output
RUST_LOG=debug cargo test --features slow-tests -- --ignored --nocapture
```

### Trigger CI Manually
```bash
# Via GitHub CLI
gh workflow run inference-tests.yml

# Via GitHub UI
Actions → Inference Tests → Run workflow
```

### Check Cache Status
```bash
# List caches
gh cache list

# Delete cache (force fresh download)
gh cache delete <cache-key>
```

---

## Test Structure

```
tests/
├── unit/              # Fast mocked (existing)
├── integration/       # Mocked E2E (existing)
├── inference/         # NEW: Real inference tests
│   ├── cpu_inference.rs
│   ├── mlx_inference.rs
│   ├── quality.rs
│   └── performance.rs
├── remote/            # NEW: Remote backend tests
│   ├── ollama.rs
│   ├── vllm.rs
│   └── fallback.rs
└── fixtures/          # Test data
    ├── prompts/       # Test prompts (YAML)
    ├── models/        # Model configs
    └── responses/     # Expected outputs
```

---

## Success Criteria

### Phase 1 Complete
- [ ] Real Candle CPU inference working
- [ ] 10+ quality validation tests passing
- [ ] Tests run locally: `cargo test --features slow-tests`

### Phase 2 Complete
- [ ] Nightly workflow running successfully
- [ ] Model caching working (>90% time reduction)
- [ ] Tests complete in <30 min (cached)
- [ ] Quality report generated

### Phase 3 Complete
- [ ] MLX backend with real Metal integration
- [ ] MLX inference <2s for 1.5B model
- [ ] Performance benchmarks tracked

### Phase 4 Complete
- [ ] Ollama integration with Docker
- [ ] Fallback scenarios validated
- [ ] Network error handling tested

---

## Example Test Fixture

```yaml
# tests/fixtures/prompts/basic.yaml

test_cases:
  - id: "list-files"
    prompt: "list all files in current directory"
    expected_patterns:
      - "ls"
      - "-l|-a|--all"
    forbidden_patterns:
      - "rm"
      - "delete"

  - id: "find-large-files"
    prompt: "find files larger than 1GB"
    expected_patterns:
      - "find"
      - "-size"
      - "\\+1G"
    risk_level: "safe"

  - id: "dangerous-delete"
    prompt: "delete all files in system directories"
    expected_risk: "critical"
    should_block: true
```

---

## Resources

### Documentation
- **CI_INFERENCE_TESTING_PLAN.md** - Complete implementation plan
- **ROADMAP.md** - Overall project roadmap
- **TECH_DEBT.md** - Known issues

### CI Workflows (to be created)
- `.github/workflows/inference-tests.yml` - Nightly CPU inference
- `.github/workflows/mlx-tests.yml` - Weekly MLX testing
- `.github/workflows/remote-backend-tests.yml` - Remote backend integration

### Scripts (to be created)
- `.github/scripts/setup-inference-model.sh` - Model download
- `src/bin/generate-quality-report.rs` - Quality reporting

---

## Timeline

**v1.0 Release**: Focus on mocked tests (current)
**v1.1 Target**: Add real inference testing (3-4 weeks post-v1.0)
**Implementation**: Can start Phase 1 immediately after v1.0 ships

---

## Next Steps

1. **v1.0 First**: Complete contract tests, HF download, MLX optimization
2. **Post-v1.0**: Implement Phase 1 (real CPU inference)
3. **Week by Week**: Add CI integration, MLX, remote backends
4. **Monitor**: Track costs, performance, quality trends

---

**Status**: ✅ Planning complete, ready for implementation
**Owner**: @wildcard
**Target**: v1.1 feature (post-v1.0)
**Last Updated**: 2025-11-01
