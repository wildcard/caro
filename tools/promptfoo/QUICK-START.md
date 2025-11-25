# Promptfoo Quick Start Guide

## 1-Minute Setup

```bash
cd tools/promptfoo
npm install
```

## Run Your First Evaluation

```bash
# Make sure cmdai is built
cd ../..
cargo build --release
cd tools/promptfoo

# Run evaluation
npm run eval:cmdai

# View results
npm run view
```

## Common Commands

```bash
# Evaluate cmdai binary
npm run eval:cmdai

# Test prompt variations (requires Ollama or Claude API)
npm run eval:prompts

# Compare providers
npm run eval:providers

# Run all evaluations
npm run eval:all

# Convert Rust datasets to promptfoo format
npm run convert

# View results in web UI
npm run view

# Clean outputs
npm run clean
```

## Required Setup for Each Provider

### cmdai Binary (Default)
✅ No additional setup needed (uses compiled binary)

### Ollama
```bash
# Install Ollama: https://ollama.ai
ollama serve
ollama pull qwen2.5-coder:latest
```

### Anthropic Claude
```bash
# Set API key
export ANTHROPIC_API_KEY=your_key_here
# Or create .env file
echo "ANTHROPIC_API_KEY=your_key_here" > .env
```

## Directory Quick Reference

- `configs/` - Evaluation configurations (YAML)
- `test-cases/` - Test datasets
- `providers/` - Custom provider code
- `scripts/` - Utility scripts
- `outputs/` - Results (gitignored)

## Troubleshooting

**Binary not found?**
```bash
cd ../.. && cargo build --release
```

**Ollama not responding?**
```bash
ollama serve
```

**Want to customize tests?**
Edit `configs/cmdai-binary.yaml`

## Next Steps

1. Read the [full README](README.md)
2. Customize configurations in `configs/`
3. Add your own test cases to `test-cases/`
4. Integrate into CI/CD

## Help

```bash
npm run help
./scripts/run-all-evals.sh --help
```
