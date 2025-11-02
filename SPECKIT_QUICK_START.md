# Speckit Quick Start - Command Execution Order

**Quick Reference**: Minimal command sequence to implement cmdai roadmap
**See**: [SPECKIT_EXECUTION_PLAN.md](SPECKIT_EXECUTION_PLAN.md) for detailed specifications

---

## Setup (One-Time)

```bash
# Optional: Define project constitution
/constitution
```

---

## v1.0 Critical Path

### 1. Contract Test Alignment (4-8 hours)

```bash
/specify
```
**Input**: "Fix contract test alignment - ~35 compilation errors in tests/contract/config_contract.rs and tests/contract/logging_contract.rs. Update test contracts to match implemented API signatures. Success: cargo test --all-features passes."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

---

### 2. HF Model Download (16-24 hours)

```bash
/specify
```
**Input**: "Implement Hugging Face model download in src/cache/hf_download.rs. Features: HTTP client with reqwest, Range request resume, SHA256 validation, progress bar with indicatif, retry logic. Integrate with src/backends/embedded/mod.rs for auto-download. Target: Qwen2.5-Coder-1.5B-Instruct-GGUF."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

---

### 3. MLX Backend Optimization (8-16 hours)

```bash
/specify
```
**Input**: "Implement real MLX backend with Metal GPU acceleration. Create C++ wrapper (mlx_wrapper.cpp), Rust FFI bridge (mlx_bridge.rs), update mlx.rs with real inference. Use cxx crate, link Metal frameworks. Target: <2s inference on Apple Silicon. Platform: macOS aarch64 only."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

---

### 4. Binary Distribution (8-12 hours)

```bash
/specify
```
**Input**: "Create binary distribution system. Update .github/workflows/release.yml for multi-platform builds (Linux x64/ARM64, macOS x64/ARM64, Windows x64). Create install.sh script. Optimize binary size <50MB. Support Homebrew formula. One-liner install support."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

**🎉 v1.0 Release Candidate**

---

## v1.1 Real Inference Testing

### 5. Real Candle CPU Backend (16-24 hours)

```bash
/specify
```
**Input**: "Replace simulated CPU inference with real Candle framework integration in src/backends/embedded/cpu.rs. Load GGUF models, tokenize with Qwen tokenizer, execute forward pass, decode tokens. Create tests/inference/cpu_inference.rs with #[ignore] and slow-tests feature flag. Target: <5s inference."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

---

### 6. Inference Test Framework (12-16 hours)

```bash
/specify
```
**Input**: "Create inference test framework in tests/inference/. Implement quality.rs (fixture loading, pattern validation), performance.rs (latency measurement), mlx_inference.rs (GPU tests). Load YAML fixtures from tests/fixtures/prompts/. Generate markdown quality reports. Feature flags: slow-tests."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

---

### 7. Remote Backend Integration (16-20 hours)

```bash
/specify
```
**Input**: "Implement real HTTP clients for Ollama and vLLM in src/backends/remote/. Ollama: /api/generate endpoint. vLLM: /v1/chat/completions (OpenAI-compatible). Include fallback to embedded backend, retry logic, timeout handling. Create tests/remote/ with Docker integration tests."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

---

### 8. Quality Report Generator (8-12 hours)

```bash
/specify
```
**Input**: "Create src/bin/generate-quality-report.rs binary. Parse test result JSON, calculate pass rates and latency stats, generate markdown reports. CLI with clap, support --input and --output flags. Used in CI workflows for quality tracking."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

**🎉 v1.1 Complete**

---

## Release Preparation

### 9. Documentation Update (4-6 hours)

```bash
/specify
```
**Input**: "Update all documentation for v1.0 release. Update README.md (installation, usage, features), create QUICKSTART.md, create TROUBLESHOOTING.md, finalize CHANGELOG.md with v1.0 release notes. Include examples for all backends."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

---

### 10. v1.0 Release Process (2-4 hours)

```bash
/specify
```
**Input**: "Execute v1.0 release. Version bump in Cargo.toml to 1.0.0, finalize CHANGELOG, create v1.0.0 git tag, create GitHub Release with binaries, publish to crates.io, update Homebrew formula, prepare announcement."

```bash
/clarify
/plan
/tasks
/analyze
/implement
```

**🚀 v1.0 Released**

---

## Command Pattern

For each feature:

