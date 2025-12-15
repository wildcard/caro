# MLX Inference Test Suite

Comprehensive MLX testing project for cmdai command generation with structured outputs and safety assessment.

## Setup

```bash
# Quick setup
make setup

# Or manual setup
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## Tests

### 🎬 Presentation Demo (`presentation_demo.py`) ⭐ NEW!
**Interactive live demo** designed for presentations and talks.

```bash
make demo
# or
python presentation_demo.py
```

**Features:**
- Beautiful color-coded output
- Interactive pacing (press Enter between demos)
- Real-time safety assessment display
- 5 carefully chosen scenarios
- Performance metrics summary
- Perfect for live demonstrations

**Use this for:** Conference talks, demos, showing cmdai to others

See **[DEMO_GUIDE.md](DEMO_GUIDE.md)** for full presentation guide!

---

### 1. Simple Inference (`simple_inference.py`)
Basic test showing MLX working with single prompt.

```bash
make run
# or
python simple_inference.py
```

### 2. Structured Inference (`structured_inference.py`)
**Comprehensive test suite** with 12 test cases covering:
- Safe operations (ls, pwd, find, df, ps)
- Moderate risk (cp, tar, large searches)
- High risk (chmod, rm -f)
- Critical/blocked (rm -rf /, mkfs)

Outputs structured JSON with:
- Generated command
- Explanation
- Risk level assessment
- Confirmation requirements

```bash
make run-structured
# or
python structured_inference.py
```

**Output:** `structured_test_results.json` with full analysis

### 3. Batch Inference (`batch_inference.py`)
Performance benchmark with 10 quick prompts for throughput testing.

```bash
make run-batch
# or
python batch_inference.py
```

**Output:** `batch_results.json` with timing statistics

## What It Demonstrates

1. **MLX Integration**: Metal GPU acceleration on Apple Silicon
2. **JSON Parsing**: Structured command output with multiple fallback strategies
3. **Safety Assessment**: Pattern matching for dangerous commands
4. **Risk Levels**: Safe → Moderate → High → Critical classification
5. **Real Test Cases**: Examples from cmdai repository specs and tests

## Test Results

See detailed analysis in:
- **[TEST_RESULTS.md](TEST_RESULTS.md)** - Comprehensive test analysis and recommendations
- **[EXAMPLES.md](EXAMPLES.md)** - Real command generation examples with outputs

### Quick Summary
- ✅ MLX inference working perfectly on Apple Silicon
- ✅ 83% JSON parse success rate with fallbacks
- ✅ Average inference: 0.7s (after warmup)
- ⚠️ Model safety assessment unreliable - need independent validation
- ✅ Command quality good, POSIX-compliant

## Performance

### Benchmark Results
```
First inference:    ~2-4s  (model load + generation)
Subsequent:         ~0.7s  (generation only)
Throughput:         1.36 prompts/sec
Peak memory:        2.3GB
GPU acceleration:   ✅ Metal active
Tokens/sec:         80+ generation speed
```

## Model Size

- **TinyLlama-1.1B**: ~1.1GB download
- First run will download and cache the model
- Subsequent runs use cached model

## Requirements

- macOS with Apple Silicon (M1/M2/M3)
- Python 3.8+
- ~2GB free disk space for model cache

## Expected Output

```
🚀 MLX Simple Inference Test
==================================================
MLX default device: Device(gpu, 0)
Metal available: True

Loading model: TinyLlama/TinyLlama-1.1B-Chat-v1.0
✅ Model loaded successfully!

Prompt: Convert this to a shell command: list all files in current directory

Generating response...
--------------------------------------------------
[generated text appears here]
--------------------------------------------------

✅ Inference complete!
```

## Performance

On Apple Silicon, you should see:
- Model load: ~2-5 seconds (after initial download)
- Inference: <2 seconds for 100 tokens
- GPU memory usage: ~1.5GB

## Troubleshooting

If you get import errors:
```bash
pip install --upgrade mlx mlx-lm
```

If Metal is not available:
- Ensure you're on Apple Silicon Mac
- Check macOS is updated to latest version
