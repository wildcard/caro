# Debugging Workflows for AI Development

> **Purpose**: Structured troubleshooting workflows for common issues. Follow these step-by-step to resolve problems systematically.

## Quick Debug Reference

| Symptom | Start With | Section |
|---------|------------|---------|
| Compilation fails | [Workflow 1](#workflow-1-compilation-errors) | Compilation Errors |
| Tests fail | [Workflow 2](#workflow-2-test-failures) | Test Failures |
| Backend unavailable | [Workflow 3](#workflow-3-backend-issues) | Backend Issues |
| Command blocked | [Workflow 4](#workflow-4-safety-validation) | Safety Validation |
| Wrong command generated | [Workflow 5](#workflow-5-generation-quality) | Generation Quality |
| Performance issues | [Workflow 6](#workflow-6-performance) | Performance |

---

## Workflow 1: Compilation Errors

### Step 1: Read the Full Error

```bash
# Get complete error output
cargo build 2>&1 | head -100
```

Key information to extract:
- **Error code** (e.g., E0277, E0382)
- **File and line number**
- **Expected vs found types**

### Step 2: Identify Error Category

| Error Code | Category | Quick Fix |
|------------|----------|-----------|
| E0277 | Trait bounds | Add required trait (Send, Sync, Clone) |
| E0382 | Borrow after move | Clone before use or use reference |
| E0597 | Lifetime | Extend scope or use owned type |
| E0433 | Import | Check Cargo.toml and use statements |

### Step 3: Apply Pattern Fix

```bash
# Check if it's an async trait issue
grep -n "async fn" src/**/*.rs | head -20

# Check for missing derives
grep -n "struct.*{" src/**/*.rs | head -20
```

### Step 4: Verify Fix

```bash
cargo check
cargo test --lib
```

### Common Fixes Checklist

- [ ] Added `#[async_trait]` for async traits
- [ ] Added `Send + Sync` bounds for concurrent types
- [ ] Added `Clone` derive for types that need copying
- [ ] Used `Arc<Mutex<T>>` for shared mutable state
- [ ] Added `.context()` for error propagation

---

## Workflow 2: Test Failures

### Step 1: Run Failing Test in Isolation

```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name
```

### Step 2: Identify Failure Type

| Failure | Diagnostic | Action |
|---------|------------|--------|
| `assertion failed` | Check expected vs actual | Verify test logic |
| `called Result::unwrap()` | Missing error handling | Add proper error handling |
| `timeout` | Async deadlock or slow | Check async code flow |
| `thread panicked` | Panic in code | Find panic source |

### Step 3: Debug with Logging

```rust
// Add temporary debug logging
#[test]
fn failing_test() {
    eprintln!("Input: {:?}", input);
    let result = operation(input);
    eprintln!("Result: {:?}", result);
    assert!(result.is_ok());
}
```

### Step 4: Check Test Environment

```bash
# Verify test fixtures exist
ls tests/fixtures/

# Check environment variables
env | grep CARO

# Run in clean environment
cargo clean && cargo test
```

### Step 5: Verify Fix

```bash
# Run full test suite
cargo test

# Run with coverage (if available)
cargo tarpaulin --out Html
```

---

## Workflow 3: Backend Issues

### Step 1: Check Backend Status

```bash
# Ollama
curl http://localhost:11434/api/tags

# vLLM
curl http://localhost:8000/v1/models

# Embedded (check binary)
caro --version
```

### Step 2: Verify Configuration

```bash
# Check config file
cat ~/.config/caro/config.toml

# Check environment overrides
env | grep CARO
```

### Step 3: Test Backend Directly

```bash
# Test Ollama
curl http://localhost:11434/api/generate \
  -d '{"model": "codellama", "prompt": "ls"}'

# Run caro with verbose
RUST_LOG=debug caro "list files"
```

### Step 4: Check Fallback Chain

```bash
# Verify fallback order in config
grep -A5 "backend" ~/.config/caro/config.toml

# Test each backend individually
caro --backend embedded "test"
caro --backend ollama "test"
```

### Step 5: Common Backend Fixes

| Issue | Fix |
|-------|-----|
| Ollama not running | `ollama serve` |
| Wrong model | Update `model_name` in config |
| Timeout | Increase `timeout_secs` |
| Connection refused | Check `base_url` |

---

## Workflow 4: Safety Validation

### Step 1: Understand the Block

```bash
# Run with verbose to see matched pattern
caro --verbose "your command"

# Check the safety result
caro --output json "your command" | jq '.safety'
```

### Step 2: Verify Pattern Match

```bash
# Find the matching pattern in source
grep -n "pattern_string" src/safety/*.rs
```

### Step 3: Determine if False Positive

Questions to answer:
1. Is the command actually dangerous?
2. Does the pattern match too broadly?
3. Is there missing context?

### Step 4: Adjust Safety

For legitimate commands:
```bash
# Use permissive mode
caro --safety permissive "command"

# Or auto-confirm
caro -y "command"
```

For pattern issues, file a bug with:
- Command that triggered false positive
- Why it's safe
- Suggested pattern refinement

### Step 5: Testing Safety Changes

```bash
# Run safety test suite
cargo test --package caro --lib safety::tests

# Run property tests
cargo test safety_never_false_negative
```

---

## Workflow 5: Generation Quality

### Step 1: Analyze the Generated Command

```bash
# Get full generation details
caro --verbose --output json "your prompt" | jq '.'
```

Check:
- Platform detection (OS, shell)
- Context used
- Model response

### Step 2: Verify Platform Detection

```bash
# Check detected context
caro assess --export json | jq '.platform'
```

### Step 3: Review the Prompt

Common prompt issues:
- Too vague ("do the thing")
- Platform-specific when shouldn't be
- Missing constraints

### Step 4: Test with Explicit Context

```bash
# Force specific shell
caro --shell bash "command"

# Force safety level
caro --safety strict "command"
```

### Step 5: Report Quality Issues

Include in bug report:
- Input prompt
- Generated command
- Expected command
- Platform details
- Why the difference matters

---

## Workflow 6: Performance

### Step 1: Measure Baseline

```bash
# Time command generation
time caro "list files"

# With detailed timing
caro --verbose "list files" 2>&1 | grep -i time
```

### Step 2: Profile Startup

```bash
# Check binary size
ls -lh target/release/caro

# Check startup time
hyperfine 'caro --help'
```

### Step 3: Identify Bottlenecks

| Phase | Expected | Check |
|-------|----------|-------|
| Startup | < 100ms | Config loading |
| Model load | < 500ms | Cache hit/miss |
| Inference | < 2s | Backend performance |
| Safety | < 50ms | Pattern count |

### Step 4: Check Model Caching

```bash
# Check cache location
ls -la ~/.cache/caro/models/

# Verify cache hit
RUST_LOG=debug caro "test" 2>&1 | grep -i cache
```

### Step 5: Optimize

| Bottleneck | Optimization |
|------------|--------------|
| Slow startup | Lazy load backends |
| Model loading | Verify cache working |
| Inference | Use smaller model |
| Safety | Pre-compile patterns (already done) |

---

## Debugging Commands Reference

### Build & Check

```bash
# Fast check without building
cargo check

# Build with all warnings
cargo build 2>&1 | tee build.log

# Release build with debug info
cargo build --release

# Check specific feature
cargo check --features embedded-mlx
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Run tests matching pattern
cargo test safety_
```

### Debugging Output

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- "prompt"

# Enable trace logging
RUST_LOG=trace cargo run -- "prompt"

# Log specific module
RUST_LOG=caro::backends=debug cargo run -- "prompt"

# With backtrace
RUST_BACKTRACE=1 cargo run -- "prompt"

# Full backtrace
RUST_BACKTRACE=full cargo run -- "prompt"
```

### Inspection

```bash
# Check dependencies
cargo tree

# Check for outdated deps
cargo outdated

# Security audit
cargo audit

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

---

## Debugging Checklist

Before asking for help, verify:

- [ ] Error message fully read and understood
- [ ] Searched for error in ERROR_SOLUTIONS.md
- [ ] Ran with RUST_LOG=debug
- [ ] Checked if it's a known issue in GitHub
- [ ] Created minimal reproducer
- [ ] Documented platform and version

## Getting Help

1. **Check existing docs**:
   - MEMORY.md
   - ERROR_SOLUTIONS.md
   - CANONICAL_PATTERNS.md

2. **Search issues**: `gh issue list --search "error message"`

3. **Create issue with**:
   - Platform details
   - Caro version
   - Minimal reproducer
   - Expected vs actual behavior
   - Debug logs

---

*Systematic debugging beats random changes every time.*
*Last updated: 2026-01-12*