```bash
# 1. Specify
/specify
# → Paste natural language description
# → Review generated spec.md

# 2. Clarify
/clarify
# → Answer clarifying questions
# → Verify answers encoded in spec

# 3. Plan
/plan
# → Review generated plan.md
# → Validate architecture

# 4. Tasks
/tasks
# → Review generated tasks.md
# → Check dependency order

# 5. Analyze
/analyze
# → Review consistency analysis
# → Resolve any issues

# 6. Implement
/implement
# → Execute tasks
# → Monitor progress
# → Review generated code

# 7. Verify
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check

# 8. Commit
git add .
git commit -m "feat: implement [feature-name]"
git push
```

---

## Checklist Progress Tracker

### v1.0 Features

- [ ] 1. Contract Test Alignment
  - [ ] /specify
  - [ ] /clarify
  - [ ] /plan
  - [ ] /tasks
  - [ ] /analyze
  - [ ] /implement
  - [ ] Tests passing

- [ ] 2. HF Model Download
  - [ ] /specify
  - [ ] /clarify
  - [ ] /plan
  - [ ] /tasks
  - [ ] /analyze
  - [ ] /implement
  - [ ] Tests passing

- [ ] 3. MLX Backend Optimization
  - [ ] /specify
  - [ ] /clarify
  - [ ] /plan
  - [ ] /tasks
  - [ ] /analyze
  - [ ] /implement
  - [ ] Tests passing

- [ ] 4. Binary Distribution
  - [ ] /specify
  - [ ] /clarify
  - [ ] /plan
  - [ ] /tasks
  - [ ] /analyze
  - [ ] /implement
  - [ ] Tests passing

### v1.1 Features

- [ ] 5. Real Candle CPU Backend
  - [ ] /specify
  - [ ] /clarify
  - [ ] /plan
  - [ ] /tasks
  - [ ] /analyze
  - [ ] /implement
  - [ ] Tests passing

- [ ] 6. Inference Test Framework
  - [ ] /specify
  - [ ] /clarify
  - [ ] /plan
  - [ ] /tasks
  - [ ] /analyze
  - [ ] /implement
  - [ ] Tests passing

- [ ] 7. Remote Backend Integration
  - [ ] /specify
  - [ ] /clarify
  - [ ] /plan
  - [ ] /tasks
  - [ ] /analyze
  - [ ] /implement
  - [ ] Tests passing

- [ ] 8. Quality Report Generator
  - [ ] /specify
  - [ ] /clarify
  - [ ] /plan
  - [ ] /tasks
  - [ ] /analyze
  - [ ] /implement
  - [ ] Tests passing

### Release

- [ ] 9. Documentation Update
- [ ] 10. v1.0 Release Process

---

## Timeline

**Week 1**: Features 1-2
**Week 2**: Features 2-3
**Week 3**: Features 3-4 → **v1.0 RC**
**Week 4**: Feature 5
**Week 5**: Feature 6
**Week 6**: Feature 7
**Week 7**: Feature 8 → **v1.1 Complete**
**Week 8**: Features 9-10 → **v1.0 Release**

---

## Tips

- **Be detailed in /specify** - More detail = better implementation
- **Answer all /clarify questions** - Thorough answers prevent issues
- **Review /plan carefully** - Architecture decisions are hard to change
- **Check /tasks dependencies** - Order matters
- **Fix /analyze issues** - Don't skip consistency problems
- **Monitor /implement** - Watch for errors during execution
- **Test frequently** - Run tests after each feature
- **Commit often** - Small commits are easier to debug

---

## Troubleshooting

**Issue**: /specify doesn't generate spec.md
**Solution**: Provide more detailed natural language description

**Issue**: /clarify has too many questions
**Solution**: Update spec with more specific requirements

**Issue**: /tasks shows circular dependencies
**Solution**: Review plan, break down into smaller components

**Issue**: /implement fails with errors
**Solution**: Check task dependencies, run tests, review error messages

**Issue**: /analyze reports inconsistencies
**Solution**: Resolve conflicts in spec/plan before implementing

---

## Next Command

Start with Feature 1:

```bash
/specify
```

Then paste:
```
Fix contract test alignment in cmdai test suite. Currently ~35 compilation
errors exist due to API signature mismatches between test contracts and
implemented APIs in tests/contract/config_contract.rs and
tests/contract/logging_contract.rs. Update test contracts to match implemented
API signatures without breaking production code. Success criteria:
cargo test --all-features succeeds without compilation errors.
```

---

**Created**: 2025-11-01
**Status**: Ready to Execute
**Estimated Time**: 8 weeks to v1.1
