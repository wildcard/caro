# Caro LLM Evaluation Harness

## Overview

Comprehensive evaluation framework for testing LLM-generated shell command quality across correctness, safety, and POSIX compliance dimensions.

## Prerequisites

### 1. Caro Binary
Build the caro binary before running evaluations:

```bash
cargo build --release
# Binary will be at: target/release/caro
```

### 2. Shellcheck (for POSIX compliance validation)

**macOS:**
```bash
brew install shellcheck
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt-get update
sudo apt-get install shellcheck
```

**Linux (Fedora/RHEL):**
```bash
sudo dnf install shellcheck
```

**Verify installation:**
```bash
shellcheck --version
# Should show version 0.8.0 or higher
```

### 3. Backend Configuration

Ensure at least one LLM backend is configured:
- MLX backend (Apple Silicon) - via `~/.caro/config.toml`
- Ollama backend (local server) - via `~/.caro/config.toml`
- vLLM backend (remote server) - via `~/.caro/config.toml`

## Quick Start

### Run Evaluation on Sample Dataset

```bash
# Navigate to evaluation directory
cd tests/evaluation

# Run correctness evaluation
cargo test --test test_correctness -- --nocapture

# Run all evaluation tests
cargo test -- --nocapture
```

### Run Full Evaluation Suite

```bash
# From project root
cargo test --package caro-evaluation -- --nocapture

# With specific backend
CARO_BACKEND=mlx cargo test --package caro-evaluation -- --nocapture

# Generate detailed report
cargo test --package caro-evaluation --test test_full_evaluation -- --nocapture
```

## Running Tests

### By Category

```bash
# Correctness validation
cargo test --package caro-evaluation --test test_correctness

# Safety pattern detection
cargo test --package caro-evaluation --test test_safety

# POSIX compliance
cargo test --package caro-evaluation --test test_posix

# Backend comparison
cargo test --package caro-evaluation --test test_backends
```

### All Tests

```bash
cargo test --package caro-evaluation -- --nocapture
```

## Adding Test Cases

Test datasets are located in `datasets/` organized by category:

```
datasets/
├── correctness/      # Command correctness tests
├── safety/           # Safety detection tests
├── posix/            # POSIX compliance tests
└── backend_comparison/  # Multi-backend consistency tests
```

### Test Case Format

Each test case requires:

```json
{
  "id": "unique_test_id",
  "prompt": "natural language input",
  "expected_command": "correct shell command",
  "category": "file_operations",
  "risk_level": "safe",
  "posix_compliant": true,
  "tags": ["basic", "ls"]
}
```

### Adding a New Test Case

1. Choose the appropriate dataset file (e.g., `datasets/correctness/file_operations.json`)
2. Add your test case to the `test_cases` array
3. Ensure the ID is unique
4. Update the dataset metadata (`total_cases`, `categories`, etc.)
5. Validate the JSON format: `jq . datasets/correctness/file_operations.json`

## Viewing Results

Results are written to `results/` with timestamped filenames:

```
results/
├── run_2026-01-08_143052.json  # Machine-readable JSON
└── run_2026-01-08_143052.md    # Human-readable Markdown
```

### View Latest Markdown Report

```bash
cat tests/evaluation/results/$(ls -t tests/evaluation/results/*.md | head -1)
```

### Parse JSON Report

```bash
# Get overall correctness score
jq '.summary_stats.average_correctness' results/run_*.json

# Get safety precision
jq '.summary_stats.safety_accuracy.precision' results/run_*.json

# List all failures
jq '.results[] | select(.correctness_score < 1.0) | {id: .test_case_id, score: .correctness_score}' results/run_*.json
```

## Troubleshooting

### Issue: "Caro binary not found"

**Solution:** Build caro first
```bash
cargo build --release
ls -la target/release/caro  # Verify binary exists
```

### Issue: "Shellcheck not installed"

**Solution:** Install shellcheck
```bash
brew install shellcheck  # macOS
sudo apt-get install shellcheck  # Linux
```

### Issue: "Permission denied when running evaluation"

**Solution:** Make caro binary executable
```bash
chmod +x target/release/caro
```

### Issue: "Tests timeout after 60s"

**Solution:** Increase timeout for slow backends
```bash
cargo test --package caro-evaluation -- --nocapture --test-threads=1 --timeout=300
```

### Issue: "Backend not configured"

**Solution:** Check caro configuration
```bash
cat ~/.caro/config.toml
# Ensure backend section is present and valid
```

## Development

### Project Structure

```
tests/evaluation/
├── Cargo.toml          # Dependencies
├── src/
│   ├── lib.rs          # Module exports
│   ├── dataset.rs      # Test dataset types
│   ├── executor.rs     # CLI invocation
│   ├── evaluator.rs    # Correctness scoring
│   ├── safety_validator.rs  # Safety validation
│   ├── posix_checker.rs     # POSIX compliance
│   └── reporter.rs     # Report generation
├── tests/
│   ├── test_correctness.rs  # Correctness tests
│   ├── test_safety.rs       # Safety tests
│   ├── test_posix.rs        # POSIX tests
│   └── test_backends.rs     # Backend tests
├── datasets/           # Test case collections
└── results/            # Evaluation outputs
```

### Running Tests During Development

```bash
# Watch mode (requires cargo-watch)
cargo watch -x 'test --package caro-evaluation'

# Run specific test
cargo test --package caro-evaluation --test test_correctness test_exact_match -- --nocapture

# Run with verbose output
RUST_LOG=debug cargo test --package caro-evaluation -- --nocapture
```

## Related Documentation

- [Specification](../../kitty-specs/022-issue-135-build/spec.md) - Feature requirements
- [Implementation Plan](../../kitty-specs/022-issue-135-build/plan.md) - Architecture
- [Data Model](../../kitty-specs/022-issue-135-build/data-model.md) - Entity definitions
- [Quickstart Guide](../../kitty-specs/022-issue-135-build/quickstart.md) - Integration scenarios
