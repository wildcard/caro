# cmdai Promptfoo Evaluation Framework

> Isolated evaluation sub-project for testing cmdai command generation quality using [promptfoo](https://www.promptfoo.dev/)

This directory contains a standalone Node.js-based evaluation framework that complements the Rust-based internal evaluation system. It provides:

- **Black-box testing** of the cmdai binary
- **Prompt engineering** experiments
- **Provider comparison** across different LLMs
- **Automated dataset conversion** from Rust YAML format
- **Web-based result viewing**

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Directory Structure](#directory-structure)
- [Evaluation Types](#evaluation-types)
- [Configuration](#configuration)
- [Usage](#usage)
- [Providers](#providers)
- [Dataset Conversion](#dataset-conversion)
- [Viewing Results](#viewing-results)
- [Integration with CI/CD](#integration-with-cicd)
- [Troubleshooting](#troubleshooting)

## Prerequisites

- **Node.js** 18+ and npm 9+
- **cmdai binary** built at `../../target/release/cmdai`
- *Optional*: Ollama running locally on port 11434
- *Optional*: Anthropic API key for Claude testing

## Quick Start

```bash
# 1. Install dependencies
cd tools/promptfoo
npm install

# 2. Build cmdai (if not already built)
cd ../..
cargo build --release
cd tools/promptfoo

# 3. Run a quick evaluation
npm run eval:cmdai

# 4. View results in web UI
npm run view
```

## Directory Structure

```
tools/promptfoo/
├── package.json                    # Node.js dependencies
├── README.md                       # This file
├── .gitignore                      # Node-specific ignores
│
├── configs/                        # Evaluation configurations
│   ├── cmdai-binary.yaml          # Test cmdai CLI directly
│   ├── prompt-variations.yaml     # Test prompt engineering
│   └── providers-comparison.yaml  # Compare LLM providers
│
├── prompts/                        # Prompt templates
│   └── system-prompts.txt         # Curated system prompt variations
│
├── test-cases/                     # Test datasets
│   ├── basic-commands.yaml        # Manual test cases
│   └── converted/                 # Auto-generated from Rust evals
│
├── providers/                      # Custom provider implementations
│   ├── cmdai-provider.js          # Exec wrapper for cmdai binary
│   └── candle-hf-provider.js      # (Future) HF Candle integration
│
├── scripts/                        # Utility scripts
│   ├── convert-dataset.js         # Convert Rust YAML → promptfoo
│   └── run-all-evals.sh          # Run all evaluations
│
└── outputs/                        # Evaluation results (gitignored)
    ├── cmdai-binary-results.json
    ├── prompt-variations-results.json
    └── providers-comparison-results.json
```

## Evaluation Types

### 1. cmdai Binary Evaluation

Tests the cmdai CLI as a black-box tool.

```bash
npm run eval:cmdai
```

**What it tests:**
- Command generation accuracy
- Safety validation
- Latency performance
- Output format consistency

**Configuration:** `configs/cmdai-binary.yaml`

### 2. Prompt Variations

Compares different prompt engineering approaches.

```bash
npm run eval:prompts
```

**What it tests:**
- Strict JSON vs conversational formats
- Safety-first vs minimal prompts
- Chain-of-thought reasoning
- Structured output formats

**Configuration:** `configs/prompt-variations.yaml`

### 3. Provider Comparison

Compares cmdai against other LLM providers.

```bash
npm run eval:providers
```

**What it compares:**
- cmdai binary (custom provider)
- Ollama (qwen2.5-coder)
- Anthropic Claude 3.5 Sonnet

**Metrics:**
- Accuracy (exact and semantic matches)
- Safety (dangerous command prevention)
- Latency (response time)
- Cost (API pricing)

**Configuration:** `configs/providers-comparison.yaml`

## Configuration

### Environment Variables

Create a `.env` file for API keys:

```bash
# Anthropic API key (optional)
ANTHROPIC_API_KEY=your_key_here

# Ollama base URL (default: http://localhost:11434)
OLLAMA_API_BASE_URL=http://localhost:11434
```

### Customizing Evaluations

Edit the YAML config files in `configs/` to:

- Add new test cases
- Adjust assertion thresholds
- Change provider settings
- Modify timeout values

Example test case:

```yaml
tests:
  - vars:
      prompt: "find all PDF files larger than 10MB"
    assert:
      - type: contains
        value: "find"
      - type: contains
        value: ".pdf"
      - type: latency
        threshold: 5000  # 5 seconds
```

## Usage

### Run Individual Evaluations

```bash
# Test cmdai binary
npm run eval:cmdai

# Test prompt variations (requires Ollama or Claude)
npm run eval:prompts

# Compare providers
npm run eval:providers
```

### Run All Evaluations

```bash
# Using npm script
npm run eval:all

# Using bash script with options
./scripts/run-all-evals.sh
./scripts/run-all-evals.sh --view              # Open web UI after
./scripts/run-all-evals.sh --only-cmdai        # Only cmdai evaluation
```

### View Results

```bash
# Open web UI for latest results
npm run view:latest

# Open web UI for all results
npm run view

# Or use promptfoo directly
npx promptfoo view outputs/cmdai-binary-results.json
```

### Clean Up

```bash
npm run clean  # Remove all outputs and converted datasets
```

## Providers

### 1. cmdai Binary Provider

**Implementation:** `providers/cmdai-provider.js`

Custom JavaScript provider that:
- Spawns cmdai binary as child process
- Captures stdout/stderr
- Extracts generated command from output
- Handles timeouts and errors

**Configuration:**

```yaml
providers:
  - id: file://providers/cmdai-provider.js
    config:
      binaryPath: ../../../target/release/cmdai
      shell: bash
      timeout: 10000  # milliseconds
```

### 2. Ollama Local Provider

**Built-in promptfoo provider**

Requires Ollama running locally:

```bash
# Start Ollama
ollama serve

# Pull a code model
ollama pull qwen2.5-coder:latest
```

**Configuration:**

```yaml
providers:
  - id: ollama:qwen2.5-coder:latest
    config:
      apiBaseUrl: http://localhost:11434
      temperature: 0.0
```

### 3. Anthropic Claude

**Built-in promptfoo provider**

Requires `ANTHROPIC_API_KEY` environment variable.

**Configuration:**

```yaml
providers:
  - id: anthropic:claude-3-5-sonnet-20241022
    config:
      temperature: 0.0
      max_tokens: 1024
```

### 4. Hugging Face Candle (Future)

For Qwen3 models via Rust HF Candle, you can:

1. **Option A**: Use Ollama to serve HF models
   ```bash
   ollama pull hf.co/username/model
   ```

2. **Option B**: Create HTTP wrapper around Candle (future implementation)
   - Implement `providers/candle-hf-provider.js`
   - Create Rust HTTP server using Candle for inference
   - Serve OpenAI-compatible API

## Dataset Conversion

Convert Rust YAML test datasets to promptfoo format:

```bash
# Convert all datasets from eval-core
npm run convert

# Or use the script directly
node scripts/convert-dataset.js

# Convert specific file
node scripts/convert-dataset.js \
  ../../crates/eval-core/datasets/basic.yaml \
  test-cases/converted/basic.yaml
```

**What it does:**

- Reads `TestCase` from Rust YAML
- Converts to promptfoo test format
- Maps assertions (exact match, semantic match, safety)
- Preserves metadata (difficulty, shell, safety level)
- Outputs to `test-cases/converted/`

**Rust format:**

```yaml
test_cases:
  - id: "find-pdf-files"
    input: "find all PDF files"
    expected_commands:
      - "find . -name '*.pdf'"
    shell: bash
    difficulty: easy
    safety_level: safe
```

**Promptfoo format:**

```yaml
tests:
  - description: "find-pdf-files"
    vars:
      prompt: "find all PDF files"
      shell: bash
    assert:
      - type: contains
        value: "find . -name '*.pdf'"
      - type: latency
        threshold: 10000
    metadata:
      difficulty: easy
```

## Viewing Results

### Web UI

The most convenient way to explore results:

```bash
npm run view
```

Opens an interactive dashboard showing:
- Pass/fail rates per test
- Comparison matrices
- Latency graphs
- Cost analysis
- Assertion details

### JSON Output

Results are saved to `outputs/`:

```bash
cat outputs/cmdai-binary-results.json | jq
```

### Export Formats

Promptfoo supports exporting to:
- JSON (default)
- CSV
- HTML
- Markdown

```bash
npx promptfoo eval -c configs/cmdai-binary.yaml -o outputs/results.csv
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Promptfoo Evaluations

on:
  pull_request:
  push:
    branches: [main]

jobs:
  evaluate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build cmdai
        run: cargo build --release

      - name: Install promptfoo dependencies
        run: |
          cd tools/promptfoo
          npm install

      - name: Run evaluations
        run: |
          cd tools/promptfoo
          ./scripts/run-all-evals.sh --only-cmdai

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: promptfoo-results
          path: tools/promptfoo/outputs/
```

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
cd tools/promptfoo
npm run eval:cmdai --silent
```

## Troubleshooting

### cmdai binary not found

```bash
# Build the binary first
cd /workspaces/cmdai
cargo build --release
```

### Ollama not responding

```bash
# Check if Ollama is running
curl http://localhost:11434/api/version

# Start Ollama
ollama serve

# Pull required model
ollama pull qwen2.5-coder:latest
```

### Anthropic API errors

```bash
# Set API key
export ANTHROPIC_API_KEY=your_key_here

# Or create .env file
echo "ANTHROPIC_API_KEY=your_key_here" > tools/promptfoo/.env
```

### Timeout errors

Increase timeout in config:

```yaml
providers:
  - id: file://providers/cmdai-provider.js
    config:
      timeout: 30000  # 30 seconds
```

### Permission denied on scripts

```bash
chmod +x scripts/*.sh
chmod +x scripts/*.js
```

### Node.js version issues

```bash
# Check version
node --version  # Should be >= 18

# Update Node.js via nvm
nvm install 18
nvm use 18
```

## Advanced Usage

### Custom Assertions

Add JavaScript assertions for complex logic:

```yaml
assert:
  - type: javascript
    value: |
      const isValid = output.includes('find') &&
                      output.includes('.pdf') &&
                      !output.includes('rm');
      return isValid;
```

### Cost Tracking

Track API costs:

```yaml
assert:
  - type: cost
    threshold: 0.01  # Max $0.01 per request
```

### Latency Profiling

Measure response times:

```yaml
defaultTest:
  assert:
    - type: latency
      threshold: 5000  # 5 seconds
```

### Matrix Testing

Test multiple variables:

```yaml
tests:
  - vars:
      prompt: "{{operation}} {{filetype}} files"
    matrix:
      - operation: ["find", "list", "count"]
        filetype: ["PDF", "image", "log"]
```

## Contributing

To add new evaluation scenarios:

1. Create a new config in `configs/`
2. Add test cases to `test-cases/`
3. Update npm scripts in `package.json`
4. Document the new evaluation type

## Resources

- [Promptfoo Documentation](https://www.promptfoo.dev/docs/)
- [cmdai Project README](../../README.md)
- [Internal Evaluation System](../../crates/eval-core/)

## License

This sub-project follows the same AGPL-3.0 license as the main cmdai project.
